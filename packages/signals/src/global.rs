use dioxus_core::prelude::{
    provide_root_context, try_consume_context, IntoAttributeValue, ScopeId,
};
use generational_box::{GenerationalRef, Storage, UnsyncStorage};
use std::fmt::{Debug, Display};
use std::{
    any::Any,
    cell::{Ref, RefCell},
    collections::HashMap,
    mem::MaybeUninit,
    ops::Deref,
    rc::Rc,
};

use crate::macros::rules::*;
use crate::{MappedSignal, ReadOnlySignal, Signal, Write};

/// A signal that can be accessed from anywhere in the application and created in a static
pub struct GlobalSignal<T> {
    initializer: fn() -> T,
}

#[derive(Clone)]
struct GlobalSignalContext {
    signal: Rc<RefCell<HashMap<*const (), Box<dyn Any>>>>,
}

fn get_global_context() -> GlobalSignalContext {
    match try_consume_context() {
        Some(context) => context,
        None => {
            let context = GlobalSignalContext {
                signal: Rc::new(RefCell::new(HashMap::new())),
            };
            provide_root_context(context).unwrap()
        }
    }
}

impl<T: 'static> GlobalSignal<T> {
    /// Create a new global signal with the given initializer.
    pub const fn new(initializer: fn() -> T) -> GlobalSignal<T> {
        GlobalSignal { initializer }
    }

    /// Get the signal that backs this global.
    pub fn signal(&self) -> Signal<T> {
        let key = self as *const _ as *const ();
        let context = get_global_context();
        let read = context.signal.borrow();

        match read.get(&key) {
            Some(signal) => *signal.downcast_ref::<Signal<T>>().unwrap(),
            None => {
                drop(read);

                // Constructors are always run in the root scope
                // The signal also exists in the root scope
                let value = ScopeId::ROOT.in_runtime(self.initializer);
                let signal = Signal::new_in_scope(value, ScopeId::ROOT);

                let entry = context.signal.borrow_mut().insert(key, Box::new(signal));
                debug_assert!(entry.is_none(), "Global signal already exists");

                signal
            }
        }
    }

    /// Get the scope the signal was created in.
    pub fn origin_scope(&self) -> ScopeId {
        ScopeId::ROOT
    }

    /// Get the current value of the signal. This will subscribe the current scope to the signal.  If you would like to read the signal without subscribing to it, you can use [`Self::peek`] instead.
    ///
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn read(&self) -> GenerationalRef<Ref<'static, T>> {
        self.signal().read_static_ref()
    }

    /// Get the current value of the signal. **Unlike read, this will not subscribe the current scope to the signal which can cause parts of your UI to not update.**
    ///
    /// If the signal has been dropped, this will panic.
    pub fn peek(&self) -> GenerationalRef<Ref<'static, T>> {
        self.signal().peek_static()
    }

    /// Get a mutable reference to the signal's value.
    ///
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn write(&self) -> Write<'static, T> {
        self.signal().write_unchecked()
    }

    /// Set the value of the signal. This will trigger an update on all subscribers.
    #[track_caller]
    pub fn set(&self, value: T) {
        self.signal().set(value);
    }

    /// Set the value of the signal without triggering an update on subscribers.
    #[track_caller]
    pub fn set_untracked(&self, value: T) {
        self.signal().set_untracked(value);
    }

    /// Run a closure with a reference to the signal's value.
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn with<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        self.signal().with(f)
    }

    /// Run a closure with a mutable reference to the signal's value.
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn with_mut<O>(&self, f: impl FnOnce(&mut T) -> O) -> O {
        self.signal().with_mut(f)
    }

    /// Map the signal to a new type.
    pub fn map<O>(
        &self,
        f: impl Fn(&T) -> &O + 'static,
    ) -> MappedSignal<GenerationalRef<Ref<'static, O>>> {
        MappedSignal::new(self.signal(), f)
    }

    /// Get the generational id of the signal.
    pub fn id(&self) -> generational_box::GenerationalBoxId {
        self.signal().id()
    }
}

