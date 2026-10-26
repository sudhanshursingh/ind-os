//! Virtual File System (VFS) Core Subsystem
//! Compliant with Bible §8.10 (SP-24), REQ-KRN-106, INV-VFS-001, and INV-VFS-002.

pub mod core;
pub mod file;
pub mod mount;
pub mod path;
pub mod vnode;

use crate::sync::spinlock::Spinlock;
pub use self::core::{Vfs, MAX_OPEN_FILES, MAX_VNODES};
pub use file::FileDescriptor;
pub use mount::{MountEntry, MountTable, MAX_MOUNTS};
pub use path::PathResolver;
pub use vnode::{Vnode, VnodeId, VnodeType};

/// Global Virtual File System singleton protected by spinlock
pub static VFS: Spinlock<Vfs> = Spinlock::new(Vfs::new());
