# dsh-plugin-qwen-token-plan-individual

Qwen Token Plan Individual provider for DSH:
the plugin owns the `qwen-token-plan-individual` provider route
over the shipped `@deepseek-ai/dsh-llm-pi-ai` adapter,
serving every serviceable chat model
the [Qwen Token Plan listing endpoint][qwen-token-plan-listing] publishes today.
The generated catalog pi-ai ships carries a stale ten-model list,
so this route maps the live listing against a curated facts table
(context window,
max output,
input modalities,
selectable thinking levels,
wire-compat pins)
with a checked-in snapshot of the same listing standing in
while the endpoint is unreachable.
The design record is
[`doc/planning/qwen-token-plan-individual-provider.md`](../../doc/planning/qwen-token-plan-individual-provider.md).

## Pi dependency trust

PJN:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## How it serves

Activation is one settings write plus one bounded async fetch:

1.  The plugin maps the checked-in snapshot and merges it into the `llm-pi-ai`
    entry's volatile `providers` field through the settings service,
    keyed as
    `qwen-token-plan-individual`.
    The shipped pi-ai adapter reads that field and registers the
    route with its adapter,
    so the model list stands without
    waiting on the network.
2.  The mount fetch resolves the `QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY`
    credential reference,
    then reads the listing endpoint
    [the Qwen Token Plan listing endpoint][qwen-token-plan-listing] within a fixed time
    bound,
    maps it,
    and merges the fresh route the same way.
3.  No resolving credential,
    a fetch failure,
    or a listing mapping to no serviceable
    model logs the reason and keeps the served snapshot,
    so an unreachable
    endpoint or an empty credential store never takes the route down.
4.  A write whose stored route already matches the mapped one is skipped,
    so an
    unchanged profile document is never rewritten.

The plugin never applies a second pi-ai adapter.
The shipped `llm-pi-ai`
instance owns every route,
and its activation re-declares the whole
configurable provider catalog,
which a second instance cannot survive.
The
loader merges `providers` per provider key instead,
so hand-declared routes
beside `qwen-token-plan-individual` keep standing untouched.

## Model mapping

The mapping itself lives in the shared core
`@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing`
(`package/agent-harness-shared/qwen-token-plan-individual-listing`),
so the pi extension projects the same facts;
this section documents the behavior the shared mapping serves.

Membership comes from the listing,
never from the curated table:
every serviceable chat id the endpoint returns is served
(fourteen on the 2026-11-11 measurement:
the thirteen chat ids plus the `auto` router alias),
and the ids whose chat probes were refused
(image,
video,
tts,
asr,
and realtime models)
are skipped.
An id absent from both the facts table and the snapshot
whose name marks it media is skipped too,
so an id arriving between refreshes cannot be served as chat by mistake.

Each listing entry maps to one pi-ai model profile:

- `name`,
  `contextWindow`,
  `maxTokens`,
  `input`,
  and the wire-compat pins come from the facts table row for the id,
  then from its older siblings (newer to older),
  then from pi's schema defaults,
  so a model the table does not know yet
  serves with understating facts instead of disappearing.
- `reasoningEfforts` is the declared efforts dict for a model offering
  selectable efforts,
  `false` for a non-reasoning model,
  and omitted for a model that reasons without a selector
  (`qwen3.6-flash`,
  `qwen3.7-max`,
  `qwen3.7-plus`,
  `auto`).
  Omitted inherits the installed catalog entry's capability,
  which is the builtin's own shape for those three models.

One recorded divergence comes from that third state:
`auto` has no installed catalog entry,
so its omitted `reasoningEfforts` inherits nothing
and this harness sees a non-reasoning model,
while the pi extension serves it as reasoning without selectable efforts
(`doc/planning/qwen-token-plan-individual-provider.md`,
"three reasoning states").

## Thinking level dispatch

Each model's efforts dict pins every thinking level explicitly:
a declared level carries its wire spelling,
a declared `null` `off` means "supported, send nothing",
and every undeclared level pins `null`
so pi-ai's asymmetric defaulting never offers one.
The expansion lives in the shared core
(`expandQwenTokenPlanThinkingLevelMap`),
and a characterization suite drives one offered dict
through both the shared expansion and the real `dsh-llm-pi-ai` adapter,
asserting identical thinking-level maps and wire spellings.

