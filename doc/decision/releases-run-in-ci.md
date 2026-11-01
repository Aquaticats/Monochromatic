# Rust crate releases run in CI

## Status

Accepted,
2026-11-01,
owner-chosen.
This decision is in force.
It is implemented by `.github/workflows/crate-release.yml` and `package/crate-release/cli`.
Its first two runs,
read with `gh run view` on 2026-11-01:
the dry run `41287463221` of forbidden-strings 0.6.1 succeeded,
and the real run `41303293802` failed at its push step,
about five minutes after it was created,
because `main` had moved ("Why a moved main is settled by a comparison").
The retry that answers that failure has not yet run on a runner (see "Limits").

## Context

Releasing a Rust crate (`forbidden-strings`,
`monochromatic-git-policy-cli`) ran on the owner's machine:
the `release` mise task of the crate edited the manifests and the exact pins of the crate,
ran `cargo check` and `cargo publish --dry-run` in a capped systemd unit,
and committed;
a person then pushed,
and the push started `.github/workflows/cargo-publish.yml`,
which compares `Cargo.toml` before and after and publishes.
The checks compile the crate,
and the scanner's `build.rs` makes that compile heavy.

## Owner's words

The repository owner,
on 2026-11-01:

> Why are we even running the release on our own maachine at all?

The owner then chose the option "Release from CI".
The option text the owner saw was:

> Add a `workflow_dispatch` release workflow (crate,
> from,
> to,
> note) that runs the existing release script on a GitHub
> runner,
> commits and pushes the bump to main;
> `cargo-publish.yml` then publishes as today.
> Locally I only dispatch it
> and watch.

The owner also wrote:

> I approve all the releases this sesion.

and

> main is not sacred,
> so as the private registry.

## Decision

A release of a crate that `package/crate-release/cli/releasable-crates.json` lists runs on a GitHub runner.
Locally a person only dispatches `crate-release.yml` and watches it.
Nothing is compiled on the developer machine.

- The crate's `release` mise task (`native:release` for the wrapper) runs
  `package/crate-release/cli/bin/dispatch-release.ts`,
  which starts the workflow with `gh workflow run` and prints the run URL.
  `release:wait` (`native:release:wait`) runs `await-release.ts`,
  which follows the release run,
  then the `cargo-publish.yml` run it started,
  then checks crates.io and the GitHub release.
- The workflow's `release` job runs `release.ts --crate <name> --host runner`,
  the existing release script with the arguments of the registry,
  then `push-release.ts`.
  Its `publish` job dispatches `cargo-publish.yml` on `main`.
- The arguments of each crate live in the registry file only.
  Reading them out of each package's `mise.toml` was rejected because the local task no longer carries them,
  and a data file in each package was rejected because finding a crate would need a scan
  and the workflow's `choice` list needs one enumeration anyway.
  A unit test (`release-workflow.unit.test.ts`) compares the workflow's `crate` choices with the registry
  and the registry with the repository.
- The workflow is mise-free (`doc/decision/mise-free-ci.md`):
  `.github/actions/task-cli-setup` installs Node and pnpm,
  `rustup` installs the toolchain,
  and the lockfile refresh that used a mise task is `cargo update --workspace` in a directory.
- Only `workflow_dispatch` starts it.
  One `concurrency` group serializes releases without cancelling a running one.
  The token has no permission by default;
  the `release` job holds `contents: write` and the `publish` job holds `actions: write`.
  A run on a ref other than `main` fails at its first step.
- `from` must equal the crate's current version.
  The release command refuses any other value with the manifest and both versions named,
  and with `--host runner` it also refuses a crate that already holds `to`.
- `dry-run` runs everything except the push and the publish.

## Why the push is an API commit

Branch protection on `main` requires signed commits.
Measured on 2026-11-01 with read-only `gh api repos/Aquaticats/Monochromatic/branches/main/protection`:
`required_signatures.enabled` is true,
`enforce_admins.enabled` is false,
and the response lists no pull request review requirement;
the latest commit of `main` reports `verification.verified: true`.
A runner has no signing key,
and no repository secret holds one,
so an unsigned commit pushed with `git push` would be refused.
The GraphQL mutation `createCommitOnBranch` is documented,
by the schema's own description,
as follows:

> Commits made using this mutation are automatically signed by GitHub if supported and will be marked as verified in the
> user interface.

(read with a `__type(name: "Mutation")` introspection query through `gh api graphql` on 2026-11-01).
`push-release.ts` therefore rebuilds the local release commit as that mutation:
on the parent that was checked out,
with the same subject and the whole contents of the files it changes,
and `expectedHeadOid` set to that parent,
so GitHub refuses it when `main` moved.
The commit is authored by the credential that sends it,
which is `github-actions[bot]`.

## Why a moved main is settled by a comparison

The real run `41303293802` was refused with
`Expected branch to point to "badf75b1e556b13256f6c8dd25017f2ce9e1ad7f" but it did not.  Pull and try again.`
Nothing was pushed or published,
which is the guard working,
but other sessions push to `main` every few minutes,
so a plain re-dispatch usually loses the same race.
The owner chose "Release from CI",
so it must work on a busy `main`.

The release verified the crate at the commit the checkout started from.
A commit rebuilt on a newer head is as safe as that verification only if nothing the release depends on changed
between the two.
`push-release.ts` therefore records those paths before it sends anything,
and settles a refusal for a moved branch as follows.

