//! Reserved for portable timer event contracts.
//!
//! The documented `TimerTickSink` will live here. A call made from an IRQ
//! handler must perform bounded, non-blocking work.
//!
//! No event sink is implemented yet. Interrupt tables, masking and controller
//! acknowledgement belong to the x86_64 adapter.
