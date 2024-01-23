// use crate::Effect;
use generational_box::GenerationalBoxId;
use generational_box::UnsyncStorage;
use std::mem::MaybeUninit;
use std::ops::Deref;

use dioxus_core::prelude::*;
use dioxus_core::ScopeId;

use generational_box::{GenerationalBox, Storage};

/// Create a new CopyValue. The value will be stored in the current component.
///
/// When this component drops, the CopyValue will also be dropped
pub fn use_copy_value<T, S: Storage<T>>(f: impl FnOnce() -> T) -> CopyValue<T, S> {
    use_hook_with_drop(
        || CopyValue::new_maybe_sync(f()),
        |value| value.value.dispose(),
    )
}

/// CopyValue is a wrapper around a value to make the value mutable and Copy.
///
/// It is internally backed by [`generational_box::GenerationalBox`].
pub struct CopyValue<T: 'static, S: 'static = UnsyncStorage> {
    pub(crate) value: GenerationalBox<T, S>,
    origin_scope: ScopeId,
}

impl<T: 'static, S: Storage<T>> CopyValue<T, S> {
    ///
    pub fn invalid() -> Self {
        Self {
            value: GenerationalBox::claim(),
            origin_scope: current_scope_id().expect("in a virtual dom"),
        }
    }

    /// Create a new CopyValue. The value will be stored in the current component.
    ///
    /// Once the component this value is created in is dropped, the value will be dropped.
    #[track_caller]
    pub fn new_maybe_sync(value: T) -> Self {
        Self {
            value: GenerationalBox::new(value),
            origin_scope: current_scope_id().expect("in a virtual dom"),
        }
    }

    pub(crate) fn new_with_caller(
        value: T,
        #[cfg(debug_assertions)] caller: &'static std::panic::Location<'static>,
    ) -> Self {
        let mut value = GenerationalBox::new(value);
        value.set_caller(caller);
        Self {
            value,
            origin_scope: current_scope_id().expect("in a virtual dom"),
        }
    }

    /// Get the scope this value was created in.
    pub fn origin_scope(&self) -> ScopeId {
        self.origin_scope
    }

    /// Try to read the value. If the value has been dropped, this will return None.
    #[track_caller]
    pub fn try_read<'a>(&'a self) -> Result<S::Ref<'a, T>, generational_box::BorrowError> {
        self.value.try_read()
    }

    /// Read the value. If the value has been dropped, this will panic.
    #[track_caller]
    pub fn read<'a>(&'a self) -> S::Ref<'a, T> {
        self.value.read()
    }

    /// Read the value as a static reference. If the value has been dropped, this will panic.
    #[track_caller]
    pub fn read_static_ref(&self) -> S::Ref<'static, T> {
        self.value.read()
    }

    /// Try to write the value. If the value has been dropped, this will return None.
    #[track_caller]
    pub fn try_write<'a>(&'a mut self) -> Result<S::Mut<'a, T>, generational_box::BorrowMutError> {
        self.value.try_write()
    }

    /// Write the value. If the value has been dropped, this will panic.
    #[track_caller]
    pub fn write<'a>(&'a self) -> S::Mut<'a, T> {
        self.value.write()
    }

    /// Set the value. If the value has been dropped, this will panic.
    #[track_caller]
    pub fn set<'a>(&'a self, value: T) {
        self.value.set(value);
    }

    /// Run a function with a reference to the value. If the value has been dropped, this will panic.
    #[track_caller]
    pub fn with<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        let write = self.read();
        f(&*write)
    }

    /// Run a function with a mutable reference to the value. If the value has been dropped, this will panic.
    #[track_caller]
    pub fn with_mut<O>(&self, f: impl FnOnce(&mut T) -> O) -> O {
        let mut write = self.write();
        f(&mut *write)
    }

    /// Get the generational id of the value.
    pub fn id(&self) -> GenerationalBoxId {
        self.value.id()
    }
}

impl<T: Clone + 'static, S: Storage<T>> CopyValue<T, S> {
    /// Get the value. If the value has been dropped, this will panic.
    pub fn value(&self) -> T {
        self.read().clone()
    }
}

impl<T: 'static, S: Storage<T>> PartialEq for CopyValue<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }
}

#[cfg(feature = "serde")]
impl<T: 'static> serde::Serialize for CopyValue<T>
where
    T: serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.read().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: 'static> serde::Deserialize<'de> for CopyValue<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = T::deserialize(deserializer)?;

        Ok(Self::new(value))
    }
}

impl<T: Copy, S: Storage<T>> Deref for CopyValue<T, S> {
    type Target = dyn Fn() -> T;

    fn deref(&self) -> &Self::Target {
        // https://github.com/dtolnay/case-studies/tree/master/callable-types

        // First we create a closure that captures something with the Same in memory layout as Self (MaybeUninit<Self>).
        let uninit_callable = MaybeUninit::<Self>::uninit();
        // Then move that value into the closure. We assume that the closure now has a in memory layout of Self.
        let uninit_closure = move || *Self::read(unsafe { &*uninit_callable.as_ptr() });

        // Check that the size of the closure is the same as the size of Self in case the compiler changed the layout of the closure.
        let size_of_closure = std::mem::size_of_val(&uninit_closure);
        assert_eq!(size_of_closure, std::mem::size_of::<Self>());

        // Then cast the lifetime of the closure to the lifetime of &self.
        fn cast_lifetime<'a, T>(_a: &T, b: &'a T) -> &'a T {
            b
        }
        let reference_to_closure = cast_lifetime(
            {
                // The real closure that we will never use.
                &uninit_closure
            },
            // We transmute self into a reference to the closure. This is safe because we know that the closure has the same memory layout as Self so &Closure == &Self.
            unsafe { std::mem::transmute(self) },
        );

        // Cast the closure to a trait object.
        reference_to_closure as &Self::Target
    }
}
