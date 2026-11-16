//! Reserved for x86_64 CPU state and context-switch mechanisms.
//!
//! Descriptor tables, task stacks and register save/restore operations belong
//! here. Task selection remains in the portable scheduler; task state and
//! context ownership remain in the executor.
//!
//! No CPU setup, halt routine or context-switch assembly is implemented yet.
