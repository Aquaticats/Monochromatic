# pi-plugin-openai-fast

Genuine pi virtual models that request priority through existing native OpenAI authentication.
Build,
 type checking,
 and the complete offline suite pass against pi `1.0.0`.
Lint and extension-host verification also pass.
Legacy Codex ordinary and priority-request live checks passed on pi `1.0.0`,
 including discovery of the globally installed replacement.
The additional native OpenAI mapping passes offline native HTTP and real CLI startup checks.
No live new-sign-in or API-key billing verification was added.

Note 2026-11-10:
build,
 type checking,
 lint,
 and the complete offline unit suite were re-verified against pi `1.5.1`
 on the workspace lockfile.
Pi stopped re-exporting TypeBox from `@earendil-works/pi-ai` at `1.5.0`,
 so the tool fixtures import `Type` from the host-provided `typebox` package.
Extension-host verification was not re-run.

## Pi dependency trust

PZA:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Model selection

Each registered base chat model gets a selectable virtual companion:

- Native OpenAI:
   `openai/<model-id>` becomes `openai-fast/<model-id>`.
- Legacy Codex:
   `openai-codex/<model-id>` becomes `openai-codex-fast/<model-id>`.

Companions are discovered from each source provider's configured native catalog,
 not a manually maintained compatibility list.
`openai-fast` appears in available-model lists only when native `openai` chat models are available.
`openai-codex-fast` appears only when legacy `openai-codex` chat models are available.
Each namespace follows its own source's configuration,
 including startup,
 logout,
 and reconfiguration;
 one configured provider does not make the other provider's companions available.
Selecting an unsupported model in fast mode surfaces the provider error.
The extension does not silently change the model or retry with an ordinary tier.
Normal native transient retries and transport recovery remain pi behavior.

The original upstream model ID is sent with `service_tier: "priority"`.
No invented `-fast` upstream model ID is sent.
Assistant history retains the original source provider and model identity;
 pi keeps the virtual selection separately.

Defaults and `enabledModels` are not changed.
Fast companions do not enter exact ordinary-model scopes.
Existing wildcards that already match the fast namespace can include them.
There are no added commands,
 footer widgets,
 badges,
 or selection notifications.
Pi's standard virtual-model presentation remains host behavior.

## Authentication and transport

`openai-fast` reuses the native `openai` provider's ChatGPT sign-in or API-key authentication.
`openai-codex-fast` continues to reuse the legacy `openai-codex` login.
There is no separate fast login,
 copied credential store,
 authentication-method restriction,
 or extension-owned billing logic.
Native credential precedence remains pi behavior.

The native provider still owns OAuth refresh,
 logout,
 ordinary model dispatch,
 and transport behavior.
Priority requests use the source provider's native stream implementation,
 preserving reasoning,
 tools,
 callback replacement payloads,
 cache/session settings,
 abort signals,
 retries,
 timeouts,
 headers,
 and request instrumentation.

Priority targets are local physical entries under the fast adapter provider,
 excluded from normal filtered availability lists.
Their IDs start with `__pi_openai_fast__/` and are never sent upstream.
Do not use that reserved prefix for configured OpenAI models.
The complete registry and pi's own routing display can still expose physical target identities.

## Backend contract

The guarantee is **requesting priority**,
 not guaranteed acceleration or confirmed priority execution.
Backend support,
 entitlement,
 rollout,
 and actual quota consumption can differ from request intent.
[OpenAI documents increased usage consumption for Fast][codex-speed].
No compatibility list or served-tier reporting UI is maintained here.

## Installation

Build and verify the package before adding its repository path to global pi packages.
Remove `npm:pi-openai-codex-fast` before loading this replacement:
its physical model IDs conflict with the new virtual entries using the same companion identities.

Keep installation in global `~/.pi/agent/settings.json`,
 not this repository's intentionally package-free project settings.
The package manifest loads `dist/final/node/index.mjs`.
The verified replacement is installed globally.
Restart pi or run `/reload`,
 then select `openai-fast/<model-id>` or `openai-codex-fast/<model-id>` through the standard model picker.
An existing installation of this local package needs only a rebuild and reload,
 not a new package declaration.
Pi may store the package declaration relative to its settings directory.

## Development verification

From the repository root:

```sh
# Repository root
mise run //package/pi-plugin/openai-fast:build:js:node
mise run //package/pi-plugin/openai-fast:lint:types
mise run //package/pi-plugin/openai-fast:lint:oxlint
mise run //package/pi-plugin/openai-fast:test:unit
mise run //package/pi-plugin/openai-fast:verify:extension
mise run //package/pi-plugin/openai-fast:verify:live
mise run //package/pi-plugin/openai-fast:verify:installed
```

Tests use built artifacts,
 synthetic provider responses,
 and disposable state.
A native payload-construction check alone does not establish backend acceleration.
Live validation must use isolated pi settings and synthetic prompts,
 never real project files or shared session state.
`verify:installed` resolves the global declaration,
 preserves its resource filters,
 and exercises native relative package discovery in disposable settings.
The live tasks target legacy Codex.
The full offline suite also exercises native OpenAI dispatch and both namespaces through the built default factory.
Live probes do not prove acceleration,
 live OAuth refresh,
 network catalog refresh,
 or completed overflow compaction/retry.

## Source boundaries

- `catalog.ts` restores configured and cached metadata for both native providers without network access.
- `catalog-credentials.ts` supplies frozen empty read-only credential capabilities.
- `virtual-registration.ts` derives native virtual definitions and synchronizes catalog changes.
- `priority-provider.ts` creates keyless targets while leaving the original native provider untouched.
- `priority-target.ts` maps local target identities and omits authentication headers.
- `keyless-auth.ts` gates adapter readiness on fresh native source availability without a separate credential.
- `priority-stream.ts` reuses native option conversion and each original wire API's full streaming.
- `priority-payload.ts` composes payload callbacks while preserving the priority request.
- `original-dispatch.ts` owns the live registry binding and resolves original-model authentication and headers.
- `index.ts` registers the adapter and virtual entries without another request-mode switch.

Small named helpers are exported so artifact tests can exercise these boundaries.
The default export is the pi extension factory.

Design and evidence:
[accepted plan](../../../doc/planning/pi-openai-fast.md)
and [verified pi integration constraints](../../../doc/troubleshooting/pi-virtual-priority-options.md).

[codex-speed]: https://developers.openai.com/codex/agent-configuration/speed
