# Questions for the owner

Open questions from the coordinator session, newest first.
This file is open in Zed, which reloads it when it changes on disk and the tab has no unsaved edits.
Write your answer under a question and save;
the session re-reads this file before every update, so a saved answer is never overwritten.
An answered question moves to the canonical doc it belongs to and leaves this file.

For this session the owner said:
"Please make any non-destructive options (when they are the ones I would likely have picked) yourself from now on for
this session."
So this file holds only choices that are destructive, reach outside this machine, or where the likely pick is unclear.
The non-destructive choices the session made itself are listed at the end, so any of them can be undone.

## The wrapper's target layout, before anything moves (asked 2026-11-05T02:28:53Z)

You asked to see the whole tree before any file moves. It is drafted, nothing has moved:
`doc/planning/cli-git-source-layout-target-tree.md` (start at its section "What to look at and how to answer",
then "The crate split" and "Open points for the owner"),
with every file's old path, new name and one-line job in `doc/planning/cli-git-source-layout-target-tree-files.md`.
A reviewer from another vendor read it and its findings are folded in.

In short:

- The wrapper becomes one library crate per job found, each its own package `package/git-policy/<name>`,
  with no loops between crates, the command-line program on top.
  As drafted that is 46 libraries (for example `safe-file-saving`, `read-git-files-without-git`,
  `push-with-a-deadline`, `land-prepared-commits`), a crate of test fixtures, and the program.
- Inside each crate, folders named as plain sentences with `uses/` links; 276 of 580 files get clearer names.
- 105 items move and 7 calls change before files move, which takes the 17 loops between files down to none.

What I need from you:

- Is the tree right? Say which crates or names are wrong, or "go".
- How far to split:
  - The 46 libraries as the target, built in-package first, then lifted into packages one at a time from the
    bottom, asking you again after the lower half (recommended).
    Pros: every crate is one reusable job, as you asked; the lift can stop early if the cost shows.
    Cons: many packages to keep in step; each bump of a lower crate edits every crate above it.
  - Fewer, bigger crates (the proposal names its weakest entries, which would merge back first).
    Pros: fewer packages. Cons: crates that hold more than one job, which you rejected before.
  Ranking: the 46 as a target > fewer now, because you said splitting must not mean fewer crates,
  and lifting one at a time keeps every step reversible.
- One shared build folder for all of these packages (`CARGO_TARGET_DIR` set in their `mise.toml`)?
  Without it, each of the ~48 packages compiles its dependencies again in its own build folder.
  Recommended: yes. Asked because build folders are a setting you have strong rules about.
- The program's package folder `package/git-policy/cli`: keep the name `cli`, or rename it so it explains itself
  (for example `package/git-policy/git-command-wrapper-program`)?
  Recommended: rename, since a package folder name still has to make sense alone, the reviewer agrees,
  and the release archive and the lock row can follow in the same release as the planned program rename.

Your answer:

## Which of our crates still go to a registry, and which one (asked 2026-11-05T00:56:04Z)

The research you asked for is done: `doc/research/cargo-crates-and-tools-without-crates-io.md`.
In short:

- Your first pick, a `workspace:`-like way, works.
  Cargo's form of it is `path` plus an exact version, which our crates already use.
  Our tools can then install from their GitHub releases, the way `mpm-shadow` already does, with no crates.io at all.
  So the session chose it for the tools (see the list at the end).
- Floating git does not work for us: Cargo can link a crate by git,
  but our catalog refuses moving git links,
  and no installer here can fetch a ready-built tool from git without compiling it, which the repository forbids.
- "GitHub package registry" as GitHub Packages has no support for Rust crates at all.
  What does work on GitHub is a plain index of files published to GitHub Pages, with the crate files kept as release
  downloads; tools such as `margo` do this. That is the reading used below; say so if you meant something else.
- One rule binds every option: a crate on crates.io may depend only on crates that are also on crates.io.

So the tools leave crates.io after their one release each.
What remains open is where the libraries go
(`monochromatic-jsonc-edit`, `monochromatic-xml-edit`, `monochromatic-catalog-specifier`,
`monochromatic-dependency-catalog`, `monochromatic-file-enforcer`, `monochromatic-browserslist`):

- Only `forbidden-strings` and `forbidden-regex` stay on crates.io; every other crate stops publishing after its one
  release, and outside use of the others waits for the pnpr registry you chose earlier
  (`doc/planning/self-hosted-cargo-registry.md`) (recommended).
  Pros: from then on crates.io gets only the two crates you exempted; no extra rules to keep.
  Cons: until pnpr serves crates, someone outside can use our other libraries only through git or a local path.
- The libraries also stay on crates.io, released less often.
  Pros: anyone can add them from crates.io.
  Cons: more crates.io versions; each one must keep every dependency on crates.io; nobody outside uses them today.
- The libraries go to a file index on GitHub Pages instead.
  Pros: no crates.io; versions, checksums and yanks still work.
  Cons: new code in `mpm-shadow`, the dependency catalog and `cargo-shadow`, the same work the pnpr registry needs;
  it would be a second home next to the pnpr one you chose.

Ranking: only the two exempted crates > libraries on crates.io > GitHub Pages index.
The first beats the second because no one outside depends on any of these libraries
and their README says not to use them, so their crates.io versions add to crates.io with no user.
The second beats the third because the third repeats the pnpr registry's work in a second place.

Your answer:

## When the one release of each crate happens (asked 2026-11-05T00:43:32Z)

You decided "One release each. And we publish less to crates.io".
A crates.io publish cannot be undone; a yank only hides a version.
Two facts set the timing:

- The wrapper (`monochromatic-git-policy-cli`) on `main` is in the middle of the lint clean-up:
  three of the four lint branches are merged together but not yet tested in a container, and the fourth is still running.
