# Oxlint 1.71 JS plugin visitors are synchronous, so awaited visitor classification reports after file context reset

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

A custom oxlint JS rule sometimes needs data from a file other than the file currently being linted.
In this repo,
`package/oxlint-plugin/no-restricted-syntax/src/rule/prefer-describe-function-ref-name.ts`
classifies relative imports to decide whether a `describe({ name: 'x' })` string should become
`describe({ name: x.name })`.

The tempting implementation is to make the visitor `async` and use `fs.promises.readFile`.
That does not work.
Oxlint does not await visitor return values.
A diagnostic reported after an `await` runs after oxlint has reset the per-file context,
which produces this error:

```text
Error: Cannot report errors in `createOnce`
    at report (.../oxlint/dist/lint.js:13877:31)
    at Object.value [as report] (.../oxlint/dist/lint.js:14400:5)
    at Program (/tmp/agent/oxlint-js-plugin-sync-probe-gXBdqD/plugin.mjs:9:21)
```

A synchronous visitor reports normally.
The failure is specifically the asynchronous continuation.

## Root cause

Source clone:
`/tmp/agent/oxc-js-plugin-sync-20260629`,
origin `https://github.com/oxc-project/oxc.git`,
commit `da0e5bf6687b4bc5f376898f2d59832c6419ce15`.

The call chain is synchronous from Rust into JS and back.

`crates/oxc_linter/src/lib.rs:805-814` calls the external linter callback and immediately waits for a
`Result`:

```rust
// crates/oxc_linter/src/lib.rs:805-814
// Pass AST and rule IDs + options IDs to JS
let result = (external_linter.lint_file)(
    path_string.to_owned(),
    external_rules.iter().map(|(rule_id, _, _)| rule_id.raw()).collect(),
    external_rules.iter().map(|(_, options_id, _)| options_id.raw()).collect(),
    settings_json,
    globals_json,
    self.workspace_uri.as_ref().map(ToString::to_string),
    allocator,
);
```

The wrapper explicitly distinguishes plugin loading from linting.
`loadPlugin` is async,
but `lintFile` is not.
`apps/oxlint/src/js_plugins/external_linter.rs:148-156` says:

```rust
// apps/oxlint/src/js_plugins/external_linter.rs:148-156
/// Wrap `lintFile` JS callback as a normal Rust function.
///
/// The returned function creates a `Uint8Array` referencing the memory of the given `Allocator`,
/// and passes it to JS side,
/// unless the `Allocator`'s buffer has already been sent to JS.
///
/// Unlike `loadPlugin`,
/// `lintFile` JS callback is not async.
/// But `ThreadsafeFunction` executes the callback
/// on main JS thread,
/// and therefore it may have to wait for a previous `lintFile` call to complete.
/// Use an `mpsc::channel` to wait for the result from JS side,
/// and block current thread until `lintFile`
/// completes execution.
```

`apps/oxlint/src/js_plugins/external_linter.rs:177-205` schedules the JS callback and then blocks on
`rx.recv()` for the callback return value:

```rust
// apps/oxlint/src/js_plugins/external_linter.rs:177-205
let status = cb.call_with_return_value(
    FnArgs::from((
        file_path,
        buffer_id,
        buffer,
        rule_ids,
        options_ids,
        settings_json,
        globals_json,
        workspace_uri,
    )),
    ThreadsafeFunctionCallMode::NonBlocking,
    move |result, _env| {
        let res = tx.send(result);
        debug_assert!(res.is_ok(), "Failed to send result of `lintFile`");
        Ok(())
    },
);

if status == Status::Ok {
    match rx.recv() {
        Ok(Ok(None)) => Ok(Vec::new()),
        Ok(Ok(Some(json))) => {
```

On the JS side,
`lintFile` is typed and implemented as a synchronous function returning `string | null`.
`apps/oxlint/src-js/plugins/lint.ts:57-66`:

```typescript
// apps/oxlint/src-js/plugins/lint.ts:57-66
export function lintFile(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  workspaceUri: string | null,
): string | null {
```

`lintFileImpl` also returns synchronously.
It calls each rule's `create` or `before` hook directly,
then walks the AST directly.
`apps/oxlint/src-js/plugins/lint.ts:214-249`:

