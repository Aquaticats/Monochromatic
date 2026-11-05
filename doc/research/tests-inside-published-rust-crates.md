# Who uses the test files inside a published Rust crate

Researched 2026-11-05;
the last system clock reading while writing was 2026-11-05T00:50:07Z.
Status:
research notes for `doc/planning/crates-io-footprint.md`,
section "What each version still published holds".
Nothing here is a decision;
where the text reads as a recommendation,
it is a reading of the evidence for the owner to accept or reject.

## The owner's words

The question,
2026-11-05,
verbatim:
"Can we avoid polluting crates.io too much?"

The lever the planning document proposed:
leave test files out of the packed crate.
Its only con was that someone cannot run our tests from the `.crate` alone.
The owner's answer to that con,
2026-11-05,
verbatim:
"\"some Linux distro packagers do that\" - that's not enough research."

## Short answer

- Several distributions do run the tests of a `.crate`,
  and the Rust project's crater does too.
  The full list with each one's mechanism is in the sections that follow.
- None of them requires the tests to be present.
  Each documents a way to disable or patch out tests,
  and the cost is packager work,
  not a rejection.
- The cost that matters is not the missing test files.
  It is that the published sources still declare them:
  `#[cfg(test)] mod x_tests;` with no `x_tests.rs`
  makes `cargo test` and `cargo check --all-targets` fail to compile on the published crate
  (measured on a scratch crate,
  section "Measured on a scratch crate").
  Every user that runs tests would hit that failure first.
- `cargo package` handles a dropped `[[test]]` target by removing the target and printing a warning,
  not by failing.
- The wrapper's `include` list means an `exclude` list would be ignored;
  only `!` patterns inside `include` remove files.
- Repology lists no distribution package under the three crate names queried,
  and crates.io shows no outside dependents,
  so no user of the tests was found for the crates that carry the "do not use" notice.

## What our crates look like

- `package/git-policy/cli/Cargo.toml` has an `include` list that holds `src/native/**/*.rs`,
  which already packs every `*_tests.rs` sidecar.
  It declares two `[[test]]` targets,
  `native_binary` at `src/native/binary/binary_tests.rs`
  and `native_candidates` at `src/native/candidate/candidate_consumer_tests.rs`.
- The sidecars are declared from production files with `#[cfg(test)]`,
  for example `package/git-policy/cli/src/native/lock/mod.rs` declares `mod git_stop_lock_tests;`
  under `#[cfg(test)]`,
  and `src/native/worktree/worktree_remove.rs` uses `#[path = "worktree_remove_guard_tests.rs"]`.
  Many hundreds of production files carry a `cfg(test)` declaration (`rg --count` over `src/native`).
- `forbidden-strings` (`package/forbidden-strings/cli`) and `forbidden-regex` (`package/forbidden-strings/core`)
  use the same pattern and an `include` of `src/**/*.rs`.
  Their integration test directories,
  `tests/` in each,
  are not in their `include` and so are already not published.
  Test-named files inside `src` in the repository:
  about 153 KiB of 405 KiB for `forbidden-strings`
  and about 165 KiB of 632 KiB for `forbidden-regex` (tracked `.rs` files,
  measured with `git ls-files` and file sizes).

## Distribution packagers

### Debian

What it does with the tests.
Debian builds from the crates.io tarball and runs the crate's tests at build time (in some cases) and in autopkgtests.
The Rust team book says:
"Debian uses the published crate tarball from crates.io as input for packaging"
([Rust team book, upstream guidelines][debian-upstream]).
For a crate without extra dev-dependencies,
`debcargo` writes `override_dh_auto_test` as `dh_auto_test -- test --all`,
or with `|| true` when the crate is marked broken;
for a crate with dev-dependencies it runs no build-time test
"we don't want extra B-D on dev-depends" ([`src/debian/mod.rs`][debcargo-rules]).
It writes an autopkgtest for every feature combination whose command is `cargo-auto-test`
([`debcargo.toml.example`][debcargo-example],
[`dh-cargo/cargo-auto-test`][dh-cargo-auto-test]),
with the generator's expected output `cargo-auto-test crate 1.0 --all-targets`
([`control/tests.rs`][debcargo-control-tests]).
`cargo-auto-test` changes into `/usr/share/cargo/registry/<crate>-<version>` and runs `cargo test`.
The book's own limit:
"`cargo-auto-test` only tests packaged library crates" ([packaging tools][debian-tools]).

What happens when tests are missing.
The book says:
"If this published version of the crate is lacking test input data,
or the test suite relies on
test helper crates which are not published,
some or all of the tests cannot be run as part of the packaging effort.
The resulting package will have a higher chance of being buggy as a result."
It then asks:
"If it is absolutely not possible to publish the full test suite,
please ensure that `cargo test` does work on the published crate nevertheless" ([upstream guidelines][debian-upstream]).
The tips page names the situation as ordinary:
tests that "lack test data (some upstreams exclude them when
publishing to crates.io)" "must be disabled" ([tips][debian-tips]).
`debcargo.toml` has `remove_target_types = ["test"]` to strip integration-test targets ([example][debcargo-remove]),
`test_is_broken` to mark a failing test flaky,
and `test_command` to override the command
([`debcargo.toml.example`][debcargo-example]).
The fix-problems page lists "missing test data/fixtures" among causes that are patched out or disabled
([fix problems][debian-fix]).
No rule found requires a crate to ship tests.

Cost of leaving the tests out.
If the sources keep `#[cfg(test)] mod x_tests;`,
the build-time `cargo test` and every autopkgtest fail to compile,
and the packager must patch the declarations out or set `test_is_broken`.
Debian's own wording is that the result is more likely to be buggy.

