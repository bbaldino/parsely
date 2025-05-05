# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/bbaldino/parsely/compare/parsely-rs-v0.1.4...parsely-rs-v0.1.5) - 2025-05-05

### Added

- update bits-io version

## [0.1.4](https://github.com/bbaldino/parsely/compare/parsely-rs-v0.1.3...parsely-rs-v0.1.4) - 2025-05-02

### Fixed

- optimize collection reads when we know the number of elements

### Other

- update bits-io version

## [0.1.3](https://github.com/bbaldino/parsely/compare/parsely-rs-v0.1.2...parsely-rs-v0.1.3) - 2025-05-01

### Added

- trait rework ([#9](https://github.com/bbaldino/parsely/pull/9))
- upgrade bits-io, re-export BitSliceUxExts

### Other

- rename test file/add a test for 'while'

## [0.1.2](https://github.com/bbaldino/parsely/compare/parsely-rs-v0.1.1...parsely-rs-v0.1.2) - 2025-04-25

### Other

- change ParselyRead trait to match ParselyWrite

## [0.1.1](https://github.com/bbaldino/parsely/compare/parsely-rs-v0.1.0...parsely-rs-v0.1.1) - 2025-04-25

### Other

- Better expression ergonomics ([#5](https://github.com/bbaldino/parsely/pull/5))
- clean up field reader code gen
- add StateSync impl for String
- remove the 'after' attribute
- remove the buffer_type attribute
- release v0.1.0

## [0.1.0](https://github.com/bbaldino/parsely/releases/tag/parsely-rs-v0.1.0) - 2025-04-23

### Added

- support adding/consuming padding on the struct and field levels

### Fixed

- typo
- add bits-io version
- pass reference to assertion function
- add some missing 'pub use' types
- allow bypassing the need for a 'when' attribute on optional in certain cases
- fix incorrect error message when validating fields with a 'fixed' attribute

### Other

- tweak name, add github actions
- clean up the way alignment handling is done (still could use some more)
- update to use new bits-io types
- change sync to a trait and call it on (almost) all types
- add support for 'while' attribute on collections
- remove custom reader/writer support
- add support for post read/write hooks
- support setting a custom buffer type
- tweak sync syntax/implementation
- first pass at implementing dependent fields
- rename Assertion -> FuncOrClosure
- rename RequiredContext -> TypedFnArgList
- test/readme updates
- tests, readme work, writer map logic
- further separate read and write data types, impl some more funcitonality when deriving writes
- rename ParselyData -> ParselyReadData
- refactor code_gen code, add start of write derive impl
- update bitcursor version
- refactor read code generation, define custom Assertion type
- add support for simple 'map' attribute use cases
- update to rust 2024
- change 'assign_from' code gen to be consistent with other paths
- add support for 'assign_from' for fields
- change 'fixed' to 'assertion'
- add support for collection types/'count' attribute
- add support for optional fields/when attributes
- use with_context instead of context
- add support for passing context down to member field
- update ui tests
- refactor the way we grab context assignments and typeS
- implement required_context
- get rid of parsely byteorder attr
- work on getting 'fixed' attribute working
- initial commit
