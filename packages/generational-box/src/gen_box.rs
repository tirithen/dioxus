use crate::innerlude::*;
use std::marker::PhantomData;

/// The core Copy state type. The generational box will be dropped when the [Owner] is dropped.
pub struct GenerationalBox<T, S: 'static = crate::UnsyncStorage> {
    pub(crate) raw: &'static MemoryLocation<S>,
    pub(crate) generation: u32,
    pub(crate) created_at: &'static std::panic::Location<'static>,
    pub(crate) _marker: PhantomData<T>,
}

/// The type erased id of a generational box.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GenerationalBoxId {
    data_ptr: usize,
    generation: u32,
}

impl<T: 'static, S: Storage<T>> GenerationalBox<T, S> {
    pub fn claim() -> Self {
        let raw = S::claim();
        let generation = raw.generation.load(std::sync::atomic::Ordering::Relaxed);
        let created_at = std::panic::Location::caller();
        Self {
            raw,
            generation,
            created_at,
            _marker: PhantomData,
        }
    }

    pub fn new(value: T) -> Self {
        let new = Self::claim();
        new.raw.data.set(value);
        new
    }

    pub fn dispose(&self) {
        // Wipe away the data.
        self.raw.data.dispose(self.raw);

        // Set the generation to the next generation, making old handles invalid.
        self.raw
            .generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn swap(&self, new: T) {
        self.raw.data.set(new);
    }

    pub fn set_caller(&mut self, created_at: &'static std::panic::Location<'static>) {
        self.created_at = created_at;
    }

    #[inline(always)]
    pub fn validate(&self) -> bool {
        self.raw
            .generation
            .load(std::sync::atomic::Ordering::Relaxed)
            == self.generation
    }

    /// Get the raw pointer to the value.
    pub fn raw_ptr(&self) -> usize {
        self.raw.data.data_ptr()
    }

    /// Get the id of the generational box.
    pub fn id(&self) -> GenerationalBoxId {
        GenerationalBoxId {
            data_ptr: self.raw.data.data_ptr() as usize,
            generation: self.generation,
        }
    }

    /// Try to read the value. Returns None if the value is no longer valid.
    #[track_caller]
    pub fn try_read<'a>(&self) -> Result<S::Ref<'a, T>, BorrowError> {
        if !self.validate() {
            return Err(BorrowError::Dropped(ValueDroppedError {
                created_at: self.created_at,
            }));
        }
        let result = self.raw.data.try_read(GenerationalRefBorrowInfo {
            borrowed_at: std::panic::Location::caller(),
            borrowed_from: &self.raw.borrow,
            created_at: self.created_at,
        });

        if result.is_ok() {
            self.raw
                .borrow
                .borrowed_at
                .write()
                .push(std::panic::Location::caller());
        }

        result
    }

    /// Read the value. Panics if the value is no longer valid.
    #[track_caller]
    pub fn read<'a>(&self) -> S::Ref<'a, T> {
        self.try_read().unwrap()
    }

    /// Try to write the value. Returns None if the value is no longer valid.
    #[track_caller]
    pub fn try_write<'a>(&'a self) -> Result<S::Mut<'a, T>, BorrowMutError> {
        if !self.validate() {
            return Err(BorrowMutError::Dropped(ValueDroppedError {
                created_at: self.created_at,
            }));
        }

        let result = self.raw.data.try_write(GenerationalRefMutBorrowInfo {
            borrowed_from: &self.raw.borrow,
            created_at: self.created_at,
        });

        if result.is_ok() {
            *self.raw.borrow.borrowed_mut_at.write() = Some(std::panic::Location::caller());
        }

        result
    }

    /// Write the value. Panics if the value is no longer valid.
    #[track_caller]
    pub fn write<'a>(&'a self) -> S::Mut<'a, T> {
        self.try_write().unwrap()
    }

    /// Set the value. Panics if the value is no longer valid.
    pub fn set(&self, value: T) {
        self.validate().then(|| self.raw.data.set(value));
    }

    /// Returns true if the pointer is equal to the other pointer.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.raw.data.data_ptr() == other.raw.data.data_ptr() && self.generation == other.generation
    }
}

impl<T, S: 'static> Copy for GenerationalBox<T, S> {}

impl<T, S> Clone for GenerationalBox<T, S> {
    fn clone(&self) -> Self {
        *self
    }
}
