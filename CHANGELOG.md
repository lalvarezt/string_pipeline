# Changelog

All notable changes to this project will be documented in this file.
<!-- ignore lint rules that are often triggered by content generated from commits / git-cliff -->
<!-- markdownlint-disable line-length no-bare-urls ul-style emphasis-style -->
## [0.9.0](https://github.com/lalvarezt/string_pipeline/compare/0.8.1..0.9.0) - 2025-06-06

### 🚜 Refactor

- *(parser)* [**breaking**] Merge strip and trim functionality, now trim allows for custom charset - ([cbcfd24](https://github.com/lalvarezt/string_pipeline/commit/cbcfd24da56091eff4f5960b44927ecea4ff3f08))

### 📚 Documentation

- *(readme)* Improved documentation, edge cases, examples - ([d1bc63f](https://github.com/lalvarezt/string_pipeline/commit/d1bc63fb4f28d8f74adb4f7d0218fe92cae3a7ff))
- *(readme)* Updated readme and split template system to a separate file - ([e9e7af1](https://github.com/lalvarezt/string_pipeline/commit/e9e7af1374faa52db788f883717acc225d7e505e))
- *(template)* Improved documentation, edge cases, examples - ([69e5d59](https://github.com/lalvarezt/string_pipeline/commit/69e5d59ab8b987351049137753041abc59e99524))

### ⚡ Performance

- *(parser)* Minor performance changes, preallocations and early exits - ([4e1722e](https://github.com/lalvarezt/string_pipeline/commit/4e1722edece1596aa40dfe78590911c50cbf72c6))

### ⚙️ Miscellaneous Tasks

- Bump to 0.9.0 - ([eb89a99](https://github.com/lalvarezt/string_pipeline/commit/eb89a99667b7b7a47e263b8211000b1dbb80e74f))


## [0.8.1](https://github.com/lalvarezt/string_pipeline/compare/0.8.0..0.8.1) - 2025-06-05

### 🐛 Bug Fixes

- *(pest)* Remove invalid operations from map - ([f6deb58](https://github.com/lalvarezt/string_pipeline/commit/f6deb58ad3733f1499d7a08a66cc9718e29ea7c1))
- *(pest)* Issue with closing braces in nested map extract_regex - ([59ba12d](https://github.com/lalvarezt/string_pipeline/commit/59ba12d87b8aa037c319cdf874c69b3a4490848c))

### 🚜 Refactor

- *(parser)* Shuffle some things around to make the code more expressive and hopefully maintainable - ([1e52190](https://github.com/lalvarezt/string_pipeline/commit/1e521908f824a624d0f6f20e845df11bfdc07699))

### ⚡ Performance

- *(pest)* Simplify grammar, remove unnecessary lookaheads - ([521a006](https://github.com/lalvarezt/string_pipeline/commit/521a0060886fd89e0462931dea5f8e6b7f25678d))

### 🧪 Testing

- *(pipeline)* Add new tests for map functionality - ([d685f63](https://github.com/lalvarezt/string_pipeline/commit/d685f63c251acfccab6c63288b11155a2816ca48))

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog - ([743b75e](https://github.com/lalvarezt/string_pipeline/commit/743b75edba85e9cee4ed4fbc03505ba377c67c87))
- Bump to 0.8.1 - ([a49cd64](https://github.com/lalvarezt/string_pipeline/commit/a49cd64990b750d55629d107a7f43075e2b2e37d))


## [0.8.0](https://github.com/lalvarezt/string_pipeline/compare/0.7.0..0.8.0) - 2025-06-04

### ⛰️  Features

- *(bench)* Update to use map - ([d9ec045](https://github.com/lalvarezt/string_pipeline/commit/d9ec04514fb68b6b3f2e8f99f2706013b9b878a5))
- *(parser)* Make map work for multiple operations - ([4d1b9e7](https://github.com/lalvarezt/string_pipeline/commit/4d1b9e78b8e7a304523dfc257b120ff6a80586e4))
- *(parser)* Add support for map, sort, reverse, unique, pad, regex_extract - ([b198d20](https://github.com/lalvarezt/string_pipeline/commit/b198d20fa80decf4d5cab4f54c0d8f476a5092bc))

### 🚜 Refactor

- *(lib)* [**breaking**] Create and expose a single Template object for users to interact with  - ([378fdc4](https://github.com/lalvarezt/string_pipeline/commit/378fdc4fb6a34aea3a8638800376703685d2a731))
- *(mod)* [**breaking**] Remove string|list handling on operations, we now have map - ([61d5a9d](https://github.com/lalvarezt/string_pipeline/commit/61d5a9df82f641ad7185827cfcac29181a4ce9cc))
- *(mod)* Clean up logic - ([a8b79bf](https://github.com/lalvarezt/string_pipeline/commit/a8b79bf7d2c16ed85bea8bff55ccb37342ba9fa1))

### 📚 Documentation

- *(readme)* Updated readme with the new features - ([2278fa8](https://github.com/lalvarezt/string_pipeline/commit/2278fa8f16667b267d0674977c1245ae4ec54ddc))
- *(readme)* Update slice and substring and other minor changes - ([9df6365](https://github.com/lalvarezt/string_pipeline/commit/9df6365ccb170f5f81b9b4168d02ca1b94892fcf))

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog - ([7ded77f](https://github.com/lalvarezt/string_pipeline/commit/7ded77f2be3f0b1e1fe30d9de218938b9e02aaa2))
- Bump to 0.8.0 - ([8f54253](https://github.com/lalvarezt/string_pipeline/commit/8f5425379ed3c12de551b7cd2e79eea352400e85))


## [0.7.0](https://github.com/lalvarezt/string_pipeline/compare/0.6.0..0.7.0) - 2025-06-03

### ⛰️  Features

- *(parser)* [**breaking**] Rework handling of espaced characters in arguments, now it behaves as expected - ([6291c4f](https://github.com/lalvarezt/string_pipeline/commit/6291c4f1d12bf8b7471b51837ac222c1b749f367))
- *(parser)* Add filter and filter_not operations - ([f0c9f1a](https://github.com/lalvarezt/string_pipeline/commit/f0c9f1aadd25368419dcb1c1aa17b329cdf1746f))
- *(parser)* Add strip_ansi operation - ([505f582](https://github.com/lalvarezt/string_pipeline/commit/505f582bdbc9af12a3f75c680a972d0cdc227cb3))
- *(parser)* Add support for imsx sed flags - ([bb16ba5](https://github.com/lalvarezt/string_pipeline/commit/bb16ba52f74f4b0a43e4d9cde4437436f904d8bd))

### 📚 Documentation

- *(parser)* Add strip ansi escape sequences documentation - ([d5343dd](https://github.com/lalvarezt/string_pipeline/commit/d5343ddf4fdabcaf0ab5a7503a1ec1fd4fa01e95))
- *(readme)* Updated readme - ([7f83031](https://github.com/lalvarezt/string_pipeline/commit/7f8303113174204f0b41d427274194edf92c1a29))

### ⚡ Performance

- Add a simple benchmark for `string_pipeline::process`  - ([93a19eb](https://github.com/lalvarezt/string_pipeline/commit/93a19eb80fa38752f4868379a47909b546c0afc5))

### 🧪 Testing

- *(parser)* New set of test, hopefully better organized to allow growth - ([6532b2c](https://github.com/lalvarezt/string_pipeline/commit/6532b2ce48c284bfc718c20a7c914f876e76bb25))

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog - ([71798ef](https://github.com/lalvarezt/string_pipeline/commit/71798ef516c528295821c83f25f3e838a04ee50a))
- Bump to 0.7.0 - ([d84ee41](https://github.com/lalvarezt/string_pipeline/commit/d84ee41279a9f778522d79553779ee355e7bcb62))

## New Contributors ❤️

* @alexpasmantier made their first contribution in [#3](https://github.com/lalvarezt/string_pipeline/pull/3)

## [0.6.0](https://github.com/lalvarezt/string_pipeline/compare/0.5.0..0.6.0) - 2025-06-02

### ⛰️  Features

- *(parser)* Add debug operator {!...} - ([8a940cf](https://github.com/lalvarezt/string_pipeline/commit/8a940cf79e30d1804d5cbf877c810bcfc1e4b1cb))
- *(parser)* Make split consistent with the other operations and don't flatten - ([9372c87](https://github.com/lalvarezt/string_pipeline/commit/9372c87448b05f4e1d0b062ab3f3e13a9e694bef))

### 📚 Documentation

- *(readme)* Revamp documentation - ([73ce4f0](https://github.com/lalvarezt/string_pipeline/commit/73ce4f0673b798e82e4d1ab227f98c580cd569af))

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog - ([1da1f0b](https://github.com/lalvarezt/string_pipeline/commit/1da1f0bb804607e68c465082e96857b8f746c77d))
- Bump to 0.6.0 - ([2454007](https://github.com/lalvarezt/string_pipeline/commit/2454007ec7f35755e8edd101d410a50f5b990fbc))


## [0.5.0](https://github.com/lalvarezt/string_pipeline/compare/0.4.0..0.5.0) - 2025-06-02

### ⛰️  Features

- *(parser)* [**breaking**] Simplify replace by using regex as default - ([4b6093a](https://github.com/lalvarezt/string_pipeline/commit/4b6093ae20a5f0d50aa0ba22b23624e0b32a5373))

### 🐛 Bug Fixes

- *(pest)* Fix error in the parser that trimmed whitespaces on the boundaries - ([829e273](https://github.com/lalvarezt/string_pipeline/commit/829e273cd761d2b577242b4296080771072679f9))

### 📚 Documentation

- Configure markdown styling - ([d97ee6e](https://github.com/lalvarezt/string_pipeline/commit/d97ee6eda6cefa771e935e888840d47ff147f600))

### 🎨 Styling

- *(pest)* Reformat with pest formatter - ([af3e95e](https://github.com/lalvarezt/string_pipeline/commit/af3e95e50d5faf1a3c2168d773bbf50e9eebbe7d))

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog - ([333585d](https://github.com/lalvarezt/string_pipeline/commit/333585d5b42976e4714f8a7dde165f16b5d3db66))
- Bump to 0.5.0 - ([b922d22](https://github.com/lalvarezt/string_pipeline/commit/b922d225608a5e8b511855906fb4ea4f63dad1ee))

### Build

- QoL workflow - ([13b82b1](https://github.com/lalvarezt/string_pipeline/commit/13b82b1487dc939356b409c5eeac3a5b306e3bcc))


## [0.4.0](https://github.com/lalvarezt/string_pipeline/compare/0.3.1..0.4.0) - 2025-06-02

### ⛰️  Features

- *(changelog)* Add git-cliff - ([d8a3b99](https://github.com/lalvarezt/string_pipeline/commit/d8a3b9971970ca4e88dd19845e24406397f838e0))
- *(parser)* Add shorthand support {n..m} - ([48a8d4e](https://github.com/lalvarezt/string_pipeline/commit/48a8d4e0d459a7ffe3183a3c07846ee7d5602e48))
- *(parser)* [**breaking**] Migrate codebase to pest parser - ([fcf0731](https://github.com/lalvarezt/string_pipeline/commit/fcf0731e6f525e9c7d03f53ed6b8ed08dc09b830))

### 📚 Documentation

- *(readme)* Added crate information - ([47ea4a5](https://github.com/lalvarezt/string_pipeline/commit/47ea4a5cf7648e0c45058c37b2312995ee0eb125))
- Added crate information - ([de2ef28](https://github.com/lalvarezt/string_pipeline/commit/de2ef28a44c3bd4dcaa2676767e72872d93e9710))

### 🧪 Testing

- *(parser)* Some refactoring and adding more tests - ([bad7d62](https://github.com/lalvarezt/string_pipeline/commit/bad7d629c8a888a9874c34bc00546ebc75c23114))

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog - ([a61bd74](https://github.com/lalvarezt/string_pipeline/commit/a61bd741066d31ebfb6bec4aa43109609af29558))
- Bump to 0.4.0 - ([351f1b1](https://github.com/lalvarezt/string_pipeline/commit/351f1b1af22300b14009ea21f03ad19b2fd1a124))

## New Contributors ❤️

* @lalvarezt-bw made their first contribution

## [0.3.1](https://github.com/lalvarezt/string_pipeline/compare/0.3.0..0.3.1) - 2025-06-01

### Build

- Add release workflow - ([4385840](https://github.com/lalvarezt/string_pipeline/commit/4385840ea1e0ebe92ff56b7cfbafe39992937d4b))


## [0.3.0](https://github.com/lalvarezt/string_pipeline/compare/0.2.0..0.3.0) - 2025-06-01

### 🚜 Refactor

- Split into library and cli - ([dd647eb](https://github.com/lalvarezt/string_pipeline/commit/dd647eb9a3590b7dbdf9d42c39e843e65545a39b))


## [0.2.0] - 2025-05-31

### ⚙️ Miscellaneous Tasks

- Bump version - ([2e58735](https://github.com/lalvarezt/string_pipeline/commit/2e58735f9a2153702b1575edb05078c4c6293011))

## New Contributors ❤️

* @lalvarezt made their first contribution

<!-- generated by git-cliff -->
