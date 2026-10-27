//! Ind OS AIS Schema Registry (REQ-AI-102, INV-INTENT-001)
//!
//! Standards:
//! - REQ-AI-102: Typed proposal schema and schema registry
//! - INV-INTENT-001: Typed proposal schema enforcement
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const MAX_REGISTERED_SCHEMAS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    DeviceControl = 1,
    FileOperation = 2,
    SystemSetting = 3,
    Notification = 4,
    NetworkTransfer = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaDescriptor {
    pub category: ActionCategory,
    pub action_code: u32,
    pub name: &'static str,
    pub required_rights: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaError {
    RegistryFull,
    AlreadyRegistered,
    NotFound,
    InvalidCategory,
}

pub struct SchemaRegistry {
    schemas: [Option<SchemaDescriptor>; MAX_REGISTERED_SCHEMAS],
    count: usize,
}

impl SchemaRegistry {
    pub const fn new() -> Self {
        Self {
            schemas: [None; MAX_REGISTERED_SCHEMAS],
            count: 0,
        }
    }

    /// Registers a new typed action schema.
    pub fn register(&mut self, descriptor: SchemaDescriptor) -> Result<(), SchemaError> {
        if self.count >= MAX_REGISTERED_SCHEMAS {
            return Err(SchemaError::RegistryFull);
        }

        // Check duplicates
        for i in 0..self.count {
            if let Some(s) = self.schemas[i] {
                if s.category == descriptor.category && s.action_code == descriptor.action_code {
                    return Err(SchemaError::AlreadyRegistered);
                }
            }
        }

        self.schemas[self.count] = Some(descriptor);
        self.count += 1;
        Ok(())
    }

    /// Looks up a schema by category and action code.
    pub fn lookup(&self, category: ActionCategory, action_code: u32) -> Option<SchemaDescriptor> {
        for i in 0..self.count {
            if let Some(s) = self.schemas[i] {
                if s.category == category && s.action_code == action_code {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// Pre-populates the registry with standard baseline schemas.
    pub fn load_baseline(&mut self) -> Result<(), SchemaError> {
        self.register(SchemaDescriptor {
            category: ActionCategory::DeviceControl,
            action_code: 101, // Toggle Bluetooth
            name: "device.bluetooth.toggle",
            required_rights: 0x02,
        })?;
        self.register(SchemaDescriptor {
            category: ActionCategory::SystemSetting,
            action_code: 201, // Adjust Brightness
            name: "system.brightness.set",
            required_rights: 0x01,
        })?;
        self.register(SchemaDescriptor {
            category: ActionCategory::Notification,
            action_code: 301, // Show User Alert
            name: "notification.alert.show",
            required_rights: 0x04,
        })?;
        self.register(SchemaDescriptor {
            category: ActionCategory::FileOperation,
            action_code: 401, // Read User Document
            name: "file.document.read",
            required_rights: 0x08,
        })?;
        Ok(())
    }
}
