# Pi OpenAI fast extension design interview

## Status

Design interview requested with `/grill-me`.
The user answered "Confirm."
 to the complete design and request-only backend contract.
Implementation and verified global replacement are now authorized.
This document tracks the accepted requirements,
 evidence,
 and implementation progress.

An additional `/grill-me` interview requested support for the non-legacy `openai` sign-in.
The original confirmation applies to the delivered Codex extension.
The user subsequently answered "Yes."
 to Q9,
 confirming the minimal additive design and authorizing the additional native provider mapping.

## Requested behavior

- Create a pi extension for OpenAI fast selection.
- Send the priority service tier.
- Use virtual models.
- Correct misunderstandings about pi before implementing.

Virtual models mean pi's native `pi.registerVirtualModel()` entries,
 not just a physical provider with companion model IDs.
Each selection ultimately uses an existing upstream Codex model ID.
The user accepted Codex-login-only authentication,
 opt-in fast companions,
 and global replacement of the incumbent extension after verification.

## Existing environment

Read-only inspection of the global `~/.pi/agent/settings.json` found:

- `npm:pi-openai-codex-fast` is already installed.
- The enabled model scope includes `openai-codex/gpt-6-sol`,
   `openai-codex/gpt-6-luna`,
   and `openai-codex/gpt-6-astra`.
- The configured transport is `auto`.
- Local pi-plugin packages are installed by their repository paths.

The project [pi settings README](../../.pi/README.md) reserves user-specific workflow packages
for global settings rather than project settings.
The installed pi documentation inspected for initial context reports version `0.99.2`.

A pre-existing modification to `pnpm-lock.yaml` existed before this work.
The user later explicitly authorized including that update because it supplies
prerequisite pi `0.99.2` and Node-type lock entries for the new extension.

## Work areas

- Provider and virtual-model registration:
   investigate pi documentation,
   installed types,
   authentication reuse,
   request translation,
   and transport preservation.
- Priority service semantics:
   investigate current OpenAI documentation and the installed fast extension.
- User-facing selection and installation:
   settle preferences through the design interview.

Read-only research completed in separate evidence notes:
[provider research](pi-openai-fast-provider-research.md)
and [service research](pi-openai-fast-service-research.md).
The main agent inspected the incumbent source and relevant native types and implementations,
 corrected physical-versus-virtual terminology,
 and ran disposable fixture probes.

## Interview frontier

### Accepted round-one answers

The user answered `Q1 A`,
 `Q2 A`,
 and `Q3 A`.

- Authentication:
   reuse the existing Codex login;
   OpenAI API-key support is out of scope for the initial extension.
- Defaults:
   fast companion models remain opt-in;
   preserve ordinary model entries and the existing default model.
- Delivery:
   build and verify the repository package,
   then install it globally and replace `pi-openai-codex-fast`.
   Check migration implications and competing hooks before replacement.

These answers settle requirements,
 not the complete implementation design.
The user subsequently confirmed shared understanding of the complete design in Q7.

### Accepted round-two answers

The user answered `Q4 A` and `Q5 A`,
 with a correction to the registration proposal.

- Leave `enabledModels` unchanged during installation.
- Register fast companions for every base Codex model pi registers,
   independently of personal scope.
- Do not manually maintain a compatibility list or filter companions by declared priority support.
- Treat fast support as a request-time assumption,
   not a verified statement that every backend model accepts priority.
- The user considers selecting an incompatible model in fast mode a user error.
  Surface the backend rejection rather than silently selecting a different model or ordinary tier.

The initial recommendation's word "compatible" is superseded:
there is no manually curated priority-capability gate.
Registering a companion does not authorize scope widening or establish backend support.

### Accepted selection-feedback answer

The user answered Q6:
"Don't need any."

Do not add selection indicators,
 footer widgets,
 request-status notifications,
 or a served-tier reporting UI.
The virtual models still need distinct selectable identities,
 but no additional feedback feature is requested.
