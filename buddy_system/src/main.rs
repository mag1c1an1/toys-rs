use std::{
    alloc::Layout,
    array,
    cmp::{max, min},
    collections::BTreeSet,
    ops::Range,
};

/// An extension trait for Rust integer types, including `u8`, `u16`, `u32`,
/// `u64`, and `usize`, to provide methods to make integers aligned to a
/// power of two.
pub trait AlignExt {
    /// Returns whether the number is a power of two
    fn is_power_of_two(&self) -> bool;

    /// Returns to the smallest number that is greater than or equal to
    /// `self` and is a multiple of the given power of two.
    ///
    /// The method panics if `power_of_two` is not a
    /// power of two or is smaller than 2 or the calculation overflows
    /// because `self` is too large.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::align_ext::AlignExt;
    /// assert_eq!(12usize.align_up(2), 12);
    /// assert_eq!(12usize.align_up(4), 12);
    /// assert_eq!(12usize.align_up(8), 16);
    /// assert_eq!(12usize.align_up(16), 16);
    /// ```
    fn align_up(self, power_of_two: Self) -> Self;

    /// Returns to the greatest number that is smaller than or equal to
    /// `self` and is a multiple of the given power of two.
    ///
    /// The method panics if `power_of_two` is not a
    /// power of two or is smaller than 2 or the calculation overflows
    /// because `self` is too large. In release mode,
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::align_ext::AlignExt;
    /// assert_eq!(12usize.align_down(2), 12);
    /// assert_eq!(12usize.align_down(4), 12);
    /// assert_eq!(12usize.align_down(8), 8);
    /// assert_eq!(12usize.align_down(16), 0);
    /// ```
    fn align_down(self, power_of_two: Self) -> Self;
}

macro_rules! impl_align_ext {
    ($( $uint_type:ty ),+,) => {
        $(
            impl AlignExt for $uint_type {
                #[inline]
                fn is_power_of_two(&self) -> bool {
                    (*self != 0) && ((*self & (*self - 1)) == 0)
                }

                #[inline]
                fn align_up(self, align: Self) -> Self {
                    assert!(align.is_power_of_two() && align >= 2);
                    self.checked_add(align - 1).unwrap() & !(align - 1)
                }

                #[inline]
                fn align_down(self, align: Self) -> Self {
                    assert!(align.is_power_of_two() && align >= 2);
                    self & !(align - 1)
                }
            }
        )*
    }
}

impl_align_ext! {
    u8,
    u16,
    u32,
    u64,
    usize,
}

fn main() {
    let regions = vec![
        Range {
            start: 0x0,
            end: 0x9fc00,
        },
        Range {
            start: 0x100000,
            end: 0x8000000,
        },
    ];
    let mut allocator = FrameAllocator::<32>::new();
    for region in regions.iter() {
        // Make the memory region page-aligned, and skip if it is too small.
        const PAGE_SIZE: usize = 4096;
        let start = region.start.align_up(PAGE_SIZE) / PAGE_SIZE;
        let end = region.end.align_down(PAGE_SIZE) / PAGE_SIZE;
        if end <= start {
            continue;
        }
        // Add global free pages to the frame allocator.
        allocator.add_frame(start, end);
    }
    println!("{:?}", allocator);
}
pub(crate) fn prev_power_of_two(num: usize) -> usize {
    1 << (usize::BITS as usize - num.leading_zeros() as usize - 1)
}

#[derive(Debug)]
pub struct FrameAllocator<const ORDER: usize = 32> {
    // buddy system with max order of ORDER
    free_list: [BTreeSet<usize>; ORDER],

    // statistics
    allocated: usize,
    total: usize,
}

impl<const ORDER: usize> FrameAllocator<ORDER> {
    /// Create an empty frame allocator
    pub fn new() -> Self {
        Self {
            free_list: array::from_fn(|_| BTreeSet::default()),
            allocated: 0,
            total: 0,
        }
    }

