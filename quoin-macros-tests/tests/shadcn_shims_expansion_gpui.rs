//! Canonical shadcn surface shims — real expansion compile test on the GPUI
//! backend (rich-elements Phase 2, `render_gpui::shadcn`).
//!
//! Like the Phase 0 button test this compiles inside the normal test target
//! (no scratch workspace). Every shim element must lower to real GPUI
//! (styled divs over the shared class transpiler, plus the real
//! `gpui_component::spinner::Spinner`), the argument contracts must hold
//! (`variant:` on alert, `fallback:`/`size:` on avatar, `text:` on kbd, and
//! the full accordion family), and the shim chrome must survive the same
//! `class:`-engine the user's own classes use.

#[cfg(feature = "gpui")]
mod expansion {
    use quoin::component;
    use quoin::quoin_render;

    component! {
        pub ShadcnShimsTest {
            render {
                quoin_render! {
                    div(class: "flex flex-col gap-4 p-6") {
                        card() {
                            card_header() {
                                card_title() { "Phase 2 shims" }
                                card_description() { "Styled shims on the GPUI backend" }
                                card_action() {
                                    button(variant: "outline", size: "sm", label: "Edit")
                                }
                            }
                            card_content() {
                                alert(title: "Heads up", description: "You can compose shims freely.")
                                alert(variant: "destructive", title: "Deleted", description: "This action cannot be undone.")
                                alert_title() { "Standalone title" }
                                alert_description() { "Standalone description" }
                                spinner()
                                kbd(text: "Ctrl+K")
                                avatar(fallback: "AL", size: 40.0)
                                avatar_fallback() { "AL" }
                            }
                            card_footer() {
                                badge(variant: "secondary") { "shimmed" }
                            }
                        }
                        accordion() {
                            accordion_item(value: "item-1") {
                                accordion_trigger() { "Is it shimmed?" }
                                accordion_content() { "Yes — generic fallback rows on GPUI." }
                            }
                            accordion_item(value: "item-2") {
                                accordion_trigger() { "Is it typed?" }
                                accordion_content() { "Yes — the family typechecks end to end." }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The expansion must typecheck: instantiate the component's Render impl
    /// by naming it (the `component!` output is only fully checked when the
    /// struct is actually used).
    #[test]
    fn shadcn_shims_expansion_typechecks() {
        let _ = core::any::type_name::<ShadcnShimsTest>();
    }
}
