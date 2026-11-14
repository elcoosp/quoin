# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-14

### Bug Fixes
- update rust crate gloo-timers to 0.4 by @renovate[bot] in [#33](https://github.com/elcoosp/quoin/pull/33) ([b403557](https://github.com/elcoosp/quoin/commit/b4035575de3b69a31673ae589c2019b645fe7d4c))
- keep the effect runtime consistent across panics by @elcoosp ([30f16e7](https://github.com/elcoosp/quoin/commit/30f16e72d503cec72844ec8cb2dfc97540d3add6))
- bound cancellation waker registry to prevent memory leaks on drop by @elcoosp ([9b4da05](https://github.com/elcoosp/quoin/commit/9b4da054ac9f88d9596e9bf7c345980b12fc1c10))
- leak VirtualDom in Dioxus conformance test to avoid TLS destruction by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([a0d5f00](https://github.com/elcoosp/quoin/commit/a0d5f00ef52ff565a36865855306c979fc14b63d))
- all tests pass by @elcoosp ([3305e44](https://github.com/elcoosp/quoin/commit/3305e446d8e9b8adeadf614384f80e697a069af6))
- working tests by @elcoosp ([877daf4](https://github.com/elcoosp/quoin/commit/877daf4efb596b8269105d17c5292fd638dd1ab8))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([65a7312](https://github.com/elcoosp/quoin/commit/65a731230cf3db2b64de685c2dc0ad1d5ec3d39e))
- enforce SKILL.md rules (jsDelivr logos, absolute links, remove license sections) by @elcoosp ([37ce0b8](https://github.com/elcoosp/quoin/commit/37ce0b814289a4fa78d487ff9bf2774bb3528391))
- correct the spawn_pooled panic claim by @elcoosp ([46eab0d](https://github.com/elcoosp/quoin/commit/46eab0d351b07d458456be3f7f459d5271f94770))
- correct the create_signal ownership claim by @elcoosp ([2116b24](https://github.com/elcoosp/quoin/commit/2116b249b04abc679d562d7c65a70a18da229f20))
- document the effect runtime and AnySignal by @elcoosp ([dfa99f4](https://github.com/elcoosp/quoin/commit/dfa99f4c94bc4844305e1de34beb667087e4a871))
- clarify GPUI global state sharing semantics by @elcoosp ([4e54c28](https://github.com/elcoosp/quoin/commit/4e54c28b4907c0f7b5610a75d6afb7c26d279780))
- fix doc links in Signal trait by @elcoosp ([d6c0c8f](https://github.com/elcoosp/quoin/commit/d6c0c8fe947827b30276c2249c598433a94d389c))
- clarify global state scoping in ReactiveContext by @elcoosp ([0b6759c](https://github.com/elcoosp/quoin/commit/0b6759ce35038bf1120b3bdf395f8bde6579943e))
- rewrite README to focus on core traits and extensions by @elcoosp ([4ce1cb6](https://github.com/elcoosp/quoin/commit/4ce1cb6472da0eda84dc345667b873ad44e050eb))
- more docs by @elcoosp ([eae3b3f](https://github.com/elcoosp/quoin/commit/eae3b3f8cb1840274b54294fa79d6eb976dd1872))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))

### Features
- keys { .. } goes native on the web backends ([#35](https://github.com/elcoosp/quoin/pull/35)) by @elcoosp in [#35](https://github.com/elcoosp/quoin/pull/35) ([62a0719](https://github.com/elcoosp/quoin/commit/62a07192d10c221951def4ee2b2effc937cc15ad))
- bridge fine-grained signals into the effect graph ([#29](https://github.com/elcoosp/quoin/pull/29)) by @elcoosp in [#29](https://github.com/elcoosp/quoin/pull/29) ([52d6ec7](https://github.com/elcoosp/quoin/commit/52d6ec7bee3e883bd149d1e1662e85c2a318a998))
- export the effect and task API from the prelude by @elcoosp ([adc6f16](https://github.com/elcoosp/quoin/commit/adc6f165c400da85a44de77d96c0906464e65a3e))
- implement TrackedSignal for AnySignal by @elcoosp ([d3ce4c1](https://github.com/elcoosp/quoin/commit/d3ce4c1dd8455062c561624532cb0cf88bb99ba9))
- add effect and erasure APIs to the prelude by @elcoosp ([f7e39e9](https://github.com/elcoosp/quoin/commit/f7e39e95b81cfb8b212f9a1753b5e44e74d2c2ae))
- export the effect and any_signal modules by @elcoosp ([614d7a3](https://github.com/elcoosp/quoin/commit/614d7a3ce7e656d270b33048a198ef6bfe8bfd0c))
- track StaticSignal reads and mutations in the effect graph by @elcoosp ([a973000](https://github.com/elcoosp/quoin/commit/a973000b2611ff6a5dd040cf2bccb5388d62ac03))
- notify the effect graph from NotifierSignal mutations by @elcoosp ([92f8799](https://github.com/elcoosp/quoin/commit/92f8799a0b2ac04c1aa19e0ff50c43035da128ee))
- flush deferred effect re-runs at batch end by @elcoosp ([587386a](https://github.com/elcoosp/quoin/commit/587386ad6beffaec2d75bb29c2bbfadc917414c9))
- add AnySignal<T>, the ADR-002 type-erased wrapper by @elcoosp ([714052f](https://github.com/elcoosp/quoin/commit/714052f7e662af83a3c9c87b134bc19ccb585574))
- add the dependency-tracked effect runtime by @elcoosp ([d33b244](https://github.com/elcoosp/quoin/commit/d33b244b44b49bb8dfb19e463113f91990be6a05))
- typed events + Send bound on signals by @elcoosp ([91bb124](https://github.com/elcoosp/quoin/commit/91bb1241b9afdcad47a14469cfe9423915e509f2))
- extend sleep future to enforce send and sync constraints by @elcoosp ([1c83b3f](https://github.com/elcoosp/quoin/commit/1c83b3f6c2c977461647eb762a9491579f18941d))
- implement portable timer module for Executor::sleep by @elcoosp ([28f9e28](https://github.com/elcoosp/quoin/commit/28f9e287b3cb7bd19313f8230f578b02f9a95c48))
- export SleepFuture and sleep in prelude by @elcoosp ([8840027](https://github.com/elcoosp/quoin/commit/8840027d0181d4712258069794c90960106c335a))
- expose timer module and SleepFuture by @elcoosp ([3e64864](https://github.com/elcoosp/quoin/commit/3e64864edf4e48ff735afa94674ef4521495df67))
- add sleep method to Executor trait by @elcoosp ([3cae9c6](https://github.com/elcoosp/quoin/commit/3cae9c6b1f89cda8f4159a4510c85d299b13cfa4))
- support globals in StaticContext by @elcoosp ([1433683](https://github.com/elcoosp/quoin/commit/1433683b09e2441191c18c21cf55e9bc25a652fb))
- add set_if_eq to Signal by @elcoosp ([75211b2](https://github.com/elcoosp/quoin/commit/75211b2f1ddc580cbe5f76519b45d91e68d8db14))
- export memo from prelude by @elcoosp ([bbedaa7](https://github.com/elcoosp/quoin/commit/bbedaa7222b279ebc49cb203037c9296a1589940))
- add shared notifier signal by @elcoosp ([7289d6f](https://github.com/elcoosp/quoin/commit/7289d6fb8fbc819c039d0d3635e1190daec0c1e5))
- expose globals, join, notify_signal, and memo by @elcoosp ([2487e8a](https://github.com/elcoosp/quoin/commit/2487e8a7a299b85661d87b97603c2579b96f0a24))
- add shared join-handle lifecycle by @elcoosp ([c94bf12](https://github.com/elcoosp/quoin/commit/c94bf12249d4088d90a7e0debfc7d52bf5ae3f65))
- add shared thread-local globals store by @elcoosp ([532457d](https://github.com/elcoosp/quoin/commit/532457d1b92409b52048d7b1e76d6a1181ad7e2d))
- add memoized derived signals by @elcoosp ([69e05e7](https://github.com/elcoosp/quoin/commit/69e05e75de5ab9ad2a0c233b041d7425e12d2a89))
- add Resource primitive for async data loading by @elcoosp ([c085e6d](https://github.com/elcoosp/quoin/commit/c085e6dbadd1a6aefd34ac3ca654e496117b146c))
- expand prelude with new extensions and types by @elcoosp ([5ad6679](https://github.com/elcoosp/quoin/commit/5ad66793b06b6dfdc55c00bf9a2958da44ab64bb))
- add LocalExecutor for !Send futures by @elcoosp ([a28f342](https://github.com/elcoosp/quoin/commit/a28f34248ecd12cfb6efd21953afb9df432624a6))
- expose new modules and re-exports for Phase 1.5 by @elcoosp ([47bfd1f](https://github.com/elcoosp/quoin/commit/47bfd1fd51ffdf094a15b7822b64e1b312cddb75))
- add blanket extension traits for signals and contexts by @elcoosp ([c13018c](https://github.com/elcoosp/quoin/commit/c13018cd92c69dcfdb2f3b71a90d607d09b748ff))
- add derived signals and zip combinator by @elcoosp ([1b4cde4](https://github.com/elcoosp/quoin/commit/1b4cde4f6e97669b3749873270ac7ffe0acf8b95))
- upgrade CancellationToken with child tokens and guards by @elcoosp ([e880972](https://github.com/elcoosp/quoin/commit/e880972d83b870b48533410ef3fcb9d9a25611f8))
- implement batched updates to collapse UI notifications by @elcoosp ([faa19ca](https://github.com/elcoosp/quoin/commit/faa19cafd844f30e80ad2e6fe6f6678faf83c98b))
- export JoinHandle in prelude by @elcoosp ([1d4fe22](https://github.com/elcoosp/quoin/commit/1d4fe22576b8f51877b4ffd52e15efa2f8fcadbc))
- add JoinHandle::detach and document structured concurrency lifecycle by @elcoosp ([46a37fa](https://github.com/elcoosp/quoin/commit/46a37fa56610a7258b01ba8578cd5ce77f822926))

### Miscellaneous
- inject precisely curated keywords for crates.io by @elcoosp ([8722cab](https://github.com/elcoosp/quoin/commit/8722cab4895292fb33f6f96a707b25a9abb9c7c2))
- use workspace metadata by @elcoosp ([9cd331a](https://github.com/elcoosp/quoin/commit/9cd331a90caa39a0d6417dda0f9fc737177ee341))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))

### Performance
- add effect-disposal scaling benchmarks by @elcoosp ([fa9cfe1](https://github.com/elcoosp/quoin/commit/fa9cfe12ab105d78cc3ae6c12949a4b42efc024c))
- reduce pool lock contention by @elcoosp ([566691a](https://github.com/elcoosp/quoin/commit/566691a11b20da5d63c9dc3e18a68063738d0608))
- benchmark set_eq changed path and memo by @elcoosp ([c2adf03](https://github.com/elcoosp/quoin/commit/c2adf030e1fde0b24cecd041fd56caa00c3d9a3d))
- add shared growable worker pool for native tasks by @elcoosp ([e574776](https://github.com/elcoosp/quoin/commit/e5747767313c0e6abfb30a45c7970253a63ed06c))
- add micro-benchmark suite for core primitives by @elcoosp ([2912d70](https://github.com/elcoosp/quoin/commit/2912d7032ee0ba1fa1912f040a17430afee619b1))

### Refactor
- introduce SyncSleepFuture and unbox allocations by @elcoosp ([311befd](https://github.com/elcoosp/quoin/commit/311befd4bf1fd427069c466836ccca954faffe2c))
- wrap native sleep futures inside type-erased pin boxes by @elcoosp ([d094b02](https://github.com/elcoosp/quoin/commit/d094b029a6f9a177fd9e1e9f93cb6d9fa2ff9eeb))
- make resource helpers const and avoid lock reentry by @elcoosp ([00d0485](https://github.com/elcoosp/quoin/commit/00d0485dba601d889a5e5efa09e2fc479f6cdd32))
- remove unused local import shim by @elcoosp ([8e0a350](https://github.com/elcoosp/quoin/commit/8e0a3506633d8872cae9a09e4ee5c4e93211023a))
- delegate set_eq and simplify global init by @elcoosp ([60340eb](https://github.com/elcoosp/quoin/commit/60340eb918fb250d28c4440007323454bd45c5fe))
- tighten cancellation waker and const APIs by @elcoosp ([8857366](https://github.com/elcoosp/quoin/commit/8857366ccf3550abfc44635d53c3ea1e05efc30e))

### Testing
- cover shared adapter primitives by @elcoosp ([d4fb758](https://github.com/elcoosp/quoin/commit/d4fb7582cf632e5f7e3900c880e3e816a34073b3))
- add minimal reference implementations for doctests by @elcoosp ([6a88ce1](https://github.com/elcoosp/quoin/commit/6a88ce1b148de3176ad587646d3f94ace8a3039a))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))

### bench
- gate effect and AnySignal costs with micro-benchmarks by @elcoosp ([95e1007](https://github.com/elcoosp/quoin/commit/95e100751d5a62f5f3a2cda1027b5561a4029002))

### build
- add futures-timer and gloo-timers dependencies by @elcoosp ([1382902](https://github.com/elcoosp/quoin/commit/1382902e00e7b8ded230d1aa587a3ee3769b847d))
- inherit workspace lints by @elcoosp ([84ffa2e](https://github.com/elcoosp/quoin/commit/84ffa2e7e113b2d772d1cdc2813d4b3f55f25779))
- add dependencies for futures, cancellation, and wasm by @elcoosp ([50c17f9](https://github.com/elcoosp/quoin/commit/50c17f99f86c4d365f294d9cf861aab5decfe9b2))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
