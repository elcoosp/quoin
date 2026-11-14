# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.1.0] - 2026-11-14

### Bug Fixes
- remove sandbox muda patch and disable default gtk feature by @elcoosp ([255a311](https://github.com/elcoosp/quoin/commit/255a3113040bf3a866abc1754fa37931170c1daf))
- update rust crate gpui-component to 0.9.0 by @renovate[bot] ([50a947a](https://github.com/elcoosp/quoin/commit/50a947a4e4c1d755fa04c1b2452efd48529e768a))
- update rust crate gpui to v0.3.12 by @renovate[bot] in [#30](https://github.com/elcoosp/quoin/pull/30) ([213b69d](https://github.com/elcoosp/quoin/commit/213b69da6dd809d51872d62a5feabc674fea045a))
- readme by @elcoosp ([f3ed105](https://github.com/elcoosp/quoin/commit/f3ed1052386c7540577898876a4e9d1a8a507c7e))
- opt deps & master gpui-component by @elcoosp ([f15e101](https://github.com/elcoosp/quoin/commit/f15e101dfc9d005eaa87fcd65695555e40acd7bf))
- rm move from listeners by @elcoosp ([bd14c75](https://github.com/elcoosp/quoin/commit/bd14c75e40be7fc1d399c880d47e055baf93a253))
- remove unused braced import, use explicit static str arms in resolve_icon by @elcoosp ([bca12ea](https://github.com/elcoosp/quoin/commit/bca12ea725869984807bbf88dec38c4a1c925456))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([65a7312](https://github.com/elcoosp/quoin/commit/65a731230cf3db2b64de685c2dc0ad1d5ec3d39e))
- note native reactive effect macro support on Xilem by @elcoosp ([b79a92e](https://github.com/elcoosp/quoin/commit/b79a92e93e814c2e13ff34c3a8eca040ffa0bc6b))
- advertise reactive effects and AnySignal in the README by @elcoosp ([e9f2766](https://github.com/elcoosp/quoin/commit/e9f27662d88889aee9c60825bd41e9901c31144c))
- document devtools demo, benchmarks, and lint policy by @elcoosp ([0fcfa65](https://github.com/elcoosp/quoin/commit/0fcfa65494ebe3760f45d3e574a60b0ba8638382))
- revamp README with Phase 1.5 features and examples by @elcoosp ([acc86c5](https://github.com/elcoosp/quoin/commit/acc86c5028587e31657355ca9fbb6f8f42560276))
- completely revamp README with better structure, TOC, and updated examples by @elcoosp ([48ea1c3](https://github.com/elcoosp/quoin/commit/48ea1c3e5381d895970347e2a776ea0c1ce08bbb))
- readme & rm navigator by @elcoosp ([2735176](https://github.com/elcoosp/quoin/commit/27351767f6d81c270d624ba9fc1eeff71ff57200))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))
- readme by @elcoosp ([537b5c8](https://github.com/elcoosp/quoin/commit/537b5c845c64400de3ba2905eb11b38346bda4c5))
- add spec by @elcoosp ([13ba55c](https://github.com/elcoosp/quoin/commit/13ba55c747915030959f70b9bee2fecf81f5e0dd))

### Features
- keys { .. } goes native on the web backends ([#35](https://github.com/elcoosp/quoin/pull/35)) by @elcoosp in [#35](https://github.com/elcoosp/quoin/pull/35) ([62a0719](https://github.com/elcoosp/quoin/commit/62a07192d10c221951def4ee2b2effc937cc15ad))
- implement runtime tree renderer for agnostic UI nodes by @elcoosp ([52dd342](https://github.com/elcoosp/quoin/commit/52dd342c369a2b095847f44abdda4398ab4b886e))
- implement live theme resolvers and text run builders by @elcoosp ([943458b](https://github.com/elcoosp/quoin/commit/943458bb9cce002335f6d6cf27c9be08f225c273))
- add inherent render methods to UCP adapter structs by @elcoosp ([ab43802](https://github.com/elcoosp/quoin/commit/ab43802505f370908ee4b1f2be084d92808962d1))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- integrate input for gpui by @elcoosp ([c51efd5](https://github.com/elcoosp/quoin/commit/c51efd5890688eebbae7a7d6c8d4e04496696049))
- quoin-gpui by @elcoosp ([551147c](https://github.com/elcoosp/quoin/commit/551147cc31be02a113f87314f9aa0dd298ccbc58))
- add UCP traits and GPUI backend by @elcoosp ([27952a9](https://github.com/elcoosp/quoin/commit/27952a9a5ea1d3cb9c4dbaa17e514eb2a4f7e062))

### Miscellaneous
- inject missing metadata and point to root README by @elcoosp ([45a16d5](https://github.com/elcoosp/quoin/commit/45a16d56d41e6e5c9958254292d93155262d8805))
- inject precisely curated keywords for crates.io by @elcoosp ([8722cab](https://github.com/elcoosp/quoin/commit/8722cab4895292fb33f6f96a707b25a9abb9c7c2))
- re-align gpui-component version constraints by @elcoosp ([76fbd57](https://github.com/elcoosp/quoin/commit/76fbd572ab2186251e026cc9c1b5164818b347e1))
- readme by @elcoosp ([722a808](https://github.com/elcoosp/quoin/commit/722a808c618db9dea01f0a95ee80b3b0f0abf91e))
- readme by @elcoosp ([ea40b5d](https://github.com/elcoosp/quoin/commit/ea40b5d3d5062abcebf010f55469c3ba6469c849))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- use edition 2024 by @elcoosp ([c18acfb](https://github.com/elcoosp/quoin/commit/c18acfb6f58778b342241de1db66e8fc0114ca57))

### Refactor
- simplify theme palette and icon mapping by @elcoosp ([051cd5e](https://github.com/elcoosp/quoin/commit/051cd5e615ffa56b1779bc2460dbb2c82d425032))

### Testing
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))

### build
- inherit workspace lints by @elcoosp ([2897f1f](https://github.com/elcoosp/quoin/commit/2897f1fe76409386fcc78fee240c44da48fa29c5))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.1.0
