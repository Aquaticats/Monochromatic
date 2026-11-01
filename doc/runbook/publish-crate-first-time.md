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
   installed by running `mise run toolchain:install` from the repository root
   (`toolchain.lock.json` records the version).
   Check with `mise run toolchain:list`,
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
Owner action remaining: none. The owner revoked the `cargo login` token created for this publish on 2026-10-29,
and `cargo logout` then removed the dead credential from `~/.cargo/credentials.toml`.

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
  Moved at the Rust file-enforcer cutover (2026-10-31): `mpm.managers.toml` is deleted, so that command no longer
  exists. The equivalent is `cargo binstall --no-confirm --root <dir> --version <version> monochromatic-mpm-shadow`,
  which `mpm-shadow install` runs for every cargo-binstall tool (`package/mpm-shadow/cli/README.md`,
  "Cargo tool installs"); a fresh machine takes the archive row of the `mpm-shadow` tool in `toolchain.lock.json`.

## The five file-enforcer crates 0.1.0

Status:
TODO,
owner action needed
([`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md), `Cutover`:
the switch waits for every crate to be published and fetched by a clean container).

The Rust file-enforcer is five new crates.
crates.io answered 404 for all five on 2026-10-31,
so each needs a first version published once with a personal API token,
exactly as `monochromatic-mpm-shadow` was.
`Setup`, `Steps` and `Restore` of this runbook apply with the substitutions under each crate heading;
this section gives the order and the shared owner steps.

Everything in this section was prepared on the branch `worktree-agent-acdb2be076c846e2c`,
and nothing was published,
no token was created,
`cargo login` was not run,
and no Trusted Publishing configuration exists yet.

### Publish order

The order follows the dependency pins,
which are exact (`=`) path versions that file-enforcer's `Cargo.toml` sync writes:

1. `monochromatic-jsonc-edit` 0.1.2.
   `monochromatic-file-enforcer` calls `parse_jsonc_with_spans`,
   which 0.1.1 (the version on crates.io) does not have.
   0.1.2 is an additive release,
   which the 0.1 series treats as a patch bump,
   so 0.1.1 users keep compiling.
   This crate already uses Trusted Publishing,
   so its release is a workflow run and needs no owner token.
2. `monochromatic-xml-edit` 0.1.0,
   `monochromatic-dependency-catalog` 0.1.0 and `monochromatic-browserslist` 0.1.0,
   in any order,
   because none of them depends on another first-party crate.
3. `monochromatic-file-enforcer` 0.1.0,
   which pins `monochromatic-jsonc-edit` to `=0.1.2`.
4. `monochromatic-file-enforcer-cli` 0.1.0,
   which pins `monochromatic-file-enforcer`,
   `monochromatic-dependency-catalog`,
   `monochromatic-browserslist` and `monochromatic-xml-edit` to `=0.1.0`.

The workflow jobs chain in the same order with `needs:`,
so a run that publishes several of them never starts a crate before the crates it depends on.
A prerequisite that was skipped because its own version did not change counts as satisfied;
a failed one blocks the dependent.

`monochromatic-mpm-shadow`,
`monochromatic-lint` and `monochromatic-git-policy-cli` also pin `monochromatic-jsonc-edit`,
and now pin `=0.1.2`.
Their jobs are not chained to `je-publish-crate`,
so 0.1.2 must be on crates.io before any of them publishes a new version.

### Owner steps in order

1. Release `monochromatic-jsonc-edit` 0.1.2 before anything else,
   from the branch,
   because `monochromatic-file-enforcer` cannot be packaged or published until 0.1.2 is on the registry:

   ```bash
   gh workflow run cargo-publish.yml --ref worktree-agent-acdb2be076c846e2c -f crate=monochromatic-jsonc-edit -f dry-run=false
   ```

   Expected outcome:
   `je-publish-crate` authenticates through Trusted Publishing and uploads 0.1.2,
   and `curl -sS -A 'publish-check' https://crates.io/api/v1/crates/monochromatic-jsonc-edit` lists `0.1.2`.
   A dispatch from a branch is possible because the workflow file exists on the default branch.
   A trusted publisher configuration has no branch field:
   its fields are repository owner, repository name, workflow file name and environment
   (RFC 3691, <https://raw.githubusercontent.com/rust-lang/rfcs/master/text/3691-trusted-publishing-cratesio.md>,
   read 2026-10-31).
   No run in this repository has authenticated from a branch other than `main`,
   so confirm it here:
   if the `Authenticate with crates.io (Trusted Publishing)` step fails, merge the bump to `main` instead,
   which also publishes it.
   The merge carries all five new crates, so do it in step 8 only after their publishers exist,
   or accept one red `cargo-publish` run where the five new jobs fail at authentication and upload nothing.
   Dispatch only after the review of `monochromatic-jsonc-edit` is final:
   0.1.2 can never be republished,
   so a later change to its source needs 0.1.3 and new pins.
2. Create one token at <https://crates.io/settings/tokens> named `monochromatic-file-enforcer-bootstrap`,
   with the shortest expiry and the scopes `publish-new`,
   `publish-update` and `trusted-publishing`
   (the last one lets the agent create the five publishers through the API, as for `monochromatic-mpm-shadow`).
   Run `cargo login` at the repository root and paste the token.
   Do not paste it into the agent conversation.
3. Prove each package before uploading,
   in the order under `Publish order`:

   ```bash
   mise run //package/xml-edit/core-rust:publish:dry-run
   mise run //package/dependency-catalog/core-native:publish:dry-run
   mise run //package/browserslist/core-rust:publish:dry-run
   ```

   Expected outcome for each:
   `Packaged N files`,
   `Verifying`,
   then `warning: aborting upload due to dry run`,
   and no `403`.
   The `ignoring test` and `ignoring example` warnings are expected:
   the tests and the example read repository files that no `.crate` holds.
4. Publish the three independent crates with `mise run //package/<path>:publish`
   for `xml-edit/core-rust`,
   `dependency-catalog/core-native` and `browserslist/core-rust`.
   Cargo waits until each crate is available.
5. Dry-run and publish `monochromatic-file-enforcer`,
   then `monochromatic-file-enforcer-cli`:

   ```bash
   mise run //package/file-enforcer/core-rust:publish:dry-run
   mise run //package/file-enforcer/core-rust:publish
   mise run //package/file-enforcer/cli:publish:dry-run
   mise run //package/file-enforcer/cli:publish
   ```

   These dry runs resolve the pinned crates from crates.io,
   so they only pass after steps 1 and 4.
   Before then the core answers
   ``failed to select a version for the requirement `monochromatic-jsonc-edit = "=0.1.2"` ``
   and the cli answers
   ``no matching package named `monochromatic-browserslist` found``.
6. Tell the agent the five crates are published and the token is logged in.
   The agent creates the five Trusted Publishing configurations through the API,
   one `POST https://crates.io/api/v1/trusted_publishing/github_configs` per crate with the body
   `{"github_config": {"crate": "<name>", "repository_owner": "Aquaticats", "repository_name": "Monochromatic",
   "workflow_filename": "cargo-publish.yml", "environment": null}}`,
   reads the token from the credential file in-process,
   prints only statuses,
   and lists them back with `GET ...?crate=<name>`.
   The owner is `Aquaticats` (id 343218767),
   not `Aquaticat`
   (the correction recorded in the `monochromatic-search-shadow` section).
   The dashboard route in steps 12 to 16 of `Steps` does the same one crate at a time.
7. Revoke the token at <https://crates.io/settings/tokens>,
   then run `cargo logout`.
8. Merge the branch to `main`.
   The pushed range adds the five new `Cargo.toml` files to `main`,
   so every `*-detect` job reads that as a version bump and runs the publish job:
   each authenticates through Trusted Publishing and logs `already on crates.io; skipping publish.`,
   and the `fec-*` jobs build the two Linux archives and create the immutable release
   `monochromatic-file-enforcer-cli-v0.1.0`.
   Dispatch a dry run first from the branch if you want to see the archives before they become permanent:

   ```bash
   gh workflow run cargo-publish.yml --ref worktree-agent-acdb2be076c846e2c -f crate=monochromatic-file-enforcer-cli -f dry-run=true
   ```

### What to check

1. Every crate answers on the registry:
   `curl -sS -A 'publish-check' https://crates.io/api/v1/crates/<name>` contains `"max_stable_version":"0.1.0"`
   (`0.1.2` for `monochromatic-jsonc-edit`) and `"license":"LGPL-3.0-or-later"`.
2. A clean container resolves the cli from the registry:
   `cargo install --version 0.1.0 --root <disposable directory> monochromatic-file-enforcer-cli`
   in a podman container capped `--cpus=2 --memory=2g`,
   then `file-enforcer --version` prints one JSON line whose `message` is `file-enforcer 0.1.0`.
3. `gh release view monochromatic-file-enforcer-cli-v0.1.0 --json assets --jq '.assets[].name'` lists the two archives
   and `monochromatic-file-enforcer-cli-0.1.0-sha256sums.txt`.
4. Discovery works without compiling:
   `cargo binstall --dry-run --no-confirm --disable-strategies compile,quick-install monochromatic-file-enforcer-cli@0.1.0`
   in a disposable directory ends with `has been downloaded from github.com`.
5. The cutover comparison against the TypeScript tool still has to run after this;
   publication is only one of its prerequisites.

### What was proven before publication

Measured on 2026-10-31 with the nightly `cargo 1.101.0-nightly (3960fa69b 2026-10-12)`,
from a clean worktree at the commit that holds the package metadata changes:

- `cargo publish --dry-run` through the package's mise task,
  with verification by compile,
  passed for `monochromatic-xml-edit` (`Packaged 27 files, 199.3KiB (52.5KiB compressed)`),
  `monochromatic-dependency-catalog` (`Packaged 43 files, 278.0KiB (70.4KiB compressed)`)
  and `monochromatic-browserslist` (`Packaged 57 files, 390.6KiB (94.6KiB compressed)`);
  `cargo package` passed for `monochromatic-jsonc-edit` 0.1.2 (`Packaged 33 files, 344.4KiB (84.3KiB compressed)`).
- The core and the cli cannot resolve unpublished pins from crates.io,
  so a plain `cargo publish --dry-run` fails for them,
  even with `--no-verify`,
  with the two errors quoted in step 5 of `Owner steps in order`.
  With a scratch-only override in the command line
  (`--config 'patch.crates-io.<crate>.path="<worktree path>"'` for each unpublished dependency),
  `cargo publish --dry-run` passed for `monochromatic-file-enforcer` (`Packaged 61 files, 507.7KiB (130.6KiB compressed)`)
  and `monochromatic-file-enforcer-cli` (`Packaged 29 files, 233.4KiB (60.8KiB compressed)`),
  verifying by compile.
  No file in the repository carries that override.
- All six `.crate` files were unpacked into separate empty directories and built in one podman container
  capped `--cpus=2 --memory=2g`,
  with the cli and the core resolving their first-party dependencies to the sibling unpacked directories
  through `--config patch.crates-io`.
  Every build finished,
  an unpatched build of `monochromatic-xml-edit` served as the toolchain control,
  and the built `file-enforcer --version` printed
  `{"schemaVersion":1,"sequence":0,"type":"version","code":"version","message":"file-enforcer 0.1.0"}`.
  Because the unpacked directories are what gets uploaded,
  a file missing from an `include` list would have failed there.
- No `.crate` contains a path dependency,
  and the `Cargo.toml` of each carries `license`, `description`, `repository`, `readme`, `keywords` and `categories`;
  `LICENSES/LGPL-3.0-or-later.txt` and `LICENSES/GPL-3.0-or-later.txt` ship in all five.

Not proven:
that crates.io's index resolves the exact pins after the upload
(only the real publish shows it),
that the tests inside a `.crate` run
(the tests and fixtures that read repository-root files are left out on purpose,
and `src/` unit tests of the core and the cli that read repository files would fail outside the repository),
that `fec-create-release` and the attestations work
(a dry run skips them),
and that `mpm`'s `cargo:` resolver installs the cli,
which needs the published release.

Proven on GitHub:
the dispatched dry run `41167704851` (`monochromatic-file-enforcer-cli`, dry-run on, from the branch,
2026-10-31) passed every `*-detect` job,
`fec-release-state`
and both `fec-build-binary` jobs (x86_64 on `ubuntu-24.04`, aarch64 on `ubuntu-24.04-arm`),
including the smoke run and the archive verification,
and skipped `fec-create-release`.
`fec-publish-crate` ran, which shows the `needs:` gate accepts skipped prerequisites,
and failed at `Build .crate package (verifies by compile)` with
``no matching package named `monochromatic-browserslist` found``,
the expected result while the libraries are not on crates.io.
It should pass once steps 1 to 5 of `Owner steps in order` are done.

### monochromatic-xml-edit 0.1.0

Status:
TODO

Crate `monochromatic-xml-edit`,
version `0.1.0`,
path `package/xml-edit/core-rust`,
license `LGPL-3.0-or-later`,
workflow jobs `xe-detect` and `xe-publish-crate`.
Token name in step 3 of `Steps`:
`monochromatic-file-enforcer-bootstrap` (one token covers the five crates).
No dependencies.

### monochromatic-dependency-catalog 0.1.0

Status:
TODO

Crate `monochromatic-dependency-catalog`,
version `0.1.0`,
path `package/dependency-catalog/core-native`,
license `LGPL-3.0-or-later`,
workflow jobs `dc-detect` and `dc-publish-crate`.
The only dependency is `toml_edit`.
`mise run //package/dependency-catalog/core-native:test` includes the golden fixed-point test over the repository's
`rust-catalog.toml` and passed with 100 tests after the `monochromatic-jsonc-edit` pin moved to `=0.1.2`.

### monochromatic-browserslist 0.1.0

Status:
TODO

Crate `monochromatic-browserslist`,
version `0.1.0`,
path `package/browserslist/core-rust`,
license `LGPL-3.0-or-later`,
workflow jobs `bl-detect` and `bl-publish-crate`.
The package leaves `tests/` and `conformance/` out,
because the tests read `browserslist-data.lock.json` from the repository root,
and keeps `fixtures/**`,
which `src/fixture.rs` embeds at compile time.
The crate also ships the `monochromatic-browserslist` binary;
it has no release archives.

### monochromatic-file-enforcer 0.1.0

Status:
TODO

Crate `monochromatic-file-enforcer`,
version `0.1.0`,
path `package/file-enforcer/core-rust`,
license `LGPL-3.0-or-later`,
workflow jobs `fe-detect` and `fe-publish-crate`
(chained after `je-publish-crate`).
Depends on `monochromatic-jsonc-edit` `=0.1.2`.
The package ships `fixtures/*.jsonc` for the schema test and leaves out `examples/write-schema.rs`.

### monochromatic-file-enforcer-cli 0.1.0

Status:
TODO

Crate `monochromatic-file-enforcer-cli`,
version `0.1.0`,
binary `file-enforcer`,
path `package/file-enforcer/cli`,
license `LGPL-3.0-or-later`,
workflow jobs `fec-detect`, `fec-publish-crate`, `fec-release-state`, `fec-build-binary` and `fec-create-release`
(the publish job is chained after the other four crates' publish jobs).
Depends on `monochromatic-file-enforcer`,
`monochromatic-dependency-catalog`,
`monochromatic-browserslist` and `monochromatic-xml-edit`,
each `=0.1.0`.
Its `[package.metadata.binstall]` table matches the `monochromatic-mpm-shadow` one,
so releases are tagged `monochromatic-file-enforcer-cli-v<version>`,
the archives are `monochromatic-file-enforcer-cli-<version>-<target>.tar.gz`
(Linux x86_64 and aarch64, glibc of Ubuntu 24.04),
and `cargo binstall monochromatic-file-enforcer-cli` finds them,
which is the route mpm's `cargo:` resolver takes.
The release profile has no LTO setting (unlike `monochromatic-mpm-shadow`),
so the archive binary is an ordinary release build.

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
