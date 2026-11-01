# Oxlint 1.81.0 `createOnce` option read during plugin registration fails JS plugin loading

## Symptom

A configurable JS plugin rule that indexes `context.options` directly inside `createOnce` makes Oxlint reject the
configuration before linting any source file:

```text
Failed to parse oxlint configuration file.

  x Failed to load JS plugin: ./oxlint-create-once-options-bad.mjs
  |   TypeError: Cannot read properties of null (reading '0')
  |     at Object.createOnce (.../oxlint-create-once-options-bad.mjs:14:37)
  |     at registerPlugin (.../oxlint/dist/lint.js:17882:32)
  |     at loadPlugin (.../oxlint/dist/lint.js:17819:51)
```

The failure occurs even when:

- rule configuration supplies a valid option;
- rule metadata supplies a JSON Schema that requires that option;
- TypeScript accepts `context.options[0]` because `Context.options` is declared as a non-null array.

Reading the same option inside `Program` succeeds and exposes configured value.

## Root cause

Source was inspected at Oxc commit `3d6ddcef3256c2de40dec417bfc20232d9d8f485`, whose
`apps/oxlint/package.json` and `npm/oxlint/package.json` both identify version `1.81.0`.

Oxlint constructs one context and calls `createOnce` while registering each plugin rule.
`apps/oxlint/src-js/plugins/load.ts:287-293` performs that callback before per-file linting:

```ts
if ("createOnce" in rule) {
  // TODO: Compile visitor object to array here, instead of repeating compilation on each file
  const visitorWithHooks = rule.createOnce(context) as SetNullable<
    VisitorWithHooks,
    "before" | "after"
  >;
```

The public source type does not expose lifecycle nullability.
`apps/oxlint/src-js/plugins/context.ts:471-483` deliberately types both properties as non-null while documenting their
runtime registration value:

```ts
export interface Context extends FileContext {
  /**
   * Rule ID, in form `<plugin>/<rule>`.
   */
  // Note: This is `null` during `createOnce` call, but we keep the type simple to make it easier for the user.
  id: string;
  /**
   * Rule options for this rule on this file.
   */
  // Note: This is `null` during `createOnce` call, but we keep the type simple to make it easier for the user.
  options: Readonly<Options>;
```

Context construction then installs `null` for options.
`apps/oxlint/src-js/plugins/context.ts:527-539` names when this changes:

```ts
// Rule options for this rule on this file.
// Initially `null` during `createOnce`, set to options object before linting a file in `lintFileImpl`.
options: {
  value: null!,
  enumerable: true,
  configurable: true,
},
```

Per-file linting installs either rule defaults or user options before any visitor runs.
`apps/oxlint/src-js/plugins/lint.ts:204-218` shows that ordering:

```ts
// Set `options` for rule
const optionsId = optionsIds[i];
debugAssert(optionsId < allOptions.length, "Options ID out of bounds");

// If the rule has no user-provided options, use the plugin-provided default
// options (which falls back to `DEFAULT_OPTIONS`).
// Reuse `OPTIONS_DESCRIPTOR` object to avoid unnecessarily creating a temporary object each time.
OPTIONS_DESCRIPTOR.value =
  optionsId === DEFAULT_OPTIONS_ID ? ruleDetails.defaultOptions : allOptions[optionsId];
Object.defineProperty(ruleDetails.context, "options", OPTIONS_DESCRIPTOR);

let { visitor } = ruleDetails;
if (visitor === null) {
  // Rule defined with `create` method
  debugAssertIsNonNull(ruleDetails.rule.create);
  visitor = ruleDetails.rule.create(ruleDetails.context);
```

Upstream's own lifecycle fixture confirms this is intentional rather than an incidental null.
`apps/oxlint/test/fixtures/createOnce/plugin.ts:20-23` captures options during registration:

```ts
// Available but `null`
const { id, options } = context;
```

Its checked snapshot,
`apps/oxlint/test/fixtures/createOnce/output.snap.md:84-92`, records:

```text
x create-once-plugin(always-run): createOnce: id: null
...
x create-once-plugin(always-run): createOnce: options: null
```

The first implementation hypothesis was that `meta.schema.minItems: 1` guaranteed a populated array whenever
`createOnce` ran.
That reading was wrong.
Schema validation governs configured rule options, while plugin registration still calls `createOnce` before Oxlint
installs any file's validated options.

## Verification

The installed command reported:

```text
$ mise exec -- oxlint --version
Version: 1.81.0
```

Create `oxlint-create-once-options-source.ts`:

```ts
const value = true;
export { value };
```

Create `oxlint-create-once-options-bad.mjs`:

```js
export default {
  meta: { name: 'options-probe' },
  rules: {
    probe: {
      meta: {
        schema: {
          type: 'array',
          minItems: 1,
          maxItems: 1,
          items: [{ type: 'string' }],
        },
      },
      createOnce(context) {
        const mode = context.options[0];
        return {
          Program(node) {
            context.report({ node, message: String(mode) });
          },
        };
      },
    },
  },
};
```

