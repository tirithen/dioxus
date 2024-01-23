use crate::{Signal, SignalData};
use generational_box::Storage;
use std::ops::{Deref, DerefMut};

/// A mutable reference to a signal's value.
///
/// T is the current type of the write
/// B is the dynamically checked type of the write (RefMut)
/// S is the storage type of the signal
/// I is the type of the original signal
pub struct Write<'a, T, S, I = T>
where
    T: 'static,
    S: Storage<SignalData<I>>,
    I: 'static,
{
    pub(crate) write: S::Mut<'a, T>,
    pub(crate) signal: SignalSubscriberDrop<I, S>,
}

impl<'a, T: 'static, S: Storage<SignalData<I>>, I: 'static> Write<'a, T, S, I> {
    /// Map the mutable reference to the signal's value to a new type.
    pub fn map<O>(myself: Self, f: impl FnOnce(&mut T) -> &mut O) -> Write<'a, O, S, I> {
        let Self { write, signal, .. } = myself;
        Write {
            write: S::map_mut(write, f),
            signal,
        }
    }

    /// Try to map the mutable reference to the signal's value to a new type
    pub fn filter_map<O>(
        myself: Self,
        f: impl FnOnce(&mut T) -> Option<&mut O>,
    ) -> Option<Write<'a, O, S, I>> {
        let Self { write, signal, .. } = myself;
        let write = S::try_map_mut(write, f);
        write.map(|write| Write { write, signal })
    }
}

impl<'a, T: 'static, S: Storage<SignalData<I>>, I: 'static> Deref for Write<'a, T, S, I> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

impl<'a, T, S: Storage<SignalData<I>>, I> DerefMut for Write<'a, T, S, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.write
    }
}

/// Since we map between values using map on `Write`, we can't actually give it a drop implementation.
///
/// The subscriber has to be passed through the `Write` so that it can be dropped when the `Write` is dropped.
pub(crate) struct SignalSubscriberDrop<T: 'static, S: Storage<SignalData<T>>>(pub Signal<T, S>);

impl<T: 'static, S: Storage<SignalData<T>>> Drop for SignalSubscriberDrop<T, S> {
    fn drop(&mut self) {
        self.0.update_subscribers();
    }
}
