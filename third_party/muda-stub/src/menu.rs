//! Root menu container, menu events and theme (in-memory only).

pub(crate) mod menu_id {
    //! Re-export path so `crate::menu::menu_id::MenuId` matches the layout
    //! used by the item modules.
    pub use crate::menu_id::MenuId;
}

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;

use crate::items::{IsMenuItem, MenuItemKind};
use crate::menu_id::MenuId;

/// In-memory menu tree shared behind [`Menu`].
#[derive(Debug, Default)]
pub(crate) struct MenuData {
    /// Child items in order.
    pub items: Vec<MenuItemKind>,
}

/// Describes a menu event emitted when a menu item is activated.
#[derive(Debug, Clone)]
pub struct MenuEvent {
    /// Id of the menu item which triggered this event.
    pub id: MenuId,
}

type EventHandler = Box<dyn Fn(MenuEvent) + Send + Sync + 'static>;

static EVENT_HANDLER: Mutex<Option<EventHandler>> = Mutex::new(None);

impl MenuEvent {
    /// Sets the global menu event handler.
    ///
    /// The stub never fires native menu events, so the handler is only
    /// stored (replaced wholesale by later calls).
    ///
    /// # Panics
    /// Panics if the handler mutex is poisoned.
    pub fn set_event_handler<F: Fn(MenuEvent) + Send + Sync + 'static>(f: Option<F>) {
        let mut guard = EVENT_HANDLER.lock().expect("menu event handler lock");
        *guard = f.map(|f| Box::new(f) as EventHandler);
    }
}

/// The window menu bar theme (API parity; unused by the stub).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTheme {
    /// Dark menu theme.
    Dark,
    /// Light menu theme.
    Light,
    /// Follow the system theme.
    Auto,
}

/// A root menu container, stored entirely in memory.
///
/// Mirrors `muda::Menu`'s construction/editing API. The `init_for_*` /
/// `show_for_*` family (native menu bar attachment) returns
/// [`crate::Error::NotSupported`] because the stub never talks to a native
/// menu bar.
#[derive(Clone, Default)]
pub struct Menu {
    id: Rc<MenuId>,
    inner: Rc<RefCell<MenuData>>,
}

impl std::fmt::Debug for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menu").field("id", &self.id).finish()
    }
}

impl Menu {
    /// Creates a new menu.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new menu with the given id.
    pub fn with_id<I: Into<MenuId>>(id: I) -> Self {
        Self {
            id: Rc::new(id.into()),
            inner: Rc::new(RefCell::new(MenuData::default())),
        }
    }

    /// Creates a new menu pre-populated with items.
    pub fn with_items(items: &[&dyn IsMenuItem]) -> Self {
        let menu = Self::new();
        let _ = menu.append_items(items);
        menu
    }

    /// Creates a new menu with an id, pre-populated with items.
    pub fn with_id_and_items<I: Into<MenuId>>(id: I, items: &[&dyn IsMenuItem]) -> Self {
        let menu = Self::with_id(id);
        let _ = menu.append_items(items);
        menu
    }

    /// Returns the menu id.
    pub fn id(&self) -> &MenuId {
        &self.id
    }

    /// Appends an item. In-memory only; always succeeds.
    pub fn append(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().items.push(item.kind());
        Ok(())
    }

    /// Appends several items.
    pub fn append_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        for item in items {
            self.append(*item)?;
        }
        Ok(())
    }

    /// Prepends an item.
    pub fn prepend(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        self.inner.borrow_mut().items.insert(0, item.kind());
        Ok(())
    }

    /// Prepends several items.
    pub fn prepend_items(&self, items: &[&dyn IsMenuItem]) -> crate::Result<()> {
        for (i, item) in items.iter().enumerate() {
            self.inner.borrow_mut().items.insert(i, (*item).kind());
        }
        Ok(())
    }

    /// Inserts an item at `position`.
    pub fn insert(&self, item: &dyn IsMenuItem, position: usize) -> crate::Result<()> {
        let mut inner = self.inner.borrow_mut();
        let position = position.min(inner.items.len());
        inner.items.insert(position, item.kind());
        Ok(())
    }

    /// Inserts several items starting at `position`.
    pub fn insert_items(&self, items: &[&dyn IsMenuItem], position: usize) -> crate::Result<()> {
        for (i, item) in items.iter().enumerate() {
            self.insert(*item, position + i)?;
        }
        Ok(())
    }

    /// Removes an item (matched by id).
    pub fn remove(&self, item: &dyn IsMenuItem) -> crate::Result<()> {
        let id = item.id();
        self.inner.borrow_mut().items.retain(|k| k.id() != id);
        Ok(())
    }

    /// Removes and returns the item at `position`.
    pub fn remove_at(&self, position: usize) -> Option<MenuItemKind> {
        let mut inner = self.inner.borrow_mut();
        if position < inner.items.len() {
            Some(inner.items.remove(position))
        } else {
            None
        }
    }

    /// Returns the menu items as an owned list of kinds.
    pub fn items(&self) -> Vec<MenuItemKind> {
        self.inner.borrow().items.clone()
    }

    /// Native menu-bar attachment: runtime error on the stub.
    pub fn init_for_gtk_window<W, C>(&self, _window: &W, _context_menu: Option<&C>) -> crate::Result<()>
    where
        W: std::fmt::Debug,
        C: std::fmt::Debug,
    {
        Err(crate::Error::NotSupported)
    }

    /// Show the GTK menu bar: runtime error on the stub.
    pub fn show_for_gtk_window<W>(&self, _window: &W) -> crate::Result<()> {
        Err(crate::Error::NotSupported)
    }

    /// Hide the GTK menu bar: runtime error on the stub.
    pub fn hide_for_gtk_window<W>(&self, _window: &W) -> crate::Result<()> {
        Err(crate::Error::NotSupported)
    }

    /// Detach the GTK menu bar: runtime error on the stub.
    pub fn remove_for_gtk_window<W>(&self, _window: &W) -> crate::Result<()> {
        Err(crate::Error::NotSupported)
    }
}
