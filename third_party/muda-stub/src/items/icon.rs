//! Menu item with a custom or native icon.

use std::cell::RefCell;
use std::rc::Rc;

use crate::accelerator::Accelerator;
use crate::icon::{Icon, NativeIcon};
use crate::items::sealed::IsMenuItemBase;
use crate::items::{IsMenuItem, MenuChild, MenuItemKind};
use crate::menu::menu_id::MenuId;

/// A menu item with an icon.
#[derive(Clone, Debug)]
pub struct IconMenuItem {
    pub(crate) id: Rc<MenuId>,
    pub(crate) inner: Rc<RefCell<MenuChild>>,
}

impl IsMenuItemBase for IconMenuItem {}
impl IsMenuItem for IconMenuItem {
    fn kind(&self) -> MenuItemKind {
        MenuItemKind::Icon(self.clone())
    }

    fn id(&self) -> &MenuId {
        &self.id
    }

    fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}

impl IconMenuItem {
    /// Creates a new icon item with an auto-generated id.
    pub fn new<S: AsRef<str>>(
        text: S,
        enabled: bool,
        icon: Option<Icon>,
        accelerator: Option<Accelerator>,
    ) -> Self {
        Self::with_id(crate::next_id(), text, enabled, icon, accelerator)
    }

    /// Creates a new icon item with the given id.
    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        icon: Option<Icon>,
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
                icon,
                ..MenuChild::default()
            })),
        }
    }

    /// Creates a new icon item with a native icon name.
    pub fn with_native_icon<S: AsRef<str>>(
        text: S,
        enabled: bool,
        native_icon: Option<NativeIcon>,
        accelerator: Option<Accelerator>,
    ) -> Self {
        Self::with_id_and_native_icon(
            crate::next_id(),
            text,
            enabled,
            native_icon,
            accelerator,
        )
    }

    /// Creates a new icon item with an id and a native icon name.
    pub fn with_id_and_native_icon<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        native_icon: Option<NativeIcon>,
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
                native_icon,
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

    /// Sets (or clears) the custom icon.
    pub fn set_icon(&self, icon: Option<Icon>) {
        self.inner.borrow_mut().icon = icon;
    }

    /// Sets (or clears) the native icon name.
    pub fn set_native_icon(&self, native_icon: Option<NativeIcon>) {
        self.inner.borrow_mut().native_icon = native_icon;
    }

    /// Converts the item into its menu ID.
    pub fn into_id(self) -> MenuId {
        (*self.id).clone()
    }
}
