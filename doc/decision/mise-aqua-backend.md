# Vet: mise's aqua backend and aqua itself

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Vet date:
 2026-06-09.
Verdict:
 keep the aqua backend,
 but harden it.
 It is sound for what this repo
uses it for,
 and every tool currently gets at least integrity verification.
 The
real gaps are a frozen baked registry,
 one unimplemented attestation key,
 and the
absence of a committed lockfile.
 All three close cheaply,
 the first with a single
`mise lock`.
 Do not remove the backend.

Scope:
 the four `aqua:` tools this repo provisions in `mise.toml`,
`Kitware/CMake`,
 `betterleaks/betterleaks`,
 `jdx/hk`,
`apple/pkl`.
 The standalone aqua CLI is out of scope because this repo does not
use it (see "What is actually in the trust chain").

Retirement update:
issue `#357` removed `jdx/hk` and `apple/pkl` from active tool declarations and regenerated `mise.lock`.
Their findings remain historical supply-chain evidence;
they no longer describe tools provisioned by the current repository.

## What is actually in the trust chain

"aqua" names two different things,
 and only one of them is in this repo's supply
chain.

- The aqua CLI (`aquaproj/aqua`,
   a Go binary) is not used here.
   mise does not
  shell out to it.
   mise carries its own Rust reimplementation of the aqua backend
  (`src/backend/aqua.rs`,
   `crates/aqua-registry/`).
   So the aqua binary's own
  distribution and signing are moot for this user.
- What is in the chain is two things:
   the aqua **registry** data
  (`aquaproj/aqua-registry`,
   the YAML package definitions) and mise's
  **reimplementation fidelity** (does mise honor what the registry declares).
   This
  vet targets those two surfaces.

This distinction matters:
 a finding like "aqua verifies cosign and SLSA" is true
of the aqua CLI but says nothing about whether mise does.
 Every claim below was
checked against mise's source and confirmed by running mise,
 not inferred from
aqua's documentation.

## Method

Standard applied:
 `choosing-technology`,
 source audit plus boundary run,
 since
this vets an incumbent dependency rather than picking a new SaaS vendor (the
six SaaS vetting layers do not apply to an open-source build tool;
 the
open-source maintenance audit does).

- Clone:
   `gh repo clone jdx/mise /tmp/agent/mise-aqua-vet-20260609 -- --depth 1`,
  at commit `38fce40`.
   Line citations below are from that clone.
- Installed mise on this host:
   `2026.5.15` (built 2026-05-23).
   Its baked registry
  snapshot pins `aquaproj/aqua-registry` tag `v4.515.0`
  (`vendor/aqua-registry/metadata.json` at `v2026.5.15`).
- Registry entries read both live (`aquaproj/aqua-registry` HEAD,
   via `gh api`)
  and as baked into mise `v2026.5.15`,
   because the two differ (see staleness).
- Boundary run:
   installed all four tools through mise's aqua backend into a
  throwaway `MISE_DATA_DIR` under `/tmp/agent`,
   with `-v`,
   and read back the
  verification steps mise actually performed (`betterleaks`,
   `hk` in one
  run;
   `apple/pkl` and `Kitware/CMake` in a second).
   The throwaway was deleted
  after each run.

## How mise's aqua backend resolves and verifies a tool

The install path is download,
 then verify,
 then extract.
 Verification has two
independent contributors,
 and this is the key to reading the per-tool table.

1.  Registry-declared verification.
     mise runs cosign,
     SLSA,
     minisign,
     and GitHub
    artifact attestations only when the package's registry entry declares them,
     and
    verifies a registry checksum file only when the entry has a `checksum:` block.
    Each is gated by `if let Some(...) = &pkg.<field>` in `verify_provenance`
    (`src/backend/aqua.rs:1872`).
     No declaration means that mechanism never runs.
    All of cosign,
     slsa,
     minisign,
     and github_attestations default to enabled
    (`settings.toml`),
     so the gate is the registry,
     not a mise setting.
