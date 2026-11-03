//! Lock-free theme tokens for the Floem backend.
//!
//! Fifteen shadcn-style tokens, each stored as one [`AtomicU32`] (`0xRRGGBBAA`,
//! alpha in the low byte — the same packing [`quoin_style::HexLit`] uses).
//! Reads are `Relaxed` loads, so a token call is a single atomic load with no
//! lock and no allocation; `set_palette` stores all fifteen surface and all
//! fifteen text values in one pass.
//!
//! # Why two tables
//!
//! The GPUI backend (and the shadcn palette it baked in before the style IR)
//! distinguishes the *surface* role of a token (`bg-secondary` → `#1e293b`)
//! from its *text* role (`text-secondary` → `#94a3b8`): a de-emphasized label
//! is not the surface color. To keep fresh-application parity, the Floem
//! backend models both roles: [`Token::surface`] reads the surface table,
//! [`Token::text`] the text one, and [`set_palette`] swaps both in one pass.
//! Each slot read is self-consistent, so a torn multi-token palette is not
//! observable.
//!
//! Backends that have no role distinction simply call [`Token::surface`].

use core::sync::atomic::{AtomicU32, Ordering};

use floem::peniko::Color;
use quoin_style::{HexLit, Token};

/// The fifteen initial surface values, in [`Token::ALL`] order
/// (`quoin_style::Token::surface_hex`).
const DEFAULT_SURFACE: [u32; 15] = [
    0x63_66_f1_ff, // primary
    0x1e_29_3b_ff, // secondary
    0x0f_17_2a_ff, // background
    0xfa_fa_fa_ff, // foreground
    0x1e_29_3b_ff, // muted
    0x1e_29_3b_ff, // muted-foreground (surface slot; the text slot carries the label color)
    0x63_66_f1_ff, // accent
    0x1e_3a_5f_ff, // info
    0x14_53_2d_ff, // success
    0x42_20_06_ff, // warning
    0x45_0a_0a_ff, // danger
    0x1e_29_3b_ff, // border
    0x1e_29_3b_ff, // input
    0x63_66_f1_ff, // ring
    0x1e_29_3b_ff, // card
];

/// The fifteen initial text-role values, in [`Token::ALL`] order
/// (`quoin_style::Token::text_hex`, the GPUI `text-*` approximations).
const DEFAULT_TEXT: [u32; 15] = [
    0x63_66_f1_ff, // primary
    0x94_a3_b8_ff, // secondary
    0xfa_fa_fa_ff, // background
    0xfa_fa_fa_ff, // foreground
    0x94_a3_b8_ff, // muted
    0x94_a3_b8_ff, // muted-foreground
    0x63_66_f1_ff, // accent
    0x3b_82_f6_ff, // info
    0x22_c5_5e_ff, // success
    0xea_b3_08_ff, // warning
    0xef_44_44_ff, // danger
    0xe2_e8_f0_ff, // border
    0x1e_29_3b_ff, // input
    0x1e_29_3b_ff, // ring
    0x1e_29_3b_ff, // card
];

/// Packs a `u32` array into a static atomic array at compile time: the
/// palette is live from the first instruction, no lazy init, no locks.
const fn init_slots(values: [u32; 15]) -> [AtomicU32; 15] {
    let mut out = [const { AtomicU32::new(0) }; 15];
    let mut i = 0;
    while i < 15 {
        out[i] = AtomicU32::new(values[i]);
        i += 1;
    }
    out
}

static SURFACE: [AtomicU32; 15] = init_slots(DEFAULT_SURFACE);
static TEXT: [AtomicU32; 15] = init_slots(DEFAULT_TEXT);

/// Reads one packed color slot as a Floem [`Color`].
#[inline]
fn read(table: &[AtomicU32; 15], index: usize) -> Color {
    let packed = table[index].load(Ordering::Relaxed);
    Color::from_rgba8(
        ((packed >> 24) & 0xff) as u8,
        ((packed >> 16) & 0xff) as u8,
        ((packed >> 8) & 0xff) as u8,
        (packed & 0xff) as u8,
    )
}

/// The slot index of a token (its position in [`Token::ALL`]).
///
/// `Token` derives `PartialEq`, so this is a plain fifteen-entry scan; the
/// optimizer turns it into a jump table at the call sites.
#[inline]
fn slot(token: Token) -> usize {
    Token::ALL
        .iter()
        .position(|candidate| *candidate == token)
        .unwrap_or(0)
}

/// Every surface-role token function: `primary()`, `secondary()`, …
macro_rules! token_fns {
    ($(($fn_name:ident, $variant:ident)),+ $(,)?) => {
        $(
            #[doc = concat!("The `", stringify!($fn_name), "` theme token (surface role).")]
            #[must_use]
            #[inline]
            pub fn $fn_name() -> Color {
                read(&SURFACE, slot(Token::$variant))
            }
        )+
    };
}

token_fns! {
    (primary, Primary),
    (secondary, Secondary),
    (background, Background),
    (foreground, Foreground),
    (muted, Muted),
    (muted_foreground, MutedForeground),
    (accent, Accent),
    (info, Info),
    (success, Success),
    (warning, Warning),
    (danger, Danger),
    (border, Border),
    (input, Input),
    (ring, Ring),
    (card, Card),
}

