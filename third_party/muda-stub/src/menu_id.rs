//! Menu item ids: opaque string wrappers, in-memory only.

use std::convert::Infallible;
use std::str::FromStr;

/// Id of a menu or menu item.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct MenuId(pub String);

impl MenuId {
    /// Creates a new id from any string-ish value.
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        Self(id.as_ref().to_string())
    }
}

impl AsRef<str> for MenuId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T: ToString> From<T> for MenuId {
    fn from(value: T) -> Self {
        Self::new(value.to_string())
    }
}

impl FromStr for MenuId {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl PartialEq<&str> for MenuId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&str> for &MenuId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
