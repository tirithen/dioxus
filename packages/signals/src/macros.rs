pub mod rules {

    macro_rules! read_impls {
    ($ty:ident $(: $extra_bounds:path)? $(, $bound_ty:ident : $bound:path, $vec_bound_ty:ident : $vec_bound:path)?) => {
        // Using default to create new signals is an easy way of causing leaks...
        // $(
        //     impl<T: Default + 'static, $bound_ty: $bound> Default for $ty<T, $bound_ty> {
        //         #[track_caller]
        //         fn default() -> Self {
        //             Self::new_maybe_sync(Default::default())
        //         }
        //     }
        // )?

        impl<T $(: $extra_bounds)? $(,$bound_ty: $bound)?> std::clone::Clone for $ty<T $(, $bound_ty)?> {
            #[track_caller]
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<T $(: $extra_bounds)? $(,$bound_ty: $bound)?> Copy for $ty<T $(, $bound_ty)?> {}

        impl<T: $($extra_bounds + )? Display + 'static $(,$bound_ty: $bound)?> Display for $ty<T $(, $bound_ty)?> {
            #[track_caller]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.with(|v| Display::fmt(v, f))
            }
        }

        impl<T: $($extra_bounds + )? Debug + 'static $(,$bound_ty: $bound)?> Debug for $ty<T $(, $bound_ty)?> {
            #[track_caller]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.with(|v| Debug::fmt(v, f))
            }
        }

        impl<T: $($extra_bounds + )? PartialEq + 'static $(,$bound_ty: $bound)?> PartialEq<T> for $ty<T $(, $bound_ty)?> {
            #[track_caller]
            fn eq(&self, other: &T) -> bool {
                self.with(|v| *v == *other)
            }
        }

        impl<T: $($extra_bounds + )? 'static $(,$vec_bound_ty: $vec_bound)?> $ty<Vec<T>, $($vec_bound_ty)?> {
            /// Returns the length of the inner vector.
            #[track_caller]
            pub fn len(&self) -> usize {
                self.with(|v| v.len())
            }

            /// Returns true if the inner vector is empty.
            #[track_caller]
            pub fn is_empty(&self) -> bool {
                self.with(|v| v.is_empty())
            }
        }
    };
}

    macro_rules! write_impls {
    ($ty:ident, $bound:path, $vec_bound:path) => {
        impl<T: Add<Output = T> + Copy + 'static, S: $bound> std::ops::Add<T> for $ty<T, S> {
            type Output = T;

            #[track_caller]
            fn add(self, rhs: T) -> Self::Output {
                self.with(|v| *v + rhs)
            }
        }

        impl<T: Add<Output = T> + Copy + 'static, S: $bound> std::ops::AddAssign<T> for $ty<T, S> {
            #[track_caller]
            fn add_assign(&mut self, rhs: T) {
                self.with_mut(|v| *v = *v + rhs)
            }
        }

        impl<T: Sub<Output = T> + Copy + 'static, S: $bound> std::ops::SubAssign<T> for $ty<T, S> {
            #[track_caller]
            fn sub_assign(&mut self, rhs: T) {
                self.with_mut(|v| *v = *v - rhs)
            }
        }

        impl<T: Sub<Output = T> + Copy + 'static, S: $bound> std::ops::Sub<T> for $ty<T, S> {
            type Output = T;

            #[track_caller]
            fn sub(self, rhs: T) -> Self::Output {
                self.with(|v| *v - rhs)
            }
        }

        impl<T: Mul<Output = T> + Copy + 'static, S: $bound> std::ops::MulAssign<T> for $ty<T, S> {
            #[track_caller]
            fn mul_assign(&mut self, rhs: T) {
                self.with_mut(|v| *v = *v * rhs)
            }
        }

        impl<T: Mul<Output = T> + Copy + 'static, S: $bound> std::ops::Mul<T> for $ty<T, S> {
            type Output = T;

            #[track_caller]
            fn mul(self, rhs: T) -> Self::Output {
                self.with(|v| *v * rhs)
            }
        }

        impl<T: Div<Output = T> + Copy + 'static, S: $bound> std::ops::DivAssign<T> for $ty<T, S> {
            #[track_caller]
            fn div_assign(&mut self, rhs: T) {
                self.with_mut(|v| *v = *v / rhs)
            }
        }

        impl<T: Div<Output = T> + Copy + 'static, S: $bound> std::ops::Div<T> for $ty<T, S> {
            type Output = T;

            #[track_caller]
            fn div(self, rhs: T) -> Self::Output {
                self.with(|v| *v / rhs)
            }
        }

        write_vec_impls!($ty, S: $vec_bound);
    };
}

    macro_rules! write_vec_impls {
    ($ty:ident $(, $vec_bound_ty:ident: $vec_bound:path)?) => {
        impl<T: 'static $(, $vec_bound_ty: $vec_bound)?> $ty<Vec<T> $(, $vec_bound_ty)?> {
            /// Pushes a new value to the end of the vector.
            #[track_caller]
            pub fn push(&mut self, value: T) {
                self.with_mut(|v| v.push(value))
            }

            /// Pops the last value from the vector.
            #[track_caller]
            pub fn pop(&mut self) -> Option<T> {
                self.with_mut(|v| v.pop())
            }

            /// Inserts a new value at the given index.
            #[track_caller]
            pub fn insert(&mut self, index: usize, value: T) {
                self.with_mut(|v| v.insert(index, value))
            }

            /// Removes the value at the given index.
            #[track_caller]
            pub fn remove(&mut self, index: usize) -> T {
                self.with_mut(|v| v.remove(index))
            }

            /// Clears the vector, removing all values.
            #[track_caller]
            pub fn clear(&mut self) {
                self.with_mut(|v| v.clear())
            }

            /// Extends the vector with the given iterator.
            #[track_caller]
            pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
                self.with_mut(|v| v.extend(iter))
            }

            /// Truncates the vector to the given length.
            #[track_caller]
            pub fn truncate(&mut self, len: usize) {
                self.with_mut(|v| v.truncate(len))
            }

            /// Swaps two values in the vector.
            #[track_caller]
            pub fn swap_remove(&mut self, index: usize) -> T {
                self.with_mut(|v| v.swap_remove(index))
            }

            /// Retains only the values that match the given predicate.
            #[track_caller]
            pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
                self.with_mut(|v| v.retain(f))
            }

            /// Splits the vector into two at the given index.
            #[track_caller]
            pub fn split_off(&mut self, at: usize) -> Vec<T> {
                self.with_mut(|v| v.split_off(at))
            }
        }
    };
}

    pub(crate) use {read_impls, write_impls, write_vec_impls};
}
