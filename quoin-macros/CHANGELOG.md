# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.4.0] - 2026-11-14

### Bug Fixes
- remove sandbox muda patch and disable default gtk feature by @elcoosp ([255a311](https://github.com/elcoosp/quoin/commit/255a3113040bf3a866abc1754fa37931170c1daf))
- remaining items from refactor by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([ace39b6](https://github.com/elcoosp/quoin/commit/ace39b60392ddb06c7589f43c4fc3734bc50b786))
- remove unsound unsafe Send+Sync impls in Dioxus conformance by @elcoosp in [#1](https://github.com/elcoosp/quoin/pull/1) ([d715e6a](https://github.com/elcoosp/quoin/commit/d715e6a6cff24e7be035bbf84671f428eb20608f))
- readme by @elcoosp ([f3ed105](https://github.com/elcoosp/quoin/commit/f3ed1052386c7540577898876a4e9d1a8a507c7e))
- rm move from listeners by @elcoosp ([bd14c75](https://github.com/elcoosp/quoin/commit/bd14c75e40be7fc1d399c880d47e055baf93a253))
- clippy warns by @elcoosp ([40ab139](https://github.com/elcoosp/quoin/commit/40ab13987c04bcda238e3b4678096858d1d72375))
- effect ui test by @elcoosp ([34bf602](https://github.com/elcoosp/quoin/commit/34bf602b2177cdb93ec75e39d2667794f62ff3de))
- trybuild mactos by @elcoosp ([79386e3](https://github.com/elcoosp/quoin/commit/79386e3f67c38dc488b0063bf7e73a2bdd8c51eb))
- running ucp example with mini devtools by @elcoosp ([091ae95](https://github.com/elcoosp/quoin/commit/091ae955b25651b3e8b772fd8d609f54f7364a36))
- almost working with component devtools by @elcoosp ([2266117](https://github.com/elcoosp/quoin/commit/2266117faacb2eab7707f8d3949b444b915f94ff))
- almost working ucp lib with mini devtools by @elcoosp ([3fec561](https://github.com/elcoosp/quoin/commit/3fec561df983e0c232fad11f716deb1c808e1d37))
- styling input by @elcoosp ([c266bbd](https://github.com/elcoosp/quoin/commit/c266bbdaf8f9333fefb08e901996ab72436f2450))
- near working macro with gap close by @elcoosp ([7746f19](https://github.com/elcoosp/quoin/commit/7746f19aa4ab1f5a4a5c888a9de36b0d2685eec2))
- cond compile macro mods by @elcoosp ([472d33c](https://github.com/elcoosp/quoin/commit/472d33c65467bfaefd049c7bf815ca2d57e3bbf6))
- mini devtools example more gaps by @elcoosp ([6708014](https://github.com/elcoosp/quoin/commit/67080143dc18766ee1f4e515310aa4c599cbdcc0))
- ucp rewowrking by @elcoosp ([1e63ff4](https://github.com/elcoosp/quoin/commit/1e63ff4d9bb4ec7c09ab52fb600460e57814ce91))
- mv ucp-lib to examples by @elcoosp ([08383e9](https://github.com/elcoosp/quoin/commit/08383e97ecd50b1560f35350079f1b7612b87937))
- working ucp-lib leptos by @elcoosp ([73b7684](https://github.com/elcoosp/quoin/commit/73b768405fd1e86bfc9ce3b7118f33bde938c750))
- working ucp-lib by @elcoosp ([9f5490f](https://github.com/elcoosp/quoin/commit/9f5490f0b1ed562beda9b8605844d0ca86356b49))
- font tw class by @elcoosp ([a1111db](https://github.com/elcoosp/quoin/commit/a1111dbe08ab715c1fe856db6e955d19b6902338))
- rm macro debug by @elcoosp ([e3dbd31](https://github.com/elcoosp/quoin/commit/e3dbd3191703fe012fea8fa088573f6aea38a845))
- remove _ctx field, match counter example pattern by @elcoosp ([954ad3d](https://github.com/elcoosp/quoin/commit/954ad3d9a29003e7b62bbffbd1367415a869b9e0))
- wrap on_click closure in cx.listener correctly by @elcoosp ([1aa0aa2](https://github.com/elcoosp/quoin/commit/1aa0aa2c133372af492db0dfe3db9f71b5ef914c))
- correctly distinguish elements from plain expressions by @elcoosp ([11d8daf](https://github.com/elcoosp/quoin/commit/11d8daf3b9f31893f225daf39a535a3134c82f26))
- mostly compile by @elcoosp ([7ee01b6](https://github.com/elcoosp/quoin/commit/7ee01b60bcc357132dde1abdad98bef45417c749))
- running ucp example by @elcoosp ([84d906a](https://github.com/elcoosp/quoin/commit/84d906a20bf4a5a6b5ef4c9a44ba886302b9e41e))
- correct type mismatch and UUID IdentFragment issue by @elcoosp ([f1be45d](https://github.com/elcoosp/quoin/commit/f1be45dd56e1bc73262d5e7c237967ef61ac856b))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([65a7312](https://github.com/elcoosp/quoin/commit/65a731230cf3db2b64de685c2dc0ad1d5ec3d39e))
- note native reactive effect macro support on Xilem by @elcoosp ([b79a92e](https://github.com/elcoosp/quoin/commit/b79a92e93e814c2e13ff34c3a8eca040ffa0bc6b))
- advertise reactive effects and AnySignal in the README by @elcoosp ([e9f2766](https://github.com/elcoosp/quoin/commit/e9f27662d88889aee9c60825bd41e9901c31144c))
- document proc-macro entry points by @elcoosp ([c23e427](https://github.com/elcoosp/quoin/commit/c23e427528034d8eabe189d2a278cb59e951e517))
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
- bridge fine-grained signals into the effect graph ([#29](https://github.com/elcoosp/quoin/pull/29)) by @elcoosp in [#29](https://github.com/elcoosp/quoin/pull/29) ([52d6ec7](https://github.com/elcoosp/quoin/commit/52d6ec7bee3e883bd149d1e1662e85c2a318a998))
- add floem-shadcn feature flag by @elcoosp ([44af53c](https://github.com/elcoosp/quoin/commit/44af53c7d6474e6151ca0a8fd113544ac60ced09))
- expose floem/xilem features + enforce exclusivity by @elcoosp ([0a7dc2c](https://github.com/elcoosp/quoin/commit/0a7dc2c97d922c3a95e9d99d7c03775876f77c73))
- working tests by @elcoosp ([f58114d](https://github.com/elcoosp/quoin/commit/f58114dc22c767674d64386ee1b9019fcaad8f36))
- mv tests & core macro to separate crates by @elcoosp ([18386b7](https://github.com/elcoosp/quoin/commit/18386b7321b30e3ee115d9a87d65c207b29658e2))
- integrate input for gpui by @elcoosp ([c51efd5](https://github.com/elcoosp/quoin/commit/c51efd5890688eebbae7a7d6c8d4e04496696049))
- dioxus ucp working by @elcoosp ([6ce8e74](https://github.com/elcoosp/quoin/commit/6ce8e744e4aeb022b88fee1a2b2d45087ec9838d))
- leptos still not compiling by @elcoosp ([a53cc66](https://github.com/elcoosp/quoin/commit/a53cc66fe6c27289329f3d3ad8ab48616a91e98e))
- almost working leptos upc example by @elcoosp ([975bdf2](https://github.com/elcoosp/quoin/commit/975bdf22b503efc8a53473350dfee57c37f4e05d))
- render_gpui mostly working by @elcoosp ([9699fa1](https://github.com/elcoosp/quoin/commit/9699fa1619959653f29181b7a00982c3052e634a))
- table by @elcoosp ([bf4e93e](https://github.com/elcoosp/quoin/commit/bf4e93ec5d1ee9241ef8e38aee7e856d26d8cc4e))
- add quoin_element! macro for custom element registration by @elcoosp ([09e1f3c](https://github.com/elcoosp/quoin/commit/09e1f3cfb73029156e6ef3d70ef05ccda80c94ea))
- add UCP table codegen for GPUI/Leptos/Dioxus by @elcoosp ([92fbfdd](https://github.com/elcoosp/quoin/commit/92fbfdd95db8e8e7e3fc959c2853c00a718c35f7))
- add Dioxus emitters for component! and quoin_render! by @elcoosp ([ec82339](https://github.com/elcoosp/quoin/commit/ec82339e3c3bb5a1067e10e78cbc2571555a7afd))
- add UCP traits and GPUI backend by @elcoosp ([27952a9](https://github.com/elcoosp/quoin/commit/27952a9a5ea1d3cb9c4dbaa17e514eb2a4f7e062))
- add quoin_render! parser and GPUI emitter with Tailwind transpiler by @elcoosp ([dcc0ebf](https://github.com/elcoosp/quoin/commit/dcc0ebf713be8648e2be00839d5b7826f39d25bb))
- add component! parser and GPUI emitter by @elcoosp ([8e9e68a](https://github.com/elcoosp/quoin/commit/8e9e68aec699c98af53e53306b2a26ede60c646e))

### Miscellaneous
- inject missing metadata and point to root README by @elcoosp ([45a16d5](https://github.com/elcoosp/quoin/commit/45a16d56d41e6e5c9958254292d93155262d8805))
- inject precisely curated keywords for crates.io by @elcoosp ([8722cab](https://github.com/elcoosp/quoin/commit/8722cab4895292fb33f6f96a707b25a9abb9c7c2))
- upgrade syn to v3 and use workspace metadata by @elcoosp ([9562360](https://github.com/elcoosp/quoin/commit/956236018811f0f17cf912bedbbc74d02ad8d3fc))
- readme by @elcoosp ([722a808](https://github.com/elcoosp/quoin/commit/722a808c618db9dea01f0a95ee80b3b0f0abf91e))
- readme by @elcoosp ([ea40b5d](https://github.com/elcoosp/quoin/commit/ea40b5d3d5062abcebf010f55469c3ba6469c849))
- lint by @elcoosp ([50b0162](https://github.com/elcoosp/quoin/commit/50b0162b843f3ee1b506521cfc3d0ae003fae76d))
- lint by @elcoosp ([af64409](https://github.com/elcoosp/quoin/commit/af6440994fab3ad3643b0cfc237fd2c721097ffc))
- fmt & fix conformance by @elcoosp ([ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- use edition 2024 by @elcoosp ([c18acfb](https://github.com/elcoosp/quoin/commit/c18acfb6f58778b342241de1db66e8fc0114ca57))
- up justfile & add tests by @elcoosp ([90427da](https://github.com/elcoosp/quoin/commit/90427daad66f0936aa2087e831697b8bdabec07f))
- scaffold quoin-macros and add read!/action! macros by @elcoosp ([18dcd8c](https://github.com/elcoosp/quoin/commit/18dcd8ccc030dcb7066f825882a40d31d09c5969))
- scaffold quoin-macros and add read!/action! macros by @elcoosp ([3fcba94](https://github.com/elcoosp/quoin/commit/3fcba945faca7a02a96474ddf25290cd16d5d321))

### Testing
- working leptos tests by @elcoosp ([916ccf5](https://github.com/elcoosp/quoin/commit/916ccf55ab2a3bf16ff8aaec4cdb6345df3c60f4))
- leptos macro pass by @elcoosp ([57998e4](https://github.com/elcoosp/quoin/commit/57998e41c6254d8fde67f26e78a8fea9841fbd73))
- tests leptos dioxus by @elcoosp ([ac8c25f](https://github.com/elcoosp/quoin/commit/ac8c25f33778c1bc9f5195a3c04645801bcd9742))
- add trybuild compile-fail test for missing render block by @elcoosp ([62107a8](https://github.com/elcoosp/quoin/commit/62107a89eec309a3885fea715954390f6f26c75a))

### build
- forward gpui-rsx-classes feature by @elcoosp ([578405d](https://github.com/elcoosp/quoin/commit/578405d8259c568777c14ad004f61520df55cef6))
- inherit workspace lints by @elcoosp ([f9d25cd](https://github.com/elcoosp/quoin/commit/f9d25cd621f5c96bcf4c77d0014d9a24dbe9c52c))
**Full Changelog**: https://github.com/elcoosp/quoin/releases/tag/v0.4.0
