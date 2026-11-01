# Extract music-player into its own repository

Status:
grilling complete and approved.
This document is the deliverable for the current session.
The session must not publish tooling,
create or populate the new repository,
transfer issues,
or remove music-player from Monochromatic.

Last updated:
2026-08-16.

## Goal

Split the music-player product out of `Aquaticat/Monochromatic` into a public repository named
`Aquaticat/music-player`.
The new repository becomes the sole development home.
Monochromatic retains no product source and points readers to the new repository from its root `README.md`.

The extraction must preserve product history,
product-owned documentation,
application identities,
licenses,
and local non-rebuildable outputs without publishing ignored local data.

## Confirmed decisions

### Ownership and scope

- Perform a full extraction,
  not a mirror or publication-only copy.
- Move all four components under `package/music-player/`:
  - `android-app`
  - `desktop-app`
  - `truepeak-core`
  - `truepeak-core.bench`
- Keep the existing `package/music-player/` nesting in the new repository.
- Remove both app `CONTEXT.md` files from the extracted branch tip.
  Their prior contents remain in filtered history.
- Preserve existing application IDs,
  package names,
  configuration paths,
  and user-data compatibility.
  Repository extraction is not a product rebrand.

### Documentation

- Move a document when music-player is its primary subject,
  regardless of whether it is a decision,
  plan,
  audit,
  handover,
  runbook,
  or troubleshooting record.
- Keep broader multi-project documents in Monochromatic.
- Replace references crossing the new repository boundary with current-branch links and provenance notes.
- Do not duplicate authoritative documents between repositories.
- Remove `package/music-player/` completely from Monochromatic.
  Do not leave a tombstone package directory.
- Add the move notice to Monochromatic's root `README.md`.

### Git history

- Preserve path-filtered history from `main`.
- Include the former `packages/music-player/` path as well as the current `package/music-player/` path.
- Include product-owned documentation across its historical locations.
- Preserve author,
  committer,
  message,
  and timestamp metadata where filtering permits.
- Accept rewritten commit IDs and unsigned rewritten commits.
- Push and verify filtered history before enabling required signed commits on the new `main` branch.
- Do not retrospectively sign historical commits with the extractor's identity.
- Do not create a migration tag or GitHub release.

### Repository visibility and governance

- Create `Aquaticat/music-player` as public.
- Use `main` as the default branch.
- Mirror Monochromatic's non-build governance:
  - issues enabled;
  - wiki,
    discussions,
    and projects disabled;
  - signed commits required after initial import;
  - resolved review conversations required;
  - protected-branch deletion disabled;
  - branch deletion after merge enabled;
  - existing merge methods retained;
  - secret scanning and push protection retained;
  - Copilot review on pushes and draft pull requests retained.
- Add no build or test CI to the new repository during extraction.
- Treat Copilot review as review automation,
  not as the excluded build and test CI.

### Issue migration

- Transfer open issues whose primary owner is music-player.
- Keep closed and cross-cutting issues in Monochromatic.
- Add reciprocal links where a retained cross-cutting issue and a transferred product issue depend on each other.
- Create matching labels before transfer so GitHub retains issue labels.
- Copy the reusable label taxonomy:
  - standard GitHub labels;
  - `needs-info`;
  - `needs-triage`;
  - `ready-for-agent`;
  - `ready-for-human`;
  - every `difficulty:*` label.
- Omit Monochromatic-specific labels such as `blocked-on-185-r2` and `deferred-new-packages`.

### Tooling publication prerequisite

Publish shared tooling before extraction.
The new repository must consume registry releases,
not source checkouts of Monochromatic tooling.

- Publish file-enforcer to npm as `@monochromatic-dev/file-enforcer-core`.
- Publish the existing Rust linter family to crates.io:
  - `monochromatic-rust-linter-core`
  - `monochromatic-rust-linter-pattern`
  - `monochromatic-rust-linter-plugin-builtin`
  - `monochromatic-rust-linter`
- Preserve the four-crate Rust plugin architecture rather than collapsing it for publication.
- Publish the Kotlin Detekt rules JAR to Maven Central as
  `cat.aquati.monochromatic:detekt-rules`.
- Require verified control of the `cat.aquati` Maven Central namespace before extraction.
- Do not fall back to `io.github.aquaticat` if verification is delayed.
- Extend Monochromatic's existing publication workflows:
  - npm publication with package inspection and provenance;
  - crates.io publication in dependency order with package verification and provenance;
  - Maven Central publication with required metadata,
    source archive,
    documentation archive,
    signatures,
    and clean-consumer verification.
