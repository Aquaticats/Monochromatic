# Oxlint 1.75 does not supply TypeScript type information to JavaScript plugins,
so custom rules need an independent semantic bridge

## Supersession

The Oxlint host limitation remains current,
but retaining the native readonly rule is no longer the repository remedy.
The project-owned rule now loads TypeScript 7's synchronous unstable API as an independent semantic bridge,
and shared configuration disables the retired native rule.

## Symptom

A custom Oxlint JavaScript rule can visit a TypeScript parameter and inspect its annotation syntax,
but it cannot resolve the annotation to a TypeScript type.
Running Oxlint with `--type-aware` does not add a type checker or TypeScript parser services to the rule context.

A probe rule reports:

```text
error type-probe(probe): parserServices keys=; count=0
```

This prevents a JavaScript replacement for
`typescript/prefer-readonly-parameter-types` from determining the following facts through the Oxlint host API alone:

- whether a named alias resolves to a mutable object;
- whether a type originates in a library or package;
- whether a generic instantiation is deeply readonly;
- whether a method-bearing type has mutators;
- whether a nested branded primitive is immutable.

The rule can still inspect syntax,
scopes,
references,
and function-body operations.
It can also import independent Node.
js libraries,
so absent Oxlint parser services do not prove that type-aware replacement semantics are impossible.

## Root cause

Originally verified against `oxc-project/oxc` commit `8de6fcaac7037d37e7f971e67a474b3ae442513a`,
cloned from `https://github.com/oxc-project/oxc.git` on 2026-07-12.
Reverified against tag `apps_v1.75.0`,
commit `83abe3b49c0913b1a984a7eec5e433a59fd76eae`,
on 2026-08-07.
The source paths and quoted behavior remain unchanged,
and the current official JavaScript-plugin documentation still lists
"Lint rules that rely on TypeScript type-awareness" as unsupported.

The JavaScript plugin `SourceCode` object explicitly installs one frozen empty parser-services object.
`apps/oxlint/src-js/plugins/source_code.ts:229-234`:

```typescript
/**
 * Parser services for the file.
 *
 * Oxlint does not offer any parser services.
 */
parserServices: Object.freeze({} as Record<string, unknown>),
```

The type-aware engine is constructed as a separate tsgolint state rather than as services attached to the regular
linter.
`crates/oxc_linter/src/lint_runner.rs:202-209`:

```rust
pub fn build(self) -> Result<LintRunner, String> {
    let directives_coordinator = DirectivesStore::new();

    let type_aware_linter = if self.type_aware_enabled {
        match TsGoLintState::try_new(
            self.lint_service_options.cwd(),
            self.regular_linter.config.clone(),
            self.fix_kind,
```

When files are linted,
the regular lint service runs first.
The separate type-aware linter runs afterward and contributes diagnostics,
not a type service for JavaScript rules.
`crates/oxc_linter/src/lint_runner.rs:253-268`:

```rust
if self.type_check_only {
    self.lint_service.collect_parse_diagnostics(fs, files.to_owned(), &tx_error);
} else {
    self.lint_service.run::<TIMINGS>(
        fs,
        files.to_owned(),
        &tx_error,
        diff_manager,
        rule_timing_store,
    );
}

if let Some(type_aware_linter) = self.type_aware_linter.take() {
    type_aware_linter.lint(
```

The JavaScript plugin host therefore receives the regular ESTree,
source APIs,
and scope analysis,
while tsgolint owns the TypeScript type analysis on another execution path.
The `--type-aware` flag enables that second path but does not change the JavaScript rule context.

## Verification

Versions under test:

- original probe:
  Oxlint 1.73.0,
  `@oxlint/plugins` 1.73.0,
  and `oxlint-tsgolint` 0.24.0;
- current revalidation:
  Oxlint 1.75.0,
  `@oxlint/plugins` 1.75.0,
  and `oxlint-tsgolint` 7.0.2001;
- Node.
  js through the repository's mise environment.

