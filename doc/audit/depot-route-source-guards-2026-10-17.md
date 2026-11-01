# Final replay source guard controls

The prepared native,
Windows,
Alpine,
and TOML routes now reject Git index exclusions
and compare configured metadata bytes directly with committed `HEAD` blobs.
This extends the boundary already recorded in issue `#1114` to these separate trial helpers.
No provider source mutation was observed,
and application source was not changed.
The [control evidence](depot-route-source-guards-2026-10-17.evidence.json)
retains actual task output,
complete source identities,
hashes,
rejected fixtures,
and pushed route proofs.

## Verified behavior

Actual allowed before/after pairs pass for native source `cb95cf6d785c3073c192b95f0632cdae7b910894`,
native desktop source `3434eaa455d569c18b52cf15100f77f9db251d21`,
the Windows and Alpine desktop routes,
and TOML source `525d2e09f8168a3bdd7e513e26d4c80b363123e5`.
Each case uses a complete disposable exact-source clone.
There are `20` accepted invocations and `40` rejected invocations.

Every case rejects unchanged and modified ordinary source hidden by each of
`assume-unchanged` and `skip-worktree`.
Git status is empty for those controls.
NUL-delimited `git ls-files -v -z` enumeration rejects exclusions across all tracked paths.

Separate controls mask each root Cargo configuration or nested Android Cargo metadata edit
with a Git clean filter in the disposable clone.
The staged blob still equals `HEAD`,
Git status is empty,
and no index exclusion is present.
The actual helper rejects the working bytes with `Configured source input differs`.
This independently exercises direct byte comparison.
Each case also rejects a provider `GITHUB_SHA` that differs from actual harness `HEAD`.
Restored before/after controls pass.

The first allowed control exposed an omitted root `mise.lock` in the enumeration rewrite.
Its rejection remains retained,
and follow-up commits restore that required input.
Earlier clean-filter fixture attempts failed their own status preconditions and remain separate from passing controls.

## Scope and exclusions

Every tracked `Cargo.toml` and `Cargo.lock` is hashed and compared directly to `HEAD`,
including `package/music-player/android-app/rust` metadata.
Root configuration and root `.cargo/` inputs are included.
Native and Windows package configuration covers `desktop-app` and `truepeak-core`;
Alpine also covers `nested-wayland-session`.
TOML uses its frozen manifest's selected and external artifact-prerequisite package roots.

Metadata hashes remain selected evidence.
Tracked status and the independent index-exclusion invariant cover ordinary tracked source outside that subset.
Untracked files,
ignored build outputs,
and installed tool binaries are outside this identity inventory.
The controls execute the actual tasks on Linux;
they do not establish native consumer results.
Original consumer workflow commands are unchanged by this repair.

## Prepared routes

- `trial/depot-coherent-native-20261017` at `0cc63fdf5768a69a7d42c55f48adfe63e3c3341a`.
- `trial/depot-coherent-windows-20261017` at `f994e3f6902ad4c4b887822f18695d57da736ca4`.
- `trial/depot-coherent-alpine-20261017` at `30ceb0c1f8e215988e53fee6389b12b78b215a6f`.
- `trial/depot-toml-continuation-20261017` at `640a576ade81c4edfee6a603543a7fbd18faa46b`.

Scoped formatting,
clean tracked status,
remote hashes,
and GitHub raw task bytes are verified for every route.
Historical prepared hashes remain in the evidence's revision history.
Development mutation run `p6vm5s00w6` retains its original `274ec49595d60f663249d61ab35a0806fe29aff0` harness.
Final coherent dispatch remains held.
