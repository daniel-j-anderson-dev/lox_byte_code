use std::ptr::NonNull;

pub struct DynamicSizeArray<T> {
    items: NonNull<T>,
    length: usize,
    capacity: usize,
}