//! User-Mode VirtIO Network Reference Driver (virtio-net)
//! Compliant with Bible §8.10 (SP-27), INDOS-SPEC-REFDRV-01, Rule N-01, Rule N-04, and REQ-KRN-107.

use crate::capability::{CapType, Capability};
use crate::driver_framework::dma_broker::{DmaBroker, DmaDirection};
use crate::driver_framework::protocol::DriverMessage;
use crate::task::ProcessId;

pub const DEFAULT_MTU: u16 = 1500;

/// User-mode VirtIO network driver instance running in isolated user process
#[derive(Clone, Copy, Debug)]
pub struct VirtioNetDriver {
    pub device_id: u32,
    pub owner_pid: ProcessId,
    pub mac_address: [u8; 6],
    pub mtu: u16,
    pub is_ready: bool,
    pub link_up: bool,
    pub tx_packets: u64,
    pub rx_packets: u64,
}

impl VirtioNetDriver {
    pub const fn new(device_id: u32, owner_pid: ProcessId, mac_address: [u8; 6]) -> Self {
        Self {
            device_id,
            owner_pid,
            mac_address,
            mtu: DEFAULT_MTU,
            is_ready: false,
            link_up: false,
            tx_packets: 0,
            rx_packets: 0,
        }
    }

    /// Initializes network interface using granted device capability token (INV-DRV-001)
    pub fn init(&mut self, dev_cap: &Capability) -> Result<(), &'static str> {
        if !dev_cap.is_valid || dev_cap.cap_type != CapType::Device(self.device_id) {
            return Err("EPERM: Access denied: Invalid device capability for virtio-net (INV-DRV-001)");
        }
        self.is_ready = true;
        self.link_up = true;
        Ok(())
    }

    /// Transmits an Ethernet frame via DMA window validation (INV-DRV-003)
    pub fn transmit_packet(
        &mut self,
        packet_len: usize,
        dma_window_id: u32,
        dma_broker: &DmaBroker,
        dma_cap: &Capability,
    ) -> Result<usize, &'static str> {
        if !self.is_ready || !self.link_up {
            return Err("ENETDOWN: Network interface is down");
        }
        if packet_len > (self.mtu as usize) + 14 {
            return Err("EMSGSIZE: Packet exceeds interface MTU");
        }

        // Validate DMA buffer bounds (Host to Device) (INV-DRV-003)
        dma_broker.validate_transfer(
            dma_window_id,
            self.owner_pid,
            0,
            packet_len,
            DmaDirection::HostToDevice,
            dma_cap,
        )?;

        self.tx_packets += 1;
        Ok(packet_len)
    }

    /// Ingests a received Ethernet frame via DMA window validation (INV-DRV-003)
    pub fn receive_packet(
        &mut self,
        packet_len: usize,
        dma_window_id: u32,
        dma_broker: &DmaBroker,
        dma_cap: &Capability,
    ) -> Result<usize, &'static str> {
        if !self.is_ready || !self.link_up {
            return Err("ENETDOWN: Network interface is down");
        }

        // Validate DMA buffer bounds (Device to Host) (INV-DRV-003)
        dma_broker.validate_transfer(
            dma_window_id,
            self.owner_pid,
            0,
            packet_len,
            DmaDirection::DeviceToHost,
            dma_cap,
        )?;

        self.rx_packets += 1;
        Ok(packet_len)
    }

    /// Handles an incoming IRQ notification from the IRQ Broker and constructs IRQ_ACK (INV-DRV-002)
    pub fn handle_irq_notification(&mut self, irq_line: u32, irq_cap: &Capability) -> Result<DriverMessage, &'static str> {
        if !irq_cap.is_valid || irq_cap.cap_type != CapType::Interrupt(irq_line) {
            return Err("EPERM: Access denied: Invalid IRQ capability in net driver (INV-DRV-002)");
        }

        // Return IRQ_ACK message to be submitted to IRQ Broker
        Ok(DriverMessage::irq_ack(irq_line, self.rx_packets))
    }
}