```typescript
// apps/oxlint/src-js/plugins/lint.ts:214-249
let { visitor } = ruleDetails;
if (visitor === null) {
  // Rule defined with `create` method
  debugAssertIsNonNull(ruleDetails.rule.create);
  visitor = ruleDetails.rule.create(ruleDetails.context);
} else {
  // Rule defined with `createOnce` method
  const { beforeHook, afterHook } = ruleDetails;
  if (beforeHook !== null) {
    // If `before` hook returns `false`, skip this rule
    const shouldRun = beforeHook();
    if (shouldRun === false) continue;
  }
  // Note: If `before` hook returned `false`, `after` hook is not called
  if (afterHook !== null) afterHooks.push(afterHook);
}

addVisitorToCompiled(visitor);
```

```typescript
// apps/oxlint/src-js/plugins/lint.ts:240-249
if (visitorState !== VISITOR_EMPTY) {
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);

  debugAssert(ancestors.length === 0, "`ancestors` should be empty before walking AST");

  if (visitorState === VISITOR_CFG) {
    walkProgramWithCfg(ast, compiledVisitor);
  } else {
    walkProgram(ast, compiledVisitor as (VisitFn | EnterExit | null)[]);
```

The public plugin types reinforce that shape.
`apps/oxlint/src-js/generated/visitor.d.ts:12-14` gives visitor methods a `void` return:

```typescript
// apps/oxlint/src-js/generated/visitor.d.ts:12-14
interface StrictVisitorObject {
  DebuggerStatement?: (node: ESTree.DebuggerStatement) => void;
  "DebuggerStatement:exit"?: (node: ESTree.DebuggerStatement) => void;
```

`apps/oxlint/src-js/plugins/types.ts:8-18` gives hooks synchronous return types only:

```typescript
// apps/oxlint/src-js/plugins/types.ts:8-18
// Hook function that runs before traversal.
// If returns `false`, traversal is skipped for the rule.
export type BeforeHook = () => boolean | void;

// Hook function that runs after traversal.
export type AfterHook = () => void;

// Visitor object returned by a `Rule`'s `createOnce` function.
export type VisitorWithHooks = Visitor & {
  before?: BeforeHook;
  after?: AfterHook;
```

The per-file context is intentionally a singleton reused across files.
`apps/oxlint/src-js/plugins/context.ts:11-19` says no new contexts are created after plugin loading:

```typescript
// apps/oxlint/src-js/plugins/context.ts:11-19
 * The difference is that we don't create new file context and rule context objects for each file, but instead reuse
 * the same objects over and over. After plugin loading is complete, no further `Context` objects are created.
 * This reduces pressure on garbage collector, and is required to support `createOnce` API.
 *
 * ## Rule context
 *
 * Each rule has its own `Context` object. It is passed to that rule's `create` and `createOnce` functions.
 * `Context` objects are created during plugin loading for each rule.
 * For each file, the same `Context` object is reused over and over.
```

When no file is active,
file-specific getters throw the same `createOnce` error.
`apps/oxlint/src-js/plugins/context.ts:329-397` shows `filename`,
`cwd`,
and `sourceCode` all guard on `filePath === null`:

```typescript
// apps/oxlint/src-js/plugins/context.ts:329-397
get filename(): string {
  // Note: If we change this implementation, also change `getFilename` method below
  if (filePath === null) throw new Error("Cannot access `context.filename` in `createOnce`");
  return filePath;
},
```

```typescript
// apps/oxlint/src-js/plugins/context.ts:391-397
/**
 * Source code of the file being linted.
 */
get sourceCode(): SourceCode {
  // Note: If we change this implementation, also change `getSourceCode` method below
  if (filePath === null) throw new Error("Cannot access `context.sourceCode` in `createOnce`");
  return SOURCE_CODE;
```

`apps/oxlint/src-js/plugins/report.ts:86-99` uses the same sentinel in `context.report`:

```typescript
// apps/oxlint/src-js/plugins/report.ts:86-99
export function report(
  diagnostic: Diagnostic,
  extraArgs: unknown[],
  ruleDetails: RuleDetails,
): void {
  if (filePath === null) throw new Error("Cannot report errors in `createOnce`");
```

So an `async` visitor is not awaited.
Its continuation runs after `lintFileImpl` returns,
after `resetFile()` has made `filePath` null,
and any late `context.report` throws the `createOnce` error.

