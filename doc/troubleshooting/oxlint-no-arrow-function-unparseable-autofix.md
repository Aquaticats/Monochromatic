# oxlint --fix with oxlint-plugin-no-restricted-syntax 0.0.1 writes unparseable text for bare-parameter arrows

Status (2026-10-22):
 the corrupting fixer is fixed in `5544041a52`,
and the glued-async regression case
 is added in `b7b416ae2f`.
The 20 files the corruption reached were restored in
 `20a3a73271` and `0c2f35836d` and re-formatted through the fixed fixer in `60d4795f95`.
 The formatting-commit step now runs the `lint:parse` gate (`9e3f228e65`).
Incident
 tracking:
issue 1794 [issue-1794][].

## Symptom

A style sweep running `oxlint --fix` through `mise run format:oxlint` (or
`mise run //package/monorepo-task/core-exec:format:oxlint`) leaves `.js`/`.mjs`/`.ts`
files that no longer parse.
Node reports the failure at consumption time:

```text
# node --check <corrupt-file>
SyntaxError: Unexpected token '=>'
```

The TypeScript parser reports the same shape as `TS1005 '(' expected` plus
`TS1005 ';' expected`.
A parameter list whose closing paren was cut inside a
comment produces a different variant:
`SyntaxError: Invalid or unexpected token`
(an unterminated block comment).

Surface patterns and the variant each produces:

- `const f = x => g(x);` (bare parameter,
  expression body):
  the fixer writes
  `function fx => g(x) { return g(x) }`,
  reported as `SyntaxError: Unexpected token '=>'`.
- `const note = entry => { record(entry); };` (bare parameter,
  block body):
  the
  fixer writes `function noteentry => { record(entry) { record(entry); }`,
  same variant.
- `export const f = x => g(x);` and `const f = async x => g(x);` (exported and
  async bare parameters):
  same variant,
  exported or `async` prefixed.
- `const wrap = (value /* ) */) => value;` (a paren inside a parameter-list
  comment):
  the fixer writes `function wrap(value /* ) { return value }`,
  reported
  as `SyntaxError: Invalid or unexpected token`.
- `const f = async(x) => g(x);` (glued `async`,
  parenthesized parameters):
  the
  fixer writes `async function fasync(x) { return g(x) }`.
  This variant PARSES;
  it silently renames the declaration from `f` to `fasync`,
  so a parse gate does
  not catch it and only a declaration-name check does.

## Root cause

The `no-restricted-syntax/no-arrow-function` rule's fixer builds the replacement
declaration by concatenating the function name with `extractParamsText`'s result
and the body:

```ts
// package/oxlint-plugin/no-restricted-syntax/src/rule/no-arrow-function.ts:179
`${exportPrefix}${asyncPrefix}function ${name}${typeParamsText}${paramsText}${returnTypeText} ${bodyText}`;
```

`extractParamsText` must return the parameter list including its parentheses.
Before `5544041a52` it scanned the arrow's source text for the first balanced
paren group,
then returned everything from the start of the scanned text through
that group's closing paren:

```ts
// package/oxlint-plugin/no-restricted-syntax/src/rule/arrow-function-params.ts:167-176 at 5544041a52^
if (ch === '(')
  depth++;
else if (ch === ')') {
  depth--;
  if (depth === 0) {
    return rest.slice(
      0,
      loopIndex + 1,
    );
  }
}
```

That scan has three failure modes,
one per input shape:

1. **Bare parameters.**
   For `label => repliesOf(label)` there is no opening paren
   at the parameter list,
   so the first balanced group is the body's call.
   The
   slice returns `label => repliesOf(label)` and the template glues it directly
   after the name,
   producing `function hoverTextlabel => repliesOf(label) { ... }`.
   This is the shape that shipped into 20 `.mjs` files in `3da31bdc3a`.
2. **Comment parens.**
   The scanner tracks string literals but not comments
   (`arrow-function-params.ts:151-164` at `5544041a52^` handles only quotes and
   backticks),
   so for `(value /* ) */)` the group ends at the comment's `)` and
   the slice truncates inside the comment.
3. **Glued `async`.**
   `indexAfterAsyncPrefix` returns zero when `async` is not
   followed by whitespace:

