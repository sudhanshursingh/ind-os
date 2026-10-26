//! Compact, Unforgeable Kernel Object Identifier (KObjectId)
//! Implements generation tracking for Anti-ABA and Use-After-Free protection (INV-OBJ-001).

use crate::object::types::KObjectType;

/// Unforgeable Kernel Object Identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KObjectId {
    pub index: u16,
    pub generation: u16,
    pub obj_type: KObjectType,
}

impl KObjectId {
    pub const fn new(index: u16, generation: u16, obj_type: KObjectType) -> Self {
        Self {
            index,
            generation,
            obj_type,
        }
    }

    /// Packs the identifier into a 32-bit compact integer for capability badges/tokens
    pub const fn to_u32(&self) -> u32 {
        ((self.obj_type as u32) << 24)
            | (((self.generation & 0xFF) as u32) << 16)
            | (self.index as u32)
    }

    /// Unpacks an identifier from a 32-bit compact integer
    pub fn from_u32(val: u32) -> Option<Self> {
        let type_val = (val >> 24) as u8;
        let generation = ((val >> 16) & 0xFF) as u16;
        let index = (val & 0xFFFF) as u16;

        let obj_type = match type_val {
            1 => KObjectType::Process,
            2 => KObjectType::Endpoint,
            3 => KObjectType::MemoryRegion,
            4 => KObjectType::Irq,
            _ => return None,
        };

        Some(Self {
            index,
            generation,
            obj_type,
        })
    }
}
