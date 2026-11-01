# dsh-llm-pi-ai 0.2.1-rc.2: a second adapter apply dies at `DUPLICATE_DIRECTORY` and its routes stay unregistered

A plugin bundle that applies the shipped pi-ai adapter a second time
(`applyPiAiAdapter`) beside the profile's `llm-pi-ai` entry fails its
configurable-provider commit,
 and every route that apply would have
registered is missing at first dispatch.

## Symptom

Boot of a profile whose bundle calls `applyPiAiAdapter` while the `llm-pi-ai`
entry is also active prints one activation warning,
 and the first request on
the route the plugin owns fails with `NO_ADAPTER`.
 Two error variants appear,
 one per phase.

Activation (`dsh <profile> --json "reply with exactly: pong"` stderr,
 frames below the two cited calls elided):

```text // ~/temp/agent/nw-run-err.txt (2026-10-21)
dsh: warning: 1 entry did not activate
neuralwatt (@monochromatic-dev/dsh-plugin-neuralwatt): LlmError: configurable provider "amazon-bedrock" is already declared
    at commit (…/@deepseek-ai/dsh-llm/lib/index.js:1935:141)
    at Proxy.registerConfigurableProviders (…/@deepseek-ai/dsh-llm/lib/index.js:1943:29)
    at ensureDirectory (…/@deepseek-ai/dsh-llm-pi-ai/lib/index.js:2922:49)
    at apply (…/@deepseek-ai/dsh-llm-pi-ai/lib/index.js:2926:2)
```

First dispatch of the route the failed apply owned:

```text // ~/temp/agent/nw-run-err.txt (2026-10-21)
dsh: NO_ADAPTER: no adapter registered for provider "neuralwatt"
```

The `amazon-bedrock` name in the first error is the first catalog collision
found,
 not the provider the plugin adds;
 the commit walks the whole shared
catalog before it mutates anything.

## Root cause

The shipped `llm-pi-ai` entry's apply builds the configurable-provider
directory from the settings profiles and registers it exactly once,
 replacing
it in place on later settings changes:

```js // @deepseek-ai/dsh-llm-pi-ai/lib/index.js:2919-2926
const ensureDirectory = () => {
    const entries = directoryEntries(profiles(), settingsNs);
    if (deepEqualJson(entries, directoryFacts)) return;
    if (directory === void 0) directory = ctx.llm.registerConfigurableProviders(entries);
    else directory.replace(entries);
    directoryFacts = entries;
};
ensureDirectory();
```

`registerConfigurableProviders` documents a single-owner contract,
 and its
commit throws on any overlap before it mutates the directory:

```js // @deepseek-ai/dsh-llm/lib/index.js:1911-1941
/**
* Declare provider routes an adapter plugin can activate through
* configuration. Registration is all-or-nothing: an empty list, invalid
* entry, or a provider already declared by any registration throws
* `LlmError` without registering the rest. Disposed with the fiber.
* @param entries - every configurable provider this plugin owns.
*/
const commit = (candidates) => {
    const detached = [];
    const own = new Set(held.map((entry) => entry.provider));
    for (const entry of candidates) {
        if (this.directory.has(entry.provider) && !own.has(entry.provider) || detached.some((seen) => seen.provider === entry.provider)) throw new LlmError(`configurable provider "${entry.provider}" is already declared`, "DUPLICATE_DIRECTORY");
        detached.push(detachDirectoryEntry(entry));
    }
    for (const entry of held) this.directory.delete(entry.provider);
    for (const entry of detached) this.directory.set(entry.provider, entry);
};
```

So the call chain is:
 the second `applyPiAiAdapter` runs `ensureDirectory()`
at its apply (`@deepseek-ai/dsh-llm-pi-ai/lib/index.js:2926`) with the full
settings-derived catalog,
 which includes providers the first registration
already owns (`amazon-bedrock` among them).
 The commit throws at
`@deepseek-ai/dsh-llm/lib/index.js:1935` before its mutation loops,
 so the
second registration withdraws nothing and registers nothing.
 The throw then