```ts
// package/oxlint-plugin/no-restricted-syntax/src/rule/arrow-function-params.ts:44-45 at 5544041a52^
if (!isWhitespaceChar(fullText.charAt(index,),))
  return 0;
```

   So `async(x) => x` keeps `async` at the start of the scanned text,
   the slice
   returns `async(x)`,
   and the template's `async ` prefix plus the name plus that
   slice yields `async function fasync(x) { ... }`,
   renaming the declaration.

The fix (`5544041a52`) takes bare parameter lists from AST parameter text
(`paramTexts` passed from `no-arrow-function.ts:143`),
slices parenthesized lists
from their real opening paren via `scanParenGroupText`
(`arrow-function-params.ts:143`),
skips comments via `indexAfterTrivia`
(`arrow-function-params.ts:77`),
and throws `ArrowParamsUnbalancedError`
(`arrow-function-params.ts:27`) on unbalanced input instead of emitting text.

Earlier readings that were wrong:

- **"52 files are corrupted."**
  32 of the reported 52 were artifacts of
  `node --experimental-strip-types --check`,
  which rejects valid type-only
  TypeScript such as `export type { X };` and `export declare const y: number;`.
  Those 32 files parse cleanly under `node --check` and under
  `typescript/sync`'s `transpileModule` with zero diagnostics.
  Only 20 files were
  actually corrupted,
  all `.mjs`,
  all by `3da31bdc3a`.
- **"<typescript@7.1.0-beta> is a native-preview facade with no transpileModule."**
  Wrong:
  the root export (`require('typescript')`) is only a version stub
  (`lib/version.cjs`),
  while the classic API is restored at `typescript/sync` and
  `typescript/async` (`new API()`,
  `transpileModule`,
  `createSourceFile`,
  `close`).
  Probing only the root export produced the false negative.

## Verification

Version under test:
`@monochromatic-dev/oxlint-plugin-no-restricted-syntax`
`0.0.1` at commits `5544041a52` (fix) and `b7b416ae2f` (regression case);
the
corrupting release was the same package at `3da31bdc3a`.

Runnable harness (from the repo root):

```sh
# runs the fix-output suite, which fails 7 of 9 cases at 5544041a52^ and passes after
mise run //package/oxlint-plugin/no-restricted-syntax:test:unit
```

```sh
# confirms any suspect file's parse state; the gate runs the same parsers
mise run //package/monorepo-task/core-exec:lint:parse -- <paths>
```

Red-first evidence:
at `5544041a52^` the suite failed with the corrupt outputs
listed in the Symptom section (recorded in the incident logs),
and the glued-async
case failed at `b7b416ae2f^` with `async function loadasync(path) { return read(path) }`.

Catalog of patterns that work cleanly (post-fix):

- `const hoverText = label => repliesOf(label);` becomes
  `function hoverText(label) { return repliesOf(label) }`.
- `const note = entry => { record(entry); };` becomes
  `function note(entry) { record(entry); }`.
- `const flipInside = sourceFile => copy => copy;` becomes
  `function flipInside(sourceFile) { return copy => copy }` (nested arrow kept).
- `export const targets = label => repliesOf(label);` becomes
  `export function targets(label) { return repliesOf(label) }`.
- `const load = async path => read(path);` becomes
  `async function load(path) { return read(path) }`.
- `const load = async(path) => read(path);` becomes
  `async function load(path) { return read(path) }` (no rename).
- `const lastOf = (label, key,) => { return label; };` becomes
  `function lastOf(label, key,) { return label; }` (list kept verbatim).
- `const wrap = (value /* ) */) => value;` becomes
  `function wrap(value /* ) */) { return value }`.
- `const shout = message => concat(message, ')');` becomes
  `function shout(message) { return concat(message, ')') }`.

Catalog of patterns that fail at `5544041a52^`,
by parse-error variant:

- `SyntaxError: Unexpected token '=>'`:
  every bare-parameter form (expression
  body,
  block body,
  nested arrow body,
  exported,
  async).
- `SyntaxError: Invalid or unexpected token`:
  the comment-paren form
  (`function wrap(value /* ) { return value }`).
