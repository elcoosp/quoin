# Changelog

All notable changes to Quoin will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [Unreleased]

### ♻️ Changed – Refactoring
- **macros**: Consolidate badge into single-function pattern ([1fee475](https://github.com/elcoosp/quoin/commit/1fee4753ec72ce93e1315f339a736b75de7e79cb))
- **macros**: Consolidate all _plain/_shadcn splits into single-function pattern ([f33a474](https://github.com/elcoosp/quoin/commit/f33a474504c38f7b23fba3d8f0dd71a711234264))
- **macros-core**: Use let-chains for cleaner pattern matching ([e121220](https://github.com/elcoosp/quoin/commit/e121220818712440889bfed1aef778f475a62bb1))
- **macros-core**: Add unused imports allows for feature-gated shadcn paths ([0e4dd0f](https://github.com/elcoosp/quoin/commit/0e4dd0fe3b1945f068fffad5e9e7bbfada339874))

### 🎨 Style – Code style changes
- **macros-core**: Minor formatting fixes in gpui emit ([43ee58c](https://github.com/elcoosp/quoin/commit/43ee58cd3e3f2075704e3753f009c5bd9c767d9e))

### 👷 CI – Continuous integration
- Overhaul GitHub Actions workflow for better caching and parallelism ([4e4f66f](https://github.com/elcoosp/quoin/commit/4e4f66fb88f2419f4624a3eb5beddf3310f8e720))

### 📚 Documentation – Documentation changes
- Doc ([035f1ea](https://github.com/elcoosp/quoin/commit/035f1ead0958d7bcbe009f9b13355d3dd94a8664))
- State of refactor ([48f0b51](https://github.com/elcoosp/quoin/commit/48f0b517ab304cd21ef12fa47f1435196a4df097))
- Update contributing guidelines with Rust 1.88+ and conventional commits ([4bc207e](https://github.com/elcoosp/quoin/commit/4bc207ef88e6690108f94901ac1fb41bb647c04f))
- Completely revamp README with better structure, TOC, and updated examples ([48ea1c3](https://github.com/elcoosp/quoin/commit/48ea1c3e5381d895970347e2a776ea0c1ce08bbb))

### 🔧 Fixed – Bug fixes and corrections
- Leptos working ([4474afd](https://github.com/elcoosp/quoin/commit/4474afd683c69aab8ce88186312291f601eba006))
- All examples build ([ec6a510](https://github.com/elcoosp/quoin/commit/ec6a510c64c4eeb839e0e2b2fa6aef29191e94c5))
- Render_gpui ([426acc6](https://github.com/elcoosp/quoin/commit/426acc6bafbcc71f8fd85e4fd471e63fa9e6d847))
- Build examples ([f5c88ce](https://github.com/elcoosp/quoin/commit/f5c88ce178993168c1e9d207b7d5fcb8fa154fba))
- **tests**: Remove unsound unsafe Send+Sync impls in Dioxus conformance ([d715e6a](https://github.com/elcoosp/quoin/commit/d715e6a6cff24e7be035bbf84671f428eb20608f))
- Leak VirtualDom in Dioxus conformance test to avoid TLS destruction ([a0d5f00](https://github.com/elcoosp/quoin/commit/a0d5f00ef52ff565a36865855306c979fc14b63d))
- Remove orphaned code in render_dioxus.rs after sed refactor ([fabfb94](https://github.com/elcoosp/quoin/commit/fabfb945f788194618568a9aecb1d4bd691d7cc2))
- Correct duplicate import and orphaned code in render_dioxus.rs ([2c3952c](https://github.com/elcoosp/quoin/commit/2c3952c8d4346565f92457eca392436fe070f972))
- Clean rebuild of render_dioxus.rs with shared helpers ([be51324](https://github.com/elcoosp/quoin/commit/be51324365e98dd7dbfcc2f658253388245b47f0))
- Cleanly apply shared helpers to render_dioxus.rs ([a93982b](https://github.com/elcoosp/quoin/commit/a93982b13aca52960e74fc67097addaa1b254286))
- Render dioxus leptos ([9ffb561](https://github.com/elcoosp/quoin/commit/9ffb561f8faa458f9408a45115d35a77a9c6c286))
- Use common helpers render leptos ([82b8ec0](https://github.com/elcoosp/quoin/commit/82b8ec0cfa2b1e830ad1aaa019a9c9f3ec9eceb8))
- Remaining items from refactor ([ace39b6](https://github.com/elcoosp/quoin/commit/ace39b60392ddb06c7589f43c4fc3734bc50b786))
- **core**: Bound cancellation waker registry to prevent memory leaks on drop ([9b4da05](https://github.com/elcoosp/quoin/commit/9b4da054ac9f88d9596e9bf7c345980b12fc1c10))
- **dioxus**: Use const initialization for thread_local ([d902042](https://github.com/elcoosp/quoin/commit/d902042b1ec5244e217a437a018aa908fbafc16a))

### 🚀 Added – New features and capabilities
- Leptos shadcn example fully working ([f07f132](https://github.com/elcoosp/quoin/commit/f07f132b690a6073d1fbfff5d46e02d620b91c4b))
- **macros**: Add skeleton, skeleton_text, skeleton_avatar elements ([ea865bb](https://github.com/elcoosp/quoin/commit/ea865bb6fd8b1f25362b9efe5b300730ef2cbd6d))
- **macros**: Add progress element with value/indeterminate variants ([9318d32](https://github.com/elcoosp/quoin/commit/9318d3215c02e7fed458531b36848fb836995a0e))
- **macros**: Add checkbox element with checked/on_checked_change ([bce4844](https://github.com/elcoosp/quoin/commit/bce4844544fa99aec0a913a99030a0c7f3dbdeb0))
- **macros**: Add switch element with checked/on_checked_change ([42d66b6](https://github.com/elcoosp/quoin/commit/42d66b6163c06ebc3d817534319f341d18663c50))
- **macros**: Add radio and radio_group elements ([ce19238](https://github.com/elcoosp/quoin/commit/ce19238cae440138bd372f61d74fbe007e72e766))
- **macros**: Add slider element with min/max/value/step ([4671487](https://github.com/elcoosp/quoin/commit/46714875ff0c8b5f44dfd67f8c3c2a9025c27b93))
- Fix render_gpui, align editions ([68a67e4](https://github.com/elcoosp/quoin/commit/68a67e42546b1cc223ebeefce767f13f3fb8c760))
- Split render functions ([007a328](https://github.com/elcoosp/quoin/commit/007a328064d631e34219ced4a5851de76e5af8cd))
- **core**: Add JoinHandle::detach and document structured concurrency lifecycle ([46a37fa](https://github.com/elcoosp/quoin/commit/46a37fa56610a7258b01ba8578cd5ce77f822926))
- **core**: Export JoinHandle in prelude ([1d4fe22](https://github.com/elcoosp/quoin/commit/1d4fe22576b8f51877b4ffd52e15efa2f8fcadbc))
- **dioxus**: Implement proper JoinHandle abort/detach with Abortable ([bfce862](https://github.com/elcoosp/quoin/commit/bfce8623ac750ce43a04d26c41559ce4c1ce41f8))
- **floem**: Implement proper JoinHandle abort/detach with Abortable ([215f985](https://github.com/elcoosp/quoin/commit/215f985f5b8c8f231e7c4d816c3165effc16f462))
- **gpui**: Implement proper JoinHandle abort/detach with Mutex and Task::detach ([6390d40](https://github.com/elcoosp/quoin/commit/6390d409e0e3ae376fa27d646fb70541ff4a1dd8))
- **leptos**: Implement proper JoinHandle abort/detach with Abortable ([9e92310](https://github.com/elcoosp/quoin/commit/9e92310a88e0a6dde2f646ad6ada86a03512748f))
- **ui-gpui**: Add inherent render methods to UCP adapter structs ([ab43802](https://github.com/elcoosp/quoin/commit/ab43802505f370908ee4b1f2be084d92808962d1))
- **xilem**: Implement proper JoinHandle abort/detach and fix request_update recursion ([f21c3d0](https://github.com/elcoosp/quoin/commit/f21c3d0b5a0dc6e92d095be9c3b9dbba274aaf14))
- **core**: Expand API surface — derived signals, resources, batching, local executors ([5bc5482](https://github.com/elcoosp/quoin/commit/5bc5482ee27fab5d467ccf5bf03a729d8cbfc5b9))

### 🧪 Tests – Test additions and changes
- Add more conformance & xilem ([f2a04ed](https://github.com/elcoosp/quoin/commit/f2a04ed4a29c103cb2af215a0a5b08cf6e34998d))
- **conformance**: Add conformance tests for executor abort and detach semantics ([ac1a062](https://github.com/elcoosp/quoin/commit/ac1a062c3b7c1a29fdf5abd551b2ff1d24e2da15))

### 🧹 Cleanup – Code maintenance and housekeeping
- Gitignore ([2881be3](https://github.com/elcoosp/quoin/commit/2881be36f1a3e4ad08422822e72ffdb90b4d3a89))
- Readme ([ea40b5d](https://github.com/elcoosp/quoin/commit/ea40b5d3d5062abcebf010f55469c3ba6469c849))
- Readme ([722a808](https://github.com/elcoosp/quoin/commit/722a808c618db9dea01f0a95ee80b3b0f0abf91e))
- Add workspace package metadata and rust-version 1.88 ([44ddd8c](https://github.com/elcoosp/quoin/commit/44ddd8c0ad2935138a15853836afe384e778c7d3))
- Update dependencies ([b383ce7](https://github.com/elcoosp/quoin/commit/b383ce74d486a4c38a280982730566d8e35a5ff8))
- Update copyright holder in LICENSE ([4fc5f84](https://github.com/elcoosp/quoin/commit/4fc5f84f341c43ae95b519302e1c25c5c841f951))
- Add lint-core, lint-gpui, and git-cliff targets ([00ad762](https://github.com/elcoosp/quoin/commit/00ad7626de4d78d500be50e713fc6f32f4838b29))
- **conformance**: Use workspace metadata ([d39b68a](https://github.com/elcoosp/quoin/commit/d39b68a0958f99ed0fade739167ce206b12e8d46))
- **core**: Use workspace metadata ([9cd331a](https://github.com/elcoosp/quoin/commit/9cd331a90caa39a0d6417dda0f9fc737177ee341))
- **dioxus**: Use workspace metadata and add wasm-bindgen-futures ([a2e5c6c](https://github.com/elcoosp/quoin/commit/a2e5c6cbc1a082859bcbc9f8839c5b37b99eef45))
- **floem**: Use workspace metadata ([1fe229b](https://github.com/elcoosp/quoin/commit/1fe229baf1ab9fe05384986108c260820fb058b5))
- **gpui**: Use workspace metadata ([c880469](https://github.com/elcoosp/quoin/commit/c88046951648bb74e0c0b3758bee4d54e6fdac23))
- **leptos**: Use workspace metadata and add wasm-bindgen-futures ([18f7e50](https://github.com/elcoosp/quoin/commit/18f7e50f7a6a3c5ec77ce80ffb33a9b54aa13250))
- **macros-core**: Upgrade syn to v3 ([e675452](https://github.com/elcoosp/quoin/commit/e67545242d1f184822851603c46345e347b2e191))
- **macros-tests**: Prevent publishing of quoin-macros-tests ([72683bc](https://github.com/elcoosp/quoin/commit/72683bc3bf509c68e6ec6aee8aee0f5699fc4cf8))
- **macros**: Upgrade syn to v3 and use workspace metadata ([9562360](https://github.com/elcoosp/quoin/commit/956236018811f0f17cf912bedbbc74d02ad8d3fc))
- **xilem**: Use workspace metadata ([616b9c6](https://github.com/elcoosp/quoin/commit/616b9c6d908855292ceb34b87a00b2fc6c330576))
- **quoin**: Use workspace metadata and update description ([01b8414](https://github.com/elcoosp/quoin/commit/01b84141564ba118349f62b432bc6112ddce4e79))
- Cliff ([1f965ca](https://github.com/elcoosp/quoin/commit/1f965ca3d3c590a20512195f518c51107f79b3bf))

<!-- generated by git-cliff -->
