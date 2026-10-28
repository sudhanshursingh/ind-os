//! Recents / Multitasking Carousel Surface (SP-130, REQ-APP-COMMON-016)
//!
//! Bible §8.18.2, §8.20, §9.9.1 REQ-APP-COMMON-016
//! Invariants:
//! - INV-RECENTS-001: Bottom edge-swipe-up gesture triggers entry into recents carousel mode.
//! - INV-RECENTS-002: Live compositor thumbnail metadata tracked per task without heap allocation.
//! - INV-RECENTS-003: Swipe-to-terminate dispatches lifecycle termination to target PID and frees slot.
//! - INV-RECENTS-004: Pinned tasks are protected from clear-all batch termination.
//! - INV-RECENTS-005: Aggregate resident memory accounting reported across active tasks.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user control over running tasks)

/// Maximum concurrent active app tasks tracked in the recents carousel.
pub const MAX_RECENTS_TASKS: usize = 16;

/// Capability required to manage tasks and terminate apps via the Recents surface.
pub const CAP_RECENTS_MANAGE: u64 = 0x0010_0000_0000;

/// Live compositor thumbnail descriptor (INV-RECENTS-002).
/// References zero-copy damage swapchain buffers created by the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThumbnailDescriptor {
    pub buffer_id: u32,
    pub width: u16,
    pub height: u16,
    pub stride: u16,
    pub checksum: u32,
}

impl ThumbnailDescriptor {
    pub const fn empty() -> Self {
        Self {
            buffer_id: 0,
            width: 0,
            height: 0,
            stride: 0,
            checksum: 0,
        }
    }

    pub const fn new(buffer_id: u32, width: u16, height: u16, stride: u16, checksum: u32) -> Self {
        Self {
            buffer_id,
            width,
            height,
            stride,
            checksum,
        }
    }
}

/// A single multitasking task card displayed in the recents carousel.
#[derive(Debug, Clone, Copy)]
pub struct RecentsTask {
    pub task_id: u32,
    pub pid: u32,
    pub app_id: &'static str,
    pub title: &'static str,
    pub thumbnail: ThumbnailDescriptor,
    pub memory_kb: usize,
    pub is_pinned: bool,
    pub last_active_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecentsError {
    MissingManageCapability,
    TaskTableFull,
    TaskNotFound,
    TaskIsPinned,
    CarouselEmpty,
    InvalidGesture,
}

/// The Recents / Multitasking Carousel surface engine (INV-RECENTS-001..005).
pub struct RecentsCarouselEngine {
    tasks: [Option<RecentsTask>; MAX_RECENTS_TASKS],
    task_count: usize,
    active_index: usize,
    is_open: bool,
    next_task_id: u32,
}

impl RecentsCarouselEngine {
    pub const fn new() -> Self {
        Self {
            tasks: [None; MAX_RECENTS_TASKS],
            task_count: 0,
            active_index: 0,
            is_open: false,
            next_task_id: 1,
        }
    }

