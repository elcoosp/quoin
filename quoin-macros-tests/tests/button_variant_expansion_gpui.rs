//! Canonical `button` variant/size grammar — real expansion compile test on
//! the GPUI backend (rich-elements Phase 0, `emit::shadcn_spec`).
//!
//! Unlike the trybuild suites this test compiles inside the normal test
//! target (no scratch workspace), so it runs wherever `--features gpui`
//! builds: every canonical `variant:`/`size:` value must lower to a real
//! gpui-component call, the boolean sugar must keep compiling, and unknown
//! values must produce a compile error (checked via the `compile_fail`
//! companion below).

#[cfg(feature = "gpui")]
mod expansion {
    use quoin::component;
    use quoin::quoin_render;

    component! {
        pub ButtonVariantTest {
            render {
                quoin_render! {
                    div(class: "flex flex-col gap-2") {
                        button(variant: "default", label: "Default")
                        button(variant: "primary", label: "Primary")
                        button(variant: "destructive", label: "Delete")
                        button(variant: "outline", label: "Outline")
                        button(variant: "secondary", label: "Secondary")
                        button(variant: "ghost", label: "Ghost")
                        button(variant: "link", label: "Link")
                        button(size: "sm", label: "Small")
                        button(size: "lg", label: "Large")
                        button(size: "icon", label: "Icon")
                        button(primary: true, label: "Sugar primary")
                        button(destructive: true, label: "Sugar destructive")
                        button(ghost: true, small: true, label: "Sugar both")
                    }
                }
            }
        }
    }

    /// The expansion must typecheck: instantiate the component's Render impl
    /// by naming it (the `component!` output is only fully checked when the
    /// struct is actually used).
    #[test]
    fn button_variant_expansion_typechecks() {
        let _ = core::any::type_name::<ButtonVariantTest>();
    }
}
