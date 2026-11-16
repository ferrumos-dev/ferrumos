//! Reserved for x86_64 memory integration.
//!
//! Boot memory-map handling, frame allocation, page tables and heap setup
//! belong here. The adapter must define mapping and ownership invariants and
//! capture allocator measurements coherently for the portable contracts.
//!
//! No memory mappings, allocator or isolation guarantees are implemented yet.
