# monochromatic-lint open issues: what still held, and what was fixed

## Purpose

The owner asked for the open issues whose titles name `monochromatic-lint` to be checked against
today's code and fixed where they still hold.
This file is the evidence for that batch at commit `57c2b7773c` (2026-11-09T00:56:36-05:00),
so the coordinator can comment on the issues without redoing the checks:

- six regex-audit issues name a script and line that no longer exist,
  so nothing changed for them;
- three issues held and are fixed,
  each in its own commit;
- one holds and waits on an owner decision,
  so it is untouched;
- one occurrence of the same regex pattern has no issue and no gate that reaches it.

Delete this file once the issues carry this evidence.

## The regex-audit issues no longer hold

Issues #720 to #725 ask for a regex to be replaced in one of two `.mjs` scripts.
Both scripts are gone:

- `package/linter/monochromatic-lint.fuzz/bin/container.mjs` was deleted in `ba6a0c3d7c`,
  along with `bin/planted-controls.mjs`;
- `package/linter/monochromatic-lint/bin/mutate-container.mjs` became
  `bin/mutate-container.ts` in `6682fc5d82`,
  and `bin/mutation-scopes.ts` took its scope table.

### The fuzz container script (#720)

The commit message of `ba6a0c3d7c` records the outcome for both patterns it removed:
"the two regular expressions (nextest status lines,
the executed-unit count) are line scans".
The executed-unit count is `executedUnitsIn` (`package/linter/monochromatic-lint.fuzz/bin/fuzz-run.ts:74`),
which splits the output on the label at line 29 and scans the leading decimal digits.

### The linter campaign script (#721 to #725)

The campaign runner now lives in `package/mutation-test/cli-cargo-mutants/src`,
and each audited pattern is already the string form those issues propose:

- `/^(\d+)\/(\d+)$/u` (#725) is `takeShard`,
  which splits on `shardSeparator` (`scope-tools.ts:339`)
  and validates each side with a digit scan and a numeric compare;
- `/^[a-f0-9]{64}$/u` (#721,
  #723,
  #724) is `isContentAddressedId` (`campaign-files.ts:42`),
  a length compare at line 43 and a character walk at line 46;
- `/^sha256:/u` (#722) is `String#replace` with the plain text `'sha256:'`
  (`campaign-container.ts:176`),
  so no pattern is involved.

### How the absence was checked

`mise run //package/linter/monochromatic-lint:lint:scripts:oxlint` over the five `bin` scripts
reports "Found 0 warnings and 0 errors" with 490 rules,
and the same task over the eight fuzz scripts reports the same.
That gate now exists in both packages,
so the issues' premise that no `lint:oxlint` task reaches
these files is also stale;
the remaining coverage gap is #679.

The rule was shown to fire in that same setup before trusting the clean result:
a scratch file holding one regex literal in the fuzz `bin` folder reported
`no-restricted-syntax(no-regex): Regex literal requires a scoped disable with justification`
and "Found 0 warnings and 2 errors",
and was deleted before any commit.

## Fixed: the gnu release archives named a floating runner (#1830)

`.github/workflows/cargo-publish.yml` built the `x86_64-unknown-linux-gnu` leg of the
forbidden-strings matrix (`build-binary`) and of the monochromatic-lint matrix (`ml-build-binary`)
on `ubuntu-latest`,
while seven sibling matrices already named `ubuntu-24.04` with the reason in a
comment.
Both gnu legs now name `ubuntu-24.04`,
with the same comment.
The musl legs stay on `ubuntu-latest`:
a statically linked musl binary carries no glibc floor.

Checked with a scratch script over the parsed workflow:
the pre-fix blob reports two floating gnu runners and the working tree reports none,
and the file still parses as 56 jobs.
Commit `57c2b7773c`.

## Fixed: the max-lines rule was attributed to a crate that does not exist (#1954)

`doc/philosophy/agents.md:802` named `monochromatic-rust-linter` as the enforcing rule.
The linter crates here are `monochromatic-lint`,
`monochromatic-lint-closure` and
`monochromatic-lint.fuzz`,
and the rule key is `rust/max-lines`
(`package/linter/monochromatic-lint/src/configuration_rules.rs:29`).
The sentence now names the rule key and the shipping tool,
as the same doc already does for `rust/max-lines` and `rust/require-rustdoc`.
Commit `59db7add7b`.

## Fixed: the qmake prerequisite comment described a discovery mode that does not exist (#4267)

`package/file-manager/variant-qt/monochromatic-lint.prerequisites.jsonc:2` said cxx-qt-build locates
Qt by running bare `qmake` and that Fedora installs only `qmake6`.
`qt-build-utils` tries `QMAKE` first,
then the PATH candidates `qmake6`,
`qmake-qt5`,
`qmake` in that
order (`qt-build-utils-0.9.1/src/installation/qmake.rs:72`,
unchanged in the 0.10.0 this package
resolves),
and runs `qmake -query` to read Qt's version.
The comment now states that order and the check's own probe (`qmake6` then `qmake`,
`package/linter/monochromatic-lint/src/closure_probe.rs:203`).
Comments only:
the parsed object is unchanged.
Commit `5b13d5986e`.

The `mise.toml` and `README.md` copies of the same claim belong to #4265,
which is open.

## Not fixed: nothing runs the whole linter (#6267)

The claim still holds:

- no workflow runs the whole tree:
  `.github/workflows/advisory-core.yml:213` lints only the
  `.md`,
  `.mdx` and `.rs` files a push touched,
  and `mise run lint` is not invoked by any workflow;
- the commit policy's `markdown/autofix` runs one rule at `warn`
  (`package/git-policy/cli/README.md:2226`:
  `"rules": ["lfs-image-url"]`).

The issue lists four options (apply the fixer to staged Markdown,
a CI job,
a daily issue,
a
refusing commit policy) and the owner's answer was "None for now".
Choosing among them is an owner decision,
so this batch changed nothing for it.

## The remaining occurrence of the audited pattern

`package/linter/monochromatic-lint/mise.toml:242` and `:243`,
in the inline `shell` script that
snapshots the tested processors image,
still carry
`replace(/^sha256:/u, '')` and `/^[a-f0-9]{64}$/u.test(base)`.
Blame puts both on `ff5706ef32` (2026-10-05),
before the audit filed its issues,
and no gate reaches a script embedded in a `mise.toml` string,
so neither the audit nor `lint:scripts:oxlint` reports it.
It was left alone:
no issue covers it,
and verifying a change there needs podman and a built base image,
while the owner has paused mutation campaigns.

## What the coordinator still needs to do

Comment on and close #720 to #725 with the evidence above,
and on #1830,
#1954 and #4267 with the
commit hashes once this branch is merged.
#6267 needs an owner choice among its four options before any work.
A new issue is worth filing for the `mise.toml:242` occurrence.
