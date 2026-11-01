# Unbash 5 command analysis fails in the legacy shell adapter

## Symptom

The maintained auto-mode type check failed before migration integration changes:

```sh
# Repository root
mise run //package/pi-plugin/auto-mode:lint:types
```

TypeScript reported `TS2305` for the removed `Node` export,
`TS2551` for `Redirect.fileDescriptor`,
and additional errors involving redirect operators,
assignment arrays,
parameter operations,
and expansion `inner` fields.
The diagnostics originate in `package/agent-harness-shared/shell-command-analyzer/src/`.

The existing built analyzer reproduced the runtime failure:

```text
TypeError: node.redirects is not iterable
```

It occurred in the simple-command,
pipeline,
redirection,
substitution,
background-statement,
function,
and loop tests.
The failure is tracked in [repository issue #593](https://github.com/Aquaticat/Monochromatic/issues/593).
It is an adapter mismatch,
not evidence that Bash input must avoid these constructs.

## Root cause

Installed `unbash` is `5.0.0`.
The inspected upstream source is tag `5.0.0`,
commit `558bceb7216b95ff8e9fd6fa36bcef07c427f4ed`.
The clone was read only;
no upstream scripts or patches were run.

### Statements no longer contain redirects

In upstream `src/parser.ts:463`,
`makeStatement` creates a statement without a `redirects` member:

```ts
// unbash/src/parser.ts, tag 5.0.0
return {
  type: "Statement",
  pos: command.pos,
  end: command.end,
  command,
  background: undefined,
};
```

The pre-repair adapter spread `node.redirects` in its statement visitor.
The runtime failure was reproduced through the package's built artifact in `proc_9973`.
The source-level type failures in `proc_4b07` independently identify the obsolete API assumptions.

Upstream `src/parser.ts:538` instead wraps compound-command redirects:

```ts
// unbash/src/parser.ts, tag 5.0.0
private withRedirects(command: CompoundCommand): CompoundCommand | Redirected {
  const redirects = this.collectTrailingRedirects();
  if (redirects.length === 0) return command;
  return { type: "Redirected", pos: command.pos, end: redirects[redirects.length - 1].end, command, redirects };
}
```

The adapter must visit that wrapper and retain its redirects.
Treating the failure as a malformed user command would misidentify the affected input.

### Other fields changed together

Upstream `src/types.ts:334` defines mixed prefix/suffix elements
and a separate argument projection:

```ts
// unbash/src/types.ts, tag 5.0.0
readonly prefix: readonly (Assignment | Redirection)[];
readonly suffix: readonly (CommandArgument | Redirection)[];
readonly args: readonly CommandArgument[];
readonly redirects: readonly Redirection[];
```

Using all suffix elements as words or all prefix elements as assignments is no longer valid.
`CommandArgument` can itself be an assignment received by a declaration builtin.

Upstream `src/types.ts:585` exports `SyntaxNode`,
not the old `Node` name.
Installed `dist/types.d.ts:194` to `267` also declares:

```ts
// unbash/dist/types.d.ts, version 5.0.0, selected members
readonly value: Word | ArrayValue;
readonly elements: readonly Word[];
export type Redirection = Redirect | HereString | HereDoc;
readonly descriptor: RedirectDescriptor | undefined;
readonly delimiter: HereDocDelimiter | undefined;
readonly body: HereDocBody;
```

Parameter expansions use the discriminated `operation` member
(`dist/types.d.ts:50` to `107`).
Command,
process,
and arithmetic command substitutions expose their parser-owned `script`
(`dist/types.d.ts:108` to `129` and `185` to `191`).
Reusing those scripts preserves the parser's source ownership and positions;
there is no `inner` field to reparse.

## Verification

The original analyzer tests and the added `ast-v5.unit.test.ts` both failed before repair.
The added tests were committed as `7296b5f899` before implementation.
Consumer repair `bac94fb9a6` passed the analyzer's type check.
A fresh package build followed by its unit tests passed in `proc_45e2`.
After type-assertion removal and artifact-import correction,
`proc_1f18` reported zero lint warnings and errors.
`proc_7a77` passed analyzer and auto-mode type checks.
`proc_68f4` rebuilt the analyzer,
passed all analyzer tests including the added assignment/parameter branches,
and rebuilt auto-mode.
`proc_864e` passed the complete auto-mode unit task inside a restricted cgroup.
The shared cgroup limits were verified as two CPUs,
2 GiB memory,
no swap,
and 256 tasks.
`proc_c98d` rebuilt terminal-title,
passed its tests through the corrected built-artifact imports,
and passed its type check.
Guardrail's rebuilt package and existing source-level tests passed `proc_9e6c`.
`proc_d059` invoked its actual bundled registered handler against eight shell forms,
using a fixture host API without executing those shell commands.
`proc_ff61` passed the affected source-only Claude guardrail,
terminal-title,
and output-filter tests.
Those source modules are that package's declared runtime artifacts.

Run the affected package and consuming package checks:

```sh
# Repository root; separate commands preserve task boundaries
mise run //package/agent-harness-shared/shell-command-analyzer:lint:types
mise run //package/agent-harness-shared/shell-command-analyzer:lint:oxlint
mise run //package/agent-harness-shared/shell-command-analyzer:build
mise run //package/agent-harness-shared/shell-command-analyzer:test:unit
mise run //package/pi-plugin/auto-mode:lint:types
```

The verified initial post-repair catalog includes simple and quoted arguments,
pipelines,
file and descriptor redirections,
heredocs and here-strings,
command/process substitutions,
background commands,
function contexts,
literal loops,
and malformed-input diagnostics.
The added catalog includes redirected compounds,
`time` and negation wrappers,
implicit loop word lists,
allocated descriptor variables,
declaration assignments,
array/index substitutions,
parameter default/removal/replacement/slice operations,
and quoted heredoc bodies.

The failing pre-repair catalog has separate signals:

- Type check:
   removed exports and obsolete field/union assumptions.
- Built consumer:
   `node.redirects is not iterable` on ordinary statements.
- Package lint after the functional repair:
   formatting findings,
   an obsolete narrowing assertion,
   and an existing own-package `/ts` test import.
  These are not runtime parser failures.

## Consumer repair

Adapt the existing analyzer rather than changing dependencies or patching installed parser files.
The maintained implementation is in
`package/agent-harness-shared/shell-command-analyzer/src/`.

- Visit `Redirected`,
  `Time`,
  and `Negation` wrappers.
- Keep mixed command redirections separate from arguments and assignment prefixes.
- Preserve scalar/array assignment values,
  append/index syntax,
  and original source spellings.
- Traverse parameter-operation words and unquoted heredoc parts.
- Preserve parser-owned nested scripts and diagnostics without reparsing a removed field.
- Represent allocated descriptor variables separately from numeric `fileDescriptor` values.

No shell command is executed by these tests.
The analyzer still reports parsed structure rather than evaluating shell expansions.
`fileDescriptorVariable` is an additive public field;
a variable name is not represented as a numeric descriptor.
No lint rule or parse-failure guard is weakened.

## What does not establish a fix

A type-check pass alone does not verify traversal through the built consumer.
The repair therefore rebuilt the package before running its behavior tests.

The existing `bun-test.unit.test.ts` imported its own package through `/ts`.
The repository linter rejected that import with `test-import(require-eventual-artifact)`.
It now imports the package's built entry,
so source-only success cannot stand in for a rebuilt artifact.

The inspected failure did not justify downgrading `unbash`,
loosening types,
or modifying upstream code.
Those approaches were not used or claimed as verified workarounds.
The catalog minimum must match the version 5 API now consumed;
its lockfile was refreshed by pnpm,
not edited by hand.

### Lockfile refresh and unrelated generated metadata

The first refresh was stopped while mise downloaded unrelated tool provenance artifacts.
The narrower invocation uses its documented tool-bootstrap skip,
not a supply-chain policy exemption:

```sh
# Repository root, inside the session's restricted cgroup
MISE_TASK_RUN_AUTO_INSTALL=false mise --no-env --no-hooks run --skip-tools prepare:pnpm:install -- --lockfile-only --no-offline --ignore-scripts
```

An offline attempt reported `ERR_PNPM_NO_OFFLINE_META`:
the pnpm 12 metadata cache could not read the existing pnpm 11 mirror.
The online lockfile-only invocation passed supply-chain verification for all 692 entries
and completed with pnpm `12.11.2` in `proc_eb21`.
No trust-lockfile or release-age bypass was used.

The generated diff changes the `unbash` minimum to `>=5.0.0`
without changing its resolved `5.0.0` version.
It also adds deprecation metadata to 24 old Yuku bindings
and removes the optional `texture-compressor` peer metadata and associated transitive-peer listings.
These unrelated metadata changes are retained as generated;
no resolved dependency versions changed.

Separate scope-launch attempts failed before their task commands started
with `systemd-run` reporting
`Failed to start transient scope unit: Transport endpoint is not connected`.
The launcher recheck succeeded with the requested limits and child inheritance intact.
That recovery does not establish the transport failure's cause.
No command was retried outside the restricted cgroup.

## Separate historical projection failure, 2026-10-18

Auto-mode's actual Pi RPC session retains an aborted assistant tool-call fragment.
Its Bash argument has 80 code units and incomplete quoting.
The call's identifier never appears in a tool-execution-start event.
Pi's agent loop checks the assistant `error` and `aborted` stop reasons before tool dispatch.
Keeping that fragment in conversation history is separate from executing it.

The next automatic review fails with
`RequestObservationUnavailableError: Tool execution requires usable request evidence`.
The retained cause is the shared analyzer's
`ShellReviewSyntaxError: Bash parser reported incomplete syntax; automatic review requires a complete parsed operation`.
The deciding owned code is `src/visible-context.ts` in `package/pi-plugin/auto-mode`:
its historical assistant projection calls the same strict `projectReviewInput` used for current proposals.
The same reuse occurs for direct Bash history and process-result metadata.
One earlier incomplete record therefore disables source collection for a later operation.

The maintained built projector reproduces this against the actual closed session,
and a clean-history control passes.
The retained log is
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/historical-shell-reproduction-corrected-sdk-scope.log`.
The first probe resolves Pi from the wrong package scope and fails before exercising this behavior;
its module-resolution failure is not product-regression evidence.

The private remedy keeps current operations strict while representing unavailable earlier syntax as data.
It must retain other visible fields,
omit raw command source,
avoid establishing historical command effects or permission,
and propagate unrelated failures.
Private regression tests and consumer verification are pending.
No shared source or installed parser is changed yet.
This is an auto-mode role-boundary defect,
not an Unbash parsing defect or a cancellation failure.

The adopted correction is `e8b7a17e03`.
Both new regression catalogs fail before the correction and pass after it,
with fresh-cache types and full units.
Rebuilt removals of strict proposal validation,
raw historical-source exclusion and unrelated-error propagation each fail the maintained catalog;
restored sources rebuild and pass.
Actual Pi replay `NEymiB` resumes the original saved history and executes the scoped read automatically.
Its captured Luna requests pass required-data and excluded-data controls.

Known rejected historical syntax also emitted repeated parser error diagnostics.
Shared analyzer `0.0.2`,
commit `8d0b81653b`,
keeps its thrown error and records expected rejection at debug level.
Unexpected parser/serialization failures retain error diagnostics and their original cause.
The built consumer exercises both classifications with an injected unexpected serialization failure as its
error-capture positive control.

## Upstream filing decision

1.  Upstream fault:
    no.
    Version 5 produces the documented current tree;
    the repository adapter consumed the previous shape.
2.  Upstream fixability:
    no upstream behavior change is needed for this consumer repair.
3.  Supported use:
    the public parser and exported tree types support AST consumption;
    compatibility with removed fields is not asserted by those types.
4.  Contribution policy:
    not evaluated for a filing,
    because no upstream change is requested.
5.  Maintainer willingness:
    not evaluated for a filing;
    no willingness or refusal is inferred.
6.  Prototype:
    the implemented and tested change belongs to this repository's consumer,
    not an upstream prototype.

No matching `unbash` or shell exemption filename was found in `.out-of-scope/`.
No upstream issue or comment is being drafted or filed regardless:
the upstream-fault condition fails.
The existing repository issue #593 is the appropriate place for repair results.
