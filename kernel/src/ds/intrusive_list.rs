//! Intrusive Doubly-Linked List for IndK
//! Embeds linkage pointers directly within items to guarantee zero dynamic allocation.

use core::ptr;

/// Intrusive list node embedded in data items
#[repr(C)]
#[derive(Debug)]
pub struct IntrusiveNode {
    pub next: *mut IntrusiveNode,
    pub prev: *mut IntrusiveNode,
}

impl IntrusiveNode {
    pub const fn new() -> Self {
        Self {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }

    #[inline(always)]
    pub fn is_linked(&self) -> bool {
        !self.next.is_null() || !self.prev.is_null()
    }
}

/// Zero-allocation intrusive doubly-linked list
pub struct IntrusiveList {
    head: *mut IntrusiveNode,
    tail: *mut IntrusiveNode,
    count: usize,
}

impl IntrusiveList {
    pub const fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            count: 0,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Append a node to the tail of the list
    pub fn push_back(&mut self, node: *mut IntrusiveNode) {
        if node.is_null() {
            return;
        }
        unsafe {
            (*node).next = ptr::null_mut();
            (*node).prev = self.tail;

            if self.tail.is_null() {
                self.head = node;
            } else {
                (*self.tail).next = node;
            }
            self.tail = node;
            self.count += 1;
        }
    }

    /// Prepend a node to the head of the list
    pub fn push_front(&mut self, node: *mut IntrusiveNode) {
        if node.is_null() {
            return;
        }
        unsafe {
            (*node).next = self.head;
            (*node).prev = ptr::null_mut();

            if self.head.is_null() {
                self.tail = node;
            } else {
                (*self.head).prev = node;
            }
            self.head = node;
            self.count += 1;
        }
    }

    /// Remove and return the node at the head of the list
    pub fn pop_front(&mut self) -> Option<*mut IntrusiveNode> {
        if self.head.is_null() {
            return None;
        }
        unsafe {
            let node = self.head;
            self.head = (*node).next;
            if self.head.is_null() {
                self.tail = ptr::null_mut();
            } else {
                (*self.head).prev = ptr::null_mut();
            }
            (*node).next = ptr::null_mut();
            (*node).prev = ptr::null_mut();
            self.count -= 1;
            Some(node)
        }
    }

    /// Unlink an arbitrary node from anywhere in the list
    pub fn remove(&mut self, node: *mut IntrusiveNode) {
        if node.is_null() {
            return;
        }
        unsafe {
            let prev = (*node).prev;
            let next = (*node).next;

            if !prev.is_null() {
                (*prev).next = next;
            } else if self.head == node {
                self.head = next;
            }

            if !next.is_null() {
                (*next).prev = prev;
            } else if self.tail == node {
                self.tail = prev;
            }

            (*node).next = ptr::null_mut();
            (*node).prev = ptr::null_mut();
            self.count = self.count.saturating_sub(1);
        }
    }
}
