//! Cross-platform clipboard abstraction.
//!
//! Provides a [`Clipboard`] trait that framework adapters implement
//! to read/write text from the system clipboard.

/// A cross-platform clipboard interface.
///
/// Implementations are provided by framework adapter crates:
/// - `quoin-ui-gpui` uses GPUI's `cx.write_to_clipboard()`
/// - Web adapters (future) use `web_sys::Clipboard`
/// A cross-platform clipboard interface.
///
/// # Example
///
/// ```ignore
/// let clipboard = GpuiClipboard::new(
///     |text| cx.write_to_clipboard(ClipboardItem::new_string(text)),
///     || cx.read_from_clipboard().map(|item| item.text())
/// );
/// clipboard.write_text("Hello");
/// ```
pub trait Clipboard: Send + Sync {
    /// Write text to the system clipboard.
    fn write_text(&self, text: &str);

    /// Read text from the system clipboard.
    ///
    /// Returns `None` if the clipboard is empty, inaccessible, or does not
    /// contain text data.
    fn read_text(&self) -> Option<String>;
}