- Publish automatically when version metadata changes reach `main`.
- Retain manual workflow dispatch for dry runs and retries.
- Use manual intervention only where a registry requires initial namespace or trusted-publisher bootstrap.
- Verify every artifact from a clean external consumer before extraction begins.

### Tool refresh behavior

- Resolve the latest published tooling releases during explicit `mise run prepare` execution.
- Use those locally resolved installations until the next preparation run.
- Do not require network resolution before each lint invocation.
- Accept that two developers can temporarily use different tool releases between preparation runs.
- Accept that a newly published release can change local checks without a music-player source commit.

### Agent policy and skills

- Use published file-enforcer as the materialization engine.
- Fetch Monochromatic's current `AGENTS.md` and complete `.agents/` directory from `main` during explicit preparation.
- Include every canonical skill and bundled skill resource,
  not only `SKILL.md` files.
- Treat fetched `AGENTS.md` as shared input.
  Generate local root `AGENTS.md` by composing it with the conflict-checked local addendum.
- Use file-enforcer to generate `CLAUDE.md` and mirror canonical skills into required consumer directories.
- Keep fetched and generated policy files ignored and local rather than committing synchronized copies.
- Run `mise run prepare:file-enforcer` before starting an agent session.
  Run Pi's `/reload` command when preparation occurs inside an already-running session.
- Keep music-player-specific additive policy in `doc/agent/music-player.md`.
- Compose the local addendum with fetched policy without relying on source order to override a shared rule.
- Reject unresolved conflicts during synchronization.
  Resolve each conflict explicitly in policy before generation continues.
- Do not treat the live policy source as a live tooling-code dependency.
  Executable tooling still comes from registries.

### Local ignored artifacts

- Preserve these ignored paths in the new local checkout without publishing them:
  - `package/music-player/truepeak-core.bench/out/`
  - `package/music-player/desktop-app/dist/`
- Do not migrate build output or caches such as:
  - `target/`
  - Gradle `build/`
  - `.gradle/`
  - `.kotlin/`
  - `.cache/`
  - generated JNI libraries
- Do not delete the old ignored tree until the copied paths have been compared
 and the user has separately authorized cleanup.

### Verification boundary

- Verify only environments available from the implementation host.
- Measure available hosts,
  SDKs,
  emulators,
  and devices during implementation rather than assuming them.
- Exercise real consumer boundaries where available,
  including a desktop launch and disposable Android emulator launch.
- Do not block extraction on unavailable macOS,
  Windows,
  or physical-device environments.
- Record every unavailable verification path explicitly.

### Licensing and credentials

- Preserve existing licensing:
  - LGPL-3.0-or-later for code;
  - CC-BY-SA-4.0 for shareable documentation and content.
- Add root license texts and an explicit root README license statement in the new repository.
- Move no signing key,
  registry token,
  account password,
  or other secret into the new public repository.
- Keep signing and notarization credential custody outside Git.
- A plaintext Penpot credential found during this grilling session was rotated by the user and removed from
  `package/figma/to-penpot/TROUBLESHOOTING.import.md` in commit `612dde5f4`.
  It is outside the music-player filter set.

## Measured current state

### Product source

At the time of measurement:

- `package/music-player/` contains 246 tracked files.
- The tracked product boundary contains four components and one package-level proposal.
- Both desktop and Android depend on `truepeak-core` by relative path.
- `truepeak-core.bench` depends on the same production core by relative path.

The current source therefore supports moving the four components as one product boundary.

### Documentation reach

The current documentation inventory contains:

- 15 tracked document paths containing `music-player`;
- 34 other tracked documents that reference `package/music-player`.

Filename matching is insufficient.
Implementation must classify each document by subject and then repair both incoming and outgoing references.

### Tooling coupling

The product subtree is not independently verifiable today:

- Rust lint tasks invoke `package/linter/rust`.
- The Android KDoc task invokes `package/linter/kotlin`.
- The Rust linter executable has three first-party path dependencies.
- File-enforcer is private and currently exposes a built artifact plus monorepo-oriented source integration.
- Root Mise configuration provisions Android SDK,
  NDK,
  Rust targets,
  signing tools,
  and desktop UI tools.
- Root file-enforcer configuration enforces Cargo profiles and generates root configuration.

The registry publication prerequisite must remove executable tooling path dependencies before repository extraction.

### Git and GitHub

At the time of measurement:

