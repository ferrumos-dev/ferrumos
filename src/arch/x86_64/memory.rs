//! Boot memory-map validation and the kernel's first reusable heap.
//!
//! The summary describes initial boot-time usable regions. Heap measurements
//! come separately from the locked allocator after its pages have been mapped.
//! Paging and raw memory operations remain inside this architecture adapter.

#[cfg(any(test, target_os = "none"))]
mod frames;
mod heap;
#[cfg(target_os = "none")]
mod paging;

pub use heap::{HeapBlock, HeapError, KernelHeap};
#[cfg(target_os = "none")]
pub use paging::{heap, initialize};

use bootloader_api::info::{MemoryRegion, MemoryRegionKind};

const FRAME_BYTES: u64 = 4096;

/// First virtual byte of the dedicated, page-aligned kernel heap.
pub const HEAP_START: u64 = 0x4444_4444_0000;

/// Virtual bytes reserved for the initial reusable kernel heap (1 MiB).
pub const HEAP_SIZE: usize = 1024 * 1024;

/// Failure to establish exclusive mappings and initialize the kernel heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryInitError {
    /// Bootloader descriptors failed validation before any mapping was changed.
    InvalidMemoryMap(MemoryMapError),
    /// The direct physical-memory offset is unaligned, noncanonical or overflows.
    InvalidPhysicalOffset,
    /// A usable frame cannot be represented by the x86_64 physical-address type.
    InvalidPhysicalAddress,
    /// The chosen virtual heap range already contains a mapping.
    HeapRangeInUse,
    /// Walking an existing page table encountered an invalid physical address.
    InvalidPageTable,
    /// Usable physical frames were exhausted during heap or page-table creation.
    FrameAllocationFailed,
    /// Creating a heap mapping failed after initialization began.
    MappingFailed,
    /// The heap was initialized previously and must retain its live allocations.
    AlreadyInitialized,
    /// An earlier attempt may own partial mappings and cannot safely be retried.
    PreviousInitializationFailed,
}

/// Invalid or unrepresentable physical memory descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMapError {
    /// An exclusive end address precedes its start address.
    ReversedRegion,
    /// Two nonempty regions describe overlapping physical bytes.
    Overlap,
    /// A total cannot be represented in a `u64`.
    Overflow,
}

/// Initial capacity reported as usable by the bootloader, before allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootMemorySummary {
    usable_bytes: u64,
    usable_frames: u64,
}

impl BootMemorySummary {
    /// Validate disjoint regions and count usable bytes and whole 4 KiB frames.
    ///
    /// Empty descriptors are ignored and descriptor order is unrestricted.
    /// Every nonempty region, including reserved ones, must be disjoint. The
    /// pairwise check runs once during boot and needs no sorting or allocation.
    /// Frame counting rounds each descriptor separately without merging adjacent
    /// regions, giving a conservative total when a frame spans two descriptors.
    pub fn from_regions(regions: &[MemoryRegion]) -> Result<Self, MemoryMapError> {
        let mut summary = Self {
            usable_bytes: 0,
            usable_frames: 0,
        };
        for (index, region) in regions.iter().enumerate() {
            let bytes = region
                .end
                .checked_sub(region.start)
                .ok_or(MemoryMapError::ReversedRegion)?;
            if bytes == 0 {
                continue;
            }
            // Compare half-open ranges. Adjoining regions may share an end
            // point, but overlapping bytes would be counted twice or assigned
            // incompatible ownership (for example Usable and Bootloader).
            if regions[..index].iter().any(|previous| {
                previous.start < previous.end
                    && previous.start < region.end
                    && region.start < previous.end
            }) {
                return Err(MemoryMapError::Overlap);
            }
            if region.kind != MemoryRegionKind::Usable {
                continue;
            }
            summary.usable_bytes = summary
                .usable_bytes
                .checked_add(bytes)
                .ok_or(MemoryMapError::Overflow)?;
            // Round the start upward and the exclusive end downward. A partial
            // frame at either edge cannot be handed to a future frame allocator.
            // Division before increment also avoids start + 4095 overflowing.
            let first_frame =
                region.start / FRAME_BYTES + u64::from(region.start % FRAME_BYTES != 0);
            let end_frame = region.end / FRAME_BYTES;
            let frames = end_frame.saturating_sub(first_frame);
            summary.usable_frames = summary
                .usable_frames
                .checked_add(frames)
                .ok_or(MemoryMapError::Overflow)?;
        }
        Ok(summary)
    }

    /// Physical bytes initially classified as usable, including partial frames.
    pub const fn usable_bytes(self) -> u64 {
        self.usable_bytes
    }

    /// Whole 4 KiB frames contained in individual usable descriptors.
    pub const fn usable_frames(self) -> u64 {
        self.usable_frames
    }
}

#[cfg(test)]
mod tests {
    use super::{BootMemorySummary, MemoryMapError};
    use bootloader_api::info::{MemoryRegion, MemoryRegionKind};

    fn usable(start: u64, end: u64) -> MemoryRegion {
        MemoryRegion {
            start,
            end,
            kind: MemoryRegionKind::Usable,
        }
    }

    #[test]
    fn excludes_reserved_bytes_and_partial_frames_in_unordered_regions() {
        let regions = [
            usable(0x8001, 0xb123),
            MemoryRegion {
                start: 0x1000,
                end: 0x3000,
                kind: MemoryRegionKind::Bootloader,
            },
            usable(0x3000, 0x5000),
            MemoryRegion::empty(),
        ];
        let summary = BootMemorySummary::from_regions(&regions).unwrap();
        assert_eq!(summary.usable_bytes(), 0x3122 + 0x2000);
        assert_eq!(summary.usable_frames(), 4);
    }

    #[test]
    fn rejects_overlaps_and_reversed_ranges() {
        assert_eq!(
            BootMemorySummary::from_regions(&[usable(2, 1)]),
            Err(MemoryMapError::ReversedRegion)
        );
        assert_eq!(
            BootMemorySummary::from_regions(&[usable(1, 4), usable(3, 5)]),
            Err(MemoryMapError::Overlap)
        );
        assert_eq!(
            BootMemorySummary::from_regions(&[
                usable(0, 4096),
                MemoryRegion {
                    start: 100,
                    end: 200,
                    kind: MemoryRegionKind::Bootloader
                },
            ]),
            Err(MemoryMapError::Overlap)
        );
    }

    #[test]
    fn empty_maps_and_top_address_rounding_remain_representable() {
        assert_eq!(
            BootMemorySummary::from_regions(&[])
                .unwrap()
                .usable_frames(),
            0
        );
        let summary = BootMemorySummary::from_regions(&[usable(u64::MAX - 1, u64::MAX)]).unwrap();
        assert_eq!(summary.usable_bytes(), 1);
        assert_eq!(summary.usable_frames(), 0);
        let summary =
            BootMemorySummary::from_regions(&[usable(0, 2048), usable(2048, 4096)]).unwrap();
        assert_eq!(summary.usable_bytes(), 4096);
        assert_eq!(summary.usable_frames(), 0);
    }
}
