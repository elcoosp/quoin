//! The shared web keystroke runtime (`quoin::keys`) — the `keys { .. }`
//! matcher and window-listener attach behind the Leptos and Dioxus
//! lowerings.
//!
//! The macro parses the GPUI-grammar keystroke string at expansion time into
//! the exact modifier mask plus the DOM key name; each `keys` entry becomes
//! one `if` arm inside a **single** window-level `keydown` listener per
//! component (the web counterpart of the Floem backend's one-listener
//! contract). The per-keystroke cost is one mask comparison plus one key
//! comparison — the same cost class as a hand-written DOM `keydown` handler.
//!
//! Semantics mirror the Floem matcher ([`quoin::floem_keys`]):
//! - modifiers are matched **exactly** (`"ctrl-shift-d"` does not fire when
//!   Alt is also held),
//! - character keys match case-insensitively (Shift produces uppercase
//!   characters on real keyboards; DOM `key` values are ASCII here),
//! - `cmd`/`super`/`meta` is its own mask flag — the macro folds it into
//!   `quoin::ev::Key::ctrl` only when building the shared handler payload.
//!
//! # Platform boundary
//!
//! The listener attach targets the browser (the web backends' render
//! target): on `wasm32` it registers a real `keydown` listener on the DOM
//! window; on native targets (SSR, desktop) there is no DOM window, so
//! [`attach_window_keydown`] registers nothing and [`KeydownHandle::remove`]
//! is a no-op — the same documented fallback class as the adapters'
//! `clipboard_write_text` SSR path. SSR builds never route keyboard input,
//! exactly as they never route clicks.

/// The four keyboard modifier flags of a keystroke or event, as bits (the
/// web counterpart of the Floem backend's `Modifiers` mask; clippy counts
/// raw `bool` fields, not bits — same shape as the macro-side `KeyFlags`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(pub u8);

impl Modifiers {
    /// Ctrl held.
    pub const CTRL: Self = Self(1 << 0);
    /// Shift held.
    pub const SHIFT: Self = Self(1 << 1);
    /// Alt/Option held.
    pub const ALT: Self = Self(1 << 2);
    /// Platform meta (Cmd/Super) held.
    pub const META: Self = Self(1 << 3);
    /// No modifier held.
    pub const EMPTY: Self = Self(0);

    /// Union of two masks (`a | b` mirrors the Floem bitflag grammar the
    /// emitted code uses).
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

/// A framework-free snapshot of one keyboard event: the fields the
/// `keys { .. }` matcher compares, converted from the framework's own event
/// object by the active web adapter.
///
/// `key` carries the DOM `KeyboardEvent.key` value the matcher compares
/// (`"d"`, `"Escape"`, `" "`); `code` carries the W3C-style physical code
/// name (`"KeyD"`, `"Escape"`) for diagnostics and the shared
/// `quoin::ev::Key` payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// The W3C-style physical code name (`"KeyD"`, `"Escape"`).
    pub code: Box<str>,
    /// The DOM `key` value (`"d"`, `"Escape"`, `" "`).
    pub key: Box<str>,
    /// The modifier flags held (matched exactly against the keystroke mask).
    pub modifiers: Modifiers,
}

impl KeyEvent {
    /// Builds a snapshot from the event's code name, key value, and modifier
    /// mask.
    #[must_use]
    pub fn new(code: impl Into<Box<str>>, key: impl Into<Box<str>>, modifiers: Modifiers) -> Self {
        Self {
            code: code.into(),
            key: key.into(),
            modifiers,
        }
    }

    /// Snapshots a browser `KeyboardEvent` (the web adapters' conversion
    /// path, only available on `wasm32`).
    #[cfg(target_arch = "wasm32")]
    #[must_use]
    pub fn from_web_event(ev: &web_sys::KeyboardEvent) -> Self {
        let mut modifiers = Modifiers::EMPTY;
        if ev.ctrl_key() {
            modifiers.0 |= Modifiers::CTRL.0;
        }
        if ev.shift_key() {
            modifiers.0 |= Modifiers::SHIFT.0;
        }
        if ev.alt_key() {
            modifiers.0 |= Modifiers::ALT.0;
        }
        if ev.meta_key() {
            modifiers.0 |= Modifiers::META.0;
        }
        Self::new(ev.code(), ev.key(), modifiers)
    }
}

/// Whether a `keydown` snapshot matches a parsed keystroke: exact modifier
/// mask plus a case-insensitive key comparison.
///
/// Character keys compare case-insensitively (Shift yields uppercase
/// characters on real keyboards); named keys compare by their fixed-case
/// DOM names, so the relaxed comparison never changes their outcome.
#[must_use]
#[inline]
pub fn keystroke_matches(ev: &KeyEvent, modifiers: Modifiers, expected_key: &str) -> bool {
    ev.modifiers == modifiers && ev.key.eq_ignore_ascii_case(expected_key)
}

/// A handle to the window-level `keydown` listener installed by
/// [`attach_window_keydown`]. Dropping the handle without [`remove`](Self::remove)
/// leaves the listener installed for the page's lifetime (the keys block
/// removes it through the component's cleanup hook).
pub struct KeydownHandle(KeydownHandleInner);

enum KeydownHandleInner {
    /// A live listener with its remover (a wasm build that found a window).
    #[cfg(target_arch = "wasm32")]
    Attached(Box<dyn FnOnce()>),
    /// No listener: native targets have no DOM window, and a wasm build
    /// without a window cannot attach one.
    Detached,
}

impl KeydownHandle {
    /// Removes the event listener. On native targets this is a no-op (there
    /// is no window to detach from).
    pub fn remove(self) {
        match self.0 {
            #[cfg(target_arch = "wasm32")]
            KeydownHandleInner::Attached(remover) => remover(),
            KeydownHandleInner::Detached => {}
        }
    }
}

/// Registers one window-level `keydown` listener that calls `handler` with
/// a [`KeyEvent`] snapshot per event. The returned handle detaches it.
///
/// On native targets (SSR, desktop) there is no DOM window: the handler is
/// accepted, never invoked, and nothing is registered — see the module docs
/// for the platform boundary.
#[must_use = "the listener stays installed until the handle's `remove` runs"]
pub fn attach_window_keydown(handler: impl Fn(&KeyEvent) + 'static) -> KeydownHandle {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return KeydownHandle(KeydownHandleInner::Detached);
        };
        let closure = wasm_bindgen::closure::Closure::new(move |ev: web_sys::Event| {
            let kb = ev.unchecked_into::<web_sys::KeyboardEvent>();
            handler(&KeyEvent::from_web_event(&kb));
        });
        let callback = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
        let _ = window.add_event_listener_with_callback("keydown", &callback);
        KeydownHandle(KeydownHandleInner::Attached(Box::new(move || {
            let _ = window.remove_event_listener_with_callback("keydown", &callback);
            drop(closure);
        })))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native platform boundary: no DOM window exists, so the handler is
        // consumed without ever being invoked (documented no-op).
        let _ = handler;
        KeydownHandle(KeydownHandleInner::Detached)
    }
}
