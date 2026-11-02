#!/bin/bash

# Script de commit automatique et conventionnel - Un commit par fichier
# Compatible Bash 3.2 (macOS) - Version nettoyée sans cassure Markdown

files=$(git status --porcelain | awk '{print $2}')

if [ -z "$files" ]; then
    echo "Aucun fichier modifie detecte."
    exit 0
fi

for file in $files; do
    if [ ! -f "$file" ]; then
        continue
    fi

    echo "Analyse et commit pour : $file"

    case "$file" in
        "Cargo.lock")
            subject="chore: synchronize lockfile dependencies for gpui framework upgrade"
            body="Update locked package structures mapping to adopt gpui-pre and zvariant ecosystem alignments."
            ;;
        "examples/counter-dioxus/Cargo.lock")
            subject="chore(examples): pin stable dioxus dependencies versions"
            body="Re-align examples lock maps targeting dioxus version 0.7.6 manifestations."
            ;;
        "examples/counter-leptos/Cargo.lock")
            subject="chore(examples): synchronize lockfile for leptos counter example"
            body="Align package mappings with tower-http version 0.6 boundaries."
            ;;
        "examples/counter-leptos/Cargo.toml")
            subject="chore(examples): downgrade tower-http to version 0.6"
            body="Align manifest package requirements with local ecosystem specifications."
            ;;
        "examples/ucp-lib/Cargo.lock")
            subject="chore(examples): update locked dependencies inside ucp library example"
            body="Sync target parameters with standard ucp-core and gpui-pre version requirements."
            ;;
        "quoin-core/src/executor.rs")
            subject="refactor(core): wrap native sleep futures inside type-erased pin boxes"
            body="Enforce uniform execution signatures matching async runtime storage boundaries."
            ;;
        "quoin-core/src/timer.rs")
            subject="feat(core): extend sleep future to enforce send and sync constraints"
            body="Let composed timers satisfy multi-threaded reactive queries seams out of the box."
            ;;
        "quoin-dioxus/tests/conformance.rs" | "quoin-floem/tests/conformance.rs" | "quoin-leptos/tests/conformance.rs")
            subject="test: prune unused duration imports from conformance suites"
            body="Clean up test harness files to eliminate compiler dead code warnings."
            ;;
        "quoin-gpui/Cargo.toml")
            subject="chore(gpui): pin gpui-pre and platform version constraints"
            body="Re-align workspace dependencies with the stable 0.3.10 component baseline."
            ;;
        "quoin-gpui/src/lib.rs")
            subject="feat(gpui): implement whole-window refresh notifiers"
            body="Provide macro-friendly window triggers to repaint every view entity concurrently on state change."
            ;;
        "quoin-macros-core/src/emit/gpui.rs")
            subject="feat(macros): implement keyboard shortcuts code-gen inside gpui backends"
            body="Extend component expansions to attach focus handles and emit global key context context nodes."
            ;;
        "quoin-macros-core/src/emit/render_gpui/button.rs")
            subject="refactor(macros): convert button elements code-gen to use real widgets"
            body="Map agnostic labels icons sizes and tooltips arguments into gpui-component parameters."
            ;;
        "quoin-macros-core/src/emit/render_gpui/clipboard_button.rs")
            subject="refactor(macros): convert clipboard buttons to use native widgets"
            body="Route agnostic value and tooltip tokens straight into the clipboard box builder paths."
            ;;
        "quoin-macros-core/src/emit/render_gpui/data_table.rs")
            subject="feat(macros): wire closure table delegates inside data tables code-gen"
            body="Persist table states across frames through the component table manager slots."
            ;;
        "quoin-macros-core/src/emit/render_gpui/dropdown_menu.rs")
            subject="refactor(macros): map dropdown menus to use popup menu components"
            body="Generate popup menu items and bind click closures with implicit move blocks."
            ;;
        "quoin-macros-core/src/emit/render_gpui/generic.rs")
            subject="feat(macros): add icon and styled text elements code-gen support"
            body="Provide native element materialization routing text runs to theme color resolvers."
            ;;
        "quoin-macros-core/src/emit/render_gpui/input.rs")
            subject="refactor(macros): wire two-way input state bindings inside inputs code-gen"
            body="Route agnostic cleanable icon prefix and small sizing arguments into inputs widgets."
            ;;
        "quoin-macros-core/src/emit/render_gpui/tabs.rs")
            subject="refactor(macros): convert tab elements code-gen to use real tab bar widgets"
            body="Map agnostic tab specs and selection indices into native children components loops."
            ;;
        "quoin-macros-core/src/emit/render_gpui/virtual_list.rs")
            subject="refactor(macros): translate virtual list nodes to use native scroll handles"
            body="Wire uniform item sizing and link item renderers to the virtual scroll vector managers."
            ;;
        "quoin-macros-core/src/parse.rs")
            subject="feat(macros): implement key shortcut sections parsing inside component AST"
            body="Add key entries parser loops extracting keystroke mappings and fat arrow expressions."
            ;;
        "quoin-macros-core/src/transpile/icon_codegen.rs")
            subject="feat(macros): implement kebab-case to pascal-case icon identifier converters"
            body="Provide compile-time string transformers fallback-mapping unknown icons to the info token."
            ;;
        "quoin-macros-core/src/transpile/tailwind.rs")
            subject="feat(macros): extend arbitrary values and hex colors transpiler rules"
            body="Provide regex filters resolving pixel metrics shadows and custom background color codes."
            ;;
        "quoin-macros-tests/Cargo.toml" | "quoin-ui-gpui/Cargo.toml")
            subject="chore: re-align gpui-component version constraints"
            body="Pin asset manifest requirements to the stable 0.8.1 registry publication."
            ;;
        "quoin-ui-gpui/src/lib.rs")
            subject="feat(ui-gpui): implement live theme resolvers and text run builders"
            body="Provide table manager and scroll manager abstractions to persist native widget state."
            ;;
        "quoin-ui-gpui/src/node_renderer.rs")
            subject="feat(ui-gpui): implement runtime tree renderer for agnostic UI nodes"
            body="Provide complete node walk materializing native elements dropdown menus and scroll-restoration hooks."
            ;;
        "quoin-ui/src/lib.rs")
            subject="feat(ui): declare text colors and styled runs types in the agnostic model"
            body="Expose shell and viewport abstractions to let framework-agnostic panels open URLs."
            ;;
        "quoin-ui/src/node.rs")
            subject="feat(ui): design runtime Node tree vocabularies for agnostic libraries"
            body="Model structured builders typing button variants input nodes data tables and tab selectors."
            ;;
        "quoin/Cargo.toml")
            subject="chore: align optional gpui package dependencies keys"
            body="Pin framework hooks matching the 0.3.10 workspace baseline configuration."
            ;;
        *)
            subject="chore: update $(basename "$file")"
            body="Refactor or clean up internal logic structures within $file."
            ;;
    esac

    # Indexation temporaire, commit conventionnel et nettoyage de la file
    git add "$file"
    git commit -m "$subject" -m "$body"

    echo "✅ Commite : $file"
    echo "--------------------------------------------"
done

echo "Félicitations, tous les fichiers de ce dépôt ont été enregistrés de manière atomique !"