### Fedora

What it does with the tests.
The guidelines say:
"Projects from crates.io MUST be packaged from the sources that are published there"
([Fedora Rust guidelines, package sources][fedora-sources]).
`rust2rpm` generates a spec with `%bcond check`,
and the generated `%check` runs `%cargo_test`
([`rust2rpm/templates/crate.spec`][rust2rpm-spec]);
the bcond is 1 unless the configuration sets `tests.run` to false or `none`
([`rust2rpm/generator/utils.py`][rust2rpm-bcond]).
Nothing in the tool inspects the tarball for tests.

What happens when tests are missing.
The guidelines name it as a normal case:
tests can be run in parts "for example,
because parts of the test suite
or large data files are not included in published sources" ([running tests][fedora-tests]).
They also say:
"Packages for Rust crates MUST set this bcond ... by either explicitly enabling or disabling tests"
([the check bcond][fedora-bcond]),
so disabling is allowed,
and skipped tests "SHOULD include comments that explain
why this is the case" ([running tests][fedora-tests]).
A Fedora Rust packager wrote in the Cargo pre-RFC thread that "published crates don't contain input files for tests",
that they "run cargo test during packaging (where possible)",
and that tests whose data is pulled in by `include_bytes!` fail "to compile" instead of failing at run time,
which is "much more annoying to deal with" ([post 22][irlo-22]).

Cost of leaving the tests out.
The same as Debian:
a dangling `mod` declaration makes `%cargo_test` fail to compile
and the packager disables the check bcond and writes the reason in the spec.

### openSUSE

What it does with the tests.
The packaging page's own recipe starts from a git checkout,
not a `.crate`:
the `_service` file runs `tar_scm` on a git URL and `cargo_vendor` for dependencies,
and the spec's `%check` runs `%{cargo_test}` ([openSUSE Packaging Rust][suse-rust]).
So the top-level package's tests run from upstream's git tree;
vendored dependencies are fetched with `cargo vendor` and their tests are not run by this recipe.
`rust2rpm` also has an `opensuse` target that emits a spec header and changelog for crates
([`docs/rust2rpm.1.asciidoc`][rust2rpm-man]),
so a crate-based path exists;
its policy page was not found.

Cost of leaving the tests out.
None found for the git-based recipe.
For a `rust2rpm`-based package the cost would equal Fedora's,
because the spec template is shared.

### Gentoo

What it does with the tests.
`cargo.eclass` downloads `https://static.crates.io/crates/<name>/<name>-<version>.crate` for every entry in `CRATES`,
unpacks them into a vendor directory,
and "if this is our target package we need it in ${WORKDIR} too to make ${S}"
([`eclass/cargo.eclass`][gentoo-eclass]),
so a top-level package can be built from its own `.crate`.
`cargo_src_test` runs `cargo test`;
`CARGO_SKIP_TESTS` skips named tests.
The Development Guide says:
"If the packaged software is equipped with a test suite,
it is sensible to run it",
and "Sometimes it is necessary to skip tests entirely.
This can be done by setting RESTRICT=\"test\" in the ebuild"
([Gentoo Development Guide, `src_test`][gentoo-srctest]).

What happens when tests are missing.
Nothing in the eclass or guide requires tests.
Whether the tests run for a user depends on Portage settings that these two documents do not describe.

Cost of leaving the tests out.
A dangling `mod` declaration fails `cargo_src_test`,
and the ebuild author sets `RESTRICT="test"`.

### Arch Linux

What it does with the tests.
The Rust package guidelines say:
"When other sources are not available,
most Rust projects are published on crates.io"
and "The downside of this source is that it usually does not include all the test files,
license files,
or other assets normally in the other sources" ([Rust package guidelines][arch-rust]).
So Arch prefers release tarballs or git sources and treats the `.crate` as the fallback.
The `check()` function is optional:
`checkdepends` are "only considered when the `check()` function is present"
([PKGBUILD][arch-pkgbuild]).

Cost of leaving the tests out.
None found for packages built from other sources;
for a PKGBUILD that falls back to the `.crate`,
the packager omits `check()`.

### NixOS

What it does with the tests.
`buildRustPackage` has `doCheck = args.doCheck or true`
([`pkgs/build-support/rust/build-rust-package/default.nix`][nix-build]),
and the manual says "the `checkPhase` is enabled by default and runs `cargo test`"
([manual, running package tests][nix-manual]).
`fetchCrate` downloads from `https://static.crates.io/crates` ([`fetchcrate.nix`][nix-fetchcrate]),
and the manual's own `buildRustPackage` example uses it for `broot`.
GitHub code search for `buildRustPackage` under `pkgs/by-name` in `NixOS/nixpkgs` returned 2844 files,
and for `fetchCrate` together with `buildRustPackage` 209 files (2026-11-05,
approximate:
code search can be incomplete),
so most Rust packages in nixpkgs use another source.

What happens when tests are missing.
The manual lists the legitimate reasons to set `doCheck = false`:
"If no tests exist -- the `checkPhase` should be explicitly disabled to skip unnecessary build steps"
and "If tests are highly impure" ([disabling package tests][nix-manual-disable]).

Cost of leaving the tests out.
The `checkPhase` of a package fetched with `fetchCrate` fails on a dangling `mod` declaration,
and the packager sets `doCheck = false`.

### Do distributions package leaf binaries and pre-1.0 crates

- Leaf binaries:
  yes,
  in every documentation read.
  Debian policy defines "application crate" as a crate with a binary-type target and bundles those targets
  into one package ([policy][debian-policy]).
  Fedora covers applications published as single crates ([Fedora guidelines][fedora-scope]).
  The `cargo-audit` README shows a Repology packaging-status badge for a binary-only tool
  ([`cargo-audit/README.md`][rustsec-readme]).
