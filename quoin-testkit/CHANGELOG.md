# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-11

### Bug Fixes
- fix TokioJoinHandle::detach and implement Executor::sleep by @elcoosp ([e35dc6f](https://github.com/elcoosp/quoin/commit/e35dc6f6724978f1be4dbfb739071c084cd6d505))
- add rt-multi-thread feature to tokio dependency by @elcoosp ([86bf423](https://github.com/elcoosp/quoin/commit/86bf423fdcac9ba7cc1b13203ec8ed5670b94863))

### Documentation
- document the TokioExecutor::new panic cases by @elcoosp ([b54a590](https://github.com/elcoosp/quoin/commit/b54a5900a3c2e25bf6875640307af9a6a99b5f39))
- add README for quoin-testkit by @elcoosp ([f9f0208](https://github.com/elcoosp/quoin/commit/f9f020875cb7822b3792a9becb20e7c155be072b))

### Features
- make TestSignal effect-tracked with a shared cell dependency by @elcoosp ([7e8cedd](https://github.com/elcoosp/quoin/commit/7e8ceddb6892718cd2bc754c861af58534d7bc8d))
- implement deterministic headless testing toolkit by @elcoosp ([d4a095b](https://github.com/elcoosp/quoin/commit/d4a095be3e298c3083954b36ead4a175d20525f5))
- add Cargo.toml for quoin-testkit crate by @elcoosp ([287aec9](https://github.com/elcoosp/quoin/commit/287aec911b481292134762126366c21ebca4aa31))

### Miscellaneous
- mark examples/test-crates as publish = false by @elcoosp ([972b245](https://github.com/elcoosp/quoin/commit/972b245679dc1fd7f49c301fe93ae32406cdb32b))

### Refactor
- Send bound on ReactiveContext signals by @elcoosp ([5c0009a](https://github.com/elcoosp/quoin/commit/5c0009ac6ca15b0f57b04a6e6edccdef33149c65))

### Testing
- expand globals, executor, and debug coverage by @elcoosp ([802bd99](https://github.com/elcoosp/quoin/commit/802bd9942a4395df333103c6409c6cf436f6698d))

### build
- inherit workspace lints by @elcoosp ([5abb2f4](https://github.com/elcoosp/quoin/commit/5abb2f487bcfcdd197b70b1deab146f379199c99))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
