# @monochromatic-dev/shell-argument-core

Neutral helpers for safe command-line arguments,
shared by tools that build shell text or validate values before
those values reach a command line.

The package is private and has no build output:
consumers import the TypeScript sources through the `./ts/*`
subpath,
for example `@monochromatic-dev/shell-argument-core/ts/shell-quote.ts`.

## Why this location and shape

- `package/shell-argument/core` follows the layout rule `package/<category>/<name>` with singular segments;
  the package name is `@monochromatic-dev/shell-argument-core`.
  The category is the concern (arguments that a shell reads),
  not a consumer.
- Before this package existed,
  `@monochromatic-dev/crate-release-cli` imported these helpers from
  `@monochromatic-dev/mutation-test-cli-cargo-mutants`,
  which made a release tool depend on a mutation-test tool.
  Both now depend on this package instead.
- No existing package owned shell quoting.
  `@monochromatic-dev/agent-harness-shared-shell-command-analyzer` parses shell source for guardrails and never writes
  it,
  and the other `shellQuote` functions in the repository are private to one file with a different grammar.
  Those are left to their owners.

## Exports

- `shellQuote(value)` encodes a value as one POSIX shell word that expands to exactly the value,
  by single-quoting it
  and spelling each single quote as `'\''`.
  A value that holds NUL is refused with `ShellArgumentError`,
  because no shell argument can hold NUL.
- `usesOnly`,
  `startsThenUses`,
  `hyphenateOutside` and `isPositiveWholeNumber` check or rewrite text against a set of
  allowed characters in one linear pass,
  with no regular expression.
  `decimalDigits`,
  `alphanumericCharacters` and `wordStartCharacters` are the common sets.
- `ShellArgumentError` is the failure type.

## Tests

`shell-quote.unit.test.ts` expands the quoted form of every adversarial value with a real `sh` and compares the
result with the value,
then pins the exact spelling for plain values,
embedded single quotes and the empty value.
`src/adversarial-fixture.ts` holds the values:
spaces,
both quote kinds,
`$`,
command separators,
substitution,
globs,
backslashes,
option-looking text,
line breaks,
tabs,
`printf` directives and non-ASCII text.

## Tasks

`lint`,
`lint:types`,
`lint:oxlint`,
`format:oxlint` and `test:unit` are the usual package checks.
