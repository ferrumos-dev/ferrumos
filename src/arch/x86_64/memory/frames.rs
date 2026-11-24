//! Allocation-free enumeration of disjoint, whole usable physical frames.

use super::{BootMemorySummary, MemoryMapError, FRAME_BYTES};
use bootloader_api::info::{MemoryRegion, MemoryRegionKind};

/// A cursor over boot descriptors, preserving their ownership boundaries.
///
/// Enumerating addresses is separate from claiming physical ownership. The
/// paging adapter's unsafe initialization contract guarantees that these usable
/// frames are exclusive before it implements the hardware frame-allocator trait.
pub(super) struct FrameCursor<'a> {
    regions: &'a [MemoryRegion],
    next_region: usize,
    next_frame: u64,
    end_frame: u64,
}

impl<'a> FrameCursor<'a> {
    pub(super) fn new(regions: &'a [MemoryRegion]) -> Result<Self, MemoryMapError> {
        BootMemorySummary::from_regions(regions)?;
        Ok(Self {
            regions,
            next_region: 0,
            next_frame: 0,
            end_frame: 0,
        })
    }
}

impl Iterator for FrameCursor<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.next_frame < self.end_frame {
                let address = self.next_frame * FRAME_BYTES;
                self.next_frame += 1;
                return Some(address);
            }
            let region = self.regions.get(self.next_region)?;
            self.next_region += 1;
            if region.kind != MemoryRegionKind::Usable {
                continue;
            }
            // Work in frame indices so rounding a start near u64::MAX never
            // overflows. The exclusive end excludes each partial final frame.
            self.next_frame =
                region.start / FRAME_BYTES + u64::from(region.start % FRAME_BYTES != 0);
            self.end_frame = region.end / FRAME_BYTES;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usable(start: u64, end: u64) -> MemoryRegion {
        MemoryRegion {
            start,
            end,
            kind: MemoryRegionKind::Usable,
        }
    }

    #[test]
    fn enumerates_whole_usable_frames_once_in_descriptor_order() {
        let regions = [
            usable(0x8001, 0xb123),
            MemoryRegion {
                start: 0x1000,
                end: 0x3000,
                kind: MemoryRegionKind::Bootloader,
            },
            usable(0x3000, 0x5000),
        ];
        let mut frames = FrameCursor::new(&regions).unwrap();
        for address in [0x9000, 0xa000, 0x3000, 0x4000] {
            assert_eq!(frames.next(), Some(address));
        }
        assert_eq!(frames.next(), None);
        assert_eq!(frames.next(), None);
    }

    #[test]
    fn never_combines_partial_frames_from_separate_descriptors() {
        let regions = [usable(0, 2048), usable(2048, 4096)];
        assert_eq!(FrameCursor::new(&regions).unwrap().next(), None);
        let regions = [usable(u64::MAX - 1, u64::MAX)];
        assert_eq!(FrameCursor::new(&regions).unwrap().next(), None);
    }

    #[test]
    fn rejects_a_map_that_could_assign_a_frame_twice() {
        assert!(matches!(
            FrameCursor::new(&[usable(0, 8192), usable(4096, 12288)]),
            Err(MemoryMapError::Overlap)
        ));
        assert!(matches!(
            FrameCursor::new(&[usable(8192, 4096)]),
            Err(MemoryMapError::ReversedRegion)
        ));
    }
}