    pub fn task_count(&self) -> usize {
        self.task_count
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    // ── Gesture & Carousel State (INV-RECENTS-001) ────────────────────────────

    /// Processes bottom edge-swipe-up gesture to open Recents carousel.
    /// Bottom nav area is y >= 2332 (panel height 2460 - nav bar 128).
    /// Upward velocity must be > 100 px/s to distinguish from a tap.
    pub fn handle_bottom_edge_gesture(
        &mut self,
        start_y: u32,
        end_y: u32,
        velocity_px_per_s: u32,
    ) -> Result<bool, RecentsError> {
        if start_y < 2332 {
            return Err(RecentsError::InvalidGesture);
        }
        if end_y >= start_y || velocity_px_per_s < 100 {
            return Err(RecentsError::InvalidGesture);
        }

        self.is_open = true;
        // Position at the most recently active task (last registered slot)
        if self.task_count > 0 {
            self.active_index = self.task_count - 1;
        }
        Ok(true)
    }

    pub fn close_carousel(&mut self) {
        self.is_open = false;
    }

    // ── Carousel Navigation ───────────────────────────────────────────────────

    pub fn scroll_next(&mut self) -> Option<usize> {
        if self.task_count == 0 {
            return None;
        }
        if self.active_index + 1 < self.task_count {
            self.active_index += 1;
        }
        Some(self.active_index)
    }

    pub fn scroll_prev(&mut self) -> Option<usize> {
        if self.task_count == 0 {
            return None;
        }
        if self.active_index > 0 {
            self.active_index -= 1;
        }
        Some(self.active_index)
    }

    /// Selects current card to bring back to foreground and closes recents.
    pub fn select_current_task(&mut self) -> Option<(u32, &'static str)> {
        if !self.is_open || self.task_count == 0 {
            return None;
        }
        let task = self.tasks[self.active_index].as_ref()?;
        let res = (task.pid, task.app_id);
        self.is_open = false;
        Some(res)
    }

    // ── Task Management (INV-RECENTS-002..005) ────────────────────────────────

    /// Register or update an app task into the recents carousel.
    pub fn register_task(
        &mut self,
        pid: u32,
        app_id: &'static str,
        title: &'static str,
        thumbnail: ThumbnailDescriptor,
        memory_kb: usize,
        caller_caps: u64,
    ) -> Result<u32, RecentsError> {
        if (caller_caps & CAP_RECENTS_MANAGE) == 0 {
            return Err(RecentsError::MissingManageCapability);
        }

        // Check if task already exists for this PID -> update thumbnail & memory
        for slot in self.tasks.iter_mut() {
            if let Some(ref mut t) = slot {
                if t.pid == pid {
                    t.thumbnail = thumbnail;
                    t.memory_kb = memory_kb;
                    t.title = title;
                    return Ok(t.task_id);
                }
            }
        }

        // Add new task
        let task_id = self.next_task_id;
        self.next_task_id += 1;

        for slot in self.tasks.iter_mut() {
            if slot.is_none() {
                *slot = Some(RecentsTask {
                    task_id,
                    pid,
                    app_id,
                    title,
                    thumbnail,
                    memory_kb,
                    is_pinned: false,
                    last_active_tick: 0,
                });
                self.task_count += 1;
                return Ok(task_id);
            }
        }

        Err(RecentsError::TaskTableFull)
    }

    /// Update live thumbnail descriptor for an existing task (INV-RECENTS-002).
    pub fn update_thumbnail(
        &mut self,
        task_id: u32,
        thumbnail: ThumbnailDescriptor,
        caller_caps: u64,
    ) -> Result<(), RecentsError> {
        if (caller_caps & CAP_RECENTS_MANAGE) == 0 {
            return Err(RecentsError::MissingManageCapability);
        }

        for slot in self.tasks.iter_mut() {
            if let Some(ref mut t) = slot {
                if t.task_id == task_id {
                    t.thumbnail = thumbnail;
                    return Ok(());
                }
            }
        }
        Err(RecentsError::TaskNotFound)
    }

    /// Pin or unpin a task to protect it from clear-all (INV-RECENTS-004).
    pub fn set_pinned(&mut self, task_id: u32, is_pinned: bool, caller_caps: u64) -> Result<(), RecentsError> {
        if (caller_caps & CAP_RECENTS_MANAGE) == 0 {
            return Err(RecentsError::MissingManageCapability);
        }

        for slot in self.tasks.iter_mut() {
            if let Some(ref mut t) = slot {
                if t.task_id == task_id {
                    t.is_pinned = is_pinned;
                    return Ok(());
                }
            }
        }
        Err(RecentsError::TaskNotFound)
    }

    /// Swipe-to-terminate a single task card (INV-RECENTS-003).
    /// Returns the PID to terminate.
    pub fn swipe_terminate(&mut self, task_id: u32, caller_caps: u64) -> Result<u32, RecentsError> {
        if (caller_caps & CAP_RECENTS_MANAGE) == 0 {
            return Err(RecentsError::MissingManageCapability);
        }

        let mut found_idx = None;
        let mut target_pid = 0;

        for (idx, slot) in self.tasks.iter().enumerate() {
            if let Some(ref t) = slot {
                if t.task_id == task_id {
                    found_idx = Some(idx);
                    target_pid = t.pid;
                    break;
                }
            }
        }

        let idx = found_idx.ok_or(RecentsError::TaskNotFound)?;

        // Shift remaining tasks to keep compact array
        for i in idx..(self.task_count - 1) {
            self.tasks[i] = self.tasks[i + 1];
        }
        self.tasks[self.task_count - 1] = None;
        self.task_count -= 1;

        if self.active_index >= self.task_count && self.task_count > 0 {
            self.active_index = self.task_count - 1;
        }

        Ok(target_pid)
    }

    /// Clear all unpinned tasks (INV-RECENTS-004).
    /// Returns an array of terminated PIDs and the count.
    pub fn clear_all_unpinned(
        &mut self,
        caller_caps: u64,
    ) -> Result<([u32; MAX_RECENTS_TASKS], usize), RecentsError> {
        if (caller_caps & CAP_RECENTS_MANAGE) == 0 {
            return Err(RecentsError::MissingManageCapability);
        }

        let mut terminated_pids = [0u32; MAX_RECENTS_TASKS];
        let mut term_count = 0;

        let mut write_idx = 0;
        for read_idx in 0..self.task_count {
            if let Some(t) = self.tasks[read_idx] {
                if t.is_pinned {
                    self.tasks[write_idx] = Some(t);
                    write_idx += 1;
                } else {
                    terminated_pids[term_count] = t.pid;
                    term_count += 1;
                }
            }
        }

        for i in write_idx..MAX_RECENTS_TASKS {
            self.tasks[i] = None;
        }
        self.task_count = write_idx;
        self.active_index = 0;

        Ok((terminated_pids, term_count))
    }

    /// Aggregate memory footprint across active tasks in KB (INV-RECENTS-005).
    pub fn total_memory_kb(&self) -> usize {
        let mut sum = 0;
        for slot in self.tasks.iter() {
            if let Some(ref t) = slot {
                sum += t.memory_kb;
            }
        }
        sum
    }

    pub fn get_task(&self, task_id: u32) -> Option<&RecentsTask> {
        self.tasks.iter().find_map(|s| {
            s.as_ref().filter(|t| t.task_id == task_id)
        })
    }

    pub fn get_task_at(&self, index: usize) -> Option<&RecentsTask> {
        if index < self.task_count {
            self.tasks[index].as_ref()
        } else {
            None
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_thumb(id: u32) -> ThumbnailDescriptor {
        ThumbnailDescriptor::new(id, 1080, 2268, 1080, 0xAABBCCDD)
    }

    fn make_recents() -> RecentsCarouselEngine {
        RecentsCarouselEngine::new()
    }

    #[test]
    // INV-RECENTS-001: bottom edge gesture opens recents
    fn test_bottom_edge_gesture_open() {
        let mut r = make_recents();
        assert!(!r.is_open());

        // Invalid: start y not at bottom edge
        let err1 = r.handle_bottom_edge_gesture(2000, 1500, 500);
        assert_eq!(err1, Err(RecentsError::InvalidGesture));

        // Invalid: downward drag
        let err2 = r.handle_bottom_edge_gesture(2400, 2450, 500);
        assert_eq!(err2, Err(RecentsError::InvalidGesture));

        // Valid upward swipe from bottom nav area
        let ok = r.handle_bottom_edge_gesture(2400, 2000, 800);
        assert_eq!(ok, Ok(true));
        assert!(r.is_open());

        r.close_carousel();
        assert!(!r.is_open());
    }

    #[test]
    // Registration requires CAP_RECENTS_MANAGE
    fn test_register_requires_cap() {
        let mut r = make_recents();
        let err = r.register_task(100, "ind.app.files", "Files", dummy_thumb(1), 8192, 0);
        assert_eq!(err, Err(RecentsError::MissingManageCapability));

        let ok = r.register_task(100, "ind.app.files", "Files", dummy_thumb(1), 8192, CAP_RECENTS_MANAGE);
        assert!(ok.is_ok());
        assert_eq!(r.task_count(), 1);
    }

    #[test]
    // Updating existing PID updates thumbnail & memory without duplicating slot
    fn test_register_updates_existing_pid() {
        let mut r = make_recents();
        let t1 = r.register_task(100, "ind.app.files", "Files", dummy_thumb(1), 8192, CAP_RECENTS_MANAGE).unwrap();
        let t2 = r.register_task(100, "ind.app.files", "Files v2", dummy_thumb(2), 12000, CAP_RECENTS_MANAGE).unwrap();

        assert_eq!(t1, t2);
        assert_eq!(r.task_count(), 1);

        let task = r.get_task(t1).unwrap();
        assert_eq!(task.title, "Files v2");
        assert_eq!(task.thumbnail.buffer_id, 2);
        assert_eq!(task.memory_kb, 12000);
    }

    #[test]
    // Carousel navigation and task selection
    fn test_navigation_and_selection() {
        let mut r = make_recents();
        r.register_task(100, "ind.app.files", "Files", dummy_thumb(1), 8192, CAP_RECENTS_MANAGE).unwrap();
        r.register_task(101, "ind.app.notes", "Notes", dummy_thumb(2), 6144, CAP_RECENTS_MANAGE).unwrap();
        r.register_task(102, "ind.app.browser", "Browser", dummy_thumb(3), 32000, CAP_RECENTS_MANAGE).unwrap();

        // Open carousel
        r.handle_bottom_edge_gesture(2400, 2000, 500).unwrap();
        assert_eq!(r.active_index(), 2); // points to last task

        r.scroll_prev();
        assert_eq!(r.active_index(), 1);

        r.scroll_prev();
        assert_eq!(r.active_index(), 0);

        r.scroll_next();
        assert_eq!(r.active_index(), 1);

        // Select task 1 (Notes)
        let selected = r.select_current_task();
        assert_eq!(selected, Some((101, "ind.app.notes")));
        assert!(!r.is_open()); // closed after selection
    }

    #[test]
    // INV-RECENTS-003: swipe to terminate removes task and returns PID
    fn test_swipe_terminate() {
        let mut r = make_recents();
        let t1 = r.register_task(100, "ind.app.files", "Files", dummy_thumb(1), 8192, CAP_RECENTS_MANAGE).unwrap();
        let _t2 = r.register_task(101, "ind.app.notes", "Notes", dummy_thumb(2), 6144, CAP_RECENTS_MANAGE).unwrap();

        assert_eq!(r.task_count(), 2);

        let term_pid = r.swipe_terminate(t1, CAP_RECENTS_MANAGE).unwrap();
        assert_eq!(term_pid, 100);
        assert_eq!(r.task_count(), 1);
        assert!(r.get_task(t1).is_none());
    }

    #[test]
    // INV-RECENTS-004: clear all unpinned preserves pinned tasks
    fn test_clear_all_unpinned_preserves_pinned() {
        let mut r = make_recents();
        let t1 = r.register_task(100, "ind.app.music", "Music", dummy_thumb(1), 16000, CAP_RECENTS_MANAGE).unwrap();
        let _t2 = r.register_task(101, "ind.app.notes", "Notes", dummy_thumb(2), 6144, CAP_RECENTS_MANAGE).unwrap();
        let _t3 = r.register_task(102, "ind.app.files", "Files", dummy_thumb(3), 8192, CAP_RECENTS_MANAGE).unwrap();

        // Pin music
        r.set_pinned(t1, true, CAP_RECENTS_MANAGE).unwrap();
        assert!(r.get_task(t1).unwrap().is_pinned);

        // Clear all unpinned
        let (pids, count) = r.clear_all_unpinned(CAP_RECENTS_MANAGE).unwrap();
        assert_eq!(count, 2);
        assert_eq!(pids[0], 101);
        assert_eq!(pids[1], 102);

        // Only Music remains
        assert_eq!(r.task_count(), 1);
        let rem = r.get_task_at(0).unwrap();
        assert_eq!(rem.pid, 100);
        assert_eq!(rem.app_id, "ind.app.music");
    }

    #[test]
    // INV-RECENTS-005: aggregate memory calculation
    fn test_aggregate_memory() {
        let mut r = make_recents();
        r.register_task(100, "ind.app.files", "Files", dummy_thumb(1), 8192, CAP_RECENTS_MANAGE).unwrap();
        r.register_task(101, "ind.app.notes", "Notes", dummy_thumb(2), 4096, CAP_RECENTS_MANAGE).unwrap();

        assert_eq!(r.total_memory_kb(), 12288);
    }
}

