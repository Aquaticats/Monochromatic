# DeepSeek Harness `node-domexception` audit

Audit date:
 2026-08-22.

Audited DeepSeek Harness revision:
 `b150a551b8d465e31e418e1b2eaf5e79bbb7d28e` (`0.1.1-rc.2`).

## Verdict

DeepSeek Harness does not directly import `node-domexception`.
The published CLI installs it through this runtime dependency path:

```text
@deepseek-ai/dsh
→ @deepseek-ai/dsh-base
→ @deepseek-ai/dsh-llm-pi-ai
→ @earendil-works/pi-ai
→ @google/genai
→ google-auth-library
→ gaxios
→ node-fetch
→ fetch-blob
→ node-domexception
```

The short answer is:
 `fetch-blob` supports Node versions from before Node had a global `DOMException`,
while DeepSeek Harness inherits that old compatibility layer through Google's HTTP and authentication stack.
DeepSeek Harness itself requires Node `^22.19.0 || >=24.0.0`,
where `DOMException` is native
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/package.json#L7-L10>,
<https://nodejs.org/docs/latest-v22.x/api/globals.html#class-domexception>).

**Assessment:**
 this is a medium dependency-packaging and documentation-hygiene finding,
not a meaningful runtime-security finding by itself.
The sloppy part is not a mistaken direct polyfill import.
It is that DeepSeek knowingly accepted a broad provider-SDK closure,
later placed the adapter in the default CLI closure,
and still describes its weight as isolated to an "opt-in adapter package."
The package is opt-in for activation but not for installation.

This finding does not support calling the whole project sloppy.
The same source shows deliberate dependency controls,
and the obsolete package is several maintainership boundaries away.

## Evidence quality and scope

The dependency path is based on primary evidence:
DeepSeek's pinned lockfile and package manifests,
published npm manifests,
and source at the exact package revisions selected by the lockfile.
The investigation also checked current upstream package manifests to determine whether an ordinary update would remove
the dependency.

No secondary article is used.
No third-party install or build command was needed to establish provenance.
Runtime-loading conclusions come from source control flow,
not from assuming that every installed package is executed.

The audit is narrow:
it evaluates why this package is present and what that says about dependency hygiene.
It is not a complete security or code-quality audit of DeepSeek Harness.

## The exact dependency path

### DeepSeek owns the first links

The published `@deepseek-ai/dsh@0.1.1-rc.2` manifest depends on `@deepseek-ai/dsh-base`,
and the published `@deepseek-ai/dsh-base@0.1.1-rc.2` manifest depends on
`@deepseek-ai/dsh-llm-pi-ai`
(<https://www.npmjs.com/package/@deepseek-ai/dsh/v/0.1.1-rc.2>,
<https://www.npmjs.com/package/@deepseek-ai/dsh-base/v/0.1.1-rc.2>).

The adapter declares `@earendil-works/pi-ai` as an ordinary runtime dependency,
not an optional dependency
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/llm/llm-pi-ai/package.json#L45-L47>,
<https://www.npmjs.com/package/@deepseek-ai/dsh-llm-pi-ai/v/0.1.1-rc.2>).

`@earendil-works/pi-ai@0.82.1` declares `@google/genai@1.52.0` as an unconditional dependency
(<https://github.com/earendil-works/pi/blob/b4f293684bba718d59cc1157679bcf6157b3a7f5/packages/ai/package.json>).
The DeepSeek lockfile resolves that exact pair
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L15345-L15355>).

### Google owns the middle links

`@google/genai@1.52.0` declares `google-auth-library`,
and the lockfile resolves `google-auth-library@10.7.0`
(<https://github.com/googleapis/js-genai/blob/v1.52.0/package.json>,
<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L15660-L15669>).

`google-auth-library@10.7.0` depends directly on `gaxios` and also reaches it through `gcp-metadata`.
DeepSeek resolves both paths to `gaxios@7.1.5`
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L18071-L18085>,
<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L18146-L18155>).

