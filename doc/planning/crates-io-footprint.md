# How much this repository puts on crates.io, and how to put less there

Status: proposal; the owner's answers are pending.
Nothing here is decided until the owner picks;
nothing on crates.io, in the toolchain lock or in the publish workflow has been changed for it.

## The owner's question

2026-11-05, verbatim: "Can we avoid polluting crates.io too much?"

## The owner's answers (2026-11-05, verbatim, to the three questions in "Options")

- Where our tools install from:
  "Not enough research. The best solution is obviously a workspace: protocol or something.
  The second best solution is floating git. The third best solution is GitHub package registry.
  Figure out which ones are workable first."
  So the option list in "Where our own tools install from" was incomplete;
  the owner's ranking is a `workspace:`-like protocol, then floating git, then GitHub's package registry,
  and the next step is research into which of them can work here.
- What happens to what is already there:
  "One release each. And we publish less to crates.io . That fact is given."
  Decided: one release of each crate (carrying the README notice), and fewer crates.io publishes from now on.
- Test files in the packed crate:
  "\"some Linux distro packagers do that\" - that's not enough research."
  Open: who runs the tests of a published crate needs evidence before the choice.

## What crates.io holds now

Read from the public API at about 2026-11-05T00:10Z
(`~/temp/agent/issue-6261/crates-io-footprint.ts`, `~/temp/agent/issue-6261/crates-io-users.ts`):

- 16 crates, 132 versions in all, 113 of them published in the last 30 days.
- About 132 MiB of `.crate` files stored in all.
- `monochromatic-git-policy-cli` alone: 64 versions between 2026-10-13T14:23:11Z and 2026-11-04T20:27:47Z,
  about 119 MiB, so about nine tenths of the bytes.
- Every crate has exactly one unyanked version, after the yank in `doc/planning/crate-release-yank-after-publish.md`.
- No crate outside this repository depends on any of ours:
  every reverse dependency crates.io lists is one of our own crates
  (`monochromatic-jsonc-edit` by five of ours, `monochromatic-catalog-specifier` and
  `monochromatic-dependency-catalog` by two, the rest by one or none).
- `crates.io/api/v1/crates/monochromatic-git-policy-cli/owners` lists one owner, the account `Aquaticat`.

Not on crates.io: `monochromatic-lint-closure`, `wg-quicker-exempt`, `truepeak-core`,
`monochromatic-codex-plugin-bash-limit`.

## Why yanking does not reduce it

A yank only stops new lock files from picking a version;
the version, its files and its page stay on crates.io.
crates.io cannot delete single versions at all;
its owner-side deletion removes a whole crate
(`https://crates.io/policies`, section "Package Ownership", read 2026-11-05).
So the yank the owner asked for, and the automatic yank step planned in `package/crate-release/cli`,
hide old versions but store just as much.

## Where crates.io sits in our installs

- Every tool release goes to two places from `.github/workflows/cargo-publish.yml`:
  `cargo publish` to crates.io,
  and an immutable GitHub release holding the built program for each Linux target.
- The toolchain lock installs seven of our tools through the `cargo:` route with `cargo-binstall`
  (`toolchain.lock.json`: `monochromatic-lint`, `monochromatic-git-policy-cli`, `monochromatic-cargo-shadow`,
  `monochromatic-pnpm-shadow`, `monochromatic-search-shadow`, `monochromatic-file-enforcer-cli`,
  `monochromatic-nested-wayland-session`).
  `cargo-binstall` reads the version and the `[package.metadata.binstall]` table from crates.io,
  then downloads the program from the GitHub release.
  So crates.io is only the place the version number and the download address are read from.
  These lock rows carry no checksum (`"artifacts": {}`).
- `mpm-shadow` already installs without crates.io:
  its lock row is `"request": "github:Aquaticats/Monochromatic@latest"`,
  `"route": "archive"`, `"tagPrefix": "monochromatic-mpm-shadow-v"`,
  with the archive address and its SHA-256 for every Linux target in the lock.
  The same route runs postinstall links (`package/mpm-shadow/cli/README.md`, "Postinstall links the lock no longer declares").
- The library crates are on crates.io only because `cargo publish` of a tool needs every dependency on crates.io;
  no other crate on crates.io uses them.
  `forbidden-strings` and `forbidden-regex` are the exception the owner named
  (no "do not use" notice, no yank); `monochromatic-git-policy-cli` depends on `forbidden-strings`.

## What makes the footprint big

### How often we publish

64 versions of one crate in about three weeks came from publishing after nearly every merge
(owner, earlier: "I'm authorizing you to bump and publish any updates this session.").
No other change shrinks the count as much as publishing less often.

### What each version holds

`cargo package --list` for `monochromatic-git-policy-cli` at `1942f0de53`
packs 1,170 files, 13,214 KiB before compression;
602 of them are test files (`*_tests.rs` and test support), 6,877 KiB, about half.
The largest single file is the 176 KiB `README.md`.
Leaving test files out of the packed crate would roughly halve every future version of it.
Whether the packed crate still builds without them is being measured
(`cargo package` with verification in a throwaway worktree; result to be added here).

