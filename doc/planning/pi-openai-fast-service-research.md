# Pi OpenAI Fast service research

Research date:
 2026-10-01.
Purpose:
 support design confirmation,
 not implementation.
No credentials,
 inference calls,
 configuration edits,
 or commits were involved.

## Settled requirements

- Codex login only.
- Opt-in companions for every base Codex model.
- Existing defaults and `enabledModels` unchanged.
- No manually maintained model compatibility list.
- Request Fast assuming acceptance;
   incompatible selection is a user error.
- No silent client fallback to another tier or model.
- Global replacement only after verification and design confirmation.

Server-side downgrade is distinct from client fallback.
Whether Codex can reveal or prevent such downgrade remains unverified.

## Installed incumbent: verified source evidence

`~/.pi/agent/npm/node_modules/pi-openai-codex-fast/package.json:3` identifies version `0.0.17`.
Its [source][incumbent-source] provides physical companion selections:
`openai-codex-fast/<original-model-id>` delegates to built-in `openai-codex`.
`index.ts:266` registers a separate physical provider with `pi.registerProvider()`.
It does not use pi's native `pi.registerVirtualModel()` API.

Unlike the settled design,
 `index.ts:26` maintains a compatibility allowlist.
It includes the currently scoped `gpt-6-sol`,
 `gpt-6-luna`,
 and `gpt-6-astra`,
but only listed models are copied.
`index.ts:98` copies base-model cost metadata.

`index.ts:217` sets the request intent:

```ts
// pi-openai-codex-fast/index.ts:217
const requestOptions: OpenAICodexResponsesOptions = {
  ...options,
  apiKey: auth.value,
  serviceTier: "priority",
};
```

`index.ts:58` resolves existing Codex auth;
it does not itself inspect credential type or enforce OAuth-only access.
`index.ts:225` delegates to `streamOpenAICodexResponses` without overriding transport.
There is no tier/model fallback or served-tier enforcement in this wrapper.

The [README][incumbent-readme] documents canonical `openai-codex` assistant history,
except context-overflow errors required for recovery.
It restores Fast selection from the latest overall `model_change` on `session_start`,
but does not reconcile branch switches through `session_tree`.
Its documented live tests were not run here.

## Codex priority semantics: documented intent, not verified service

OpenAI's [Codex source][codex-tier-source],
`codex-rs/protocol/src/config_types.rs`,
implements `ServiceTier::request_value` as:

```rust
// codex-rs/protocol/src/config_types.rs, ServiceTier::request_value
Self::Fast => "priority",
```

This is executable client mapping,
 not an inference from SDK types.
[`ModelInfo::service_tier_for_request`][codex-model-source] filters unsupported catalog tiers.
[Codex's request builder][codex-client-source] assigns the resulting value to request `service_tier`
for HTTP and WebSocket paths.
This supports choosing `priority` for Codex Fast intent,
not a promise that arbitrary models accept it.

[Codex Speed][codex-speed] documents GPT-6 Sol,
 Luna,
 Astra,
 and GPT-6.1 Sol Fast availability,
subject to plan,
 client,
 workspace,
 and rollout.
Fast consumes included subscription limits at 2.5 times Standard;
purchased credits and Enterprise pay-as-you-go at twice Standard.
These are consumption multipliers,
 not speed guarantees.
[Codex authentication][codex-auth] distinguishes subscription access from API-key billing.

## Served-tier observability and verification gaps

The incumbent forwards Pi assistant events at `index.ts:227`;
it has no independent raw-response inspection,
 requested-versus-served comparison,
or diagnostic for absent tier evidence.
Copying cost metadata does not verify subscription accounting.
Pi's adapter hooks and whether they expose a returned tier are the main investigation's responsibility.

Live backend behavior through this login and Pi is **unverified**:
`priority` acceptance,
 effective-tier reporting,
 model rejection,
server-side downgrade,
 subscription consumption,
 and latency.
A successful request alone would not prove priority service.
A missing returned tier must remain “unknown”,
 not be relabeled Standard or Fast.
Offline request-shape verification cannot establish those backend facts.

## Public API documentation: separate contract

The [API guide][api-fast] explicitly accepts `priority` and `fast`
on Responses and Chat Completions for supported models.
Priority processing was renamed Fast mode on July 30, 2026.
The [rate card][rate-card] lists GPT-6 Sol,
 Luna,
 and Astra;
fine-tuned models and embeddings are excluded by the guide.
API requests normally use bearer API keys or [workload-identity tokens][api-auth].

The guide documents possible Standard downgrade,
 Standard billing,
and returned `service_tier: "default"` under excessive traffic ramping.
Actual processing can differ from requested processing.
Older models retain `priority`;
 newer API models report Fast.
Cached-input discounts remain;
Fast costs twice applicable Standard rates for the scoped GPT-6 models.
The rate card lists no GPT-6 latency or uptime SLA.
These API rules and prices are **not established guarantees for Codex subscription transport**.

An authorized [ChatGPT plan-sharing preview][plan-inference] also uses public Responses
with specifically issued OAuth tokens.
That does not establish equivalent permissions for Pi's existing Codex login.

## Other extension precedent

[`@diegopetrucci/pi-openai-fast@0.1.1`][diego-old]
and its [source][diego-source] implement a default-off `/fast` toggle,
 not virtual models.
They inject `priority` only for OAuth GPT-5.4/GPT-5.5 and preserve existing tier fields.
The [current unified successor][diego-current] keeps separate API/Codex request values and compatibility lists.
Its accounting warning depends on effective-tier reporting;
its footer indicates eligibility/request intent,
 not confirmed service.
Neither compatibility-list approach satisfies the settled companion requirement.

[incumbent-source]: https://github.com/2h2d-co/pi-openai-codex-fast/blob/v0.0.17/index.ts
[incumbent-readme]: https://github.com/2h2d-co/pi-openai-codex-fast/blob/v0.0.17/README.md
[codex-tier-source]: https://github.com/openai/codex/blob/main/codex-rs/protocol/src/config_types.rs
[codex-model-source]: https://github.com/openai/codex/blob/main/codex-rs/protocol/src/openai_models.rs
[codex-client-source]: https://github.com/openai/codex/blob/main/codex-rs/core/src/client.rs
[codex-speed]: https://developers.openai.com/codex/agent-configuration/speed
[codex-auth]: https://developers.openai.com/codex/auth
[api-fast]: https://developers.openai.com/api/docs/guides/fast-mode
[rate-card]: https://openai.com/api-fast-mode/
[api-auth]: https://developers.openai.com/api/reference/overview#authentication
[plan-inference]: https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference
[diego-old]: https://cdn.jsdelivr.net/npm/@diegopetrucci/pi-openai-fast@0.1.1/README.md
[diego-source]: https://cdn.jsdelivr.net/npm/@diegopetrucci/pi-openai-fast@0.1.1/index.ts
[diego-current]: https://github.com/diegopetrucci/pi-extensions/blob/main/extensions/fast/README.md
