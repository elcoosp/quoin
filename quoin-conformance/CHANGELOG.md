# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-11

### Bug Fixes
- update rust crate gpui to v0.3.12 by @renovate[bot] in [#30](https://github.com/elcoosp/quoin/pull/30) ([213b69d](https://github.com/elcoosp/quoin/commit/213b69da6dd809d51872d62a5feabc674fea045a))
- fully qualify TestContextProvider in effect suite macro by @elcoosp ([0701a68](https://github.com/elcoosp/quoin/commit/0701a6849ac497b3b33f09d5593113cf71e26102))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
- smoke test by @elcoosp ([43bf300](https://github.com/elcoosp/quoin/commit/43bf300705be194d36604d7d89a54422f99acc63))
- warns by @elcoosp ([39d52de](https://github.com/elcoosp/quoin/commit/39d52dec1bc87055a3885a3933df2756146bf7e9))
- conformance dioxus leptos by @elcoosp ([def3f6e](https://github.com/elcoosp/quoin/commit/def3f6e58b6dbab876eb7fbefc14d6cfa66a610f))

### Documentation
- reword the Executor::sleep contract docs by @elcoosp ([cf6a4e6](https://github.com/elcoosp/quoin/commit/cf6a4e6c1c58a089dfd2052dc211732c653e41a6))
- add doc code by @elcoosp ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))
- readme by @elcoosp ([02f8361](https://github.com/elcoosp/quoin/commit/02f83612af6717970c324abc915a7bce631111cf))

### Features
- mutable signal by @elcoosp ([2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))
- chunk 2 by @elcoosp ([55f37f7](https://github.com/elcoosp/quoin/commit/55f37f7f32db0c2fe8c19cd7753786fa13802df8))

### Miscellaneous
- mark examples/test-crates as publish = false by @elcoosp ([972b245](https://github.com/elcoosp/quoin/commit/972b245679dc1fd7f49c301fe93ae32406cdb32b))
- wire quoin-style, muda patch, xkb config, gpui-pre by @elcoosp ([b5fadfb](https://github.com/elcoosp/quoin/commit/b5fadfb3e0881a4804806b804fb15c2217bcb4cd))
- use workspace metadata by @elcoosp ([d39b68a](https://github.com/elcoosp/quoin/commit/d39b68a0958f99ed0fade739167ce206b12e8d46))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- use edition 2024 by @elcoosp ([c18acfb](https://github.com/elcoosp/quoin/commit/c18acfb6f58778b342241de1db66e8fc0114ca57))

### Refactor
- remove SleepExt and add executor sleep conformance checks by @elcoosp ([6e8ac79](https://github.com/elcoosp/quoin/commit/6e8ac79176098d84dbfced118c35300a01410f97))
- generate sync and async conformance tests from one macro by @elcoosp ([856ef44](https://github.com/elcoosp/quoin/commit/856ef4444481bbe5ef81fb732f97d8476edd2af2))

### Testing
- import TestContextProvider explicitly in test runner by @elcoosp ([920ef32](https://github.com/elcoosp/quoin/commit/920ef326403579dd03462a1544374ea3da53ee45))
- run the effect suite against the testkit context by @elcoosp ([3519eec](https://github.com/elcoosp/quoin/commit/3519eec6f86202588423e802a25a28f57e23e0cc))
- add the effect conformance suite and macro by @elcoosp ([041a28c](https://github.com/elcoosp/quoin/commit/041a28c49b851c4e335085ebbf0599592d5aaffe))
- make trait-bound smoke test compile-check real types by @elcoosp ([1bb82ae](https://github.com/elcoosp/quoin/commit/1bb82ae5fa3d65e7f686758409edd68027b6a023))
- add testkit dev-dependency and workspace lints by @elcoosp ([3a866f0](https://github.com/elcoosp/quoin/commit/3a866f018748f83ac6785a334fac4549a89fb93a))
- expand suite with Phase 1.5 API checks by @elcoosp ([db37725](https://github.com/elcoosp/quoin/commit/db3772526a8a46762ce48d5e8b25ccda6b7ae622))
- add conformance tests for executor abort and detach semantics by @elcoosp ([ac1a062](https://github.com/elcoosp/quoin/commit/ac1a062c3b7c1a29fdf5abd551b2ff1d24e2da15))
- add more conformance & xilem by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([f2a04ed](https://github.com/elcoosp/quoin/commit/f2a04ed4a29c103cb2af215a0a5b08cf6e34998d))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))
- new conformance with macro by @elcoosp ([da7948e](https://github.com/elcoosp/quoin/commit/da7948e760e357a4788c5e32542cf80606e880e8))
- leptos conformance by @elcoosp ([1800728](https://github.com/elcoosp/quoin/commit/180072848aacccb35faf7b23f0590ab52f1e9b1b))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
