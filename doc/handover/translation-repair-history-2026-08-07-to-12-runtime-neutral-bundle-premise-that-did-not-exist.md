# Translation repair history 2026-08-07 to 2026-08-12: the runtime-neutral bundle that did not exist, and a question to a reviewer that was never answered

How a lint error was held up on the belief that exporting a module would break a runtime-neutral bundle,
the discovery that the package has no such bundle,
and the lesson that a question sent to a reviewer and never answered leaves a premise looking examined.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The runtime-neutral bundle this package supposedly has does not exist

Recorded because it once blocked a decision on a false premise.

A `test-import(require-eventual-artifact)` error on
 `src/corpus-run/sheet-path.unit.test.ts` was held up on the belief that the
 rule's suggested remedy,
 exporting the module from the package entry,
would break an invariant:
 that `dist/final/neutral/index.mjs` carries ZERO `node:` specifiers,
 the library being runtime-neutral while all filesystem IO lives in corpus-run
 tooling.

That invariant was never verified.
The question was sent to `pi` and the call died fetching the provider's model
 list,
 so no answer ever came back,
and the premise sat unexamined.

There is no neutral bundle.
`dist/final/` holds `node` and `types` only,
 and `neutral` appears nowhere in the package's `mise.toml`,
`package.json`,
or
 config files;
the sole JavaScript target is `build:js:node`.
So exporting a module that imports `node:fs/promises` through the barrel breaks
 nothing.

The error itself is long resolved along exactly the route that was doubted:
 `sheet-path.ts` is exported from `sheet-barrel.ts`,
 the test imports `../../dist/final/node/index.mjs`,
and the package reports zero `require-eventual-artifact` findings.
Later exports through that barrel (`readSheetIdentity`,
`trackDrawOutputs`,
 `indexReadingsByIssue`) follow the same established pattern rather than
 inventing one.

The general lesson is about the failure mode rather than the bundle:
 a question sent to a reviewer and never answered leaves a premise looking
 examined when nothing examined it.
A dead call is not a deferred answer.
