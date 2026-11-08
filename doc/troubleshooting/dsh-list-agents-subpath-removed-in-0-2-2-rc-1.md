# DSH 0.2.2-rc.1 dropped the `dsh-tool-subagent-control/list-agents` subpath, so a profile patch row for it never starts and no session can be created

## Symptom

On 2026-11-07,
`dsh --profile web` on 0.2.2-rc.1 booted and served,
and every call that creates a session failed.
Against the real `~/.dsh` home,
`session/create` returned this error:

```text
tool-subagent-list-agents (@deepseek-ai/dsh-tool-subagent-control/list-agents): never started
```

`session/prompt` on the same session id then failed with
`resume failed for session "trial-real-home-1": RemoteError: tool-subagent-list-agents (...): never started`.
`session/wait` still succeeded and `session/events` returned no records,
so the failure left no session behind except one projection cache file.

The server's stderr held only the two `skipping profile bundle` lines for the `neuralwatt` and `advisor` plugins.
Nothing in stderr named the cause.
A throwaway `DSH_HOME` holding a copy of only the `web` profile directory created sessions without error,
which is what separated the cause from the plugin warnings.

## Root cause

### The patch row names a subpath that no longer exists

The owner's user-level patch,
`~/.dsh/cordis.patch.yml`,
carried the same two-line row three times inside the `delegation` group of the preset lists,
before the fix at lines 173 to 174,
338 to 339,
and 515 to 516:

```yaml
# ~/.dsh/cordis.patch.yml, inside the delegation group config list (removed on 2026-11-07T21:49:42-05:00)
- id: tool-subagent-list-agents
  name: '@deepseek-ai/dsh-tool-subagent-control/list-agents'
```

The installed 0.2.1-rc.5 package exports that subpath,
and the installed 0.2.2-rc.1 package does not.
Both read from the pnpm store,
`~/.local/share/pnpm/store/v11/links/@deepseek-ai/dsh-tool-subagent-control/<version>/<hash>/node_modules/@deepseek-ai/dsh-tool-subagent-control/package.json`:

```json
// 0.2.1-rc.5: exports keys
[".", "./list-agents", "./list-agents/locale/*.json", "./locale/*.json", "./package.json", "./src/*"]
// 0.2.2-rc.1: exports keys
[".", "./locale/*.json", "./package.json", "./src/*"]
```

The 0.2.2-rc.1 source at commit `fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159`
declares the exports without a `./list-agents` entry,
at [`packages/subagent/tool-subagent-control/package.json` lines 16 to 23][exports-source]:

```json
"exports": {
  ".": {
    "types": "./lib/types/index.d.ts",
    "default": "./lib/index.js"
  },
  "./src/*": "./src/*",
  "./package.json": "./package.json",
  "./locale/*.json": "./locale/*.json"
},
```

The tool did not go away.
The package root now registers it,
at [`packages/subagent/tool-subagent-control/src/index.ts` line 41][register-source]:

```ts
const listAgents = registerListAgents(ctx)
```

So `list_agents` is a built-in of the root plugin in 0.2.2-rc.1,
and the extra row asks the loader for a module path that does not resolve.
The row's plugin never starts,
and the error surfaces when the first session composes its agent,
not at server boot.

### Evidence that the user patch is the only difference

`dsh --profile web --dump-config` run against the real home and against the throwaway home
differ in the composed tree by this extra row,
among other lines the patch changes:

```diff
 # throwaway home (works) versus real home (fails), secret-like lines filtered
+          - id: tool-subagent-list-agents
+            name: '@deepseek-ai/dsh-tool-subagent-control/list-agents'
```

The dump does not fail for a broken row.
It composes configuration only,
so it shows the row as present and valid,
and it cannot detect this fault before a session is created.

### A wrong first reading

The `skipping profile bundle` stderr lines for `neuralwatt` and `advisor` were the visible errors,
and they looked like the cause.
The throwaway run printed the same two lines and created sessions,
so they are a separate, unrelated incompatibility with the same upgrade.

## Verification

Version under test:
dsh 0.2.2-rc.1,
`@deepseek-ai/dsh-tool-subagent-control` 0.2.2-rc.1,
repository commit `fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159` (committed 2026-11-04T16:40:00+08:00).

The harness is the HTTP client in `doc/agent/drive-dsh-as-a-subagent.md`.

Failing catalog,
real home before the fix,
2026-11-07T21:47:17-05:00 run:

- `session/create` for a new session id returned `ok: false` with the `never started` message.
- `session/prompt` for the same id returned `ok: false` with `resume failed for session`.

Working catalog:

- Throwaway home with only the `web` profile copied:
  `session/create`,
  both `session/prompt` calls,
  `session/wait`,
  and `session/events` returned `ok: true`.
- Real home after the fix:
  the same calls returned `ok: true`,
  and the event log shows a `tool/call` for `list_agents` at seq 24,
  so the built-in tool runs.
- Real home after the fix and after the plugin version pins moved,
  one short prompt returned the text `OK-PLUGINS`
  with both `neuralwatt` and `advisor` loading and no `skipping profile bundle` line in stderr.

## Verified workarounds

Delete the three stale rows from `~/.dsh/cordis.patch.yml`,
keeping a backup:

```sh
# run in ~/.dsh
cp --no-clobber cordis.patch.yml "cordis.patch.yml.bak-$(date +%Y%m%dT%H%M%S%z)"
sed -i '/^ *- id: tool-subagent-list-agents$/{N;/dsh-tool-subagent-control\/list-agents/d}' cordis.patch.yml
```

The diff against the backup removed exactly six lines,
and `dsh --profile web --dump-config | rg tool-subagent-list-agents` printed nothing afterwards.
The backup from the real fix is `~/.dsh/cordis.patch.yml.bak-20261107T214942-0500`.

Tradeoffs:

- The patch is valid for 0.2.2-rc.1 and later only.
  A rollback to 0.2.1-rc.5 needs the rows back,
  because that version provides the tool only through the subpath.
- GNU `sed -i` replaces a symlink with a regular file.
  The patch was a regular file here,
  so nothing moved,
  and a symlinked patch needs its target edited instead.
- Nothing else in the patch changes,
  and `list_agents` comes from the root plugin,
  so the model-visible tool set stays the same.

## What does not work

- Treating the `neuralwatt` and `advisor` skip lines as the cause.
  The throwaway run proves they do not block session creation.
- Checking with `--dump-config`.
  It prints the broken row as normal configuration.
- A per-launch `--patch` overlay was not tried.
  The row sits inside a `cordis:group` `config` list,
  so removing it by overlay appears to need the whole list restated,
  and this was reasoned from the patch structure and not run.

## Related changes in this repository

The same upgrade also moved two pins that the repository holds,
recorded here because the failures were found together.

- The catalog in `pnpm-workspace.yaml` pinned the `@deepseek-ai/dsh-*` family at 0.2.1-rc.5.
  Moving them to 0.2.2-rc.1 alone failed `mise run prepare:pnpm:install` on
  `unmet peer @deepseek-ai/cordis`,
  installed 4.0.5 and wanted `~4.1.0`.
  Pinning `@deepseek-ai/cordis` to 4.1.0 cleared it.
  `npm view @deepseek-ai/cordis` shows 4.1.0 as the `latest` tag,
  so no newer line exists.
- `package/dsh-plugin/advisor/package.json` and `package/dsh-plugin/neuralwatt/package.json`
  declare `engines.dsh` as one exact version.
  `doc/troubleshooting/dsh-rc5-profile-startup.md` records why that makes DSH skip the bundle after any upgrade,
  so every DSH bump needs the field moved again.
  Both pins now read 0.2.2-rc.1.
  `mise run //package/dsh-plugin/advisor:lint:types` and `:test:unit` passed with 35 passing test lines,
  and the same pair for `neuralwatt` passed with 7.

## Upstream filing decision

- Is it upstream's fault?
  No.
  The package moved a tool into its root export,
  and the owner's patch pointed at an internal subpath.
  The package manifest makes no stability promise for that subpath that was found in the files read.
- The other constraints were not walked,
  because the first fails.
- The tracker duplicate search,
  the work-in-progress search,
  the trust-system search,
  and the `.out-of-scope/` check did not run.

No upstream draft is kept.
One observation could be raised by the owner if they choose to:
the `never started` message does not say that the module path did not resolve.
Filing it is the owner's decision,
and a report written by an agent can be judged low-effort by a maintainer,
so the owner's account carries that risk.

[exports-source]: https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/subagent/tool-subagent-control/package.json#L16-L23
[register-source]: https://github.com/deepseek-ai/deepseek-harness/blob/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159/packages/subagent/tool-subagent-control/src/index.ts#L41