- Monochromatic's default branch is `main`.
- `Aquaticat/music-player` does not yet exist.
- No music-player Git tag or GitHub release exists.
- `origin/truepeak-quarter-answer` contains no commits ahead of `main`.
- Other remote branches contain no unique product change requiring migration;
  observed music-player path touches are merge commits carrying `main`.
- Monochromatic is public.
- Monochromatic requires signed commits on protected `main` and has an active Copilot review ruleset.

### Ignored local data

At the time of measurement,
`package/music-player/` occupies 80 GB locally:

- `truepeak-core.bench/out/` occupies 141 MB;
- `desktop-app/dist/` occupies 23 MB;
- build targets and caches account for the remaining measured space.

The benchmark output includes local JSONL data that may not be reproducible without its original corpus.
It must remain ignored and private unless a separate audit authorizes publication.

## Target repository shape

The retained nesting decision produces this shape:

```text
# Aquaticat/music-player after local preparation
.
├── .agents/                  # ignored, fetched
├── .github/
├── AGENTS.md                 # ignored, generated
├── CLAUDE.md                 # ignored, generated
├── LICENSES/
├── README.md
├── doc/
├── file-enforcer.config.ts
├── mise.no-env.toml
├── mise.toml
└── package/
    └── music-player/
        ├── android-app/
        ├── desktop-app/
        ├── truepeak-core/
        └── truepeak-core.bench/
```

The exact root package-manager files follow from the published file-enforcer consumption design.
Do not copy unrelated Monochromatic package families merely to satisfy build paths.

## Future implementation sequence

The current session does not execute these phases.
A later action request must authorize them.

### Phase 1: Safeguard local ignored artifacts

- Confirm the selected ignored source paths exist and remain ignored.
- Copy them to a private staging directory outside both repositories before other migration work.
- Compare file lists,
  byte counts,
  and checksums between the original and staged copies.
- Leave the original ignored tree untouched until separate cleanup authorization.

### Phase 2: Prepare publishable tooling

- Make file-enforcer's published package self-contained at its supported public exports.
- Remove broken or source-only exports from the packed artifact,
  or include every file required by those exports.
- Ensure bundled first-party code is not also declared as unavailable runtime dependencies.
- Add package metadata,
  public access configuration,
  packed-file inspection,
  and external-consumer tests.
- Replace Rust path-only dependency assumptions with registry-compatible version plus path declarations.
- Verify each Rust crate package independently from the generated `.crate` archive.
- Add Maven coordinates,
  publication metadata,
  sources,
  documentation,
  signing,
  and Detekt consumer fixtures to the Kotlin linter.
- Keep each tool's existing runtime behavior unchanged.

### Phase 3: Extend publication workflows

- Add file-enforcer to the npm workflow without widening publication to unrelated private packages.
- Publish Rust crates in this dependency order:
  1.  `monochromatic-rust-linter-core`
  2.  `monochromatic-rust-linter-pattern`
  3.  `monochromatic-rust-linter-plugin-builtin`
  4.  `monochromatic-rust-linter`
- Confirm each upstream crate is available from crates.io before publishing its dependent.
- Verify the `cat.aquati` Maven Central namespace before publication or extraction.
- Dry-run and inspect every packed artifact before external mutation.
- Complete required first-publication bootstrap.
- Configure trusted publishing or equivalent release authentication after bootstrap.
- Trigger publication automatically when each version change reaches `main`.
- Retain manual dispatch for dry runs and retries.
- Install each published artifact in a disposable clean consumer and exercise its real function.

### Phase 4: Prepare filtered history

- Refresh the document inventory and record each path as product-owned or retained.
- Refresh from the final `main` chosen for cutover.
- Filter all historical product paths,
  including the `packages/` to `package/` rename era.
- Add product-owned document paths to the filter set.
- Keep the retained `package/music-player/` prefix unchanged.
- Do not include ignored local files.
- Verify commit count,
  oldest and newest product commits,
  author identities,
  rename continuity,
  and representative file blame.
- Compare the filtered tip's tracked product files with the selected Monochromatic commit.
- Repeat the filter or replay a bounded delta if `main` changed before cutover.

### Phase 5: Build the standalone repository tip

- Add the root README,
  licenses,
  Mise source configuration,
  file-enforcer configuration,
  Git ignore rules,
  and repository-specific metadata.
