# Provisioning from a program that is not an installed wrapper

Status: a proposal, unapproved and non-final.
Written by the Pi round of issue 6586 on 2026-11-27 (America/New_York).
It changes no release behaviour and no test, and nothing here is to be built until the owner answers
the questions at its end.
The round that wrote it fixed the other half of issue 6586: a test build no longer offers the manager a
program of its own, and reads this session's runtime directory and bus address only from what a control
handed in
(`package/git-policy/cli/src/native/foundation/session_runtime_directory_and_bus_address.rs`,
`package/git-policy/cli/src/native/source_guard/no_release_read_of_the_session_places_tests.rs`).
Whether a *release* should refuse to provision at all, from a program that is not an installed wrapper,
is the question this file is about.

## What the release does today

The release provisions from whichever program is running, and names that program in the link every
worktree's service is started through.

`choose_route` takes the program from the route's inputs, and takes any regular file as that program
(`package/git-policy/cli/src/native/landing/service_route.rs`):

```rust
// The service is a mode of the wrapper's own executable (`service_main.rs`), so the unit names that file; a path that
//  is not a file (a deleted executable) is no program for a unit to name.
let service_binary: PathBuf = match inputs.own_executable {
    Some(executable) if executable.is_file() => executable.to_path_buf(),
```

The running program is remembered from the process itself, and the crate's own note says what it is for
(`service_route.rs`):

```rust
///  Identification only:
/// nothing is started from this path (`doc/planning/cli-git-no-self-invocation.md`);
/// the path is
///  read to name this executable in a unit file that the operating system's manager runs,
/// as the program of its service mode.
```

Nothing checks where that file came from.
`process_resolution_inputs` fills it from `std::env::current_exe()`
(`package/git-policy/cli/src/native/invocation/real_git.rs`),
so a debug build, a release build made in a worktree, a test binary and the installed release are one
and the same fact to the code that follows.

Provisioning then points the account-wide link at it before it touches the manager
(`package/git-policy/cli/src/native/service/service_provision.rs`):

```rust
// The manager starts whatever the link names, so it names this release before any unit can be started.
point_program_link(host);
```

and the unit text names the link rather than any executable
(`package/git-policy/cli/src/native/service/service_provision_the_unit_texts.rs`):
`ExecStart=@SERVICE_BINARY@ monochromatic-git-policy-cli service run ...`, where the placeholder is
filled with `program_link(host.runtime_dir)`.
The reason to point it at all is stated on `point_program_link`
(`package/git-policy/cli/src/native/service/service_provision_the_links_the_units_run.rs`):
"the commit service is the only writer of a commit, this release will not hand one to another release's
service, and the manager reads the link from the text it holds, which this release cannot change without
writing that text."

What that costs when the running program is not a real wrapper is measured, not imagined.
On 2026-11-27 a threaded test run of this crate provisioned with the test binary as the program:
the user journal holds 100 starts that ended with
`monochromatic-git-policy-cli-service[<pid>]: error: Unrecognized option: 'journal'` between 07:31:27 and
08:01:43, five starts each for two fixture worktrees until their units reached `Result=start-limit-hit`,
and the link `/run/user/1000/monochromatic-git-policy-cli-bin` named the test binary until another
session's command pointed it back (issue 6582's round,
its `keep/red-wide-run-captured-notices.txt` and `keep/notes-the-red-run-left/`;
issue 6586).

The same account's link names a program that is not an install today: on 2026-11-27T08:36:23-05:00 it was
`/home/user/Monochromatic/.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli`,
a build made from `main` and kept by hand.
So any rule about "an installed wrapper" also decides whether this machine may keep committing from a
build, which is the practice every round of this worktrees' development depends on.

## Option A: keep provisioning from whatever program runs, and refuse nothing

The release behaves as it does today: `choose_route` takes any regular file, the link is pointed at it,
and the manager starts it.

Example: an agent builds `target/release/monochromatic-git-policy-cli` in a worktree, installs it as `git`
in that worktree's tool folder, and commits.
The link moves to that build, the worktree's service runs it, and the commit lands through it.

Pros:

- This machine's practice and every development install keep working, so no new refusal, message or
  setting is needed.
- A temporary program never outlives its use by accident: the next real command points the link again,
  which is what happened within seconds of the measured failure.