unwinds the plugin apply past its own route registration calls,
 which never
run,
 and first dispatch reports `NO_ADAPTER`.
 Contract line `@param entries -
every configurable provider this plugin owns` names the caller obligation
the second apply broke:
 it re-declared providers it does not own.

An earlier hypothesis was wrong and is worth recording:
 swallowing the
`LlmError` and continuing to register the route does not work.
 The `NO_ADAPTER`
variant above is the evidence:
 the route registration calls sit after the
throw site in apply,
 so swallowing only converts a boot failure into a
missing-route failure.

## Verification

Versions under test:

- `@deepseek-ai/dsh-llm-pi-ai@0.2.1-rc.2`,
  from `deepseek-harness` `packages/llm/llm-pi-ai`
  (repository `git+https://github.com/deepseek-ai/deepseek-harness.git`).
- `@deepseek-ai/dsh-llm@0.2.1-rc.2`.
- Harness `dsh` next build `v11/1483ad-18e0966c47886467-0`.

Runnable harness:
 a throwaway `DSH_HOME` with a profile that activates the
`llm-pi-ai` entry and one bundle applying `applyPiAiAdapter` a second time.
 The
historical failing plugin build is the parent of the fix commit
`a9fd6a5851` in `Monochromatic`:

```sh // reproducer outline
git -C /var/home/user/Monochromatic worktree add "${HOME}/temp/agent/dupdir-repro" a9fd6a5851^
# build package/dsh-plugin/neuralwatt in that worktree, then link and run it:
DSH_HOME="${HOME}/temp/agent/dsh-home-dupdir" dsh --profile dupdir --from-default-profile headless --dump-config
DSH_HOME="${HOME}/temp/agent/dsh-home-dupdir" dsh plugin --profile dupdir add "${HOME}/temp/agent/dupdir-repro/package/dsh-plugin/neuralwatt"
DSH_HOME="${HOME}/temp/agent/dsh-home-dupdir" dsh dupdir --json "reply with exactly: pong"
```

The `--from-default-profile headless` step matters:
 the one-shot `--json` flag
belongs to the headless app,
 so a profile initialized from any other
template rejects it.

The recorded run of that harness is `~/temp/agent/nw-run-err.txt`,
 quoted in
Symptom.

Patterns that work cleanly:

- One adapter instance per composition;
  its `directory.replace(entries)`
  updates routes in place when settings change
  (`@deepseek-ai/dsh-llm-pi-ai/lib/index.js:2923`).
- Settings-feed route writes into the `llm-pi-ai` entry land without touching
  the directory registration;
  three consecutive one-shot runs boot clean.
- A registration that withdraws and re-declares its own held providers
  (the `own` set at `@deepseek-ai/dsh-llm/lib/index.js:1931`) replaces them.

Patterns that fail:

- A second `applyPiAiAdapter` beside the active `llm-pi-ai` entry
  (`DUPLICATE_DIRECTORY`,
  the stack in Symptom).
- Swallowing the commit error and continuing
  (`NO_ADAPTER`,
  the second
  variant in Symptom).
- Registering an empty entry list
  (`INVALID_DIRECTORY`,
  "a configurable-provider registration must declare at least one provider").
- Re-declaring any provider name another registration already holds
  (`DUPLICATE_DIRECTORY`),
  even when the new registration would add routes.

## Verified workarounds

Keep exactly one adapter instance and feed routes through the settings layer.
 The plugin maps its listing and merges it into the `llm-pi-ai` entry's
volatile `providers` field
 (`SettingsForms.update('llm-pi-ai', { providers: { …
} })`,
 implemented in `package/dsh-plugin/neuralwatt/src/providers.ts` and run
by `mise run //package/dsh-plugin/neuralwatt:test:unit`).
 The shipped
`ensureDirectory` then replaces the directory with the merged set.
 Tradeoffs:
 the write lands in the user's `cordis.patch.yml` as a machine-managed
block that survives uninstall,
 and the bundle install itself still needs a
host restart to mount.

Declare the route by hand in the profile patch
`llm-pi-ai` `config.providers`
and ship no plugin.
 Tradeoffs:
 no live listing refresh,
 so context windows and
