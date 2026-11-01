# oxlint 1.78.0: `no-class` silently ignores its `suffixes` option
and `promise/prefer-await-to-callbacks` matches parameter names only,
so implementing a Node `Writable` trips two rules with no configurable escape

Two rules fire on the same small piece of code,
 a Node stream test double,
and neither offers the escape its message implies.
`no-restricted-syntax/no-class` is ours and advertises a configurable allowlist that never took effect;
`promise/prefer-await-to-callbacks` is oxlint's and has no options at all.

## Symptom

Writing a `Writable` test double to exercise stdout backpressure in
`package/mcp/stdio/src/transport.unit.test.ts` produced two findings on one class:

```text
! promise(prefer-await-to-callbacks): Prefer `async`/`await` to the callback pattern
   ,-[src/transport.unit.test.ts:104:5]
 103 |     _encoding: unknown,
 104 |     callback: () => void,
     :     ^^^^^^^^^^^^^^^^^^^^
 105 |   ): void {
   help: Refactor to use an `async` function with `await` instead of passing callbacks
         for cleaner error handling and control flow.

! no-restricted-syntax(no-class): Classes are banned unless the direct superclass identifier
  or the class's own name ends with a configured suffix (default: `Error`, `Element`).
  Replace with a factory function returning a frozen object; `Symbol.dispose` belongs in the
  returned literal, not as a class member.
   ,-[src/transport.unit.test.ts:77:1]
  77 | class BackpressuredSink extends Writable {
```

Both are warnings,
 and the repo's lint task exits non-zero on warnings,
so either one fails `mise run //package/mcp/stdio:lint`.

The `no-class` message names "a configured suffix",
which reads as an invitation to add `Writable` to the list.
Doing so changes nothing,
 with no error and no warning that the option was discarded.

## Root cause

### `no-class`: options read at a moment when oxlint has not populated them

Ours,
 in `package/oxlint-plugin/no-restricted-syntax/src/rule/no-class.ts`.
The rule declares a real schema and a real default:

```ts
// package/oxlint-plugin/no-restricted-syntax/src/rule/no-class.ts:159
    schema: [
      {
        type: 'object',
        properties: {
          suffixes: {
            type: 'array',
            items: {
              type: 'string',
            },
          },
        },
        additionalProperties: false,
      },
    ],
```

and,
 before this investigation,
 resolved the option list once inside `createOnce`:

```ts
// package/oxlint-plugin/no-restricted-syntax/src/rule/no-class.ts, before the fix
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    const { options, } = context;
    const suffixes = readSuffixes(options ?? [],);
```

Oxlint does not populate `context.options` at that point.
Its own context factory documents the lifecycle:

```ts
// apps/oxlint/src-js/plugins/context.ts:533 (oxc @ 44231fb5f7646020719533fee250ac313a3d997c)
      // Rule options for this rule on this file.
      // Initially `null` during `createOnce`, set to options object before linting a file in `lintFileImpl`.
      options: {
        value: null!,
        enumerable: true,
        configurable: true,
      },
```

and the declared type deliberately hides it,
 so TypeScript raises nothing:

```ts
// apps/oxlint/src-js/plugins/context.ts:479
  /**
   * Rule options for this rule on this file.
   */
  // Note: This is `null` during `createOnce` call, but we keep the type simple to make it easier for the user.
  options: Readonly<Options>;
```

So `options` was `null`,
 `null ?? []` gave `[]`,
and `readSuffixes([])` fell through its `Array.isArray(suffixes)` guard to the defaults:

```ts
// package/oxlint-plugin/no-restricted-syntax/src/rule/no-class.ts:64
  const [first,] = options;
  if (!isFirstOption(first,))
    return DEFAULT_SUFFIXES;
```

Every file therefore linted against `['Error', 'Element']` no matter what the config said.

This is not an oxlint limitation.
Options are plumbed through to external plugins:

```rust
// crates/oxc_linter/src/external_plugin_store.rs:45
    /// The rule ID is also stored, so that can merge options with the rule's default options on JS side.
    options: IndexVec<ExternalOptionsId, (ExternalRuleId, SmallVec<[serde_json::Value; 1]>)>,
```

An earlier reading of this was wrong and is worth recording:
the first hypothesis was that oxlint drops options for JS plugins entirely.
The positive control below disproves it,
since the same config that changed nothing before the fix changes behavior after it,
with no oxlint upgrade in between.

Scope of the defect across the repo:
three first-party rules declare a `schema`
(`no-class`,
 `stylistic/comma-dangle`,
 `stylistic/semi`),
