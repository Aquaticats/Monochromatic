# Figures the survey does not count itself

Each file here holds the command and the whole output of one figure the proposal quotes
that `../output.txt` does not count.
Every figure the proposal takes from the survey itself is a row of `../output.txt`.

Run by the round that reran the survey at commit
`bc5b4ca2bae7fc5bb80475448c31d4d57f51f83c` (2026-11-28T03:19:11-05:00)
unless a file says otherwise.

A record of a commit is `git log` output,
which is its own command and output:

- `96c930201f-the-revert-and-fix-repair.txt`:
  the commit that repaired 771 doc paragraphs in 367 files of `package/git-policy/cli`
  by running the installed fix over the pre-sweep copies.
  Its commit date is 2026-11-27T22:35:41-05:00.
- `81650758c4-what-the-repair-reached.txt`:
  the commit that records what that route reached:
  90 of the 548 one- and two-word doc lines the sweep cut,
  with the 458 that were already alone still standing,
  and 3,367 such lines in the whole package.
  Its commit date is 2026-11-27T22:54:08-05:00.
- `beb61ec46b-the-sweep-merge.txt`:
  the merge that ran the whitespace-only sweep over `package/git-policy/cli`,
  whose result the handover passage below counts as 4,062 findings in 480 files.
  Its commit date is 2026-11-27T18:38:01-05:00.
- `a287f226a0-the-merge-that-brought-the-repair.txt`:
  the merge that brought `96c930201f` and `81650758c4` to `main`,
  after the survey of 2026-11-27T22:56:16-05:00 had recorded its counts.
- `7cfdc9199e-no-rule-reports-an-unterminated-fence.txt`:
  the commit that records,
and adds to the linter's `README.md`,
  that no rule reports a fence that is never closed.

A record copied out of a file,
with the lines the proposal cites:

- `cli-git-rust-implementation-the-4-062-passage.txt`:
  the passage of `doc/handover/cli-git-rust-implementation.md`
  that counts the sweep as 4,062 findings in 480 files,
  lines 2749 and 2750 at the rerun commit.
- `repository-linter-stays-at-zero-the-repository-wide-run-and-the-58-of-498.txt`:
  the repository-wide run at `136a40fe69` with its 174,916 findings,
  and the sentence about the 58 of 498 files whose rendered text changed,
  lines 14 and 16,
and line 186,
at the rerun commit.
- `repository-linter-stays-at-zero-the-hold.txt`:
  the owner's words "Put the semantic line break findings on hold.",
  lines 39 to 43 at the rerun commit,
moved there verbatim on 2026-11-23.

A measurement this round ran with the installed binary:

- `host-width-of-a-joined-doc-line.txt`:
  a joined virtual doc line of 119 characters is a host line of 123,
127 or 131 characters
  at an item indent of 0,
4 or 8.
- `no-rule-bounds-a-prose-line.txt`:
  the 30 shipped rule ids,
none of which bounds a prose line's width,
  and a 123-character prose line passing with exit 0.
- `exit-status-and-fix-at-each-severity.txt`:
  `warn` exits 0 and `error` exits 1 on the same finding,
  and `--fix` applies at both severities.
- `emphasis-boundary-reproduced.txt`:
  the installed fix breaks after `.**` and settles there,
  which is why a line ending inside emphasis is counted apart.
- `reported-paragraph-control.txt`:
  the control the after-the-fix measurement uses,
  which holds one break at no boundary before the fix and none after it,
  measured with the installed binary.

`../findings-left-on-the-fixed-copies.jsonl`
is what the same binary reported on the fixed copies of the run's `222` host files:
`65` findings,
`64` of them in one file
and one in the old control copy that run's harness never passed to the fix.

`../repository-configuration-on-package-git-policy-cli.jsonl`
is the output of

```sh
# from the worktree root
/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-lint --concurrency 2 package/git-policy/cli
```

at the same commit,
which reported 26 findings in 20 files and exited 1:
14 `rust/max-lines` errors,
11 `rust/folder-uses-links` errors,
and one `rust/avoidable-closure` warning.
