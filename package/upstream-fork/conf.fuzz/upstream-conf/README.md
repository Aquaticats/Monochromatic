# Vendored upstream `conf` snapshot

Verbatim copies of upstream [`conf`](https://github.com/sindresorhus/conf)
 `source/index.ts` and `source/types.ts` at commit
 `83e267178fb6d19e6447eb3ac3ed0ce5ea1258a0`,
 the GitHub state this fork was cut from.

## Why vendored

The differential oracle must compare against the same upstream the fork
 mirrors.
 npm `conf` 15.1.0 predates this commit and differs observably:
 its `store` getter leaks the `__internal__` bookkeeping key that this
 commit's `_readUserStore` strips,
 and its `reset` resolves defaults by literal key lookup where this commit
 goes through `dot-prop`.
 Comparing against the npm release would normalize real behavior into
 "known drift" forever;
 comparing against this snapshot keeps every difference meaningful.

## License

Upstream `conf` is MIT,
 Copyright (c) Sindre Sorhus; see `../../conf/LICENSES/MIT.txt` for the
 verbatim notice.
 These files are test-oracle fixtures only,
 never shipped or re-exported.

## Files

- `index.ts`:
   upstream `source/index.ts`,
   with one mechanical portability edit:
   its type-only import specifier `./types.js` reads `./types.ts` so Node's
   type stripping can resolve it (the import statement is retained at
   runtime even though every binding is a type).
   Apart from the added first line described below, nothing else differs from upstream bytes.
- `types.ts`:
   upstream `source/types.ts`,
   byte-for-byte.

Both files also carry one added first line, a `// @ts-nocheck` comment,
 because upstream type-checks them under its own tsconfig rather than this repository's.

## Do not reformat or autofix

These files must stay byte-for-byte upstream (plus the two edits above).
 A repository-wide autofix pass (commit `3da31bdc3a`, the `no-restricted-syntax/no-arrow-function` fixer)
 once rewrote the arrow function `set` inside `Conf#set` into a function declaration,
 which loses the lexical `this`,
 so every `appendToArray` and nested `set` call threw
 `Cannot read properties of undefined (reading '#options')`
 and the differential test failed.
 To restore them,
 fetch `source/index.ts` and `source/types.ts` at commit `83e267178fb6d19e6447eb3ac3ed0ce5ea1258a0`
 with `gh api repos/sindresorhus/conf/contents/<path>?ref=<commit>`
 and reapply the edits above.

Runtime dependencies (`ajv`,
 `ajv-formats`,
 `atomically`,
 `debounce-fn`,
 `dot-prop`,
 `env-paths`,
 `semver`,
 `uint8array-extras`) are declared by this sidecar's `package.json` so the
 snapshot resolves them without a build step.