`no-class` was the only one that ever read `context.options`,
and it read them at the wrong time.
The other two never read options at all,
so no first-party rule in this repo was configurable before this fix.

### `promise/prefer-await-to-callbacks`: a name match, with no options

Oxlint's own rule,
 unchanged by anything local.
It reports the last parameter of any function whose identifier is literally `callback` or `cb`:

```rust
// crates/oxc_linter/src/rules/promise/prefer_await_to_callbacks.rs:131
    fn check_last_params_for_callback(params: &FormalParameters, ctx: &LintContext) {
        let Some(param) = params.items.last() else {
            return;
        };

        let id = param.pattern.get_identifier_name();
        if matches!(id.as_deref(), Some("callback" | "cb")) {
            ctx.diagnostic(prefer_await_to_callbacks(param.span));
        }
    }
```

There is no type analysis,
 no check for whether the function implements an external interface,
and no configuration:
`pub struct PreferAwaitToCallbacks;` is a unit struct,
the file contains no `from_configuration`,
and `declare_oxc_lint!` declares no schema.
The upstream test list is consistent with a pure name match,
`"function getData(id, callback) {}"` being a failing case
while nothing in the passing list concerns types or interfaces.

The practical consequence is that `_write(chunk, encoding, callback)`,
whose parameter name is fixed by Node's stream contract only by convention,
is indistinguishable to this rule from a hand-rolled callback API.
Renaming the parameter to `done` silences it,
 which is why renaming is a trap rather than a fix.

## Verification

Versions under test:

-   `oxlint` 1.78.0 (`node_modules/.bin/oxlint --version` reports `Version: 1.78.0`).
-   oxc source at commit `44231fb5f7646020719533fee250ac313a3d997c`,
    whose `crates/oxc_linter/Cargo.toml` declares `version = "1.78.0"`,
    matching the installed binary.

### Harness

```sh
HARNESS="${HOME}/temp/agent/oxlint-class-callback-harness"
mkdir --parents "$HARNESS"
cat > "$HARNESS/fixture.ts" <<'EOF'
import { PassThrough, Writable, } from 'node:stream';

// 1. class extending Writable
class BackpressuredSink extends Writable {
  override _write(_chunk: unknown, _encoding: unknown, callback: () => void,): void {
    callback();
  }
}

// 2. class whose own name ends in Writable
class LoggingWritable extends Writable {}

// 3. Writable constructed with an options object, no class declaration
const optionSink = new Writable({
  write(_chunk: unknown, _encoding: unknown, callback: () => void,) {
    callback();
  },
},);

// 4. same, parameter renamed away from callback/cb
const renamedSink = new Writable({
  write(_chunk: unknown, _encoding: unknown, done: () => void,) {
    done();
  },
},);

// 5. PassThrough, no _write implementation at all
const passSink = new PassThrough({ highWaterMark: 1, },);

export { BackpressuredSink, LoggingWritable, optionSink, passSink, renamedSink, };
EOF
node_modules/.bin/oxlint -c oxlint.config.ts "$HARNESS/fixture.ts"
```

### Patterns that lint clean

-   `new PassThrough({ highWaterMark: 1 })` with no write implementation:
    neither rule fires.
-   A `write` option whose last parameter is named anything other than `callback` or `cb`
    (case 4 above):
     `prefer-await-to-callbacks` does not fire.
    `no-class` never fires on `new Writable({ ... })`,
    because there is no class declaration to report.

### Patterns that fail, by rule

`no-restricted-syntax(no-class)`,
 reported at the class declaration:

-   `class BackpressuredSink extends Writable { ... }` (fixture line 4).
-   `class LoggingWritable extends Writable {}` (fixture line 11):
    the class's own name ends in `Writable`,
     which is not a default suffix.

`promise(prefer-await-to-callbacks)`,
 reported twice per construct,
once at the parameter and once at the call:

-   The `callback` parameter of a class `_write` method (fixture line 5,
     column 56),
    and the `callback()` call inside it (line 6).
-   The `callback` parameter of an options-object `write` (line 15,
     column 46),
    and its call (line 16).

Note what this catalog settles:
moving from a class to `new Writable({ ... })` clears `no-class` but **not**
`prefer-await-to-callbacks`,
 because the parameter survives the move.

### Positive control for the `no-class` option fix

The control is a case that must move.
`class MyError extends Error {}` is allowed by the default suffixes,
so configuring `suffixes: []` must start reporting it if options are honored.

