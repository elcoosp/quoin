# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-14

### Bug Fixes
- remove sandbox muda patch and disable default gtk feature by @elcoosp ([255a311](https://github.com/elcoosp/quoin/commit/255a3113040bf3a866abc1754fa37931170c1daf))
- keep clipboard_write_text non-const on native by @elcoosp ([4c3545f](https://github.com/elcoosp/quoin/commit/4c3545fb4e964db6d82f5d093bd3e2a2ce8033bb))
- disable default features for leptos-shadcn-ui in quoin-leptos by @elcoosp ([1aa6bcc](https://github.com/elcoosp/quoin/commit/1aa6bcc84f18c8dd2f6cc7979ec02c461c8eb1b8))
- compiling ucp leptos by @elcoosp ([b3edf4f](https://github.com/elcoosp/quoin/commit/b3edf4ff0173f45e2c89eebb0006156a7f1d651b))
- use fork of leptos-shadcn-ui by @elcoosp ([c2f1370](https://github.com/elcoosp/quoin/commit/c2f1370d91967dce2aa9b69d3df3feae7d21c606))
- explicit leptos-shadcn feature by @elcoosp ([37d61a5](https://github.com/elcoosp/quoin/commit/37d61a5c9b85feca0ae50bbec8d16a1efafcf5aa))
- render_leptos by @elcoosp ([4627778](https://github.com/elcoosp/quoin/commit/46277787aac977053f4647236af53d55aca48f19))
- almost working render_leptos by @elcoosp ([4b9c884](https://github.com/elcoosp/quoin/commit/4b9c8846f6c1ad38e31b60025b72b8c1aa0049f9))
- quoin macro dioxus by @elcoosp ([9c8940c](https://github.com/elcoosp/quoin/commit/9c8940cb5fb254dbde7bb7845eaf05b61af7ca16))
- readme by @elcoosp ([f3ed105](https://github.com/elcoosp/quoin/commit/f3ed1052386c7540577898876a4e9d1a8a507c7e))
- rm move from listeners by @elcoosp ([bd14c75](https://github.com/elcoosp/quoin/commit/bd14c75e40be7fc1d399c880d47e055baf93a253))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
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
- expand leptos-shadcn-ui features 8 -> 28 by @elcoosp ([1b6271f](https://github.com/elcoosp/quoin/commit/1b6271f8dee2db35c298727f2e7b324c67184ddd))
- implement proper JoinHandle abort/detach with Abortable by @elcoosp ([9e92310](https://github.com/elcoosp/quoin/commit/9e92310a88e0a6dde2f646ad6ada86a03512748f))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- mutable signal by @elcoosp ([2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))

### Miscellaneous
- inject missing metadata and point to root README by @elcoosp ([45a16d5](https://github.com/elcoosp/quoin/commit/45a16d56d41e6e5c9958254292d93155262d8805))
- inject precisely curated keywords for crates.io by @elcoosp ([8722cab](https://github.com/elcoosp/quoin/commit/8722cab4895292fb33f6f96a707b25a9abb9c7c2))
- bump subcrate dependencies and drop const from clipboard by @elcoosp ([84b6dd7](https://github.com/elcoosp/quoin/commit/84b6dd703224e2d8421ba3bafdc85e6a6690abcc))
- use workspace metadata and add wasm-bindgen-futures by @elcoosp ([18f7e50](https://github.com/elcoosp/quoin/commit/18f7e50f7a6a3c5ec77ce80ffb33a9b54aa13250))
- readme by @elcoosp ([722a808](https://github.com/elcoosp/quoin/commit/722a808c618db9dea01f0a95ee80b3b0f0abf91e))
- readme by @elcoosp ([ea40b5d](https://github.com/elcoosp/quoin/commit/ea40b5d3d5062abcebf010f55469c3ba6469c849))
- more doc + ci by @elcoosp ([f55a2d6](https://github.com/elcoosp/quoin/commit/f55a2d6a794ef3d5b5dfca73a09c3199b4c6c105))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- use edition 2024 by @elcoosp ([c18acfb](https://github.com/elcoosp/quoin/commit/c18acfb6f58778b342241de1db66e8fc0114ca57))

### Performance
- use shared worker pool and implement LocalExecutor by @elcoosp ([d876543](https://github.com/elcoosp/quoin/commit/d87654359499824ce2d62ae5eed8c38684ac02b4))

### Refactor
- Send bound on ReactiveContext signals by @elcoosp ([5c0009a](https://github.com/elcoosp/quoin/commit/5c0009ac6ca15b0f57b04a6e6edccdef33149c65))
- remove SleepExt implementation from test harness by @elcoosp ([e8d4b68](https://github.com/elcoosp/quoin/commit/e8d4b6804bd5d3d45ee3d6e85ef9e644910b626d))
- reuse shared abort join handle by @elcoosp ([4f7a626](https://github.com/elcoosp/quoin/commit/4f7a626dd49fbe45242074a514538484d877507c))

### Testing
- prune unused duration imports from conformance suites by @elcoosp ([af9df13](https://github.com/elcoosp/quoin/commit/af9df132e2890313a84f28b71087c2412200dc9b))
- update conformance harness by @elcoosp ([f273a8e](https://github.com/elcoosp/quoin/commit/f273a8e31149d33b5dd9d8971562b3569a441d90))
- implement LocalExecutor for conformance harness by @elcoosp ([85c48ec](https://github.com/elcoosp/quoin/commit/85c48ec5d7895275268af519097995520fb5c6b1))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))
- new conformance with macro by @elcoosp ([da7948e](https://github.com/elcoosp/quoin/commit/da7948e760e357a4788c5e32542cf80606e880e8))
- conformance by @elcoosp ([4d9368e](https://github.com/elcoosp/quoin/commit/4d9368e5912f052cbb51be5fb452a045d5e0feaf))
- leptos conformance by @elcoosp ([1800728](https://github.com/elcoosp/quoin/commit/180072848aacccb35faf7b23f0590ab52f1e9b1b))

### build
- inherit workspace lints by @elcoosp ([5c77c50](https://github.com/elcoosp/quoin/commit/5c77c50b46a2512c2f2ec47ef44d46f1c589494f))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
