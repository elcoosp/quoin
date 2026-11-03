# Benchmarks

Micro-benchmarks for the quoin primitives. There is deliberately **no
benchmark framework**: both harnesses are plain `std::time::Instant` loops in
release mode, so they run on any machine with zero extra setup.

```bash
just bench-core                                        # quoin-core primitives
just bench-devtools                                    # quoin-devtools event log
```

Numbers are **machine-relative regression baselines**: compare across commits
on the same machine, not across hardware. The reference machine for the table
below was a shared CI-class Linux VM (rustc 1.99, release profile);
absolute values will differ on your hardware — the *ratios* are the signal.

## quoin-core (`quoin-core/examples/bench.rs`)

| Benchmark | Ops | ns/op | Notes |
| --- | ---: | ---: | --- |
| `signal.set` | 1,000,000 | 15 | `StaticSignal` (Arc+RwLock) write |
| `signal.set` (batched ×10) | 100,000 | 146 | 10 writes + 1 update request |
| `signal.get` (clone u32) | 1,000,000 | 14 | clone-based read |
| `signal.with` (no clone) | 1,000,000 | 21 | borrow-based read |
| `signal.set_eq` (equal: skipped) | 1,000,000 | 19 | single write-lock compare, no write |
| `signal.set_eq` (changed: written) | 1,000,000 | 18 | one write lock (was read + write) |
| `derived.get` (x2) | 1,000,000 | 14 | recompute per read |
| `derived.get` (vec len) | 1,000,000 | 19 | recompute per read |
| `memo.get` (cached, 64-elem sum) | 1,000,000 | 37 | source compare, no recompute |
| `memo.get` (uncached after change) | 1,000,000 | 65 | clone + compare + recompute |
| `token.child_token` + drop | 100,000 | 48 | registration + prune |
| `token.cancel` (no waiters) | 1,000,000 | 16 | atomic swap |
| `token.guard` (cancel on drop) | 1,000,000 | 20 | RAII cancel |
| `spawn_local_task` + run | 10,000 | 290 | queue + drain + block_on |
| `pool.spawn_pooled` (trivial fut) | 10,000 | 550 | shared worker pool |

**`set_eq` takeaway:** `Signal::set_if_eq` compares under a single write-lock
acquisition. The *changed* path (a value that differs, the common case when
UI state actually moves) previously paid a read lock followed by a write lock
(~34 ns here); it now pays one write lock (~18 ns) — roughly half. The
*skipped* path is unchanged (it was already one lock).

**Worker-pool takeaway:** the `worker_loop` restructure (queue guard held
only inside the `next_job` decision function, never across job execution)
also trimmed `spawn_pooled` from ~780 ns to ~550 ns per trivial task on this
machine.

**Memo takeaway:** a cached `memo` read costs ~37 ns where the plain
`derived` read of the same 64-element sum would cost the full recompute on
every read (here ~65 ns including the recompute; the gap widens with
genuinely expensive maps — sorting, formatting, filtering — where `memo`
skips the work entirely and pays only the source clone + `PartialEq`).

## quoin-devtools (`quoin-devtools/examples/bench-devtools.rs`)

Aggregate views read **incremental counters** maintained on push and on ring
eviction, so they no longer scan the retained window. Before → after for the
two affected views (same machine, same method):

| Benchmark | Before (scan) | After (incremental) | Speed-up |
| --- | ---: | ---: | ---: |
| `summary()` over 4096 events | 29,737 ns/op | **88 ns/op** | ~340× |
| `hot_signals()` over 4096 events | 8,179 ns/op | **19 ns/op** | ~430× |

Full current table:

| Benchmark | Ops | ns/op | Notes |
| --- | ---: | ---: | --- |
| `record` (enabled, small ring) | 1,000,000 | 71 | +5 ns for the counter update |
| `record` (disabled) | 1,000,000 | 2 | one relaxed atomic load |
| raw `set` (no wrapper) | 1,000,000 | 18 | baseline without `Instrumented` |
| `Instrumented` set (recording off) | 1,000,000 | 20 | +2 ns wrapper overhead |
| `Instrumented` set (recording on) | 1,000,000 | 94 | +76 ns per recorded mutation |
| `summary()` over 4096 events | 10,000 | 88 | small list clone, no scan |
| `hot_signals()` over 4096 events | 10,000 | 19 | small list clone + sort |
| `len()` | 1,000,000 | 12 | single lock, no iteration |
| `with_events(count)` (zero-copy view) | 100,000 | 14 | no event cloning |

**Zero-cost gate takeaway:** with recording disabled, the `Instrumented`
wrapper adds ~2 ns over a raw mutation (one relaxed atomic load, no
allocation), and a disabled `record` call is ~2 ns — safe to ship wired up.

**Incremental-aggregate trade-off:** keeping the summary counters costs ~5 ns
per recorded event (one linear scan over the few distinct `(scope, kind)`
keys — sessions have a handful), and buys O(distinct-keys) `summary()` /
`hot_signals()` instead of an O(retained-events) scan. Counters are
decremented when the ring evicts, so the views always agree with a fresh
scan of the retained window (pinned by regression tests).

**Aggregate-view takeaway:** `with_events` still gives zero-copy access to
the raw retained slice for custom views.

## Running on your machine

```bash
just bench-core
just bench-devtools
```

Record before/after numbers in your PR when a change is meant to move these
numbers — the PR template reminds you of it.
