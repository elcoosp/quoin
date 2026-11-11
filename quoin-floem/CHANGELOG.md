# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-11

### Bug Fixes
- multi-level ../ path deps → git by @elcoosp ([ce320dd](https://github.com/elcoosp/quoin/commit/ce320dd36eac61cfbb76cca76b6f4ea52c4b1196))
- floem-shadcn from elcoosp/floem-shadcn git, not local path by @elcoosp ([6192165](https://github.com/elcoosp/quoin/commit/6192165587e9d9603a506e06f2999625ae7facc6))
- github dep by @elcoosp ([4855a30](https://github.com/elcoosp/quoin/commit/4855a30a17a765f6f18422279cdb72a6ff43db3c))
- un-vendor floem-shadcn; unified bare-git floem source by @elcoosp ([4726daa](https://github.com/elcoosp/quoin/commit/4726daa8d487d8e4fd3e1dd506df9342cc2386f2))
- unify global state semantics and use worker pool by @elcoosp ([f2d2e48](https://github.com/elcoosp/quoin/commit/f2d2e4816abe502ad94b201d0032cfb32de2e35d))
- quoin macro dioxus by @elcoosp ([9c8940c](https://github.com/elcoosp/quoin/commit/9c8940cb5fb254dbde7bb7845eaf05b61af7ca16))
- fmt by @elcoosp ([52cbd4c](https://github.com/elcoosp/quoin/commit/52cbd4c7c689e966bf808ae53323d28855373f78))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))

### Documentation
- doc by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([035f1ea](https://github.com/elcoosp/quoin/commit/035f1ead0958d7bcbe009f9b13355d3dd94a8664))
- add doc code by @elcoosp ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- readme by @elcoosp ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))

### Features
- bridge fine-grained signals into the effect graph ([#29](https://github.com/elcoosp/quoin/pull/29)) by @elcoosp in [#29](https://github.com/elcoosp/quoin/pull/29) ([52d6ec7](https://github.com/elcoosp/quoin/commit/52d6ec7bee3e883bd149d1e1662e85c2a318a998))
- add shadcn feature and floem-shadcn dependency by @elcoosp ([98e4e40](https://github.com/elcoosp/quoin/commit/98e4e4094bbff55d86f4f67d89c10049a55275e2))
- export the effect and keys modules by @elcoosp ([7b7401c](https://github.com/elcoosp/quoin/commit/7b7401c783fa2b663252407fae33913197add951))
- add the keystroke matcher for the keys lowering by @elcoosp ([9b27720](https://github.com/elcoosp/quoin/commit/9b277208e6db36d1bd1b2548dfe9337baf5d4db7))
- add the effect cleanup slot for the effect! lowering by @elcoosp ([a4b46fb](https://github.com/elcoosp/quoin/commit/a4b46fbb7e844034f5736bf8ae2d6395f2a5952c))
- Mutex-backed signals + lock-free theme by @elcoosp ([9de367b](https://github.com/elcoosp/quoin/commit/9de367b4e1ffca9ed36826f1b8bb67d20168301b))
- implement LocalExecutor for FloemExecutor by @elcoosp ([171905c](https://github.com/elcoosp/quoin/commit/171905c8783fc6abc2088b59b5bbcae30f12ff8d))
- implement proper JoinHandle abort/detach with Abortable by @elcoosp ([215f985](https://github.com/elcoosp/quoin/commit/215f985f5b8c8f231e7c4d816c3165effc16f462))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- mutable signal by @elcoosp ([2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))
- complete ReactiveContext implementation by @elcoosp ([39a8659](https://github.com/elcoosp/quoin/commit/39a865931a0b9ec93bdcfad3bb414a3cfb5bc311))

### Miscellaneous
- fmt by @elcoosp ([cdec62c](https://github.com/elcoosp/quoin/commit/cdec62ca16ea8d7e20c820c0161a383e184a3f4c))
- add the dhat and quoin-macros dev dependencies by @elcoosp ([88dfe41](https://github.com/elcoosp/quoin/commit/88dfe416d92d07a888fbde8c81a26b95c47bcc31))
- use workspace metadata by @elcoosp ([1fe229b](https://github.com/elcoosp/quoin/commit/1fe229baf1ab9fe05384986108c260820fb058b5))
- more doc + ci by @elcoosp ([f55a2d6](https://github.com/elcoosp/quoin/commit/f55a2d6a794ef3d5b5dfca73a09c3199b4c6c105))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- add quoin-xilem and quoin-floem skeletons to workspace by @elcoosp ([0c969a1](https://github.com/elcoosp/quoin/commit/0c969a1fb3280344a8878ec13a990252a37ecd05))

### Refactor
- remove SleepExt implementation from test harness by @elcoosp ([b543f59](https://github.com/elcoosp/quoin/commit/b543f59f273c10d2f0731478f7c2cf0b2e2f3189))
- reuse shared abort join handle by @elcoosp ([5e6edb8](https://github.com/elcoosp/quoin/commit/5e6edb87966e2d7bc297b3fc768b172a1f555486))

### Testing
- canonical Tier-1+Tier-2 expansion, both cfg branches by @elcoosp ([16627de](https://github.com/elcoosp/quoin/commit/16627de767c745d8bb7270f3152c2092419c4b3a))
- cover effect cleanup re-run and scope-disposal semantics by @elcoosp ([13a8305](https://github.com/elcoosp/quoin/commit/13a8305231208ce4c3aa6a0a54d786717d6b8353))
- add the allocation gate with the minimum-over-runs methodology by @elcoosp ([9a5379d](https://github.com/elcoosp/quoin/commit/9a5379d9e6658c4f9de28be9ca02cbc70cca78ab))
- prune unused duration imports from conformance suites by @elcoosp ([3a2c8e9](https://github.com/elcoosp/quoin/commit/3a2c8e971d0e5718cbdc3791565736a5455ad400))
- update conformance harness by @elcoosp ([d48d663](https://github.com/elcoosp/quoin/commit/d48d663e037141b20ea9c24b7e48083fcd01db5d))
- implement LocalExecutor for conformance harness by @elcoosp ([e91ac6f](https://github.com/elcoosp/quoin/commit/e91ac6fc4040cc81c978dbaa1cc7ca16b2e167c3))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))
- add conformance tests for xilem and floem adapters by @elcoosp ([db0aca8](https://github.com/elcoosp/quoin/commit/db0aca86f393d77396c61428f07fe5086cdf74fd))

### build
- point floem-shadcn dependency to local sibling path by @elcoosp ([ace53c2](https://github.com/elcoosp/quoin/commit/ace53c2d0c4fc46fa6648f68601bac460a5e9f10))
- inherit workspace lints by @elcoosp ([9443130](https://github.com/elcoosp/quoin/commit/944313041ee775184734701055b6fff2d368cc8b))

### style
- normalize rustfmt formatting across the workspace by @elcoosp ([06f03bc](https://github.com/elcoosp/quoin/commit/06f03bcaff88f970fa6448cfa50887be88108053))
- sort quoin imports in shadcn_expansion test by @elcoosp ([2b3ff1f](https://github.com/elcoosp/quoin/commit/2b3ff1f98770c3c4a10ac60aae1fc3b19f34e7ef))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
