use crate::{
    innerlude::MemoryLocationBorrowInfo, BorrowError, BorrowMutError, GenerationalRefBorrowInfo,
    GenerationalRefMutBorrowInfo,
};
use std::{
    ops::{Deref, DerefMut},
    sync::atomic::AtomicU32,
};

/// A raw memory location backed by any storage type
///
/// The two storage types we provide are SyncStorage and UnsyncStorage.
/// We use a marker so that we can implement Send/Sync for the Box rather than deferring to an enum for the storage type.
pub struct MemoryLocation<S> {
    pub(crate) data: S,
    pub(crate) generation: AtomicU32,
    pub(crate) borrow: MemoryLocationBorrowInfo,
}

/// A trait for a storage backing type. (RefCell, RwLock, etc.)
pub trait Storage<Data = ()>: 'static + Sized {
    /// The reference this storage type returns.
    type Ref<'a, T: ?Sized + 'static>: Deref<Target = T>;

    /// The mutable reference this storage type returns.
    type Mut<'a, T: ?Sized + 'static>: DerefMut<Target = T>;

    /// Claim a new instance of this storage type
    ///
    /// It's up to you to dispose of the memory location when you're done with it!
    fn claim() -> &'static MemoryLocation<Self>;

    /// Drop the inner value and return the memory location to the runtime.
    ///
    /// This ensures the box handle is stable but the underlying value is dropped.
    fn dispose(&self, location: &'static MemoryLocation<Self>);

    /// Get the data pointer. No guarantees are made about the data pointer. It should only be used for debugging.
    fn data_ptr(&self) -> usize;

    /// Try to map the mutable ref.
    fn try_map_mut<'a, T, U>(
        mut_ref: Self::Mut<'a, T>,
        f: impl FnOnce(&mut T) -> Option<&mut U>,
    ) -> Option<Self::Mut<'a, U>>
    where
        U: ?Sized + 'static;

    /// Map the mutable ref.
    fn map_mut<'a, T, U: ?Sized + 'static>(
        mut_ref: Self::Mut<'a, T>,
        f: impl FnOnce(&mut T) -> &mut U,
    ) -> Self::Mut<'a, U> {
        Self::try_map_mut(mut_ref, |v| Some(f(v))).unwrap()
    }

    /// Try to map the ref.
    fn try_map<'a, T, U: ?Sized + 'static>(
        ref_: Self::Ref<'a, T>,
        f: impl FnOnce(&T) -> Option<&U>,
    ) -> Option<Self::Ref<'a, U>>;

    /// Map the ref.
    fn map<'a, T, U: ?Sized + 'static>(
        ref_: Self::Ref<'a, T>,
        f: impl FnOnce(&T) -> &U,
    ) -> Self::Ref<'a, U> {
        Self::try_map(ref_, |v| Some(f(v))).unwrap()
    }

    /// Try to read the value. Returns None if the value is no longer valid.
    fn try_read<'a>(
        &'static self,
        at: GenerationalRefBorrowInfo,
    ) -> Result<Self::Ref<'a, Data>, BorrowError>;

    /// Try to write the value. Returns None if the value is no longer valid.
    fn try_write<'a>(
        &'static self,
        at: GenerationalRefMutBorrowInfo,
    ) -> Result<Self::Mut<'a, Data>, BorrowMutError>;

    /// Set the value
    fn set(&'static self, value: Data);
}