2.  GitHub API digest fallback.
     Independently,
     if the GitHub releases API returns a
    `digest` for the downloaded asset,
     mise records it and verifies the bytes
    against it even with no registry checksum config
    (`src/backend/aqua.rs:148`,
     `:538`,
     `:1684`;
     logged as
    "using GitHub API digest for checksum verification").
     This is integrity against
    GitHub's own record,
     fetched over a separate API call than the CDN download.
     It
    catches transit,
     mirror,
     and corruption tampering.
     It does not bind the artifact
    to a publisher build identity,
     so it is weaker than cosign,
     SLSA,
     or
    attestations,
     and it is absent for assets GitHub has no digest for (older
    releases,
     non-GitHub hosts),
     in which case the next paragraph applies.
3.  Final checksum gate.
     `verify_checksum` (`src/backend/mod.rs:2284`) verifies the
    artifact against whatever checksum was recorded by step 1 or 2.
     If neither
    recorded one,
     it does not fail:
     it either generates a trust-on-first-use blake3
    when a lockfile is in play,
     or does nothing.
     So with no declared verification,
    no API digest,
     and no lockfile,
     the install is TLS-in-transit only.

### The registry mise reads is the baked snapshot, not the live registry

By default (`aqua.baked_registry = true`,
 `settings.toml:92`),
 mise uses only the
registry snapshot compiled into the mise binary at release time.
 It does not fetch
`aquaproj/aqua-registry` at all.
 `configured_registry_urls`
(`src/aqua/aqua_registry_wrapper.rs:91`) returns an empty list in the default case
and the source list becomes `[Baked]`.
 mise downloads the live registry only if
you set `aqua.baked_registry = false` or configure `aqua.registries`.
 The
one-week `aqua.registry_cache_ttl` therefore does nothing in the default
configuration;
 there is no network fetch to cache.

Consequence:
 the snapshot is frozen at the installed mise's release.
 On this host
that is `v4.515.0` (2026-05-23).
 Registry fixes and newly added verification
config do not reach you until you upgrade mise (or disable the baked registry).
Version resolution,
 by contrast,
 is live:
 mise queries the GitHub releases API for
the latest tag,
 then applies the frozen asset-naming template.
 Live version plus
frozen template is exactly the shape that breaks when upstream renames assets.

## Per-tool verification, as the installed mise actually performs it

Confirmed by the boundary run on mise `2026.5.15` (baked `v4.515.0`):

- `betterleaks/betterleaks`:
   integrity only.
   The `v4.515.0` snapshot declares a
  `checksum:` block but no cosign (`registry.yml:19163` in that tag).
   The live
  registry HEAD has added a cosign sigstore bundle bound to the betterleaks
  release workflow (`registry.yml:19355` in the HEAD snapshot),
   but the installed
  mise predates it.
   Net:
   verified against the GitHub API digest,
   no publisher
  signature.
   A newer mise,
   or a live registry,
   would add cosign.
- `jdx/hk`:
   integrity only,
   and this one is not staleness.
   The `v4.515.0` entry
  does declare attestations,
   via `github_release_attestations: true`
  (`registry.yml:50253`).
   But mise does not implement that key at all (see the
  reimplementation gap below),
   so it is silently ignored and hk falls back to the
  GitHub API digest.
   Verified against the digest,
   no attestation.
- `Kitware/CMake`,
   `apple/pkl`:
   integrity only.
   Their registry
  entries declare no checksum,
   cosign,
   SLSA,
   minisign,
   or attestation,
   confirmed in
  both the live registry and the baked `v4.515.0` snapshot.
   They rely entirely on
  the GitHub API digest,
   and the boundary run confirmed both took the digest
  path:
   CMake as a tarball,
   pkl as a `format: raw` single binary (the
  digest check is per-asset,
   so it is format-agnostic).
   If GitHub ever lacks a
  digest for one of these assets (older pin,
   host change),
   the install would drop
  to TLS-only,
   because nothing else backs it.

