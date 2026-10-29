# First publication of a Rust crate to crates.io

Use this when a crate in this repository has never been published,
for example `monochromatic-jsonc-edit` 0.1.0 in `package/jsonc-edit/core-rust`.

crates.io Trusted Publishing,
the route `.github/workflows/cargo-publish.yml` already uses for `forbidden-strings`,
`monochromatic-nested-wayland-session` and `forbidden-regex`,
cannot be configured for a crate that does not exist yet.
The registry's own documentation states the prerequisite:
"Your crate must already be published to crates.io (initial publish requires an API token)"
(<https://crates.io/docs/trusted-publishing>,
read 2026-09-25).
So the very first version of every crate here is published once with a personal API token,
and Trusted Publishing takes over afterwards.

Bridges attempted before this handoff,
on 2026-09-25 for `monochromatic-jsonc-edit` 0.1.0:

- The token already stored in `~/.cargo/credentials.toml` was used for
   `cargo publish --no-verify`;
   crates.io answered `status 403 Forbidden` with `authentication failed`,
   so that credential no longer authorizes publishing.
- `gh secret list` in this repository returned seven secrets,
   none of them a crates.io or cargo token,
   so no workflow secret can substitute for a personal token.
- `rust-lang/crates-io-auth-action` (OIDC) cannot bootstrap,
   because Trusted Publishing requires the crate to exist first.
- Browser automation was not used:
   creating an API token needs an interactive crates.io sign-in through GitHub,
   and a freshly minted token is a secret that policy keeps out of the agent conversation.

Division of labor after this runbook:
you create the token,
sign cargo in,
and configure Trusted Publishing in the crates.io dashboard.
Everything else,
the real `cargo publish`,
registry verification,
a disposable consumer build against the published crate,
and the workflow dispatch that proves the OIDC route,
is agent work.

## Crate and version covered

- Crate name:
   `monochromatic-jsonc-edit`
- Version:
   `0.1.0`
- Path:
   `package/jsonc-edit/core-rust`
- Workflow:
   `.github/workflows/cargo-publish.yml`,
   jobs `je-detect` and `je-publish-crate`
- License:
   `LGPL-3.0-or-later`,
   with `LICENSES/LGPL-3.0-or-later.txt` and `LICENSES/GPL-3.0-or-later.txt` shipped in the package

For another crate,
substitute its name,
version,
path and workflow job prefix throughout.

### Setup

Status:
TODO

Prerequisites,
each verifiable on a clean machine:

1. A Linux host with this repository checked out and on `main`,
   at a commit that contains the version you intend to publish.
   Check with `git -C <repo> log --oneline -1` and
   `grep '^version' <repo>/package/jsonc-edit/core-rust/Cargo.toml`.
2. The Rust toolchain this repository pins,
   installed by running `mise run toolchain:sync` from the repository root
   (`toolchain.lock.json` records the version).
   Check with `mise run toolchain:status`,
   which prints observed versus locked versions and exits non-zero on drift,
   and `cargo --version`,
   which must print a version rather than an error.
3. Network access to `crates.io`,
   `static.crates.io` and `index.crates.io`.
   Check with `curl -sS -o /dev/null -w '%{http_code}\n' -A 'setup-check' https://crates.io/api/v1/crates/serde`,
   which must print `200`.
4. A browser signed in to the crates.io account that owns this repository's other crates,
   reached through GitHub sign-in.
   Check by opening <https://crates.io/> and confirming your avatar in the top-right corner
   rather than a **Log in** button.
5. Owner rights on the crate,
   which a first publication establishes automatically for the publishing account.

### Steps

Status:
TODO

1. Open <https://crates.io/settings/tokens>.
   Expected outcome:
   the **API Tokens** page for your account,
   listing any existing tokens.
2. Click **New Token**.
   Expected outcome:
   a token creation form appears.
3. Type the name `monochromatic-jsonc-edit-bootstrap` in the **Name** field.
   Expected outcome:
   the name is accepted;
   it is only a label,
   but naming it after the crate makes revocation unambiguous later.
4. Set **Expiration** to the shortest offered period,
   one day where the form offers it.
   Expected outcome:
   the form shows the chosen expiry.
   A bootstrap token never needs to live longer than the publish.
