# Release the nested Wayland compositor as prebuilt binaries on GitHub

## Status

Accepted,
2026-10-22,
owner-authorized.

## Context

The repository owner asked for GitHub releases CI ("Please setup gh releases CI")
and chose two products:
the nested Wayland compositor binaries (`package/nested-wayland-session/cli`,
crate `monochromatic-nested-wayland-session`) and the IDE.
This decision covers the compositor only.
The IDE has its own workflow.

The request assumed the compositor had no GitHub release and no prebuilt binary,
so that `mise` compiled it from source everywhere.
That was already untrue when the work started,
and the evidence matters for what this decision changes:

- `.github/workflows/cargo-publish.yml` already had the jobs `nws-detect`,
  `nws-publish-crate`,
  `nws-build-binary` and `nws-create-release`
  (added in commit `6c0c5de42c`).
  They built the two Linux gnu targets and created the release in one `gh release create` call.
- The crate's `Cargo.toml` already carried `[package.metadata.binstall]`
  (added with 0.1.2 in commit `414e181591`).
  `cargo metadata` on the manifest prints it,
  and the registry copy of 0.6.1 carries it too:
  a `cargo-binstall` dry run of `monochromatic-nested-wayland-session@0.6.1`
  with `--disable-strategies compile,quick-install` exited 0
  and reported `has been downloaded from github.com`
  from `releases/download/monochromatic-nested-wayland-session-v0.6.1/`.
- The releases `monochromatic-nested-wayland-session-v0.5.0`,
  `v0.6.0` and `v0.6.1` exist,
  each with an x86_64 and an aarch64 gnu archive.
  The release is immutable (`isImmutable: true`) and is not a pre-release,
  like the other crates' releases.
- `mise.toml` declares `"cargo:monochromatic-nested-wayland-session" = "latest"`
  and `mise.lock` records 0.6.1 under that backend,
  so `mise` already installs the prebuilt archive through `cargo-binstall`
  (see [the binstall fallback record](../troubleshooting/mise-cargo-binstall-falls-back-to-compiling.md)).
- The forbidden-strings flow the request pointed to as the model has no `[package.metadata.binstall]` at all,
  and no `forbidden-strings-binstall*.md` decision exists.
  The record of that gap is the last section of the troubleshooting document above,
  and issue 1031 tracks it.
  The compositor is therefore ahead of forbidden-strings here,
  not behind it.

What was missing,
measured against the owner's specification for a release pipeline:

- A dry run did nothing for the binaries.
  `nws-build-binary` carried `&& !inputs.dry-run`,
  so the first time the matrix compiled was the real release,
  and a failure there leaves no immutable release.
- No smoke test ran the built binary.
- The x86_64 archive built on `ubuntu-latest`,
  a label GitHub moves to Ubuntu 26.04 (glibc 2.43) in November 2026,
  which would raise the glibc floor of every later archive.
- A real dispatch for a version whose tag exists failed at `gh release create`.
- The release had no checksums file.
- The archive held the binary and README but not the licenses of an LGPL-3.0-or-later crate.
- `cargo build` ran without `--locked`.
- A comment in the workflow said the crate needed no `[package.metadata.binstall]`,
  which stopped being true when the table was added.

## Decision

Keep the existing jobs and close those gaps in `.github/workflows/cargo-publish.yml`.
The targets stay `x86_64-unknown-linux-gnu` on `ubuntu-24.04`
and `aarch64-unknown-linux-gnu` on `ubuntu-24.04-arm`.
The compositor is a Wayland compositor,
so there is no macOS,
Windows or musl target.
The build installs the same apt packages as `nws-publish-crate`
(`pkg-config`,
`libwayland-dev`,
`libxkbcommon-dev`,
`libdrm-dev`,
`libegl-dev`,
`clang`,
`libclang-dev`),
which are the Ubuntu equivalents of the packages in the crate's `Containerfile`.

The changes:

- **Dry run builds.**
  `nws-build-binary` is no longer gated on `!inputs.dry-run`,
  as `ml-build-binary` and `gpc-build-binary` already are.
  Attestation and the release stay behind `!inputs.dry-run`.
  The workflow artifact of a dry run lives one day;
  a real run keeps seven days like the sibling jobs.
- **Smoke test and archive check.**
  Each runner runs its own binary,
  and `--version` must print `monochromatic-nested-wayland-session <Cargo.toml version>`.
  After packaging,
  the archive is extracted into a fresh directory,
  the layout is checked,
  and the extracted binary runs `--version` again.
- **Archive contents.**
  `monochromatic-nested-wayland-session-<version>-<target>/` holds the binary,
  `README.md` and `LICENSES/*.txt`.
  The forbidden-strings archives hold only the binary and README,
  so "like forbidden-strings" applies to the layout,
  not to the licenses.
  The layout still matches `bin-dir` in `[package.metadata.binstall]`,
  and the archive name matches its `pkg-url`.