/// Text-role variants: the colors the GPUI backend uses for `text-*` classes.
macro_rules! text_token_fns {
    ($(($fn_name:ident, $variant:ident)),+ $(,)?) => {
        $(
            #[doc = concat!("The `", stringify!($fn_name), "` theme token (text role, matching GPUI's `text-*` approximation).")]
            #[must_use]
            #[inline]
            pub fn $fn_name() -> Color {
                read(&TEXT, slot(Token::$variant))
            }
        )+
    };
}

text_token_fns! {
    (primary_text, Primary),
    (secondary_text, Secondary),
    (background_text, Background),
    (foreground_text, Foreground),
    (muted_text, Muted),
    (muted_foreground_text, MutedForeground),
    (accent_text, Accent),
    (info_text, Info),
    (success_text, Success),
    (warning_text, Warning),
    (danger_text, Danger),
    (border_text, Border),
    (input_text, Input),
    (ring_text, Ring),
    (card_text, Card),
}

/// Reads a style-IR token in the given role as a Floem [`Color`].
///
/// This is what the emitter's `Col::Token` lowering calls; the concrete
/// functions above exist for hand-written adapters that want named access.
/// (`text_role = false` selects the surface table.)
#[must_use]
#[inline]
pub fn token_color(token: Token, text_role: bool) -> Color {
    if text_role {
        read(&TEXT, slot(token))
    } else {
        read(&SURFACE, slot(token))
    }
}

/// A full palette: fifteen surface and fifteen text values, packed
/// `0xRRGGBBAA`, in [`Token::ALL`] order.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Surface-role colors (`bg-*`, borders, backgrounds).
    pub surface: [u32; 15],
    /// Text-role colors (`text-*`).
    pub text: [u32; 15],
}

impl Default for Palette {
    fn default() -> Self {
        Self::shadcn_dark()
    }
}

impl Palette {
    /// Builds a palette from the default shadcn dark values of the style IR.
    ///
    /// The surface roles use [`Token::surface_hex`] and the text roles
    /// [`Token::text_hex`], so a fresh application is color-parity with the
    /// GPUI backend.
    #[must_use]
    pub const fn shadcn_dark() -> Self {
        let mut surface = [0u32; 15];
        let mut text = [0u32; 15];
        let mut i = 0;
        while i < 15 {
            surface[i] = Token::ALL[i].surface_hex().value;
            text[i] = Token::ALL[i].text_hex().value;
            i += 1;
        }
        Self { surface, text }
    }
}

/// Replaces the whole palette in one pass (thirty relaxed stores).
pub fn set_palette(palette: &Palette) {
    for (index, value) in palette.surface.iter().enumerate() {
        SURFACE[index].store(*value, Ordering::Relaxed);
    }
    for (index, value) in palette.text.iter().enumerate() {
        TEXT[index].store(*value, Ordering::Relaxed);
    }
}

/// Reads a token color by runtime index ([`Token::ALL`] order).
#[must_use]
pub fn token_by_index(index: usize, text_role: bool) -> Color {
    if text_role {
        read(&TEXT, index)
    } else {
        read(&SURFACE, index)
    }
}

/// The default palette (for tests and probes).
#[must_use]
pub const fn default_palette() -> Palette {
    Palette::shadcn_dark()
}

/// A literal from the style IR as a Floem [`Color`].
#[must_use]
#[inline]
pub const fn hex_color(lit: HexLit) -> Color {
    Color::from_rgba8(lit.red(), lit.green(), lit.blue(), lit.alpha())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    /// The palette lives in process-global atomics, so tests that read or
    /// write it must not run concurrently with each other.
    fn palette_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        match LOCK.get_or_init(|| Mutex::new(())).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[test]
    fn defaults_match_shadcn_dark() {
        let _guard = palette_lock();
        set_palette(&Palette::shadcn_dark());
        assert_eq!(primary(), Color::from_rgba8(0x63, 0x66, 0xf1, 0xff));
        assert_eq!(background(), Color::from_rgba8(0x0f, 0x17, 0x2a, 0xff));
        // text-role secondary is the lighter label color, not the surface one
        assert_eq!(secondary_text(), Color::from_rgba8(0x94, 0xa3, 0xb8, 0xff));
        assert_eq!(secondary(), Color::from_rgba8(0x1e, 0x29, 0x3b, 0xff));
    }

    #[test]
    fn set_palette_swaps_every_slot() {
        let _guard = palette_lock();
        let mut palette = Palette::default();
        palette.surface[0] = 0x12_34_56_ff;
        palette.text[14] = 0xaa_bb_cc_dd;
        set_palette(&palette);
        assert_eq!(primary(), Color::from_rgba8(0x12, 0x34, 0x56, 0xff));
        assert_eq!(card_text(), Color::from_rgba8(0xaa, 0xbb, 0xcc, 0xdd));
        // restore for other tests
        set_palette(&Palette::default());
    }

    #[test]
    fn token_color_matches_named_access() {
        assert_eq!(token_color(Token::Danger, false), danger());
        assert_eq!(
            token_color(Token::MutedForeground, true),
            muted_foreground_text()
        );
    }
}
