//! Menu item types: plain text, check, icon, predefined and submenu.

mod check;
mod icon;
mod normal;
mod predefined;
mod submenu;

pub use check::CheckMenuItem;
pub use icon::IconMenuItem;
pub use normal::MenuItem;
pub use predefined::PredefinedMenuItem;
pub use submenu::Submenu;

use crate::about_metadata::AboutMetadata;
use crate::accelerator::Accelerator;
use crate::icon::{Icon, NativeIcon};
use crate::menu::menu_id::MenuId;

/// Shared in-memory state behind every item type.
///
/// Some fields (`id`, `predefined`) are retained for API fidelity even
/// where Floem's accessors don't read them back through this stub.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct MenuChild {
    /// Item id.
    pub id: MenuId,
    /// Visible label.
    pub text: String,
    /// Whether the item responds to activation.
    pub enabled: bool,
    /// Keyboard shortcut, retained for display.
    pub accelerator: Option<Accelerator>,
    /// Check state (check items only).
    pub checked: bool,
    /// Custom icon (icon items only).
    pub icon: Option<Icon>,
    /// Native icon name (icon items only).
    pub native_icon: Option<NativeIcon>,
    /// Child items (submenus only).
    pub children: Vec<MenuItemKind>,
    /// Kind discriminator for predefined items.
    pub predefined: Option<PredefinedItemKind>,
}

/// Which well-known action a [`PredefinedMenuItem`] stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PredefinedItemKind {
    /// Visual separator.
    Separator,
    /// Copy.
    Copy,
    /// Cut.
    Cut,
    /// Paste.
    Paste,
    /// Select all.
    SelectAll,
    /// Undo.
    Undo,
    /// Redo.
    Redo,
    /// Minimize window.
    Minimize,
    /// Maximize window.
    Maximize,
    /// Toggle fullscreen.
    Fullscreen,
    /// Hide window.
    Hide,
    /// Hide other windows.
    HideOthers,
    /// Show all windows.
    ShowAll,
    /// Close window.
    CloseWindow,
    /// Quit application.
    Quit,
    /// About box.
    About(Option<AboutMetadata>),
    /// macOS services menu.
    Services,
    /// Bring all windows to front.
    BringAllToFront,
}

/// A trait that defines a generic item in a menu, which may be one of
/// [`MenuItemKind`].
pub trait IsMenuItem: sealed::IsMenuItemBase {
    /// Returns a [`MenuItemKind`] associated with this item.
    fn kind(&self) -> MenuItemKind;
    /// Returns a unique identifier associated with this menu item.
    fn id(&self) -> &MenuId;
    /// Convert this menu item into its menu ID.
    fn into_id(self) -> MenuId;
}

mod sealed {
    /// Sealing trait: only the stub's item types implement `IsMenuItem`.
    pub trait IsMenuItemBase {}
}

/// An enum of every item kind a menu can hold.
#[derive(Clone, Debug)]
pub enum MenuItemKind {
    /// Plain text item.
    MenuItem(MenuItem),
    /// Submenu with children.
    Submenu(Submenu),
    /// Well-known action item.
    Predefined(PredefinedMenuItem),
    /// Checkable item.
    Check(CheckMenuItem),
    /// Item with a custom or native icon.
    Icon(IconMenuItem),
}

impl MenuItemKind {
    /// Returns the item id, whatever the concrete kind.
    pub fn id(&self) -> &MenuId {
        match self {
            MenuItemKind::MenuItem(i) => i.id(),
            MenuItemKind::Submenu(i) => i.id(),
            MenuItemKind::Predefined(i) => i.id(),
            MenuItemKind::Check(i) => i.id(),
            MenuItemKind::Icon(i) => i.id(),
        }
    }

    /// Converts the item into its menu ID.
    pub fn into_id(self) -> MenuId {
        match self {
            MenuItemKind::MenuItem(i) => i.into_id(),
            MenuItemKind::Submenu(i) => i.into_id(),
            MenuItemKind::Predefined(i) => i.into_id(),
            MenuItemKind::Check(i) => i.into_id(),
            MenuItemKind::Icon(i) => i.into_id(),
        }
    }
}
