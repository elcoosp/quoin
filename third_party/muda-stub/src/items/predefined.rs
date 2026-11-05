//! Well-known action menu items (separator, copy, quit, …).

use std::cell::RefCell;
use std::rc::Rc;

use crate::about_metadata::AboutMetadata;
use crate::items::sealed::IsMenuItemBase;
use crate::items::{IsMenuItem, MenuChild, MenuItemKind, PredefinedItemKind};
use crate::menu::menu_id::MenuId;

/// A menu item for a well-known action. The action itself is never executed
/// by the stub (native menus are unsupported); the item is retained so menu
/// trees can be inspected and rendered by in-app menu views.
#[derive(Clone, Debug)]
pub struct PredefinedMenuItem {
    pub(crate) id: Rc<MenuId>,
    pub(crate) inner: Rc<RefCell<MenuChild>>,
}

impl IsMenuItemBase for PredefinedMenuItem {}
impl IsMenuItem for PredefinedMenuItem {
    fn kind(&self) -> MenuItemKind {
        MenuItemKind::Predefined(self.clone())
    }

    fn id(&self) -> &MenuId {
        &self.id
    }

    fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}

fn predefined(kind: PredefinedItemKind) -> PredefinedMenuItem {
    PredefinedMenuItem {
        id: Rc::new(crate::next_id()),
        inner: Rc::new(RefCell::new(MenuChild {
            predefined: Some(kind),
            ..MenuChild::default()
        })),
    }
}

impl PredefinedMenuItem {
    /// A visual separator.
    pub fn separator() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Separator)
    }

    /// Copy.
    pub fn copy() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Copy)
    }

    /// Cut.
    pub fn cut() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Cut)
    }

    /// Paste.
    pub fn paste() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Paste)
    }

    /// Select all.
    pub fn select_all() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::SelectAll)
    }

    /// Undo.
    pub fn undo() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Undo)
    }

    /// Redo.
    pub fn redo() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Redo)
    }

    /// Minimize the window.
    pub fn minimize() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Minimize)
    }

    /// Maximize the window.
    pub fn maximize() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Maximize)
    }

    /// Toggle fullscreen.
    pub fn fullscreen() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Fullscreen)
    }

    /// Hide the window.
    pub fn hide() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Hide)
    }

    /// Hide all other windows.
    pub fn hide_others() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::HideOthers)
    }

    /// Show all windows.
    pub fn show_all() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::ShowAll)
    }

    /// Close the window.
    pub fn close_window() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::CloseWindow)
    }

    /// Quit the application.
    pub fn quit() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Quit)
    }

    /// About box with optional metadata.
    pub fn about(metadata: Option<AboutMetadata>) -> PredefinedMenuItem {
        predefined(PredefinedItemKind::About(metadata))
    }

    /// macOS services submenu.
    pub fn services() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::Services)
    }

    /// Bring all windows to front.
    pub fn bring_all_to_front() -> PredefinedMenuItem {
        predefined(PredefinedItemKind::BringAllToFront)
    }

    /// Returns the item id.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Returns the visible label (empty for separators).
    pub fn text(&self) -> String {
        self.inner.borrow().text.clone()
    }

    /// Sets the visible label.
    pub fn set_text<S: AsRef<str>>(&self, text: S) {
        self.inner.borrow_mut().text = text.as_ref().to_string();
    }

    /// Converts the item into its menu ID.
    pub fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}