There is also no oxlint-provided cross-file parser or resolver service for JS plugins.
The current `SourceCode` only exposes the current file's text.
`apps/oxlint/src-js/plugins/source_code.ts:229-234` says parser services are empty:

```typescript
// apps/oxlint/src-js/plugins/source_code.ts:229-234
/**
 * Parser services for the file.
 *
 * Oxlint does not offer any parser services.
 */
parserServices: Object.freeze({} as Record<string, unknown>),
```

`apps/oxlint/src-js/plugins/source_code.ts:261-269` shows `getText()` reads only the current source text:

```typescript
// apps/oxlint/src-js/plugins/source_code.ts:261-269
/**
 * Get the source code for the given node.
 * @param node? - The AST node to get the text for.
 * @param beforeCount? - The number of characters before the node to retrieve.
 * @param afterCount? - The number of characters after the node to retrieve.
 * @returns Source text representing the AST node.
 */
getText(node?: Ranged | null, beforeCount?: number | null, afterCount?: number | null): string {
  if (sourceText === null) initSourceText();
```

## Verification

Versions under test:

- Installed oxlint:
  `1.71.0`.
  Verified with `/var/home/user/Monochromatic/node_modules/.bin/oxlint --version`.
- Source trace:
  Oxc commit `da0e5bf6687b4bc5f376898f2d59832c6419ce15`.
- Probe directory:
  `/tmp/agent/oxlint-js-plugin-sync-probe-gXBdqD`.

The probe plugin had two rules.
The `sync` rule reports in `Program` before returning.
The `awaited` rule awaits one resolved promise before reporting:

```javascript
// /tmp/agent/oxlint-js-plugin-sync-probe-gXBdqD/plugin.mjs
export default {
  meta: { name: 'sync-probe' },
  rules: {
    awaited: {
      create(context) {
        return {
          async Program(node) {
            await Promise.resolve();
            context.report({ node, message: 'reported after await' });
          },
        };
      },
    },
    sync: {
      create(context) {
        return {
          Program(node) {
            context.report({ node, message: 'reported synchronously' });
          },
        };
      },
    },
  },
};
```

The config enabled both rules:

```json
// /tmp/agent/oxlint-js-plugin-sync-probe-gXBdqD/.oxlintrc.json
{
  "jsPlugins": ["./plugin.mjs"],
  "rules": {
    "sync-probe/awaited": "error",
    "sync-probe/sync": "error"
  }
}
```

The task ran oxlint against one input file:

```toml
# /tmp/agent/oxlint-js-plugin-sync-probe-gXBdqD/mise.toml
[tasks.probe]
run = "/var/home/user/Monochromatic/node_modules/.bin/oxlint --config .oxlintrc.json input.js"
```

Command:

```bash
cd /tmp/agent/oxlint-js-plugin-sync-probe-gXBdqD
mise run probe
```

Observed output:

```text
input.js:1:7: warning eslint(no-unused-vars): Variable 'value' is declared but never used. Unused variables should start with a '_'. help: Consider removing this declaration.
input.js:1:1: error sync-probe(sync): reported synchronously
file:///var/home/user/Monochromatic/node_modules/.pnpm/oxlint@1.71.0_oxlint-tsgolint@0.23.0/node_modules/oxlint/dist/lint.js:13877
	if (filePath === null) throw Error("Cannot report errors in `createOnce`");
	                             ^

Error: Cannot report errors in `createOnce`
    at report (file:///var/home/user/Monochromatic/node_modules/.pnpm/oxlint@1.71.0_oxlint-tsgolint@0.23.0/node_modules/oxlint/dist/lint.js:13877:31)
    at Object.value [as report] (file:///var/home/user/Monochromatic/node_modules/.pnpm/oxlint@1.71.0_oxlint-tsgolint@0.23.0/node_modules/oxlint/dist/lint.js:14400:5)
    at Program (file:///tmp/agent/oxlint-js-plugin-sync-probe-gXBdqD/plugin.mjs:9:21)
```

A second probe with only the synchronous rule reports normally and does not throw the late-context error:

```text
input.js:1:7: warning eslint(no-unused-vars): Variable 'value' is declared but never used. Unused variables should start with a '_'. help: Consider removing this declaration.
input.js:1:1: error sync-probe(sync): reported synchronously
```

Working patterns:

- Synchronous visitor logic.
- Synchronous `before` hook logic returning `false` or `void`.
- Synchronous reads isolated behind a helper and cached per path.

