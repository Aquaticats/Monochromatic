# Claude Code plugins cluster

Workspace packages for Claude Code plugins,
 plus the source package that owns
their handler logic.

## Account-wide subagent model policy

Accepted and installed on 2026-10-15.
The policy covers ordinary Agent delegation on this machine.
The built-in general-purpose agent defaults to Sonnet;
explicit Opus and Fable model selections are denied.
The separately configured Fable advisor remains enabled.
Forks,
subagent skills,
and teammates are outside this policy's scope.

The live configuration is `~/.claude/settings.json`,
which is outside this repository.
This committed fragment records the installed values for reconstruction;
it is not a complete replacement settings file.
Preserve unrelated settings and existing permission rules when restoring it.

```jsonc
// ~/.claude/settings.json: merge these fields into the existing configuration.
{
  "env": {
    "CLAUDE_CODE_SUBAGENT_MODEL": "sonnet"
  },
  "permissions": {
    "deny": [
      "Agent(model:*opus*)",
      "Agent(model:*fable*)"
    ]
  },
  "advisorModel": "fable"
}
```

The deny patterns match explicit model arguments;
they do not inspect the resolved model of custom agent definitions.
Such definitions can override the Sonnet default,
including through `model: inherit`.
No complete prohibition across custom named agents has been demonstrated.
The installed values were read back and `claude doctor` passed on version 2.1.300;
model selection and rejection were not verified through a live delegation.