### How many crates

The owner's crate split (`doc/planning/cli-git-source-layout.md`, "Splitting into crates (owner, 2026-11-04)")
makes each reusable part its own published crate with exact version pins.
On crates.io each new crate means:

- a new name;
- a first publish by hand, because Trusted Publishing needs the crate to exist first
  (`doc/runbook/publish-crate-first-time.md`);
- the new-crate rate limit, a burst of five and then one every ten minutes
  (`https://crates.io/docs/rate-limits`, read 2026-11-05);
- a new version of every crate that pins a changed one.

## Deleting crates

crates.io's own code, `src/controllers/krate/delete.rs` at `e0edaf98831e94b4a6d0969b957bda0094a45f3d`:

- An owner may delete a crate within 72 hours of its first publish,
  or later if it has one owner, fewer than 1,000 downloads for each month since it was published
  (`DOWNLOADS_PER_MONTH_LIMIT: u64 = 1000`),
  and no reverse dependency on crates.io.
- The name is free for anyone 24 hours after the deletion (`AVAILABLE_AFTER: TimeDelta = TimeDelta::hours(24)`).
- Deleting the crate row deletes its versions and their dependency rows
  (`migrations/20170708133715_ensure_crate_id_foreign_keys_cascade/up.sql` and
  `migrations/20170708135123_ensure_version_id_foreign_keys_cascade/up.sql`: `ON DELETE CASCADE`),
  so once a tool crate is gone, the libraries only it used have no reverse dependency left.

Measured downloads (all-time): `monochromatic-git-policy-cli` 709, `forbidden-strings` 724,
every other crate of ours 560 or fewer, all well under the limit.
The tools nothing depends on today:
`monochromatic-git-policy-cli`, `monochromatic-cargo-shadow`, `monochromatic-pnpm-shadow`, `monochromatic-mpm-shadow`,
`monochromatic-search-shadow`, `monochromatic-nested-wayland-session`, `monochromatic-file-enforcer-cli`.
Deletion cannot be undone, and the freed name can then be taken by a stranger.

## Options

Three separate choices, asked separately.

### Where our own tools install from

- A. From the GitHub release, as `mpm-shadow` already does:
  move the seven `cargo:` lock rows to the `archive` route,
  remove the crates.io publish steps for tools and for the libraries only they use,
  and mark those crates `publish = false`.
  Pros: day-to-day releases add nothing to crates.io;
  the split crates never need crates.io names, first publishes by hand, or the new-crate limit;
  the lock gains a SHA-256 per archive;
  installs can no longer fall back to compiling (`doc/troubleshooting/mise-cargo-binstall-falls-back-to-compiling.md`).
  Cons: one-time work in the lock, the plan and the workflow;
  the crates.io copies stop at their current version,
  without the "do not use" notice unless one more release carries it;
  someone running `cargo install` gets that old version.
- B. Keep crates.io, publish less often (for example only when the toolchain lock needs the change).
  Pros: no change to how tools install.
  Cons: still a version per release;
  the crate split still needs a name, a first publish by hand and new versions of the pinning crates for every part;
  fixes reach the repository's own commits later.
- C. Keep publishing after nearly every merge, with the yank step.
  Cons: about three versions a day of the wrapper alone, more with the split; yanks store nothing less.

Ranking: A > B > C.
A beats B because B still adds a version per release and the split multiplies them, while A adds none,
and its one-time route change is the one `mpm-shadow` already runs.
B beats C because it adds fewer versions at no install change.

### What happens to what is already there

- a. Delete the crates that qualify, after A has moved installs off them:
  the seven tools first, then the libraries left without reverse dependencies,
  keeping `forbidden-strings` and `forbidden-regex`.
  Pros: removes 113 of the 132 versions and about 129 MiB of the 132 MiB.
  Cons: cannot be undone; each name is free for anyone after 24 hours.
- b. One last release of each crate carrying the README notice, then nothing more.
  Pros: crates.io shows "EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE", as the owner asked.
  Cons: one more version of each crate.
- c. Leave them as they are.
  Cons: crates.io keeps showing the READMEs without the notice.

Ranking: a > b > c, if the owner accepts the name risk.
a beats b because it removes what is stored instead of adding to it;
b beats c because it puts the owner's notice where crates.io users see it.

### What each version still published holds

- Leave test files out of the packed crate (an `exclude` list set through `file-enforcer.config.jsonc`,
  next to the `**/uses` exclude the layout plan already needs).
  Pros: about half the bytes of each wrapper version.
  Cons: someone cannot run our tests from the `.crate` alone.
- Keep them in.

Ranking: leave out > keep, because no one outside the repository runs these tests
and the repository runs them from its own checkout.

## Not verified yet

- That the packed `monochromatic-git-policy-cli` builds with its test files left out (measurement running).
- That every postinstall form the seven tools use (`link`, and `run` for `search-shadow`) works on the `archive` route;
  the README documents both for the lock, and `mpm-shadow` itself uses `link`.