5. Select the **publish-new** scope and the **publish-update** scope.
   Expected outcome:
   both scopes are checked,
   and no other scope is.
   `publish-new` creates the crate;
   `publish-update` is needed if the first attempt fails partway and you retry.
6. Click **Generate Token**.
   Expected outcome:
   the new token appears in the list and its secret value is shown once,
   as a string beginning with `cio`.
7. Copy that secret value.
   Do not paste it into the agent conversation,
   an issue,
   a commit message or a shell history entry you intend to share.
8. In a terminal at the repository root,
   run `cargo login`.
   Expected outcome:
   the prompt `please paste your API token, then press Enter:` appears.
9. Paste the token and press **Enter**.
   Expected outcome:
   the prompt returns with no `error:` line.
   Cargo stores the token in `~/.cargo/credentials.toml` with mode `600`.
10. Run `cd package/jsonc-edit/core-rust && cargo publish --dry-run --no-verify`.
    Expected outcome:
    `Packaged 31 files`,
    then `Uploading monochromatic-jsonc-edit v0.1.0`,
    then `warning: aborting upload due to dry run`,
    and no `403`.
    A `403 Forbidden` here means the login did not take;
    repeat from step 8.
11. Tell the agent that the dry run passed.
    The agent then runs `cargo publish --no-verify`,
    verifies the crate on the registry,
    and builds a disposable consumer against the published version.
    If you prefer to publish yourself,
    run `cargo publish --no-verify` in `package/jsonc-edit/core-rust`;
    expected outcome is `Uploading monochromatic-jsonc-edit v0.1.0` with no error line.
12. Open <https://crates.io/crates/monochromatic-jsonc-edit/settings>.
    Expected outcome:
    the crate's settings page,
    which only exists once step 11 succeeded.
13. Click the **Trusted Publishing** tab.
    Expected outcome:
    a page describing trusted publishers,
    with an **Add** button and no publisher listed yet.
14. Click **Add** and select **GitHub Actions**.
    Expected outcome:
    four fields appear:
    **Repository owner**,
    **Repository name**,
    **Workflow filename** and **Environment**.
15. Fill **Repository owner** with `Aquaticats`,
    **Repository name** with `Monochromatic`,
    **Workflow filename** with `cargo-publish.yml`,
    and leave **Environment** empty.
    Expected outcome:
    the fields hold exactly those values.
    This workflow does not use GitHub environments,
    so an environment name here would never match and every publish would fail.
    `Aquaticats` is the organization that owns the repository.
    `Aquaticat` is the maintainer's personal account and a different owner:
    crates.io records the owner's numeric id (66041952 for `Aquaticat`,
    343218767 for `Aquaticats`, which `gh api repos/Aquaticats/Monochromatic --jq .owner.id` reports as the repository's owner),
    so a publisher entered with `Aquaticat` does not describe this repository.
    Older entries of this runbook and the GitHub URLs under `Aquaticat/Monochromatic` still resolve only
    because GitHub redirects from the old owner name to `Aquaticats/Monochromatic`.
16. Click **Save**.
    Expected outcome:
    a GitHub Actions publisher row appears,
    showing `Aquaticats/Monochromatic`,
    `cargo-publish.yml` and an empty environment.
17. Open <https://crates.io/settings/tokens> again and click **Revoke** on
    `monochromatic-jsonc-edit-bootstrap`.
    Expected outcome:
    the token disappears from the list.
    From now on the workflow authenticates with a 30-minute OIDC token instead.

### What to check

Status:
TODO

1. The crate exists on the registry.
   Run:

   ```bash
   curl -sS -A 'publish-check' https://crates.io/api/v1/crates/monochromatic-jsonc-edit
   ```

   The body must contain `"name":"monochromatic-jsonc-edit"` and
   `"max_stable_version":"0.1.0"`.
   A `404` means the upload did not land.
2. The version carries the right metadata.
   The same response must contain `"license":"LGPL-3.0-or-later"`,
   `"crate_size"` near `72070`,
   which is the compressed upload size in bytes rather than the
   `286.9KiB` that `cargo package` reports uncompressed,
   and a non-empty `"description"`.
