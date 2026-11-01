# Depot cache applicability and trust review

## Evidence boundary

This review covers the eleven deployed Cache integration guides,
69,290 captured source bytes and 50 claim-level findings.
The [platform ledger](depot-platform-surface-ledger.json)
keeps captured pages separate from reviewed claims and actual consumers.
The [complete integration review](../audit/depot-cache-integration-document-source-review-2026-10-18.evidence.json.gz)
joins the deployed source identities and substantive findings.
The complete frozen Git inventory has 13,860 paths at
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`,
with positive controls for the existing Mise and Kotlin Gradle owners.
Named-manifest searches measure applicability;
they do not assert that every source file was searched for integration names.

The [evaluation report](depot-expanded-ci-evaluation.md)
and [cache measurements](depot-developer-pool-cache-research.md)
retain source-bound workload results.
Published integration support alone supplies no cache-hit or speed credit.

## Automatic Actions cache and branch trust

[Depot's GitHub Actions Cache guide](https://depot.dev/docs/cache/integrations/github-actions)
says cache API clients on Depot runners automatically use Depot Cache.
Standard GitHub runners continue using GitHub's backend.
This affects Actions cache clients;
it does not automatically enable Rust compiler caching or Gradle task-output reuse.

The guide explicitly says entries have repository scope and no branch isolation.
An exact-key or restore-prefix strategy therefore needs an explicit writer-trust model.
The bounded marker campaign completed six sequential jobs using identical workflow source on sibling research refs,
provider-specific exact keys and matching cache-version/path inputs.
Both providers passed real same-branch exact-hit restoration controls.
GitHub then missed the nondefault sibling branch's entry;
Depot downloaded and rehashed its exact 53-byte marker.
The pinned client is `actions/cache` 6.1.0 at `55cc8345863c7cc4c66a329aec7e433d2d1c52a9`.
The [actual trust-scope receipt](depot-cache-trust-scope-research.md)
retains source,
key,
version,
marker and original ZIP identities.

The public GitHub setup logs explicitly report the default `write` cache mode;
alternate access-mode overrides were not tested.
GitHub separately documents restoration from the default branch,
so this sibling result does not mean all cross-branch GitHub cache reuse is unavailable.
Fork scheduling,
cross-repository access,
write-poisoning and every possible restore-prefix policy remain separate surfaces.
Both owned probe branches were removed;
the benign cache entries remain subject to unmeasured retention/eviction.
Branch deletion does not prove cache deletion.

The guide documents default retention of 14 days and no size limit,
with configurable retention and capacity options.
These are published defaults,
not inspected organization settings or an enforced invoice limit.
No organization setting or broad token is changed for this research.

## Existing Gradle and Rust owners

[The Gradle guide](https://depot.dev/docs/cache/integrations/gradle)
describes HTTP task-output caching with explicit caching and read/write policy.
Runner-supplied `init.gradle` does not remove the requirement to enable caching,
and host configuration does not automatically enter a nested container.
Multi-organization user tokens can select an organization through the Basic username.
Container examples need explicit secret propagation.

The frozen repository has five Gradle build/settings files belonging to Kotlin linter and Android owners.
The measured Kotlin-linter command is `test --rerun jar`:
compilation outputs can be restored while six fresh JUnit cases execute.
Its repeated command-level benefit is qualified separately from slower,
pnpm-confounded complete jobs.
This is not an Android Gradle cache measurement.

[The sccache guide](https://depot.dev/docs/cache/integrations/sccache)
describes a WebDAV compiler cache and explicitly sets `RUSTC_WRAPPER` in its host example.
An endpoint variable or server-start command alone does not prove Cargo used the wrapper or returned hits.
Root `.cargo/config.toml`,
`mise.toml` and `mise.no-env.toml` have no matching wrapper or sccache configuration in the verified search.
Provider-injected runtime state is still an observation to record,
not something that search proves absent.

The measured Git-policy compiler-cache hit counters remain distinct from compiled-target archive restoration,
prepared snapshots and fresh Nextest execution.
Its single cache-disabled baseline does not establish repeated complete-job benefit.
No cache mechanism justifies skipping test bodies or masking source failures.

## Integrations without a frozen source owner

[Bazel](https://depot.dev/docs/cache/integrations/bazel)
supports the HTTP remote-cache protocol,
not a demonstrated repository remote-execution service.
The frozen inventory has no Bazel owner manifest.
Its container example expands a token into a `.bazelrc` file:
an input secret mount does not establish that a separately written credential file is excluded from outputs.
This is a source-level example concern;
the example was not adopted with a real token and no actual leak is claimed.

[Go](https://depot.dev/docs/cache/integrations/gocache)
requires Go 1.24 or later and `GOCACHEPROG=depot gocache`.
The frozen inventory has no Go source,
`go.mod` or `go.work`.
The guide's `go clean -cache` example is not taken as verified remote erasure.

[Maven](https://depot.dev/docs/cache/integrations/maven)
requires an organization token,
the Maven Build Cache extension and matching remote/server configuration.
Prepopulated `settings.xml` alone does not enable that extension.
The repository has no `pom.xml` or `.mvn` manifest;
its JVM owners use Gradle.
No broad organization token is created to exercise an unused integration.

[Moon](https://depot.dev/docs/cache/integrations/moonrepo)
and [Pants](https://depot.dev/docs/cache/integrations/pants)
describe explicit gRPC cache configuration,
authentication and organization-selection headers.
Neither has a frozen workspace manifest here.
Their tool-specific placeholder and merged-header behavior was not executed.

[Nx](https://depot.dev/docs/cache/integrations/nx)
uses a direct self-hosted remote-cache protocol,
distinct from the Actions guide's older shared-directory archive example.
The Nx guide says it cannot send an organization-selection header,
so a multi-organization user token cannot select its organization that way.
[Turborepo](https://depot.dev/docs/cache/integrations/turbo)
has its own endpoint,
token and team configuration.
The inventory has no `nx.json`,
`turbo.json` or `turbo.jsonc`.
Adding either task runner would be a separate design decision;
Mise already owns this repository's tasks.

## Xcode is a separate Mac workload

[The Xcode guide](https://depot.dev/docs/cache/integrations/xcode)
describes Xcode 26 compilation caching on Depot GitHub Mac runners,
with no current local-workstation support in that guide.
An existing `XCODE_XCCONFIG_FILE` prevents the automatic override,
and `DEPOT_XCODE_CACHE_ENABLED=0` supplies a per-job opt-out.
Prefix mapping and language settings can affect compatibility and cache hits.
The guide's example counters are vendor examples,
not repository measurements.

The frozen repository has no Xcode project,
workspace,
Swift source or `Package.swift`.
Running a Rust owner on macOS does not qualify an `xcodebuild` cache consumer.
Mac runner eligibility and actual source-owner execution remain separate evaluation gates.
