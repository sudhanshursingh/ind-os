//! Global Search Foundation — cross-app indexed search surface (SP-128, REQ-APP-COMMON-010)
//!
//! Bible §8.18.2, §9.9.1 REQ-APP-COMMON-010
//! Invariants:
//! - INV-SEARCH-001: Message bodies, photos, and browser history are excluded from the index
//!                   by default; inclusion requires explicit per-app user opt-in.
//! - INV-SEARCH-002: Search index entries are capability-gated: a consumer app may only surface
//!                   results for apps that have granted the search service the required read cap.
//! - INV-SEARCH-003: The index stores only metadata descriptors — no raw content bodies.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user controls opt-in)

/// Maximum number of indexed entries.
pub const MAX_INDEX_ENTRIES: usize = 256;
/// Maximum length of a search-term or result title (bytes).
pub const MAX_TERM_LEN: usize = 128;
/// Maximum results returned per query.
pub const MAX_QUERY_RESULTS: usize = 20;

// ─── Source classification ────────────────────────────────────────────────────

/// The origin category of a search index entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSourceKind {
    App,
    Contact,
    RecentFile,
    Setting,
    /// Excluded by default (INV-SEARCH-001). Requires explicit opt-in.
    MessageBody,
    /// Excluded by default (INV-SEARCH-001). Requires explicit opt-in.
    Photo,
    /// Excluded by default (INV-SEARCH-001). Requires explicit opt-in.
    BrowserHistory,
}

impl SearchSourceKind {
    /// Returns true if this source kind is permitted without opt-in.
    pub const fn is_default_included(self) -> bool {
        matches!(
            self,
            SearchSourceKind::App
                | SearchSourceKind::Contact
                | SearchSourceKind::RecentFile
                | SearchSourceKind::Setting
        )
    }
}

// ─── Index entry ─────────────────────────────────────────────────────────────

/// A single metadata descriptor in the search index (INV-SEARCH-003: no raw content bodies).
#[derive(Debug, Clone, Copy)]
pub struct SearchEntry {
    pub entry_id: u32,
    /// Source app PID that contributed this entry.
    pub source_pid: u32,
    pub source_kind: SearchSourceKind,
    /// Human-readable title for display in results.
    pub title: [u8; MAX_TERM_LEN],
    pub title_len: usize,
    /// Searchable keyword token (lower-cased ASCII, space-stripped).
    pub token: [u8; MAX_TERM_LEN],
    pub token_len: usize,
    /// Capability required for a consumer to read this entry (INV-SEARCH-002).
    pub required_cap: u64,
    /// Whether the entry was explicitly opted-in by the user (required for excluded kinds).
    pub user_opted_in: bool,
}

impl SearchEntry {
    /// Returns true if the entry is eligible for indexing given the opt-in state.
    pub fn is_indexable(&self) -> bool {
        if self.source_kind.is_default_included() {
            return true;
        }
        // Excluded-by-default sources require explicit user opt-in (INV-SEARCH-001).
        self.user_opted_in
    }

    /// Case-insensitive prefix match on the token.
    pub fn matches_prefix(&self, query: &[u8]) -> bool {
        if query.len() > self.token_len {
            return false;
        }
        for (a, b) in self.token[..query.len()].iter().zip(query.iter()) {
            let a_lower = if a.is_ascii_uppercase() { a + 32 } else { *a };
            let b_lower = if b.is_ascii_uppercase() { b + 32 } else { *b };
            if a_lower != b_lower {
                return false;
            }
        }
        true
    }
}

// ─── Search result ────────────────────────────────────────────────────────────

/// A resolved search result returned to the calling surface.
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub entry_id: u32,
    pub source_pid: u32,
    pub source_kind: SearchSourceKind,
    pub title: [u8; MAX_TERM_LEN],
    pub title_len: usize,
}

// ─── Opt-in record for excluded sources ──────────────────────────────────────

/// Tracks which (source_pid, source_kind) pairs the user has opted in to index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchOptIn {
    pub source_pid: u32,
    pub source_kind: SearchSourceKind,
}

pub const MAX_OPT_INS: usize = 32;

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchError {
    /// Source kind requires opt-in which has not been granted.
    OptInRequired,
    /// Indexer lacks the capability to contribute entries of this kind.
    MissingIndexCapability,
    /// Consumer lacks the capability to read results from this source.
    MissingReadCapability,
    /// Index is full.
    IndexFull,
    /// Opt-in table is full.
    OptInTableFull,
    /// Entry not found.
    EntryNotFound,
    /// Query is too long.
    QueryTooLong,
}

// ─── Engine ──────────────────────────────────────────────────────────────────

/// Capability required to contribute entries to the search index.
pub const CAP_SEARCH_INDEX: u64 = 0x0000_0000_0000_4000;
/// Capability required to query the search index.
pub const CAP_SEARCH_QUERY: u64 = 0x0000_0000_0000_8000;

