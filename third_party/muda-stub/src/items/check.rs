//! Checkable menu item.

use std::cell::RefCell;
use std::rc::Rc;

use crate::accelerator::Accelerator;
use crate::items::sealed::IsMenuItemBase;
use crate::items::{IsMenuItem, MenuChild, MenuItemKind};
use crate::menu::menu_id::MenuId;

/// A menu item with a checkbox.
#[derive(Clone, Debug)]
pub struct CheckMenuItem {
    pub(crate) id: Rc<MenuId>,
    pub(crate) inner: Rc<RefCell<MenuChild>>,
}

impl IsMenuItemBase for CheckMenuItem {}
impl IsMenuItem for CheckMenuItem {
    fn kind(&self) -> MenuItemKind {
        MenuItemKind::Check(self.clone())
    }

    fn id(&self) -> &MenuId {
        &self.id
    }

    fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}

impl CheckMenuItem {
    /// Creates a new check item with an auto-generated id.
    pub fn new<S: AsRef<str>>(
        text: S,
        enabled: bool,
        checked: bool,
        accelerator: Option<Accelerator>,
    ) -> Self {
        Self::with_id(crate::next_id(), text, enabled, checked, accelerator)
    }

    /// Creates a new check item with the given id.
    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        checked: bool,
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
                checked,
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

    /// Whether the item is checked.
    pub fn is_checked(&self) -> bool {
        self.inner.borrow().checked
    }

    /// Sets the check state.
    pub fn set_checked(&self, checked: bool) {
        self.inner.borrow_mut().checked = checked;
    }

    /// Converts the item into its menu ID.
    pub fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}
