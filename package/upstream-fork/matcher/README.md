## module-matcher-fork

TypeScript fork of [`matcher`](https://github.com/sindresorhus/matcher),
 in-repo so its behavior is owned,
 tested,
 fuzzed,
 and mutation-tested here instead of trusted from a third-party package.

### Attribution

Derived from [`matcher`](https://github.com/sindresorhus/matcher) by
[Sindre Sorhus](https://sindresorhus.com),
 released upstream under the [MIT License](LICENSES/MIT.txt).
Matching semantics,
 validation rules,
 and error message texts come from `matcher` 6.1.0.
Upstream's copyright and permission notice are preserved verbatim in
`LICENSES/MIT.txt`;
 this fork's own code is licensed `LGPL-3.0-or-later AND MIT`
as declared in `package.json`.

### Usage

```ts
// package/upstream-fork/matcher/example.ts
import {
  isMatch,
  matcher,
} from '@monochromatic-dev/upstream-fork-matcher';

matcher({
  inputs: ['foo', 'bar', 'moo'],
  patterns: ['*oo', '!foo'],
}); // => ['moo']

isMatch({
  inputs: 'unicorn',
  patterns: 'uni*',
}); // => true
```

### Behavior

#### Patterns

`matcher({ inputs, patterns, options })` returns the inputs matching the
patterns, in input order with duplicates kept.
`isMatch({ inputs, patterns, options })` reports whether any input matches.
Both accept a string or a readonly string list for inputs and patterns.

`inputs` and `patterns` also accept `undefined` (normalizing to an empty
list, matching upstream's omitted-argument behavior).
Array holes and explicit `undefined` entries are dropped.
Any other non-string value throws `InvalidInputsError` or
`InvalidPatternsError` with upstream's message text verbatim.

#### Wildcards

`*` matches zero or more characters, including newlines.
A backslash makes the next character literal, so an escaped `\*` matches a
literal asterisk and a trailing backslash stays literal.
A leading `!` negates the pattern; only the leading position negates, so
`!!foo` is a negated pattern matching `!foo`.

#### Matching order

Negated patterns are checked first and exclude immediately.
With no normal pattern present, an input is kept when no negation hits.
Otherwise the default mode keeps an input hitting any normal pattern, while
`allPatterns` requires every normal pattern to hit.

`isMatch` follows the same per-input rule, except under `allPatterns` with
more than one negated pattern and no normal pattern: then every input must
match, matching upstream's multi-negation handling.

#### Case folding

Matching folds case unless `caseSensitive` is set.
Folding uppercases exactly as a case-insensitive regular expression does:
characters whose uppercasing changes length (`ﬁ` to `FI`) or turns non-ASCII
into ASCII (`ß` to `SS`, `ı` to `I`) are kept, so lookalike characters never
match ASCII patterns.
Surrogates fold as lone UTF-16 units, so a wildcard can match half of a
surrogate pair exactly as a non-`u`-flag regular expression does.

#### Options

`undefined` and `null` options both mean defaults.
Only own flag properties count: prototype-inherited `caseSensitive` or
`allPatterns` never enable matching behavior.
Truthy flag values enable the flag, matching upstream's truthiness test.

#### Cache

Compiled patterns are cached by case-sensitivity prefix plus raw pattern
text, holding at most 1000 entries with oldest-first eviction, matching
upstream's bound.
The cache holds flat strings of its own so larger sliced sources stay
collectable.

### Deviations from upstream matcher

#### Call shape

Entry points take one destructured options object
(`matcher({ inputs, patterns, options })`)
 instead of positional arguments
(`matcher(inputs, patterns, options)`).
Repository lint bans multi-positional-parameter declarations outright.

#### Error types

Invalid inputs throw `InvalidInputsError` and invalid patterns throw
`InvalidPatternsError`
 instead of bare `TypeError`s.
Both extend `TypeError` and carry upstream's message text verbatim,
 so
migrating callers see identical diagnostics and the fuzz sidecar's
differential oracle can compare both implementations directly.

#### Internal structure

The single-file upstream is split into `inputs.ts` (normalization),
`matcher-options.ts` (options),
 `case-fold.ts` (case folding),
 `pattern.ts`
(splitting and matching),
 `pattern-cache.ts` (bounded cache),
 and `matcher.ts`
(set compilation),
 leaving the package with zero runtime dependencies.

### Layout

`matcher.ts` owns set compilation,
 `pattern.ts` splitting and matching,
`case-fold.ts` case folding,
 `pattern-cache.ts` the bounded cache,
`inputs.ts` normalization,
 `matcher-options.ts` options,
 and `errors.ts` the
error classes.
Test files sit beside each module as `<stem>.unit.test.ts`,
 so mutation
testing selects each module's tests automatically.

### Testing

```bash
# package/upstream-fork/matcher/mise.toml

# Unit tests against the built dist
mise run //package/upstream-fork/matcher:buildAndTest

# Property-based fuzz campaign (invariants plus an upstream differential oracle)
mise run //package/upstream-fork/matcher.fuzz:fuzz

# Coverage-reachability gate over this package's src
mise run //package/upstream-fork/matcher.fuzz:fuzz:coverage

# Container-isolated mutation testing
mise run //package/upstream-fork/matcher:test:mutation
```
