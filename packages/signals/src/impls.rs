use crate::copyvalue::CopyValue;
use crate::signal::Signal;
use crate::write_guard::Write;
use crate::SignalData;
use generational_box::Storage;
use generational_box::{GenerationalRef, UnsyncStorage};
use std::cell::Ref;
use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Sub},
};

use crate::macros::rules::{read_impls, write_impls, write_vec_impls};

read_impls!(CopyValue, S: Storage<T>, S: Storage<Vec<T>>);

impl<T: 'static, S: Storage<Vec<T>>> CopyValue<Vec<T>, S> {
    /// Read a value from the inner vector.
    #[track_caller]
    pub fn get<'a>(&'a self, index: usize) -> Option<S::Ref<'a, T>> {
        S::try_map(self.read(), move |v| v.get(index))
    }

    #[track_caller]
    pub fn get_static_ref(&self, index: usize) -> Option<S::Ref<'static, T>> {
        S::try_map(self.read_static_ref(), move |v| v.get(index))
    }
}

impl<T: 'static, S: Storage<Option<T>>> CopyValue<Option<T>, S> {
    /// Unwraps the inner value and clones it.
    #[track_caller]
    pub fn unwrap(&self) -> T
    where
        T: Clone,
    {
        self.with(|v| v.clone()).unwrap()
    }

    /// Attempts to read the inner value of the Option.
    #[track_caller]
    pub fn as_ref<'a>(&'a self) -> Option<S::Ref<'a, T>> {
        S::try_map(self.read(), |v| v.as_ref())
    }
}

write_impls!(CopyValue, Storage<T>, Storage<Vec<T>>);

impl<T: 'static, S: Storage<Option<T>>> CopyValue<Option<T>, S> {
    /// Takes the value out of the Option.
    #[track_caller]
    pub fn take(&self) -> Option<T> {
        self.with_mut(|v| v.take())
    }

    /// Replace the value in the Option.
    #[track_caller]
    pub fn replace(&self, value: T) -> Option<T> {
        self.with_mut(|v| v.replace(value))
    }

    /// Gets the value out of the Option, or inserts the given value if the Option is empty.
    #[track_caller]
    pub fn get_or_insert<'a>(&'a self, default: T) -> S::Ref<'a, T> {
        self.get_or_insert_with(|| default)
    }

    /// Gets the value out of the Option, or inserts the value returned by the given function if the Option is empty.
    #[track_caller]
    pub fn get_or_insert_with<'a>(&'a self, default: impl FnOnce() -> T) -> S::Ref<'a, T> {
        let borrow = self.read();
        if borrow.is_none() {
            drop(borrow);
            self.with_mut(|v| *v = Some(default()));
            S::map(self.read(), |v| v.as_ref().unwrap())
        } else {
            S::map(borrow, |v| v.as_ref().unwrap())
        }
    }
}

read_impls!(Signal, S: Storage<SignalData<T>>, S: Storage<SignalData<Vec<T>>>);

impl<T: 'static, S: Storage<SignalData<Vec<T>>>> Signal<Vec<T>, S> {
    /// Read a value from the inner vector.
    pub fn get<'a>(&'a self, index: usize) -> Option<S::Ref<'a, T>> {
        S::try_map(self.read(), move |v| v.get(index))
    }

    pub fn get_static_ref(&self, index: usize) -> Option<S::Ref<'static, T>> {
        todo!()
        // S::try_map(self.read(), move |v| v.get(index))
    }
}

impl<T: 'static, S: Storage<SignalData<Option<T>>>> Signal<Option<T>, S> {
    /// Unwraps the inner value and clones it.
    pub fn unwrap(&self) -> T
    where
        T: Clone,
    {
        self.with(|v| v.clone()).unwrap()
    }

    /// Attempts to read the inner value of the Option.
    pub fn as_ref<'a>(&'a self) -> Option<S::Ref<'a, T>> {
        S::try_map(self.read(), |v| v.as_ref())
    }
}

write_impls!(Signal, Storage<SignalData<T>>, Storage<SignalData<Vec<T>>>);

