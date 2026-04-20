use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_gpui_dropdown(
    trigger_expr: &syn::Expr,
    menu_items: &[MenuItemDef],
) -> TokenStream {
    let item_tokens: Vec<TokenStream> = menu_items.iter().enumerate().map(|(i, item)| {
        let label = &item.label;
        let on_click = &item.on_click;
        quote! {
            .item(
                gpui_component::popup_menu::PopupMenuItem::new(concat!("quoin-menu-", #i))
                    .label(#label)
                    .on_click(cx.listener(|_, _, _| #on_click()))
            )
        }
    }).collect();
    quote! {
        {
            let __trigger = #trigger_expr;
            __trigger.dropdown_menu(move |menu, _window, cx| {
                menu
                #(#item_tokens)*
            })
        }
    }
}

pub fn generate_leptos_dropdown(
    trigger_expr: &syn::Expr,
    menu_items: &[MenuItemDef],
) -> TokenStream {
    let item_tokens: Vec<TokenStream> = menu_items.iter().map(|item| {
        let label = &item.label;
        let on_click = &item.on_click;
        quote! {
            <leptos_shadcn_ui::dropdown_menu::DropdownMenuItem on:click=move |_| #on_click()>
                #label
            </leptos_shadcn_ui::dropdown_menu::DropdownMenuItem>
        }
    }).collect();
    quote! {
        <leptos_shadcn_ui::dropdown_menu::DropdownMenu>
            <leptos_shadcn_ui::dropdown_menu::DropdownMenuTrigger>
                {#trigger_expr}
            </leptos_shadcn_ui::dropdown_menu::DropdownMenuTrigger>
            <leptos_shadcn_ui::dropdown_menu::DropdownMenuContent>
                #(#item_tokens)*
            </leptos_shadcn_ui::dropdown_menu::DropdownMenuContent>
        </leptos_shadcn_ui::dropdown_menu::DropdownMenu>
    }
}

pub fn generate_dioxus_dropdown(
    trigger_expr: &syn::Expr,
    menu_items: &[MenuItemDef],
) -> TokenStream {
    let item_tokens: Vec<TokenStream> = menu_items.iter().map(|item| {
        let label = &item.label;
        let on_click = &item.on_click;
        quote! {
            shadcn_dioxus::dropdown_menu::DropdownMenuItem {
                on_click: move |_| #on_click(),
                #label
            }
        }
    }).collect();
    quote! {
        shadcn_dioxus::dropdown_menu::DropdownMenu {
            shadcn_dioxus::dropdown_menu::DropdownMenuTrigger {
                #trigger_expr
            }
            shadcn_dioxus::dropdown_menu::DropdownMenuContent {
                #(#item_tokens)*
            }
        }
    }
}

pub struct MenuItemDef {
    pub label: syn::Expr,
    pub on_click: syn::Expr,
}
