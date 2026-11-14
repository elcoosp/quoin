# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-14

### Bug Fixes
- remove sandbox muda patch and disable default gtk feature by @elcoosp ([255a311](https://github.com/elcoosp/quoin/commit/255a3113040bf3a866abc1754fa37931170c1daf))
- unify global state semantics and use worker pool by @elcoosp ([d669ab4](https://github.com/elcoosp/quoin/commit/d669ab4722a6f2d0a689b966578072e7856e463b))
- use const initialization for thread_local by @elcoosp ([d902042](https://github.com/elcoosp/quoin/commit/d902042b1ec5244e217a437a018aa908fbafc16a))
- leak VirtualDom in Dioxus conformance test to avoid TLS destruction by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([a0d5f00](https://github.com/elcoosp/quoin/commit/a0d5f00ef52ff565a36865855306c979fc14b63d))
- remove unsound unsafe Send+Sync impls in Dioxus conformance by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([d715e6a](https://github.com/elcoosp/quoin/commit/d715e6a6cff24e7be035bbf84671f428eb20608f))
- add inherent methods on DioxusSignal to avoid trait conflicts by @elcoosp ([a4ad6e3](https://github.com/elcoosp/quoin/commit/a4ad6e3f9a5bdd75d18b0824a5ba0c1ca842b8d5))
- add Default for DioxusContext and remove needless returns in dioxus emitter by @elcoosp ([78f194c](https://github.com/elcoosp/quoin/commit/78f194c0c6f6ee44c8a79a3b2dc7215e4bbf573e))
- quoin macro dioxus by @elcoosp ([9c8940c](https://github.com/elcoosp/quoin/commit/9c8940cb5fb254dbde7bb7845eaf05b61af7ca16))
- clippy by @elcoosp ([db643bb](https://github.com/elcoosp/quoin/commit/db643bb49dc84201aef79d61120cc7f43615b4e7))
- readme by @elcoosp ([f3ed105](https://github.com/elcoosp/quoin/commit/f3ed1052386c7540577898876a4e9d1a8a507c7e))
- rm move from listeners by @elcoosp ([bd14c75](https://github.com/elcoosp/quoin/commit/bd14c75e40be7fc1d399c880d47e055baf93a253))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
- working dioxus counter by @elcoosp ([0d12bac](https://github.com/elcoosp/quoin/commit/0d12bac7884da3f174665af9633dccf471fb91bd))
- up signal by @elcoosp ([b76cd35](https://github.com/elcoosp/quoin/commit/b76cd35697c2daba2d10927f6db8d36ada25d0ae))
- conformance dioxus leptos by @elcoosp ([def3f6e](https://github.com/elcoosp/quoin/commit/def3f6e58b6dbab876eb7fbefc14d6cfa66a610f))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([65a7312](https://github.com/elcoosp/quoin/commit/65a731230cf3db2b64de685c2dc0ad1d5ec3d39e))
- enforce SKILL.md rules (jsDelivr logos, absolute links, remove license sections) by @elcoosp ([37ce0b8](https://github.com/elcoosp/quoin/commit/37ce0b814289a4fa78d487ff9bf2774bb3528391))
- note native reactive effect macro support on Xilem by @elcoosp ([b79a92e](https://github.com/elcoosp/quoin/commit/b79a92e93e814c2e13ff34c3a8eca040ffa0bc6b))
- advertise reactive effects and AnySignal in the README by @elcoosp ([e9f2766](https://github.com/elcoosp/quoin/commit/e9f27662d88889aee9c60825bd41e9901c31144c))
- document devtools demo, benchmarks, and lint policy by @elcoosp ([0fcfa65](https://github.com/elcoosp/quoin/commit/0fcfa65494ebe3760f45d3e574a60b0ba8638382))
- revamp README with Phase 1.5 features and examples by @elcoosp ([acc86c5](https://github.com/elcoosp/quoin/commit/acc86c5028587e31657355ca9fbb6f8f42560276))
- completely revamp README with better structure, TOC, and updated examples by @elcoosp ([48ea1c3](https://github.com/elcoosp/quoin/commit/48ea1c3e5381d895970347e2a776ea0c1ce08bbb))
- doc by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([035f1ea](https://github.com/elcoosp/quoin/commit/035f1ead0958d7bcbe009f9b13355d3dd94a8664))
- add doc code by @elcoosp ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- readme & rm navigator by @elcoosp ([2735176](https://github.com/elcoosp/quoin/commit/27351767f6d81c270d624ba9fc1eeff71ff57200))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))
- readme by @elcoosp ([537b5c8](https://github.com/elcoosp/quoin/commit/537b5c845c64400de3ba2905eb11b38346bda4c5))
- readme by @elcoosp ([02f8361](https://github.com/elcoosp/quoin/commit/02f83612af6717970c324abc915a7bce631111cf))
- add spec by @elcoosp ([13ba55c](https://github.com/elcoosp/quoin/commit/13ba55c747915030959f70b9bee2fecf81f5e0dd))

### Features
- keys { .. } goes native on the web backends ([#35](https://github.com/elcoosp/quoin/pull/35)) by @elcoosp in [#35](https://github.com/elcoosp/quoin/pull/35) ([62a0719](https://github.com/elcoosp/quoin/commit/62a07192d10c221951def4ee2b2effc937cc15ad))
- bridge fine-grained signals into the effect graph ([#29](https://github.com/elcoosp/quoin/pull/29)) by @elcoosp in [#29](https://github.com/elcoosp/quoin/pull/29) ([52d6ec7](https://github.com/elcoosp/quoin/commit/52d6ec7bee3e883bd149d1e1662e85c2a318a998))
- implement LocalExecutor for DioxusExecutor by @elcoosp ([8dbaaee](https://github.com/elcoosp/quoin/commit/8dbaaee905d6e46b7a054459a93ceb6835cfe0c5))
- implement proper JoinHandle abort/detach with Abortable by @elcoosp ([bfce862](https://github.com/elcoosp/quoin/commit/bfce8623ac750ce43a04d26c41559ce4c1ce41f8))
- add clipboard_write_text and wire clipboard_button emitter by @elcoosp ([2f9048a](https://github.com/elcoosp/quoin/commit/2f9048ae6208f3a227b384a0041a4875829cc728))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- mutable signal by @elcoosp ([2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))

### Miscellaneous
- inject missing metadata and point to root README by @elcoosp ([45a16d5](https://github.com/elcoosp/quoin/commit/45a16d56d41e6e5c9958254292d93155262d8805))
- inject precisely curated keywords for crates.io by @elcoosp ([8722cab](https://github.com/elcoosp/quoin/commit/8722cab4895292fb33f6f96a707b25a9abb9c7c2))
- use workspace metadata and add wasm-bindgen-futures by @elcoosp ([a2e5c6c](https://github.com/elcoosp/quoin/commit/a2e5c6cbc1a082859bcbc9f8839c5b37b99eef45))
- readme by @elcoosp ([722a808](https://github.com/elcoosp/quoin/commit/722a808c618db9dea01f0a95ee80b3b0f0abf91e))
- readme by @elcoosp ([ea40b5d](https://github.com/elcoosp/quoin/commit/ea40b5d3d5062abcebf010f55469c3ba6469c849))
- more doc + ci by @elcoosp ([f55a2d6](https://github.com/elcoosp/quoin/commit/f55a2d6a794ef3d5b5dfca73a09c3199b4c6c105))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- use edition 2024 by @elcoosp ([c18acfb](https://github.com/elcoosp/quoin/commit/c18acfb6f58778b342241de1db66e8fc0114ca57))

### Refactor
- Send bound on ReactiveContext signals by @elcoosp ([5c0009a](https://github.com/elcoosp/quoin/commit/5c0009ac6ca15b0f57b04a6e6edccdef33149c65))
- remove SleepExt implementation from test harness by @elcoosp ([8da49b3](https://github.com/elcoosp/quoin/commit/8da49b3896d26e8ee3533b82bb447b415de6d338))
- reuse shared abort join handle by @elcoosp ([48c6e9d](https://github.com/elcoosp/quoin/commit/48c6e9da1481408dbb40257c6b1c9cdb475d81eb))

### Testing
- prune unused duration imports from conformance suites by @elcoosp ([1ff715f](https://github.com/elcoosp/quoin/commit/1ff715f1e944aa74c8fa41a801e3a337addfa83d))
- update conformance harness by @elcoosp ([2d6506f](https://github.com/elcoosp/quoin/commit/2d6506f91a461fad610ac819fb7ee63fe613f6bf))
- implement LocalExecutor for conformance harness by @elcoosp ([ae18a88](https://github.com/elcoosp/quoin/commit/ae18a882685006e9dcbcd16984aed44526470b0a))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))
- new conformance with macro by @elcoosp ([da7948e](https://github.com/elcoosp/quoin/commit/da7948e760e357a4788c5e32542cf80606e880e8))
- conformance by @elcoosp ([4d9368e](https://github.com/elcoosp/quoin/commit/4d9368e5912f052cbb51be5fb452a045d5e0feaf))
- leptos conformance by @elcoosp ([1800728](https://github.com/elcoosp/quoin/commit/180072848aacccb35faf7b23f0590ab52f1e9b1b))

### build
- inherit workspace lints by @elcoosp ([3bb3414](https://github.com/elcoosp/quoin/commit/3bb3414b9089447947f21388a226a680d0271405))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