So today,
 all five get at least GitHub-digest integrity.
 None of the five gets a
publisher-bound signature on the installed mise:
 betterleaks because of staleness,
hk because of the missing key,
 the other three because nothing is declared.

## Findings

### mise does not implement `github_release_attestations`

aqua-registry has two attestation keys:
 `github_artifact_attestations` (with a
`signer_workflow`) and `github_release_attestations: true` (GitHub's newer
release-level attestation).
 mise's `AquaPackage` parses only the former
(`crates/aqua-registry/src/types.rs:83`);
 there is zero handling of
`github_release_attestations` anywhere in mise's source,
 and no serde alias maps
it.
 Unknown keys are silently dropped on deserialize,
 so any package using it gets
no attestation under mise.
 Twenty packages in the current registry rely on this
key,
 including hk.
 This is a mise fidelity gap,
 not a registry or aqua problem,
and a candidate upstream contribution.

### Baked-snapshot staleness is both a reliability and a coverage risk

- Reliability:
   the repo's own note,
   `aqua:sharkdp/fd broken as of 2026-03-07,
  falling back to cargo` (`mise.toml:40`),
   is this failure mode.
   The registry
  re-scaffolded fd around then (aqua-registry PR #50406,
   2026-03-15,
   plus an
  asset-not-found issue #50402 in the same window).
   On a baked registry,
   even the
  registry fix would not ship until a mise upgrade.
   The exact Linux-side error the
  repo hit was not captured here,
   so the specific cause is labeled unpinned;
   the
  mechanism (live latest plus frozen template,
   fix gated behind a mise upgrade) is
  what matters and is verified.
- Coverage:
   betterleaks above is the security version of the same thing.
   The
  registry strengthened verification (added cosign) and the installed mise cannot
  see it.

### No committed lockfile, so nothing is pinned across machines or CI

The `lockfile` setting defaults to true (`src/config/settings.rs:593`),
 but mise
only reads or writes a `mise.lock` that already exists:
 reads return a default
when the file is absent (`src/lockfile.rs:497`) and every write path is gated on
`lockfile_path.exists()` (`:1002`,
 `:1249`,
 `:1367`).
 This repo has no `mise.lock`.
So no checksum or digest is persisted;
 the in-memory value computed at install is
discarded,
 and every fresh checkout (including CI) re-derives trust from scratch
rather than from a pinned record.

## Governance and maintenance of aqua and the registry

Active,
 but concentrated.
 Measured 2026-06-09 via `gh api`.

- `aquaproj/aqua` (the CLI,
   not used here but the registry's steward):
   1729 stars,
  not archived,
   pushed 2026-06-09,
   latest release `v2.60.0` (2026-06-09) with
  several in the prior two weeks.
   Contributors are dominated by one human,
  suzuki-shunsuke (2825),
   then renovate and the release bot,
   then scop (21).
   Bus
  factor is effectively one maintainer.
- `aquaproj/aqua-registry`:
   335 stars,
   MIT,
   pushed 2026-06-09,
   very high volume
  driven by `aquaproj-aqua-registry[bot]` (37563) and renovate (7300),
   with
  suzuki-shunsuke (5704) as the lead human.
   Package updates are automated and
  merge fast.
   The registry is "secure by default":
   only the standard registry is
  trusted without an explicit policy,
   which is the governance boundary mise
  inherits.

Interpretation:
 maintenance is healthy and fast,
 but the single-maintainer
concentration is the standing risk for both the CLI and the registry.
 This mirrors
the separate finding for mise itself (one dominant maintainer) recorded in
`../audit/mise-keep-vs-build-own.md`;
 the conclusion there,
 solve gaps at our
boundary rather than fork,
 applies here too.

## Alternatives considered

Per `choosing-technology`,
 the alternatives to the aqua backend for these tools,
with why each is not a wholesale replacement:

- Keep aqua plus commit a lockfile (recommended).
   Lowest cost,
   keeps the single
  declarative backend,
   and pins bytes across machines and CI.
   Does not retro-add
  publisher signatures for the unverified tools;
   it pins what you first fetched.
- `ubi:` backend (mise's GitHub-release installer).
   Rejected as a blanket swap:
   it
  resolves assets heuristically without the registry's curated naming,
   so it trades
  the staleness risk for a different fragility and still has no publisher signature
  for most of these tools.
- Language backends (`cargo:`,
   `npm:`).
   Already used here for fd,
   fastmod,
   and
  others,
   and the right move per-tool when a tool ships that way (fd fell back to
  `cargo:fd-find`).
   Rejected as a blanket swap:
   CMake and pkl have no
  better-verified language-native path,
   and building from source has its own
  supply-chain surface.
- `github:` backend with attestations.
   Plausible for tools that publish GitHub
  attestations,
   but mise's attestation handling has the same `github_release_*`
  gap,
   so it would not currently verify hk's attestation either.

## Recommendation

Keep the aqua backend.
 Harden in this order.

1.  Run `mise lock` and commit the resulting `mise.lock`.
     This pins exact bytes
    (digest or checksum) for all five tools across every machine and CI,
     records
    provenance where it does run,
     and makes mise fail closed on a later mismatch or
    verification downgrade instead of silently re-deriving trust.
     Caveat to state
    plainly:
     a lockfile pin is trust-on-first-use;
     it guarantees reproducibility,
    not that the first download was publisher-authentic for the three tools with no
    declared verification.
2.  Keep mise current.
     The installed `2026.5.15` is behind (`2026.6.1` is
    available),
     and the baked registry only advances with mise.
     Upgrading is what
    delivers the betterleaks cosign config and fd-style registry fixes.
3.  Optionally set `aqua.baked_registry = false` if you want registry fixes and new
    verification config without waiting for a mise upgrade.
     The trade is a network
    fetch on cache miss (weekly TTL) and a live dependency on the registry repo.
    Reasonable for a single-maintainer machine,
     weigh it for CI.
4.  Treat hk's missing attestation as a known gap,
     not a regression to chase
    locally.
     The fix is upstream in mise (implement `github_release_attestations`);
    until then hk is digest-verified,
     which is the same posture as the other tools.

This is an assessment,
 not a change.
 No `mise.toml`,
 settings,
 or lockfile were
modified as part of this vet.

## When to revisit

- mise gains `github_release_attestations` support:
   re-check hk and the other 19
  affected packages,
   and drop that finding.
- The single-maintainer concentration on aqua or the registry changes (handoff,
  slowdown across the bot-plus-maintainer model,
   archival):
   re-run the
  build-versus-trust question for the registry the way `../audit/mise-keep-vs-build-own.md`
  did for mise.
- A second baked-snapshot breakage like fd recurs:
   that is the signal to disable
  the baked registry by default rather than per-incident.

## References

- `../audit/mise-keep-vs-build-own.md`:
   the companion decision to keep mise,
   with
  the maintainer-health method this vet reuses.
- `mise.toml`:
   the five `aqua:` entries and the fd fallback note.
- jdx/mise,
   cloned at `38fce40`:
   `src/backend/aqua.rs`,
   `src/backend/mod.rs`,
  `src/aqua/aqua_registry_wrapper.rs`,
   `src/config/settings.rs`,
   `src/lockfile.rs`,
  `crates/aqua-registry/src/types.rs`,
   `build.rs`,
   `settings.toml`,
  `vendor/aqua-registry/`.
- aquaproj/aqua-registry:
   per-package `pkgs/<owner>/<repo>/registry.yaml`;
   tag
  `v4.515.0` is the snapshot baked into mise `v2026.5.15`.