Failing patterns:

- `async Program(...)` that reports after an `await`.
- `async before()` whose result would need to decide whether to skip a rule.
- `fs.promises.readFile()` inside a visitor when the result affects diagnostics.

## Verified workarounds

### Keep the visitor synchronous and cache synchronous reads

Use `readFileSync` only inside a tiny helper,
then memoize by absolute path:

```typescript
// package/oxlint-plugin/no-restricted-syntax/src/rule/prefer-describe-function-ref-name.ts
const sourceTextByPath = new Map<string, string>();

function readSourceTextOrEmpty(sourcePath: string,): string {
  const cached = sourceTextByPath.get(sourcePath,);
  if (cached !== undefined)
    return cached;
  try {
    // oxlint-disable-next-line node/no-sync -- sync oxlint visitor; classification must finish before report.
    const content = readFileSync(
      sourcePath,
      'utf8',
    );
    sourceTextByPath.set(sourcePath, content,);
    return content;
  }
  catch (readError: unknown) {
    void readError;
    sourceTextByPath.set(sourcePath, '',);
    return '';
  }
}
```

Tradeoffs:

- The helper keeps one lint run's first read result.
  If a source file appears or changes after its first lookup in the same oxlint process,
  the rule keeps using the old value.
- Failed reads are cached as empty strings.
  That avoids repeated misses,
  but it also means a file created later in the same lint process remains invisible.
- This is still a heuristic.
  It does not follow re-exports or workspace package resolution.

### Move cross-file analysis outside oxlint

Generate a manifest before running oxlint,
then have the rule read that manifest synchronously.

Tradeoffs:

- Adds a build step and invalidation surface.
- The manifest must be kept in sync with the same file set oxlint sees.
- It is heavier than the per-path cache for a narrow rule.

### Write a native Rust oxlint rule

A native rule could use oxlint's internal data structures or a different preprocessing strategy.

Tradeoffs:

- Requires upstream or forked oxlint work.
- Does not help this repo's local JS plugin package immediately.

## What does not work

- Replacing `readFileSync` with `fs.promises.readFile` inside the visitor:
  the visitor returns a promise,
  oxlint ignores that return value,
  and any later `context.report` sees reset file context.
- Returning a promise from `before` or `after`:
  the source type is `() => boolean | void` or `() => void`,
  and the runtime calls the hook without awaiting it.
- Using `context.sourceCode.getText()` for the imported file:
  it only exposes the current file's source text.
- Using parser services or type-aware services from JS plugins:
  `parserServices` is an empty object in oxlint 1.71.

## Upstream filing decision

Checked `.out-of-scope/`:
no Oxc or oxlint exemption matched.

Duplicate search:
`gh search issues --repo oxc-project/oxc "oxlint JS plugin async visitor" --limit 10`,
`gh search issues --repo oxc-project/oxc "Cannot report errors in createOnce async" --limit 10`,
`gh search prs --repo oxc-project/oxc "createOnce async visitor" --limit 10`,
and `gh search issues --repo oxc-project/oxc "JS plugins type-awareness" --limit 10`
returned no matching issues or PRs.

Constraint check:

### Is it really upstream's fault?

No.
The evidence above shows a synchronous API boundary,
not a bug.
Oxlint advertises ESLint plugin compatibility,
and ESLint-style visitors are synchronous.

### Can upstream fix it?

They could add a new async JS plugin mode,
but that would be a feature request and likely a large runtime contract change.

### Are they supporting this use case?

Partly.
They support JS plugins,
but the public docs say JS plugin rules that rely on TypeScript type-awareness are not supported,
and the source shows no parser services or cross-file resolver service.

### Would the repo welcome our contribution?

`CONTRIBUTING.md` says contributions are welcome and AI use is allowed with disclosure,
but low-quality or unreviewed AI content is closed immediately.
`.github/ISSUE_TEMPLATE/linter_bug_report.yaml` asks for version,
command,
config,
and what happened.

### Will they likely fix it?

Unknown for a feature request,
but this is not a demonstrated bug.

### Have we prototyped a minimal fix compatible with their architecture?

No.
Because constraint one fails,
there is no upstream bug fix to prototype.
The consumer-side workaround is implemented in this repo instead.

Decision:
do not file upstream.
There is no additive bug report;
this doc records the architectural boundary and the local workaround.
