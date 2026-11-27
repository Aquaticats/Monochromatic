# Rust files keep the 300-line budget after the vertical reformat

Status: accepted by the owner on 2026-11-26, through the question tool (asked at about 12:55, answered by 16:10 -05:00).
The question was recorded first in `doc/handover/questions-for-the-owner.md`.

## Why it was asked

The repository linter's `rust/max-lines` is 300 code lines, as an error, for every `.rs` file
(`monochromatic-lint.config.jsonc`, the `rust` block; `*_tests.rs` files are exempt).
The root `rustfmt.toml` the owner set on 2026-11-25 puts one struct field, one import and one parameter on each line,
and the git wrapper was formatted with it as `00b6a940b4` (2026-11-26T01:51:40-05:00).
Measured on `main` at `987635373a`
(`~/temp/agent/wrapper-line-budget-after-the-reformat-2026-11-26/keep/code-lines-now-and-before-the-reformat.tsv`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here)):
the wrapper's code lines grew 1.472 times across 1740 files;
281 of its files were over 300, 19 of them over before the reformat and 262 only because of it;
the other crates held 152 files over 300.
For example, `worktree/worktree_remove_hand_over.rs` had 572 code lines before the reformat and 1012 after.

## The answer

"Keep 300, split everything".

So the rule stays as written,
and every file over 300 code lines is split by role into sibling files, as `AGENTS.md`'s MXL says,
in the wrapper and in every other crate.
Not chosen, with the costs the question gave:
"Count without the layout" (a linter change, and a count that differs from what an editor shows),
"Raise the budget to 450" (a number that fits this layout only),
and "Undo the vertical settings" (undoes the format choice and reformats every crate again).

## What follows

The wrapper's splits are round 2 of the crate split's order (`doc/planning/cli-git-source-layout-target-tree.md`, "Next"),
which runs before the loop breaks and the move;
a file whose split would add an allow-list line waits for the loop break that frees it.
The other crates' splits are rounds of their own packages.
