//! Plain clickable menu item.

use std::cell::RefCell;
use std::rc::Rc;

use crate::accelerator::Accelerator;
use crate::items::{MenuChild, IsMenuItem};
use crate::menu::menu_id::MenuId;
use crate::items::sealed::IsMenuItemBase;
use crate::items::MenuItemKind;

/// A plain text menu item.
#[derive(Clone, Debug)]
pub struct MenuItem {
    pub(crate) id: Rc<MenuId>,
    pub(crate) inner: Rc<RefCell<MenuChild>>,
}

impl IsMenuItemBase for MenuItem {}
impl IsMenuItem for MenuItem {
    fn kind(&self) -> MenuItemKind {
        MenuItemKind::MenuItem(self.clone())
    }

    fn id(&self) -> &MenuId {
        &self.id
    }

    fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}

impl MenuItem {
    /// Creates a new item with an auto-generated id.
    pub fn new<S: AsRef<str>>(text: S, enabled: bool, accelerator: Option<Accelerator>) -> Self {
        Self::with_id(crate::next_id(), text, enabled, accelerator)
    }

    /// Creates a new item with the given id.
    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        accelerator: Option<Accelerator>,
    ) -> Self {
        let id = id.into();
        Self {
            id: Rc::new(id.clone()),
            inner: Rc::new(RefCell::new(MenuChild {
                id,
                text: text.as_ref().to_string(),
                enabled,
                accelerator,
                ..MenuChild::default()
            })),
        }
    }

    /// Returns the item id.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Returns the visible label.
    pub fn text(&self) -> String {
        self.inner.borrow().text.clone()
    }

    /// Sets the visible label.
    pub fn set_text<S: AsRef<str>>(&self, text: S) {
        self.inner.borrow_mut().text = text.as_ref().to_string();
    }

    /// Whether the item is enabled.
    pub fn is_enabled(&self) -> bool {
        self.inner.borrow().enabled
    }

    /// Enables or disables the item.
    pub fn set_enabled(&self, enabled: bool) {
        self.inner.borrow_mut().enabled = enabled;
    }

    /// Sets the keyboard shortcut.
    pub fn set_accelerator(&self, accelerator: Option<Accelerator>) {
        self.inner.borrow_mut().accelerator = accelerator;
    }

    /// Converts the item into its menu ID.
    pub fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}
