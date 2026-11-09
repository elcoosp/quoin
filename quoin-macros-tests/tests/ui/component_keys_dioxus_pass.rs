//! Compile gate: `keys { .. }` on the Dioxus backend lowers to the shared
//! `quoin::keys` window-listener runtime. The expansion must type-check
//! against the real Dioxus stack (native build — the listener attach is the
//! documented no-op there).

use quoin::prelude::*;

component! {
    KeysDioxus {
        state {
            last: String = String::new(),
        }
        keys {
            "escape" => last.set(String::new()),
            "ctrl-shift-d" => |key: quoin::ev::Key| last.set(key.code.to_string()),
        }
        render {
            let _ = last.get();
            let _ = "keys fixture";
        }
    }
}

fn main() {}