/// Global Search Foundation engine (REQ-APP-COMMON-010, INV-SEARCH-001..003).
pub struct GlobalSearchEngine {
    entries: [Option<SearchEntry>; MAX_INDEX_ENTRIES],
    entry_count: usize,
    next_entry_id: u32,
    opt_ins: [Option<SearchOptIn>; MAX_OPT_INS],
    opt_in_count: usize,
}

impl GlobalSearchEngine {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_INDEX_ENTRIES],
            entry_count: 0,
            next_entry_id: 1,
            opt_ins: [None; MAX_OPT_INS],
            opt_in_count: 0,
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    // ── Opt-in management (INV-SEARCH-001) ───────────────────────────────────

    /// Record that the user has opted a (source_pid, source_kind) into the search index.
    pub fn grant_opt_in(
        &mut self,
        source_pid: u32,
        source_kind: SearchSourceKind,
    ) -> Result<(), SearchError> {
        // Idempotent: already opted-in is fine.
        if self.is_opted_in(source_pid, source_kind) {
            return Ok(());
        }
        for slot in self.opt_ins.iter_mut() {
            if slot.is_none() {
                *slot = Some(SearchOptIn { source_pid, source_kind });
                self.opt_in_count += 1;
                return Ok(());
            }
        }
        Err(SearchError::OptInTableFull)
    }

    fn is_opted_in(&self, source_pid: u32, source_kind: SearchSourceKind) -> bool {
        self.opt_ins.iter().any(|o| {
            o.map(|oi| oi.source_pid == source_pid && oi.source_kind == source_kind)
                .unwrap_or(false)
        })
    }

    // ── Indexing ──────────────────────────────────────────────────────────────

    /// Contribute a metadata entry to the search index.
    /// `source_caps` must include CAP_SEARCH_INDEX.
    /// Excluded-by-default source kinds require prior user opt-in (INV-SEARCH-001).
    pub fn index_entry(
        &mut self,
        source_pid: u32,
        source_caps: u64,
        source_kind: SearchSourceKind,
        title: &[u8],
        token: &[u8],
        required_cap: u64,
    ) -> Result<u32, SearchError> {
        if (source_caps & CAP_SEARCH_INDEX) == 0 {
            return Err(SearchError::MissingIndexCapability);
        }
        if title.len() > MAX_TERM_LEN || token.len() > MAX_TERM_LEN {
            return Err(SearchError::QueryTooLong);
        }

        // Opt-in check for excluded source kinds (INV-SEARCH-001).
        if !source_kind.is_default_included() && !self.is_opted_in(source_pid, source_kind) {
            return Err(SearchError::OptInRequired);
        }

        let entry_id = self.next_entry_id;
        self.next_entry_id += 1;

        let mut entry = SearchEntry {
            entry_id,
            source_pid,
            source_kind,
            title: [0u8; MAX_TERM_LEN],
            title_len: title.len(),
            token: [0u8; MAX_TERM_LEN],
            token_len: token.len(),
            required_cap,
            user_opted_in: !source_kind.is_default_included(),
        };
        entry.title[..title.len()].copy_from_slice(title);
        entry.token[..token.len()].copy_from_slice(token);

        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(entry);
                self.entry_count += 1;
                return Ok(entry_id);
            }
        }
        Err(SearchError::IndexFull)
    }

    // ── Query ─────────────────────────────────────────────────────────────────

    /// Execute a prefix search against the index.
    /// Results are filtered by the caller's capabilities (INV-SEARCH-002).
    /// Returns up to MAX_QUERY_RESULTS entries in index order.
    pub fn query(
        &self,
        query_term: &[u8],
        caller_caps: u64,
    ) -> Result<([Option<SearchResult>; MAX_QUERY_RESULTS], usize), SearchError> {
        if query_term.len() > MAX_TERM_LEN {
            return Err(SearchError::QueryTooLong);
        }
        if (caller_caps & CAP_SEARCH_QUERY) == 0 {
            return Err(SearchError::MissingReadCapability);
        }

        let mut results = [None::<SearchResult>; MAX_QUERY_RESULTS];
        let mut count = 0usize;

        for slot in self.entries.iter() {
            if count >= MAX_QUERY_RESULTS {
                break;
            }
            if let Some(ref entry) = slot {
                if !entry.is_indexable() {
                    continue;
                }
                // Capability gate (INV-SEARCH-002): caller must hold the entry's required cap,
                // or the entry has no cap requirement (0 means open).
                if entry.required_cap != 0 && (caller_caps & entry.required_cap) == 0 {
                    continue;
                }
                if entry.matches_prefix(query_term) {
                    results[count] = Some(SearchResult {
                        entry_id: entry.entry_id,
                        source_pid: entry.source_pid,
                        source_kind: entry.source_kind,
                        title: entry.title,
                        title_len: entry.title_len,
                    });
                    count += 1;
                }
            }
        }

        Ok((results, count))
    }

    /// Remove an entry from the index.
    pub fn remove_entry(
        &mut self,
        entry_id: u32,
        caller_pid: u32,
    ) -> Result<(), SearchError> {
        for slot in self.entries.iter_mut() {
            if let Some(ref e) = slot {
                if e.entry_id == entry_id {
                    if e.source_pid != caller_pid {
                        // Only the source app may remove its own entries.
                        return Err(SearchError::MissingIndexCapability);
                    }
                    *slot = None;
                    self.entry_count -= 1;
                    return Ok(());
                }
            }
        }
        Err(SearchError::EntryNotFound)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> GlobalSearchEngine {
        GlobalSearchEngine::new()
    }

    fn index_setting(engine: &mut GlobalSearchEngine, title: &[u8], token: &[u8]) -> u32 {
        engine
            .index_entry(
                10,
                CAP_SEARCH_INDEX,
                SearchSourceKind::Setting,
                title,
                token,
                0, // no read cap required
            )
            .unwrap()
    }

    #[test]
    // REQ-APP-COMMON-010: basic indexing and query
    fn test_index_and_query() {
        let mut e = make_engine();
        index_setting(&mut e, b"Wi-Fi Settings", b"wi-fi");
        index_setting(&mut e, b"Bluetooth Settings", b"bluetooth");

        let (results, count) = e.query(b"wi", CAP_SEARCH_QUERY).unwrap();
        assert_eq!(count, 1);
        assert_eq!(&results[0].unwrap().title[..results[0].unwrap().title_len], b"Wi-Fi Settings");
    }

    #[test]
    // INV-SEARCH-001: excluded source kind requires opt-in
    fn test_excluded_source_requires_optin() {
        let mut e = make_engine();
        // MessageBody without opt-in → rejected.
        let err = e.index_entry(
            10,
            CAP_SEARCH_INDEX,
            SearchSourceKind::MessageBody,
            b"Hello",
            b"hello",
            0,
        );
        assert_eq!(err, Err(SearchError::OptInRequired));
    }

    #[test]
    // INV-SEARCH-001: excluded source kind succeeds after opt-in
    fn test_excluded_source_after_optin() {
        let mut e = make_engine();
        e.grant_opt_in(10, SearchSourceKind::MessageBody).unwrap();
        let ok = e.index_entry(
            10,
            CAP_SEARCH_INDEX,
            SearchSourceKind::MessageBody,
            b"Hello from Mom",
            b"hello from mom",
            0,
        );
        assert!(ok.is_ok());
    }

    #[test]
    // INV-SEARCH-002: caller without required_cap does not receive gated results
    fn test_capability_gating_on_query() {
        let mut e = make_engine();
        const CONTACTS_CAP: u64 = 0x00FF;
        // Index a contact entry requiring CONTACTS_CAP.
        e.index_entry(
            10,
            CAP_SEARCH_INDEX,
            SearchSourceKind::Contact,
            b"Priya Sharma",
            b"priya",
            CONTACTS_CAP,
        )
        .unwrap();

        // Query without CONTACTS_CAP — should return 0 results.
        let (_, count) = e.query(b"priya", CAP_SEARCH_QUERY).unwrap();
        assert_eq!(count, 0);

        // Query with CONTACTS_CAP — should return 1 result.
        let (_, count2) = e
            .query(b"priya", CAP_SEARCH_QUERY | CONTACTS_CAP)
            .unwrap();
        assert_eq!(count2, 1);
    }

    #[test]
    // Query requires CAP_SEARCH_QUERY
    fn test_query_requires_cap() {
        let mut e = make_engine();
        index_setting(&mut e, b"Display", b"display");
        let err = e.query(b"disp", 0).unwrap_err();
        assert_eq!(err, SearchError::MissingReadCapability);
    }

    #[test]
    // REQ-APP-COMMON-010: indexing requires CAP_SEARCH_INDEX
    fn test_index_requires_cap() {
        let mut e = make_engine();
        let err = e.index_entry(
            10,
            0, // no caps
            SearchSourceKind::App,
            b"App",
            b"app",
            0,
        );
        assert_eq!(err, Err(SearchError::MissingIndexCapability));
    }

    #[test]
    // Case-insensitive prefix matching
    fn test_case_insensitive_match() {
        let mut e = make_engine();
        index_setting(&mut e, b"Display Brightness", b"Display");

        // Query with mixed case
        let (results, count) = e.query(b"DISP", CAP_SEARCH_QUERY).unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            &results[0].unwrap().title[..results[0].unwrap().title_len],
            b"Display Brightness"
        );
    }

    #[test]
    // Entry removal only allowed by the source app
    fn test_remove_entry_ownership() {
        let mut e = make_engine();
        let eid = index_setting(&mut e, b"Sound", b"sound");
        // Different PID → rejected
        let err = e.remove_entry(eid, 99);
        assert_eq!(err, Err(SearchError::MissingIndexCapability));
        // Correct PID → ok
        e.remove_entry(eid, 10).unwrap();
        assert_eq!(e.entry_count(), 0);
    }
}
