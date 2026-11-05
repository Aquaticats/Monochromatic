# Cargo registry protocol and repository integration

## Scope at 2026-11-04T19:05:29-05:00

The owner chose hosting our own crates with anonymous downloads,
while third-party dependencies continue to come directly from crates.io.
This note records protocol facts and integration boundaries;
it does not adopt a registry implementation or authorize deployment.

The existing [pnpr decision](../decision/private-npm-registry.md) supplies deployment precedent:
Coolify,
Caddy,
anonymous reads,
authenticated publishing through GitHub Actions OIDC,
and no upstream proxy.
Its no-backup decision applies to npm storage;
the archive consequences for Cargo need an explicit Rust decision.

## Cargo uses an alternate registry

Cargo associates a registry name with an index URL in `.cargo/config.toml`.
A dependency selects it through its own `registry` key;
there is no npm scope mapping in this documented interface.
A `sparse+https` index fetches metadata for the needed crates over HTTP.
Cargo supports a registry allow-list through `package.publish`.
Crates published to crates.io cannot depend on another registry's crates.
These facts make the public-release and hosted-release dependency graphs a packaging concern.
Source:
[Cargo registries](https://doc.rust-lang.org/cargo/reference/registries.html#:~:text=crates.io%20does%20not%20accept%20packages%20that%20depend%20on%20crates%20from%20other%20registries.).

Source replacement assumes equivalent source contents and prohibits adding crates absent from the original source.
It therefore does not implement this owner's hosted-crate requirement.
Source:
[Cargo source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html#:~:text=source%20replacement%20is%20not%20appropriate).

## Authentication must reach Cargo publishing

Cargo's authenticated HTTP API receives an API token in the `Authorization` header.
A browser sign-in mechanism alone does not establish a Cargo publishing integration.
Source:
[Cargo registry API](https://doc.rust-lang.org/cargo/reference/registry-web-api.html#web-api).

Cargo retrieves credentials through built-in or external credential providers.
The `cargo:token` provider reads registry-specific token environment variables;
those variables do not work when that provider is absent from the configured provider list.
Source:
[Cargo authentication](https://doc.rust-lang.org/cargo/reference/registry-authentication.html#:~:text=If%20this%20credential%20provider%20is%20not%20listed).

The existing public workflow authenticates against crates.io through `rust-lang/crates-io-auth-action`
and supplies its output as `CARGO_REGISTRY_TOKEN`
(`.github/workflows/cargo-publish.yml`,
lines 191 to 205).
Its action documents a JWT exchange for a temporary access token;
an alternate registry must support the selected exchange or its own workload-token verification.
Source:
[crates.io authentication action](https://github.com/rust-lang/crates-io-auth-action#usage).
The pnpr Cargo authentication path is being checked separately against the deployed fork revision.

## Package archives carry the dependency graph

Cargo packages source into a `.crate` archive,
rewrites its manifest,
and removes workspace and patch sections.
Published normal path dependencies need a version;
Cargo ignores their local paths in the published package.
Source:
[Cargo package behavior](https://doc.rust-lang.org/cargo/commands/cargo-package.html#description).

The repository currently keeps internal dependencies as paths with exact generated version requirements.
This is an accepted [Rust catalog decision](../decision/rust-dependency-catalog.md).
Publishing these crates requires checking the normalized archive's dependency registry identities,
then proving a fresh external consumer resolves internal dependencies from the hosted registry
and external dependencies from crates.io.
Dependency-first publication alone does not prove correct registry selection.

The index records each archive's SHA-256 checksum,
requires a unique package version,
and allows changing only the yanked field after insertion.
Source:
[Cargo index format](https://doc.rust-lang.org/cargo/reference/registry-index.html#:~:text=The%20JSON%20objects%20should%20not%20be%20modified).
Consequently,
rebuilding and republishing a version after storage loss is not a substitute for retaining its exact archive bytes.
Whether losing old locked versions is acceptable remains an owner choice.

## Repository consumers need owner changes

The current `.cargo/config.toml` contains environment settings only.
The Cargo tools add constraints beyond Cargo's native alternate-registry support:

- `package/dependency-catalog/core-native/src/site_classify.rs`,
  lines 34 to 38 and 181 to 199,
  excludes `registry` from dependency keys and reports that other registries need a decision record.
  Its `Entry::Registry` carries only `version` and `package`
  (`src/types.rs`,
  lines 37 to 45).
  `file-enforcer.config.jsonc`,
  lines 163 to 169,
  assigns generated Cargo dependency tables to this owner.
  These are source findings,
  not a reproduced file-enforcer invocation.
- `package/cargo-shadow/cli/src/lockfile.rs`,
  lines 90 and 169 to 190,
  explicitly refuses non-crates.io registry sources.
  This protects comparisons and caches that identify versions only by `name@version`.
  Merely allowing the new URL would risk checking a private crate against unrelated crates.io history.
- The implicit install step selects the current Cargo root
  (`package/cargo-shadow/cli/src/install_step.rs`,
  lines 442 to 500).
  Standalone `cargo-shadow install` enumerates all locked roots
  (`package/cargo-shadow/cli/src/commands.rs`,
  line 561).
  A consumer outside a repository with `rust-catalog.toml` bypasses the implicit catalog step by design.

Registry identity and trust-policy routing belong in the existing catalog and cargo-shadow owners.
Publishing while keeping repository-internal dependencies as paths may avoid this consumer migration;
the packaging and actual consumer checks still need to demonstrate that route.

## Reproduced wrapper boundary at 2026-11-04T19:05:29-05:00

The existing `package/cargo-shadow/cli/target/release/cargo-shadow` binary was exercised in a disposable fixture.
Binary SHA-256:
`19fcb3ce3c25ea79f7f59f84ba8a591e835015628f3b66117b45352b7dc2b22d`.
Its build revision was not established;
source inspection used checkout `8e6b179d09ae7eec9bb0d1acf04db71f5d9bcdfd`.

The fixture supplied a deterministic Cargo executable that accepted freshness checks
and returned workspace metadata.
This isolates cargo-shadow's policy behavior from network and Cargo dependency resolution.
With `trustPolicy = "no-downgrade"`,
`trustLockfile = false`,
and `minimumReleaseAge = 0`:

- The path-only lockfile control exited 0 for `cargo-shadow install --frozen-lockfile`.
- Adding a package with source `sparse+https://registry.example.invalid/index/` exited 1.
  The diagnostic was
  `which the policy checks do not support: they ask crates.io only`.

The fixture and driver remain under `~/temp/agent/cargo-registry-protocol/`.
The command ran under `CPUQuota=200%`,
`MemoryMax=1G`,
`TasksMax=64`,
and `IOWeight=10`.
No repository manifest,
lockfile,
registry,
or deployed configuration was changed.
This reproduces the wrapper refusal;
it does not test Cargo against pnpr.

The cited Cargo text fragments were opened and matched against rendered body text with `agent-browser`.