- Renaming,
  not parse failure:
  the glued `async` form
  (`async function loadasync(path) { ... }`).

## Verified workarounds

1. **Restore each corrupted file from `68415aeb53` when it has no later legitimate
   edits.**
   The indent sweep and every earlier revision parse.
   Tradeoff:
   any
   formatting intended after that revision is discarded and must be re-applied;
   under concurrent worktree changes the restore must be per-file with explicit
   bases,
   never a whole-file `git restore` against a moving tree.
2. **Re-apply formatting through the fixed fixer and gate the result.**
   `mise run //package/monorepo-task/core-exec:format:oxlint -- <paths>` followed by
   `mise run //package/monorepo-task/core-exec:lint:parse -- <paths>`.
   Tradeoff:
   all-rules fix runs over a whole tree have historically needed several passes to
   settle (the catch-up sweep recorded 9 passes over 190 files),
   while a scoped
   set can converge in one (the 20-file re-apply did,
   checksum-identical on the
   second pass).
   The gate proves parse state only,
   not semantic drift.
3. **Run `lint:parse` at the formatting-commit step.**
   `node --check` covers
   `.js`/`.mjs`/`.cjs` and `typescript/sync`'s `transpileModule` covers TypeScript
   and JSX.
   Tradeoff:
   it is a parse gate;
   the glued-async renaming variant and
   other semantically wrong but parseable output need rule-level tests instead.

## What does not work

- `node --experimental-strip-types --check` as a corruption detector.
  It
  false-positives on valid type-only TypeScript (32 of the 52 reported files),
  so its "failures" cannot be counted as corruption.
  Use `node --check` for JS
  flavors and a real TypeScript parser for TS.
- A parse gate against the glued `async` renaming variant.
  The output parses;
  the
  breakage is a wrong declaration name,
  visible only to a name/shape check or the
  rule's own tests.
- Probing `require('typescript')` for the classic API in 7.1 betas.
  The root is a
  version stub;
  import `typescript/sync` or `typescript/async` and read
  `dist/api/sync/api.d.ts` before concluding a method is missing.
  Note also that
  `api.d.ts` can run ahead of the shipped build:
  `getSyntacticDiagnostics` is
  declared there but is `undefined` at runtime in `7.1.0-beta`.
- `transpileModule` with a `.d.*` `fileName` as the parse check for declaration
  files in `typescript@7.1.0-beta`.
  It panics with `panic: Debug failure. False
  expression: Output generation failed` (measured on
  `package/pnpm-override/stub-silent/index.d.cts`) because the call always emits and a
  declaration file has nothing to emit.
  Parse declaration content under the
  matching non-declaration suffix instead (what `lint:parse` does);
  parse
  diagnostics still report,
  and a broken `.d.cts` fixture yields `TS1110`.
- Rebuilding probe copies with `sed` when checking comment-terminator escaping
  (`*\/` in TSDoc).
  A `sed` round trip corrupted the copy and masked a real
  `SyntaxError: Unexpected identifier` failure;
  rebuild the copy byte-accurately.

## Upstream filing decision

Nothing to file,
anywhere.
The corrupting code is first-party
(`package/oxlint-plugin/no-restricted-syntax`);
oxlint and oxc only apply the fix
text our fixer returns,
and their parsers correctly reject the corrupt text.
The
6-constraint check,
walked explicitly:

1. **Is it really upstream's fault?**
   No. The faulty scan is ours
   (`arrow-function-params.ts` before `5544041a52`).
2. **Can upstream fix it?**
   Not applicable;
   the fix belongs to this repository and
   landed in `5544041a52`.
3. **Are they supporting this use case?**
   Not applicable.
4. **Would the repo welcome our contribution?**
   Not applicable;
   we own the code.
5. **Will they likely fix it?**
   Not applicable.
6. **Have we prototyped a minimal fix?**
   Yes,
   in this repository:
   `5544041a52`
   with 9 fix-output cases,
   red-first.

`.out-of-scope/` was read;
no entry covers this tool,
and none is needed because
there is no upstream surface (constraint 1 fails at the top).
No draft issue and
no comment draft exist or should be filed.

[issue-1794]: https://github.com/Aquaticats/Monochromatic/issues/1794