The current revalidation used the same plugin,
configuration,
and TypeScript input in a disposable directory.
It supplied the installed platform `tsgolint` binary through `OXLINT_TSGOLINT_PATH`
because dependency lookup starts from the disposable working directory.
Oxlint 1.75.0 produced the same output:

```text
probe.ts:1:1: error type-probe(probe): parserServices keys=; count=0
```

The following disposable harness creates a plugin,
config,
and TypeScript input:

```bash
probe_dir=$(mktemp --directory)

printf '%s\n' \
  "export default { meta: { name: 'type-probe' }, rules: { probe: { create(context) { return { Program(node) { const services = context.sourceCode.parserServices; context.report({ node, message: 'parserServices keys=' + Object.keys(services).join(',') + '; count=' + Object.keys(services).length }); } }; } } } };" \
  > "$probe_dir/plugin.mjs"

printf '%s\n' \
  '{"jsPlugins":["./plugin.mjs"],"rules":{"type-probe/probe":"error"}}' \
  > "$probe_dir/.oxlintrc.json"

printf '%s\n' \
  'export type Mutable = { value: string };' \
  'export function probe(input: Mutable): void { void input; }' \
  > "$probe_dir/probe.ts"

node_modules/.bin/oxlint \
  --type-aware \
  --config "$probe_dir/.oxlintrc.json" \
  "$probe_dir/probe.ts"
```

Observed output:

```text
/tmp/tmp.eoWjjJfbNk/probe.ts:1:1: error type-probe(probe): parserServices keys=; count=0
```

### Available patterns

These inputs are available to a JavaScript rule:

- TypeScript annotation AST nodes;
- parameter bindings and lexical references;
- assignments,
  updates,
  deletes,
  member accesses,
  and call syntax in the function body;
- source text,
  tokens,
  comments,
  and locations;
- Oxlint scope analysis.

### Unavailable patterns

These remain unavailable even with `--type-aware`:

- a TypeScript `Program` or `TypeChecker` supplied by Oxlint;
- populated `parserServices`;
- symbol and alias resolution;
- inferred types;
- package provenance for resolved declarations;
- structural readonly analysis across declarations.

## Verified workarounds

### Use a syntax or body-analysis contract

Implement the project rule against information the JavaScript plugin actually receives.
For example,
the rule can require selected readonly annotation syntax,
or it can report writes through parameter bindings.

Tradeoff:
this is a new policy,
not a drop-in implementation of deep type readonlyness.
Named aliases and method semantics remain unknown unless the rule adopts explicit syntax conventions or heuristic lists.

### Retain the native type-aware rule

Keep `typescript/prefer-readonly-parameter-types` for code that requires deep structural type analysis.

Tradeoff:
this retains the allow-list,
false-positive,
and external-boundary maintenance that motivated replacement.

### Load an independent semantic pipeline inside the plugin

A JavaScript plugin can import ordinary Node.
js dependencies.
Candidate pipelines include a TypeScript `Program` and `TypeChecker`,
`oxc-parser` paired with TypeScript semantics,
Yuku's parser and analyzer,
Oxc isolated declaration generation,
and the declaration pipeline exposed by `rolldown-plugin-dts`.

Tradeoff:
none of these candidates receives the already-running tsgolint checker.
Each needs proof for semantic depth,
Oxlint-node mapping,
project and package resolution,
cache invalidation,
editor lifecycle,
and consumer-boundary behavior.
Parser or declaration throughput alone does not prove that a candidate can answer deep readonly queries.
The repository selected TypeScript 7's synchronous unstable API after the separate technology audit in
`doc/audit/tech-readonly-parameter-semantic-bridge-vet-2026-07-12.md`.

### Persist semantic summaries outside one plugin process

Oxlint's JavaScript host keeps module state while linting files in one process.
At Oxc commit `8de6fcaac7037d37e7f971e67a474b3ae442513a`,
`apps/oxlint/src-js/plugins/lint.ts` defines module-level `buffers` and `afterHooks` stores and explicitly reuses the
hook array for every file.
`apps/oxlint/src/run.rs` creates the external JavaScript linter during each CLI process startup.
A later CLI invocation therefore cannot reuse ordinary JavaScript module state from the prior process.

