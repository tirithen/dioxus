use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dioxus_html::{
    input_data::keyboard_types::{Code, Modifiers},
    ShortcutProvider, ShortcutRegistryError,
};
use slab::Slab;
use wry::application::{
    accelerator::{Accelerator, AcceleratorId},
    event_loop::EventLoopWindowTarget,
    global_shortcut::{GlobalShortcut, ShortcutManager},
    keyboard::{KeyCode, ModifiersState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// An global id for a shortcut.
pub struct ShortcutId {
    id: AcceleratorId,
    number: usize,
}

struct Shortcut {
    shortcut: GlobalShortcut,
    callbacks: Slab<Box<dyn FnMut()>>,
}

impl Shortcut {
    fn insert(&mut self, callback: Box<dyn FnMut()>) -> usize {
        self.callbacks.insert(callback)
    }

    fn remove(&mut self, id: usize) {
        let _ = self.callbacks.remove(id);
    }

    fn is_empty(&self) -> bool {
        self.callbacks.is_empty()
    }
}

type ShortcutMap = Rc<RefCell<HashMap<AcceleratorId, Shortcut>>>;

#[derive(Clone)]
pub(crate) struct ShortcutRegistry {
    manager: Rc<RefCell<ShortcutManager>>,
    shortcuts: ShortcutMap,
}

impl ShortcutRegistry {
    pub fn new<T>(target: &EventLoopWindowTarget<T>) -> Self {
        Self {
            manager: Rc::new(RefCell::new(ShortcutManager::new(target))),
            shortcuts: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub(crate) fn call_handlers(&self, id: AcceleratorId) {
        if let Some(Shortcut { callbacks, .. }) = self.shortcuts.borrow_mut().get_mut(&id) {
            for (_, callback) in callbacks.iter_mut() {
                (callback)();
            }
        }
    }

    pub(crate) fn add_shortcut(
        &self,
        accelerator: Accelerator,
        callback: Box<dyn FnMut()>,
    ) -> Result<ShortcutId, ShortcutRegistryError> {
        let accelerator_id = accelerator.clone().id();
        let mut shortcuts = self.shortcuts.borrow_mut();
        Ok(
            if let Some(callbacks) = shortcuts.get_mut(&accelerator_id) {
                let id = callbacks.insert(callback);
                ShortcutId {
                    id: accelerator_id,
                    number: id,
                }
            } else {
                match self.manager.borrow_mut().register(accelerator) {
                    Ok(global_shortcut) => {
                        let mut slab = Slab::new();
                        let id = slab.insert(callback);
                        let shortcut = Shortcut {
                            shortcut: global_shortcut,
                            callbacks: slab,
                        };
                        shortcuts.insert(accelerator_id, shortcut);
                        ShortcutId {
                            id: accelerator_id,
                            number: id,
                        }
                    }
                    Err(err) => return Err(ShortcutRegistryError::Other(Box::new(err))),
                }
            },
        )
    }

    pub(crate) fn remove_shortcut(&self, id: ShortcutId) {
        let mut shortcuts = self.shortcuts.borrow_mut();
        if let Some(callbacks) = shortcuts.get_mut(&id.id) {
            callbacks.remove(id.number);
            if callbacks.is_empty() {
                if let Some(shortcut) = shortcuts.remove(&id.id) {
                    let _ = self.manager.borrow_mut().unregister(shortcut.shortcut);
                }
            }
        }
    }

    pub(crate) fn remove_all(&self) {
        let mut shortcuts = self.shortcuts.borrow_mut();
        shortcuts.clear();
        let _ = self.manager.borrow_mut().unregister_all();
    }
}

impl ShortcutProvider for ShortcutRegistry {
    fn new_shortcut(
        &self,
        _cx: &dioxus_core::ScopeState,
        accelerator: dioxus_html::Accelerator,
        handler: Box<dyn FnMut() + 'static>,
    ) -> Result<Box<dyn dioxus_html::Shortcut>, ShortcutRegistryError> {
        let key_code = into_key_code(accelerator.key);
        let mut modifiers = ModifiersState::empty();
        if accelerator.modifiers.contains(Modifiers::ALT) {
            modifiers |= ModifiersState::ALT;
        }
        if accelerator.modifiers.contains(Modifiers::CONTROL) {
            modifiers |= ModifiersState::CONTROL;
        }
        if accelerator.modifiers.contains(Modifiers::SHIFT) {
            modifiers |= ModifiersState::SHIFT;
        }
        if accelerator.modifiers.contains(Modifiers::SUPER) {
            modifiers |= ModifiersState::SUPER;
        }
        let accelerator = Accelerator::new(modifiers, key_code);
        let id = self.add_shortcut(accelerator, handler)?;
        Ok(Box::new(DesktopShortcut {
            id,
            manager: self.clone(),
        }))
    }
}

/// An global id for a shortcut.
struct DesktopShortcut {
    id: ShortcutId,
    manager: ShortcutRegistry,
}

impl dioxus_html::Shortcut for DesktopShortcut {
    fn remove(&mut self) {
        self.manager.remove_shortcut(self.id);
    }
}

fn into_key_code(code: Code) -> KeyCode {
    match code {
Code::Backspace => KeyCode::Backspace,
Code::Tab => KeyCode::Tab,
Code::NumpadClear => KeyCode::NumpadClear,
Code::Enter => KeyCode::Enter,
Code::ShiftLeft  => KeyCode::ShiftLeft,
Code::ShiftRight  => KeyCode::ShiftRight,
Code::ControlLeft  => KeyCode::ControlLeft,
Code::ControlRight  => KeyCode::ControlRight,
Code::AltLeft  => KeyCode::AltLeft,
Code::AltRight  => KeyCode::AltRight,
Code::Pause => KeyCode::Pause,
Code::CapsLock => KeyCode::CapsLock,
Code::Escape => KeyCode::Escape,
Code::Space => KeyCode::Space,
Code::PageUp => KeyCode::PageUp,
Code::PageDown => KeyCode::PageDown,
Code::End => KeyCode::End,
Code::Home => KeyCode::Home,
Code::ArrowLeft => KeyCode::ArrowLeft,
Code::ArrowUp => KeyCode::ArrowUp,
Code::ArrowRight => KeyCode::ArrowRight,
Code::ArrowDown => KeyCode::ArrowDown,
Code::Insert => KeyCode::Insert,
Code::Delete => KeyCode::Delete,
Code::Numpad0 => KeyCode::Numpad0,
Code::Numpad1 => KeyCode::Numpad1,
Code::Numpad2 => KeyCode::Numpad2,
Code::Numpad3 => KeyCode::Numpad3,
Code::Numpad4 => KeyCode::Numpad4,
Code::Numpad5 => KeyCode::Numpad5,
Code::Numpad6 => KeyCode::Numpad6,
Code::Numpad7 => KeyCode::Numpad7,
Code::Numpad8 => KeyCode::Numpad8,
Code::Numpad9 => KeyCode::Numpad9,
Code::KeyA => KeyCode::KeyA,
Code::KeyB => KeyCode::KeyB,
Code::KeyC => KeyCode::KeyC,
Code::KeyD => KeyCode::KeyD,
Code::KeyE => KeyCode::KeyE,
Code::KeyF => KeyCode::KeyF,
Code::KeyG => KeyCode::KeyG,
Code::KeyH => KeyCode::KeyH,
Code::KeyI => KeyCode::KeyI,
Code::KeyJ => KeyCode::KeyJ,
Code::KeyK => KeyCode::KeyK,
Code::KeyL => KeyCode::KeyL,
Code::KeyM => KeyCode::KeyM,
Code::KeyN => KeyCode::KeyN,
Code::KeyO => KeyCode::KeyO,
Code::KeyP => KeyCode::KeyP,
Code::KeyQ => KeyCode::KeyQ,
Code::KeyR => KeyCode::KeyR,
Code::KeyS => KeyCode::KeyS,
Code::KeyT => KeyCode::KeyT,
Code::KeyU => KeyCode::KeyU,
Code::KeyV => KeyCode::KeyV,
Code::KeyW => KeyCode::KeyW,
Code::KeyX => KeyCode::KeyX,
Code::KeyY => KeyCode::KeyY,
Code::KeyZ => KeyCode::KeyZ,
Code::NumpadMultiply => KeyCode::NumpadMultiply,
Code::NumpadAdd => KeyCode::NumpadAdd,
Code::NumpadSubtract => KeyCode::NumpadSubtract,
Code::NumpadDecimal => KeyCode::NumpadDecimal,
Code::NumpadDivide => KeyCode::NumpadDivide,
Code::F1 => KeyCode::F1,
Code::F2 => KeyCode::F2,
Code::F3 => KeyCode::F3,
Code::F4 => KeyCode::F4,
Code::F5 => KeyCode::F5,
Code::F6 => KeyCode::F6,
Code::F7 => KeyCode::F7,
Code::F8 => KeyCode::F8,
Code::F9 => KeyCode::F9,
Code::F10 => KeyCode::F10,
Code::F11 => KeyCode::F11,
Code::F12 => KeyCode::F12,
Code::NumLock => KeyCode::NumLock,
Code::ScrollLock => KeyCode::ScrollLock,
Code::Semicolon => KeyCode::Semicolon,
Code::NumpadEqual => KeyCode::Equal,
Code::Comma => KeyCode::Comma,
Code::Period => KeyCode::Period,
Code::Slash => KeyCode::Slash,
Code::Backquote => KeyCode::Backquote,
Code::BracketLeft => KeyCode::BracketLeft,
Code::Backslash => KeyCode::Backslash,
Code::BracketRight => KeyCode::BracketRight,
Code::Quote => KeyCode::Quote,
Code::IntlBackslash => KeyCode::IntlBackslash,
Code::Power => KeyCode::Power,
Code::NumpadEnter => KeyCode::NumpadEnter,
            key => panic!("Failed to convert {:?} to tao::keyboard::KeyCode, try using tao::keyboard::KeyCode directly", key),
        }
}
