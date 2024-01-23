use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use crate::innerlude::MemoryLocationBorrowInfo;

/// A reference to a value in a generational box.
pub struct GenerationalRef<R> {
    pub(crate) inner: R,
    pub(crate) borrow: GenerationalRefBorrowInfo,
}

impl<R> GenerationalRef<R> {
    pub(crate) fn new(inner: R, borrow: GenerationalRefBorrowInfo) -> Self {
        Self { inner, borrow }
    }
}

impl<T: ?Sized + Debug, R: Deref<Target = T>> Debug for GenerationalRef<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.deref().fmt(f)
    }
}

impl<T: ?Sized + Display, R: Deref<Target = T>> Display for GenerationalRef<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.deref().fmt(f)
    }
}

impl<T: ?Sized + 'static, R: Deref<Target = T>> Deref for GenerationalRef<R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

/// Information about a borrow.
///
/// WHen compiled with `debug_assertions` or the `debug_borrows` feature, this struct will contain nothing, making it zero-sized.
pub struct GenerationalRefBorrowInfo {
    pub(crate) borrowed_at: &'static std::panic::Location<'static>,
    pub(crate) borrowed_from: &'static MemoryLocationBorrowInfo,
    pub(crate) created_at: &'static std::panic::Location<'static>,
}

impl Drop for GenerationalRefBorrowInfo {
    fn drop(&mut self) {
        self.borrowed_from
            .borrowed_at
            .write()
            .retain(|location| std::ptr::eq(*location, self.borrowed_at as *const _));
    }
}

/// A mutable reference to a value in a generational box.
pub struct GenerationalRefMut<W> {
    /// The inner value
    pub(crate) inner: W,
    pub(crate) borrow: GenerationalRefMutBorrowInfo,
}

impl<T: 'static, R: DerefMut<Target = T>> GenerationalRefMut<R> {
    pub(crate) fn new(inner: R, borrow: GenerationalRefMutBorrowInfo) -> Self {
        Self { inner, borrow }
    }
}

impl<T: ?Sized + 'static, W: DerefMut<Target = T>> Deref for GenerationalRefMut<W> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: ?Sized + Debug, R: Deref<Target = T>> Debug for GenerationalRefMut<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.deref().fmt(f)
    }
}

impl<T: ?Sized + Display, R: Deref<Target = T>> Display for GenerationalRefMut<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.deref().fmt(f)
    }
}

impl<T: ?Sized + 'static, W: DerefMut<Target = T>> DerefMut for GenerationalRefMut<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

/// Information about a mutable borrow.
pub struct GenerationalRefMutBorrowInfo {
    /// The location where the borrow occurred.
    pub(crate) borrowed_from: &'static MemoryLocationBorrowInfo,
    pub(crate) created_at: &'static std::panic::Location<'static>,
}

impl Drop for GenerationalRefMutBorrowInfo {
    fn drop(&mut self) {
        self.borrowed_from.borrowed_mut_at.write().take();
    }
}
