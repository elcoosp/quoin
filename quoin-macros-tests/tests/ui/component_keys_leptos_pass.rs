//! Compile gate: `keys { .. }` on the Leptos backend lowers to the shared
//! `quoin::keys` window-listener runtime. The expansion must type-check
//! against the real Leptos stack (native build — the listener attach is the
//! documented SSR no-op there).

use quoin::prelude::*;
use leptos::view;

component! {
    KeysLeptos {
        state {
            last: String = String::new(),
        }
        keys {
            "escape" => last.set(String::new()),
            "ctrl-shift-d" => |key: quoin::ev::Key| last.set(key.code.to_string()),
        }
        render {
            let _ = last.get();
            view! { <div></div> }
        }
    }
}

fn main() {}
