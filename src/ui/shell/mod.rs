//! Shell input parsing, command dispatch and textual feedback.
//!
//! Commands will use kernel reports and service APIs. This layer must not
//! access page tables, hardware registers or allocator internals directly.