impl<T, S> Signal<Option<T>, S>
where
    T: 'static,
    S: Storage<SignalData<Option<T>>>,
{
    /// Takes the value out of the Option.
    pub fn take(&mut self) -> Option<T> {
        self.with_mut(|v| v.take())
    }

    /// Replace the value in the Option.
    pub fn replace(&mut self, value: T) -> Option<T> {
        self.with_mut(|v| v.replace(value))
    }

    /// Gets the value out of the Option, or inserts the given value if the Option is empty.
    pub fn get_or_insert<'a>(&'a mut self, default: T) -> S::Ref<'a, T> {
        self.get_or_insert_with(|| default)
    }

    /// Gets the value out of the Option, or inserts the value returned by the given function if the Option is empty.
    pub fn get_or_insert_with<'a>(&'a mut self, default: impl FnOnce() -> T) -> S::Ref<'a, T> {
        if self.read().is_some() {
            let borrow = self.read();
            return S::map(borrow, |v| v.as_ref().unwrap());
        }

        self.with_mut(|v| *v = Some(default()));
        S::map(self.read(), |v| v.as_ref().unwrap())
    }
}

/// An iterator over the values of a `CopyValue<Vec<T>>`.
pub struct CopyValueIterator<T: 'static, S: Storage<Vec<T>>> {
    index: usize,
    value: CopyValue<Vec<T>, S>,
}

impl<T, S: Storage<Vec<T>>> Iterator for CopyValueIterator<T, S> {
    type Item = S::Ref<'static, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.value.get_static_ref(index)
    }
}

impl<T: 'static, S: Storage<Vec<T>>> IntoIterator for CopyValue<Vec<T>, S> {
    type IntoIter = CopyValueIterator<T, S>;

    type Item = S::Ref<'static, T>;

    fn into_iter(self) -> Self::IntoIter {
        CopyValueIterator {
            index: 0,
            value: self,
        }
    }
}

impl<T: 'static, S: Storage<Vec<T>>> CopyValue<Vec<T>, S> {
    /// Write to an element in the inner vector.
    pub fn get_mut(&self, index: usize) -> Option<S::Mut<'static, T>> {
        todo!()
        // S::try_map_mut(self.write(), |v: &mut Vec<T>| v.get_mut(index))
    }
}

impl<T: 'static, S: Storage<Option<T>>> CopyValue<Option<T>, S> {
    /// Deref the inner value mutably.
    pub fn as_mut(&self) -> Option<S::Mut<'static, T>> {
        todo!()
        // S::try_map_mut(self.write(), |v: &mut Option<T>| v.as_mut())
    }
}

/// An iterator over items in a `Signal<Vec<T>>`.
pub struct SignalIterator<T: 'static, S: Storage<SignalData<Vec<T>>>> {
    index: usize,
    value: Signal<Vec<T>, S>,
}

impl<T, S: Storage<SignalData<Vec<T>>>> Iterator for SignalIterator<T, S> {
    type Item = S::Ref<'static, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.value.get_static_ref(index)
    }
}

impl<T: 'static, S: Storage<SignalData<Vec<T>>>> IntoIterator for Signal<Vec<T>, S> {
    type IntoIter = SignalIterator<T, S>;

    type Item = S::Ref<'static, T>;

    fn into_iter(self) -> Self::IntoIter {
        SignalIterator {
            index: 0,
            value: self,
        }
    }
}

impl<T: 'static, S: Storage<SignalData<Vec<T>>>> Signal<Vec<T>, S> {
    /// Returns a reference to an element or `None` if out of bounds.
    pub fn get_mut(&mut self, index: usize) -> Option<Write<T, S, Vec<T>>> {
        Write::filter_map(self.write(), |v| v.get_mut(index))
    }
}

impl<T: 'static, S: Storage<SignalData<Option<T>>>> Signal<Option<T>, S> {
    /// Returns a reference to an element or `None` if out of bounds.
    pub fn as_mut(&mut self) -> Option<Write<T, S, Option<T>>> {
        Write::filter_map(self.write(), |v| v.as_mut())
    }
}