3. Documentation builds.
   Open <https://docs.rs/monochromatic-jsonc-edit/0.1.0/monochromatic_jsonc_edit/>.
   Expected outcome:
   HTTP `200` and a module page listing `parse_jsonc`,
   `emit_jsonc_value`,
   `jsonc_set_comment` and `JsoncNumberIdentity`.
   docs.rs builds asynchronously,
   so a first load may still return `404` while the build is queued;
   measured on 2026-09-26,
   the page served `200` within minutes of the upload.
   The `/crate/.../status` route is not a public JSON endpoint:
   it returned an HTML `404` for this crate even after the docs page worked.
4. A downstream crate can consume it.
   The agent performs this check in a disposable directory under `~/temp/agent`,
   with `monochromatic-jsonc-edit = "0.1.0"` resolved from the registry rather than a path.
   Expected outcome:
   the consumer compiles and its assertions pass against the downloaded crate.
5. Trusted Publishing works end to end.
   Two dispatches answer two different questions.
   First,
   prove the job wiring without touching authentication:

   ```bash
   gh workflow run cargo-publish.yml --ref main -f crate=monochromatic-jsonc-edit -f dry-run=true
   ```

   Then open the run and confirm the `je-publish-crate` job reached
   `Publish (dry run)` and printed `warning: aborting upload due to dry run`,
   with the attestation and authentication steps skipped.
   Measured on 2026-09-26,
   run `36217693067` concluded `success` this way.

   Second,
   prove the OIDC route itself.
   While the version in `Cargo.toml` is already on crates.io,
   a **dry-run** unchecked dispatch authenticates and then skips the upload,
   because the publish step curls
   `https://crates.io/api/v1/crates/monochromatic-jsonc-edit/${VERSION}` first:

   ```bash
   gh workflow run cargo-publish.yml --ref main -f crate=monochromatic-jsonc-edit -f dry-run=false
   ```

   Confirm `Authenticate with crates.io (Trusted Publishing)` succeeded and that the publish
   step logged
   `##[notice]monochromatic-jsonc-edit 0.1.0 already on crates.io; skipping publish.`.
   A failure at the authentication step means one of the four publisher fields does not match
   the workflow's identity.
   Measured on 2026-09-26,
   run `36217828047` concluded `success` with exactly that notice,
   and the registry still listed a single version afterwards.

   A later real version is published by bumping `version` in
   `package/jsonc-edit/core-rust/Cargo.toml`,
   pushing to `main`,
   and letting the `je-detect` job notice the bump;
   or by dispatching the same workflow with **dry-run** unchecked once the version is new.

### Restore

Status:
TODO

1. Revoke the bootstrap token if step 17 was skipped:
   open <https://crates.io/settings/tokens> and click **Revoke** on
   `monochromatic-jsonc-edit-bootstrap`.
2. Remove the stored credential from this machine with `cargo logout`.
   Expected outcome:
   `~/.cargo/credentials.toml` no longer holds a `token` entry for `crates.io`.
   Skip this if another crate on this machine still publishes by token.
3. Delete the Trusted Publisher only if the crate must stop being publishable from CI:
   crate **Settings**,
   **Trusted Publishing** tab,
   then the delete control on the GitHub Actions row.
4. A published version cannot simply be deleted.
   crates.io keeps every uploaded version permanently,
   and removal is a policy decision handled through the crates.io team rather than a setting.
   Before republishing a broken 0.1.0,
   yank it instead:
   `cargo yank --version 0.1.0 monochromatic-jsonc-edit`,
   which keeps the version downloadable for existing lockfiles while stopping new resolutions.
   Then bump the version in `Cargo.toml` and publish again.
5. Local build state created by the checks lives in
   `package/jsonc-edit/core-rust/target` and in the disposable consumer directory under
   `~/temp/agent`.
   Both are untracked and safe to delete.

## monochromatic-lint 0.1.0

Status:
TODO,
owner action needed since 2026-10-06
([`doc/handover/unified-linter-cutover.md`](../handover/unified-linter-cutover.md), `Blocker`).

Run `Setup`,
`Steps`,
and `Restore` with these substitutions:

- Crate name:
   `monochromatic-lint`
- Version:
   `0.1.0`
- Path:
   `package/linter/monochromatic-lint`
- Workflow jobs:
   `ml-detect`,
   `ml-publish-crate`,
   `ml-build-binary`,
   and `ml-create-release`
