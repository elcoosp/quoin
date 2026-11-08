//! Compare quoin-style and gpui-rsx-core class engines on identical inputs.

use gpui_rsx_core::{ClassMode, ClassOutcome, SpacingScale, analyze_class, known_classes};
use quoin_style::{KNOWN_CLASSES, parse_tailwind};

fn gpui_rsx_resolve(class: &str, tailwind_scale: bool) -> String {
    let mode = if tailwind_scale {
        ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind)
    } else {
        ClassMode::PERMISSIVE
    };
    let analysis = analyze_class(class, mode);
    let variant = analysis
        .variant
        .as_ref()
        .map(|v| format!("[{}] ", v.method))
        .unwrap_or_default();
    match &analysis.outcome {
        ClassOutcome::Supported(ts) => format!("SUPPORTED {variant}{ts}"),
        ClassOutcome::PassThrough { tokens, .. } => format!("PASSTHRU  {variant}{tokens}"),
        ClassOutcome::Ignored { message } => format!("IGNORED   {message}"),
        ClassOutcome::Error { message } => format!("ERROR     {message}"),
        _ => "OTHER".to_string(),
    }
}

fn quoin_resolve(class: &str) -> String {
    let parsed = parse_tailwind(class);
    if !parsed.unknown.is_empty() {
        return format!("UNKNOWN   {:?}", parsed.unknown);
    }
    let n_base = parsed.base.len();
    let n_hover = parsed.hover.len();
    format!(
        "OK        decls(base={n_base}, hover={n_hover}): {:?} {:?}",
        parsed.base, parsed.hover
    )
}

fn main() {
    // 1. Vocabulary sizes
    let grx_known: Vec<String> = known_classes().to_vec();
    println!("=== VOCABULARY ===");
    println!("quoin KNOWN_CLASSES:    {}", KNOWN_CLASSES.len());
    println!("gpui-rsx known_classes: {}", grx_known.len());
    println!(
        "gpui-rsx SUPPORTED_VARIANTS: {:?}",
        gpui_rsx_core::SUPPORTED_VARIANTS
    );

    // 2. Same class strings through both engines
    let cases = [
        "flex",
        "flex-col",
        "gap-4",
        "p-4",
        "px-2",
        "m-1",
        "bg-gray-900",
        "bg-blue-600",
        "text-white",
        "text-blue-400",
        "text-2xl",
        "text-xs",
        "font-bold",
        "font-semibold",
        "rounded-md",
        "rounded",
        "border",
        "border-b",
        "border-t-2",
        "border-red-500",
        "w-full",
        "h-full",
        "w-[280px]",
        "bg-[#ff0000]",
        "opacity-50",
        "shadow-sm",
        "cursor-pointer",
        "truncate",
        "overflow-y-scroll",
        "justify-between",
        "items-center",
        "hover:text-blue-400",
        "hover:bg-gray-800",
        "focus:border-blue-500",
        "active:bg-blue-700",
        "group-hover:bg-red-500",
        // classes only one engine may know
        "bg-surface", // quoin theme token
        "bg-primary", // gpui-rsx theme token (rsx.toml)
        "text-red",   // malformed color probe
        "whitespace-nowrap",
        "select-none",
        "inset-0",
        "top-2",
        "min-w-0",
        "max-w-full",
        "size-full",
        "self-center",
        "flex-1",
    ];

    println!("\n=== PER-CLASS COMPARISON ===");
    for class in cases {
        println!("\n--- `{class}` ---");
        println!("  quoin:          {}", quoin_resolve(class));
        println!("  grsx(Px):       {}", gpui_rsx_resolve(class, false));
        println!("  grsx(Tailwind): {}", gpui_rsx_resolve(class, true));
    }

    // 3. Full multi-class string through both engines
    println!("\n=== FULL STRING ===");
    let full = "flex flex-col gap-4 p-4 bg-gray-900 text-white hover:text-blue-400";
    let q = parse_tailwind(full);
    println!(
        "quoin: base={} hover={} unknown={:?}",
        q.base.len(),
        q.hover.len(),
        q.unknown
    );
    for class in full.split_whitespace() {
        let _ = gpui_rsx_resolve(class, true);
    }
    println!("gpui-rsx per-class results printed above");

    // 4. Overlap analysis
    println!("\n=== OVERLAP ===");
    let grx: std::collections::HashSet<String> = grx_known.into_iter().collect();
    let only_quoin: Vec<&str> = KNOWN_CLASSES
        .iter()
        .copied()
        .filter(|c| !grx.contains(*c))
        .collect();
    println!(
        "quoin classes NOT in gpui-rsx known list: {}",
        only_quoin.len()
    );
    for chunk in only_quoin.chunks(12) {
        println!("  {chunk:?}");
    }
}
