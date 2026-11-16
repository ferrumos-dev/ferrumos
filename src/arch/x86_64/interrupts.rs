//! Reserved for x86_64 exception, IRQ and timer integration.
//!
//! This adapter will own interrupt-table setup, masking, acknowledgement and
//! delivery of timer events to the portable core. Interrupt handlers must
//! define their reentrancy and synchronization rules and keep work bounded.
//!
//! No interrupt tables, controller setup or handlers are implemented yet.
