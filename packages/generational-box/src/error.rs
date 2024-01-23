use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug, Clone)]
/// An error that can occur when trying to borrow a value.
pub enum BorrowError {
    /// The value was dropped.
    Dropped(ValueDroppedError),

    /// The value was already borrowed mutably.
    AlreadyBorrowedMut(AlreadyBorrowedMutError),
}
impl Error for BorrowError {}

impl Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BorrowError::Dropped(error) => Display::fmt(error, f),
            BorrowError::AlreadyBorrowedMut(error) => Display::fmt(error, f),
        }
    }
}

#[derive(Debug, Clone)]
/// An error that can occur when trying to borrow a value mutably.
pub enum BorrowMutError {
    /// The value was dropped.
    Dropped(ValueDroppedError),
    /// The value was already borrowed.
    AlreadyBorrowed(AlreadyBorrowedError),
    /// The value was already borrowed mutably.
    AlreadyBorrowedMut(AlreadyBorrowedMutError),
}

impl Error for BorrowMutError {}
impl Display for BorrowMutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BorrowMutError::Dropped(error) => Display::fmt(error, f),
            BorrowMutError::AlreadyBorrowedMut(error) => Display::fmt(error, f),
            BorrowMutError::AlreadyBorrowed(error) => Display::fmt(error, f),
        }
    }
}

/// An error that can occur when trying to use a value that has been dropped.
#[derive(Debug, Copy, Clone)]
pub struct ValueDroppedError {
    pub(crate) created_at: &'static std::panic::Location<'static>,
}
impl Error for ValueDroppedError {}
impl Display for ValueDroppedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to borrow because the value was dropped.")?;
        f.write_fmt(format_args!("created_at: {}", self.created_at))?;
        Ok(())
    }
}

/// An error that can occur when trying to borrow a value that has already been borrowed mutably.
#[derive(Debug, Copy, Clone)]
pub struct AlreadyBorrowedMutError {
    pub(crate) borrowed_mut_at: &'static std::panic::Location<'static>,
}
impl Error for AlreadyBorrowedMutError {}
impl Display for AlreadyBorrowedMutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to borrow because the value was already borrowed mutably.")?;
        f.write_fmt(format_args!("borrowed_mut_at: {}", self.borrowed_mut_at))?;
        Ok(())
    }
}

/// An error that can occur when trying to borrow a value mutably that has already been borrowed immutably.
#[derive(Debug, Clone)]
pub struct AlreadyBorrowedError {
    pub(crate) borrowed_at: Vec<&'static std::panic::Location<'static>>,
}
impl Error for AlreadyBorrowedError {}
impl Display for AlreadyBorrowedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to borrow mutably because the value was already borrowed immutably.")?;
        f.write_str("borrowed_at:")?;
        for location in self.borrowed_at.iter() {
            f.write_fmt(format_args!("\t{}", location))?;
        }
        Ok(())
    }
}
