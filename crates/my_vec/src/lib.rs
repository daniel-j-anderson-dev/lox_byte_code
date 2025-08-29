use std::{
    alloc::{self, Layout, LayoutError},
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
}

// mutators
impl<T> DynamicSizeArray<T> {
    /// Extend capacity by doubling or adding 1 at first call.
    pub fn grow(&mut self) -> Result<(), Error> {
        // [3]
        let (new_capacity, new_pointer) = if self.capacity == 0 {
            let new_capacity = 1;
            let new_layout = Layout::array::<T>(new_capacity)?;
            // SAFETY: new_capacity == 1
            let new_pointer = unsafe { alloc::alloc(new_layout) }; // [1]

            (new_capacity, new_pointer)
        } else {
            let new_capacity = 2 * self.capacity; // [4]
            let new_layout = Layout::array::<T>(new_capacity)
                .map_err(Error::Layout)
                .and_then(|new_capacity| {
                    if new_capacity.size() <= MAX_ALLOCATION_SIZE {
                        //[5]
                        Ok(new_capacity)
                    } else {
                        Err(Error::AllocationTooLarge)
                    }
                })?;

            let old_layout = Layout::array::<T>(self.capacity)?;
            let old_pointer = self.elements.as_ptr() as _;
            // SAFETY:
            // `old_pointer` was allocated with [alloc::alloc] using the same global allocator
            // `old_layout` was use to allocate and is therefore the same as the size used to allocate. see [1], [2]
            // `new_layout.size()` is unsigned and not 0. see [3], [4]
            // `new_layout.size()` <= [isize::MAX]. see [5]
            let new_pointer = unsafe { alloc::realloc(old_pointer, old_layout, new_layout.size()) }; // [2]

            (new_capacity, new_pointer)
        };

        self.elements = NonNull::new(new_pointer as _).ok_or(Error::NewPointerIsNull)?;
        self.capacity = new_capacity;

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
            // `self.elements` is [NonNull]. `self.length` will always be a valid offset because `self.length` only increases after an element has been written
            let popped_element_pointer = unsafe { self.elements.as_ptr().add(self.length) };

            // SAFETY:
            // `popped_element_pointer` is valid for reads and points to a properly initialized value. see above
            // `popped_element_pointer` is properly aligned because `self.elements` is aligned and `add` keeps alignment
            let popped_element = unsafe { ptr::read(popped_element_pointer) };

            Some(popped_element)
        }
    }
}

pub enum Error {
    Layout(LayoutError),
    AllocationTooLarge,
    NewPointerIsNull,
}
impl From<LayoutError> for Error {
    fn from(value: LayoutError) -> Self {
        Self::Layout(value)
    }
}
