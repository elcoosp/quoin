# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-11

### Bug Fixes
- update rust crate gpui_platform to v0.3.12 by @renovate[bot] in [#31](https://github.com/elcoosp/quoin/pull/31) ([1249706](https://github.com/elcoosp/quoin/commit/1249706aec002588d6083452ed6dafc55255f610))
- update rust crate gpui to v0.3.12 by @renovate[bot] in [#30](https://github.com/elcoosp/quoin/pull/30) ([213b69d](https://github.com/elcoosp/quoin/commit/213b69da6dd809d51872d62a5feabc674fea045a))
- unify global state, batch notifies, and refactor signal by @elcoosp ([478f05d](https://github.com/elcoosp/quoin/commit/478f05d8a47e5038e6e2d6c475f9224120fda6fd))
- extract UpdateNotifier type alias to reduce type complexity in GpuiContext by @elcoosp ([154127b](https://github.com/elcoosp/quoin/commit/154127be77ec96684bcf750e5092e2ca6b615a4e))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
- auto notify signal updates gpui by @elcoosp ([0bb0fb9](https://github.com/elcoosp/quoin/commit/0bb0fb9f4b786956177a48f87190ab0f62e634fc))
- working gpui example by @elcoosp ([a201750](https://github.com/elcoosp/quoin/commit/a201750def56dbc922e5b97500406c63af98db37))
- warns by @elcoosp ([39d52de](https://github.com/elcoosp/quoin/commit/39d52dec1bc87055a3885a3933df2756146bf7e9))
- conformance dioxus leptos by @elcoosp ([def3f6e](https://github.com/elcoosp/quoin/commit/def3f6e58b6dbab876eb7fbefc14d6cfa66a610f))

### Documentation
- doc by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([035f1ea](https://github.com/elcoosp/quoin/commit/035f1ead0958d7bcbe009f9b13355d3dd94a8664))
- add doc code by @elcoosp ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))
- readme by @elcoosp ([02f8361](https://github.com/elcoosp/quoin/commit/02f83612af6717970c324abc915a7bce631111cf))

### Features
- implement whole-window refresh notifiers by @elcoosp ([d80968e](https://github.com/elcoosp/quoin/commit/d80968e5e3e211f6cbe3c21980a5f1bdb3629c12))
- implement proper JoinHandle abort/detach with Mutex and Task::detach by @elcoosp ([6390d40](https://github.com/elcoosp/quoin/commit/6390d409e0e3ae376fa27d646fb70541ff4a1dd8))
- fix render_gpui, align editions by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([68a67e4](https://github.com/elcoosp/quoin/commit/68a67e42546b1cc223ebeefce767f13f3fb8c760))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- .into() ctx by @elcoosp ([d750859](https://github.com/elcoosp/quoin/commit/d750859c215980910592ff17479ab4063128e7e6))
- mutable signal by @elcoosp ([2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))

### Miscellaneous
- pin gpui-pre and platform version constraints by @elcoosp ([0524bed](https://github.com/elcoosp/quoin/commit/0524bedab725ba0c4d1e36efbb2f75366c15cfab))
- use workspace metadata by @elcoosp ([c880469](https://github.com/elcoosp/quoin/commit/c88046951648bb74e0c0b3758bee4d54e6fdac23))
- more doc + ci by @elcoosp ([f55a2d6](https://github.com/elcoosp/quoin/commit/f55a2d6a794ef3d5b5dfca73a09c3199b4c6c105))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- use edition 2024 by @elcoosp ([c18acfb](https://github.com/elcoosp/quoin/commit/c18acfb6f58778b342241de1db66e8fc0114ca57))

### Refactor
- Send bound on ReactiveContext signals by @elcoosp ([5c0009a](https://github.com/elcoosp/quoin/commit/5c0009ac6ca15b0f57b04a6e6edccdef33149c65))
- update test harness to use Executor::sleep by @elcoosp ([44c32b5](https://github.com/elcoosp/quoin/commit/44c32b5d198946afab4678fc97a4eda6d4a1ebbb))
- reuse shared globals, notifier signal, and join handle by @elcoosp ([63a326e](https://github.com/elcoosp/quoin/commit/63a326eea88a61ec25924d5c73a1dd20c02c8fd6))

### Testing
- update conformance harness by @elcoosp ([cca4584](https://github.com/elcoosp/quoin/commit/cca4584cfa2a6d8b05b823bf0ead15ad4ebb12ed))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))
- new conformance with macro by @elcoosp ([da7948e](https://github.com/elcoosp/quoin/commit/da7948e760e357a4788c5e32542cf80606e880e8))
- gpui conformance by @elcoosp ([c1928fc](https://github.com/elcoosp/quoin/commit/c1928fce70d80970b69ff689eccb53d60582ac20))
- leptos conformance by @elcoosp ([1800728](https://github.com/elcoosp/quoin/commit/180072848aacccb35faf7b23f0590ab52f1e9b1b))

### build
- inherit workspace lints by @elcoosp ([3be1631](https://github.com/elcoosp/quoin/commit/3be16315a10802d416e89da0d682c7e77e8b35f4))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