Normal internal logging requirements remain in effect.

## Local selection evidence

`package/pi-shared/model-selection/src/scope-patterns.ts` resolves scope by model identity.
A disposable fixture probe called the real `resolveModelPatterns()` implementation:

- Positive control:
   `openai-codex/gpt-6-sol*` selected the ordinary fixture and its `-fast` companion.
- Existing exact entry:
   `openai-codex/gpt-6-sol` selected only the ordinary fixture.

The probe also called the real `scoreModelSpeed()` from
`package/pi-shared/model-selection/src/speed-signals.ts`.
The ordinary fixture scored `0`;
 the fixture with a `-fast` suffix and `Fast` display-name suffix scored `90`.
These scores are local name heuristics,
 not measured backend speed.
`package/pi-plugin/auto-mode/src/budget-model.ts` uses that scoring to sort eligible automatic judge candidates.

The first-round wording that opt-in always requires explicit selection was incomplete:
preserving the default model alone does not prevent automatic auxiliary selection.
Scope inclusion and auxiliary-selection eligibility must also be considered.
No aliases have been implemented or added to the user's scope.

Independent Advisor review identified the same boundary and highlighted session migration,
 transport preservation,
 authentication refresh,
 and requested-versus-served tier observability.

## Verified pi distinctions

The incumbent `pi-openai-codex-fast@0.0.17` registers physical companions,
 not native virtual models,
 and uses a manually maintained allowlist.
Native virtual routing returns physical model,
 thinking level,
 and optional state;
 it has no request-options return channel.
Pi resolves the physical target from the catalog and discards changes to a cloned route target.

A disposable `ModelRuntime` fixture verified:

- A genuine virtual selection has API `pi-virtual`.
- A physical routing target can be omitted from `getAvailable()` through a native provider filter,
   while a virtual entry remains available and routes to it.
- A cloned target's changed API and an extra route `serviceTier` value do not reach the resolved request.

A second probe invoked the real native Codex adapter,
 with synthetic authentication and a payload callback that stopped before networking.
The full native stream constructed `service_tier: "priority"`.
Native `streamSimple` discarded an extra top-level `serviceTier` value during option conversion.
The exported `buildBaseOptions()` preserved the abort signal,
 `transport: "auto"`,
 and session ID.
These are offline construction and routing probes,
 not a completed extension or live backend verification.

## Accepted implementation design

- Register genuine virtual entries using the incumbent
   `openai-codex-fast/<base-model-id>` identities to preserve existing selections.
- Enumerate every registered base Codex model without a priority compatibility list.
- Route fast entries through internal physical priority targets that are excluded from availability lists.
- Keep ordinary Codex model entries,
   OAuth ownership,
   configuration overrides,
   catalog refresh,
   and ordinary streaming behavior intact.
- Translate internal targets back to the live upstream base identity before native streaming.
- Reuse the native simple-option conversion helper and reasoning mapping,
   then call native Codex streaming with priority request options.
   Preserve asynchronous payload callbacks,
   callback replacement payloads,
   response instrumentation,
   cancellation,
   timeouts,
   retries,
   cache behavior,
   headers,
   environment,
   and transport selection.
- Preserve canonical base-model assistant history while pi persists the virtual selection separately.
- Preserve defaults and `enabledModels`.
- Add no extension-authored feedback UI.
  Pi's normal model display and virtual-routing presentation remain host behavior.
- Build the package at `package/pi-plugin/openai-fast`,
   verify it through the host,
   then replace the incumbent globally without loading conflicting physical and virtual identities together.

The filter probe establishes availability-list exclusion,
 not invisibility in every registry API or host display.
Provider composition,
 OAuth reuse,
 request dispatch,
 refresh synchronization,
 session recovery,
 reasoning,
 tool calls,
 accounting,
 and transport behavior need candidate verification.
Independent Advisor review required those checks and warned against global mutable fast-state tagging.

## Backend contract and remaining evidence gaps

