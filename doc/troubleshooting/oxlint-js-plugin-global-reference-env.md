# Oxlint 1.85.0 JavaScript plugins lose `Array` and `Object` globals when a config sets `env` without `builtin`

## Symptom

A JavaScript plugin rule that asks `context.sourceCode.isGlobalReference(identifier,)` gets `false`
for `Array` and `Object` whenever the config declares an explicit `env` map without `"builtin": true`,
while `Set`,
`Promise`,
and `Buffer` still read as global.
No diagnostic is printed;
rules that guard on "is this the global `Array`" silently stop reporting.

In this repository it disabled the `Array.from`,
`Object.keys`,
and `new Array` checks of `no-restricted-syntax/no-useless-spread`
and the `Array.from` arm of `no-restricted-syntax/prefer-spread` under the shared config,
whose `env` was `{ browser, node, es2026, serviceworker, webextensions, worker }`.
Isolated test configs without `env` passed,
so the defect only showed when linting through the real config.

## Root cause

### An explicit root `env` replaces the default `builtin` env

The default environment holds only `builtin`,
`crates/oxc_linter/src/config/env.rs:65-71`:

```rust
impl Default for OxlintEnv {
    fn default() -> Self {
        let mut map = FxHashMap::default();
        map.insert("builtin".to_string(), true);

        Self(map)
    }
}
```

`Oxlintrc` deserializes with `#[serde(default, …)]` (`crates/oxc_linter/src/config/oxlintrc.rs:174`),
so a present `env` map replaces that default rather than merging into it.
Oxc's own test pins this,
`crates/oxc_linter/src/config/env.rs:111`:

```rust
assert!(!env.contains("builtin"));
```

### Rust rules still see builtin globals

The Rust side treats builtin globals as always enabled,
`crates/oxc_linter/src/context/mod.rs:206-210` and `:248-251`:

```rust
fn get_env_global_entry(&self, var: &str) -> Option<GlobalValue> {
    // builtin is always readonly
    if GLOBALS_BUILTIN.contains_key(var) {
        return Some(GlobalValue::Readonly);
    }
```

```rust
fn env_contains_var(&self, var: &str) -> bool {
    if GLOBALS_BUILTIN.contains_key(var) {
        return true;
    }
```

### The JavaScript plugin scope manager only adds the listed envs

`addGlobals` in `apps/oxlint/src-js/plugins/scope.ts:149-182` creates global variables for each serialized env
and never adds the builtin preset:

```ts
for (const envName in envs) {
  const preset = ENVS.get(envName);
  if (preset === undefined) continue;
  const { readonly, writable } = preset;
  for (let i = 0, len = readonly.length; i < len; i++) {
    const varName = readonly[i];
    if (!Object.hasOwn(globals, varName)) createGlobalVariable(varName, globalScope, false);
  }
```

The `es20xx` presets are difference lists over ES5.
`ENV_BUILTIN` (`apps/oxlint/src-js/generated/envs.ts:1337`) lists `Array` and `Object`;
`ENV_ES_2024` (`apps/oxlint/src-js/generated/envs.ts:1842-1874`) lists only later additions such as `Map`,
`Promise`,
and `Set`.
`isGlobalReference` (`apps/oxlint/src-js/plugins/scope.ts:334-360`)
requires a global-scope variable with no definitions:

```ts
const variable = globalScope.set.get(node.name);
// Global variables are not defined by any node, so they should have no definitions
if (variable === undefined || variable.defs.length > 0) return false;
```

With `builtin` absent,
`Array` never enters the global scope,
so the lookup returns `false`.

### The config reference misdescribes `builtin`

The environment list in the config reference
(<https://oxc.rs/docs/guide/usage/linter/config-file-reference.html>,
generated from `crates/oxc_linter/src/config/env.rs:17`) says:

```rust
/// - builtin - Latest ECMAScript globals, equivalent to es2026.
```

Measured on 1.85.0 with the probe in "Verification",
`"env": { "es2026": true }` alone leaves `Array` and `Object` out of the JavaScript plugin global scope,
while `"env": { "builtin": true }` alone includes them.
`builtin` covers ES5 plus later globals;
`es2026` is a difference list,
so the two are not equivalent,
and the reference does not mention that an explicit `env` drops the default `builtin`.

ESLint's eslintrc implementation always prepends the builtin env
(`Object.assign({ builtin: true }, config.env, envInFile)` in `lib/linter/linter.js`,
ESLint 9.39.0),
so the JavaScript side of oxlint diverges from both oxlint's Rust side and ESLint.

## Verification

Oxlint 1.85.0 (`node_modules/.pnpm/oxlint@1.85.0_oxlint-tsgolint@7.0.2002`),
oxc source at commit `f51e678` (2026-09-25).

