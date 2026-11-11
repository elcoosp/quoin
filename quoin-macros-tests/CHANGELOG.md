# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.0.0] - 2026-11-11

### Bug Fixes
- multi-level ../ path deps → git by @elcoosp ([ce320dd](https://github.com/elcoosp/quoin/commit/ce320dd36eac61cfbb76cca76b6f4ea52c4b1196))
- update rust crate gpui-component to 0.9.0 by @renovate[bot] ([50a947a](https://github.com/elcoosp/quoin/commit/50a947a4e4c1d755fa04c1b2452efd48529e768a))
- update rust crate gpui to v0.3.12 by @renovate[bot] in [#30](https://github.com/elcoosp/quoin/pull/30) ([213b69d](https://github.com/elcoosp/quoin/commit/213b69da6dd809d51872d62a5feabc674fea045a))
- wire the new backends + GPUI parity fixes by @elcoosp ([d616cc7](https://github.com/elcoosp/quoin/commit/d616cc7ad16a2b9cc9c016c5c4a2f5c7531bb3e8))
- remaining items from refactor by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([ace39b6](https://github.com/elcoosp/quoin/commit/ace39b60392ddb06c7589f43c4fc3734bc50b786))
- quoin macro dioxus by @elcoosp ([9c8940c](https://github.com/elcoosp/quoin/commit/9c8940cb5fb254dbde7bb7845eaf05b61af7ca16))
- opt deps & master gpui-component by @elcoosp ([f15e101](https://github.com/elcoosp/quoin/commit/f15e101dfc9d005eaa87fcd65695555e40acd7bf))
- justfile by @elcoosp ([4291ae4](https://github.com/elcoosp/quoin/commit/4291ae486d28c0ec7744a1eb83309335829c938b))
- passing tests rectified deps by @elcoosp ([911e4dd](https://github.com/elcoosp/quoin/commit/911e4dd3be95703aeed00e1c2c0e6e536373d1ef))
- dioxus macro tests trybuild by @elcoosp ([9c83111](https://github.com/elcoosp/quoin/commit/9c83111c27756ba8c9c53836c17e017370126daa))
- tests by @elcoosp ([a447b17](https://github.com/elcoosp/quoin/commit/a447b17d7230d37d27e22384c5cafd953d347781))

### Documentation
- document trybuild fixture crate by @elcoosp ([0e93bc6](https://github.com/elcoosp/quoin/commit/0e93bc64508ae8cc0272d4c7f8dea6225e9834a0))
- document feature capture build script by @elcoosp ([673cc00](https://github.com/elcoosp/quoin/commit/673cc007b0b7de946c274adb6bd3a2945c8479ab))

### Features
- keys { .. } goes native on the web backends ([#35](https://github.com/elcoosp/quoin/pull/35)) by @elcoosp in [#35](https://github.com/elcoosp/quoin/pull/35) ([62a0719](https://github.com/elcoosp/quoin/commit/62a07192d10c221951def4ee2b2effc937cc15ad))
- discover xkbcommon and propagate it to trybuild by @elcoosp ([1ba3fe6](https://github.com/elcoosp/quoin/commit/1ba3fe6ff542f15a4b02ff1274a05c950026e57d))
- fix render_gpui, align editions by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([68a67e4](https://github.com/elcoosp/quoin/commit/68a67e42546b1cc223ebeefce767f13f3fb8c760))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- mv tests & core macro to separate crates by @elcoosp ([18386b7](https://github.com/elcoosp/quoin/commit/18386b7321b30e3ee115d9a87d65c207b29658e2))

### Miscellaneous
- wire quoin-style, muda patch, xkb config, gpui-pre by @elcoosp ([b5fadfb](https://github.com/elcoosp/quoin/commit/b5fadfb3e0881a4804806b804fb15c2217bcb4cd))
- adapt syntax processing engines to syn 3.0 specs by @elcoosp ([26ec4ba](https://github.com/elcoosp/quoin/commit/26ec4ba7f16a182519cbaaaa735e072c0c53f36f))
- re-align gpui-component version constraints by @elcoosp ([b345649](https://github.com/elcoosp/quoin/commit/b345649a15a534ec5d6152a54b822f8dd98f1f01))
- prevent publishing of quoin-macros-tests by @elcoosp ([72683bc](https://github.com/elcoosp/quoin/commit/72683bc3bf509c68e6ec6aee8aee0f5699fc4cf8))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))

### Testing
- GPUI shadcn shim expansion by @elcoosp ([c9c8f4d](https://github.com/elcoosp/quoin/commit/c9c8f4d76472c4f98310547cd4bbd91aceab0f59))
- refresh unknown-element snapshot for the accordion family by @elcoosp ([e7668a9](https://github.com/elcoosp/quoin/commit/e7668a9c51312713e8434bd76cea2d50c698db79))
- refresh unknown-element snapshot for Tier-1 elements by @elcoosp ([8c1b822](https://github.com/elcoosp/quoin/commit/8c1b8227c841260b96eb58de745e48754adc6dce))
- canonical variant/size expansion compile test by @elcoosp ([77173bc](https://github.com/elcoosp/quoin/commit/77173bc9ccf64550cc0c1e070f1498a4b195f304))
- record svg in the unknown element error list by @elcoosp ([e85a38d](https://github.com/elcoosp/quoin/commit/e85a38db89c20f832f6f773f27747367dcb5f7a2))
- drop the explicit data_table id from the UI pass case by @elcoosp ([20bf786](https://github.com/elcoosp/quoin/commit/20bf786fb1643c86353bd49435244b3b33c2c1c6))
- rely on cargo feature propagation for trybuild by @elcoosp ([fc43bdf](https://github.com/elcoosp/quoin/commit/fc43bdf4487f12a788f6aaea10c1880cc616c417))
- add more tests by @elcoosp ([e9ebc6f](https://github.com/elcoosp/quoin/commit/e9ebc6f91b5a50e8944adb3139d276e3ee8e2ec3))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))

### build
- add gpui-rsx-classes feature by @elcoosp ([59650d7](https://github.com/elcoosp/quoin/commit/59650d7eed2a1774e2dabd99e3f97ea1a8f228ed))
- inherit workspace lints by @elcoosp ([784c1cc](https://github.com/elcoosp/quoin/commit/784c1cca0da9bfe6832d7d806fad5f65bcc82bd9))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.0.0