- Token name in step 3:
   `monochromatic-lint-bootstrap`
- Step 10:
   `cd package/linter/monochromatic-lint && cargo publish --dry-run --no-verify`,
   run from a checkout of the commit `doc/handover/unified-linter-cutover.md` names as the commit to publish.
   Expected outcome:
   `Uploading monochromatic-lint v0.1.0`,
   then `warning: aborting upload due to dry run`,
   and no `403`.
- Step 12:
   <https://crates.io/crates/monochromatic-lint/settings>

Unlike `monochromatic-jsonc-edit`,
this crate ships release binaries.
After the Trusted Publishing form is saved,
the agent dispatches the workflow with dry-run off on a ref whose
`package/linter/monochromatic-lint` and `package/jsonc-edit/core-rust` directories are byte-identical
to the published commit:

```bash
gh workflow run cargo-publish.yml --ref main -f crate=monochromatic-lint -f dry-run=false
```

That run authenticates,
logs `monochromatic-lint 0.1.0 already on crates.io; skipping publish.`,
builds the eight targets,
attests each archive,
and creates the release `monochromatic-lint-v0.1.0`.
The repository has immutable releases,
so the release is created once with every archive;
a target that fails leaves no release at all.
`What to check` then adds two items:
`gh release view monochromatic-lint-v0.1.0` lists the archives,
and `cargo binstall monochromatic-lint` in a disposable container installs the prebuilt binary,
which lints a small Markdown and Rust fixture.

## monochromatic-search-shadow 0.1.0

Status:
0.1.0 published by hand on 2026-10-29; 0.1.1 published by CI through Trusted Publishing on the same day, with release archives;
owner action remaining: revoke the `trusted-publishing` token.

Crate `monochromatic-search-shadow`,
version `0.1.0`,
path `package/search-shadow/cli`,
license `LGPL-3.0-or-later`.
Workflow jobs: `ss-detect`, `ss-publish-crate`, `ss-release-state`, `ss-build-binary`, `ss-create-release`
(added on 2026-10-29; the release jobs run once `Cargo.toml` carries `[package.metadata.binstall]`).

What happened,
as measured:

- The stored token in `~/.cargo/credentials.toml` (modified 2026-10-18) was tried first by
  `mise run //package/search-shadow/cli:publish` and answered `403 Forbidden: authentication failed`;
  the registry still answered 404 for the crate, so nothing was uploaded.
  This repeats the finding at the top of this runbook, so a stored token should not be assumed to work.
- The owner created a token named `monochromatic-search-shadow-bootstrap` with the publish-new,
  publish-update and trusted-publishing scopes, and ran `cargo login`.
  The publish then succeeded and cargo waited until the crate was available.
- Registry check: `max_stable_version` `0.1.0`, license `LGPL-3.0-or-later`, 39,464 bytes compressed.
- Consumer check: `cargo install --version 0.1.0 --root <disposable directory> monochromatic-search-shadow`
  built from the registry in about 9 seconds, and `search-shadow --version` printed `search-shadow 0.1.0`.
- Trusted Publishing was created without the dashboard, through the API,
  using the token's `trusted-publishing` scope:
  `POST https://crates.io/api/v1/trusted_publishing/github_configs` with a body
  `{"github_config": {"crate": "monochromatic-search-shadow", "repository_owner": "Aquaticat",
  "repository_name": "Monochromatic", "workflow_filename": "cargo-publish.yml", "environment": null}}`
  and the token in the `Authorization` header.
  The response was HTTP 200 with config id `29471` and `repository_owner_id` 66041952.
  The script read the token from the credential file in-process and printed only statuses.
  `GET` on the same path with `?crate=<name>` lists configurations and `DELETE .../{id}` removes one.
- That entry was wrong, and was replaced the same day. The runbook's own step 15 had told the agent to use `Aquaticat`,
  but the repository's owner is `Aquaticats` (id 343218767; `gh api repos/Aquaticats/Monochromatic --jq .owner.id`), and
  the workflow's OIDC identity carries the repository's owner, so the first CI publish would have failed at authentication.
  The agent found this when the workflow run URL showed `Aquaticats`. The first token then answered
  `403 authentication failed` on every call (revoked or expired), so the owner created a second token with only the
  `trusted-publishing` scope and ran `cargo login` again. With it the agent listed the other crates' publishers,
  deleted config `29471` (HTTP 204) and created config `29473` with owner `Aquaticats`, owner id `343218767` (HTTP 200).
  The listing showed that every other published crate here carries two rows, one for `Aquaticat` and one for `Aquaticats`;
  the `Aquaticat` rows are legacy from before the repository moved, and `Aquaticats` is the current one.
  Step 15 above now names `Aquaticats`.

