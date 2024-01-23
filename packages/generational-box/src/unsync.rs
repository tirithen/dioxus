use crate::innerlude::*;
use std::cell::{Ref, RefCell, RefMut};

/// A unsync storage. This is the default storage type.
#[derive(Default)]
pub struct UnsyncStorage(RefCell<Option<Box<dyn std::any::Any>>>);

thread_local! {
    static UNSYNC_RUNTIME: RefCell<Vec<&'static MemoryLocation<UnsyncStorage>>> = RefCell::new(Vec::new());
}

impl<T: 'static> Storage<T> for UnsyncStorage {
    type Ref<'a, R: ?Sized + 'static> = GenerationalRef<Ref<'static, R>>;
    type Mut<'a, W: ?Sized + 'static> = GenerationalRefMut<RefMut<'static, W>>;

    fn claim() -> &'static MemoryLocation<Self> {
        UNSYNC_RUNTIME.with(|runtime| {
            if let Some(location) = runtime.borrow_mut().pop() {
                location
            } else {
                &*Box::leak(Box::new(MemoryLocation {
                    data: Self::default(),
                    generation: 0.into(),
                    borrow: Default::default(),
                }))
            }
        })
    }

    fn dispose(&self, location: &'static MemoryLocation<Self>) {
        self.0.borrow_mut().take();
        UNSYNC_RUNTIME.with(|runtime| runtime.borrow_mut().push(location));
    }

    fn data_ptr(&self) -> usize {
        self.0.as_ptr() as usize
    }

    fn set(&self, value: T) {
        *self.0.borrow_mut() = Some(Box::new(value));
    }

    fn try_read<'a>(
        &'static self,
        at: crate::GenerationalRefBorrowInfo,
    ) -> Result<Self::Ref<'a, T>, BorrowError> {
        let borrow = self.0.try_borrow();

        let borrow = borrow.map_err(|_| at.borrowed_from.borrow_error())?;

        Ref::filter_map(borrow, |any| any.as_ref()?.downcast_ref())
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
        let borrow = self.0.try_borrow_mut();

        let borrow = borrow.map_err(|_| at.borrowed_from.borrow_mut_error())?;

        RefMut::filter_map(borrow, |any| any.as_mut()?.downcast_mut())
            .map_err(|_| {
                BorrowMutError::Dropped(ValueDroppedError {
                    created_at: at.created_at,
                })
            })
            .map(|guard| GenerationalRefMut::new(guard, at))
    }

    fn try_map<'a, I, U: ?Sized + 'static>(
        _self: Self::Ref<'a, I>,
        f: impl FnOnce(&I) -> Option<&U>,
    ) -> Option<Self::Ref<'a, U>> {
        let GenerationalRef { inner, borrow, .. } = _self;
        Ref::filter_map(inner, f)
            .ok()
            .map(|inner| GenerationalRef { inner, borrow })
    }

    fn try_map_mut<'a, I, U: ?Sized + 'static>(
        mut_ref: Self::Mut<'a, I>,
        f: impl FnOnce(&mut I) -> Option<&mut U>,
    ) -> Option<Self::Mut<'a, U>> {
        let GenerationalRefMut { inner, borrow, .. } = mut_ref;
        RefMut::filter_map(inner, f)
            .ok()
            .map(|inner| GenerationalRefMut {
                inner,
                borrow: GenerationalRefMutBorrowInfo {
                    borrowed_from: borrow.borrowed_from,
                    created_at: borrow.created_at,
                },
            })
    }
}
