use crate::input_data::keyboard_types::Modifiers;
use dioxus_core::ScopeState;
use keyboard_types::Code;
use std::{rc::Rc, str::FromStr};

/// A struct that implements EvalProvider is sent through [`ScopeState`]'s provide_context function
/// so that [`use_eval`] can provide a platform agnostic interface for evaluating JavaScript code.
pub trait ShortcutProvider {
    fn new_shortcut(
        &self,
        cx: &ScopeState,
        accelerator: Accelerator,
        handler: Box<dyn FnMut() + 'static>,
    ) -> Result<Box<dyn Shortcut>, ShortcutRegistryError>;
}

pub trait Shortcut {
    fn remove(&mut self);
}

/// Get a closure that executes any JavaScript in the WebView context.
pub fn use_global_shortcut(
    cx: &ScopeState,
    accelerator: impl IntoAccelerator,
    handler: impl FnMut() + 'static,
) -> &Result<(), ShortcutRegistryError> {
    cx.use_hook(move || {
        let provider: Rc<dyn ShortcutProvider> = cx
            .consume_context()
            .expect("This platform does not support global shortcuts");
        provider.new_shortcut(cx, accelerator.accelerator()?, Box::new(handler))?;
        Ok(())
    })
}

pub struct Accelerator {
    pub modifiers: Modifiers,
    pub key: Code,
}

impl std::str::FromStr for Accelerator {
    type Err = AcceleratorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        accelerator_from_str(s)
    }
}

pub trait IntoAccelerator {
    fn accelerator(&self) -> Result<Accelerator, ShortcutRegistryError>;
}

struct ShortcutHandle {
    shortcut: Box<dyn Shortcut>,
}

impl Drop for ShortcutHandle {
    fn drop(&mut self) {
        self.shortcut.remove()
    }
}

impl IntoAccelerator for (Code, Modifiers) {
    fn accelerator(&self) -> Result<Accelerator, ShortcutRegistryError> {
        Ok(Accelerator {
            modifiers: self.1,
            key: self.0,
        })
    }
}

impl IntoAccelerator for (Modifiers, Code) {
    fn accelerator(&self) -> Result<Accelerator, ShortcutRegistryError> {
        Ok(Accelerator {
            modifiers: self.0,
            key: self.1,
        })
    }
}

impl IntoAccelerator for Code {
    fn accelerator(&self) -> Result<Accelerator, ShortcutRegistryError> {
        Ok(Accelerator {
            modifiers: Modifiers::empty(),
            key: *self,
        })
    }
}

impl IntoAccelerator for &str {
    fn accelerator(&self) -> Result<Accelerator, ShortcutRegistryError> {
        accelerator_from_str(self).map_err(ShortcutRegistryError::InvalidShortcut)
    }
}

#[non_exhaustive]
#[derive(Debug)]
/// An error that can occur when registering a shortcut.
pub enum ShortcutRegistryError {
    /// The shortcut is invalid.
    InvalidShortcut(AcceleratorParseError),
    /// An unknown error occurred.
    Other(Box<dyn std::error::Error>),
}

fn accelerator_from_str(accelerator: &str) -> Result<Accelerator, AcceleratorParseError> {
    let mut mods = Modifiers::empty();
    let mut key = Code::Unidentified;

    for raw in accelerator.split('+') {
        let token = raw.trim().to_string();
        if token.is_empty() {
            return Err(AcceleratorParseError::FoundEmptyToken);
        }

        match token.to_uppercase().as_str() {
            "OPTION" | "ALT" => {
                mods.set(Modifiers::ALT, true);
            }
            "CONTROL" | "CTRL" => {
                mods.set(Modifiers::CONTROL, true);
            }
            "COMMAND" | "CMD" | "SUPER" => {
                mods.set(Modifiers::SUPER, true);
            }
            "SHIFT" => {
                mods.set(Modifiers::SHIFT, true);
            }
            "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => {
                #[cfg(target_os = "macos")]
                mods.set(Modifiers::SUPER, true);
                #[cfg(not(target_os = "macos"))]
                mods.set(Modifiers::CONTROL, true);
            }
            _ => {
                // check if a main key has already been registered
                if key != Code::Unidentified {
                    return Err(AcceleratorParseError::MultipleMainKeys);
                }
                if let Ok(keycode) = Code::from_str(&token) {
                    match keycode {
                        Code::Unidentified => return Err(AcceleratorParseError::InvalidKeyCode),
                        _ => key = keycode,
                    }
                }
            }
        }
    }

    Ok(Accelerator {
        modifiers: mods,
        key,
    })
}

#[non_exhaustive]
#[derive(Debug)]
pub enum AcceleratorParseError {
    FoundEmptyToken,
    MultipleMainKeys,
    InvalidKeyCode,
}