thinking levels rot against the API.

## What does not work

- A second `applyPiAiAdapter`:
  the failure itself.
- Swallowing the commit `LlmError` and registering afterwards:
  the route
  registration calls sit after the throw site in apply,
  so the route never
  registers.
- Merging routes through the bundle `cordis.patch.yml` into
  `config.providers`:
  the user layer replaces the entry `config` wholesale,
  so
  bundle-supplied routes vanish when the user patch supplies the row.
- `!!js` expressions carrying functions:
  loader config is data-only and no
  function crosses into it.
- `isolate: { llm }`:
  forks the shared LLM service instead of extending it.
- Disabling the `llm-pi-ai` row:
  drops every hand-declared user route with it.
- Constructing `PiAiAdapter` directly:
  the class is not exported,
  only the
  apply entry.

## Upstream filing artifact

### Upstream filing decision

- `.out-of-scope/`:
  no exemption names `deepseek-harness`,
  `dsh-llm`,
  or
  `dsh-llm-pi-ai`;
  `codex-harness.md` covers Codex only and
  `claude-code-upstream-bugs.md` covers Claude Code only.
- Duplicate search:
  `gh search issues '"configurable provider" "already
  declared"'` (0 results),
  `gh search issues --repo
  deepseek-ai/deepseek-harness "configurable provider"` (0 results),
  `gh search
  prs '"registerConfigurableProviders"'` (4 results,
  all third-party plugin
  repos,
  none upstream).
  No duplicate exists to comment on.
- Constraint 1,
  really upstream's fault:
  no.
  `@deepseek-ai/dsh-llm/lib/index.js:1911-1917` documents the
  all-or-nothing single-owner contract and `:1935` implements it;
  the
  collision came from our plugin re-declaring providers it does not own.
- Constraint 2,
  can upstream fix it:
  not applicable after constraint 1.
  A
  clearer error name or a documented route-feeding seam would be an
  enhancement,
  and the settings seam already supports the use case.
- Constraint 3,
  supporting this use case:
  yes through the settings layer,
  whose
  profiles the shipped `ensureDirectory` reads
  (`@deepseek-ai/dsh-llm-pi-ai/lib/index.js:2920`);
  the workaround below uses
  that seam.
- Constraint 4,
  welcome contribution:
  fails outright.
  `gh repo view
  deepseek-ai/deepseek-harness --json hasIssuesEnabled` reports
  `hasIssuesEnabled: false`,
  so no issue can be filed at all.
- Constraint 5,
  likely fix:
  not applicable after constraints 1 and 4.
- Constraint 6,
  prototyped minimal fix:
  not applicable.
  The consumer-side
  settings-feed workaround is the fix and runs in production in this repo.

Decision:
 do not file.
 The draft below stays as an auditable record and is marked
"do not file as-is".

~~~md
do not file as-is

Title: configurable-provider commit names the first catalog collision when a second adapter registers the full catalog

Labels: bug

Description:
  Applying the shipped pi-ai adapter a second time beside the active llm-pi-ai
  entry throws `LlmError: configurable provider "amazon-bedrock" is already
  declared` (`DUPLICATE_DIRECTORY`) at
  `@deepseek-ai/dsh-llm/lib/index.js:1935`, unwinding apply before its route
  registrations, so the plugin's routes fail `NO_ADAPTER` at first dispatch.

  Source trace: `@deepseek-ai/dsh-llm-pi-ai/lib/index.js:2919-2926` registers
  the full settings-derived catalog at apply;
  `@deepseek-ai/dsh-llm/lib/index.js:1929-1941` commits all-or-nothing and
  throws at the first overlap before any mutation.

  Reproduction: the harness in the Verification section of
  doc/troubleshooting/dsh-llm-pi-ai-duplicate-configurable-provider.md

  Suggested fix: name every colliding provider in the error, or document the
  settings-feed seam as the supported route-adding path.
  Constraints 1 and 3
  read the current behavior as contract, so this is an enhancement note,
  not
  a defect report.
~~~
