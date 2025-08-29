use std::{
    alloc::{Layout, LayoutError, alloc, dealloc, handle_alloc_error, realloc},
    mem::size_of,
    ptr::{self, NonNull},
};

pub const MAX_ALLOCATION_SIZE: usize = isize::MAX as _;

pub struct DynamicSizeArray<T> {
    elements: NonNull<T>,
    length: usize,
    capacity: usize,
}

unsafe impl<T: Send> Send for DynamicSizeArray<T> {}
unsafe impl<T: Sync> Sync for DynamicSizeArray<T> {}

// constructors
impl<T> DynamicSizeArray<T> {
    pub const fn new() -> Self {
        assert!(size_of::<T>() != 0, "ZST not implemented");
        // TODO: ensure size of T < isize::MAX
        Self {
            elements: NonNull::dangling(),
            length: 0,
            capacity: 0,
        }
    }
}

// accessors
impl<T> DynamicSizeArray<T> {
    const fn is_empty(&self) -> bool {
        self.length == 0
    }

    const fn is_full(&self) -> bool {
        self.length == self.capacity
    }

    const fn layout(&self) -> Result<Layout, LayoutError> {
        Layout::array::<T>(self.capacity)
    }
}

// mutators
impl<T> DynamicSizeArray<T> {
    /// Extend capacity by doubling or adding 1 at first call.
    pub fn grow(&mut self) -> Result<(), Error> {
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
            
            self.capacity = 2 * self.capacity; // [4]
            let new_layout = self
                .layout()
                .map_err(Error::Layout)
                .and_then(|new_capacity| {
                    if new_capacity.size() <= MAX_ALLOCATION_SIZE {
                        //[5]
                        Ok(new_capacity)
                    } else {
                        Err(Error::AllocationTooLarge)
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

        self.elements = NonNull::new(new_pointer as _).ok_or(Error::AllocationFail(new_layout))?;

        Ok(())
    }

    pub fn push(&mut self, element: T) -> Result<(), Error> {
        if self.is_full() {
            self.grow()?;
        }

        unsafe {
            let destination = self.elements.as_ptr().add(self.length);
            // SAFETY:
            // destination is not null because `self.items` is [NotNull]
            // destination was created with [Layout::array] and is therefore properly aligned
            ptr::write(destination, element);
        }

        self.length += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.length -= 1;

            // SAFETY:
            // `self.elements` is [NonNull]. `self.length` will always be a valid offset because `self.length` only increases after an element has been written and it was just decremented
            let popped_element_pointer = unsafe { self.elements.as_ptr().add(self.length) };

            // SAFETY:
            // `popped_element_pointer` is valid for reads and points to a properly initialized value. see above
            // `popped_element_pointer` is properly aligned because `self.elements` is aligned and `add` keeps alignment
            let popped_element = unsafe { ptr::read(popped_element_pointer) };

            Some(popped_element)
        }
    }
}
impl<T> Drop for DynamicSizeArray<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            while let Some(_) = self.pop() {}

            if let Ok(layout) = self.layout() {
                unsafe {
                    // SAFETY:
                    // `self.elements` was allocated by the global allocator so can be deallocated by the global allocator.
                    // `layout` is the same use to deallocate because is exactly the same [Layout] that was used for that allocation, because
                    //   we always compute it with `Layout::array::<T>(self.capacity)`.
                    dealloc(self.elements.as_ptr() as _, layout);
                }
            }
        }
    }
}

pub enum Error {
    Layout(LayoutError),
    AllocationTooLarge,
    AllocationFail(Layout),
}
impl From<LayoutError> for Error {
    fn from(value: LayoutError) -> Self {
        Self::Layout(value)
    }
}
