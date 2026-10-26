//! User-Mode VirtIO Block Reference Driver (virtio-blk)
//! Compliant with Bible §8.10 (SP-27), INDOS-SPEC-REFDRV-01, Rule N-01, Rule N-04, and REQ-KRN-107.

use crate::capability::{CapType, Capability};
use crate::driver_framework::dma_broker::{DmaBroker, DmaDirection};
use crate::driver_framework::protocol::{DriverMessage, DriverOp, DriverStatus};
use crate::ipc::IpcMessage;
use crate::task::ProcessId;

pub const SECTOR_SIZE: usize = 512;

/// User-mode VirtIO block driver instance running in isolated user process
#[derive(Clone, Copy, Debug)]
pub struct VirtioBlkDriver {
    pub device_id: u32,
    pub owner_pid: ProcessId,
    pub capacity_sectors: u64,
    pub is_ready: bool,
    pub total_reads: u64,
    pub total_writes: u64,
}

impl VirtioBlkDriver {
    pub const fn new(device_id: u32, owner_pid: ProcessId, capacity_sectors: u64) -> Self {
        Self {
            device_id,
            owner_pid,
            capacity_sectors,
            is_ready: false,
            total_reads: 0,
            total_writes: 0,
        }
    }

    /// Initializes device binding using granted device capability token (INV-DRV-001)
    pub fn init(&mut self, dev_cap: &Capability) -> Result<(), &'static str> {
        if !dev_cap.is_valid || dev_cap.cap_type != CapType::Device(self.device_id) {
            return Err("EPERM: Access denied: Invalid device capability for virtio-blk (INV-DRV-001)");
        }
        self.is_ready = true;
        Ok(())
    }

    /// Handles a sector read request mediated by DMA window validation (INV-DRV-003)
    pub fn handle_io_read(
        &mut self,
        lba: u64,
        sector_count: u32,
        dma_window_id: u32,
        dma_broker: &DmaBroker,
        dma_cap: &Capability,
    ) -> Result<usize, &'static str> {
        if !self.is_ready {
            return Err("EBUSY: Driver not initialized");
        }
        if lba + (sector_count as u64) > self.capacity_sectors {
            return Err("EINVAL: Read beyond device capacity");
        }

        let bytes_to_transfer = (sector_count as usize) * SECTOR_SIZE;

        // Enforce DMA bounds checking and SMMU isolation (INV-DRV-003)
        dma_broker.validate_transfer(
            dma_window_id,
            self.owner_pid,
            0,
            bytes_to_transfer,
            DmaDirection::DeviceToHost,
            dma_cap,
        )?;

        self.total_reads += 1;
        Ok(bytes_to_transfer)
    }

    /// Handles a sector write request mediated by DMA window validation (INV-DRV-003)
    pub fn handle_io_write(
        &mut self,
        lba: u64,
        sector_count: u32,
        dma_window_id: u32,
        dma_broker: &DmaBroker,
        dma_cap: &Capability,
    ) -> Result<usize, &'static str> {
        if !self.is_ready {
            return Err("EBUSY: Driver not initialized");
        }
        if lba + (sector_count as u64) > self.capacity_sectors {
            return Err("EINVAL: Write beyond device capacity");
        }

        let bytes_to_transfer = (sector_count as usize) * SECTOR_SIZE;

        // Enforce DMA bounds checking and SMMU isolation (INV-DRV-003)
        dma_broker.validate_transfer(
            dma_window_id,
            self.owner_pid,
            0,
            bytes_to_transfer,
            DmaDirection::HostToDevice,
            dma_cap,
        )?;

        self.total_writes += 1;
        Ok(bytes_to_transfer)
    }

    /// Dispatches an incoming DriverProtocol IPC message
    pub fn handle_ipc_message(
        &mut self,
        msg: &IpcMessage,
        dma_broker: &DmaBroker,
        dma_cap: &Capability,
    ) -> IpcMessage {
        let req = DriverMessage::from_ipc_message(msg);
        match req.op {
            DriverOp::IoRead => {
                let lba = req.arg1;
                let sectors = req.arg2 as u32;
                let dma_win_id = req.target_id as u32;
                match self.handle_io_read(lba, sectors, dma_win_id, dma_broker, dma_cap) {
                    Ok(bytes) => DriverMessage::io_response(req.correlation_id, DriverStatus::Ok, bytes).to_ipc_message(),
                    Err(_) => DriverMessage::io_response(req.correlation_id, DriverStatus::Einval, 0).to_ipc_message(),
                }
            }
            DriverOp::IoWrite => {
                let lba = req.arg1;
                let sectors = req.arg2 as u32;
                let dma_win_id = req.target_id as u32;
                match self.handle_io_write(lba, sectors, dma_win_id, dma_broker, dma_cap) {
                    Ok(bytes) => DriverMessage::io_response(req.correlation_id, DriverStatus::Ok, bytes).to_ipc_message(),
                    Err(_) => DriverMessage::io_response(req.correlation_id, DriverStatus::Einval, 0).to_ipc_message(),
                }
            }
            DriverOp::Status => {
                DriverMessage::new(DriverOp::Status, req.correlation_id, self.capacity_sectors, self.total_reads, self.total_writes)
                    .to_ipc_message()
            }
            _ => DriverMessage::io_response(req.correlation_id, DriverStatus::Einval, 0).to_ipc_message(),
        }
    }
}
