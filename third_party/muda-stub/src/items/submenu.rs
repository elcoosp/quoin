//! Submenu: a menu item holding child items.

use std::cell::RefCell;
use std::rc::Rc;

use crate::icon::{Icon, NativeIcon};
use crate::items::sealed::IsMenuItemBase;
use crate::items::{IsMenuItem, MenuChild, MenuItemKind};
use crate::menu::menu_id::MenuId;

/// A menu item which expands to show child items.
#[derive(Clone, Debug)]
pub struct Submenu {
    pub(crate) id: Rc<MenuId>,
    pub(crate) inner: Rc<RefCell<MenuChild>>,
}

impl IsMenuItemBase for Submenu {}
impl IsMenuItem for Submenu {
    fn kind(&self) -> MenuItemKind {
        MenuItemKind::Submenu(self.clone())
    }

    fn id(&self) -> &MenuId {
        &self.id
    }

    fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}

impl Submenu {
    /// Creates a new submenu with an auto-generated id.
    pub fn new<S: AsRef<str>>(text: S, enabled: bool) -> Self {
        Self::with_id(crate::next_id(), text, enabled)
    }

    /// Creates a new submenu with the given id.
    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S, enabled: bool) -> Self {
        let id = id.into();
        Self {
            id: Rc::new(id.clone()),
            inner: Rc::new(RefCell::new(MenuChild {
                id,
                text: text.as_ref().to_string(),
                enabled,
                ..MenuChild::default()
            })),
        }
    }

    /// Creates a new submenu pre-populated with items.
    pub fn with_items<S: AsRef<str>>(text: S, enabled: bool, items: &[&dyn IsMenuItem]) -> Self {
        let submenu = Self::new(text, enabled);
        for item in items {
            let _ = submenu.append(*item);
        }
        submenu
    }

    /// Creates a new submenu with an id, pre-populated with items.
    pub fn with_id_and_items<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        items: &[&dyn IsMenuItem],
    ) -> Self {
        let submenu = Self::with_id(id, text, enabled);
        for item in items {
            let _ = submenu.append(*item);
        }
        submenu
    }

    /// Returns the submenu id.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Appends a child item. In-memory only; always succeeds.
    pub fn append(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().children.push(item.kind());
        Ok(())
    }

    /// Appends several child items.
    pub fn append_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        for item in items {
            self.append(*item)?;
        }
        Ok(())
    }

    /// Prepends a child item.
    pub fn prepend(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().children.insert(0, item.kind());
        Ok(())
    }

    /// Prepends several child items.
    pub fn prepend_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        for (i, item) in items.iter().enumerate() {
            self.inner.borrow_mut().children.insert(i, (*item).kind());
        }
        Ok(())
    }

    /// Inserts a child item at `position`.
    pub fn insert(&self, item: &dyn IsMenuItem, position: usize) -> crate::Result<()> {
        let mut inner = self.inner.borrow_mut();
        let position = position.min(inner.children.len());
        inner.children.insert(position, item.kind());
        Ok(())
    }

    /// Inserts several child items starting at `position`.
    pub fn insert_items(&self, items: &[&dyn IsMenuItem], position: usize) -> crate::Result<()> {
        for (i, item) in items.iter().enumerate() {
            self.insert(*item, position + i)?;
        }
        Ok(())
    }

    /// Removes a child item (matched by id).
    pub fn remove(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        let id = item.id();
        self.inner.borrow_mut().children.retain(|k| k.id() != id);
        Ok(())
    }

    /// Removes and returns the child item at `position`.
    pub fn remove_at(&self, position: usize) -> Option<MenuItemKind> {
        let mut inner = self.inner.borrow_mut();
        if position < inner.children.len() {
            Some(inner.children.remove(position))
        } else {
            None
        }
    }

    /// Returns the child items as an owned list of kinds.
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.inner.borrow().children.clone()
    }

    /// Returns the visible label.
    pub fn text(&self) -> String {
        self.inner.borrow().text.clone()
    }

    /// Sets the visible label.
    pub fn set_text<S: AsRef<str>>(&self, text: S) {
        self.inner.borrow_mut().text = text.as_ref().to_string();
    }

    /// Whether the submenu is enabled.
    pub fn is_enabled(&self) -> bool {
        self.inner.borrow().enabled
    }

    /// Enables or disables the submenu.
    pub fn set_enabled(&self, enabled: bool) {
        self.inner.borrow_mut().enabled = enabled;
    }

    /// Sets (or clears) the custom icon.
    pub fn set_icon(&self, icon: Option<Icon>) {
        self.inner.borrow_mut().icon = icon;
    }

    /// Sets (or clears) the native icon name.
    pub fn set_native_icon(&self, native_icon: Option<NativeIcon>) {
        self.inner.borrow_mut().native_icon = native_icon;
    }

    /// Native menu-bar attachment: runtime error on the stub.
    pub fn set_as_windows_menu_for_nsapp(&self) -> crate::Result<()> {
        Err(crate::Error::NotSupported)
    }

    /// Native help-menu attachment: runtime error on the stub.
    pub fn set_as_help_menu_for_nsapp(&self) -> crate::Result<()> {
        Err(crate::Error::NotSupported)
    }

    /// Converts the submenu into its menu ID.
    pub fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}