The extension's intended guarantee is requesting `service_tier: "priority"`,
 not guaranteed acceleration or confirmed priority execution.
Do not silently fall back to a different model or tier after a rejection.
A server-side outcome is distinct from client fallback.

[OpenAI's Codex speed documentation][codex-speed] documents increased subscription-limit consumption for Fast.
A [pi maintainer comment][pi-fast-comment] questioned Fast effectiveness in non-Codex harnesses.
Neither source establishes this login's current backend behavior through pi.
Actual acceptance,
 served-tier interpretation,
 latency,
 entitlement,
 and quota consumption have not been measured here.
The user does not want a reporting UI or compatibility catalog to manage those uncertainties.

## Verification obligations after confirmation

- Validate startup,
   explicit model selection,
   scope preservation,
   reload,
   catalog changes,
   and incumbent-session migration in disposable host fixtures.
- Show normal requests retain their normal tier path and fast requests use the original model ID plus priority.
- Exercise asynchronous payload replacement,
   reasoning levels,
   tool follow-ups,
   overflow recovery,
   aborts,
   retry controls,
   and native transport delegation.
- Verify OAuth refresh and logout with disposable credentials before touching the real installation.
- Verify canonical history and virtual selection through resume,
   fork,
   branch navigation,
   and extension removal.
- Run scoped package lint,
   type checking,
   tests,
   build,
   and consumer-facing host verification.
- Perform a live Codex smoke request after implementation without claiming that success alone proves acceleration.

## Design-stage verification

The scoped `mise run lint:markdown` command passed for this plan,
 both research notes,
 and [the troubleshooting record](../troubleshooting/pi-virtual-priority-options.md).
Local renderer tests passed for those documents and confirmed that the embedded troubleshooting harnesses
match the executed scratch programs.
The routing and native-adapter fixture tests passed without live inference.

Independent reviews were used to check selection-consent boundaries
and the proposed native virtual routing architecture.
At the design-stage checkpoint,
 no extension source,
 global settings,
 default model,
 or enabled-model scope had been changed.
Implementation began only after Q7 confirmation.

## Implementation progress

- Package metadata,
   catalog bootstrap,
   genuine virtual registration,
   and native priority transport have been committed.
- Independent review identified double-configuration and header-precedence risks in an original-provider overlay.
   That implementation was replaced with a keyless adapter under `openai-codex-fast`.
   The original `openai-codex` provider is never replaced.
- Internal target metadata omits request-auth headers;
   injected original-registry dispatch receives caller overrides without inherited adapter auth headers.
- The adapter reads only the live original provider's catalog,
   never recursively enumerating itself.
   Structural registration uses an owned reentrancy latch and scope disposal.
- A frozen capability factory privately owns the session registry binding,
   and actual inference before `session_start` is explicitly rejected.
- Catalog bootstrap uses an empty read-only credential capability and cached-only refresh for the original provider.
   Native cache restoration precedes auth/network phases.
   No real stored credentials or network refresh are used.
- Priority dispatch is bound to the live host's original-model registry path,
   so model-specific headers and native OAuth resolution remain at the original identity.
- Native streaming/payload modules and host lifecycle tests have been captured and committed.
   Test fixture cleanup is being handled independently for host and stream tests.
- The keyless adapter exposed an incorrect early token requirement in the simple-stream bridge.
   Injected registry dispatch now resolves auth later;
   direct native dispatch retains the missing-token error.
- The initial type check identified exact optional-property omissions;
   absent options are now omitted rather than supplied as `undefined`.
   Built-artifact imports require the first package build before final type verification.
- A filtered offline pnpm install added only the generated `openai-fast` importer,
   leaving the pre-existing lockfile update unchanged.
- The user selected option A to authorize committing that pre-existing update with the generated importer.
   Commit `a2b216772` records that dependency prerequisite and the authorization.

The initial package build passed;
 subsequent test-fixture typing and assertion issues were corrected.
The current build,
 type check,
 complete offline suite,
 lint,
 and extension-host verification all pass against pi `1.0.0`.
Native adapter readiness now uses an explicit side-effect-free auth check.
Host investigation found that provider-scoped availability does not populate its auth snapshot,
 and availability-only refresh can be superseded by registration-triggered cached refreshes.
The fixture now awaits `ModelRuntime.refresh({ allowNetwork: false })`,
 matching real CLI initialization.
That readiness control passed and native request,
 auth,
 history,
 tool,
 and overflow-recognition scenarios reached their intended boundaries.
Independent review corrected catalog inventories,
 native wrapper reference identity,
 and raw-code-versus-message assertions without weakening the request contract.

The guarded `verify:live` task passed through the real pi CLI for ordinary and fast Luna selections.
Both returned the expected marker.
Captured native request metadata used the original `gpt-6-luna` ID,
 with priority only for the fast selection.
The verifier used disposable settings and the existing access token through an environment override,
 without copying or refreshing the real credential store.
This proves live request intent and response handling,
 not acceleration or live OAuth refresh.
The recursion-rejection control also emitted the expected `LiveVerificationError` before credential access.
Allowed/rejected disposable fixtures and a removed-guard scratch control passed,
 showing the guard assertion depends on the guard.

Global replacement has been performed using the native pi package commands.
The old `npm:pi-openai-codex-fast` declaration and npm package were removed.
The repository replacement is now declared as a native relative local path.
The installation task's non-package settings comparison passed.
Its initial absolute-path assertion then failed before comparing unrelated package declarations;
 that assertion was corrected using native source/path semantics.
Do not claim that the interrupted comparison verified the original unrelated-package baseline.

Native npm uninstall reported changing 80 packages and removing the incumbent.
Its log identifies affected active npm extensions,
 including process management,
 provider integrations,
 Radius,
 subagents,
 and BTW.
This is dependency reconciliation,
 not proof that versions remained unchanged.
No `npm audit fix` was run.
The reported audit summary was 14 vulnerabilities:
 1 low,
 2 moderate,
 and 11 high.

The installed replacement passed ordinary and fast Luna requests through native package discovery.
A subsequent probe preserved package filters and exercised a rebased relative declaration.
Disposable installation controls passed for string and versioned filtered-object incumbents.
They retained non-package settings and unrelated package declarations.
Relative native discovery controls passed for allowed,
 disabled,
 wrong,
 and missing package declarations.
All active affected npm package factories loaded without extension errors in credential-free offline state.
This does not constitute complete behavioral testing of those other extensions.

## Pi 1.0 target update

The user reported that pi updated to `1.0` before global installation.
Installed package probes found coding-agent and pi-ai `1.0.0`,
 and the new package's peer links now point to those versions.
The native virtual routing,
 custom-provider,
 and extension contracts remain present in the installed 1.0 documentation and declarations.
The current package build,
 type check,
 and complete offline tests passed on pi `1.0.0` without a source compatibility change.
Lint completed with zero warnings and zero errors across 46 files.
Both ordinary and fast live probes passed on `1.0.0`,
 including native installed-package discovery.
The original model ID was retained;
 only fast requests supplied priority.
This does not establish acceleration.
Earlier `0.99.2` evidence is historical,
 not proof for the updated host.

A fresh `pnpm-lock.yaml` modification appeared with the upgrade and is concurrent work.
Do not stage it as an extension source change.

## Communication correction

An unsupported usage estimate was retracted after the user's correction.
Task completion is determined by accepted requirements and verification,
 not an unmeasured token or context count.

Proposed `AGENTS.md` tightening:
 replace `QJ1` with the following wording,
 retaining its measurement requirement and adding an explicit completion boundary:

> QJ1:
> Measure sizes,
> counts,
> usage,
> and timings before quantitative claims or adjectives.
> Omit unbuilt-fix estimates.
> Unmeasured token or context usage is not a stopping condition.

## Verification limits

- No claim of backend acceleration or confirmed served priority.
- Live probes reused a valid access token rather than exercising a live OAuth refresh.
- Catalog-change controls used native fixture refresh,
   not a real network catalog refresh.
- History tests cover persisted JSONL resume and branching;
   cold-runtime restored-session dispatch remains unverified.
- Overflow recognition was verified;
   completed summary compaction and automatic retry remain unverified.

## Delivery checkpoint

Implementation and global replacement are complete on pi `1.0.0`.
Build,
 types,
 lint,
 offline tests,
 native extension verification,
 live ordinary/priority probes,
 and disposable installation/discovery controls passed.
Rendered documentation and the exact embedded installation reproduction also passed.
The scoped Markdown check passed after semantic line-wrap corrections.

Restart pi to load the replacement,
 then select `openai-codex-fast/<model-id>` in its normal model picker.
Defaults and exact enabled-model scope were retained.
Native npm removal reconciled other dependencies;
 affected active factories loaded successfully,
 but their complete behavior was not audited.
The verification limits remain explicit.

## Non-legacy OpenAI sign-in interview

### Requested addition and retained constraints

The user requested that the existing extension also map the new OpenAI provider sign-in.
Treat this as additive:
retain `openai-codex-fast/<model-id>` routing to its original legacy provider,
 and propose `openai-fast/<model-id>` routing to the native `openai` provider.
Do not silently repoint saved legacy selections or copy credentials between providers.

Retain the accepted native virtual-model design,
 unchanged defaults and model scopes,
 original-provider authentication ownership,
 no extra selection UI,
 no compatibility allowlist,
 and no model or tier fallback.
The guarantee remains requesting priority,
 not demonstrated acceleration or confirmed served priority.

### Installed-source evidence

`pi --version` reports `1.0.0`.
The extension's peer links resolve to installed pi-ai and coding-agent `1.0.0`.
Within `package/pi-plugin/openai-fast/node_modules/@earendil-works/pi-ai/dist/`:

- `providers/openai-codex.js:9` names the existing provider `OpenAI Codex (legacy)`.
  It retains `openai-codex-responses` and the ChatGPT backend endpoint.
- `providers/openai.js:8` registers provider identity `openai`.
  Lines 10 to 21 declare the public OpenAI endpoint,
   API-key authentication,
   and the native `Sign in with ChatGPT` OAuth option.
- `auth/oauth/openai-chatgpt.js:256` defines native login,
   token refresh,
   and access-token resolution for the new sign-in.
- `api/openai-responses.js:23` distinguishes ChatGPT sign-in using original provider identity,
   endpoint,
   and resolved credential.
  The extension must preserve that native identity during dispatch.
- `api/openai-responses.js:254` maps native `serviceTier` into the request's `service_tier` field.
  This establishes request construction,
   not backend priority support through the new sign-in.

The extension currently hardcodes the legacy source,
 companion namespace,
 and Codex transport in `src/constants.ts`,
 `src/original-dispatch.ts`,
 `src/priority-target.ts`,
 and `src/priority-stream.ts`.
Changing only the provider string would not preserve the new native request path.

### Accepted authentication-scope answer

The user answered `Q8 B`:

> We don't do special handling with API-key billing and verification.
> We don't explicitly reject API-key-backed fast requests.
> Keep the changes as minimum as possible.

Both native ChatGPT sign-in and API-key authentication may back `openai-fast` requests.
Delegate to the native provider without authentication-method restrictions,
 copied credentials,
 billing logic,
 or a separate API-key verification campaign.

The subscription-only recommendation is rejected.
The suggestion that supporting both methods requires dedicated API-key billing verification is superseded
by the user's explicit constraint.
Normal scoped build,
 lint,
 type,
 test,
 and consumer-host checks remain part of implementation verification.

### Confirmed minimal additive design

- Retain `openai-codex-fast/<model-id>` mapped to the legacy `openai-codex` provider.
- Add `openai-fast/<model-id>` mapped to the native `openai` provider.
- Reuse the existing virtual registration,
   internal-target,
   and priority-payload mechanisms with the minimum provider-specific changes.
  Do not introduce a new authentication or billing layer.
- Preserve each original provider's identity,
   transport API,
   effective model metadata,
   credential resolution,
   and native request behavior during dispatch.
- Preserve defaults,
   model scopes,
   catalog-derived companion coverage,
   no additional UI,
   and no model or tier fallback.
- Keep `service_tier: "priority"` as request intent,
   not a promise of acceleration or confirmed served priority.

Independent Advisor review confirmed that final shared-understanding confirmation was the sole remaining frontier.
The user answered "Yes."
 to Q9 and authorized this design.
The original provider identity must survive dispatch.
Ordinary regression checks do not require a separate live API-key or billing campaign.

### Delivery and verification boundaries

- Provider mapping is complete:
   the additional native provider mapping reuses the existing registration and request machinery.
- Regression verification is complete:
   package-scoped build,
   types,
   lint,
   tests,
   and consumer-host checks passed.
- Documentation is updated and verification results are recorded.
- Existing scoped tests and host fixtures were extended for the additional mapping,
   identical model IDs across providers,
   native priority payloads,
   and unchanged legacy and ordinary requests.
- Existing Codex live evidence does not verify the new native sign-in.
  Do not claim new-provider live acceptance,
   served priority,
   or acceleration without a corresponding probe.
- No real authentication or personal settings changes are required.
  The existing globally installed local package loads its rebuilt artifact.

The scoped `mise run lint:markdown -- doc/planning/pi-openai-fast.md` check passed.
A local `marked` render confirmed the interview headings,
 companion identity code spans,
 preserved reference link,
 and absence of unintended emphasis or tables in the added section.
The initial provider mapping is committed in `1ed0a2517`.
Its package-scoped build and type check passed before the new native-provider regression fixture was added.
The additional fixture exercises native request construction,
 source-bound authentication delegation,
 equal upstream IDs across providers,
 caller overrides,
 and ordinary/priority separation without external HTTP.
The first full test run passed the new native dispatch and direct-stream checks,
 plus the existing legacy scenarios.
The configured-catalog test failed because its exact-equality assertion
did not account for native optional-property normalization.
It now compares the configured model's declared metadata after resolving its identity.
The first lint run reported declaration,
 foreign-boundary,
 documentation,
 and formatting findings;
 these are being corrected without rule changes.
A disposable CLI startup check now exercises the built default factory rather than manual registration.
Installed `openai-responses.js:177` confirms that native simple streaming uses `buildBaseOptions`,
 passes through `toolChoice`,
 and converts clamped reasoning into `reasoningEffort`.
The shared priority bridge retains that conversion;
 the new host assertions check serialized high reasoning and exact default/scope preservation.
The rebuilt package type check passed.
Source lint then passed with zero warnings and zero errors across 48 files.
The native catalog inventory found 44 `openai` chat models,
 all using `openai-responses`,
 and 9 legacy models,
 all using `openai-codex-responses`.
This verifies the installed default catalog,
 not arbitrary user overrides.

The CLI startup fixture discovered both namespaces but also exposed the repository logger's warning:
`3328 startup records dropped before a backend verified (buffer cap 10000)`.
The exact emitter is `package/logger/core/src/create-logger.ts:393`.
Its startup buffer holds records until asynchronous sink verification settles.
The default extension factory now awaits its tagged logger's existing `flush()` before synchronous catalog registration.
This retains logging rather than suppressing stderr or raising the buffer limit.
The committed fixture failed before this lifecycle correction;
 the corrected startup result is pending.

A reasoning assertion was corrected to check the exact `reasoning.effort` property,
 without rejecting native `reasoning.summary: "auto"`.
The corrected package-scoped build,
 type check,
 complete offline suite,
 and source lint passed.
The real CLI startup check now discovers both companion namespaces without stderr diagnostics.
Scoped Markdown lint and local rendered-document checks passed.
A read-only global settings probe confirmed that the installed relative declaration
still resolves to this local package.
No reinstall,
 real authentication change,
 or personal settings edit was needed.
The final `mise run //package/pi-plugin/openai-fast:verify:extension` task also passed.
The rebuilt installed local package is ready for pi reload or restart.
No live new-provider verification has been performed.

## Source-configuration visibility correction

The user reported that a fast virtual provider remains visible when its own base provider is unconfigured.
This is a defect in the extension's readiness declaration,
 not a request to restrict authentication methods or change billing behavior.

A committed CLI matrix in `d60df7fae` reproduces the symptom in disposable homes:

- Both sources configured:
   the control passes.
- Only native OpenAI configured:
   the legacy fast provider incorrectly appears.
- Only legacy Codex configured:
   the native fast provider incorrectly appears.
- Neither configured:
   both fast providers incorrectly appear.

The reproduction command rebuilds the package,
 checks types,
 and runs `startup.unit.test.ts` through the scoped task.
Inherited provider credentials are excluded from the child environment.

Installed pi-ai `dist/models.js:262` authenticates providers before applying availability filters.
Installed coding-agent `dist/core/virtual-models.js:114` retains virtual entries after filtering physical models.
The extension's `src/keyless-auth.ts:19` declares readiness unconditionally.
Fresh CLI homes disprove stale user cache as the cause of the startup symptom.

The correction will make adapter availability depend on a fresh native source-only check.
Startup will use pi's native authentication/configuration handling;
 after session binding,
 the active registry will own that check.
Request authentication remains in the original provider.
No fast credential will be copied or stored.

The earlier verification covered authenticated sources but omitted unconfigured-source visibility.
Proposed `AGENTS.md` tightening of the existing test-coverage rule:

> TCV:
> Tests cover every branch and configured/unconfigured states,
> not just happy paths;
> compare test names against branches and availability conditions before claiming completeness.

Readiness gating is committed in `bea0d23af`.
The original CLI matrix now passes all configured/unconfigured cases.
Both startup and bound-session checks query source-only chat availability using native authentication handling.
The startup check uses the original native auth path;
 the bound check uses the active registry,
 so custom in-memory credentials and runtime configuration remain host-owned.
Metadata-only catalog bootstrap stays credential-free.

The expanded fixtures cover stored OAuth,
 stored API keys,
 environment keys,
 configuration keys,
 command-backed keys without execution,
 and source logout/reconfiguration.
The expanded CLI matrix passed stored OAuth,
 stored API keys,
 environment keys,
 configuration keys,
 and command-backed keys without executing the command or changing auth-file bytes.
The full suite exposed a stale logout assertion:
 after availability is correctly removed,
 pi rejects the stale virtual route with `which has no credentials`
 before reaching original-provider request authentication.
The assertion now checks that native routing diagnostic and still forbids dispatch or fallback.
Lint findings were resolved with native callback return-type aliases,
 explicit fixture scope,
 literal-space parsing of CLI columns,
 and declaration formatting,
 without suppressions or rule changes.
The fresh rebuilt package type check,
 complete offline suite,
 and source lint passed.
The bound-session tests verify disappearance after source logout,
 independent native/legacy visibility,
 and reappearance after source credentials are restored.
The final extension-host task also passed.
The source-configuration visibility correction is complete.
Scoped Markdown lint and rendered checks passed for the package README,
 planning record,
 and troubleshooting record.
The source trace and verified credential matrix are documented in
[the troubleshooting record](../troubleshooting/pi-virtual-priority-options.md#pi-10-fast-providers-remain-visible-without-source-configuration).

[codex-speed]: https://developers.openai.com/codex/agent-configuration/speed
[pi-fast-comment]: https://github.com/earendil-works/pi/issues/6738#issuecomment-4995103821
