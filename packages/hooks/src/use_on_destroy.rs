use dioxus_core::{prelude::use_on_destroy, use_hook};

#[deprecated(
    note = "Use `use_on_destroy` instead, which has the same functionality. \
This is deprecated because of the introduction of `use_on_create` which is better mirrored by `use_on_destroy`. \
The reason why `use_on_create` is not `use_on_mount` is because of potential confusion with `dioxus::events::onmounted`."
)]
pub fn use_on_unmount<D: FnOnce() + 'static>(destroy: D) {
    use_on_destroy(destroy);
}

/// Creates a callback that will be run before the component is dropped
pub fn use_on_drop<D: FnOnce() + 'static>(ondrop: D) {
    use_on_destroy(ondrop);
}

pub fn use_hook_with_cleanup<T: Clone + 'static>(
    hook: impl FnOnce() -> T,
    cleanup: impl FnOnce(T) + 'static,
) -> T {
    let value = use_hook(hook);
    let _value = value.clone();
    use_on_destroy(move || cleanup(_value));
    value
}
