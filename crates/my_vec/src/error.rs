use core::{error::Error, fmt::Display};
use std::alloc::{Layout, LayoutError};

#[derive(Debug)]
pub enum GrowError {
    Layout(LayoutError),
    AllocationTooLarge,
    AllocationFail(Layout),
}
impl Display for GrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for GrowError {}
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
impl Display for InsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for InsertError {}
impl From<GrowError> for InsertError {
    fn from(value: GrowError) -> Self {
        Self::Grow(value)
    }
}
