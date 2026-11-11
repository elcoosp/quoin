# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-11

### Documentation
- add README for quoin-devtools by @elcoosp ([a716aa4](https://github.com/elcoosp/quoin/commit/a716aa4aee0e8eb136f2ea38ea5d076846d2afe7))

### Features
- add typed style IR shared by every backend by @elcoosp ([cb7b126](https://github.com/elcoosp/quoin/commit/cb7b126260be817a61c07c9ad66077260a3c5501))
- implement signal tracing and event logging by @elcoosp ([e9f3a42](https://github.com/elcoosp/quoin/commit/e9f3a4281c61bdd68530ba24fd4f4b2c77b43a9e))
- add Cargo.toml for quoin-devtools crate by @elcoosp ([dd00a68](https://github.com/elcoosp/quoin/commit/dd00a683da7f2aca3913f8f011397d24ec0fcf98))

### Miscellaneous
- zero-overhead gates + bench-devtools rename by @elcoosp ([24b67b1](https://github.com/elcoosp/quoin/commit/24b67b1728ba52bf89498b2d98d7f5156932e9e8))

### Performance
- add event-log benchmarks by @elcoosp ([e92f182](https://github.com/elcoosp/quoin/commit/e92f18209182e49c7b6b00caaa1f3d5d4d4bfd59))

### Refactor
- Send bound on ReactiveContext signals by @elcoosp ([5c0009a](https://github.com/elcoosp/quoin/commit/5c0009ac6ca15b0f57b04a6e6edccdef33149c65))
- move recording helpers into record module by @elcoosp ([2a6be1e](https://github.com/elcoosp/quoin/commit/2a6be1ec4ec5b971bdef64efe52435c056378d8d))
- split lib into event, record, instrumented modules by @elcoosp ([b9a0c8c](https://github.com/elcoosp/quoin/commit/b9a0c8c29f1202db1c301702bd0e9f8873d49ce6))
- split instrumented context wrappers by @elcoosp ([f0ba2a0](https://github.com/elcoosp/quoin/commit/f0ba2a0a656527dc17e14f998b4976926edc7b62))
- split event log and incremental aggregates by @elcoosp ([52f680e](https://github.com/elcoosp/quoin/commit/52f680eb12602f049f1bf57ffc191d657b44380e))

### build
- inherit workspace lints by @elcoosp ([9b8784f](https://github.com/elcoosp/quoin/commit/9b8784f8ab97f1b261b30a406e5e684137348538))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