Probe plugin reporting what the scope manager says:

```js
// probe-plugin.mjs
const rule = {
  create(context) {
    return {
      Identifier(node) {
        if (!['Array', 'Object', 'Set', 'Promise', 'Buffer',].includes(node.name,)) return;
        context.report({ node, message: `${node.name} global=${context.sourceCode.isGlobalReference(node,)}`, },);
      },
    };
  },
};
export default { meta: { name: 'probe', }, rules: { probe: rule, }, };
```

```ts
// probe.ts
export const a = Array.from([1,],);
export const o = Object.keys({},);
export const s = new Set([1,],);
export const p = Promise.all([],);
export const b = Buffer.concat([],);
```

```sh
oxlint --config env.json --format unix probe.ts
```

### Configs that keep `Array` and `Object` global

- No `env` key:
  `Array`,
  `Object`,
  `Set`,
  `Promise` global;
  `Buffer` not global (no Node env).
- `"env": { "builtin": true, "browser": true, "node": true, "es2026": true, … }`
  (the shared config after the fix):
  all five global.

### Configs that drop them

- `"env": { "es2026": true }`:
  `Array global=false`,
  `Object global=false`;
  `Set` and `Promise` global;
  `Buffer` not global.
  Compare `"env": { "builtin": true }`,
  where `Array` and `Object` are global.
- `"env": { "browser": true, "node": true, "es2024": true }`:
  `Array global=false`,
  `Object global=false`;
  `Set`,
  `Promise`,
  `Buffer` global.
- The shared config's `env` without `builtin`
  (`browser`,
  `node`,
  `es2026`,
  `serviceworker`,
  `webextensions`,
  `worker`):
  the same result.

## Verified workarounds

### Add `builtin: true` to every explicit `env`

`package/config/oxlint/src/config-base.ts` now declares `builtin: true` first in `env`;
`oxlint --init` writes the same default (`apps/oxlint/src/mode/init.rs:15`).
Tradeoff:
none observed for Rust rules,
which already treated builtin globals as enabled;
JavaScript plugin rules now see ES5 globals,
which is what their ESLint originals assume.

### Resolve globals by the absence of a file-local declaration

The spread rules ask the scope chain whether any declaration in the file binds the name
(`package/oxlint-plugin/no-restricted-syntax/src/rule/spread-evidence/undeclared-reference.ts`),
falling back from `isGlobalReference`.
Tradeoff:
an implicit global assigned elsewhere at runtime counts as a host global,
which is acceptable because such code has no file-visible binding for the rule to reason about.
The rule tests keep an `env` without `builtin` on purpose
(`package/test-fixture/oxlint-no-restricted-syntax/.oxlintrc.no-useless-spread.fixture.json`),
so this path stays exercised.

## What does not work

- Relying on `isGlobalReference` alone:
  correct only for configs that happen to include `builtin` or omit `env` entirely.