```sh
cat > "$HARNESS/error-probe.ts" <<'EOF'
class MyError extends Error {}
export { MyError, };
EOF
# Generate a config identical to the repo's except for the no-class options.
node --input-type=module-typescript -e "
import base from './oxlint.config.ts';
import { writeFileSync } from 'node:fs';
const p = structuredClone(base);
p.rules = { ...p.rules, 'no-restricted-syntax/no-class': ['warn', { suffixes: [] }] };
writeFileSync(process.env.HOME + '/temp/agent/oxlint-class-callback-harness/empty-suffixes.oxlintrc.json', JSON.stringify(p, null, 2));
"
node_modules/.bin/oxlint -c "$HARNESS/empty-suffixes.oxlintrc.json" "$HARNESS/error-probe.ts" | rg -c 'no-class'
```

Before the fix:
 `0` reports.
After the fix:
 `1` report.

And with `suffixes: ['Error', 'Element', 'Writable']` against a three-class probe:

```ts
class BackpressuredSink extends Writable {}  // line 3
class LoggingWritable {}                     // line 4
class SomethingElse {}                       // line 5
```

Before the fix,
 lines 3,
 4,
 and 5 are all reported.
After it,
 only lines 4 and 5 are:
line 3 is allowed by the new suffix,
while line 4 stays reported because it has no `extends` clause at all
and the rule reports superclass-less classes before it ever consults the class's own name.

## Verified workarounds

### Use a `PassThrough` and test the writer directly

What this repo shipped in `package/mcp/stdio/src/transport.unit.test.ts`.
A `PassThrough` with a one-byte high-water mark refuses writes until something consumes it,
which is all a backpressure test needs,
 so no `_write` implementation exists to carry a
`callback` parameter and no class is declared:

```ts
const sink = new PassThrough({ highWaterMark: 1, },);
const writer = processStdoutWriter({ stream: sink, },);
const pending = writer.write(new TextEncoder().encode(`${'x'.repeat(1024,)}\n`,),);
// ... assert `pending` is still parked, then:
sink.resume();
expect(await pending,).toBe(1025,);
```

Tradeoff:
 it tests the writer rather than the whole `serve` loop.
The loop-level property,
 that `serve` does not return with replies still buffered,
follows from this plus the existing `await` in `writeSerializedMessage`,
but is inferred rather than measured.
Measuring it directly needs a slow consumer,
 which reintroduces timing dependence.

### Configure `suffixes` (only works after the fix below)

```jsonc
// oxlint config
"no-restricted-syntax/no-class": ["error", { "suffixes": ["Error", "Element", "Writable"] }]
```

Tradeoff,
 and the reason this repo did not adopt it:
the suffix is matched with `String.prototype.endsWith` against the direct superclass identifier
or the class's own name,
 so allowlisting `Writable` also permits any class merely *named*
`*Writable` that extends something else entirely,
 and permits every `Writable` subclass in the
workspace,
 not only test doubles.
It also does nothing for `prefer-await-to-callbacks`,
 which still fires on the `_write`
parameter,
 so the class route needs a second escape regardless.

### Scoped disable for `prefer-await-to-callbacks`

If a Node interface implementation is unavoidable:

```ts
// oxlint-disable-next-line promise/prefer-await-to-callbacks -- Node's Writable._write contract dictates this parameter; the rule matches the name only
```

Tradeoff:
 `LN3` requires a written justification proving configuration cannot work
before a suppression stands,
 which for this rule is provable (it has no options)
but still leaves a permanent local exception where a design change removed the need entirely.

## The fix applied to `no-class`

Read the options where oxlint has populated them,
 per file rather than per rule:

```diff
--- a/package/oxlint-plugin/no-restricted-syntax/src/rule/no-class.ts
+++ b/package/oxlint-plugin/no-restricted-syntax/src/rule/no-class.ts
-    const { options, } = context;
-    const suffixes = readSuffixes(options ?? [],);
-
     function matchesSuffix(name: string,): boolean {
-      return suffixes.some(function endsWith(suffix,): boolean {
-        return name.endsWith(suffix,);
-      },);
+      const { options, } = context;
+      return readSuffixes(options ?? [],)
+        .some(function endsWith(suffix,): boolean {
+          return name.endsWith(suffix,);
+        },);
     }
```

Verification command and result are the positive control above:
`0` reports before,
 `1` after,
 with no oxlint change in between.
`mise run //package/oxlint-plugin/no-restricted-syntax:test:unit` passes,
and `mise run //package/mcp/stdio:lint` still reports `0 warnings and 0 errors`,
confirming the repo's own linting is unchanged because its config sets no `suffixes`.

