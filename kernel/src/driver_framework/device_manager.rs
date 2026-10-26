//! IndOS User-Mode Device Manager
//! Compliant with Bible §8.10 (SP-26), INDOS-SPEC-DRV-01, Rule N-01, Rule N-04, and Invariant INV-DRV-001.

use crate::capability::{CapRights, CapType, Capability};
use crate::task::ProcessId;

pub const MAX_DEVICES: usize = 16;

/// Hardware and virtual device class categorization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceType {
    Generic,
    Block,
    Net,
    Display,
    Input,
    Serial,
}

/// Lifecycle state of a registered device
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceState {
    Unregistered,
    Registered,
    Bound,
    Active,
    Faulted,
}

/// Bounded device descriptor for managed hardware & VirtIO devices
#[derive(Clone, Copy, Debug)]
pub struct DeviceDescriptor {
    pub device_id: u32,
    pub name: [u8; 32],
    pub name_len: usize,
    pub vendor_id: u32,
    pub device_type: DeviceType,
    pub mmio_base: u64,
    pub mmio_size: usize,
    pub irq_line: u32,
    pub state: DeviceState,
    pub bound_driver_pid: Option<ProcessId>,
    pub bound_endpoint: Option<u32>,
    pub badge: u64,
}

impl DeviceDescriptor {
    pub const fn empty() -> Self {
        Self {
            device_id: 0,
            name: [0u8; 32],
            name_len: 0,
            vendor_id: 0,
            device_type: DeviceType::Generic,
            mmio_base: 0,
            mmio_size: 0,
            irq_line: 0,
            state: DeviceState::Unregistered,
            bound_driver_pid: None,
            bound_endpoint: None,
            badge: 0,
        }
    }

    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

/// Registry and access broker for user-mode drivers
pub struct DeviceManager {
    pub devices: [DeviceDescriptor; MAX_DEVICES],
    pub device_count: usize,
}

impl DeviceManager {
    pub const fn new() -> Self {
        Self {
            devices: [DeviceDescriptor::empty(); MAX_DEVICES],
            device_count: 0,
        }
    }

    /// Registers a new hardware or virtual device with the framework
    pub fn register_device(
        &mut self,
        name: &str,
        vendor_id: u32,
        device_type: DeviceType,
        mmio_base: u64,
        mmio_size: usize,
        irq_line: u32,
    ) -> Result<u32, &'static str> {
        if self.device_count >= MAX_DEVICES {
            return Err("ENOMEM: Device manager registry full");
        }

        let dev_id = (self.device_count + 1) as u32;
        let mut desc = DeviceDescriptor::empty();
        desc.device_id = dev_id;
        desc.vendor_id = vendor_id;
        desc.device_type = device_type;
        desc.mmio_base = mmio_base;
        desc.mmio_size = mmio_size;
        desc.irq_line = irq_line;
        desc.state = DeviceState::Registered;
        desc.badge = 0xD000_0000 | (dev_id as u64);

        let bytes = name.as_bytes();
        let len = if bytes.len() > 32 { 32 } else { bytes.len() };
        desc.name[..len].copy_from_slice(&bytes[..len]);
        desc.name_len = len;

        self.devices[self.device_count] = desc;
        self.device_count += 1;
        Ok(dev_id)
    }

    /// Binds an isolated user-mode driver process to a device via capability authentication (INV-DRV-001)
    pub fn bind_driver(
        &mut self,
        device_id: u32,
        driver_pid: ProcessId,
        endpoint_id: u32,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        // Enforce INV-DRV-001: Zero unprivileged device access
        if !cap.is_valid {
            return Err("EPERM: Access denied: Presented capability token is invalid (INV-DRV-001)");
        }
        if cap.cap_type != CapType::Device(device_id) {
            return Err("EPERM: Access denied: Capability does not grant access to this device (INV-DRV-001)");
        }
        if !cap.rights.contains(CapRights::WRITE) {
            return Err("EPERM: Access denied: Capability lacks BIND/WRITE rights (INV-DRV-001)");
        }

        let slot = (device_id as usize).checked_sub(1).ok_or("EINVAL: Invalid device id")?;
        if slot >= self.device_count || self.devices[slot].device_id != device_id {
            return Err("ENODEV: Device not found");
        }

        let dev = &mut self.devices[slot];
        if dev.state == DeviceState::Bound || dev.state == DeviceState::Active {
            return Err("EBUSY: Device already bound to a driver");
        }

        dev.bound_driver_pid = Some(driver_pid);
        dev.bound_endpoint = Some(endpoint_id);
        dev.state = DeviceState::Bound;
        Ok(())
    }

    /// Transitions a bound device to active operational state
    pub fn activate_device(
        &mut self,
        device_id: u32,
        driver_pid: ProcessId,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        if !cap.is_valid || cap.cap_type != CapType::Device(device_id) {
            return Err("EPERM: Invalid device capability");
        }

        let slot = (device_id as usize).checked_sub(1).ok_or("EINVAL: Invalid device id")?;
        if slot >= self.device_count || self.devices[slot].device_id != device_id {
            return Err("ENODEV: Device not found");
        }

        let dev = &mut self.devices[slot];
        if dev.bound_driver_pid != Some(driver_pid) {
            return Err("EPERM: Caller is not the bound driver for this device");
        }

        dev.state = DeviceState::Active;
        Ok(())
    }

    /// Unbinds a driver from a device, resetting state to Registered
    pub fn unbind_driver(
        &mut self,
        device_id: u32,
        driver_pid: ProcessId,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        if !cap.is_valid || cap.cap_type != CapType::Device(device_id) {
            return Err("EPERM: Invalid device capability");
        }

        let slot = (device_id as usize).checked_sub(1).ok_or("EINVAL: Invalid device id")?;
        if slot >= self.device_count || self.devices[slot].device_id != device_id {
            return Err("ENODEV: Device not found");
        }

        let dev = &mut self.devices[slot];
        if dev.bound_driver_pid != Some(driver_pid) {
            return Err("EPERM: Caller is not the bound driver for this device");
        }

        dev.bound_driver_pid = None;
        dev.bound_endpoint = None;
        dev.state = DeviceState::Registered;
        Ok(())
    }

    /// Looks up a device by ID
    pub fn get_device(&self, device_id: u32) -> Option<&DeviceDescriptor> {
        let slot = (device_id as usize).checked_sub(1)?;
        if slot < self.device_count && self.devices[slot].device_id == device_id {
            Some(&self.devices[slot])
        } else {
            None
        }
    }

    /// Looks up a device by name
    pub fn find_device_by_name(&self, name: &str) -> Option<&DeviceDescriptor> {
        for i in 0..self.device_count {
            if self.devices[i].name() == name {
                return Some(&self.devices[i]);
            }
        }
        None
    }
}