`gaxios@7.1.5` declares `node-fetch@^3.3.2`
and selects it as the default server-side fetch implementation through a dynamic import
(<https://github.com/googleapis/google-cloud-node/blob/gaxios-v7.1.5/core/packages/gaxios/package.json#L102-L105>,
<https://github.com/googleapis/google-cloud-node/blob/gaxios-v7.1.5/core/packages/gaxios/src/gaxios.ts#L670-L677>).

This is not just a stale DeepSeek pin.
The current `gaxios@7.3.1` manifest still depends on `node-fetch@^3.3.2`
(<https://www.npmjs.com/package/gaxios/v/7.3.1>).

### The `node-fetch` packages own the final links

`node-fetch@3.3.2` depends on `fetch-blob@^3.1.4` and eagerly imports `fetch-blob/from.js` from its main module
(<https://github.com/node-fetch/node-fetch/blob/v3.3.2/package.json#L64-L68>,
<https://github.com/node-fetch/node-fetch/blob/v3.3.2/src/index.js#L27-L37>).

DeepSeek resolves `fetch-blob@3.2.0`,
which declares `node-domexception@^1.0.0`.
Its `from.js` imports that package and uses it to throw a `NotReadableError` when a file-backed blob changes
before it is read
(<https://github.com/node-fetch/fetch-blob/blob/b8c8176ba48a2118c65c9d646089e7d9fa3cb8c2/package.json#L50-L54>,
<https://github.com/node-fetch/fetch-blob/blob/b8c8176ba48a2118c65c9d646089e7d9fa3cb8c2/from.js#L1-L3>,
<https://github.com/node-fetch/fetch-blob/blob/b8c8176ba48a2118c65c9d646089e7d9fa3cb8c2/from.js#L83-L90>).

The lockfile therefore records `node-domexception@1.0.0` and the registry deprecation notice,
"Use your platform's native DOMException instead"
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L13599-L13606>,
<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L18002-L18005>).

## Why the package existed

`fetch-blob@3.2.0` supports Node `^12.20 || >=14.13`,
while Node added global `DOMException` in Node 17
(<https://github.com/node-fetch/fetch-blob/blob/b8c8176ba48a2118c65c9d646089e7d9fa3cb8c2/package.json#L25-L28>,
<https://nodejs.org/docs/latest-v22.x/api/globals.html#class-domexception>).
The compatibility need was real for that package's original runtime range.

`node-domexception@1.0.0` is a compatibility extractor rather than a large DOM implementation.
It uses the native global when present;
only an older runtime without that global enters its `worker_threads.MessageChannel` fallback.
It then exports `globalThis.DOMException`
(<https://github.com/jimmywarting/node-domexception/blob/824361dc9b02a78828343075ba3763ee601ac4d2/index.js#L1-L16>).
Its manifest declares no runtime dependencies and no lifecycle scripts
(<https://github.com/jimmywarting/node-domexception/blob/824361dc9b02a78828343075ba3763ee601ac4d2/package.json>).

On DeepSeek's Node floor,
the fallback is dead code and the export is the platform's native constructor.
The package remains useful only because an older-baseline transitive package imports its name.

Upstream has acknowledged the mismatch.
`fetch-blob` issue 175 reports that all `node-domexception` versions are deprecated,
and pull request 176 removes the dependency by raising `fetch-blob`'s Node floor.
That pull request has remained open since 2025
(<https://github.com/node-fetch/fetch-blob/issues/175>,
<https://github.com/node-fetch/fetch-blob/pull/176>).
The current stable `fetch-blob@4.0.0` release still declares `node-domexception@^1.0.0`,
and `node-fetch@3.3.2` cannot consume that major version because its range is `fetch-blob@^3.1.4`
(<https://github.com/node-fetch/fetch-blob/blob/a1a182e5978811407bef4ea1632b517567dda01f/package.json#L49-L52>,
<https://github.com/node-fetch/node-fetch/blob/v3.3.2/package.json#L64-L68>).
A separate Dependabot pull request to move from `node-domexception` 1 to 2 has remained open since 2023,
and it would not solve the deprecation because the package itself remains deprecated
(<https://github.com/node-fetch/fetch-blob/pull/165>).

## Installed does not mean loaded

The published DeepSeek CLI installation includes the package through `dsh-base`,
but a DeepSeek-only request does not need to execute it.

`pi-ai` registers Google and Vertex providers with lazy API loaders
(<https://github.com/earendil-works/pi/blob/b4f293684bba718d59cc1157679bcf6157b3a7f5/packages/ai/src/api/google-generative-ai.lazy.ts>,
<https://github.com/earendil-works/pi/blob/b4f293684bba718d59cc1157679bcf6157b3a7f5/packages/ai/src/api/google-vertex.lazy.ts>).
When Google's API code does load,
`@google/genai` bypasses `GoogleAuth` for an API-key request and uses it for ambient
or explicit Google Cloud credentials
(<https://github.com/googleapis/js-genai/blob/v1.52.0/src/node/_node_auth.ts#L29-L51>).

`gaxios` imports `node-fetch` only when its default adapter first needs a fetch implementation.
Once that happens,
`node-fetch` eagerly loads `fetch-blob/from.js`,
which loads `node-domexception`
(<https://github.com/googleapis/google-cloud-node/blob/gaxios-v7.1.5/core/packages/gaxios/src/gaxios.ts#L670-L677>,
<https://github.com/node-fetch/node-fetch/blob/v3.3.2/src/index.js#L27-L37>).

The practical effects are therefore different:

- **Install and audit surface:**
   present for every published CLI install.
- **Module-loading surface:**
   reached conditionally through the Google authentication request path.
- **Behavior on supported Node:**
   exports the native constructor and does not enter its fallback.

## What DeepSeek knew and what changed

The dependency weight was an explicit tradeoff from the adapter's first commit.
The 2026-06-12 introduction called `dsh-llm-pi-ai` a design-verification twin of a hand-written DeepSeek adapter.
Its original README said that `pi-ai` installed the OpenAI,
 Anthropic,
 Google,
 Mistral,
 and AWS SDKs,
and that this was accepted for a package whose purpose was design verification
(<https://github.com/deepseek-ai/deepseek-harness/commit/ab19fed77c4b744f4447d72de0dfd59dca7c82a3>).

The adapter later became a general multi-provider feature.
The default base composition now mounts it in a dormant state,
ready for user settings to activate provider routes
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/bundle/base/cordis.patch.yml#L75-L96>).
The base package also declares it as a normal dependency,
so dormancy avoids route registration but does not avoid installation
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/bundle/base/package.json#L63-L66>,
<https://www.npmjs.com/package/@deepseek-ai/dsh-base/v/0.1.1-rc.2>).

The current adapter README says provider SDK weight is "isolated to this opt-in adapter package"
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/llm/llm-pi-ai/README.md#L166-L168>).
That sentence is true only if "opt-in" means runtime activation or direct package consumption.
It is false as an install-footprint claim for the published `dsh` CLI,
which unconditionally depends on `dsh-base` and receives the adapter through that base package
(<https://www.npmjs.com/package/@deepseek-ai/dsh/v/0.1.1-rc.2>,
<https://www.npmjs.com/package/@deepseek-ai/dsh-base/v/0.1.1-rc.2>).

This documentation drift is the clearest DeepSeek-owned defect in the chain.

## How sloppy is it?

### Runtime engineering: low concern

DeepSeek's own source uses the platform `DOMException` directly for abort classification and tests;
it does not import the deprecated package
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/web/web-search-deepseek/src/provider.ts#L341>,
<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/packages/e2b/fs-e2b/src/index.ts#L137>).
The transitive module is minimal,
has no install script,
and becomes a native-constructor export on the supported runtime.
There is no evidence here of a security boundary failure.

### Dependency packaging: medium concern

DeepSeek deliberately chose a provider aggregation package whose SDKs are unconditional install dependencies.
That tradeoff was defensible for a separate design-verification adapter,
but the package subsequently entered the default CLI and Python runtime closures without becoming install-optional
(<https://github.com/deepseek-ai/deepseek-harness/commit/ab19fed77c4b744f4447d72de0dfd59dca7c82a3>,
<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/python/sdk-runtime/package.json#L49-L53>).

The stale polyfill is also not the only deprecation recorded in the lockfile.
At the audited revision the lock contains four `deprecated:` entries,
including `node-domexception`,
 two `glob` resolutions,
 and `tsconfck`
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L12817-L12825>,
<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L13599-L13602>,
<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-lock.yaml#L14231-L14234>).
A source scan of `package.json`,
 `pnpm-workspace.yaml`,
 `scripts/`,
 and `.github/` found no audit,
outdated-package,
or deprecation-failure command at this revision.
That is a policy gap,
not proof that maintainers did not inspect the warnings.

### Supply-chain process: mitigating evidence

The repository does have deliberate controls.
It uses Dependabot for npm,
 Python,
 and GitHub Actions with a configured cooldown
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/.github/dependabot.yml>).
It also blocks dependency lifecycle scripts by default and explicitly denies the no-op scripts from
`@google/genai` and `protobufjs`,
showing that maintainers inspected at least that part of the inherited Google closure
(<https://github.com/deepseek-ai/deepseek-harness/blob/b150a551b8d465e31e418e1b2eaf5e79bbb7d28e/pnpm-workspace.yaml#L35-L50>).

An update bot cannot currently remove this package by taking routine compatible updates.
The latest published `pi-ai@0.84.2` still declares `@google/genai@1.52.0`,
and the latest `gaxios@7.3.1` still declares `node-fetch@^3.3.2`
(<https://www.npmjs.com/package/@earendil-works/pi-ai/v/0.84.2>,
<https://www.npmjs.com/package/gaxios/v/7.3.1>).
Most of the persistence therefore belongs to upstream package architecture and maintenance,
not to a missed DeepSeek patch update.

## Recommended disposition

No emergency patch is warranted for `node-domexception@1.0.0` alone.
The supported runtime never needs its fallback,
and replacing it with `node-domexception` 2 would retain a deprecated package without shrinking the dependency chain.

The appropriate actions are complementary:

1. Correct `packages/llm/llm-pi-ai/README.md` to say the adapter is default-installed but dormant until configured.
2. Decide whether multi-provider support justifies default installation.
   If it does,
    document the closure honestly.
   If it does not,
    remove `dsh-llm-pi-ai` from `dsh-base` and make provider installation a real opt-in boundary.
3. Add a lockfile-deprecation inventory gate with reviewed allowlist entries,
   so known upstream blockers remain visible without making every transitive warning an immediate release failure.
4. Track upstream removal through `fetch-blob` pull request 176 and Google's `gaxios` transport.
   The durable fix is for the Google path to stop defaulting to `node-fetch`,
   or for the provider SDK closure to become separable.

The dependency is untidy dead weight on DeepSeek's runtime floor.
The evidence supports criticizing package-boundary and documentation choices,
but not treating this one small transitive shim as evidence of unsafe implementation quality.