Release path, as measured the same day:

- A dispatched dry run (run `40863232508`, from `main`) passed `ss-detect`, `ss-publish-crate` (dry), `ss-release-state`
  and both `ss-build-binary` jobs (x86_64 on `ubuntu-24.04`, aarch64 on `ubuntu-24.04-arm`) including the smoke run and the
  archive verification, and skipped `ss-create-release`.
- Commit `7b3d373a07` bumped `version` to `0.1.1` and added `[package.metadata.binstall]` (the table is read from the
  published manifest, so 0.1.0 could not carry it). The push run `40863841089` concluded `success` in every job:
  Trusted Publishing authenticated with the corrected owner (`Aquaticats`), crates.io lists `0.1.1` and `0.1.0`,
  and release `monochromatic-search-shadow-v0.1.1` is immutable with the two archives and
  `monochromatic-search-shadow-0.1.1-sha256sums.txt`.
- `cargo binstall --dry-run --no-confirm --disable-strategies compile,quick-install --version 0.1.1 monochromatic-search-shadow`
  in a disposable directory logged `has been downloaded from github.com` and would install `search-shadow`,
  so discovery works without compiling.

Still to do:

1. Revoke the `trusted-publishing` token at <https://crates.io/settings/tokens> (it cannot publish, but it can edit publishers),
   then the agent runs `cargo logout`.

## monochromatic-mpm-shadow 0.1.0

Status:
0.1.0 published and released on 2026-10-29; Trusted Publishing configured and exercised by CI.
Owner action remaining: none for the crate; the `cargo login` token the owner created for this publish should be revoked
at <https://crates.io/settings/tokens> once no further manual publish is planned.

Crate `monochromatic-mpm-shadow`,
version `0.1.0`,
path `package/mpm-shadow/cli`,
license `LGPL-3.0-or-later`.
Workflow jobs: `ms-detect`, `ms-publish-crate`, `ms-release-state`, `ms-build-binary`, `ms-create-release`.

What happened,
as measured:

- The owner's `cargo login` token (scopes publish-new, publish-update and trusted-publishing) worked on the first try:
  `cargo publish --dry-run --no-verify` packaged 24 files (40.5 KiB compressed) and aborted the upload without a 403,
  then `mise run //package/mpm-shadow/cli:publish` published `0.1.0`.
  The `cargo publish` run from inside the package directory is refused by the Git wrapper's `require-root` policy
  when it is chained after a `git status`; run Git from the repository root.
- Trusted Publishing was created through the API with the token's trusted-publishing scope, as for search-shadow:
  `POST https://crates.io/api/v1/trusted_publishing/github_configs` with owner `Aquaticats`, repository `Monochromatic`,
  workflow `cargo-publish.yml` and no environment returned config `29493` with `repository_owner_id` 343218767.
  `GET` with `?crate=monochromatic-mpm-shadow` lists it.
- A dispatched dry run (run `40883596611`) passed every `ms-*` job including both `ms-build-binary` targets
  (smoke run and archive verification) and skipped `ms-create-release`.
- The dispatch with dry-run off (run `40883917579`) authenticated through Trusted Publishing, skipped the existing upload,
  built and attested both archives, and created the immutable release `monochromatic-mpm-shadow-v0.1.0` with
  the two archives and `monochromatic-mpm-shadow-0.1.0-sha256sums.txt`.
  0.1.0 already carried `[package.metadata.binstall]`, so no 0.1.1 was needed.
