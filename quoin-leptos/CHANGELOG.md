# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-11

### Bug Fixes
- keep clipboard_write_text non-const on native by @elcoosp ([4c3545f](https://github.com/elcoosp/quoin/commit/4c3545fb4e964db6d82f5d093bd3e2a2ce8033bb))
- disable default features for leptos-shadcn-ui in quoin-leptos by @elcoosp ([1aa6bcc](https://github.com/elcoosp/quoin/commit/1aa6bcc84f18c8dd2f6cc7979ec02c461c8eb1b8))
- compiling ucp leptos by @elcoosp ([b3edf4f](https://github.com/elcoosp/quoin/commit/b3edf4ff0173f45e2c89eebb0006156a7f1d651b))
- use fork of leptos-shadcn-ui by @elcoosp ([c2f1370](https://github.com/elcoosp/quoin/commit/c2f1370d91967dce2aa9b69d3df3feae7d21c606))
- explicit leptos-shadcn feature by @elcoosp ([37d61a5](https://github.com/elcoosp/quoin/commit/37d61a5c9b85feca0ae50bbec8d16a1efafcf5aa))
- render_leptos by @elcoosp ([4627778](https://github.com/elcoosp/quoin/commit/46277787aac977053f4647236af53d55aca48f19))
- almost working render_leptos by @elcoosp ([4b9c884](https://github.com/elcoosp/quoin/commit/4b9c8846f6c1ad38e31b60025b72b8c1aa0049f9))
- quoin macro dioxus by @elcoosp ([9c8940c](https://github.com/elcoosp/quoin/commit/9c8940cb5fb254dbde7bb7845eaf05b61af7ca16))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
- conformance dioxus leptos by @elcoosp ([def3f6e](https://github.com/elcoosp/quoin/commit/def3f6e58b6dbab876eb7fbefc14d6cfa66a610f))

### Documentation
- doc by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([035f1ea](https://github.com/elcoosp/quoin/commit/035f1ead0958d7bcbe009f9b13355d3dd94a8664))
- add doc code by @elcoosp ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))
- readme by @elcoosp ([02f8361](https://github.com/elcoosp/quoin/commit/02f83612af6717970c324abc915a7bce631111cf))

### Features
- bridge fine-grained signals into the effect graph ([#29](https://github.com/elcoosp/quoin/pull/29)) by @elcoosp in [#29](https://github.com/elcoosp/quoin/pull/29) ([52d6ec7](https://github.com/elcoosp/quoin/commit/52d6ec7bee3e883bd149d1e1662e85c2a318a998))
- expand leptos-shadcn-ui features 8 -> 28 by @elcoosp ([1b6271f](https://github.com/elcoosp/quoin/commit/1b6271f8dee2db35c298727f2e7b324c67184ddd))
- implement proper JoinHandle abort/detach with Abortable by @elcoosp ([9e92310](https://github.com/elcoosp/quoin/commit/9e92310a88e0a6dde2f646ad6ada86a03512748f))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- mutable signal by @elcoosp ([2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))

### Miscellaneous
- bump subcrate dependencies and drop const from clipboard by @elcoosp ([84b6dd7](https://github.com/elcoosp/quoin/commit/84b6dd703224e2d8421ba3bafdc85e6a6690abcc))
- use workspace metadata and add wasm-bindgen-futures by @elcoosp ([18f7e50](https://github.com/elcoosp/quoin/commit/18f7e50f7a6a3c5ec77ce80ffb33a9b54aa13250))
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
