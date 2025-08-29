pub mod error;

use std::{
    alloc::{Layout, LayoutError, alloc, dealloc, handle_alloc_error, realloc},
    mem::size_of,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    usize,
};

use error::GrowError;

use crate::error::InsertError;

pub const MAX_ALLOCATION_SIZE: usize = isize::MAX as _;

pub struct RawDynamicSizeArray<T> {
    elements: NonNull<T>,
    capacity: usize,
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

// accessors
impl<T> RawDynamicSizeArray<T> {
    const fn layout(&self) -> Result<Layout, LayoutError> {
        Layout::array::<T>(self.capacity)
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

            self.capacity *= 2; // [4]
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
        if self.capacity != 0 && Self::ELEMENT_SIZE != 0 {
            if let Ok(layout) = self.layout() {
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
}

pub struct DynamicSizeArray<T> {
    buffer: RawDynamicSizeArray<T>,
    length: usize,
}

// constructors
impl<T> DynamicSizeArray<T> {
    pub const fn new() -> Self {
        Self {
            buffer: RawDynamicSizeArray::new(),
            length: 0,
        }
    }
}

// accessors
impl<T> DynamicSizeArray<T> {
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
    pub const fn is_full(&self) -> bool {
        self.length == self.buffer.capacity
    }
    pub const fn pointer(&self) -> *mut T {
        self.buffer.elements.as_ptr()
    }
    pub const fn capacity(&self) -> usize {
        self.buffer.capacity
    }
    pub const fn length(&self) -> usize {
        self.length
    }
}

// mutators
impl<T> DynamicSizeArray<T> {
    fn grow(&mut self) -> Result<(), GrowError> {
        self.buffer.grow()
    }

    pub fn push_checked(&mut self, element: T) -> Result<(), GrowError> {
        if self.is_full() {
            self.grow()?;
        }

        unsafe {
            let destination = self.buffer.elements.as_ptr().add(self.length);
            // SAFETY:
            // destination is not null because `self.items` is [NotNull]
            // destination was created with [Layout::array] and is therefore properly aligned
            ptr::write(destination, element);
        }

        self.length += 1;

        Ok(())
    }

    pub fn push(&mut self, element: T) {
        self.push_checked(element).unwrap()
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.length -= 1;

            // SAFETY:
            // `self.elements` is [NonNull]. `self.length` will always be a valid offset because `self.length` only increases after an element has been written and it was just decremented
            let popped_element_pointer = unsafe { self.buffer.elements.as_ptr().add(self.length) };

            // SAFETY:
            // `popped_element_pointer` is valid for reads and points to a properly initialized value. see above
            // `popped_element_pointer` is properly aligned because `self.elements` is aligned and `add` keeps alignment
            let popped_element = unsafe { ptr::read(popped_element_pointer) };

            Some(popped_element)
        }
    }

    pub fn insert_checked(&mut self, index: usize, element: T) -> Result<(), InsertError> {
        if index > self.length {
            return Err(InsertError::IndexOutOfBounds);
        }

        if self.is_full() {
            self.grow()?;
        }

        let pointer = self.pointer();

        // SAFETY:
        // index is in bounds
        let target = unsafe { pointer.add(index) };

        // move all elements one to the right
        // SAFETY:
        // target is valid, target + 1 is valid because we grew.
        unsafe {
            ptr::copy(target, target.add(1), self.length - index);
        }

        // write the element
        // SAFETY: target is properly aligned because of invariants of [RawDynamicSizeArray]
        unsafe {
            ptr::write(target, element);
        }

        self.length += 1;

        Ok(())
    }

    pub fn inset(&mut self, index: usize, element: T) {
        self.insert_checked(index, element).unwrap()
    }

    pub fn remove_checked(&mut self, index: usize) -> Option<T> {
        // [1]
        if index >= self.length {
            return None;
        }
        self.length = self.length.checked_sub(1)?;

        // SAFETY:
        // index is inbounds
        let removed_element_pointer = unsafe { self.pointer().add(index) };

        // SAFETY:
        // invariants on [RawDynamicSizeArray]
        let removed_element = unsafe { ptr::read(removed_element_pointer) };

        // shift elements left to fill the hole from the removed element
        // SAFETY:
        // source is valid because index is at most self.length - 1. see [1]
        // destination is valid because  it is in bounds
        unsafe {
            ptr::copy(
                removed_element_pointer.add(1),
                removed_element_pointer,
                self.length - index,
            );
        }

        Some(removed_element)
    }
}

impl<T> Deref for DynamicSizeArray<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY:
            // - `self.raw.elements` is [NonNull] and was created with [Layout::array] so is valid for reads for len * size_of::<T>() bytes,
            //   is properly aligned and The entire memory range of this slice must be contained within a single allocation.
            // - TODO: data must be non-null and aligned even for zero-length slices or slices of ZSTs.
            // - Each call to push/insert ensures that each element is a properly initialized value of type T.
            // - returns an shared reference that can't be mutated.
            // - Every call to [Self::grow] ensures the total size of the slice must be no larger than isize::MAX.
            std::slice::from_raw_parts(self.buffer.elements.as_ptr(), self.length)
        }
    }
}

impl<T> DerefMut for DynamicSizeArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY:
            // - `self.raw.elements` is [NonNull] and was created with [Layout::array] so is valid for reads for len * size_of::<T>() bytes,
            //   is properly aligned and The entire memory range of this slice must be contained within a single allocation.
            // - TODO: data must be non-null and aligned even for zero-length slices or slices of ZSTs.
            // - Each call to push/insert ensures that each element is a properly initialized value of type T.
            // - returns a mutable reference the borrow checker makes sure this is the only point of access and we dent give out any raw pointers.
            // - Every call to [Self::grow] ensures the total size of the slice must be no larger than isize::MAX.
            std::slice::from_raw_parts_mut(self.buffer.elements.as_ptr(), self.length)
        }
    }
}
