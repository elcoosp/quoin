# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-11

### Bug Fixes
- unify global state, batch notifies, and add LocalExecutor by @elcoosp ([208524b](https://github.com/elcoosp/quoin/commit/208524bf865cee293ece184eed2de5dc6dd149f6))
- factor complex UpdateNotifier type into type alias by @elcoosp ([bca46a6](https://github.com/elcoosp/quoin/commit/bca46a69399ac573faad71665587805d4b30e903))
- quoin macro dioxus by @elcoosp ([9c8940c](https://github.com/elcoosp/quoin/commit/9c8940cb5fb254dbde7bb7845eaf05b61af7ca16))
- fmt by @elcoosp ([52cbd4c](https://github.com/elcoosp/quoin/commit/52cbd4c7c689e966bf808ae53323d28855373f78))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
- provide runtime in conformance test harness by @elcoosp ([c0331ea](https://github.com/elcoosp/quoin/commit/c0331ea89dc39b580a81353128cfbcdddb6199e2))

### Documentation
- add doc code by @elcoosp ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))

### Features
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
- use workspace metadata by @elcoosp ([616b9c6](https://github.com/elcoosp/quoin/commit/616b9c6d908855292ceb34b87a00b2fc6c330576))
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