- Replace custom-linter relative paths with registry-backed tool invocations.
- Resolve latest tool releases during `mise run prepare`.
- Add the committed local addendum at `doc/agent/music-player.md`.
- Add `prepare:file-enforcer` to fetch shared policy and skills,
  reject conflicts,
  and generate ignored local policy outputs.
- Remove both `CONTEXT.md` files.
- Move product-owned docs into the new `doc/` tree.
- Repair relative and cross-repository links.
- Update Cargo repository and homepage metadata to `Aquaticat/music-player` where ownership moved.
- Preserve application identities and runtime state paths.
- Add no build or test workflow.
- Copy the staged ignored outputs into the ignored paths of the new local checkout.
- Compare the new copies with both the staged copies and untouched originals.

### Phase 6: Verify before public cutover

- Run file-enforcer and inspect every generated diff.
- Run all applicable local package lint and test tasks through Mise.
- Exercise every available end-user boundary.
- Confirm a fresh clone can prepare tools from registries without a Monochromatic source checkout.
- Confirm `mise run prepare:file-enforcer` materializes agent policy and skills in a fresh clone.
- Confirm a Pi session started afterward loads them.
- Confirm Pi's `/reload` loads them when preparation occurs after session startup.
- Confirm a disposable conflicting-policy fixture makes synchronization fail.
- Scan the complete tracked tip for credentials and unintended local identifiers.
- Confirm ignored benchmark output and desktop distribution artifacts are absent from Git objects.

### Phase 7: Create and configure GitHub repository

- Create the empty public `Aquaticat/music-player` repository without initializing files.
- Push filtered and prepared `main` before enabling signed-commit protection.
- Verify remote history and tracked contents.
- Apply repository features,
  security settings,
  merge settings,
  branch protection,
  and Copilot review ruleset.
- Create reusable labels before issue transfer.

### Phase 8: Cut over ownership

- Reconfirm the new remote tip and selected Monochromatic source tip agree on product content.
- Refresh the complete open-issue inventory and record each issue as product-owned or retained.
- Treat the first issue transfer as the irreversible-numbering cutover point.
  Start transfer only after every preceding verification and governance check passes.
- Transfer every product-owned open issue and verify redirects,
  comments,
  assignees,
  and labels.
- Remove product source and product-owned docs from Monochromatic.
- Remove music-player-specific Cargo profile enforcement.
- Remove root Android SDK,
  NDK,
  Cargo NDK,
  and preparation configuration when no remaining package consumes them.
- Remove music-player-specific ignore entries and task references.
- Repair retained multi-project documentation links.
- Add the new repository link to Monochromatic's root `README.md`.
- Run Monochromatic's affected root checks.
- Commit and push the removal as the final ownership cutover.

## Rollback

Before the first issue transfer,
rollback means deleting the pre-cutover new repository or replacing its imported branch and continuing development
 in Monochromatic.

After the first issue transfer,
rollback requires explicit acceptance that transferred issues receive different numbers when transferred back.
Transfer them back before restoring Monochromatic ownership.

After the Monochromatic removal commit,
rollback also requires two explicit actions:

- restore the removed Monochromatic paths from the parent of the removal commit;
- archive or clearly mark the new repository so two active development homes cannot exist.

Do not leave both repositories writable as authoritative homes.

## Acceptance criteria for future execution

- Every prerequisite tool is publicly available from its chosen registry and passes a clean-consumer test.
- Each prerequisite's version change on `main` triggers automatic publication,
  while manual dry-run and retry dispatch remain available.
- The new repository builds and tests through every locally available path without a Monochromatic tooling checkout.
- `doc/agent/music-player.md` is tracked as the local policy source.
- Fresh-clone preparation materializes ignored shared policy,
  skills,
  composed `AGENTS.md`,
  and `CLAUDE.md`.
- A disposable conflicting-policy fixture proves synchronization rejects unresolved conflicts.
- Pi loads prepared policy both at startup and through `/reload` after preparation.
- The new repository contains all four product components and every classified product-owned document.
- Every retained cross-repository document link is repaired and carries provenance.
- Application IDs,
  package names,
  configuration paths,
  and user-data paths match the selected Monochromatic source tip.
- The new repository contains no `CONTEXT.md` file.
- Filtered history includes the old and current product paths and representative blame remains useful.
- Initial history is imported before signed-commit protection is enabled.
- The complete pre-cutover open-issue inventory has a recorded product-owned or retained classification.
- The target label set includes the selected reusable labels and excludes Monochromatic-specific blockers.
- Every classified product-owned open issue is transferred after labels exist in the target repository.
- GitHub settings,
  transferred-issue state,
  and Copilot review match this plan.
