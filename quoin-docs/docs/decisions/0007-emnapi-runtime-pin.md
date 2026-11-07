# ADR 0007 — Pin `@emnapi/runtime` to 1.11.3 for `astro check` on Node 24

- **Status:** Accepted
- **Date:** 2026-11-07
- **Deciders:** Documentation Architect (autonomous run)

## Context

`pnpm check` (`astro check`) failed on this environment before checking anything: importing `@astrojs/check` crashed with `this.bridge.setLastError is not a function` (and, at process exit, `this.bridge.deleteEnv is not a function`).

Root cause, traced through the dependency graph:

- `@astrojs/check` → `@astrojs/language-server` → `@astrojs/astro2tsx` → `@napi-rs/wasm-runtime` → `@emnapi/core` + `@emnapi/runtime`.
- `@napi-rs/wasm-runtime@1.2.4` pairs **`@emnapi/core@1.11.3`** (whose `instantiateNapiModuleSync` calls the v1 `createEnv(filename, version, fn, fn, abort, nodeBinding)` signature) with a peer on **`@emnapi/runtime@2.0.0-beta.2`** (whose `createEnv(filename, version, bridge, nodeBinding)` expects arg 3 to be the bridge object).
- With the v2 beta runtime installed, the function passed in slot 3 is spread as the bridge, producing an object without `setLastError`/`deleteEnv` — hence the crash on every `astro check` run.

## Decision

Pin the runtime to the version matching the core, via `pnpm-workspace.yaml`:

```yaml
overrides:
  "@emnapi/runtime": 1.11.3
```

Notes recorded for future maintainers:

- pnpm 12 no longer reads `pnpm.overrides` from `package.json`; the setting lives in `pnpm-workspace.yaml`.
- `rolldown` pins `@emnapi/runtime@2.0.0-beta.2` exactly, so the store retains both versions; the build path uses rolldown's native binary and is unaffected.
- After changing overrides, delete `pnpm-lock.yaml` (or force re-resolution) — otherwise the previous resolution is reused.

## Consequences

- `@astrojs/check` imports cleanly and `pnpm check` passes (0 errors, 0 warnings, 0 hints) with TypeScript 6.0.3 on Node 24.22.2.
- Removing the override restores the crash; if a future `@napi-rs/wasm-runtime` ships a matched core/runtime pair, this pin can be dropped.