Create `oxlint-create-once-options-good.mjs` by moving one line into `Program`:

```js
export default {
  meta: { name: 'options-probe' },
  rules: {
    probe: {
      meta: {
        schema: {
          type: 'array',
          minItems: 1,
          maxItems: 1,
          items: [{ type: 'string' }],
        },
      },
      createOnce(context) {
        return {
          Program(node) {
            const mode = context.options[0];
            context.report({ node, message: String(mode) });
          },
        };
      },
    },
  },
};
```

Point otherwise identical config files at each plugin:

```json
{
  "jsPlugins": ["./oxlint-create-once-options-bad.mjs"],
  "rules": {
    "options-probe/probe": ["error", "configured"]
  }
}
```

Run each from directory containing harness files:

```sh
mise exec -- oxlint \
  --config ./oxlint-create-once-options-bad.json \
  ./oxlint-create-once-options-source.ts
mise exec -- oxlint \
  --config ./oxlint-create-once-options-good.json \
  ./oxlint-create-once-options-source.ts
```

### Patterns that work cleanly

- Read `context.options` inside `Program` or another AST visitor.
  Verified output was `error options-probe(probe): configured`.
- Read `context.options` inside `before` for state needed by that file's subsequent visitors.
  `apps/oxlint/src-js/plugins/lint.ts:210-228` installs options before invoking `beforeHook`.
- Use ESLint-compatible `create`, which Oxlint invokes per file after installing options at
  `apps/oxlint/src-js/plugins/lint.ts:212-218`.

### Patterns that fail during plugin loading

- Index `context.options[0]` directly in `createOnce`.
  The verified result is `TypeError: Cannot read properties of null (reading '0')`.
- Destructure `const { options } = context` in `createOnce` and use captured value later.
  Upstream's checked fixture records captured value as `null`, so closure remains stale after context property is
  redefined.
- Assume `meta.defaultOptions` or a required options schema populates context before `createOnce`.
  Per-file installation happens later in `lintFileImpl`.

## Verified workarounds

### Read options inside `Program`

```diff
 createOnce(context) {
-  const mode = context.options[0];
   return {
     Program(node) {
+      const mode = context.options[0];
       context.report({ node, message: String(mode) });
     },
   };
 }
```

This preserves `createOnce` and reads options once per file at a callback guaranteed to run for every parsed file.
Tradeoff: option decoding occurs per file rather than once per plugin process.

`package/oxlint-plugin/stylistic/src/rule/require-asterisk-prefix.ts` uses this pattern.
`mise run //package/oxlint-plugin/stylistic:test:unit` passed after the change, including separate `always`, `never`,
omitted-mode, invalid-mode, and bidirectional autofix cases.

### Read options inside `before`

A rule can decode options in `before` and retain result for its visitors.
Tradeoff: Oxlint's writing-plugin documentation warns that `before` is not guaranteed to run for every file in future
optimized traversal.
Use this only when retained state is consumed by visitors whose execution is coupled to that hook.

### Use `create`

Replacing `createOnce` with ESLint's per-file `create` makes option access valid in callback body.
Tradeoff: this abandons Oxlint's alternative lifecycle and its intended future interop optimizations.

## What does not work

- Strengthening option schema does not change registration lifecycle.
  Schema controls validity, not timing.
- Adding `defaultOptions` does not populate registration context.
  `lint.ts` chooses defaults while beginning each file.
- Capturing null in `createOnce` and expecting later `Object.defineProperty` calls
  to update local binding does not work.
  Only later reads through `context.options` see current descriptor value.
- Catching null and selecting fallback inside `createOnce` silently ignores valid per-file configuration.
  It avoids crash by replacing requested semantics with fallback.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?** No.
   Runtime behavior is explicit in `context.ts`, covered by upstream fixture and snapshot, and follows one-context
   `createOnce` lifecycle.
   Consumer read occurred at wrong lifecycle boundary.
2. **Can upstream fix it?** Documentation and public typing could expose lifecycle nullability, but changing runtime to
   provide file options during one-time registration would conflict with per-file configuration.
3. **Are they supporting this use case?** Yes.
   Oxlint documentation supports both rule options and `createOnce`, and directs per-file setup into callbacks.
4. **Would the repo welcome our contribution?** Yes, with conditions.
   `CONTRIBUTING.md:7-20` welcomes contributions and permits AI assistance when disclosed, reviewed, tested, understood,
   and adapted.
   `.github/ISSUE_TEMPLATE/linter_bug_report.yaml:1-42` requests version, command, config, and reproduction.
5. **Will they likely fix it?** Not applicable as runtime bug because source and snapshots assert behavior.
   GitHub issue and PR searches for `createOnce context options null` across open and closed state found no duplicate.
6. **Have we prototyped a minimal fix compatible with their architecture?** No upstream patch is warranted because
   constraint 1 fails.
   Consumer-side deferred read was implemented and verified instead.

Nothing should be filed upstream from this incident.
The tested behavior matches upstream's source-level contract, and a report would not identify a runtime defect or add
new evidence to an existing thread.
