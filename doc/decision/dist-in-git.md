# Keep dist committed for Claude Code plugin bundles

Records the decision not to strip the `dist/` carve-out in `.gitignore` that allows the eight Claude Code plugins
under `package/claude-code-plugin/` to ship their built bundles inside this repo.
Future sessions consult this before re-proposing a clean-up of "dist in git" against this carve-out.

This document is appended to,
 not rewritten.
When new constraints force re-evaluation,
 mark this decision superseded;
 do not delete it.

## Context

The repo distributes Claude Code plugins through `.claude-plugin/marketplace.json`,
 which points consumers at
directories such as `./package/claude-code-plugin/terminal-title`.
Each plugin's `.claude-plugin/plugin.json` declares hooks of the form:

```json
{
  "type": "command",
  "command": "${CLAUDE_PLUGIN_ROOT}/dist/final/node/index.mjs"
}
```

When a user installs the plugin,
 Claude Code copies the plugin source into a per-user cache
(`~/.claude/plugins/cache`,
 per the Plugins reference at
<https://code.claude.com/docs/en/plugins-reference> under "Plugin caching and file resolution") and resolves
`${CLAUDE_PLUGIN_ROOT}` to that cache path.
Claude Code does not automatically run `npm install`,
 `pnpm install`,
 a `prepare` script,
 or any other
package manager step on the cached copy.

The hook `command` field accepts a full shell command in shell form
(per <https://code.claude.com/docs/en/hooks> under "Exec form and shell form"):

> Shell form runs when `args` is absent.
>  The `command` string is passed to a shell:
>  `sh -c` on macOS and
> Linux,
>  Git Bash on Windows,
>  or PowerShell when Git Bash isn't installed.

Shell features such as environment variable expansion,
 pipes,
 `&&`,
 `;`,
 globs,
 and redirects are supported.
This means the `command` field is not restricted to pointing at a pre-built artifact;
it can also invoke a TypeScript-capable runtime against source.

Claude Code also provides `${CLAUDE_PLUGIN_DATA}`,
 a persistent per-plugin directory documented at
<https://code.claude.com/docs/en/plugins-reference> under "Persistent data directory".
The reference includes a canonical example of a SessionStart hook that runs `npm install` into
`${CLAUDE_PLUGIN_DATA}` when the plugin's bundled `package.json` differs from the persisted copy:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "diff -q \"${CLAUDE_PLUGIN_ROOT}/package.json\" \"${CLAUDE_PLUGIN_DATA}/package.json\" >/dev/null 2>&1 || (cd \"${CLAUDE_PLUGIN_DATA}\" && cp \"${CLAUDE_PLUGIN_ROOT}/package.json\" . && npm install) || rm -f \"${CLAUDE_PLUGIN_DATA}/package.json\""
          }
        ]
      }
    ]
  }
}
```

So a no-dist-in-git plugin workflow is officially supported when (a) the plugin's dependencies are
resolvable through a public package manager,
 or (b) the plugin can run from source through an interpreter
the user already has on PATH.

The `.gitignore` carve-out for the workspace currently looks like this:

```gitignore
# Claude Code plugin bundles. Marketplace distribution requires committed built files.
!/packages/claude-code-plugin/*/dist/
!/packages/claude-code-plugin/*/dist/final/*.js
```

The carve-out covers 21 files across eight plugins:
`bash-output-filter`,
`claude-spawn`,
`correction-reminder`,
`guardrail`,
`prompt-time`,
`session-start-housekeeping`,
`stop-reminders`,
`terminal-title`,
and the standalone CLI bundles
(`ccssh.js`,
 `ccsr.js`,
 `cctt.js`) that those plugins reference.

Each plugin's `package.json` (verified at e.g. `package/claude-code-plugin/terminal-title/package.json:18`)
depends on:

- `@monochromatic-dev/claude-code-plugin-source` via `workspace:*` (runtime).
- `@monochromatic-dev/config-tsdown` via `workspace:*` (devDependency,
   build).
- `@monochromatic-dev/config-typescript` via `workspace:*` (devDependency,
   build).

None of these workspace packages are published to npm.
The committed `dist/final/node/index.mjs` files are `tsdown` bundles that inline all workspace dependencies
into a single self-contained file the consumer can run with plain Node.

## Decision

Keep the carve-out.
Do not strip the committed `dist/` files for the eight Claude Code plugins.
Do not introduce new committed `dist/` paths outside this carve-out without amending this document.

The decision applies only to plugins distributed through `.claude-plugin/marketplace.json`.
All other packages (libraries,
 web apps,
 dev scripts) continue to keep `dist/` out of git,
 per the default
`.gitignore` rule.

## Reasoning

A no-dist-in-git workflow is technically available for Claude Code plugins.
The specific blocker for this workspace is the dependency shape of its plugins,
 not the Claude Code contract.

The current bundled `dist/final/node/index.mjs` files do two things that no individual move replicates
without other costs:

1. Inline the workspace's shared plugin source (`@monochromatic-dev/claude-code-plugin-source`) into each
   plugin without that source needing to exist on npm.
2. Inline the rest of the runtime dependency tree so the consumer needs only the Node binary,
    with no
   package manager step,
    no network access,
    and no per-machine build toolchain.

Every alternative trades one of those properties for fewer committed files.
For an eight-plugin scope,
 the trade is not worth it (see "Rejected alternatives" below).

The cost the carve-out is paying is also bounded:

- The 21 committed files live under a single namespaced subtree,
   isolated from human-edited code.
- Each plugin's `dist/final/` is a deterministic `tsdown` bundle.
- `.gitignore` denies new `dist/` paths anywhere else.
  The carve-out stays scoped to `package/claude-code-plugin/*/dist/`,
  with `.js` shims re-included because the global `*.js` rule would otherwise hide them.
- The `session-start-housekeeping` plugin already cleans stale dist artifacts on session start,
  reducing the "forgot to rebuild" risk for the workspace's own consumption.

## Rejected alternatives

### Source-only plugins with SessionStart-installs-dependencies

Ship `src/` only.
Each plugin's `plugin.json` declares a SessionStart hook that runs `npm install` into `${CLAUDE_PLUGIN_DATA}`
when `package.json` changes,
 mirroring the canonical pattern in the Claude Code plugin docs.
Runtime hooks then invoke `node` against either the installed package or the source through `tsx`.

Rejected because:

- Every plugin depends on workspace-only packages
  (`@monochromatic-dev/claude-code-plugin-source` and the two `config-*` packages).
  None are on npm.
  `npm install` from the cached plugin directory would fail to resolve them.
- Publishing those workspace packages to npm to unblock this path means adding three new release surfaces
  (the shared plugin source,
   the tsdown config,
   the typescript config),
   each with its own version bump,
  changelog,
   and supply-chain considerations.
  The workspace's philosophy (`PHILOSOPHY.tool-choices.md` and the broader monorepo design) explicitly
  avoids that kind of release plumbing.
- Even if the deps were on npm,
   every consumer would need a working Node and `npm`/`pnpm`,
  plus network access on first run,
   plus the ability for `npm install` to succeed on their machine.
  Native-module compile failures,
   registry outages,
   and proxy/firewall environments turn into
  user-visible plugin failures with no in-tree signal.

### Source-only plugins with `bun run` as the hook command

Ship `src/` only.
Set the hook `command` to `bun run "${CLAUDE_PLUGIN_ROOT}"/src/index.ts`.
Bun reads TypeScript directly and resolves imports at runtime.

Rejected because:

- The plugins' imports still resolve to workspace-only packages.
  Bun cannot pull `@monochromatic-dev/claude-code-plugin-source` out of thin air.
  The same workspace-publishing or source-inlining problem from the previous alternative applies.
- Even if the imports resolved,
   every consumer would need Bun on `PATH`.
  The workspace itself now prefers Node,
   so pushing a Bun requirement onto plugin consumers (who may not be
  Bun users at all) is a new constraint imposed by the distribution choice.
  The current Node-only `.mjs` bundle has none of this friction.

### Pre-bundle to a single `.ts` file instead of `.mjs`

Run `tsdown` to produce one self-contained `.ts` file per plugin,
 commit that,
 and have the hook command
invoke `bun run` (or `tsx`) against the committed file.

Rejected because:

- The artifact is still a generated bundle inside git.
  The complaint that motivated this question ("dist in repo") applies identically.
- The change adds a Bun-or-tsx requirement on every consumer for no reduction in committed files.

### Compile each plugin to native binaries

Use `bun build --compile` to produce platform-specific binaries (linux-x64,
 macos-arm64,
 etc.) per plugin
and commit those.

Rejected because:

- Native binaries are megabytes each.
  Committing one per platform per plugin multiplies the carve-out's footprint by an order of magnitude.
- The artefacts are still committed;
   the original concern is unchanged.
- Cross-compiling at release time still needs CI plumbing the workspace has not built.

### Move the plugins to a separate distribution repo

Split `package/claude-code-plugin/` into its own repository,
 publish dist there,
 keep this monorepo
free of any committed dist.

Rejected because:

- The plugins depend on workspace packages
  (`@monochromatic-dev/logger-core`,
   internal config packages,
   shared types) that exist only in this
  monorepo.
  A separate repo would need to either vendor these dependencies or publish them to a registry.
- Plugin development currently benefits from the monorepo's lint,
   type,
   and build infrastructure
  (`mise run buildAndTest`,
   the lint hooks,
   the shared `tsdown` config).
  A standalone repo would duplicate these or accept divergence.
- The current arrangement keeps plugin source and consumer-facing dist co-located,
   so a contributor sees
  the build output diff alongside the source change in a single PR.
  Splitting hides that signal.

## Conditions for re-evaluation

Reconsider this decision only if at least one of the following becomes true:

1. The workspace begins publishing `@monochromatic-dev/claude-code-plugin-source` and its build-config
   peers to npm,
    eliminating the workspace-only dependency blocker.
   At that point the SessionStart-install pattern becomes practical and the trade reopens.
2. Claude Code's plugin contract adds a richer install-time phase (a documented "package manager run on
   the cache directory" step,
    or a pluggable build pipeline) that does not require `npm install` to
   succeed inside `${CLAUDE_PLUGIN_DATA}`.
3. The plugins migrate out of this monorepo into a dedicated distribution repo,
    at which point this
   decision is superseded by the new repo's policy.
4. The committed `dist/` footprint grows past the eight-plugin scope (for example,
    by adding new plugin
   categories),
    and the carve-out starts catching files it should not.
5. A bug class emerges where stale committed dist drifts from source in ways that confuse contributors,
   despite the session-start-housekeeping cleanup.

The general "no dist in git" default for the rest of the workspace stays in force regardless of how this
specific carve-out evolves.

## Superseded in part, 2026-07-11

`doc/decision/gitignore-negations.md` retires the `.gitignore` carve-out mechanism only:
 the committed bundles move from `dist/` to a distinct tracked directory
 (one-line `outDir` override per plugin),
 so the blanket `dist/` ignore needs no re-include.
Every constraint recorded here stands:
 Claude Code still copies plugin sources into the per-user cache without an install step,
 and built bundles stay committed.
Only their directory name changes.

Implemented 2026-07-12 (issue #365):
 bundles now live in tracked `bundle/node/` per plugin,
 `plugin.json` hooks and `package.json` `bin` entries point there,
 and the carve-out lines left `.gitignore`.
