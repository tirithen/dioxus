use dioxus_core::ScopeId;

use crate::CopyValue;

/// Hoist a signal to a scope.
///
/// Inserts the signal to be owned by the "owner" of the scope.
pub trait Hoist {
    fn hoist_to(&self, scope: ScopeId);
}

impl<T, V> Hoist for CopyValue<T, V> {
    fn hoist_to(&self, scope: ScopeId) {
        todo!()
    }
}