Landed as commit `50eeca98b`.

A pre-existing,
 unrelated `prefer-readonly-parameter-types` error in
`package/oxlint-plugin/no-restricted-syntax/src/rule/prefer-caught-value-text.syntax.ts`
makes that package's full `lint` task fail;
 it is untouched by this change.

## What does not work

-   **Renaming the parameter to `done` or `finished`.**
    It silences `prefer-await-to-callbacks` because the rule matches names only,
    which is precisely why it is not a fix:
    the callback is still there,
     and the next reader learns nothing.
-   **Switching from `class X extends Writable` to `new Writable({ write })`.**
    Clears `no-class`,
     leaves `prefer-await-to-callbacks` firing on the same parameter.
    Verified as fixture case 3.
-   **Adding `Writable` to `suffixes` before the options fix.**
    Silently no-ops.
     This is what sent the investigation into oxlint's source.
-   **Expecting the type checker to catch the `createOnce` options bug.**
    Oxlint types `options` as non-nullable and documents the `null` in a comment,
    so `const { options } = context` type-checks and the `?? []` looks like sound defensiveness.

## Upstream filing decision

`.out-of-scope/` was checked before considering any filing.
The directory holds `bun-install.md`,
 `cargo-workspace.md`,
 `claude-code-upstream-bugs.md`,
`codex-harness.md`,
 `jsr.md`,
 `lightningcss.md`,
 `low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
 `pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
 and `typescript-project-references.md`.
None covers oxlint or oxc,
 so no exemption applies and the six constraints are walked below.

The `no-class` half is ours and needs no upstream anything;
 it is fixed above.
The remaining candidate is `promise/prefer-await-to-callbacks` matching parameter names
without regard to whether the function implements an externally dictated interface.

1.  **Is it really upstream's fault?**
     Partly.
    The rule behaves exactly as its implementation describes,
     and its documented examples
    (`function getData(id, callback) {}`) are honest about being name-based.
    What is arguably a defect is the absence of any escape short of a file- or line-level
    disable when a name is dictated by an external contract.
    That is a feature request,
     not a misbehavior.
2.  **Can upstream fix it?**
     Yes.
    Adding options to a rule is routine in oxc;
     many rules already carry
    `from_configuration`.
     Nothing architectural blocks an `allowedNames` or
    `ignorePattern` option.
3.  **Are they supporting this use case?**
     Not established.
    The rule is a port of `eslint-plugin-promise`'s rule,
     whose upstream also has no options.
    Nothing in the oxc docs or tests addresses Node stream interface implementations.
4.  **Would the repo welcome our contribution?**
     Yes,
     conditionally.
    `CONTRIBUTING.md` carries an "AI Usage Policy" section that reads
    "We encourage the use of AI tools to assist with development",
     subject to three conditions:
     disclose AI usage,
     accept responsibility for what you submit,
     and expect that
    "Low-quality or unreviewed AI content will be closed immediately".
    No ban exists,
     so the constraint passes,
     but any filing must disclose assistance and name the
    reproduction and source trace a human verified.
5.  **Will they likely fix it?**
     No signal in either direction,
     which per this check's own wording passes rather than fails.
    The tracker was searched across both open and closed state:
     `gh search issues --repo oxc-project/oxc prefer-await-to-callbacks`,
     the same query against `gh search prs`,
     and the widened terms `createOnce options null` and `js plugin rule options`.
    The nearest hit is
    [oxc#20343](https://github.com/oxc-project/oxc/issues/20343),
     closed,
     reporting that the rule flags `Promise.resolve()`.
    That is a detection false positive,
     not a request for an allowlist option,
     so it is not a duplicate and no additive comment belongs on it.
    Nothing in the tracker leans against adding options to this rule.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     No.

Constraint 3 is unmet and constraint 6 is unmet,
 so the auto-prototype trigger
("constraints 1 through 5 hold or sorta-hold") never fires,
 and **nothing is filed**.
The local need evaporated independently once the test stopped implementing `_write` at all,
 so there is no live incident to report even if the gate had opened.
No draft issue is kept:
 with no upstream-visible failure left on our side,
 a draft would describe a problem we no longer have.
A future session that genuinely needs an escape from this rule should start at
constraint 3,
 look for oxc docs,
 tests,
 or examples covering rule options on ported
`eslint-plugin-promise` rules,
 then prototype the `from_configuration` change before drafting anything.
The duplicate search and contribution-policy read are done and recorded here;
 redoing them is only worthwhile if the tracker has moved.