The [research and installation record](../../doc/planning/claude-subagent-model-restrictions.md)
preserves the mechanism evidence and verification limits.
The [resume guard observation](./guardrail/README.md#resume-blocking-status)
records why the existing resume check remains unchanged.

## Layout

```text
package/claude-code-plugin/
  source/                              Workspace package; owns handler logic, runtime, types
    src/
      runtime/handler-runtime.ts       Generic stdin -> parse -> handle -> writeOutput shell
      handler/{plugin}.ts             Per-plugin handler function + parser + writer
      test-fixtures/hook-events/       Shared hook-event fixtures (per handler)
    package.json                       Exports `./runtime` and `./handler/{plugin}`
  hook-types/                          Canonical hook-protocol type definitions
  {plugin}/                            One per Claude Code plugin in marketplace.json
    .claude-plugin/plugin.json         Hand-maintained plugin manifest (hooks config, name, etc.)
    src/index.ts                       Four-line shim; imports handler from source and runs it
    package.json                       Slim; declares bin name and depends on source
    mise.toml                          Slim; lint tasks only
    tsconfig.json                      One-line extends
    rolldown.node.config.ts              Spreads the shared .node.ts config; overrides outDir
    bundle/node/index.mjs              Committed bundled entry; what marketplace install runs
    README.md                          Per-plugin documentation
  statusline/                          Not a Claude Code plugin; configured directly in
                                         ~/.claude/settings.json statusLine field
  verbose-tool-output/                 Abandoned; see WONTFIX.md
```

The marketplace lives at `.claude-plugin/marketplace.json` at the repo root.
Each entry's `source:` path points at one of the per-plugin directories under
this cluster.

## How a plugin is structured

The handler logic for each plugin lives in `source/src/handler/{plugin}.ts`,
exported as three named functions:

- `{plugin}Handler`:
   pure function from parsed event to typed response.
- `{plugin}Parser`:
   raw stdin string to typed event.
- `{plugin}Writer`:
   typed response to stdout string.

The per-plugin directory's `src/index.ts` is a four-line shim:

```ts
#!/usr/bin/env node

import {
  guardrailHandler,
  guardrailParser,
  guardrailWriter,
} from '@monochromatic-dev/claude-code-plugin-source/handler/guardrail';
import {
  runHookPlugin,
} from '@monochromatic-dev/claude-code-plugin-source/runtime';

await runHookPlugin({
  parser: guardrailParser,
  handler: guardrailHandler,
  writer: guardrailWriter,
},);
```

The standard rolldown build bundles the shim with the source package contents
inlined via `alwaysBundle: [/^@monochromatic-dev\//]` in
`@monochromatic-dev/config-rolldown/.node.ts`.
 The bundled output goes to
`bundle/node/index.mjs` (a tracked directory,
 unlike gitignored `dist/`;
 see `doc/decision/gitignore-negations.md`),
 which is the path declared in
`.claude-plugin/plugin.json`.

## Adding or porting a plugin

For each plugin to migrate from the old per-plugin source-tree layout into the
source package:

1. Build the current plugin with `mise run //package/claude-code-plugin/{plugin}:build:js:node`
   so a baseline `bundle/node/index.mjs` exists.
2. Capture stdin -> stdout fixtures from the baseline binary covering every
   distinct decision-tree path in the handler.
    Store them under
   `$TMPDIR/{plugin}-baseline/{NN-name}.in.json` and `.out.json`.
3. In the source package,
    create `src/handler/{plugin}.ts` exporting three
   named functions:
    `{plugin}Handler`,
    `{plugin}Parser`,
    `{plugin}Writer`.
   Move all handler logic,
    helper modules,
    and constants here.
4. Add `"./handler/{plugin}": "./src/handler/{plugin}.ts"` to the source
   package's `exports` map.
5. In the per-plugin directory,
    replace `src/index.ts` with the four-line shim
   pattern shown above.
6. Update the per-plugin `package.json`:
    replace `hook-types` and `hook-utils`
   dependencies with `"@monochromatic-dev/claude-code-plugin-source": "workspace:*"`.
7. Run `pnpm install --strict-peer-dependencies=false` to wire the new dep.
8. Run `mise run //package/claude-code-plugin/{plugin}:build:js:node` to
   rebuild the bundled entry.
9. Replay every captured fixture through the new entry and confirm byte-equal
   stdout against the baseline `.out.json` files.
    This is the migration's exit
   criterion.

### Migration order

1. session-start-housekeeping:
    single event,
    single file,
    simplest.
    **Done.
   **
2. stop-reminders:
    single event,
    multi-file logic.
    **Done.
   **
3. bash-output-filter:
    single event,
    multi-file logic.
    **Done.
   **
4. terminal-title:
    multi-event,
    multi-file logic.
    **Done.
   **
5. claude-spawn:
    six events plus the user-facing `spawn-claude` CLI bin.
   CLI bin lives in source's `bin` field;
    the per-plugin keeps a four-line
   `src/cli.ts` shim so the SessionStart hook's auto-symlink target
   (`${PLUGIN_ROOT}/src/cli.ts`) still resolves for marketplace installs.
   Root `package.json` lists the source package as a devDependency so its
   bin hoists to `node_modules/.bin/spawn-claude`.
    **Done.
   **
6. research-agent:
    not a hook handler;
    ships only an agent definition.
    Its
   directory stays as-is;
    no shim or handler module needed.
    **No work needed.
   **

`hook-utils` deleted.
 The runtime in
`source/src/runtime/handler-runtime.ts` absorbs the equivalent
`readStdin`/`writeOutput` logic.

## ADR; per-plugin rolldown config wrappers, not multi-entry from source

Recorded against the tsdown toolchain;
tsdown was replaced by raw rolldown through
`@monochromatic-dev/config-rolldown` on 2026-07-15
(`doc/planning/tsdown-removal.md`).
The decision's shape carries over unchanged
(per-plugin config wrappers,
 `perEntryNodeConfig`,
 committed `bundle/node/`),
so tsdown mentions in the alternatives below are historical.

### Decision

Each plugin keeps its own `rolldown.node.config.ts` (a few lines,
 spreading the
shared `@monochromatic-dev/config-rolldown/.node.ts` with the committed
`bundle/node` outDir override).
 The source package is
source-only;
 it does not own a build orchestrator.
 A full build runs N rolldown
invocations in parallel via `mise`'s task fanout.

### Context

Three layouts were considered for bundling N plugins from one source package:

1. **Per-plugin tsdown wrappers (chosen).
   ** Each plugin has its own one-line
   tsdown config.
    Build runs N rolldown invocations in parallel.
    Per-plugin
   directory contains:
    minimal `package.json`,
    `mise.toml`,
    `tsconfig.json`,
   `rolldown.node.config.ts`,
    `.claude-plugin/plugin.json`,
    `README.md`,
   `src/index.ts` (shim),
    `bundle/`.
    About six files of trivial scaffolding per
   plugin in addition to the marketplace-required and content files.
2. **Multi-entry tsdown from source with `outDir: '..'`.
   ** One config file in
   the source package,
    one tsdown invocation,
    output fanned into per-plugin
   `dist/` directories via entry-name path segments (e.g.
   `'guardrail/dist/final/node/index': './src/entries/guardrail.ts'`).
3. **Generated install roots (M4).
   ** A Rolldown plugin in the source's tsdown
   config generates `.claude-plugin/plugin.json`,
    per-plugin `package.json`,
   and `README.md` from per-plugin metadata modules.
    tsdown owns the entire
   install-root tree.
    `Marketplace.json` paths change.

### Why not multi-entry (option 2)

Tsdown's default `clean: true` recursively walks `outDir` and wipes
hand-maintained sibling files.
 With `outDir: '..'`,
 it wiped not only
`package/claude-code-plugin/{plugin}/.claude-plugin/`,
 `README.md`,
 and
sibling source trees,
 but also `package/config/tsdown/src/`:
 the workspace
package the build itself depended on.
 Recovery via `git checkout HEAD --` was
clean,
 but the footgun is severe.

Two workarounds exist:

- `clean: false`:
   prevents the wipe but accumulates orphan files when entries
  are renamed or removed.
   The cleanup safety net stops working.
- `clean: ['**/dist/final/node/*.mjs']`:
   restricts clean to a glob covering
  only generated artifacts.
   Tsdown supports this,
   but the principle that
  `clean: true` defaults exist because "you never know what your build actually
  generates" still applies in spirit:
   a future contributor adding a new
  generated file format would bypass the glob and silently leak orphans.

### Why not generated install roots (option 3)

Honors the `clean: true` principle by making the install root entirely
tsdown-owned,
 with `.claude-plugin/plugin.json`,
 `README.md`,
 and per-plugin
`package.json` all generated from per-plugin metadata modules.
 The trade is a
custom Rolldown plugin (~30 to 50 lines) shipping inside the tsdown config,
 a
new metadata source file per plugin (`source/src/plugin-config/{plugin}.ts`),
`marketplace.json` path changes,
 and loss of git history for the `plugin.json` and
README files.
 Functionally a build pipeline maintained inside the cluster.

The trade saved roughly 14 trivial lines per plugin (the tsdown wrapper,
tsconfig,
 and shim location) at the cost of about 30 to 50 lines of Rolldown
plugin code plus a metadata module per plugin.
 Net cost increased;
 the design
got bigger,
 not smaller.

### Why option 1 wins

The maintained-logic-per-plugin in option 1 is the `bin` name in the
`package.json` and the import names in the shim.
 About four lines of genuine
content;
 the rest is mechanical scaffolding that does not drift.
 The N parallel
tsdown invocations cost a few seconds of total build time,
 which the mise
fanout already absorbs.

The deepening that matters (handler logic centralized in one source package,
runtime shared across plugins,
 tests colocated with handlers) is achieved
without any of the trade-offs of options 2 or 3.

### Consequences

- Per-plugin directories continue to contain their own `rolldown.node.config.ts`,
  `tsconfig.json`,
   `mise.toml`,
   and `package.json`.
   New plugins follow the
  pattern documented above.
- Each plugin's standard tsdown build produces a self-contained bundle with
  source-package contents inlined via `alwaysBundle: [/^@monochromatic-dev\//]`.
- Future architecture reviews that suggest "consolidate the per-plugin
  packages into one" should be referred to this ADR.
   The logical consolidation
  has already happened in the source package;
   the per-plugin directories are
  install containers with cherry-pickable marketplace boundaries.
- If a future contributor wants to attempt multi-entry again,
   the failure modes
  of `clean: true` with `outDir: '..'` are documented above.
