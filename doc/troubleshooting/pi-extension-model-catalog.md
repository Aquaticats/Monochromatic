# Pi 1.0.0 extensions cannot read, hide, or veto models, so hiding a model requires replacing its whole provider

## Symptom

An extension that wants to remove models from pi's catalog has no API for it,
and every
nearby mechanism silently does something else:

- Reading the catalog from an extension factory yields nothing.
  The factory's `pi` object
  has no registry,
  no model list,
  and no lookup method,
  so there is nothing to filter.
- `enabledModels` in settings looks like a hide list and is not one.
  The picker opens on
  the scoped view and one Tab keystroke shows every model again.
- `model_select` looks like an interception point and is a notification.
  A handler cannot
  refuse the selection,
  and the session has already switched by the time it runs.
- `pi --list-models` never fires an extension event,
  so nothing an extension does at
  `session_start` can change its output.
- `pi.registerProvider(name, { models })` does remove models,
  and also replaces everything
  else the provider's owner registered,
  including another extension's `streamSimple`
  handler and credentials.
- Filtering a provider whose models lack a `baseUrl` makes pi throw,
  and the throw aborts
  the extension's whole pass:

  ```text
  Provider azure-openai-responses: "baseUrl" is required when defining custom models.
  ```

  pi reports it as an `extension_error` record rather than failing the session,
  so a
  partially applied filter looks like success unless the record is read.

## Root cause

Version under test:
`@earendil-works/pi-coding-agent` 1.0.0 and `@earendil-works/pi-ai`
1.0.0,
resolved from
`node_modules/.pnpm/@earendil-works+pi-coding-agent@1.0.0_supports-color@10.2.2/`.

### The extension API exposes no model read

`ExtensionAPI` spans `dist/core/extensions/types.d.ts:1143-1363`.
Its only model members
are `setModel`,
`registerProvider`,
`unregisterProvider`,
`registerVirtualModel`,
`unregisterVirtualModel`,
and the `model_select` event.
A runtime dump of the factory
argument's own property names matches the declaration exactly:

```text
appendEntry, events, exec, getActiveTools, getAllTools, getCommands, getFlag,
getMcpServers, getSessionName, getSettings, getThinkingLevel, on, registerCommand,
registerEntryRenderer, registerFlag, registerMarkdownTransformer, registerMcpServer,
registerMessageRenderer, registerProvider, registerShortcut, registerTool,
registerVirtualModel, sendMessage, sendUserMessage, setActiveTools, setLabel, setModel,
setSessionName, setThinkingLevel, unregisterMcpServer, unregisterProvider,
unregisterVirtualModel
```

Registry reads live on the event context instead
(`dist/core/extensions/types.d.ts:225`):

```ts
modelRegistry: ModelRegistry;
```

so they are reachable only from a handler,
after the factory has returned.

### Model selection cannot be vetoed

`dist/core/extensions/types.d.ts:1179` declares `model_select` with no result type,
while
the neighbouring `tool_call` declaration at `:1181` has one:

```ts
on(event: "model_select", handler: ExtensionHandler<ModelSelectEvent>): () => void;
on(event: "thinking_level_select", handler: ExtensionHandler<ThinkingLevelSelectEvent>): () => void;
on(event: "tool_call", handler: ExtensionHandler<ToolCallEvent, ToolCallEventResult>): () => void;
```

`AgentSession.setModel` assigns the new model before emitting
(`dist/core/agent-session.js:1910-1926`),
so even a handler that could object would run
too late.

### `enabledModels` scopes the default view, it does not hide

`dist/modes/interactive/components/model-selector.js:39,52,119,201-219`:

```js
scope = "all";
...
this.scope = scopedModels.length > 0 ? "scoped" : "all";
...
this.activeModels = this.scope === "scoped" ? this.scopedModelItems : this.allModels;
...
setScope(scope) {
    if (this.scope === scope) return;
    this.scope = scope;
    this.activeModels = this.scope === "scoped" ? this.scopedModelItems : this.allModels;
}
```

The scope hint is bound to Tab (`:206`),
so the full catalog is one keystroke away.

### Settings are read-only from an extension

`pi.getSettings()` returns a structured clone
(`dist/core/settings-manager.js:305-307`),
and `SettingsManager.setEnabledModels()`
(`dist/core/settings-manager.d.ts:236`) is not reachable from `ExtensionAPI`.

### The only catalog mutation replaces a provider wholesale

`dist/core/provider-composer.js:171-178`:

