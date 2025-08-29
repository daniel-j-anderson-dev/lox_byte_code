use std::ptr::NonNull;

pub struct DynamicSizeArray<T> {
    items: NonNull<T>,
    length: usize,
    capacity: usize,
}

unsafe impl<T: Send> Send for DynamicSizeArray<T> {}
unsafe impl<T: Sync> Sync for DynamicSizeArray<T> {}
