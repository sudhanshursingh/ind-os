//! Bounded Panic-Free Collections for Ind OS Userspace
//!
//! Constitutional References:
//! - REQ-USE-202: no_std-friendly collections
//! - INV-LIB-001: Bounded Collections without heap allocation
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionError {
    CapacityExceeded,
    NotFound,
    Empty,
}

/// Fixed-capacity vector with pre-allocated storage.
#[derive(Debug, Clone)]
pub struct StaticVec<T: Copy + Default, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T: Copy + Default, const N: usize> StaticVec<T, N> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), CollectionError> {
        if self.len >= N {
            return Err(CollectionError::CapacityExceeded);
        }
        self.data[self.len] = item;
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(self.data[self.len])
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(&self.data[index])
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

/// Bounded FIFO circular ring buffer.
#[derive(Debug, Clone)]
pub struct RingBuffer<T: Copy + Default, const N: usize> {
    buffer: [T; N],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T: Copy + Default, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), CollectionError> {
        if self.count >= N {
            return Err(CollectionError::CapacityExceeded);
        }
        self.buffer[self.tail] = item;
        self.tail = (self.tail + 1) % N;
        self.count += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        let item = self.buffer[self.head];
        self.head = (self.head + 1) % N;
        self.count -= 1;
        Some(item)
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == N
    }
}

/// Fixed-capacity UTF-8 string wrapper.
#[derive(Debug, Clone, Copy)]
pub struct StaticString<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> StaticString<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0u8; N],
            len: 0,
        }
    }

    pub fn from_str(s: &str) -> Result<Self, CollectionError> {
        if s.len() > N {
            return Err(CollectionError::CapacityExceeded);
        }
        let mut res = Self::new();
        let src = s.as_bytes();
        let mut i = 0;
        while i < src.len() {
            res.bytes[i] = src[i];
            i += 1;
        }
        res.len = src.len();
        Ok(res)
    }

    pub fn as_str(&self) -> &str {
        match core::str::from_utf8(&self.bytes[..self.len]) {
            Ok(s) => s,
            Err(_) => "",
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Bounded associative key-value map using linear search.
#[derive(Debug, Clone)]
pub struct StaticMap<K: Copy + PartialEq + Default, V: Copy + Default, const N: usize> {
    keys: [K; N],
    values: [V; N],
    occupied: [bool; N],
    count: usize,
}

impl<K: Copy + PartialEq + Default, V: Copy + Default, const N: usize> StaticMap<K, V, N> {
    pub fn new() -> Self {
        Self {
            keys: [K::default(); N],
            values: [V::default(); N],
            occupied: [false; N],
            count: 0,
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> Result<(), CollectionError> {
        // Update if existing
        for i in 0..N {
            if self.occupied[i] && self.keys[i] == key {
                self.values[i] = val;
                return Ok(());
            }
        }
        // Insert new
        for i in 0..N {
            if !self.occupied[i] {
                self.keys[i] = key;
                self.values[i] = val;
                self.occupied[i] = true;
                self.count += 1;
                return Ok(());
            }
        }
        Err(CollectionError::CapacityExceeded)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        for i in 0..N {
            if self.occupied[i] && &self.keys[i] == key {
                return Some(&self.values[i]);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &K) -> bool {
        for i in 0..N {
            if self.occupied[i] && &self.keys[i] == key {
                self.occupied[i] = false;
                self.count -= 1;
                return true;
            }
        }
        false
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
