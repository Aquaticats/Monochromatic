# `@h3mantd/ip-kit` 1.1.0 `CIDR.parse()` accepts non-decimal prefix text

## Symptom

`@h3mantd/ip-kit` 1.1.0 accepts CIDR prefixes whose full text is not decimal.
The published artifact produced:

```text
192.0.2.1/24junk -> 24
192.0.2.1/0x18   -> 0
192.0.2.1/2e1    -> 2
192.0.2.1/ 24    -> 24
192.0.2.1/24     -> 24
```

The final input in that display has trailing whitespace.
`CIDR.parse('192.0.2.1/24junk').toString()` becomes `192.0.2.1/24`,
 so normalization erases the junk.

Malformed addresses,
 stacked slash suffixes,
 empty prefixes,
 and numeric prefixes above family bounds are rejected.
The failure is non-decimal text that `parseInt` partially consumes.

## Root cause

The source is tag `v1.1.0`,
 commit
[`cf077b0316ba484c5e357403e2aeb650b7b2695b`][release-commit].
The inspection clone was `~/temp/agent/ip-kit-2026-07-28`.

`src/domain/cidr.ts:23-27` checks only that splitting on `/` yields two parts.
`src/domain/cidr.ts:28-31` then parses the prefix and rejects only `NaN`:

```ts
const prefix = parseInt(parts[1], 10);
if (isNaN(prefix)) {
  throw new ParseError(`Invalid prefix: ${parts[1]}`);
}
```

With radix ten,
 `parseInt` stops at the first unsupported character.
It maps `24junk` to 24,
 `2e1` to 2,
 and `0x18` to 0,
 while ignoring surrounding whitespace.
Every result is numeric,
 so the guard passes.

`src/domain/cidr.ts:32-43` then checks only whether that truncated number falls within 0 to 32 for IPv4 or 0 to
128 for IPv6.
The converted values do,
 so the parser constructs a CIDR and loses the suffix text.

An earlier possibility was that only alphabetic suffixes passed.
The measured hexadecimal-looking,
 exponent-looking,
 and whitespace cases show that partial numeric parsing is
the general cause.

## Verification

### Version and artifact

Verified on 2026-07-28 against:

- npm `@h3mantd/ip-kit` 1.1.0;
- integrity
  `sha512-oD9D9uHVkz/na6uYFcoQru/46WXlWMjkP8Pqy4xkb8wn8If0DNKEzVxRHvPmg6wSAufw40JF2lsz7Isl75zyWw==`;
- tag and commit `cf077b0316ba484c5e357403e2aeb650b7b2695b`;
- Node 24.18.0 on Linux x86-64 in a network-disabled container.

The catalog transcript is `~/temp/agent/wg-cidr-validation/logs/proc_26-stdout.log`.
The broader consumer result is `proc_16-stdout.log` in the same directory.

### Runnable published-package harness

```js
// reproduce.mjs
import { CIDR } from '@h3mantd/ip-kit';

for (const input of [
  '192.0.2.1/24junk',
  '192.0.2.1/0x18',
  '192.0.2.1/2e1',
  '192.0.2.1/ 24',
  '192.0.2.1/24 ',
]) {
  console.log(input, CIDR.parse(input).prefix);
}
```

```console
npm install --ignore-scripts --save-exact @h3mantd/ip-kit@1.1.0
node reproduce.mjs
```

### Patterns handled correctly

Published 1.1.0 correctly handles decimal prefixes through family bounds.
It rejects an IPv4 prefix above 32,
 an IPv6 prefix above 128,
 an empty prefix,
 stacked slash suffixes,
 and malformed
addresses.

### Patterns accepted incorrectly

Published 1.1.0 accepts trailing alphabetic text,
 hexadecimal-looking text,
 exponent-looking text,
 and leading or
trailing whitespace.
It converts them to the numeric fragments shown in the symptom catalog.

### Prototype result

The minimal prototype requires one or more ASCII decimal digits before converting with `Number`.
It preserves leading-zero decimal text such as `/024`,
 which still means prefix 24.
The patch is [`ip-kit-trailing-prefix-text.patch`](ip-kit-trailing-prefix-text.patch).
It was applied in a fresh disposable clone at the release commit.

The first sandbox attempt passed all 161 tests but Vitest then failed to write
`/work/node_modules/.vite/vitest/results.json` because the development dependency mount was read-only.
That was a harness-only filesystem error.
The corrected sandbox used a writable disposable dependency mount.

The corrected network-disabled run passed lint,
 type checking,
 161 tests in 13 files,
 the ESM and CommonJS build,
declaration generation,
 and a built-artifact malformed-prefix catalog.
It exited zero in six seconds with `prototype prefix consumer passed`.
The transcript is `~/temp/agent/wg-cidr-validation/logs/proc_27-stdout.log`.

## Verified workarounds

### Keep `cidr-tools`

The current `wg-allowedips` plan uses `cidr-tools` 12.1.3.
Its published parser rejected `/24junk`.
The planned `node:net` address check and explicit family bounds close its separate validation gaps.