```js
function applyExtension(providerId, models, config) {
    if (!config) return [...models];
    if (!config.models) {
        return config.baseUrl ? models.map((model) => ({ ...model, baseUrl: config.baseUrl })) : [...models];
    }
    return config.models.map((definition) => extensionModelFromDefinition(providerId, models, config, definition));
}
```

`extensionModelFromDefinition` (`:119-136`) throws when it cannot resolve an endpoint,
and
discards per-model headers for all three model types:

```js
const baseUrl = definition.baseUrl ?? config.baseUrl ?? defaults?.baseUrl;
if (!baseUrl)
    throw new Error(`Provider ${providerId}: "baseUrl" is required when defining custom models.`);
...
return { ...definition, api: api, provider: providerId, baseUrl, headers: undefined };
```

`ModelRegistry.getRegisteredProviderConfig` returns a configuration only for providers
registered through the configuration form.
A provider registered as a native object
appears in `getRegisteredProviderIds()` but has no readable configuration,
so an extension
that re-registers it replaces the owner's implementation with a plain catalog list.

### Two mutation paths exist, and the native one takes precedence

`ModelRuntime` keeps extension registrations in two maps,
and composition prefers the
native one (`dist/core/model-runtime.js:152`):

```js
const base = this.nativeExtensionProviders.get(providerId) ?? this.builtins.get(providerId);
```

`registerNativeProvider` fills that map from a whole `Provider` object
(`dist/core/model-runtime.js:626-634`):

```js
registerNativeProvider(provider) {
    if (!provider.id.trim())
        throw new Error("Provider id must not be empty.");
    this.extensionProviders.delete(provider.id);
    this.nativeExtensionProviders.set(provider.id, provider);
    this.recomposeProvider(provider.id);
```

The configuration form merges instead of replacing,
and deletes any native registration
(`dist/core/model-runtime.js:656-673`):

```js
this.nativeExtensionProviders.delete(providerId);
// Re-registration merges defined values over the previous registration and
// preserves undefined ones, matching the legacy ModelRegistry contract.
const previous = this.extensionProviders.get(providerId);
const effective = { ...previous };
for (const [key, value] of Object.entries(config)) {
    if (value !== undefined)
        effective[key] = value;
}
```

`ModelRegistry.getProvider(name)` returns the composed provider for any provider,
native or
builtin,
and a `Provider` is a plain object whose `getModels()` and optional
`getAllModels()` are ordinary methods (`pi-ai/dist/models.d.ts:57-95`).
Spreading one and
replacing only those two methods filters a provider without rebuilding any model metadata,
and `pi.registerProvider` accepts the result through its `Provider` overload
(`dist/core/extensions/runner.js:274`).

### `models.json` overrides are applied after extension replacement

`dist/core/provider-composer.js:337-353`:

```js
// models.json modelOverrides are the topmost user-config layer: they apply once,
// after custom-model upserts, extension model replacement, and legacy OAuth projection.
const getAllModels = () => {
    // Upstream is one line; wrapped here for width, tokens unchanged.
    let models = applyExtension(
        providerId,
        applyModelsJson(providerId, getAllProviderModels(base), config),
        currentExtension());
    ...
    return models.map((model) => {
        const override = config?.modelOverrides?.[model.id];
        return override && isModelType(model, "chat") ? applyModelOverride(model, override) : model;
    });
};
```

So a re-registered provider list cannot lose a user's `contextWindow` override.

### `getAll()` is chat-only

`dist/core/model-registry.d.ts:26-45` separates the reads:

```ts
getAll(): Model<Api>[];
getAvailable(): Model<Api>[];
...
/** Every known model of a type (chat, image, classifier), optionally for one provider. */
getModelsOfType<TType extends ModelType>(type: TType, provider?: string): readonly ModelTypeMap[TType][];
```

Measured against the bundled catalog:
1604 entries in total,
of which 1532 chat,
57 image,
and 15 classifier.
`getAll()` returned exactly 1532,
so a filter built on it deletes image
and classifier models on re-registration unless they are read separately.

### Startup model resolution precedes extension registration

`findInitialModel` runs in `dist/core/sdk.js:92-118` and `dist/main.js:657-674`,
before
`new AgentSession` flushes `pendingProviderRegistrations`
(`dist/core/agent-session.js:197`,
`dist/core/runner.js:248-267`).
A session can therefore
start on a model an extension is about to remove,
and
`_refreshCurrentModelFromRegistry()` keeps the removed model object when the lookup fails.

