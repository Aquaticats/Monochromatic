# test-fixture-toml-edit

Reusable TOML fixture data for `@monochromatic-dev/toml-edit-core` tests.

The `src/valid/` directory contains TOML inputs expected to parse and round-trip
byte-for-byte in splice mode with the current `toml-edit` parser stack.
 Some
files use the `toml10-invalid-toml11-valid-` prefix because they preserve
upstream fixture IDs for syntax that TOML 1.0 rejected and TOML 1.1 accepts.
The `src/invalid/` directory contains TOML inputs expected to throw `TomlEditError`
during parse.
