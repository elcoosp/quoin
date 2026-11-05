//! API-compatible in-memory stand-in for the `muda` menu crate (0.17.1).
//!
//! Provenance: quoin's root `Cargo.toml` `[patch.crates-io]` replaces `muda`
//! with this crate so the workspace builds without GTK 3 development files
//! (upstream `muda`'s Linux backend requires them). The originally committed
//! stub was a deliberately empty *resolution* stub which made `quoin-floem`
//! unbuildable; this version implements the menu API surface Floem actually
//! exercises as **in-memory data structures**:
//!
//! - constructors, `append`/`prepend`/`insert`/`remove`, `items()`, text,
//!   enabled/checked state, accelerators and icons all work in memory, so
//!   Floem's `MenuBuilder` layer and its Linux fallback `context_menu_view`
//!   (which renders menus from `items()` using Floem widgets) behave
//!   correctly;
//! - the *native-menu* operations (`init_for_*`, `show_for_*`, …) return
//!   [`Error::NotSupported`], matching the note in the root `Cargo.toml`:
//!   native menus are unsupported on Linux in this configuration;
//! - [`MenuEvent::set_event_handler`] stores the handler but nothing ever
//!   fires it, for the same reason.
//!
//! The public API mirrors muda 0.17.1 (types, signatures, trait shapes) so
//! downstream crates compile unchanged against either crate.

#![allow(clippy::module_name_repetitions)]

use std::sync::atomic::{AtomicU64, Ordering};

pub mod about_metadata;
pub mod accelerator;
pub mod error;
pub mod icon;
mod items;
pub mod menu;
pub mod menu_id;

pub use about_metadata::AboutMetadata;
pub use error::{Error, Result};
pub use icon::{BadIcon, Icon, NativeIcon};
pub use items::{
    CheckMenuItem, IconMenuItem, IsMenuItem, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu,
};
pub use menu::{Menu, MenuEvent, MenuTheme};
pub use menu_id::MenuId;

/// Auto-generated unique item id (`"muda-stub-<n>"`).
pub(crate) fn next_id() -> MenuId {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    MenuId::new(format!("muda-stub-{}", COUNTER.fetch_add(1, Ordering::Relaxed)))
}

/// DPI types mirroring muda's `muda::dpi` re-exports (logical/physical).
pub mod dpi {
    /// A position in logical pixels.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct LogicalPosition {
        /// Horizontal coordinate.
        pub x: f64,
        /// Vertical coordinate.
        pub y: f64,
    }

    /// A position in physical pixels.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct PhysicalPosition {
        /// Horizontal coordinate.
        pub x: f64,
        /// Vertical coordinate.
        pub y: f64,
    }

    /// Either a logical or a physical position.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Position {
        /// Logical pixels.
        Logical(LogicalPosition),
        /// Physical pixels.
        Physical(PhysicalPosition),
    }
}

/// Helper trait with methods to help creating a context menu.
///
/// The stub never attaches native menus, so every method is a no-op (or an
/// [`Error::NotSupported`] result where a `Result` is expected).
pub trait ContextMenu {
    /// Attach to a GTK window. Runtime error on the stub.
    fn init_for_gtk_window<W, C>(
        &self,
        _window: &W,
        _context_menu: Option<&C>,
    ) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Detach from the GTK window. Runtime error on the stub.
    fn remove_for_gtk_window<W>(&self, _window: &W) -> Result<()> {
        Err(Error::NotSupported)
    }

    /// Show the menu on a GTK window. No-op on the stub.
    fn show_context_menu_for_gtk_window<W>(
        &self,
        _window: &W,
        _position: Option<dpi::Position>,
    ) {
    }
}