impl<T: 'static> IntoAttributeValue for GlobalSignal<T>
where
    T: Clone + IntoAttributeValue,
{
    fn into_value(self) -> dioxus_core::AttributeValue {
        self.signal().into_value()
    }
}

impl<T: Clone + 'static> GlobalSignal<T> {
    /// Get the current value of the signal. This will subscribe the current scope to the signal.
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn cloned(&self) -> T {
        self.read().clone()
    }
}

impl GlobalSignal<bool> {
    /// Invert the boolean value of the signal. This will trigger an update on all subscribers.
    pub fn toggle(&self) {
        self.set(!self.cloned());
    }
}

impl<T: 'static> PartialEq for GlobalSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

/// Allow calling a signal with signal() syntax
///
/// Currently only limited to copy types, though could probably specialize for string/arc/rc
impl<T: Clone + 'static> Deref for GlobalSignal<T> {
    type Target = dyn Fn() -> T;

    fn deref(&self) -> &Self::Target {
        // https://github.com/dtolnay/case-studies/tree/master/callable-types

        // First we create a closure that captures something with the Same in memory layout as Self (MaybeUninit<Self>).
        let uninit_callable = MaybeUninit::<Self>::uninit();
        // Then move that value into the closure. We assume that the closure now has a in memory layout of Self.
        let uninit_closure = move || Self::read(unsafe { &*uninit_callable.as_ptr() }).clone();

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
            // We transmute self into a reference to the closure. This is safe because we know that the closure has the same memory layout as Self so &Closure == &self.
            unsafe { std::mem::transmute(self) },
        );

        // Cast the closure to a trait object.
        reference_to_closure as &Self::Target
    }
}

/// A signal that can be accessed from anywhere in the application and created in a static
pub struct GlobalMemo<T: 'static> {
    selector: fn() -> T,
}

impl<T: PartialEq + 'static> GlobalMemo<T> {
    /// Create a new global signal
    pub const fn new(selector: fn() -> T) -> GlobalMemo<T>
    where
        T: PartialEq,
    {
        GlobalMemo { selector }
    }

    /// Get the signal that backs this global.
    pub fn signal(&self) -> ReadOnlySignal<T> {
        let key = self as *const _ as *const ();

        let context = get_global_context();

        let read = context.signal.borrow();
        match read.get(&key) {
            Some(signal) => *signal.downcast_ref::<ReadOnlySignal<T>>().unwrap(),
            None => {
                drop(read);
                // Constructors are always run in the root scope
                let signal = ScopeId::ROOT.in_runtime(|| Signal::selector(self.selector));
                context.signal.borrow_mut().insert(key, Box::new(signal));
                signal
            }
        }
    }

    /// Get the scope the signal was created in.
    pub fn origin_scope(&self) -> ScopeId {
        ScopeId::ROOT
    }

    /// Get the current value of the signal. This will subscribe the current scope to the signal.  If you would like to read the signal without subscribing to it, you can use [`Self::peek`] instead.
    ///
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn read(&self) -> GenerationalRef<Ref<'static, T>> {
        self.signal().inner.read_static_ref()
    }

    /// Get the current value of the signal. **Unlike read, this will not subscribe the current scope to the signal which can cause parts of your UI to not update.**
    ///
    /// If the signal has been dropped, this will panic.
    pub fn peek(&self) -> GenerationalRef<Ref<'static, T>> {
        self.signal().inner.peek_static()
    }

    /// Run a closure with a reference to the signal's value.
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn with<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        self.signal().with(f)
    }

    /// Get the generational id of the signal.
    pub fn id(&self) -> generational_box::GenerationalBoxId {
        self.signal().id()
    }
}

impl<T: PartialEq + 'static> IntoAttributeValue for GlobalMemo<T>
where
    T: Clone + IntoAttributeValue,
{
    fn into_value(self) -> dioxus_core::AttributeValue {
        self.signal().into_value()
    }
}

impl<T: PartialEq + Clone + 'static> GlobalMemo<T> {
    /// Get the current value of the signal. This will subscribe the current scope to the signal.
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    pub fn cloned(&self) -> T {
        self.read().clone()
    }
}