- The planned rename of the wrapper's program and command word
  (`doc/planning/git-policy-cli-program-rename.md`) needs a wrapper release of its own,
  and it starts as soon as the lint branches are merged.
- Our tools pin our libraries to exact versions,
  so a new library version means a new version of every tool that pins it, in the same batch.

Options:

- After the lint container run and the rename, in one batch (recommended).
  Each crate gets one new version carrying the README notice, the lint clean-up and the renamed program.
  Cons: the notice shows on crates.io a day or so later.
- Right after the lint container run, in one batch.
  Pros: the notice shows sooner.
  Cons: the rename then needs a second wrapper version, against "publish less".
- Now, from `main` as it is.
  Pros: the notice shows today.
  Cons: ships the wrapper's untested half-done clean-up, and the lint merge then needs another release.

Ranking: after the rename > after the lint run > now.
After the rename beats after the lint run because it saves a second wrapper version for a delay of about a day.
After the lint run beats now because now ships untested code and still needs another release.

In each case the older version of each crate is yanked right after its new one is published, as you asked.

Your answer:

## Add this way of asking to AGENTS.md (asked 2026-11-05T00:43:32Z)

You wanted questions asked "async": a question file open in an editor that reloads on change,
updated as questions come up, in place of the question tool.
`AGENTS.md` now says the opposite (UQT: "put it in the question tool").
Proposed new rule (unused code, checked against both forbidden-strings lists before adding):

```md
QFL:
 When the owner asks for questions async,
 keep them in `doc/handover/questions-for-the-owner.md`, opened in Zed (it reloads unchanged files),
 and update it as questions arise, instead of the question tool.
```

Options:

- Add it as written (recommended).
- Add it with changes you write here.
- Do not add it; this stays a one-session arrangement.

Ranking: as written > with changes > not at all,
because without a rule the next session falls back to the question tool,
and the wording above already names the file and the editor.

Your answer:

## Choices the session made itself (non-destructive; say "undo" next to any of them)

- 2026-11-05, target layout: the proposal's technical points are taken as drafted:
  the failure codes stay with the commit service's messages;
  the crash points for tests and the durable index-lock code get their own crates;
  the source-text guard tests move to the program crate;
  sidecar tests move with their code and no new tests are written for the move;
  the two extra call changes (the copy error type, the hint functions moving with their users);
  the `include` list goes at the split;
  pins are kept in step by the catalog sync;
  names that fail the "monkey" test are renamed while grouping;
  `commit-command`, `doctor-report`, `merge-in-isolation`, `typed-push-planning` fold into the program,
  and `service-after-commit-work` into the commit service program, because no outside program would use them alone;
  `new-worktrees-quickly` is split first after the lower half.
- 2026-11-05: the layout rule will read a folder's file-list marker even when it wraps over two lines,
  because the repository's line-break rule demands a break after its comma; being built on its own branch.

- 2026-11-05: index files in the new layout lint rules (`mod.rs`, and `lib.rs` used as an index) may hold only
  the top doc, `mod` lines and re-exports; anything else is a finding,
  and a name reached through a re-export counts as a use of the file that defines it.
  Reason: your principle "Who depends on who must be immediately visible from looking at the file structure";
  code inside `mod.rs` hides who uses whom.
  Built on branch `feat/monochromatic-lint-index-files-hold-only-declarations` and merged to `main`
  (840 tests pass, 108 layout tests in the container); rules stay off by default.
  On the wrapper it finds no index holding code, and no loop was hidden behind a re-export.
  A `main.rs` that is its folder's index may hold `fn main` and its `use` lines as well;
  `extern crate` and an inline `mod` with a body count as code.
  The same check now also covers a `main.rs` beside a `lib.rs` and single-file `src/bin/*.rs` programs
  (merged to `main`; 842 tests, 110 in the container).
  A repo-wide survey with the file rule on finds 74 entry-point findings in 15 program files outside the wrapper,
  and 107 index-file findings in 11 files; none in the wrapper.
- 2026-11-05: after the four lint branches land, one more pass removes most closure-lint suppressions
  by giving the called functions a guard or trait form,
  keeping only those that `std::thread::scope` and `catch_unwind` cannot do without,
  and merges the three suppression notes into one note beside the wrapper's code.
  Reason: the repository's lint rules say to restructure rather than suppress.
- 2026-11-05: the one test function inside `testing/no_threads_child_test_support.rs`
  moved into a `*_tests.rs` sidecar, which clears the last `rust/tests-in-sidecars` finding in the wrapper
  (`5983d1603d` on the lint integration branch; the probe and a test that relies on it pass).
- 2026-11-05: our seven tools will install from their GitHub releases (the `archive` route `mpm-shadow` uses)
  and stop publishing to crates.io after their one release each.
  Reason: it is your first pick ("a workspace: protocol or something"),
  and the research found it the only route that works both for crates using each other and for installing tools.
  Not started: it waits until the layout and lint work land, as you asked.
- 2026-11-05: the toolchain lock keeps following the newest GitHub release of each tool, as `mpm-shadow`'s row does,
  rather than the version written in the checkout; that needs no new code.
- 2026-11-05: test files stay inside the crates we still publish.
  Reason (`doc/research/tests-inside-published-rust-crates.md`): Debian, Fedora, Gentoo, NixOS and the Rust
  project's crater runs all run `cargo test` from a published crate,
  and leaving our sidecar test files out makes that fail with "file not found for module" (E0583)
  unless the sources change too.
  Once the tools stop publishing, what is left to publish is small
  (about 153 KiB of test code in `forbidden-strings` and 165 KiB in `forbidden-regex`, before compression).