- **Tag lookup.**
  The new job `nws-release-state` asks the GitHub API for the tag.
  A real run for an existing tag skips the build and the release without failing;
  a lookup failure other than 404 fails the job instead of reading as "absent".
  A dry run builds regardless,
  because dry-running the current version is the usual case.
- **Checksums.**
  `nws-create-release` verifies the expected archives are present,
  writes `monochromatic-nested-wayland-session-<version>-sha256sums.txt`,
  and passes it with the archives to the single `gh release create` call.
  The archives carry build provenance attestations;
  the checksums file is not attested separately,
  because its content is derived from archives whose digests the attestations already cover.
- **Release ordering.**
  `nws-create-release` also needs `nws-publish-crate`,
  as `gpc-create-release` does,
  so no immutable release exists for a version that is not on crates.io
  (`cargo-binstall` reads the registry copy of the manifest).
- **Hygiene.**
  `cargo build --release --locked`,
  the toolchain printed before the build,
  the target passed through `env` rather than interpolated into the script,
  and `persist-credentials: false` on the build checkout.

No `version` changed,
and neither `Cargo.toml` nor any other path in the push trigger's filter was edited,
so merging this change starts no `cargo-publish` run.
A later push that touches a listed `Cargo.toml` without changing this crate's version
makes `nws-detect` compute equal `before` and `after`,
so `should_publish` is `false`,
and every nws job is skipped.

## Consequences

- The first real run of the new jobs needs the owner's go-ahead.
  It happens when the owner bumps the version (a push to `main`) or dispatches the workflow with `dry-run` off
  for a version whose tag does not exist.
  Until then the only evidence for the new steps is local:
  the step scripts ran against the existing release binary,
  and `actionlint` and `zizmor` pass.
  The owner's dry-run dispatch is the first run on real runners.
  It proves the apt install,
  the aarch64 build,
  the smoke test,
  the archive check and the one-day artifacts,
  but not attestation,
  `gh release create`,
  or the checksums file in a release.
- Releases are immutable.
  A release cannot be repaired by adding the missing asset,
  and a deleted release's tag name cannot be reused
  (<https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases>,
  read 2026-10-22).
  A target that fails therefore blocks the whole release,
  which is why the dry run builds every target first.
  A broken release is replaced by the next patch version.
- The 0.6.1 release keeps its two archives without licenses and checksums.
  The first release with them is the next version.
- The glibc floor is that of Ubuntu 24.04 (2.39),
  held by the named runners.
  Moving to a newer runner raises it for every later archive,
  so it needs its own decision.
- The release has no pre-release flag,
  matching the existing compositor release and the `monochromatic-lint` and `monochromatic-git-policy-cli` releases.
  GitHub marks one release per repository "Latest",
  and this repository releases several crates,
  so "Latest" carries no meaning for the compositor.

### Follow-up: the `mise` backend

Not changed here.
The recommendation is to keep `"cargo:monochromatic-nested-wayland-session"`.

- The evidence that it already downloads a prebuilt archive:
  the probe above resolves the 0.6.1 archive from the registry manifest,
  and the install directory `installs/cargo-monochromatic-nested-wayland-session/0.6.1/bin/` holds the binary.
- `mise`'s documentation of the cargo backend
  (<https://mise.jdx.dev/dev-tools/backends/cargo.html>,
  read 2026-10-22) says it "downloads a prebuilt binary when cargo-binstall can find one
  and builds the crate with Cargo otherwise",
  falling back to `cargo install` on exit code 94.
  The fallback is silent and takes minutes,
  so a missing archive for a future version would be noticed late.
  The setting `cargo.binstall_only` turns that into a failure,
  with the repository-wide cost described in the troubleshooting document.
- The alternative is the `github:` backend
  (<https://mise.jdx.dev/dev-tools/backends/github.html>,
  read 2026-10-22).
  It checks the GitHub-reported digest or a published checksum file,
  and checks GitHub artifact attestations when a release asset has them by default.
  That would verify the provenance this workflow already creates,
  which `cargo-binstall` does not.
  Its costs:
  tags of the form `monochromatic-nested-wayland-session-v0.6.1` need a `version_prefix`
  (the page does not document this tag shape,
  so it needs a probe with `mise ls-remote`),
  version resolution moves from crates.io to GitHub releases,
  and the existing `minimum_release_age_excludes` entry and `mise.lock` entry change backend.
- It can only be tested against a release that carries a checksums file,
  which no existing release does.
  Revisit after the first release made by this pipeline,
  by probing `mise ls-remote github:Aquaticats/Monochromatic` with the `version_prefix` and an `asset_pattern`
  in a disposable `mise` configuration.
