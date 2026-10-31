# Rust file-enforcer rewrite handover

## Purpose

Carry the Rust + clap + JSONC file-enforcer rewrite across the user's reboot
and into implementation.
The design is accepted:
[`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md).
Every question,
answer,
and its history is in
[`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md).
Implementation is tracked in issue #1818.

## State on 2026-10-29

- The decision record is accepted and committed,
  with the dependency approvals (`toml_edit`,
  `browserslist-rs`,
  `schemars`,
  `serde_json`) and the in-repo XML edit crate.
- All three gate probes finished
  ([`doc/research/file-enforcer-rust-rewrite-gate-probes.md`](../research/file-enforcer-rust-rewrite-gate-probes.md)),
  and the user then chose direct registry lookups for every npm and PyPI version.
- The four halted vet drafts are committed whole under `doc/audit/tech-file-enforcer-*-vet-2026-10-29.md`.
  Their scratch (crate downloads,
  clones,
  and extractions) stays under `${HOME}/temp/agent/`.
- No code exists yet.

## Update 2026-10-30:
in-repo browserslist and parallel implementation

The oxc look is done
([`doc/research/file-enforcer-rust-rewrite-oxc-browserslist.md`](../research/file-enforcer-rust-rewrite-oxc-browserslist.md)),
and the user replaced both crates:
"Change of plans,
for browserslist,
we retrieve the data and write something conforming to the original browserslist ourselves,
only supporting the queries we use now,
throw on everything non-conforming with \"not implemented\"."
The decision record carries the change.
The user also said "Don't serialize aggressively paralleling work.",
so implementation areas now run as parallel worktree agents (limit five at once),
sharing the brief `${HOME}/temp/agent/fe-rust/brief-common.md`.

Running:

- XML edit crate (task 26).
- dependency-catalog Rust library (task 21).
- forbidden-strings and git-policy rules list with `?` (task 23).
- pnpr owns `config.yaml` (task 24).
- browserslist subset specification,
  writing `doc/research/browserslist-subset-semantics.md` (docs only).

Queued for the next free agent slot,
with briefs saved at `${HOME}/temp/agent/fe-rust/brief-task-20.md` and `brief-task-22.md`:

- file-enforcer `core-rust` and `cli` (task 20),
  with browserslist,
  dependency catalog,
  and JetBrains XML behind rule-kind seams.
- mpm-shadow (task 22),
  with the npm and PyPI registry lookups behind a seam until an HTTP client is chosen.

Waiting on the user:
the browserslist data source,
the Firefox ESR source,
the HTTP client crate,
and the JSON Schema validator crate for the schema golden test.
After those:
the in-repo browserslist crate (task 28),
then the cutover (task 25).

## Superseded next action:
look at oxc-browserslist

The user said the next step is to look at `oxc-browserslist`,
before building on `browserslist-rs`.
Why it matters:
the gate probe measured `browserslist-rs` 0.21.2 on the repository's `.browserslistrc` against npm `browserslist`
and found older data
(14 targets against 18,
`chrome151` against `chrome156`,
a hard-coded `firefox140` ESR against `firefox153`,
Node up to 26.8.0 against 26.12.0).

What is already known,
from
[`doc/audit/tech-file-enforcer-browserslist-engine-vet-2026-10-29.md`](../audit/tech-file-enforcer-browserslist-engine-vet-2026-10-29.md)
(sections "oxc-browserslist 5.0.1" and "E7" to "E9"):

- `oxc-browserslist` 5.0.1 (2026-08-12,
  MIT) is the oxc project's fork of `browserslist-rs`,
  passed screening,
  and its published crate matches its tag byte for byte.
- It has no config-file loading,
  no `BROWSERSLIST*` environment variables,
  and no `extends`,
  so file-enforcer parses `.browserslistrc` itself either way.
- Its data inputs at 5.0.1 (`caniuse-lite 1.0.30001809`,
  `browserslist 4.28.8`) are older than `browserslist-rs` 0.21.2's,
  so the published crate alone does not close the data gap;
  its `cargo codegen` data regeneration has not been run.
- Unread:
  how it resolves `Firefox ESR`,
  `not dead`,
  and the `ChromeAndroid` aliases.
- Single crates.io owner (`Boshen`).

The user picked this candidate and it is already in a repository vet report,
so this is not broad vetting,
and the check-in that `.agents/skills/choosing-technology/SKILL.md` requires before enumerating candidates does not apply.
If looking at it means searching for candidates beyond memory,
the user's picks,
and repository precedent,
it becomes broad:
ask the user who runs that vet first.
The probe scratch for `browserslist-rs` is under `${HOME}/temp/agent/fe-rust/gates/`,
and an oxc clone is at `${HOME}/temp/agent/oxc-browserslist-2026-10-29`.

## After that

Start the implementation areas tracked in issue #1818:

1.  `package/file-enforcer/core-rust` and `package/file-enforcer/cli`.
2.  The dependency-catalog Rust library.
3.  mpm-shadow:
    `sync`,
    `status`,
    `update [--check]`,
    `lock`,
    `lock --check`,
    `bin-paths`,
    `--global`,
    the lock digest,
    and bootstrap.
4.  forbidden-strings and git-policy:
    the comma-separated rules list with a trailing `?` for optional entries.
5.  pnpr generates and checks its own `config.yaml`.
6.  The in-repo XML edit crate.
7.  Conversions and the one-switch cutover.

Open facts for implementation are listed in the decision record's "Consequences":
`schemars` derive without serde `Deserialize` is unproven,
the golden test's JSON Schema validator crate is unnamed,
and tag `v8.2.0` of mpm resolves to `99eb596b` while `doc/decision/mpm-toolchain-adoption.md` cites `f095931a`.

## Lesson recorded from this work

The grilling settled the design first and left its dependencies as vet gates after acceptance;
the user then approved them without vets.
The user asked that grilling enumerate and pick every dependency first,
now `GXD` in `AGENTS.md`,
and that each broad vet check in with the user before enumerating candidates,
now "Check in before broad discovery" in `.agents/skills/choosing-technology/SKILL.md`.

## How to respond

Reply in the session that resumes this work;
it reads this file,
then the decision record,
then issue #1818.
