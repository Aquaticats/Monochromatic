# dsh 0.2.1-alpha.4 fixture profiles for plugin verification mount no inspect tools and resolve no model credentials

## Symptom

Booting a disposable fixture profile
(`dsh --profile <fixture> --from-default-profile headless`)
to verify a plugin bundle fails in two independent ways.

1. Silent tool-surface gap:
   `cordis_inspect_list` and `cordis_inspect_query` are absent
   from the fixture session's tool surface.
   No error is raised;
   the checklist steps that call them cannot start,
   and the fixture agent reports the tools as missing.
2. Route credential failures while the fixture's own agent boots,
   with three verbatim variants:

   `dsh: MISSING_CREDENTIAL: llm-deepseek: no API key for provider route "deepseek-official"; store DEEPSEEK_API_KEY through the credentials service (the web Models page writes it), or export DEEPSEEK_API_KEY in the launching environment`

   `dsh: MISSING_CREDENTIAL: llm-pi-ai: no credential for provider route "opencode-go"; its profile resolves OPENCODE_GO_API_KEY, which is not set — store OPENCODE_GO_API_KEY through the credentials service (the web Models page writes it) or export it, and remove apiKeyEnv only if this provider should authenticate from pi-ai's own environment discovery`

   `dsh: PI_AI_ERROR: Provider is not configured: opencode-go`

Surface triggers:
a fixture created from the `headless` template,
route keys absent from the launching environment,
providers declaring `apiKeyEnv`,
and providers omitting `apiKeyEnv`
for custom providers that only a patch declares.

## Root cause

### Inspect tools need one host row and one preset row

`@deepseek-ai/dsh-tool-cordis/README.md:28`
states the mounting rule outright:

> Creator mode includes this toolset.
> Other compositions mount `@deepseek-ai/dsh-tool-cordis/host`
> once in the host composition beside the host runner
> that provides `cordisInspect`,
> and `@deepseek-ai/dsh-tool-cordis`
> in each agent preset that exposes the tools;
> a preset row alone registers no Host providers.

`@deepseek-ai/dsh-tool-cordis/README.md:38`
adds the registration constraint:

> the host entry owns the Host provider registrations
> and the preset row owns the two tools,
> each through Cordis effects;
> the registry rejects a duplicate provider id,
> which is why the providers register once per process
> rather than per preset.

The `headless` template composes neither row.
`dsh --profile advisor-fixture-minimal --dump-config`
contains no `cordis-host-runner`,
no `cordis-inspect-providers`,
and no `tool-cordis` row,
while the live `web` profile wires all three:
`cordis-host-runner`,
`cordis-inspect-providers` (`@deepseek-ai/dsh-tool-cordis/host`),
and `tool-cordis` nested in its `preset-cordis` plugins list
(`/home/user/.dsh/profiles/web/cordis.patch.yml`).

### Route credentials resolve per composition, not per machine

The `headless` template's `agent-default-model` row
names `provider: deepseek-official`, `model: deepseek-flash`.
The `MISSING_CREDENTIAL` diagnostics
describe the resolution order:
the launching environment first,
then the credentials service,
with pi-ai's own environment discovery
only when a provider omits `apiKeyEnv`.

Measured facts behind the failures:

- Tool shells do not inherit the harness process environment.
  The harness PID's `/proc/<pid>/environ` carries
  `DEEPSEEK_API_KEY` and `LLMGATEWAY_DEVPASS_API_KEY`;
  the same names are absent from the tool shell environment.
- A fixture does not resolve the live profile's route keys.
  The fixture mounts the identical `dsh-credentials-local` row
  and still reports `MISSING_CREDENTIAL` for `opencode-go`.
- `OPENCODE_GO_API_KEY` is named only
  in `/home/user/.dsh/profiles/web/cordis.patch.yml`.
  Its value is absent from the harness process environment,
  and no on-disk store under `~/.dsh` names it,
  so the exact store the live profile uses stays unknown.
- Removing `apiKeyEnv` from a patch-declared custom provider
  routes resolution into pi-ai discovery,
  which rejects the provider with `PI_AI_ERROR`
  because only the patch declares it.

An earlier reading was wrong and is recorded here
so it is not re-derived:
"the credentials service is shared across profiles,
so a fixture mounting the same `credentials` row resolves the live routes."
The fixture's own `MISSING_CREDENTIAL` output for `opencode-go`,
quoted in the Symptom section,
disproves it.

## Verification

Version under test:
`dsh --version` prints `0.2.1-alpha.4`
from the mise npm install `npm-deepseek-ai-dsh` alpha v11.

Run the harness from a shell with full file access.
The workspace-only sandbox cannot run `--dump-config`
because profile preparation writes the composed `cordis.yml`.

```bash
# fixture creation and bundle link
dsh --profile advisor-fixture-minimal --from-default-profile headless --dump-config
dsh plugin --profile advisor-fixture-minimal add /abs/path/to/package/dsh-plugin/advisor

# boot with the disposable overlay and an exported route key
export LLMGATEWAY_DEVPASS_API_KEY=... # from the launching environment
dsh --profile advisor-fixture-minimal --patch "${HOME}/temp/agent/advisor-fixture-llm.patch.yml" -
```