### No model carries age metadata

`BaseModel` (`pi-ai/dist/types.d.ts:900-912`) has `id`,
`name`,
`api`,
`provider`,
`baseUrl`,
`input`,
`inputLimits`,
`cost`,
and `headers`.
A sweep of all 42 bundled catalog
files found no per-model release date,
deprecation flag,
or version field.
The only
timestamp is catalog-wide,
in `pi-ai/dist/providers/data/.manifest.json`
(`generatedAt: "2026-10-01T18:57:11.882Z"`).

## Verification

Both harnesses below are read-only against pi settings and need no credentials.

Disposable host,
used by
`mise run //package/pi-plugin/model-retirement:verify:host`:

```bash
# agent directory holding only a models.json with one contextWindow override
node <pi>/dist/bundle/cli.js --mode rpc --no-extensions \
  --extension src/verify-probe.ts \
  --extension dist/final/node/index.mjs < /dev/null
```

RPC mode starts a session and exits once stdin closes,
which makes `session_start` and
`resources_discover` observable without calling a model.
The probe loads first,
so its
`session_start` snapshot precedes the filter and its `resources_discover` snapshot follows
it.

Real host,
same command without `--no-extensions`,
so all configured packages load.

### What works

- Filtering a builtin provider:
  `openai-codex` went from 9 chat models to 6,
  with
  `gpt-6-sol` removed,
  `gpt-6.1-sol` kept,
  and `gpt-6-luna` still reporting the overridden
  `contextWindow` 750000 against a bundled 272000.
- Preserving a configuration-registered provider:
  `radius` went from a 2-key
  configuration with `streamSimple` to a 3-key configuration with `streamSimple`,
  the
  added key being the filtered model list.
  `synthetic` kept all 4 keys,
  credentials
  included.
- Wrapping a native registration:
  `hyper`,
  whose configuration
  `getRegisteredProviderConfig` does not return,
  went from 23 chat models to 17 with
  `stream`,
  `streamSimple`,
  and `auth` intact and still registered as native.
- Wrapping a builtin whose models carry no endpoint:
  `azure-openai-responses` went from
  44 chat models to 21,
  with `gpt-4.1` removed and `gpt-5.5` kept.
- Keeping `models.json` overrides through a wrapper:
  `openai/gpt-6-luna` and
  `openai/gpt-6.1-sol` reported their configured 750000 both before and after a pass that
  wrapped `openai` as a native registration.
- Passing image and classifier models through:
  57 and 15 before and after.
- Metadata fidelity:
  a filtered model still carries `thinkingLevelMap` (7 levels),
  `compat`,
  `promptCache`,
  `samplingParams`,
  and `inputLimits`.
- Per-provider isolation:
  after one provider threw,
  the remaining 34 still filtered.

### What fails

- Any catalog read from the factory:
  no property exposes one.
- Any veto of a model selection:
  `model_select` has no result channel.
- Any change to `pi --list-models` output:
  the command fires no extension event,
  and in a
  credential-free host it prints `No models available` because availability is
  auth-filtered while `getAll()` is not.
- Re-declaring a native registration as a configuration:
  `hyper`,
  `openai-fast`,
  and
  `openai-codex-fast` return no configuration,
  so a configuration registration replaces
  their owner's implementation.
  Wrapping the composed provider works instead.
- Re-declaring `azure-openai-responses` as a configuration:
  all 44 of its chat models
  carry an empty `baseUrl`,
  and pi throws.
  Wrapping works instead.
- Preserving per-model `headers` through a configuration registration:
  `extensionModelFromDefinition` sets them to `undefined`.
  In the bundled catalog this
  affects `github-copilot` (34 models) and `nvidia` (19).
  Wrapping preserves them,
  since
  it passes model objects through by reference.

## Verified workarounds

All five are implemented in `package/pi-plugin/model-retirement`.

1.  Filter at `session_start`,
    the earliest event carrying `ctx.modelRegistry`.
    Tradeoff:
    the startup model choice and `pi --list-models` are both settled before the
    filter runs,
    so neither reflects it.
    A session can start on a model the pass then
    removes,
    and keeps running on it.
2.  Merge a configuration where `getRegisteredProviderConfig` returns one,
    since pi merges
    defined values over the previous registration and so preserves that owner's `apiKey`,
    `oauth`,
    and `streamSimple`.
    Tradeoff:
    the model list must be rebuilt from registry reads,
    which drops per-model
    `headers` and needs an `api` and `baseUrl` per model.