The semantic rule uses both scopes deliberately:

- a process-local final-index cache reuses completed fixed-point propagation across unchanged files in one stable
  Oxlint input snapshot;
- content-addressed data-only JSON persists direct summaries across processes;
- the persistent identity includes analyzer code,
  TypeScript version,
  compiler options,
  complete project source and declaration contents,
  governing lockfile,
  source path,
  and source text;
- complete nested validation,
  entry-size limits,
  atomic publication,
  and periodic age/count/byte eviction turn corrupt or stale data into misses.

`closeSemanticBridge()` closes external implementation projects and clears every process-local effect,
lockfile,
and final-index cache.
This lifecycle boundary prevents a later invocation in the same host process from inheriting prior snapshot state.

Package tests run the built analyzer in independent Node processes and prove that the second process reads persistent
summaries without rebuilding direct summaries.
Separate tests change an imported implementation while leaving the caller unchanged and corrupt a nested payload;
both cases miss conservatively.
This is a project-side performance workaround,
not an Oxlint cache API.

## What does not work

### Pass `--type-aware`

The verification harness passes this flag and still observes zero parser-service keys.
The flag starts the separate tsgolint path shown in `lint_runner.rs`.

### Read `context.sourceCode.parserServices`

The property exists for ESLint API compatibility,
but Oxlint initializes it to a frozen empty object.
Existence of the property does not imply TypeScript parser services.

### Add TypeScript types to the plugin package

Type declarations can describe an API only if the runtime supplies it.
Changing local `Context` typings would not populate parser services or connect the JavaScript host to tsgolint.

### Port the native rule line for line using only the Oxlint ESTree visitor

The visitor can reproduce annotation-shape checks,
but the Oxlint context has no equivalent input for calls that require resolved symbols,
properties,
generics,
or package origins.
A direct port that adds no independent semantic bridge would either fail or replace semantic checks with undocumented
guesses.

## Upstream filing decision

The exact feature request already exists as
[`oxc-project/oxc#19596`](https://github.com/oxc-project/oxc/issues/19596),
"linter:
 make type information available to custom JS plugins.
"
The issue is open.
On 2026-06-12 its assignee reported that there was no update.
An Oxc maintainer previously explained that tsgo does not expose type information for JavaScript plugins and that
crossing the Go-to-JavaScript boundary without losing performance is difficult.

Searches of open issues and open and closed pull requests for
`type information JS plugins` found no separate implementation pull request.
The exact issue contains the current upstream state,
so there is no additive comment to post.

The repository's `CONTRIBUTING.md:12-22` permits AI-assisted contributions with disclosure,
review,
understanding,
and validation.
No upstream communication is proposed here.

The filing constraints resolve as follows:

1.  **Upstream fault:
    ** no.
    Current documentation explicitly lists type-aware JavaScript rules as unsupported.
    This is a known feature gap,
    not behavior contradicting the supported contract.
2.  **Upstream can fix it:
    ** yes in principle.
    The open issue tracks that work,
    while maintainer comments identify architecture and performance constraints.
3.  **Supported use case:
    ** no at present.
    Oxlint's JavaScript plugin documentation lists rules relying on TypeScript type awareness under unsupported APIs.
4.  **Contribution welcome:
    ** conditional yes.
    Oxc welcomes contributions and permits reviewed,
    disclosed AI assistance,
    but no patch exists to submit.
5.  **Likely upstream action:
    ** unresolved.
    The issue remains open and assigned,
    with no update as of 2026-06-12.
6.  **Compatible minimal fix prototyped:
    ** no.
    A bridge between tsgo and JavaScript plugin contexts is upstream architecture work.
    Project-side independent semantic pipelines remain a separate investigation and do not constitute an upstream fix.

Because the behavior is documented as unsupported,
constraint 1 fails and no upstream prototype or new filing is warranted.
The retained filing artifact is an explicit nothing-to-add note for issue 19596.