Catalog that works cleanly:

- With the overlay's three inserted rows,
  `cordis_inspect_list` and `cordis_inspect_query` are callable
  inside the fixture session.
- `Tool.listTools` reports the plugin's `advisor` tool
  with its parameter schema.
  Filtered and paged inspect queries fail separately
  in [dsh-cordis-inspect-query-input-coerced-to-string](dsh-cordis-inspect-query-input-coerced-to-string.md).
- The `Config.listConfigs` entry check for the plugin row
  is blocked by that defect
  (unfiltered calls return only the first page);
  the composed `--dump-config` tree
  shows `id: advisor` with `config: {}`
  as the composition-level evidence.
- The fixture agent boots and answers a task
  on `llmgateway-devpass`/`mimo-v2.6-pro`.

Catalog that fails, by variant:

1. Boot without exported route keys
   and without an overlay:
   `MISSING_CREDENTIAL` for `deepseek-official`.
2. Overlay declaring `apiKeyEnv: OPENCODE_GO_API_KEY`
   without exporting it:
   `MISSING_CREDENTIAL` for `opencode-go`.
3. The same overlay with `apiKeyEnv` removed:
   `PI_AI_ERROR: Provider is not configured: opencode-go`.
4. Any boot without the three inserted rows:
   the inspect tools are silently absent
   from the session tool surface.

## Verified workarounds

### Disposable overlay inserting the inspect rows and one routed provider

```yaml
# ${HOME}/temp/agent/advisor-fixture-llm.patch.yml
- id: llm-pi-ai
  name: "@deepseek-ai/dsh-llm-pi-ai"
  config:
    providers:
      llmgateway-devpass:
        apiKeyEnv: LLMGATEWAY_DEVPASS_API_KEY
        api: openai-completions
        baseURL: https://api.llmgateway.io/v1
        models:
          - id: mimo-v2.6-pro
            name: MiMo V2.6 Pro
            contextWindow: 1048576
- id: agent-default-model
  name: "@deepseek-ai/dsh-agent-default-model"
  config:
    provider: llmgateway-devpass
    model: mimo-v2.6-pro
- insert:
    - id: cordis-host-runner
      name: "@deepseek-ai/dsh-cordis-host-runner"
    - id: cordis-inspect-providers
      name: "@deepseek-ai/dsh-tool-cordis/host"
    - id: tool-cordis
      name: "@deepseek-ai/dsh-tool-cordis"
```

Tradeoffs:
the fixture's agent route differs from the live profile's route,
the route key travels through the launch environment,
and the overlay must track the live patch
when provider lists change.

### Export route keys from the harness process environment

Read the key names from `/proc/<pid>/environ` of the harness process
and export them for the fixture launch,
keeping values out of command output.

Tradeoffs:
the value passes through the fixture process environment,
and the export must be repeated per launch.
Storing the key through the web Models page
does not help fixture profiles,
which do not resolve the live profile's store.

## What does not work

- Relying on fixture inheritance of the live profile's
  stored route credentials.
  The identical `credentials` row is mounted
  and the lookup still fails.
- Declaring `apiKeyEnv` for a key that is neither exported
  nor stored in the fixture's reachable store.
- Removing `apiKeyEnv`
  to fall back to pi-ai discovery
  for a custom provider that only a patch declares.
  pi-ai reports `Provider is not configured`.
- Expecting `cordis_inspect_*`
  in a headless-template session
  without the host and preset rows.
  The tools are simply not registered;
  there is no error to catch.
- Running `dsh --dump-config`
  inside the fixture session's own workspace-only sandbox.
  Profile preparation writes `cordis.yml` and hits `EROFS`.

## Upstream filing decision

`.out-of-scope/` holds no entry covering dsh fixture composition
(checked `bun-install.md`, `codex-harness.md`,
and the other listed exemptions).

1. Is it really upstream's fault?
   No.
   Both behaviors are documented design:
   the mounting rule is explicit
   in `@deepseek-ai/dsh-tool-cordis/README.md:28`,
   and the credential diagnostics state their resolution order.
   The friction came from our fixture plan
   assuming inspect tools and shared credentials.
2. Can upstream fix it?
   Not applicable.
   There is no defect,
   only documentation we misread.
3. Are they supporting this use case?
   Yes.
   The plugin-development and composition skills
   describe fixture and bundle verification,
   and the READMEs above describe the row requirements.
4. Would the repo welcome our contribution?
   Not applicable while no defect is filed.
5. Will they likely fix it?
   Not applicable.
6. Have we prototyped a minimal fix?
   Not applicable;
   the consumer-side workaround above
   is the complete fix on our side.

Outcome:
do not file.
The kept artifact is this note,
not a new-issue draft:

~~~md
Nothing to file upstream.
Fixture verification needed creator-mode inspect rows
and exported route keys,
both documented in the dsh package READMEs
and the MISSING_CREDENTIAL diagnostics.
Our planning doc assumed otherwise;
the assumption is corrected there.
~~~