Tradeoff:
 this keeps one runtime dependency and does not gain `ip-kit`'s broader unused address toolkit.
It preserves the smallest production set-operation boundary.

### Guard the full prefix before `CIDR.parse`

```js
function parseStrictCidr(input, CIDR) {
  const parts = input.split('/');
  const prefix = parts[1] ?? '';
  const invalidDigit = Array.from(prefix).some(
    (character) => character < '0' || character > '9',
  );

  if (parts.length !== 2 || prefix.length === 0 || invalidDigit) {
    throw new Error(`Invalid prefix: ${prefix}`);
  }

  return CIDR.parse(input);
}
```

The published-package probe rejected every malformed catalog entry and accepted `/024` as 24.
Its transcript ends with `consumer guard passed` in `proc_26-stdout.log`.

Tradeoff:
 this creates a second prefix parser at the consumer boundary.
It weakens the production-code and validation-clarity advantages being evaluated.

### Carry the minimal patch in a fork

Apply [`ip-kit-trailing-prefix-text.patch`](ip-kit-trailing-prefix-text.patch) and consume a forked release.
The prototype passed every upstream check and a built-artifact regression catalog.

Tradeoff:
 the project owns publishing,
 provenance,
 updates,
 and rebasing until upstream releases the fix.
Using `ip-kit` still requires family partitioning and host-to-CIDR conversion for `wg-allowedips`.

## What does not work

- `Number.isNaN(prefix)` after `parseInt` does not work because partial parses produce ordinary numbers.
- Family prefix bounds do not work because 24,
   2,
   and 0 are valid IPv4 bounds.
- `node:net` address validation does not work because it validates only the address before `/`.
- Comparing normalized output to input is not a clean fix because valid host-bit normalization can also change text.
- Relying on the upstream suite does not work because its 160 release tests contain no non-decimal prefix catalog.

## Upstream filing artifact

### Duplicate and policy checks

No matching issue or pull request was found in open and closed searches for:

- `prefix trailing junk parseInt CIDR`;
- `invalid prefix`;
- `CIDR parser prefix`;
- `parseInt prefix`;
- `trailing junk`.

`CONTRIBUTING.md` explicitly invites bug reports with a minimal reproduction,
 expected and actual behavior,
 and
environment details.
The repository has no issue template,
 pull request template,
 or AI-assistance policy.
No matching exemption exists under this repository's `.out-of-scope/` directory.
Pull request [1][pr-1] shows recent maintainer work on stricter errors and CIDR correctness,
 but it is owner-authored,
so it does not prove external-contributor turnaround.

### Upstream filing decision

1.  **Is it really upstream's fault?
    ** Yes.
    The public parser discards invalid prefix text after its own `parseInt` call.
2.  **Can upstream fix it?
    ** Yes.
    Full-text decimal validation before conversion is local and preserves the API.
3.  **Are they supporting this use case?
    ** Yes.
    `CIDR.parse()` is public,
     `ParseError` represents invalid input,
     and the contribution guide requires edge-case
    tests.
4.  **Would the repo welcome our contribution?
    ** Yes.
    `CONTRIBUTING.md` invites issues and pull requests,
     and no policy bars this contribution.
5.  **Will they likely fix it?
    ** Yes with moderate confidence.
    The repository has no duplicate or won't-fix signal and recently shipped CIDR correctness work.
6.  **Have we prototyped a minimal compatible fix?
    ** Yes.
    The two-hunk patch passed all upstream checks and a built-artifact regression catalog.

All filing constraints pass.
This report keeps a fileable draft but does not send it without separate authorization for external communication.

### Draft issue

~~~md
Title: `CIDR.parse()` accepts trailing and non-decimal prefix text

## Description

`@h3mantd/ip-kit` 1.1.0 accepts CIDR prefixes whose complete text is not a
decimal prefix length. `parseInt` silently consumes a leading numeric fragment.

## Reproduction

```js
import assert from 'node:assert/strict';
import { CIDR } from '@h3mantd/ip-kit';

for (const input of [
  '192.0.2.1/24junk',
  '192.0.2.1/0x18',
  '192.0.2.1/2e1',
  '192.0.2.1/ 24',
  '192.0.2.1/24 ',
]) {
  assert.throws(() => CIDR.parse(input));
}
```

Actual prefixes are 24, 0, 2, 24, and 24. Expected behavior is `ParseError` for
each input.

## Root cause

`src/domain/cidr.ts:28-31` uses `parseInt(parts[1], 10)` and checks only
`isNaN`. `parseInt` accepts a leading integer fragment and ignores trailing or
surrounding unsupported text.

## Suggested fix

Require the entire prefix part to contain one or more ASCII decimal digits,
then convert it with `Number`. Add the malformed catalog as a parser regression
test. A two-hunk prototype passes lint, type checking, all 161 tests, the build,
and a separate assertion against the rebuilt ESM artifact.
~~~

[pr-1]: https://github.com/h3mantD/ip-kit/pull/1
[release-commit]: https://github.com/h3mantD/ip-kit/commit/cf077b0316ba484c5e357403e2aeb650b7b2695b