- Pre-1.0 crates marked unstable:
  no document read forbids or requires them.
  The only version rule found is Debian's request to avoid dependencies on alpha or pre-release versions
  "unless your crate is currently itself released with an alpha/../pre-release version"
([upstream guidelines][debian-upstream]),
  which concerns pre-release version strings such as `1.0.0-alpha`,
  not `0.x`.
  Whether any distribution has refused a crate for a "do not use" README line was not found.
- Requiring tests:
  none of the six requires a crate to ship tests.
- Our crates:
  Repology's API returned an empty list for `forbidden-strings`,
  `forbidden-regex`
  and `monochromatic-git-policy-cli` (2026-11-05,
  `https://repology.org/api/v1/project/<name>`),
  so no tracked distribution carries them under those names.

## Rust project infrastructure

### crater

What it does with the tests.
Crater picks,
for every crate on crates.io,
the newest version that is not yanked
([`src/crates/sources/registry.rs`][crater-registry]).
It downloads the `.crate` through the registry's download address and unpacks it
([rustwide `src/crates/registry.rs`][rustwide-registry]).
The default mode,
`build-and-test`,
"run[s] `cargo build` and `cargo test` on every crate" ([`docs/bot-usage.md`][crater-modes]).
The code does `cargo build`,
then `cargo test --no-run`,
then `cargo test`
([`src/runner/test.rs`][crater-test]);
`check-only` runs `cargo check --all --all-targets` ([same file][crater-check]).
A per-crate `skip-tests` setting in `config.toml` turns build-and-test into build-only for that crate
([`src/runner/worker.rs`][crater-worker]).