- The one class of risk the round could remove is already removed: a test process can no longer offer a
  program, because the executable a test build remembers is never handed to the manager.

Cons:

- Any other program that reaches the route, such as a debug build or a binary built by another tool, still
  arms the account's manager and can leave a worktree's unit failed until the next command repairs it.
- Nothing in a person's output says which program the link now names, so the failure is found in the
  journal rather than where it happened.

## Option B: refuse unless the program is the account's installed wrapper

Before anything is written, the release compares its own path with the path the account's install names,
and stops with a message when they differ.

Example: the installed release is `~/.cargo/bin/monochromatic-git-policy-cli`.
A commit from `target/debug/monochromatic-git-policy-cli` stops with "this program is not the installed
wrapper; install it with `cargo install`, and commit again", and nothing is written; a commit from the
installed path provisions as it does today.

Pros:

- The manager only ever runs a program the account installed on purpose, so the measured class of failure
  cannot reach the manager from a build at all.
- The refusal is cheap to reach: it is one comparison and one message, before any write.

Cons:

- It refuses the practice this machine uses now, where the link names a build from `main`
  (`/home/user/Monochromatic/.cache/wrapper-built-by-cargo-from-main-…`), so every commit from a build
  would stop until the install is the program being run.
- It needs a decision the code does not have: which path is "the install" (cargo's, mise's, whatever
  `PATH` finds, or one the person states), and what a machine with two installs answers.
- It also refuses the build the gate's own binary tests make, unless those name their build as an install.

## Option C: refuse unless the build was made as a release

The manifest sets a compile-time marker that only the release workflow sets, and a build without it
refuses to provision.

Example: `cargo build --release` in a worktree produces a program with no marker, so its commit stops
with the same refusal; the archive published by the release workflow carries the marker and provisions.

Pros:

- A test binary, a debug build and any local build are refused by construction, so no path comparison and
  no new setting are needed.
- The refusal is a compile-time fact, so no `Environment=` line or variable can turn it off.

Cons:

- It refuses this machine's own practice just as option B does: a release build kept under `.cache` is
  still not the release workflow's archive.
- It splits a program's behaviour by how it was built, which is the one thing
  `doc/decision/cli-git-test-only-switches-compiled-out-of-release.md` already does for test-only
  switches, and doing it a second time makes "what a build can do" harder to read.
- Every release step, the archive's own controls and any install by hand have to carry the marker, or a
  program people already run stops provisioning after an upgrade.

## Option D: provision as today, and say which program the link now names

The release keeps option A's behaviour and prints one line naming the program it wrote into the link,
saying so when that path is not the account's install.

Example: a commit from a worktree build prints
`monochromatic-git-policy-cli: the program link now names /…/target/release/monochromatic-git-policy-cli,
which is not the installed wrapper`, and lands as it does today.

Pros:

- Nothing new can stop a commit, so no practice and no test breaks.
- The measured failure would have been visible where it happened instead of in the journal, and the line
  costs one message.

Cons:

- A test process captures its own output, so the line that would have helped a person goes to a buffer
  nobody reads, and the manager is still armed by the time the line exists.
- The line says what happened after the link moved, so it warns rather than prevents.

## Ranking

Ranking: A > D > B > C.

- A above D: both keep every commit working, and A is the smaller promise, because the line D prints is
  read after the link has moved and is often not read at all, while A changes nothing that is not already
  measured and repaired by the next command.
- D above B: D keeps the development practice of this machine, which B refuses, and B's promise needs a
  fact the code does not have yet (which path is the install), so B buys prevention with a decision and a
  broken practice.
- B above C: both refuse the same commits, and B can be told to accept a build by naming it, while C's
  compile-time marker cannot be overridden at all, so C is the stricter rule with the same cost.

## Questions for the owner

1. Should a build made in a worktree keep provisioning the account's manager, as the link on this machine
   does today, or must the manager only ever run an installed release?
2. If a refusal is wanted, which program counts as installed: the one `cargo install` or mise put on
   `PATH`, the one the link already names, or a path a person states in a setting?
3. Should a line be printed whenever the link is pointed at a program outside the install, whatever else
   is decided?
4. Where should a refusal stop: before the link is pointed (nothing written, the old program still
   serving) or before the manager is asked (units written, link moved)?
5. Is one account-wide link per runtime directory still the right shape, given that two worktrees of two
   different builds share it?