- No build or test CI workflow exists in the new repository.
- No migration release or tag exists.
- Monochromatic contains no music-player source subtree and links to the new repository from root `README.md`.
- Every retained Monochromatic reference to moved content points to the new repository with provenance.
- Monochromatic no longer provisions product-only Android tooling.
- Preserved ignored outputs exist in the new local checkout and untouched original tree,
  remain untracked,
  and were checksum-compared.
- No credential or signing secret appears in the extracted tracked tree.

## Rejected alternatives

- A synchronized copy was rejected because it creates competing sources of truth.
- A publish-only mirror was rejected because development must leave Monochromatic.
- Flattening components to the new repository root was rejected.
- Code-only extraction was rejected because it loses product rationale and troubleshooting history.
- Snapshot-only Git history was rejected because it loses local blame and authorship context.
- Transplanting Monochromatic's linters into the new repository was superseded by registry publication.
- Following Monochromatic source for executable tooling was superseded by registry publication.
- Exact or compatible tooling version pins were rejected in favor of latest-at-prepare resolution.
- Build and test CI was rejected for initial extraction.
- Moving build caches was rejected.
- Deleting ignored artifacts during cutover was rejected.
- A tombstone at `package/music-player/README.md` was rejected in favor of a root README notice.
- Re-signing filtered historical commits was rejected because it would misstate signature provenance.
- Keeping or converting `CONTEXT.md` was rejected in favor of deriving terminology from current source.
- Collapsing the Rust linter family into one crate was rejected because it would redesign the plugin boundary.
- Manual-dispatch-only publication was rejected because latest-at-prepare depends on consistent releases.
- Committing synchronized policy and skill copies was rejected because explicit preparation is the lifecycle boundary.
- Falling back to `io.github.aquaticat` was rejected to avoid permanent coordinate fragmentation.

## Evidence commands

These commands produced the local measurements recorded in this plan:

```sh
# doc/planning/music-player-repository-extraction.md
git ls-files package/music-player | wc --lines
du --summarize --human-readable package/music-player
find package/music-player -type f | wc --lines
git ls-files 'doc/**' | rg 'music-player' | wc --lines
rg --files-with-matches 'package/music-player' doc | rg --invert-match 'music-player' | wc --lines
git log --reverse --format='%h %cs %s' -- package/music-player
git log --all --format='%h %cs %s' -- packages/music-player
git tag --list '*music*' '*player*'
gh release list --repo Aquaticat/Monochromatic --limit 100
gh repo view Aquaticat/Monochromatic --json nameWithOwner,visibility,isPrivate,url
gh issue list --repo Aquaticat/Monochromatic --state all --search 'music-player' --limit 100
gh pr list --repo Aquaticat/Monochromatic --state all --search 'music-player' --limit 100
```

GitHub's current documentation states that transferred open issues retain comments and assignees,
and retain labels only when matching labels exist in the target repository.[^github-issue-transfer]
It also states that branches requiring signed commits reject unsigned pushes,
which is why protection follows the initial filtered import.[^github-protected-branches]

npm's current documentation requires public access for a scoped public package
 and recommends inspecting package contents
before publication.[^npm-scoped-public]
Cargo's current documentation states that crates.io versions are permanent and recommends package verification
 before upload.[^cargo-publish]
Maven Central requires controlled coordinates and supports either reverse-DNS
 or GitHub-account namespace verification.[^maven-namespace]

Pi loads context files and skill descriptions at startup,
and its `/reload` command reloads context files and skills.[^pi-readme][^pi-skills]
This establishes the required sequencing for prepare-only policy materialization.

[^github-issue-transfer]:
    <https://docs.github.com/en/issues/tracking-your-work-with-issues/administering-issues/transferring-an-issue-to-another-repository>

[^github-protected-branches]:
    <https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches>

[^npm-scoped-public]:
    <https://docs.npmjs.com/creating-and-publishing-scoped-public-packages/>

[^cargo-publish]:
    <https://doc.rust-lang.org/cargo/reference/publishing.html>

[^maven-namespace]:
    <https://central.sonatype.org/register/namespace/>

[^pi-readme]:
    `@earendil-works/pi-coding-agent@0.84.1` `README.md`,
    sections "Commands" and "Context Files".

[^pi-skills]:
    `@earendil-works/pi-coding-agent@0.84.1` `docs/skills.md`,
    sections "Locations" and "How Skills Work".