- Testing rules only with an `env`-free config:
  every spread-rule test passed while the shared config silently disabled the `Array` and `Object` checks.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` has no entry for oxlint or oxc lint behavior
(checked:
`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`).

Duplicate search:
oxc issues and pull requests,
open and closed,
for `isGlobalReference`,
`env builtin`,
`builtin globals`,
`globals Array env plugin`,
and `no-undef env Array` found no matching report.

1. **Upstream's fault:**
   yes.
   The JavaScript scope manager disagrees with oxlint's own Rust global lookup and with ESLint,
   and the config reference describes `builtin` as equivalent to `es2026`,
   which the probe disproves.
2. **Upstream can fix it:**
   yes;
   the prototype changes one loop header in `apps/oxlint/src-js/plugins/scope.ts`.
3. **Supported use case:**
   yes.
   `isGlobalReference` is a documented JavaScript plugin API ported from ESLint
   (`scope.ts:335` cites ESLint's `source-code.js`),
   and `env` is a documented config key.
4. **Contribution welcome:**
   yes,
   with disclosure.
   `CONTRIBUTING.md:12-21` ("AI Usage Policy") requires AI usage to be disclosed and reviewed;
   `AGENTS.md:7-15` repeats it;
   `.github/ISSUE_TEMPLATE/linter_bug_report.yaml` has no AI field.
   No ban was found.
5. **Likely to fix:**
   plausible.
   No won't-fix signal;
   globals resolution changed recently
   (pull request 25905,
   "resolve globals by reference,
    not by name",
   merged 2026-08-19).
6. **Minimal fix prototyped:**
   yes.
   In a disposable clone of `oxc-project/oxc` at `f51e67812ed15cea95c9bddfe614d889685f0703`:

   ```diff
   -  for (const envName in envs) {
   +  const enabledEnvs = Object.assign({ builtin: true }, envs);
   +  for (const envName in enabledEnvs) {
   ```

   Full hunk,
   including the updated comments:
   `apps/oxlint/src-js/plugins/scope.ts` lines 158 to 163 in the clone.
   Verified on a copy of the installed oxlint 1.85.0 package with the equivalent edit to `dist/lint.js:12877`
   (`let enabledEnvs = Object.assign({ builtin: !0 }, envs); for (let envName in enabledEnvs)`),
   running the probe with `"env": { "browser": true, "node": true, "es2024": true }`:
   before,
   `Array global=false`,
   `Object global=false`;
   after,
   `Array`,
   `Object`,
   `Set`,
   `Promise`,
   and `Buffer` all `global=true`.
   The env-free config still reports `Buffer global=false` after the patch.
   Gap:
   the upstream vitest fixtures `apps/oxlint/test/fixtures/globals`
   and `sourceCode_scope_methods` need the napi test build
   and were not run;
   their snapshots may change.

All six hold,
so the draft is fileable once the person filing re-runs the reproduction and fills in the disclosure bracket,
which `CONTRIBUTING.md` requires to name human-verified checks.
Posting it is an external action that needs the user's approval.

### Draft issue

~~~md
Title: linter(plugins): `sourceCode.isGlobalReference` returns false for `Array`/`Object` when config sets `env` without `builtin`

Template: `.github/ISSUE_TEMPLATE/linter_bug_report.yaml` (fill its version, config, and reproduction fields
from the sections below)

### Description

With a config whose root `env` is set but does not list `builtin`, JS plugin rules see no ES5 globals:
`context.sourceCode.isGlobalReference(node)` returns `false` for `Array` and `Object`, while `Set`, `Promise`
(and `Buffer` with `node`) return `true`. Rust rules are unaffected.

Cause:

- An explicit root `env` replaces the default `{ builtin: true }` (`crates/oxc_linter/src/config/env.rs:65-71`,
  `Oxlintrc` is `#[serde(default)]`; `env.rs:111` asserts `!env.contains("builtin")`).
- The Rust side always treats builtin globals as enabled (`crates/oxc_linter/src/context/mod.rs:206-210`, `:248-251`).
- `addGlobals` in `apps/oxlint/src-js/plugins/scope.ts:149-182` only iterates the serialized `envs`, and the
  `es20xx` presets in `src-js/generated/envs.ts` are post-ES5 difference lists, so `Array`/`Object` never enter the
  global scope and `isGlobalReference` (`scope.ts:334-360`) returns `false`.
- ESLint always prepends builtin: `Object.assign({ builtin: true }, config.env, envInFile)` (`lib/linter/linter.js`).

The env docs also say "builtin - Latest ECMAScript globals, equivalent to es2026." (`env.rs:17`, rendered in the
config reference), but `env: { es2026: true }` alone omits `Array`/`Object` in JS plugin scope, while
`env: { builtin: true }` alone includes them.

### Reproduction (oxlint 1.85.0)

```js
// probe-plugin.mjs
const rule = { create(context) { return { Identifier(node) {
  if (!['Array', 'Object', 'Set'].includes(node.name)) return;
  context.report({ node, message: `${node.name} global=${context.sourceCode.isGlobalReference(node)}` });
} }; } };
export default { meta: { name: 'probe' }, rules: { probe: rule } };
```

```json
{ "plugins": [], "categories": { "correctness": "off" }, "env": { "es2026": true },
  "jsPlugins": ["./probe-plugin.mjs"], "rules": { "probe/probe": "error" } }
```

```ts
// probe.ts
export const a = Array.from([1]);
export const o = Object.keys({});
export const s = new Set([1]);
```

`oxlint --format unix probe.ts` prints `Array global=false`, `Object global=false`, `Set global=true`.
With `"env": { "builtin": true }` all three are `true`.

### Suggested fix

Always enable `builtin` in `addGlobals`, matching the Rust side and ESLint:

```diff
-  for (const envName in envs) {
+  const enabledEnvs = Object.assign({ builtin: true }, envs);
+  for (const envName in enabledEnvs) {
```

Verified by applying the equivalent change to the bundled `dist/lint.js` of oxlint 1.85.0: the probe then reports
`Array`/`Object` as global under every `env`. Not yet run: `apps/oxlint/test/fixtures/globals` and
`sourceCode_scope_methods` snapshots. Separately, the `builtin` line in `env.rs:17` could say "ES5 and later globals;
always enabled for Rust rules" and note that an explicit `env` replaces the default.

AI disclosure: investigation, source trace, and prototype were done with an AI coding assistant.
[Filer: re-run the reproduction and the patched-bundle probe yourself, then replace this bracket with what you
verified.]
~~~