- The watched paths are the files of the release commit,
  the directory of the crate,
  the directories whose lockfile the release refreshed,
  every path dependency in the closure of the crate and of each lock directory as `cargo metadata --locked` reports it
  at the checkout,
  and the manifest and lockfile of each workspace root.
  The closure is read from the checkout the check and the dry run built,
  not guessed from directory names.
  Run for the registered crates on 2026-11-01,
  `cargo metadata --locked` answered in well under a second for each manifest,
  and for `forbidden-strings` the path dependencies it listed were
  `package/forbidden-strings/core`,
  `package/git-policy/cli` and `package/jsonc-edit/core-rust`,
  none of them a directory of the crate itself.
- On a refusal it reads the new head with the reference endpoint and compares the checkout base with it through the
  compare endpoint.
  A changed path counts when it is a watched file,
  or is or lies below a watched directory,
  and a renamed file counts by its old and its new path.
- When none counts,
  it sends the same commit again,
  expecting the new head.
  Each send is preceded by a fresh comparison from the checkout base,
  and the sends are bounded (six by default,
  the first included).
- When one counts,
  when the head or the comparison cannot be read,
  when the branch is not ahead of the checkout,
  or when the branch moved before every allowed send,
  it pushes nothing,
  names the cause and fails the run:
  dispatch the release again.
- Every step is printed:
  the base,
  the watched paths,
  each new head,
  the comparison and the decision.

The compare endpoint lists at most three hundred files for a whole comparison
([GitHub documents][compare-file-limit]:
"The list of changed files is only shown on the first page of results,
and it includes up to 300 changed files for the entire comparison".
The link is the section anchor,
which a browser check on 2026-11-01 confirmed scrolls to the section;
a text fragment on the sentence did not scroll there).
Measured on 2026-11-01 over the latest hundred commits of `main`,
a thirty-minute window ending at one commit held 108 commits and reached the limit,
and the other four sampled windows held between two and 165 files.
A list that reaches the limit may have been cut and cannot show a path unchanged,
so then the decision uses the object names of the watched paths instead:
one GraphQL query reads each path at the base and at the new head,
and equal names prove a path unchanged whatever else changed,
because the name of a tree covers everything below it.
A query for a directory,
a file and a missing path was run read-only on 2026-11-01 and answered as described:
a name for each that exists,
`null` for the one that does not.

## Why the publish is dispatched

Events caused by the repository's `GITHUB_TOKEN` start no workflow,
except `workflow_dispatch` and `repository_dispatch`
([GitHub documentation][token-trigger-rule],
text fragment confirmed in a browser on 2026-11-01).
The commit from the push step therefore starts no `push` run of `cargo-publish.yml`,
and the `publish` job starts it with `gh workflow run cargo-publish.yml --ref main`
and `actions: write`.
`cargo-publish.yml` publishes the crate named by a dispatch whatever changed
and reads the crate's version from the commit it runs on,
so the dispatch names `main` after the bump is there,
and it sets `dry-run` to false because its default is true.

A `workflow_call` of `cargo-publish.yml` was rejected.
A called workflow checks out the caller's commit,
which is the commit before the bump,
so every checkout step of a three-thousand-line workflow would need a ref,
and crates.io Trusted Publishing is configured for the workflow filename `cargo-publish.yml`
(`doc/runbook/publish-crate-first-time.md`).
A personal access token that makes the push start the workflow was rejected,
because the repository rule is to use `GITHUB_TOKEN` and the existing secrets only.

## Consequences

- The push of the bump starts no workflow,
  so the checks that run on `push` to `main`
  (the scanner scan,
  for one) skip that commit.
  They run on the next push.
- A release that fails after its push leaves the bump on `main` without a publish.
  The recovery is in the package README:
  a fresh dispatch of `cargo-publish.yml` when nothing was uploaded,
  and `gh run rerun <run id> --failed` once the crate upload succeeded,
  because the `forbidden-strings` block of that workflow has no skip for a version that is already on crates.io
  (read at `.github/workflows/cargo-publish.yml`,
  jobs `detect` and `publish-crate`;
  the other crates' publish steps check crates.io first).
- The local `--host local` mode of the release command still exists,
  compiles the crate on the machine that runs it,
  and is used by no task.

## Limits

- The workflow has not run.
  It cannot be dispatched before it is on the default branch,
  so the first release is its first run.
  Run it once with `--dry-run` first.
  What the unit tests prove is the argument,
  request and command construction,
  the registry against the repository,
  and the workflow file's shape;
  they do not prove that a runner can compile the crates,
  that the token may call
  `createCommitOnBranch` on the protected branch,
  or that GitHub signs that commit under `GITHUB_TOKEN`.
- Rulesets and branch protection were read,
  not exercised.
  A rule that this record does not list would refuse the push.
- The retry is proved against a fake GitHub in unit tests:
  the unchanged case sends again and lands,
  a changed path of each kind refuses,
  the send limit refuses,
  the path filter is exact,
  and a list at the limit is decided by object names.
  Each of those tests was shown to fail when its guard was removed.
  The retry has not run against the real API.
  Its first real use is the next release.
- Files that a build script or `include_str!` reads from outside the crate and its path dependency directories are not
  in `cargo metadata`,
  so a change to them on a moved `main` is not seen.
  `cargo publish --dry-run` packages the crate directory only,
  so such a read in code it compiles would already fail it.
  The one found in the two registered crates,
  with `rg` on 2026-11-01,
  is in a `#[cfg(test)]` module
  (`package/forbidden-strings/cli/src/rule/frx/compile_tests.rs` reads the repository's `forbidden-strings.append.txt`),
  which neither the check nor the dry run compiles.
- Only the two crates with release tasks are in the registry.
  The other crates of `cargo-publish.yml` are released by a bump pushed to `main`,
  as before.

[compare-file-limit]: https://docs.github.com/en/rest/commits/commits#compare-two-commits
[token-trigger-rule]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow#:~:text=events%20triggered%20by%20the%20GITHUB_TOKEN%20will%20not%20create%20a%20new%20workflow%20run