The route pins `thinkingFormat: "qwen"`,
so a selected level dispatches as the top-level `reasoning_effort` parameter
beside `enable_thinking`,
and the same pins the builtin catalog carries
(`supportsDeveloperRole: false`,
`supportsStore: false`,
`supportsStrictMode: true`,
`supportsReasoningEffort` true exactly when the model offers efforts)
land per model,
so a model the builtin also serves dispatches identically to the builtin.

## Credential seam

`apiKeyEnv: QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY` is a credential reference,
not an environment requirement:
it addresses a record in the DSH credentials service
(`$DSH_HOME/.credentials.yaml`,
written from the web Models page),
and the launching environment is only a fallback source of that record.
The written route keeps the pin the hand-declared block carried,
so the record already present keeps resolving without any migration.
The listing fetch resolves the same reference
through the plugin context's credentials service,
exactly as the installed `@deepseek-ai/dsh-llm-pi-ai` resolves `apiKeyEnv`
names,
once per mount and never cached.
The lookup runs when the listing fetch is about to run,
not at plugin `apply`,
because the loader can register the credentials service after this plugin
activates;
a composition that mounts none, or one that registers after the mount-race
budget, keeps the served snapshot instead of refusing to load.

## Mount-race writes

The settings write lands through the same `package.json` writer lock the
plugin-manager holds across its boot reload
(measured hold 11151 ms starting 9695 ms into a web-profile boot,
against `@deepseek-ai/dsh-atomic-write` `DEFAULT_LOCK_WAIT_MS` = 2 s).
A write refused by that contention, or by a `@deepseek-ai/dsh-config-editor`
reload race, retries within a bounded budget of ten attempts spaced 2 s
apart,
which outlasts every measured hold;
a permanent refusal
(no `llm-pi-ai` entry in the profile,
a schema rejection)
throws immediately without spending the budget.

## Replacing the hand-declared block

The owner decision "full live list in both" means this plugin's settings write
replaces the hand-declared two-model `qwen-token-plan-individual` block
under the `llm-pi-ai` row's `providers` key in the profile's own
`cordis.patch.yml`.
The write lands the whole mapped route under that one key.
The loader merges that field per provider key,
so the other routes in the file survive,
and comments outside the row's `config` node survive too.
The block is machine-managed but not machine-removed:
uninstalling the bundle leaves it behind,
and dropping it is a hand edit.

## Deliberately omitted scope

Media generation dispatch
(image,
video,
and audio endpoints)
is out of scope:
those ids are skipped,
not served.
No pricing,
quota,
or cost surface exists here either:
this provider serves models and their capabilities,
and request-level token accounting stays the harness's concern.

## Wiring

`engines.dsh` declares the verified host version `0.2.2-rc.1` explicitly.
The exact prerelease limits the compatibility claim to the tested host;
revalidate activation before changing it.
DSH reads this field before peer dependencies,
so local directory links can retain the workspace's `catalog:` dependency specifications.
See the [startup diagnosis](../../../doc/troubleshooting/dsh-rc5-profile-startup.md).

The bundle is `@monochromatic-dev/dsh-plugin-qwen-token-plan-individual`,
installed through
`plugin_manager` `install_bundle` like any workspace bundle.
The `qwen-token-plan-individual`
route has exactly one owner,
so a profile `cordis.patch.yml` that still declares a hand-written
`qwen-token-plan-individual` block under `llm-pi-ai` `providers` must
drop that block;
the plugin's route replaces it.

`@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing`'s
constants name the route key,
wire protocol,
credential reference,
and the listing fetch bound in milliseconds;
this package owns the DSH endpoint base
`https://token-plan.maas.qwencloudapi.com/compatible-mode/v1`
because base URLs stay per harness
(the pi extension keeps the builtin's host).

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one from the repository root with
`mise run //package/dsh-plugin/qwen-token-plan-individual:<task>`.

Unit tests import package behavior from the built `dist` entry,
so run `build`
before `test:unit` after source changes.

[qwen-token-plan-listing]: https://token-plan.maas.qwencloudapi.com/compatible-mode/v1/models