What happens when tests are missing.
A crate with no tests at all passes:
`cargo test` succeeds with zero tests (observed on the scratch crate).
A crate whose test build fails is recorded as `BuildFail`,
and a `BuildFail` on both toolchains compares as `SameBuildFail`,
not a regression
([`src/report/mod.rs`][crater-compare]).
Because the first two steps share one result,
a dangling test declaration hides the crate's ordinary build
regressions as well:
both toolchains report the same `BuildFail`.
The Cargo team member `epage` wrote on the Cargo issue about not downloading `tests/`:
"For `tests/` and `benches/`,
that is important for crater and people wanting to do similar for crater"
([cargo#13491 comment][cargo-13491-epage]).
In the pre-RFC thread he added:
"Crater also runs on the repositories so skipping on the tests isn't a complete loss
but I can't really say to what is still lost" ([post 17][irlo-17]),
and another participant said
"it would be bad if we can't use crater any more because most things end up with broken tests" ([post 4][irlo-4]).

Cost of leaving the tests out.
If the sources stay consistent (no dangling declarations),
the crate still builds and passes with fewer tests;
crater loses its test signal and keeps its build signal.
If the sources stay inconsistent,
crater gets no usable signal from the crate.
One more note:
the documentation says `top-{n}` runs the n "most downloaded" crates,
but the registry list is sorted by a count of dependency records and reads no download count
(`registry.rs`,
the final `sort_unstable_by_key`);
how `top-n` then orders that list was not traced.
Every unyanked crate is inside a `full` run.

### docs.rs

What it does with the tests.
docs.rs builds documentation,
not tests.
Its build passes `-Zrustdoc-scrape-examples` to Cargo ([`build.rs`][docsrs-build]),
and Cargo's documentation says that by default only example targets are scraped and
"enabling `doc-scrape-examples` on test targets will not currently have any effect"
([Cargo unstable book][cargo-scrape]).

Cost of leaving the tests out.
None found;
examples matter to docs.rs,
test targets do not.

### Cargo's own publish verification

`cargo package` "Extract[s] the `.crate` file and build[s] it to verify it can build",
not test it
([`cargo-package.md`][cargo-package-man]).
The Cargo book tells authors the opposite of "include everything":
"You may want to check the size of the `.crate` file to ensure you didn't accidentally package up large assets
that are not required to build your package,
such as test data,
website documentation,
or code generation"
([publishing][cargo-publishing]).

The Cargo team's position on tests in `.crate` files,
from the issue tracker:

- On `cargo package` running tests:
  "Nothing forces a crate to include tests.
  In fact,
  there are valid reasons (circular dependencies) for people to setup their package for their dev
  dependencies to be stripped,
  making the package untestable" (`epage`,
  [cargo#14685 comment][cargo-14685-epage]).
- The team discussion:
  it "would need more design work and exploration",
  and "running tests" for some projects "can be a relatively complex thing" (`ehuss`,
  [cargo#14685 comment][cargo-14685-ehuss]).
- The issue is open;
  the verify step is described as either a packaging sanity check or a baseline quality check,
  and "As a packaging check,
  we likely don't need to run tests" (`epage`,
  [cargo#14685 comment][cargo-14685-epage-verify]).
- `epage`,
  `ehuss` and `weihanglo` are members of the Cargo team ([`teams/cargo.toml`][team-cargo]).

No guidance on including or excluding tests was found in the Rust API Guidelines
(searched the `src` directory of the repository for "exclude",
"include",
"tarball",
".crate" at the pinned commit).

### The 2026 pre-RFC thread

"[Pre-RFC] `cargo package` should include fewer files by default" on Rust Internals (April 2026) is the most direct
public discussion of this question,
and the only place several users stated their positions in their own words.
It is a discussion,
not a decision;
the proposal is not adopted.
Positions relevant here:

- A crates.io author,
  `kornel`:
  "Cargo does not run tests when publishing a package either",
  and "packages are downloaded way way more times than they're tested via Crater or packaged in a Linux distro",
  so he leans toward excluding all test files ([post 12][irlo-12]).
- A Fedora packager,
  `decathorpe`:
  missing test inputs are the second most frequent packaging problem,
  and fewer default files "would make things worse" ([post 22][irlo-22]).
- A participant,
  `Vorpal`:
  "crater and Linux distros are likely the only users of the tests currently" ([post 30][irlo-30]).
- The proposer,
  `SwishSwushPow`,
  reported that his company's reviewers first asked maintainers to drop tests,
  then "moved away from that position",
  and that when they checked the tests in such crates or the last crater runs
  "a lot of them" were already failing ([post 7][irlo-7]).

## Supply-chain tools and reviewers

### cargo-vet

It fetches and unpacks the `.crate` and diffs versions;
the only paths it skips when counting changes are
`Cargo.lock`,
`.cargo_vcs_info.json` and `.cargo-ok` ([`src/storage.rs`][vet-storage]).
Its `safe-to-deploy` criterion asks reviewers to "fully reason about the behavior of all unsafe blocks and usage of
powerful imports" and says they "are not required to perform a full logic review of the entire crate"
([`src/criteria/safe-to-deploy.txt`][vet-criteria]).
Tests are not required and are not excluded;
they only add lines to review.
Cost of leaving the tests out:
smaller diffs for reviewers.

### cargo-crev

It computes a recursive digest over the unpacked crate directory,
with an ignore list of exact relative paths
([`crev-lib/src/util/mod.rs`][crev-util]);
it does not need tests.
The repository ships a reviewing guide for agents which says "Do not trust test files to reflect real behavior"
and "Tests are useful context,
not evidence of correctness" and asks reviewers to record when they skipped the test
directory ([`skills/cargo-crev-review/SKILL.md`][crev-skill]);
that file is guidance inside the tool's repository,
not the tool's behavior.
Cost of leaving the tests out:
none for the digest;
a reviewer reads less.

### cargo-audit

It audits `Cargo.lock` ([`cargo-audit/README.md`][rustsec-readme]);
it reads no test files.
Cost of leaving the tests out:
none.

### Reviewers in the pre-RFC thread

`SwishSwushPow` (company supply-chain review) and `kornel` said fewer files make review easier,
and `kornel` cited the "Jia Tan" attack as evidence that "test files may not be harmless" ([post 12][irlo-12]).
The Fedora packager `decathorpe` also reviews crate contents and "file[s] pull requests to exclude problematic files"
([post 22][irlo-22]).

## Other users the sources show

- Archive and resume-development use:
  the author of cargo#14685 argued a `.crate` should let someone resume development
  if the repository is lost;
  `epage` answered that "I don't think a `.crate` file should be setup for that intent"
  ([comment][cargo-14685-epage]).
- Downstream maintainers who patch around missing data:
  the same issue cites a Debian patch,
  `disable-tests-missing-testdata.patch`,
  for `cargo-mutants` ([cargo#14685][cargo-14685];
  that patch URL returned
  a 404 for the version named in the issue,
  so the patch itself was not read).
- The crate author,
  as a user of their own published tests:
  `kornel` wrote that he never checks whether the tests
  still work after packaging,
  and that some users "expect my tests to work and may be wasting time investigating
  the failures" ([post 12][irlo-12]).
- People who unpack a crate to see whether a published version ever worked:
  `kpreid` ([post 3][irlo-3]) values tests being present for that.
  This is a stated preference,
  not a measured use.

## Cargo's behavior

### A dropped test target

`cargo package` passes the list of packaged files to `prepare_for_publish`,
which filters every target kind.
At commit `d6b1b43ed13a7e12072b5f1685056719546e127a` of `rust-lang/cargo`
the function is `prepare_target_for_publish` in `src/workspace/parser/mod.rs`:

```text
// src/workspace/parser/mod.rs, prepare_target_for_publish
if let Some(packaged_files) = packaged_files {
    if !packaged_files.contains(&path) {
        let name = target.name.as_ref().expect("previously normalized");
        gctx.shell().warn(format!(
            "ignoring {context} `{name}` as `{}` is not included in the published package",
            path.display()
        ))?;
        return Ok(None);
    }
}
```

The caller applies it to the library,
binaries,
examples,
tests and benchmarks
([`prepare_for_publish`][cargo-prepare]),
and `cargo package` passes `Some(&included)`
([`src/ops/cargo_package/mod.rs`][cargo-package-op]).
So a `[[test]]` whose `path` is not packaged is removed from the published manifest with a warning,
and packaging continues.
Cargo's own test `discovery_inferred_other_excluded` asserts the warning text for tests
([`tests/testsuite/package.rs`][cargo-package-test]).
A dropped `[[bin]]` is handled by the same function,
so a wrong pattern can silently remove the executable
from the published manifest (it prints "ignoring binary").
`cargo-binstall` refuses an archive that lacks a declared `[[bin]]`,
as the wrapper's `Cargo.toml` comment says.

One forum remark,
that a `[[test]]` entry missing its source "is currently an error" (`kpreid`,
[post 3][irlo-3],
marked "IIRC"),
describes a manifest that keeps the entry;
the packaging step removes it,
as measured in the section "Measured on a scratch crate".

### include, exclude and negation

- "The options are mutually exclusive;
  setting `include` will override an `exclude`.
  If you need to have exclusions to a set of `include` files,
  use the `!` operator"
  ([Cargo book, manifest][cargo-manifest-include]).
- The code agrees:
  `exclude` rules feed one matcher and `include` rules another,
  and when an `include` exists the `exclude` matcher is never consulted
  ([`src/sources/path.rs`, `_list_files`][cargo-path]).
- So the wrapper's `include` list means an `exclude` set through `file-enforcer.config.jsonc` would have no effect;
  the removal must be a `!` pattern inside `include`,
  such as `!src/**/*_tests.rs`.
- `Cargo.toml` and `Cargo.lock` are always packed regardless of patterns.
- The `regex` crate uses this form in its own manifest:
  its `include` ends with `"!/tests/fuzz/mod.rs"` (section "Measured ecosystem practice").

### Measured on a scratch crate

A scratch crate (not one of ours;
`${HOME}/temp/agent/tests-research-2026-11-05/scratch-crate`,
deleted after the run as the brief required) with
`include = ["src/**/*.rs", "!src/**/*_tests.rs", "/Cargo.toml"]`,
one `[[test]]` at `src/it_tests.rs`,
and `#[cfg(test)] mod x_tests;` in `src/lib.rs`,
run with the repository toolchain `cargo 1.101.0-nightly (31d3246e0 2026-10-27)`:

- `cargo package --list` printed `Cargo.lock`,
  `Cargo.toml`,
  `Cargo.toml.orig`,
  `src/lib.rs`:
  the negated pattern removed both test files,
  and the listing printed no warning about the dropped target.
- `cargo package` printed ``warning: ignoring test `it` as `src/it_tests.rs` is not included in the published package``,
  packaged the crate,
  and its verify step (a build) succeeded.
- The published `Cargo.toml` had no `[[test]]` entry and gained `autotests = false`.
- `cargo test` in the unpacked crate failed:
  `` error[E0583]: file not found for module `x_tests` ``,
  `` could not compile `scratch-crate` (lib test) ``.
- `cargo check --all-targets` in the unpacked crate failed with the same error.
- Adding `test = false` under `[lib]` of the unpacked manifest made `cargo test` pass with zero tests,
  but `cargo check --all-targets` still failed with `E0583`.
  So that key helps `cargo test` users and does not help crater's `check-only` mode or Debian's `--all-targets` runs
  (Debian's generated autopkgtest command passes `--all-targets`,
  see the `debcargo` text in the Debian section).

## Measured ecosystem practice

Method.
The twenty crates with the most all-time downloads on crates.io on 2026-11-05 (`/api/v1/crates?sort=downloads`),
plus `tokio`,
`regex`,
`clap`,
`anyhow` and `serde_json`,
each at its newest stable version;
the `.crate` from `static.crates.io` listed with `tar --list` and the original `Cargo.toml.orig` read;
one request per second,
user agent `monochromatic-research`.
The scripts were `measure-crates.ts` and `measure-named-crates.ts` in the scratch directory,
deleted after the run
as the brief required;
the method is the text above.
This is a sample of popular crates,
not a random sample.

Published `.crate` contains a top-level `tests/` directory:
`hashbrown`,
`syn`,
`getrandom`,
`rand_core`,
`libc` (one file),
`quote`,
`base64`,
`proc-macro2`,
`indexmap`,
`itertools`,
`thiserror`,
`cfg-if`,
`unicode-ident`,
`tokio`,
`regex`,
`anyhow`,
`serde_json`.
For example `tokio` 1.54.1 holds 193 files under `tests/`,
`thiserror` 2.0.21 holds 86.

Published `.crate` contains no top-level `tests/` directory:

- `serde` 1.0.229:
  no `include` or `exclude`,
  no `tests/`;
  one source file carries a test attribute.
- `clap` 4.6.7:
  `include` is `/build.rs`,
  `/src/**/*`,
  `/Cargo.toml`,
  `/Cargo.lock`,
  `/LICENSE*`,
  `/README.md`,
  `/examples/**/*`;
  no test code in `src`,
  examples published.
- `rand` 0.10.3:
  `include = ["src/", "LICENSE-*", "README.md", "CHANGELOG.md", "COPYRIGHT"]`;
  twenty-one of its source files still hold inline `#[cfg(test)]` or `#[test]` code.
- `bitflags` 2.13.2:
  `exclude = ["/tests", "/.github"]`;
  its source holds many inline test modules.
- `memchr` 2.8.3:
  `exclude` lists `/.github`,
  `/benchmarks`,
  `/fuzz`,
  `/scripts`,
  `/tmp`;
  inline tests remain.
- `regex-syntax` 0.8.11,
  `windows-sys` 0.61.2 and `thiserror-impl` 2.0.21:
  no `tests/`.

Two of the manifests use the `include` plus `!` form (`regex` with `"!/tests/fuzz/mod.rs"`),
and `syn` and `libc` use `include` lists that name `tests/` on purpose.
Reading:
the popular crates are split.
Most publish integration tests;
a substantial minority,
including three of the most downloaded (`serde`,
`clap`,
`rand`),
do not,
and the ones that omit `tests/` mostly keep inline unit tests in `src`,
which is the shape our sidecars have
(they are source files,
so a `src/**/*.rs` include keeps them).

## What this means for our crates

### The unstable crates

These carry the "EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE" README line and have no outside dependents.

- Users of the tests found:
  crater (every crate's newest unyanked version,
  so each of ours once published,
  in every `full` run) and the packagers covered in the distribution sections,
  none of whom carries our crates according to Repology.
  Reviewers read them but do not need them.
- What leaving them out costs,
  per user,
  is in each section.
  In short:
  no user is blocked;
  packagers would disable tests,
  crater would lose test signal,
  reviewers would read less.
- The cost that is easy to miss is the dangling declaration.
  Dropping the files while the sources keep `#[cfg(test)] mod x_tests;` makes `cargo test` and
  `cargo check --all-targets` fail on the published crate,
  and makes crater see a permanent `BuildFail`.
  Debian explicitly asks that `cargo test` keep working on a published crate with a reduced test suite.
  How to make the sources consistent was not tested here beyond the `[lib] test = false` result,
  which only partly helps.
- The `exclude` approach in the planning document would not work for the wrapper,
  because its `include` list makes
  `exclude` inert;
  the patterns must be `!` entries in `include`.
- The README line addresses packagers directly;
  nothing found makes a distribution refuse a crate that says so,
  and nothing found makes it require tests.
- Reading of the evidence (the owner decides):
  for crates with no known user,
  the evidence found does not show a
  reader who depends on the published tests;
  the owner's objection that the original con was thin was right,
  and the full list is in the distribution sections.

### forbidden-strings and forbidden-regex

- These two are the crates meant for outside use,
  with no "do not use" notice,
  and the only ones a packager or a reviewer outside the repository would plausibly pick up.
- They already omit their `tests/` directories from the published crate (`include = ["src/**/*.rs", ...]`),
  and keep their sidecar unit tests,
  which are source files.
- The saving from removing the sidecars is small:
  roughly a third of the `.rs` bytes for `forbidden-strings`
  and roughly a quarter for `forbidden-regex`,
  about 153 KiB and 165 KiB uncompressed per version.
- They have the same dangling-declaration problem if the sidecars were dropped.
- Reading of the evidence:
  for these two,
  the cost to the users who do run tests (Debian,
  Fedora,
  Gentoo,
  NixOS,
  crater)
  is larger than for the unstable crates and the saving is small,
  so the planning document's ranking "leave out > keep" fits the unstable crates better than these two.
  Whether the published tests of these two actually pass from the `.crate` was not measured,
  because the brief barred packaging the repository's crates.

## Facts, assumptions, limits and unknowns

### Facts

- Debian,
  Fedora and openSUSE-via-`rust2rpm` default to running `cargo test` on the crate;
  each documents disabling it.
- Gentoo's and NixOS's Rust tooling run `cargo test` by default for the package being built and document skipping it.
- Arch treats the `.crate` as a fallback source and says it usually lacks test files;
  `check()` is optional.
- Crater runs `cargo build`,
  `cargo test --no-run` and `cargo test` on the newest unyanked version of every crate.
- `cargo package` removes a `[[test]]` whose file is not packaged and warns;
  it does not fail.
- With an `include` list present,
  `exclude` is not consulted;
  `!` inside `include` works.
- On the scratch crate,
  `cargo test` and `cargo check --all-targets` fail with `E0583` when a `#[cfg(test)] mod`
  file is missing;
  `[lib] test = false` fixes only `cargo test`.
- docs.rs,
  cargo-audit,
  and cargo-vet and cargo-crev digests do not need tests.
- Repology returned empty lists for three of our crates by name.

### Assumptions

- That Repology's project names for our crates would equal the crate names;
  an empty list could also mean a
  different normalization.
- That Debian's installed crate directory contains `tests/` when the crate ships them
  (inferred from `cargo-auto-test` changing into that directory and running `cargo test`;
  not checked on an installed package).
- That the verification of Cargo's behavior on the repository toolchain matches the pinned source commit.

### Limits

- Primary sources only,
  but several statements are forum or issue posts by individuals
  (marked with the poster's name and role),
  not project policy.
- The openSUSE wiki,
  Arch wiki and Fedora documentation pages were fetched as rendered pages,
  not from a source repository.
- The ecosystem sample is popular crates by all-time downloads,
  not a random sample.
- Counts of distribution packages that use `.crate` sources are approximate (GitHub code search for nixpkgs only).
- No repository crate was built or packaged;
  the wrapper-specific build question belongs to a separate check.

### Unknowns

- Whether any distribution has refused or accepted a crate because of an "unstable" README notice.
- Whether our crates,
  or the repository,
  are in crater's lists,
  and whether their tests would pass in its sandbox.
- How Portage decides to run `cargo_src_test` for a user (the eclass and guide say how,
  not when).
- Whether openSUSE has a crate-based packaging policy page beyond the git-based recipe.
- Which Cargo release introduced the "ignoring test" warning;
  only the pinned commit and one nightly were read.
- What fraction of nixpkgs' Rust packages fetch from crates.io (only the approximate code-search counts).
- Whether the published tests of `forbidden-strings` and `forbidden-regex` pass when run from the `.crate`.
- Which design (gating declarations,
  stub files,
  a separate test-only crate) best keeps a reduced `.crate` consistent;
  none was built.

## Sources read

Third-party clones read in `${HOME}/temp/agent` on 2026-11-05,
with the commit read:

- `rust-lang/cargo` `d6b1b43ed13a7e12072b5f1685056719546e127a`
- `rust-lang/crater` `72694cc403967cbb298b053955b2d46ec8a180eb`
- `rust-lang/rustwide` (single files) `bc9b7f2814e548949a44e738f00e30bc15d986d1`
- `rust-lang/docs.rs` `371fc8444c132b85e1d7ed2a02231ed03af99e19`
- `rust-lang/team` (single file) `e563fa03d6cad0bf948c331c53ea57ca001acdc7`
- `rust-lang/api-guidelines` `97a0969cb07fe4cabb0eed8a56234053f47d83dc`
- `mozilla/cargo-vet` `cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472`
- `crev-dev/cargo-crev` `fb944fe955ab451f0a339636ea140eebd1a83608`
- `rustsec/rustsec` `bec1cd21fcfdd2a102109273914587bb2d51914e`
- Salsa `rust-team/debcargo` `20ed85c0329b75207ac3af7a2c9dba9414314a33`
- Salsa `rust-team/dh-cargo` `2581f5679f6d81fcaed997924582aeb30cd6c0d2`
- Salsa `rust-team/book` `aa5e573ff9636e2d19745a5ba78b76e8f148c30c`
- Codeberg `rust2rpm/rust2rpm` `91294eb4c82eb41cc38890943547bf63f76155ed`
- `gentoo/gentoo` (single file) `0a93dd6f96eb19269041f2469d44e7f5044fb0f3`
- `NixOS/nixpkgs` (sparse) `f302ca427f06601dacfaef254928ca1365b259b3`

Link confirmation (2026-11-05):

- Every commit-pinned source link (GitHub,
  Salsa,
  Codeberg) and every Internals post and issue comment link
  returned HTTP 200 when fetched.
  The line ranges were checked against the local clones at the pinned commits,
  not in the rendered page.
- Text-fragment links were opened in `agent-browser` and the quoted text was found on the rendered page for the
  Debian Rust team book (four links),
  the openSUSE page,
  and the Gentoo Development Guide.
- Unconfirmed in a browser:
  the four Fedora links and the two Arch links,
  because both sites answer the browser with an Anubis access-denied page (not worked around).
  The quoted text on those pages was read with `curl` and the user agent `monochromatic-research` instead.
- Salsa links use GitLab's `#Lstart-end` anchor form.
  The page loads and keeps the hash in `agent-browser`,
  but the line highlight itself was not confirmed.
  GitHub and Codeberg anchors (`#Lstart-Lend`) were not checked for highlight either.
- The openSUSE fragment was confirmed as text on the page,
  not as a highlighted fragment match.
  No fragment link was confirmed as a highlight,
  only as text present on the page.

[debian-upstream]: https://rust-team.pages.debian.net/book/upstream.html#:~:text=Debian%20uses%20the%20published%20crate%20tarball%20from%20crates.io%20as%20input%20for%20packaging
[debian-tips]: https://rust-team.pages.debian.net/book/tips.html#:~:text=some%20upstreams%20exclude%20them%20when%20publishing%20to%20crates.io
[debian-fix]: https://rust-team.pages.debian.net/book/fix-problems.html#:~:text=missing%20test%20data%2Ffixtures
[debian-tools]: https://rust-team.pages.debian.net/book/packaging-tools.html#:~:text=cargo-auto-test%20only%20tests%20packaged%20library%20crates
[debian-policy]: https://salsa.debian.org/rust-team/book/-/blob/aa5e573ff9636e2d19745a5ba78b76e8f148c30c/src/policy.md#L38-39
[debcargo-rules]: https://salsa.debian.org/rust-team/debcargo/-/blob/20ed85c0329b75207ac3af7a2c9dba9414314a33/src/debian/mod.rs#L507-556
[debcargo-example]: https://salsa.debian.org/rust-team/debcargo/-/blob/20ed85c0329b75207ac3af7a2c9dba9414314a33/debcargo.toml.example#L256-293
[debcargo-remove]: https://salsa.debian.org/rust-team/debcargo/-/blob/20ed85c0329b75207ac3af7a2c9dba9414314a33/debcargo.toml.example#L63-69
[debcargo-control-tests]: https://salsa.debian.org/rust-team/debcargo/-/blob/20ed85c0329b75207ac3af7a2c9dba9414314a33/src/debian/control/tests.rs#L405
[dh-cargo-auto-test]: https://salsa.debian.org/rust-team/dh-cargo/-/blob/2581f5679f6d81fcaed997924582aeb30cd6c0d2/cargo-auto-test#L21-26
[fedora-sources]: https://docs.fedoraproject.org/en-US/packaging-guidelines/Rust/#:~:text=Projects%20from%20crates.io%20MUST%20be%20packaged%20from%20the%20sources
[fedora-scope]: https://docs.fedoraproject.org/en-US/packaging-guidelines/Rust/#:~:text=primarily%20libraries%2C%20but%20also
[fedora-tests]: https://docs.fedoraproject.org/en-US/packaging-guidelines/Rust/#:~:text=not%20included%20in%20published%20sources
[fedora-bcond]: https://docs.fedoraproject.org/en-US/packaging-guidelines/Rust/#:~:text=Packages%20for%20Rust%20crates%20MUST%20set%20this%20bcond
[rust2rpm-spec]: https://codeberg.org/rust2rpm/rust2rpm/src/commit/91294eb4c82eb41cc38890943547bf63f76155ed/rust2rpm/templates/crate.spec#L324-L341
[rust2rpm-bcond]: https://codeberg.org/rust2rpm/rust2rpm/src/commit/91294eb4c82eb41cc38890943547bf63f76155ed/rust2rpm/generator/utils.py#L116-L129
[rust2rpm-man]: https://codeberg.org/rust2rpm/rust2rpm/src/commit/91294eb4c82eb41cc38890943547bf63f76155ed/docs/rust2rpm.1.asciidoc#L188-L193
[suse-rust]: https://en.opensuse.org/openSUSE:Packaging_Rust#:~:text=It%20will%20fetch%20the%20sources%20from%20git
[gentoo-eclass]: https://github.com/gentoo/gentoo/blob/0a93dd6f96eb19269041f2469d44e7f5044fb0f3/eclass/cargo.eclass#L555-L562
[gentoo-srctest]: https://devmanual.gentoo.org/ebuild-writing/functions/src_test/index.html#:~:text=If%20the%20packaged%20software%20is%20equipped%20with%20a%20test%20suite%2C%20it%20is%20sensible%20to%20run%20it
[arch-rust]: https://wiki.archlinux.org/title/Rust_package_guidelines#:~:text=The%20downside%20of%20this%20source%20is%20that%20it%20usually%20does%20not%20include%20all%20the%20test%20files
[arch-pkgbuild]: https://wiki.archlinux.org/title/PKGBUILD#:~:text=These%20dependencies%20are%20only%20considered%20when%20the%20check%28%29%20function%20is%20present
[nix-build]: https://github.com/NixOS/nixpkgs/blob/f302ca427f06601dacfaef254928ca1365b259b3/pkgs/build-support/rust/build-rust-package/default.nix#L169
[nix-manual]: https://github.com/NixOS/nixpkgs/blob/f302ca427f06601dacfaef254928ca1365b259b3/doc/languages-frameworks/rust.section.md#L305-L311
[nix-manual-disable]: https://github.com/NixOS/nixpkgs/blob/f302ca427f06601dacfaef254928ca1365b259b3/doc/languages-frameworks/rust.section.md#L343-L349
[nix-fetchcrate]: https://github.com/NixOS/nixpkgs/blob/f302ca427f06601dacfaef254928ca1365b259b3/pkgs/build-support/rust/fetchcrate.nix#L11-L17
[rustsec-readme]: https://github.com/rustsec/rustsec/blob/bec1cd21fcfdd2a102109273914587bb2d51914e/cargo-audit/README.md#L18-L38
[crater-registry]: https://github.com/rust-lang/crater/blob/72694cc403967cbb298b053955b2d46ec8a180eb/src/crates/sources/registry.rs#L174-L200
[rustwide-registry]: https://github.com/rust-lang/rustwide/blob/bc9b7f2814e548949a44e738f00e30bc15d986d1/src/crates/registry.rs#L175-L181
[crater-modes]: https://github.com/rust-lang/crater/blob/72694cc403967cbb298b053955b2d46ec8a180eb/docs/bot-usage.md#L80-L87
[crater-test]: https://github.com/rust-lang/crater/blob/72694cc403967cbb298b053955b2d46ec8a180eb/src/runner/test.rs#L327-L378
[crater-check]: https://github.com/rust-lang/crater/blob/72694cc403967cbb298b053955b2d46ec8a180eb/src/runner/test.rs#L393-L414
[crater-worker]: https://github.com/rust-lang/crater/blob/72694cc403967cbb298b053955b2d46ec8a180eb/src/runner/worker.rs#L283-L297
[crater-compare]: https://github.com/rust-lang/crater/blob/72694cc403967cbb298b053955b2d46ec8a180eb/src/report/mod.rs#L494-L539
[docsrs-build]: https://github.com/rust-lang/docs.rs/blob/371fc8444c132b85e1d7ed2a02231ed03af99e19/crates/lib/docs_rs_rustwide/src/build.rs#L548-L552
[cargo-scrape]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/doc/book/src/reference/unstable.md#L1396-L1411
[cargo-package-man]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/doc/man/cargo-package.md#L39-L42
[cargo-publishing]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/doc/book/src/reference/publishing.md#L87-L91
[cargo-manifest-include]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/doc/book/src/reference/manifest.md#L427-L429
[cargo-path]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/sources/path.rs#L580-L640
[cargo-prepare]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/workspace/parser/mod.rs#L3205-L3216
[cargo-package-op]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/ops/cargo_package/mod.rs#L887
[cargo-package-test]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/tests/testsuite/package.rs#L5312-L5350
[cargo-13491-epage]: https://github.com/rust-lang/cargo/issues/13491#issuecomment-1964582591
[cargo-14685]: https://github.com/rust-lang/cargo/issues/14685
[cargo-14685-epage]: https://github.com/rust-lang/cargo/issues/14685#issuecomment-2435383395
[cargo-14685-ehuss]: https://github.com/rust-lang/cargo/issues/14685#issuecomment-2444649480
[cargo-14685-epage-verify]: https://github.com/rust-lang/cargo/issues/14685#issuecomment-2556048428
[team-cargo]: https://github.com/rust-lang/team/blob/e563fa03d6cad0bf948c331c53ea57ca001acdc7/teams/cargo.toml#L5-L25
[vet-storage]: https://github.com/mozilla/cargo-vet/blob/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/src/storage.rs#L85-L86
[vet-criteria]: https://github.com/mozilla/cargo-vet/blob/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/src/criteria/safe-to-deploy.txt#L1-L10
[crev-util]: https://github.com/crev-dev/cargo-crev/blob/fb944fe955ab451f0a339636ea140eebd1a83608/crev-lib/src/util/mod.rs#L50-L66
[crev-skill]: https://github.com/crev-dev/cargo-crev/blob/fb944fe955ab451f0a339636ea140eebd1a83608/skills/cargo-crev-review/SKILL.md#L473-L477
[irlo-3]: https://internals.rust-lang.org/t/pre-rfc-cargo-package-should-include-fewer-files-by-default/24188/3
[irlo-4]: https://internals.rust-lang.org/t/pre-rfc-cargo-package-should-include-fewer-files-by-default/24188/4
[irlo-7]: https://internals.rust-lang.org/t/pre-rfc-cargo-package-should-include-fewer-files-by-default/24188/7
[irlo-12]: https://internals.rust-lang.org/t/pre-rfc-cargo-package-should-include-fewer-files-by-default/24188/12
[irlo-17]: https://internals.rust-lang.org/t/pre-rfc-cargo-package-should-include-fewer-files-by-default/24188/17
[irlo-22]: https://internals.rust-lang.org/t/pre-rfc-cargo-package-should-include-fewer-files-by-default/24188/22
[irlo-30]: https://internals.rust-lang.org/t/pre-rfc-cargo-package-should-include-fewer-files-by-default/24188/30
