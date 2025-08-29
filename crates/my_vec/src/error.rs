use std::alloc::{Layout, LayoutError};

#[derive(Debug)]
pub enum GrowError {
    Layout(LayoutError),
    AllocationTooLarge,
    AllocationFail(Layout),
}
impl From<LayoutError> for GrowError {
    fn from(value: LayoutError) -> Self {
        Self::Layout(value)
    }
}
impl From<Layout> for GrowError {
    fn from(value: Layout) -> Self {
        Self::AllocationFail(value)
    }
}

#[derive(Debug)]
pub enum InsertError {
    Grow(GrowError),
    IndexOutOfBounds,
}
impl From<GrowError> for InsertError {
    fn from(value: GrowError) -> Self {
        Self::Grow(value)
    }
}