3.  Wrap the composed provider from `getProvider` everywhere else,
    spreading it and
    replacing only `getModels` and `getAllModels`.
    Tradeoff:
    the wrapper becomes that provider's native registration,
    so a later
    configuration registration by its owner would apply on top of the wrapper.
    In exchange
    it needs no endpoint metadata,
    loses no model field,
    keeps per-model headers,
    and
    filters live,
    so a catalog refresh is filtered on the next read.
4.  Validate a configuration plan's endpoints first,
    counting provider-level `api` and
    `baseUrl` as fallbacks the way pi does,
    and fall back to wrapping when a model cannot be
    re-declared.
    Tradeoff:
    a provider is all-or-nothing on the configuration path,
    so without the
    fallback one unregisterable model would keep every superseded model in that provider
    visible.
5.  Isolate each registration in its own `try` and report the caught value.
    Tradeoff:
    a failed provider is logged rather than fatal,
    so a partial pass is the
    normal failure mode and the warning lines are the only evidence.

Awaiting `ModelRegistry.refresh({ allowNetwork: false })` before reading is cheap and
matches pi's own instruction on that method (`dist/core/model-registry.d.ts:25`,
"Await
before making synchronous registry reads"),
but it changed nothing in either host:
the
override was already applied in the disposable host and already absent in the real host,
because the override for that provider no longer exists in the local `models.json`.

## What does not work

- `refreshModels` as a filtering hook.
  Its context
  (`pi-ai/dist/models.d.ts:13-31`) exposes `credential`,
  `stored`,
  `publish`,
  `allowNetwork`,
  `force`,
  and `signal`,
  but not the composed model list,
  so there is
  nothing to filter.
- `pi.registerCommand("model", ...)` to replace the picker.
  `/model` is handled in
  `dist/modes/interactive/interactive-mode.js:2505-2510` and returns before
  `session.prompt()` dispatches extension commands
  (`dist/core/agent-session.js:1490`),
  so the registration is shadowed in TUI mode.
- Writing `settings.json` from the extension to change `enabledModels`.
  It cannot hide
  anything,
  per the `model-selector.js` trace,
  and pi offers no settings write API.
- `session_start` plus `pi.setModel()` as a hiding mechanism.
  It changes one session's
  model,
  not the catalog,
  and it swaps a model the user chose.
- Reading the bundled catalog from `@earendil-works/pi-ai` inside the factory instead of
  using the registry.
  It sees neither `models.json` overrides nor models other extensions
  registered,
  so the list written back would be stale for both.
- A `models.json` edit as a hiding mechanism.
  `applyModelsJson` runs before
  `applyExtension`,
  so an extension-registered provider replaces whatever `models.json`
  added.
  This repo recorded that earlier in
  `doc/troubleshooting/pi-synthetic-provider-startup-model-scope.md`.

## Upstream filing decision

Default policy is not to file,
and all six constraints are not met.

1.  Really upstream's fault?
    Mostly no. The event contracts,
    the `registerProvider`
    replacement semantics,
    and the `models`-required fields are documented behaviour,
    and
    `docs/extensions.md` names the exported declarations as the contract.
    The genuine gap
    is narrow:
    an extension can replace a provider's models but cannot read them at the
    only point where replacing them is early enough.
2.  Could upstream fix it?
    Yes,
    by exposing a registry read on `ExtensionAPI`,
    or by firing
    an event with the registry before model resolution.
3.  Are they supporting this use case?
    Partly.
    `docs/virtual-models.md` and the
    `registerProvider` examples cover adding and routing models,
    not removing them,
    and no
    bundled example filters a catalog.
    `examples/extensions/model-status.ts` observes
    `model_select` for a status bar only.
4.  Would the repo welcome a contribution?
    Not established.
    No `CONTRIBUTING.md` check was
    run for this doc,
    and no policy search was performed,
    so this constraint is unverified
    rather than passed.
5.  Would they likely fix it?
    No signal either way;
    no tracker search was run.
6.  Is there a prototyped minimal fix compatible with their architecture?
    No. The
    workaround lives entirely on this side of the boundary,
    and no patch against pi was
    written.

Constraints 4,
5,
and 6 fail or are unverified,
so nothing is filed.
If a future session
wants to file,
constraint 6 needs a patch against
`packages/coding-agent/src/core/extensions/types.ts` exposing a read-only catalog accessor
on `ExtensionAPI`,
plus a test showing a factory-time filter reaching `--list-models`.
