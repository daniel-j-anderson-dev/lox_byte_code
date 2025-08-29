use std::{
    alloc::{Layout, LayoutError, alloc, dealloc, realloc},
    mem::size_of,
    ptr::NonNull,
};

use crate::error::GrowError;

pub const MAX_ALLOCATION_SIZE: usize = isize::MAX as _;

pub struct RawDynamicSizeArray<T> {
    pub(crate) elements: NonNull<T>,
    pub(crate) capacity: usize,
}

unsafe impl<T: Send> Send for RawDynamicSizeArray<T> {}
unsafe impl<T: Sync> Sync for RawDynamicSizeArray<T> {}

// constructors
impl<T> RawDynamicSizeArray<T> {
    const ELEMENT_SIZE: usize = size_of::<T>();
    const NEW_CAPACITY: usize = if Self::ELEMENT_SIZE == 0 {
        usize::MAX
    } else {
        usize::MIN
    };

    pub const fn new() -> Self {
        Self {
            elements: NonNull::dangling(),
            capacity: Self::NEW_CAPACITY,
        }
    }
}

impl<T> Default for RawDynamicSizeArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

// accessors
impl<T> RawDynamicSizeArray<T> {
    const fn layout(&self) -> Result<Layout, LayoutError> {
        Layout::array::<T>(self.capacity)
    }
    const fn larger_capacity(&self) -> usize {
        if self.capacity < 8 {
            8
        } else {
            self.capacity * 2
        }
    }
}

// mutators
impl<T> RawDynamicSizeArray<T> {
    /// Extend capacity by doubling or adding 1 at first call.
    pub fn grow(&mut self) -> Result<(), GrowError> {
        // [3]
        let (new_layout, new_pointer) = if self.capacity == 0 {
            self.capacity = 1;
            let new_layout = self.layout()?;
            // SAFETY: new_capacity == 1
            let new_pointer = unsafe { alloc(new_layout) }; // [1]

            (new_layout, new_pointer)
        } else {
            let old_layout = self.layout()?;
            let old_pointer = self.elements.as_ptr() as _;

            self.capacity = self.larger_capacity(); // [4]
            let new_layout = self
                .layout()
                .map_err(GrowError::Layout)
                .and_then(|new_capacity| {
                    //[5]
                    if new_capacity.size() <= MAX_ALLOCATION_SIZE {
                        Ok(new_capacity)
                    } else {
                        Err(GrowError::AllocationTooLarge)
                    }
                })?;

            // SAFETY:
            // `old_pointer` was allocated with [alloc::alloc] using the same global allocator
            // `old_layout` was use to allocate and is therefore the same as the size used to allocate. see [1], [2]
            // `new_layout.size()` is unsigned and not 0. see [3], [4]
            // `new_layout.size()` <= [isize::MAX]. see [5]
            let new_pointer = unsafe { realloc(old_pointer, old_layout, new_layout.size()) }; // [2]

            (new_layout, new_pointer)
        };

        self.elements =
            NonNull::new(new_pointer as _).ok_or(GrowError::AllocationFail(new_layout))?;

        Ok(())
    }
}

impl<T> Drop for RawDynamicSizeArray<T> {
    fn drop(&mut self) {
        if self.capacity != 0
            && Self::ELEMENT_SIZE != 0
            && let Ok(layout) = self.layout()
        {
            unsafe {
                // SAFETY:
                // `self.elements` was allocated by the global allocator so can be deallocated by the global allocator.
                // `layout` is the same use to deallocate because is exactly the same [Layout] that was used for that allocation, because
                //  we always compute it with `Layout::array::<T>(self.capacity)`.
                dealloc(self.elements.as_ptr() as _, layout);
            }
        }
    }
}