    /// Add a range of frame number [start, end) to the allocator
    pub fn add_frame(&mut self, start: usize, end: usize) {
        assert!(start <= end);

        let mut total = 0;
        let mut current_start = start;

        while current_start < end {
            let lowbit = if current_start > 0 {
                current_start & (!current_start + 1)
            } else {
                32
            };
            let size = min(
                min(lowbit, prev_power_of_two(end - current_start)),
                1 << (ORDER - 1),
            );
            total += size;

            self.free_list[size.trailing_zeros() as usize].insert(current_start);
            current_start += size;
        }

        self.total += total;
    }

    /// Add a range of frames to the allocator.
    pub fn insert(&mut self, range: Range<usize>) {
        self.add_frame(range.start, range.end);
    }

    /// Allocate a range of frames from the allocator, returning the first frame of the allocated
    /// range.
    pub fn alloc(&mut self, count: usize) -> Option<usize> {
        let size = count.next_power_of_two();
        self.alloc_power_of_two(size)
    }

    /// Allocate a range of frames with the given size and alignment from the allocator, returning
    /// the first frame of the allocated range.
    /// The allocated size is the maximum of the next power of two of the given size and the
    /// alignment.
    pub fn alloc_aligned(&mut self, layout: Layout) -> Option<usize> {
        let size = max(layout.size().next_power_of_two(), layout.align());
        self.alloc_power_of_two(size)
    }

    /// Allocate a range of frames of the given size from the allocator. The size must be a power of
    /// two. The allocated range will have alignment equal to the size.
    fn alloc_power_of_two(&mut self, size: usize) -> Option<usize> {
        let class = size.trailing_zeros() as usize;
        for i in class..self.free_list.len() {
            // Find the first non-empty size class
            if !self.free_list[i].is_empty() {
                // Split buffers
                for j in (class + 1..i + 1).rev() {
                    if let Some(block_ref) = self.free_list[j].iter().next() {
                        let block = *block_ref;
                        self.free_list[j - 1].insert(block + (1 << (j - 1)));
                        self.free_list[j - 1].insert(block);
                        self.free_list[j].remove(&block);
                    } else {
                        return None;
                    }
                }

                let result = self.free_list[class].iter().next();
                if let Some(result_ref) = result {
                    let result = *result_ref;
                    self.free_list[class].remove(&result);
                    self.allocated += size;
                    return Some(result);
                } else {
                    return None;
                }
            }
        }
        None
    }

    /// Deallocate a range of frames [frame, frame+count) from the frame allocator.
    ///
    /// The range should be exactly the same when it was allocated, as in heap allocator
    pub fn dealloc(&mut self, start_frame: usize, count: usize) {
        let size = count.next_power_of_two();
        self.dealloc_power_of_two(start_frame, size)
    }

    /// allocated size
    pub fn allocated(&self) -> usize {
        self.allocated
    }

    /// total size
    pub fn total(&self) -> usize {
        self.total
    }
    /// Deallocate a range of frames which was previously allocated by [`alloc_aligned`].
    ///
    /// The layout must be exactly the same as when it was allocated.
    pub fn dealloc_aligned(&mut self, start_frame: usize, layout: Layout) {
        let size = max(layout.size().next_power_of_two(), layout.align());
        self.dealloc_power_of_two(start_frame, size)
    }

    /// Deallocate a range of frames with the given size from the allocator. The size must be a
    /// power of two.
    fn dealloc_power_of_two(&mut self, start_frame: usize, size: usize) {
        let class = size.trailing_zeros() as usize;

        // Merge free buddy lists
        let mut current_ptr = start_frame;
        let mut current_class = class;
        while current_class < self.free_list.len() {
            let buddy = current_ptr ^ (1 << current_class);
            if self.free_list[current_class].remove(&buddy) {
                // Free buddy found
                current_ptr = min(current_ptr, buddy);
                current_class += 1;
            } else {
                self.free_list[current_class].insert(current_ptr);
                break;
            }
        }

        self.allocated -= size;
    }
}
