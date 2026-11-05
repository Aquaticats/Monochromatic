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

- 2026-11-05: index files in the new layout lint rules (`mod.rs`, and `lib.rs` used as an index) may hold only
  the top doc, `mod` lines and re-exports; anything else is a finding,
  and a name reached through a re-export counts as a use of the file that defines it.
  Reason: your principle "Who depends on who must be immediately visible from looking at the file structure";
  code inside `mod.rs` hides who uses whom.
  Built on branch `feat/monochromatic-lint-index-files-hold-only-declarations`; rules stay off by default.
- 2026-11-05: after the four lint branches land, one more pass removes most closure-lint suppressions
  by giving the called functions a guard or trait form,
  keeping only those that `std::thread::scope` and `catch_unwind` cannot do without,
  and merges the three suppression notes into one note beside the wrapper's code.
  Reason: the repository's lint rules say to restructure rather than suppress.
- 2026-11-05: the one test function inside `testing/no_threads_child_test_support.rs`
  moved into a `*_tests.rs` sidecar, which clears the last `rust/tests-in-sidecars` finding in the wrapper
  (`5983d1603d` on the lint integration branch; the probe and a test that relies on it pass).
