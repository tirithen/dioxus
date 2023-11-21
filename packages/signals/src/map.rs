use generational_box::Mappable;use crate::CopyValue;
use dioxus_core::ScopeId;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Deref;

/// A read only signal that has been mapped to a new type.
pub struct SignalMap<U: 'static + ?Sized, R: Deref<Target = U> + 'static> {
    pub(crate) origin_scope: ScopeId,
    pub(crate) mapping: CopyValue<Box<dyn Fn() -> R>>,
}


impl<U: ?Sized, R: Deref<Target = U> + 'static> SignalMap<U, R> {
    /// Get the scope that the signal was created in.
    pub fn origin_scope(&self) -> ScopeId {
        self.origin_scope
    }

    /// Get the current value of the signal. This will subscribe the current scope to the signal.
    pub fn read(&self) -> R {
        (self.mapping.read())()
    }

    /// Run a closure with a reference to the signal's value.
    pub fn with<O>(&self, f: impl FnOnce(&U) -> O) -> O {
        f(&*self.read())
    }
}

impl<U: ?Sized + Clone, R:  Deref<Target = U> + 'static> SignalMap<U, R> {
    /// Get the current value of the signal. This will subscribe the current scope to the signal.
    pub fn value(&self) -> U {
        self.read().clone()
    }
}

impl<U: ?Sized, R:  Deref<Target = U> + 'static> PartialEq for SignalMap<U, R> {
    fn eq(&self, other: &Self) -> bool {
        self.mapping == other.mapping
    }
}

impl<U, R: Deref<Target = U> + 'static> std::clone::Clone for SignalMap<U, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<U, R:  Deref<Target = U> + 'static> Copy for SignalMap<U, R> {}

impl<U: ?Sized + Display, R: Deref<Target = U> + 'static> Display for SignalMap<U, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with(|v| Display::fmt(v, f))
    }
}

impl<U: ?Sized + Debug, R:  Deref<Target = U> + 'static> Debug for SignalMap<U, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with(|v| Debug::fmt(v, f))
    }
}

impl<U, R: Mappable<Vec<U>> + Deref<Target = Vec<U>> + 'static> SignalMap<Vec<U>, R> {
    /// Read a value from the inner vector.
    pub fn get(&self, index: usize) -> Option<<R as Mappable<Vec<U>>>::Mapped<U>> {
        R::try_map(self.read(), |v| v.get(index))
    }
}

impl<U: Clone + 'static, R: Mappable<Option<U>> + Deref<Target = Option<U>> + 'static> SignalMap<Option<U>, R> {
    /// Unwraps the inner value and clones it.
    pub fn unwrap(&self) -> U
    where
        U: Clone,
    {
        self.with(|v| v.clone()).unwrap()
    }

    /// Attemps to read the inner value of the Option.
    pub fn as_ref(&self) -> Option<<R as Mappable<Option<U>>>::Mapped<U>> {
        R::try_map(self.read(), |v| v.as_ref())
    }
}
