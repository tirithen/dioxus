use crate::{
    error::{AlreadyBorrowedError, AlreadyBorrowedMutError},
    innerlude::*,
};

/// Information about the borrow state of a memory location.
#[derive(Debug, Default)]
pub struct MemoryLocationBorrowInfo {
    pub(crate) borrowed_at: parking_lot::RwLock<Vec<&'static std::panic::Location<'static>>>,
    pub(crate) borrowed_mut_at: parking_lot::RwLock<Option<&'static std::panic::Location<'static>>>,
}

impl MemoryLocationBorrowInfo {
    pub fn borrow_mut_error(&self) -> BorrowMutError {
        if let Some(borrowed_mut_at) = self.borrowed_mut_at.read().as_ref() {
            BorrowMutError::AlreadyBorrowedMut(AlreadyBorrowedMutError { borrowed_mut_at })
        } else {
            BorrowMutError::AlreadyBorrowed(AlreadyBorrowedError {
                borrowed_at: self.borrowed_at.read().clone(),
            })
        }
    }

    pub fn borrow_error(&self) -> BorrowError {
        BorrowError::AlreadyBorrowedMut(AlreadyBorrowedMutError {
            borrowed_mut_at: self.borrowed_mut_at.read().unwrap(),
        })
    }
}
