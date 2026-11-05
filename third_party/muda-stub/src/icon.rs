//! Icons and native icon names: retained as plain data, never loaded.

use std::fmt;

/// An RGBA icon, retained as raw bytes (mirrors `muda::Icon`).
///
/// The stub performs no image decoding or platform registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    /// RGBA8 pixel data, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
}

/// Errors returned when constructing an [`Icon`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadIcon {
    /// The RGBA byte length is not `width * height * 4`.
    ByteCountNotDivisibleBy4 {
        /// The offending byte count.
        byte_count: usize,
    },
    /// The RGBA byte length does not match the declared dimensions.
    DimensionsMismatch {
        /// Declared width.
        width: u32,
        /// Declared height.
        height: u32,
        /// The offending byte count.
        byte_count: usize,
    },
    /// The file could not be read (path-based construction).
    OsError(String),
}

impl fmt::Display for BadIcon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadIcon::ByteCountNotDivisibleBy4 { byte_count } => write!(
                f,
                "byte count must be divisible by 4 (got {byte_count})"
            ),
            BadIcon::DimensionsMismatch {
                width,
                height,
                byte_count,
            } => write!(
                f,
                "{byte_count} bytes do not fit {width}x{height} RGBA8"
            ),
            BadIcon::OsError(message) => write!(f, "icon load failed: {message}"),
        }
    }
}

impl std::error::Error for BadIcon {}

impl Icon {
    /// Builds an icon from raw RGBA8 bytes, validating the size.
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
        if rgba.len() % 4 != 0 {
            return Err(BadIcon::ByteCountNotDivisibleBy4 {
                byte_count: rgba.len(),
            });
        }
        let expected = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4);
        if rgba.len() != expected {
            return Err(BadIcon::DimensionsMismatch {
                width,
                height,
                byte_count: rgba.len(),
            });
        }
        Ok(Self {
            rgba,
            width,
            height,
        })
    }

    /// Reads an icon file from disk (platform icon decoding unsupported:
    /// only the raw bytes are retained).
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, BadIcon> {
        let bytes = std::fs::read(path).map_err(|e| BadIcon::OsError(e.to_string()))?;
        // Treat the file bytes as opaque 1x1 payload; decoding is not the
        // stub's business. Kept lenient so callers can still attach icons.
        let len = bytes.len().div_ceil(4) * 4;
        let mut rgba = bytes;
        rgba.resize(len.max(4), 0);
        Ok(Self {
            rgba,
            width: 1,
            height: 1,
        })
    }

    /// Windows resource icon: unsupported, always an error.
    pub fn from_resource(
        _ordinal: u16,
        _size: Option<(u32, u32)>,
    ) -> Result<Self, BadIcon> {
        Err(BadIcon::OsError("resource icons unsupported".into()))
    }
}

/// Names of platform-provided icons (mirrors `muda::NativeIcon`).
///
/// The stub never renders these; the variant list matches upstream so app
/// code compiles unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeIcon {
    /// Plus toolbar icon.
    Add,
    /// Advanced toolbar icon.
    Advanced,
    /// Bluetooth icon.
    Bluetooth,
    /// Bookmarks icon.
    Bookmarks,
    /// Caution icon.
    Caution,
    /// Color panel toolbar icon.
    ColorPanel,
    /// Column view icon.
    ColumnView,
    /// Computer icon.
    Computer,
    /// Enter full screen toolbar icon.
    EnterFullScreen,
    /// Everyone permissions icon.
    Everyone,
    /// Exit full screen toolbar icon.
    ExitFullScreen,
    /// Flow view icon.
    FlowView,
    /// Folder icon.
    Folder,
    /// Burnable folder icon.
    FolderBurnable,
    /// Smart folder icon.
    FolderSmart,
    /// Follow link freestanding icon.
    FollowLinkFreestanding,
    /// Font panel toolbar icon.
    FontPanel,
    /// Go left toolbar icon.
    GoLeft,
    /// Go right toolbar icon.
    GoRight,
    /// Home icon.
    Home,
    /// iChat theater toolbar icon.
    IChatTheater,
    /// Icon view icon.
    IconView,
    /// Info icon.
    Info,
    /// Invalid data freestanding icon.
    InvalidDataFreestanding,
    /// Left facing triangle icon.
    LeftFacingTriangle,
    /// List view icon.
    ListView,
    /// Locked lock icon.
    LockLocked,
    /// Unlocked lock icon.
    LockUnlocked,
    /// Mixed state menu icon.
    MenuMixedState,
    /// On-state menu icon.
    MenuOnState,
    /// MobileMe icon.
    MobileMe,
    /// Multiple documents icon.
    MultipleDocuments,
    /// Network icon.
    Network,
    /// General path icon.
    Path,
    /// General preferences icon.
    PreferencesGeneral,
    /// Quick look icon.
    QuickLook,
    /// Refresh freestanding icon.
    RefreshFreestanding,
    /// Refresh icon.
    Refresh,
    /// Minus toolbar icon.
    Remove,
    /// Reveal freestanding icon.
    RevealFreestanding,
    /// Right facing triangle icon.
    RightFacingTriangle,
    /// Share icon.
    Share,
    /// Slideshow icon.
    Slideshow,
    /// Smart badge icon.
    SmartBadge,
    /// Available small status icon.
    StatusAvailable,
    /// None small status icon.
    StatusNone,
    /// Partially available small status icon.
    StatusPartiallyAvailable,
    /// Unavailable small status icon.
    StatusUnavailable,
    /// Stop progress freestanding icon.
    StopProgressFreestanding,
    /// Stop progress icon.
    StopProgress,
    /// Empty trash icon.
    TrashEmpty,
    /// Full trash icon.
    TrashFull,
    /// User permissions icon.
    User,
    /// User accounts permissions icon.
    UserAccounts,
    /// User group permissions icon.
    UserGroup,
    /// User guest permissions icon.
    UserGuest,
}
