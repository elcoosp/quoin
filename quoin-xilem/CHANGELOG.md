# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-14

### Bug Fixes
- remove sandbox muda patch and disable default gtk feature by @elcoosp ([255a311](https://github.com/elcoosp/quoin/commit/255a3113040bf3a866abc1754fa37931170c1daf))
- unify global state, batch notifies, and add LocalExecutor by @elcoosp ([208524b](https://github.com/elcoosp/quoin/commit/208524bf865cee293ece184eed2de5dc6dd149f6))
- factor complex UpdateNotifier type into type alias by @elcoosp ([bca46a6](https://github.com/elcoosp/quoin/commit/bca46a69399ac573faad71665587805d4b30e903))
- quoin macro dioxus by @elcoosp ([9c8940c](https://github.com/elcoosp/quoin/commit/9c8940cb5fb254dbde7bb7845eaf05b61af7ca16))
- fmt by @elcoosp ([52cbd4c](https://github.com/elcoosp/quoin/commit/52cbd4c7c689e966bf808ae53323d28855373f78))
- readme by @elcoosp ([f3ed105](https://github.com/elcoosp/quoin/commit/f3ed1052386c7540577898876a4e9d1a8a507c7e))
- rm move from listeners by @elcoosp ([bd14c75](https://github.com/elcoosp/quoin/commit/bd14c75e40be7fc1d399c880d47e055baf93a253))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
- provide runtime in conformance test harness by @elcoosp ([c0331ea](https://github.com/elcoosp/quoin/commit/c0331ea89dc39b580a81353128cfbcdddb6199e2))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([65a7312](https://github.com/elcoosp/quoin/commit/65a731230cf3db2b64de685c2dc0ad1d5ec3d39e))
- note native reactive effect macro support on Xilem by @elcoosp ([b79a92e](https://github.com/elcoosp/quoin/commit/b79a92e93e814c2e13ff34c3a8eca040ffa0bc6b))
- advertise reactive effects and AnySignal in the README by @elcoosp ([e9f2766](https://github.com/elcoosp/quoin/commit/e9f27662d88889aee9c60825bd41e9901c31144c))
- document devtools demo, benchmarks, and lint policy by @elcoosp ([0fcfa65](https://github.com/elcoosp/quoin/commit/0fcfa65494ebe3760f45d3e574a60b0ba8638382))
- revamp README with Phase 1.5 features and examples by @elcoosp ([acc86c5](https://github.com/elcoosp/quoin/commit/acc86c5028587e31657355ca9fbb6f8f42560276))
- completely revamp README with better structure, TOC, and updated examples by @elcoosp ([48ea1c3](https://github.com/elcoosp/quoin/commit/48ea1c3e5381d895970347e2a776ea0c1ce08bbb))
- add doc code by @elcoosp ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- readme & rm navigator by @elcoosp ([2735176](https://github.com/elcoosp/quoin/commit/27351767f6d81c270d624ba9fc1eeff71ff57200))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))
- readme by @elcoosp ([537b5c8](https://github.com/elcoosp/quoin/commit/537b5c845c64400de3ba2905eb11b38346bda4c5))
- add spec by @elcoosp ([13ba55c](https://github.com/elcoosp/quoin/commit/13ba55c747915030959f70b9bee2fecf81f5e0dd))

### Features
- keys { .. } goes native on the web backends ([#35](https://github.com/elcoosp/quoin/pull/35)) by @elcoosp in [#35](https://github.com/elcoosp/quoin/pull/35) ([62a0719](https://github.com/elcoosp/quoin/commit/62a07192d10c221951def4ee2b2effc937cc15ad))
- expose new effect module under library roots by @elcoosp ([42d6111](https://github.com/elcoosp/quoin/commit/42d61116b2298e1fba17e354e78d61d450925c56))
- add component-scoped effects and thread-safe cleanup slots by @elcoosp ([69c028e](https://github.com/elcoosp/quoin/commit/69c028eb860a57ab203e19ff05d18c81615c633e))
- expose the context runtime for run_app! by @elcoosp ([a6c6fbb](https://github.com/elcoosp/quoin/commit/a6c6fbbff66546507c6632b2f5a52f4ad9da5798))
- rebuild pump + theme + allocation gate by @elcoosp ([312a533](https://github.com/elcoosp/quoin/commit/312a533394c99d04ab8b1535bb859e7c7806294f))
- implement proper JoinHandle abort/detach and fix request_update recursion by @elcoosp ([f21c3d0](https://github.com/elcoosp/quoin/commit/f21c3d0b5a0dc6e92d095be9c3b9dbba274aaf14))
- fix render_gpui, align editions by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([68a67e4](https://github.com/elcoosp/quoin/commit/68a67e42546b1cc223ebeefce767f13f3fb8c760))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- working xilem example by @elcoosp ([baf98d0](https://github.com/elcoosp/quoin/commit/baf98d0aa8448fbc6dd3ec924a0e8546a61c55bd))
- mutable signal by @elcoosp ([2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))
- complete ReactiveContext implementation by @elcoosp ([0fc2180](https://github.com/elcoosp/quoin/commit/0fc218073da17f60de1fd4df8e8437521ded4675))

### Miscellaneous
- inject missing metadata and point to root README by @elcoosp ([45a16d5](https://github.com/elcoosp/quoin/commit/45a16d56d41e6e5c9958254292d93155262d8805))
- inject precisely curated keywords for crates.io by @elcoosp ([8722cab](https://github.com/elcoosp/quoin/commit/8722cab4895292fb33f6f96a707b25a9abb9c7c2))
- use workspace metadata by @elcoosp ([616b9c6](https://github.com/elcoosp/quoin/commit/616b9c6d908855292ceb34b87a00b2fc6c330576))
- readme by @elcoosp ([722a808](https://github.com/elcoosp/quoin/commit/722a808c618db9dea01f0a95ee80b3b0f0abf91e))
- readme by @elcoosp ([ea40b5d](https://github.com/elcoosp/quoin/commit/ea40b5d3d5062abcebf010f55469c3ba6469c849))
- more doc + ci by @elcoosp ([f55a2d6](https://github.com/elcoosp/quoin/commit/f55a2d6a794ef3d5b5dfca73a09c3199b4c6c105))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- add quoin-xilem and quoin-floem skeletons to workspace by @elcoosp ([0c969a1](https://github.com/elcoosp/quoin/commit/0c969a1fb3280344a8878ec13a990252a37ecd05))

### Performance
- introduce quoin-xilem effect path micro-benchmarks by @elcoosp ([cd8f7d0](https://github.com/elcoosp/quoin/commit/cd8f7d02c77d5501e08508dc8def067d3cce2ce4))

### Refactor
- remove SleepExt implementation from test harness by @elcoosp ([77714be](https://github.com/elcoosp/quoin/commit/77714beb07656745291bc05454dbf84302d25d36))
- reuse shared globals, notifier signal, and join handle by @elcoosp ([df4f8b8](https://github.com/elcoosp/quoin/commit/df4f8b818bd59db71d2a9a47c5589e589a7dbc77))

### Testing
- wire Xilem signals into portable effect conformance suite by @elcoosp ([3a04bf4](https://github.com/elcoosp/quoin/commit/3a04bf44a903f28bc2e663fad3f59332d4a309f1))
- serialize the pump tests and cover repeated-notify coalescing by @elcoosp ([f8b3fba](https://github.com/elcoosp/quoin/commit/f8b3fbaf77d83def283f8123e3c717f00acc86f1))
- update conformance harness by @elcoosp ([ac06a38](https://github.com/elcoosp/quoin/commit/ac06a384aa40d9b92efb4ee54eb1d2e08b592375))
- implement LocalExecutor for conformance harness by @elcoosp ([fa89fce](https://github.com/elcoosp/quoin/commit/fa89fce97074aa91198bb1371a2639617e394891))
- add more conformance & xilem by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([f2a04ed](https://github.com/elcoosp/quoin/commit/f2a04ed4a29c103cb2af215a0a5b08cf6e34998d))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))
- add conformance tests for xilem and floem adapters by @elcoosp ([db0aca8](https://github.com/elcoosp/quoin/commit/db0aca86f393d77396c61428f07fe5086cdf74fd))

### build
- inherit workspace lints by @elcoosp ([409e354](https://github.com/elcoosp/quoin/commit/409e3542cdf238befea9dcaefd039978df672234))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
