use std::{
    alloc::{self, Layout, LayoutError},
    mem::size_of,
    ptr::NonNull,
};

pub const MAX_ALLOCATION_SIZE: usize = isize::MAX as _;

pub struct DynamicSizeArray<T> {
    items: NonNull<T>,
    length: usize,
    capacity: usize,
}

unsafe impl<T: Send> Send for DynamicSizeArray<T> {}
unsafe impl<T: Sync> Sync for DynamicSizeArray<T> {}

impl<T> DynamicSizeArray<T> {
    pub const fn new() -> Self {
        assert!(size_of::<T>() != 0, "ZST not implemented");
        // TODO: ensure size of T < isize::MAX
        Self {
            items: NonNull::dangling(),
            length: 0,
            capacity: 0,
        }
    }

    pub fn grow(&mut self) -> Result<(), Error> {
        let (new_capacity, new_layout, new_pointer) = if self.capacity == 0 { // [3]
            let new_capacity = 1;
            let new_layout = Layout::array::<T>(new_capacity)?;
            // SAFETY: new_capacity == 1
            let new_pointer = unsafe { alloc::alloc(new_layout) }; // [1]

            (new_capacity, new_layout, new_pointer)
        } else {
            let new_capacity = 2 * self.capacity; // [4]
            let new_layout = Layout::array::<T>(new_capacity)
                .map_err(Error::Layout)
                .and_then(|new_capacity| {
                    if new_capacity.size() <= MAX_ALLOCATION_SIZE { //[5]
                        Ok(new_capacity)
                    } else {
                        Err(Error::AllocationTooLarge)
                    }
                })?;

            let old_layout = Layout::array::<T>(self.capacity)?;
            let old_pointer = self.items.as_ptr() as _;
            // SAFETY:
            // `old_pointer` was allocated with [alloc::alloc] using the same global allocator
            // `old_layout` was use to allocate. see [1], [2]
            // `new_layout.size()` is unsigned and not 0. see [3], [4]
            // `new_layout.size()` <= [isize::MAX]. see [5]
            let new_pointer = unsafe { alloc::realloc(old_pointer, old_layout, new_layout.size()) }; // [2]

            (new_capacity, new_layout, new_pointer)
        };

        self.items = NonNull::new(new_pointer as _).ok_or(Error::NewPointerIsNull)?;
        self.capacity = new_capacity;

        Ok(())
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
