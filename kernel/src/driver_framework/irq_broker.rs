//! IndOS User-Mode IRQ Broker
//! Compliant with Bible §8.10 (SP-26), INDOS-SPEC-DRV-01, Rule N-01, Rule N-04, and Invariant INV-DRV-002.

use crate::capability::{CapRights, CapType, Capability};
use crate::task::ProcessId;

pub const MAX_IRQS: usize = 16;

/// State of an interrupt line managed by the broker
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqState {
    Inactive,
    Subscribed,
    Pending,
    Servicing,
}

/// Active interrupt subscription mapping hardware IRQ line to driver endpoint
#[derive(Clone, Copy, Debug)]
pub struct IrqSubscription {
    pub irq_line: u32,
    pub driver_pid: ProcessId,
    pub endpoint_id: u32,
    pub state: IrqState,
    pub masked: bool,
    pub delivery_count: u64,
    pub ack_count: u64,
}

impl IrqSubscription {
    pub const fn empty() -> Self {
        Self {
            irq_line: 0,
            driver_pid: 0,
            endpoint_id: 0,
            state: IrqState::Inactive,
            masked: true,
            delivery_count: 0,
            ack_count: 0,
        }
    }
}

/// Microkernel IRQ Broker routing hardware interrupts to isolated user-mode driver endpoints
pub struct IrqBroker {
    pub subscriptions: [IrqSubscription; MAX_IRQS],
    pub subscription_count: usize,
}

impl IrqBroker {
    pub const fn new() -> Self {
        Self {
            subscriptions: [IrqSubscription::empty(); MAX_IRQS],
            subscription_count: 0,
        }
    }

    /// Subscribes a user-mode driver to an IRQ line via capability authentication (INV-DRV-002)
    pub fn subscribe(
        &mut self,
        irq_line: u32,
        driver_pid: ProcessId,
        endpoint_id: u32,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        // Enforce capability authorization
        if !cap.is_valid {
            return Err("EPERM: Access denied: Invalid capability token (INV-DRV-002)");
        }
        if cap.cap_type != CapType::Interrupt(irq_line) {
            return Err("EPERM: Access denied: Capability does not grant access to this IRQ line (INV-DRV-002)");
        }
        if !cap.rights.contains(CapRights::READ) {
            return Err("EPERM: Access denied: Capability lacks READ/RECEIVE rights (INV-DRV-002)");
        }

        // Check if already subscribed
        for i in 0..self.subscription_count {
            if self.subscriptions[i].irq_line == irq_line {
                return Err("EBUSY: IRQ line already subscribed");
            }
        }

        if self.subscription_count >= MAX_IRQS {
            return Err("ENOMEM: IRQ broker subscription table full");
        }

        let sub = IrqSubscription {
            irq_line,
            driver_pid,
            endpoint_id,
            state: IrqState::Subscribed,
            masked: false,
            delivery_count: 0,
            ack_count: 0,
        };

        self.subscriptions[self.subscription_count] = sub;
        self.subscription_count += 1;
        Ok(())
    }

    /// Traps hardware interrupt at EL1, masks line at distributor, and delivers notification (INV-DRV-002)
    pub fn dispatch_irq(&mut self, irq_line: u32) -> Result<u32, &'static str> {
        for i in 0..self.subscription_count {
            let sub = &mut self.subscriptions[i];
            if sub.irq_line == irq_line {
                if sub.state == IrqState::Inactive {
                    return Err("ENOENT: IRQ subscription inactive");
                }

                // Mask interrupt line to prevent interrupt storm while driver services device
                sub.masked = true;
                sub.state = IrqState::Pending;
                sub.delivery_count += 1;
                return Ok(sub.endpoint_id);
            }
        }
        Err("ENODEV: No driver subscribed to this IRQ")
    }

    /// Authenticates driver acknowledgment and unmasks interrupt line (INV-DRV-002)
    pub fn acknowledge_irq(
        &mut self,
        irq_line: u32,
        driver_pid: ProcessId,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        // Enforce INV-DRV-002: Unauthenticated unmask rejected
        if !cap.is_valid {
            return Err("EPERM: Access denied: Presented capability token is invalid (INV-DRV-002)");
        }
        if cap.cap_type != CapType::Interrupt(irq_line) {
            return Err("EPERM: Access denied: Capability does not authorize acknowledging this IRQ (INV-DRV-002)");
        }
        if !cap.rights.contains(CapRights::WRITE) {
            return Err("EPERM: Access denied: Capability lacks WRITE/ACK rights (INV-DRV-002)");
        }

        for i in 0..self.subscription_count {
            let sub = &mut self.subscriptions[i];
            if sub.irq_line == irq_line {
                if sub.driver_pid != driver_pid {
                    return Err("EPERM: Caller is not the registered subscriber for this IRQ (INV-DRV-002)");
                }

                // Unmask line and return to Subscribed state
                sub.masked = false;
                sub.state = IrqState::Subscribed;
                sub.ack_count += 1;
                return Ok(());
            }
        }
        Err("ENODEV: IRQ subscription not found")
    }

    /// Unsubscribes driver from IRQ line
    pub fn unsubscribe(
        &mut self,
        irq_line: u32,
        driver_pid: ProcessId,
        cap: &Capability,
    ) -> Result<(), &'static str> {
        if !cap.is_valid || cap.cap_type != CapType::Interrupt(irq_line) {
            return Err("EPERM: Invalid IRQ capability");
        }

        for i in 0..self.subscription_count {
            if self.subscriptions[i].irq_line == irq_line {
                if self.subscriptions[i].driver_pid != driver_pid {
                    return Err("EPERM: Caller is not the registered subscriber");
                }
                self.subscriptions[i].state = IrqState::Inactive;
                self.subscriptions[i].masked = true;
                return Ok(());
            }
        }
        Err("ENODEV: IRQ subscription not found")
    }
}