- 0.1.1 followed the same day. Installing 0.1.0 showed that `cargo binstall` updates `~/.cargo/.crates.toml` but not
  `.crates2.json`, which the shadow read, so the shadow now reads `.crates.toml`. A plain push of the version bump to
  `main` ran `ms-detect` through `ms-create-release` (run `40886969789`): Trusted Publishing authenticated, crates.io
  lists 0.1.1 and 0.1.0, and release `monochromatic-mpm-shadow-v0.1.1` is immutable with both archives and the
  checksums file. Installed through mpm and the extension, the binary's SHA-256 equals the 0.1.1 archive's, and
  `toolchain:sync` then created `.cache/mpm-shadow-bin/mpm`; `mpm` in the mise environment ran the real mpm 8.2.0
  and deleted the stale 0.1.0 copies from the cargo registry.
- Consumer checks: `sha256sum --check` and `gh attestation verify` on the x86_64 archive both passed, and
  `mpm --config mpm.managers.toml --cargo-binstall-monochromatic-mpm-shadow install monochromatic-mpm-shadow`
  in a disposable cargo home installed a binary whose SHA-256 equals the archive's binary, so binstall used the release
  and did not compile. That command needs `cargo-binstall` on PATH; the repository provisions it into
  `.cache/toolchain-bin`, and without it cargo answers `no such command: binstall`.

## monochromatic-nested-wayland-session releases after the first publication

Status:
TODO,
owner go-ahead needed for the first real run of the release jobs
([`doc/decision/nested-wayland-session-releases.md`](../decision/nested-wayland-session-releases.md)).

The crate has been on crates.io since 0.1.0,
so Trusted Publishing needs no setup.
This section covers cutting a new version with its Linux archives,
using the jobs `nws-detect`,
`nws-publish-crate`,
`nws-release-state`,
`nws-build-binary` and `nws-create-release`.

### Prove the release targets build

1. Dispatch a dry run from the branch that holds the change
   (a dispatch from a branch other than `main` works because the workflow file exists on it):

   ```bash
   gh workflow run cargo-publish.yml --ref <branch> -f crate=monochromatic-nested-wayland-session -f dry-run=true
   ```

2. Open the run.
   Expected outcome:
   `nws-release-state` logs `Tag monochromatic-nested-wayland-session-v<version> already exists` for a version
   that is already released,
   and `Tag ... does not exist yet.` for a new one.
   `nws-build-binary` runs for both targets either way,
   and both pass `Smoke-run the binary` (the log prints equal `got:` and `want:` lines)
   and `Verify the archive`.
   `Attest binary provenance`,
   `nws-create-release` and the crates.io authentication stay skipped.
3. Download the artifacts `nws-bin-x86_64-unknown-linux-gnu` and `nws-bin-aarch64-unknown-linux-gnu`
   from the run page within one day.
   Expected outcome:
   each holds one `monochromatic-nested-wayland-session-<version>-<target>.tar.gz`
   whose directory holds the binary,
   `README.md` and `LICENSES/`.

### Cut the release

1. Bump `version` in `package/nested-wayland-session/cli/Cargo.toml` and push to `main`,
   or dispatch the workflow with **dry-run** unchecked for a version whose tag does not exist.
   `nws-detect` compares the version before and after the push,
   so a push that leaves it unchanged runs the detect job and nothing else.
2. Expected outcome:
   `nws-publish-crate` uploads the crate (or logs that the version is already on crates.io),
   the two builds attest their archives,
   and `nws-create-release` creates `monochromatic-nested-wayland-session-v<version>` once,
   with two archives and `monochromatic-nested-wayland-session-<version>-sha256sums.txt`.
3. Check the release:

   ```bash
   gh release view monochromatic-nested-wayland-session-v<version> --json assets --jq '.assets[].name'
   ```

   Expected outcome:
   three names,
   the two archives and the checksums file.
   Then download one archive and run `sha256sum --check` against the checksums file
   and `gh attestation verify <archive> --repo Aquaticats/Monochromatic`.
4. Check discovery in a disposable directory:

   ```bash
   cargo binstall --dry-run --no-confirm --disable-strategies compile,quick-install \
     monochromatic-nested-wayland-session@<version>
   ```

   Expected outcome:
   the output ends with `has been downloaded from github.com`.

### If a run fails

The release is immutable,
so it either exists with every asset or does not exist.
A real run for a version whose tag exists skips the build and the release without failing:
that is the expected result of re-running a finished release.
If a build or smoke step failed before the release was created,
fix the cause and re-run;
the tag is free.
If a release was created with a wrong asset,
it cannot be repaired and its tag name cannot be reused,
so publish the next patch version.