impl<T: PartialEq + 'static> PartialEq for GlobalMemo<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

/// Allow calling a signal with signal() syntax
///
/// Currently only limited to copy types, though could probably specialize for string/arc/rc
impl<T: PartialEq + Clone + 'static> Deref for GlobalMemo<T> {
    type Target = dyn Fn() -> T;

    fn deref(&self) -> &Self::Target {
        // https://github.com/dtolnay/case-studies/tree/master/callable-types

        // First we create a closure that captures something with the Same in memory layout as Self (MaybeUninit<Self>).
        let uninit_callable = MaybeUninit::<Self>::uninit();
        // Then move that value into the closure. We assume that the closure now has a in memory layout of Self.
        let uninit_closure = move || Self::read(unsafe { &*uninit_callable.as_ptr() }).clone();

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
            // We transmute self into a reference to the closure. This is safe because we know that the closure has the same memory layout as Self so &Closure == &self.
            unsafe { std::mem::transmute(self) },
        );

        // Cast the closure to a trait object.
        reference_to_closure as &Self::Target
    }
}

read_impls!(GlobalSignal);

impl<T: 'static> GlobalSignal<Vec<T>> {
    /// Read a value from the inner vector.
    pub fn get(&'static self, index: usize) -> Option<GenerationalRef<Ref<'static, T>>> {
        <UnsyncStorage as Storage>::try_map(self.read(), move |v| v.get(index))
    }
}

impl<T: 'static> GlobalSignal<Option<T>> {
    /// Unwraps the inner value and clones it.
    pub fn unwrap(&'static self) -> T
    where
        T: Clone,
    {
        self.with(|v| v.clone()).unwrap()
    }

    /// Attempts to read the inner value of the Option.
    pub fn as_ref(&'static self) -> Option<GenerationalRef<Ref<'static, T>>> {
        <UnsyncStorage as Storage>::try_map(self.read(), |v| v.as_ref())
    }
}

write_vec_impls!(GlobalSignal);

impl<T: 'static> GlobalSignal<Option<T>> {
    /// Takes the value out of the Option.
    pub fn take(&self) -> Option<T> {
        self.with_mut(|v| v.take())
    }

    /// Replace the value in the Option.
    pub fn replace(&self, value: T) -> Option<T> {
        self.with_mut(|v| v.replace(value))
    }

    /// Gets the value out of the Option, or inserts the given value if the Option is empty.
    pub fn get_or_insert(&self, default: T) -> GenerationalRef<Ref<'static, T>> {
        self.get_or_insert_with(|| default)
    }

    /// Gets the value out of the Option, or inserts the value returned by the given function if the Option is empty.
    pub fn get_or_insert_with(
        &self,
        default: impl FnOnce() -> T,
    ) -> GenerationalRef<Ref<'static, T>> {
        let borrow = self.read();
        if borrow.is_none() {
            drop(borrow);
            self.with_mut(|v| *v = Some(default()));
            <UnsyncStorage as Storage>::map(self.read(), |v| v.as_ref().unwrap())
        } else {
            <UnsyncStorage as Storage>::map(borrow, |v| v.as_ref().unwrap())
        }
    }
}

read_impls!(GlobalMemo: PartialEq);

impl<T: PartialEq + 'static> GlobalMemo<Vec<T>> {
    /// Read a value from the inner vector.
    pub fn get(&'static self, index: usize) -> Option<GenerationalRef<Ref<'static, T>>> {
        <UnsyncStorage as Storage>::try_map(self.read(), move |v| v.get(index))
    }
}

impl<T: PartialEq + 'static> GlobalMemo<Option<T>> {
    /// Unwraps the inner value and clones it.
    pub fn unwrap(&'static self) -> T
    where
        T: Clone,
    {
        self.with(|v| v.clone()).unwrap()
    }

    /// Attempts to read the inner value of the Option.
    pub fn as_ref(&'static self) -> Option<GenerationalRef<Ref<'static, T>>> {
        <UnsyncStorage as Storage>::try_map(self.read(), |v| v.as_ref())
    }
}
