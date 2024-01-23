use crate::innerlude::*;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::{
    any::Any,
    sync::{Arc, OnceLock},
};

/// A thread safe storage. This is slower than the unsync storage, but allows you to share the value between threads.
#[derive(Default)]
pub struct SyncStorage(RwLock<Option<Box<dyn Any + Send + Sync>>>);

fn sync_runtime() -> &'static Arc<Mutex<Vec<&'static MemoryLocation<SyncStorage>>>> {
    static SYNC_RUNTIME: OnceLock<Arc<Mutex<Vec<&'static MemoryLocation<SyncStorage>>>>> =
        OnceLock::new();

    SYNC_RUNTIME.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

impl<T: Sync + Send + 'static> Storage<T> for SyncStorage {
    type Ref<'a, R: ?Sized + 'static> = GenerationalRef<MappedRwLockReadGuard<'static, R>>;
    type Mut<'a, W: ?Sized + 'static> = GenerationalRefMut<MappedRwLockWriteGuard<'static, W>>;

    fn claim() -> &'static MemoryLocation<Self> {
        sync_runtime().lock().pop().unwrap_or_else(|| {
            &*Box::leak(Box::new(MemoryLocation {
                data: Self::default(),
                generation: 0.into(),
                borrow: Default::default(),
            }))
        })
    }

    fn dispose(&self, location: &'static MemoryLocation<Self>) {
        self.0.write().take();
        sync_runtime().lock().push(location);
    }

    fn data_ptr(&self) -> usize {
        self.0.data_ptr() as usize
    }

    fn try_read<'a>(
        &'static self,
        at: crate::GenerationalRefBorrowInfo,
    ) -> Result<Self::Ref<'a, T>, BorrowError> {
        let read = self.0.try_read();

        let read = read.ok_or_else(|| at.borrowed_from.borrow_error())?;

        RwLockReadGuard::try_map(read, |any| any.as_ref()?.downcast_ref())
            .map_err(|_| {
                BorrowError::Dropped(ValueDroppedError {
                    created_at: at.created_at,
                })
            })
            .map(|guard| GenerationalRef::new(guard, at))
    }

    fn try_write<'a>(
        &'static self,
        at: crate::GenerationalRefMutBorrowInfo,
    ) -> Result<Self::Mut<'a, T>, BorrowMutError> {
        let write = self.0.try_write();

        let write = write.ok_or_else(|| at.borrowed_from.borrow_mut_error())?;

        RwLockWriteGuard::try_map(write, |any| any.as_mut()?.downcast_mut())
            .map_err(|_| {
                BorrowMutError::Dropped(ValueDroppedError {
                    created_at: at.created_at,
                })
            })
            .map(|guard| GenerationalRefMut::new(guard, at))
    }

    fn set(&self, value: T) {
        *self.0.write() = Some(Box::new(value));
    }

    fn try_map<'a, I, U: ?Sized + 'static>(
        ref_: Self::Ref<'a, I>,
        f: impl FnOnce(&I) -> Option<&U>,
    ) -> Option<Self::Ref<'a, U>> {
        let GenerationalRef { inner, borrow, .. } = ref_;
        MappedRwLockReadGuard::try_map(inner, f)
            .ok()
            .map(|inner| GenerationalRef { inner, borrow })
    }

    fn try_map_mut<'a, I, U: ?Sized + 'static>(
        mut_ref: Self::Mut<'a, I>,
        f: impl FnOnce(&mut I) -> Option<&mut U>,
    ) -> Option<Self::Mut<'a, U>> {
        let GenerationalRefMut { inner, borrow, .. } = mut_ref;
        MappedRwLockWriteGuard::try_map(inner, f)
            .ok()
            .map(|inner| GenerationalRefMut { inner, borrow })
    }
}
