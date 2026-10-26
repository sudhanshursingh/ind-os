//! Trusted Consent Broker Subsystem (Consent Broker v1)
//! Compliant with Bible §8.9 (SP-21), §10, §14 and Invariant INV-CON-001.

pub mod broker;
pub mod record;
pub mod surface;

use crate::sync::spinlock::Spinlock;
pub use broker::{ConsentBroker, MAX_CONSENT_RECORDS};
pub use record::{ConsentDecision, ConsentRecord};
pub use surface::{IsolatedSurface, PlaneId};

/// Global Consent Broker instance protected by spinlock
pub static CONSENT_BROKER: Spinlock<ConsentBroker> = Spinlock::new(ConsentBroker::new());
