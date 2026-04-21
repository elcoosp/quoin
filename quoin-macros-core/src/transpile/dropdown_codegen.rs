#[allow(unused)]
use proc_macro2::TokenStream;
#[allow(unused)]
use quote::quote;
pub struct MenuItemDef {
    pub label: syn::Expr,
    pub on_click: syn::Expr,
}

#[cfg(feature = "gpui")]
pub fn generate_gpui_dropdown(trigger_expr: &syn::Expr, menu_items: &[MenuItemDef]) -> TokenStream {
    let items = menu_items.iter().enumerate().map(|(i, item)| {
        let label = &item.label;
        let on_click = &item.on_click;
        quote! {
            .item(
                gpui_component::popup_menu::PopupMenuItem::new(concat!("menu-", #i))
                    .label(#label)
                    .on_click(move |_cx| { #on_click(); })
            )
        }
    });

    quote! {{
        let trigger = #trigger_expr;
        gpui_component::popup_menu::PopupMenu::new(trigger)
            .menu(move |menu, _cx| {
                menu
                #(#items)*
            })
    }}
}

#[cfg(feature = "leptos")]
pub fn generate_leptos_dropdown(
    trigger_expr: &syn::Expr,
    menu_items: &[MenuItemDef],
) -> TokenStream {
    let items = menu_items.iter().map(|item| {
        let label = &item.label;
        let on_click = &item.on_click;
        quote! {
            <leptos_shadcn_ui::dropdown_menu::DropdownMenuItem on:click=move |_| #on_click()>
                #label
            </leptos_shadcn_ui::dropdown_menu::DropdownMenuItem>
        }
    });
    quote! {
        <leptos_shadcn_ui::dropdown_menu::DropdownMenu>
            <leptos_shadcn_ui::dropdown_menu::DropdownMenuTrigger>
                {#trigger_expr}
            </leptos_shadcn_ui::dropdown_menu::DropdownMenuTrigger>
            <leptos_shadcn_ui::dropdown_menu::DropdownMenuContent>
                #(#items)*
            </leptos_shadcn_ui::dropdown_menu::DropdownMenuContent>
        </leptos_shadcn_ui::dropdown_menu::DropdownMenu>
    }
}

#[cfg(feature = "dioxus")]
pub fn generate_dioxus_dropdown(
    trigger_expr: &syn::Expr,
    menu_items: &[MenuItemDef],
) -> TokenStream {
    let items = menu_items.iter().map(|item| {
        let label = &item.label;
        let on_click = &item.on_click;
        quote! {
            shadcn_dioxus::dropdown_menu::DropdownMenuItem {
                on_click: move |_| #on_click(),
                #label
            }
        }
    });
    quote! {
        shadcn_dioxus::dropdown_menu::DropdownMenu {
            shadcn_dioxus::dropdown_menu::DropdownMenuTrigger {
                #trigger_expr
            }
            shadcn_dioxus::dropdown_menu::DropdownMenuContent {
                #(#items)*
            }
        }
    }
}
