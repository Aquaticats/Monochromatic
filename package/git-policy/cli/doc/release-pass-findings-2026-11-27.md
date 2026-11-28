# What a release pass of the wrapper found

Rounds of 2026-11-27 and 2026-11-28,
unapproved and non-final.

Two passes read the release `0.6.1-alpha.48`:
a behaviour pass,
which exercised alpha.47 and alpha.48 side by side,
and a splits pass,
which read what the 344 refactor commits of the range did.
This record keeps the findings and the measurements that no other file holds,
in the passes' own words where they had them.
Every figure is the pass's own measurement,
taken on 2026-11-27;
nothing here was re-measured for that record.

A later second pass read `0.6.1-alpha.52` against `0.6.1-alpha.51` on 2026-11-28,
and what a round did about its findings is
"The alpha.52 behaviour pass" at the end of this file.
That pass was another agent of the same vendor,
so still not an independent review,
and the round that acted on it re-ran each finding before changing anything.

The behaviour the release rests on is the code,
the gate,
and the service-manager install proof named under
"No change of behaviour is not demonstrated" below.

## The behaviour pass found no High finding

Neither pass raised a High finding.
The behaviour pass's two Mediums and six Lows were all fixed by rounds merged on 2026-11-27,
each with a control that fails without the fix;
what each said is history,
and the controls pin the fixed words.

The splits pass's Mediums and Lows are below.

## The behaviour pass: what held

These held,
exercised with both binaries:

- the slot refusals,
  for the spellings they name;
- the merge pass's union of deletions;
- the self-check's memory,
  62.7 to 66.1 MB in alpha.47 and 9.3 to 11.0 MB in alpha.48;
- named pipes,
  answered and not waited on;
- the typed ssh relay at a terminal,
  which ended about 100 ms sooner.

## The behaviour pass: read and not run

Three surfaces were read only,
because nothing outside the process can start them:
the prune race,
the conversion environment's precedence,
and the service-only changes.

## The splits pass: no change of behaviour is not demonstrated

Its verdict is that "no change of behaviour" is not demonstrated.
Of the items compared at both ends,
97.9 percent have identical text,
no test-only switch is in either release binary,
no static became two and no trait impl was lost;
but 318 of the 344 refactor commits carry changed lines that are not moves,
and the pass read three of them in full.

Neither pass ran `git commit` in a release binary,
because without a session bus every commit form stops at one refusal.
So for the service paths the evidence is the reading,
the gate,
and the service-manager install proof,
which issue 6587's second round ran to its end (`fdd9ce0e83`):
alpha.48 installed over alpha.47 in a container of the gate image,
and the proof printed

```text
THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once
```

## The splits pass: reach

At least 31 definitions became nameable from outside the crate,
seven of them private helpers a split made bare `pub`,
one of which removes directories.
The list of published names is
`src/native/source_guard/items_nameable_from_outside_release.txt`,
and its control is
`src/native/source_guard/nothing_becomes_nameable_from_outside_tests.rs`,
which fails on a published name that has no line there.

## The splits pass: refactor commits read and not read

The extract-function refactors are unread but for ten commits.
A second read of the seven the pass named first,
all on the client's and the service's commit paths,
found each equivalent by reading
(`e90482f100`,
`dde5c4be3b`,
`bad45d0420`,
`01c2f32e2c`,
`6a24c6342b`,
`f4e8d2fb9b`,
`8af8f56255`).
One guard covers less than its name says:
`src/native/source_guard/hook_self_invocation_tests.rs` reads 263 of the 699 code lines
the commit preparation spans,
which the list-guards round took first.
About three hundred refactor commits stay unread.

## The splits pass: guards that name moved files

The pass found 71 pairs of a source guard and a file it names,
where code left the file in a split and the guard does not name where it went.
Two were checked,
and neither was a hole.
The list-guards round holds the pairs now.

## The splits pass: low findings

Eight points,
as the pass put them:

- the sweep's warnings carry four new log tags;
- a commit labelled `docs` changed the `shutdown-unmount` help row
  (`066e33a47d`,
  noted on the commit);
- the panic notice's source locations moved;
- 34 items lost a platform gate on split-off files,
  with no effect on x86_64 Linux;
- tag constants are copied into more files;
- the README's allow-list counts are stale;
- two commits in the range cannot build alone;
- the linter the binary carries moved four alphas.

The platform gates are fixed:
a split-off file keeps the platform gate of the file it left (`4d6f09f1f2`).

## The splits pass: a split that was not a pure move

Not every split is a pure move.
The landing split's last commit (`3b9ada357b`) boxed the commit inside the service's answers
and made the commit command pass its own copy of its client configuration,
both for Clippy after the move,
which the release pass that reads the range reads as a change.

## The alpha.52 behaviour pass

A second pass read `0.6.1-alpha.52` (`bf46e60f6d`) against `0.6.1-alpha.51` (`81b06993bd`) on 2026-11-28,
over 230 rows of spellings,
each row on four arms:
alpha.51,
alpha.52,
a byte-identical copy of alpha.52,
and real Git 2.56.1 as the ground truth.
It was another agent of the same vendor,
so a second pass and not an independent review.
Its scratch folder expires,
so the evidence each finding rested on is restated here,
and every fixture the round re-ran is described in full so a reader can rebuild it.

The round that acted on it is this worktree's branch,
whose commits are `ef2206f5eb`,
`503c681ea9` and
`c87f309101`.
Nothing in this section is approved or final.

### The fixture every row below was measured in

Real Git only,
`/usr/bin/git`,
never the `git` on this machine's `PATH`:

- one repository `main`
  whose commit of `a.txt` holds `a1\n`;
- `main` dirty:
  `a.txt` edited to `a1 edited\n`,
  `staged.txt` staged with `s1\n`,
  and `u.txt` untracked;
- linked worktrees `wt1` (clean tracked content),
  `wt2` (dirty,
  with an unstaged edit and a staged `s2.txt`)
  and
  `wt3` (completely clean);
- `g = !git` in the fixture's global configuration.

Every wrapper run had `env --unset=DBUS_SESSION_BUS_ADDRESS`,
`XDG_RUNTIME_DIR` a short folder of the run's own,
`HOME` and the four `XDG_*_HOME` inside the fixture,
`GIT_CONFIG_NOSYSTEM=1` and a fixture `GIT_CONFIG_GLOBAL`.

### M1: a `core.worktree` main checkout is judged as plain, and the alias resets it

The read's words:

> With `core.worktree` set to the main checkout in its own configuration (the main checkout is dirty),
> typed in `main/sub`:
> `git g -C <wt3> reset --hard`
> (also `clean --force`,
> `checkout -- a.txt`,
> `stash`,
> and the same through `git -c alias.cc=!git cc -C <wt3> reset --hard`)
> changes `main` files and index in real Git,
> and alpha.52 runs it and loses the staged `s.txt` and the edit of `a.txt`.

Reproduced on 2026-11-28,
with one change of verdict for the round:
the reading was fixed,
so the same rows are refused now.
The fixture the round used has `core.worktree = <main>` in `main/.git/config`,
`main` dirty as above and `wt3` clean.
Real Git typed in `main/sub` answers:
`rev-parse --git-dir` = `<main>/.git` (absolute),
`--absolute-git-dir` the same,
`--show-toplevel` = `<main>`,
and the fixture's own `!printf %s "$GIT_DIR"` alias prints `<main>/.git`.
Both binaries ran `g -C <wt3> reset --hard` with exit 0,
no standard error,
and afterwards `main` held no staged or unstaged work at all:
`git status --porcelain` was empty,
`a.txt` was back to `a1\n`,
and `staged.txt` was gone.
The controls in the same fixture:
the direct spelling `git -C <wt3> reset --hard` typed in `main/sub` changed nothing,
and the same effect spelled directly,
`git -C <main> reset --hard` from `wt3`,
ran and lost the same work,
which is what real Git would do.
Without `core.worktree` the same alias words change nothing.

What Git does,
from its own source at the tag this machine runs,
Git 2.56.1 (`setup.c`):

- discovery takes the explicit path when the repository's own configuration names a work tree:
  `repo_discover_implicit_gitdir` at `setup.c:1247` to `:1254` reads
  `if (getenv(GIT_WORK_TREE_ENVIRONMENT) || discovery->format.work_tree)`;
- that path stores the Git directory it found,
  and `setup_git_directory_gently` exports whatever it stored:
  `setup.c:2063` to `:2071`,
  through `apply_and_export_relative_gitdir`,
  which calls `xsetenv(GIT_DIR_ENVIRONMENT, path, 1)` at `setup.c:1102`;
- only a plain main checkout whose `.git` is the directory Git found reaches that block with nothing stored,
  which is the one shape where the nested Git finds the repository by itself;
- `core.worktree` is read from the repository's `config` (`setup.c:585` to `:597`)
  and from `config.worktree` when the extension is on (`setup.c:786` to `:793`),
  and a value in a shared configuration is ignored when the Git directory holds a `commondir`
  (`setup.c:805` to `:812`).

The reading the release shipped compared `<top>/.git` with `--absolute-git-dir`,
which with `core.worktree` set to the checkout are the same directory,
so it answered that Git gave the alias's shell nothing and passed no `--git-dir`.

What changed:
`external_git_command_the_git_dir_the_shell_reads.rs` asks Git
`git config --show-scope --get-all core.worktree`,
and counts only a value the `local` or `worktree` scope set,
because discovery reads the repository's own files and never a `-c` on the command line.
The question is asked only where the comparison has already read the shape as the plain `.git` at the top,
so the ordinary case pays nothing,
and a main checkout pays one more Git process per `!` alias whose own words name a place.
A read that fails answers nothing to add:
the shape it asks about is the one the comparison has already read,
and pinning a repository on an answer nobody read would judge a place the command may not act in.

Controls:
`external_git_command_tests.rs`'s `the_git_dir_the_alias_shell_reads_is_the_one_git_gives_it`
now holds a main checkout whose configuration names its own work tree,
at its top and in a subdirectory,
beside the seven places it held;
`entry_shell_alias_judged_where_its_words_point_tests.rs`'s
`a_checkout_naming_its_own_work_tree_is_the_state_the_shell_has`
holds the alias spelling of every guarded form to the direct spelling of the state the nested Git acts in.
With the new clause taken out,
the first control failed with
`left: false` against `right: true` at the checkout's own top,
and the second failed at its first response.

The citations these files carried before the round,
`setup.c:1126` to `:1129` and
`setup.c:1657` to `:1662`,
name other code in Git 2.56.1;
the round replaced them with the lines above in every file it touched.

### The three shapes the read did not reach

The read named three shapes as unchecked.
Each was measured with real Git 2.56.1 on 2026-11-28,
in a repository of its own,
with the same `printgitdir` alias and a row through alpha.52:

- `.git` a symbolic link to a directory.
  At the top the alias shell is given nothing,
  `--git-dir` prints `.git`,
  and `--absolute-git-dir` prints `<top>/gitdir-real`;
  in `sub/` the shell is still given nothing.
  The reading canonicalizes both sides of its comparison,
  so it answers "none" as Git does,
  and the row `g -C <clean linked worktree> reset --hard` typed in `sub/` changed nothing.
- `extensions.worktreeConfig` with `core.worktree` in `config.worktree`.
  The alias shell is given `.git` at the top and `<repo>/.git` in `sub/`,
  and `git config --show-scope --get-all core.worktree` prints `worktree\t<repo>`.
  The reading as shipped answered "none",
  and the row `g -C <clean linked worktree> reset --hard` typed in `sub/` reset the dirty checkout and lost
  its work,
  which is M1's loss by another route.
  The question added for M1 accepts that scope,
  so this shape is covered by the same change;
  no control builds it,
  so the coverage rests on the probe above and on the scope the reading accepts.
- a worktree path that holds a newline.
  `rev-parse --is-bare-repository --git-dir --absolute-git-dir --show-toplevel` then prints more lines
  than it has answers,
  and the reading takes the second and third of them:
  at the top of a main checkout at `<root>/new\nline-main` it read `.git` as the Git directory,
  `<root>/new` as the absolute one and `line-main/.git` as the top,
  and in `sub/` it read the truncated `<root>/new` as the Git directory itself.
  Both rows,
  `g -C <clean linked worktree> reset --hard` typed at the top and in `sub/`,
  were refused,
  with the outside-a-worktree refusal,
  and nothing was lost.
  `worktree_identity.rs` refuses such a path outright
  ("Repository paths containing line feeds are not supported"),
  so the shape is named and not supported,
  and the alias reading answers in the safe direction at the cost of a false refusal.

### The `git` inside an unread alias's shell, on an installed system

The read's L1 showed that an alias the reading leaves unread runs with no guard,
and its fixtures put the real Git behind the alias's `git`.
What that `git` is on an installed system was measured on 2026-11-28,
with the wrapper first on the fixture's `PATH` under the name `git`:

- here,
  `command -v git` inside the alias's shell prints `/usr/lib/git-core/git`,
  `readlink -f` of it prints `/usr/bin/git`,
  and the first `PATH` entry is `/usr/lib/git-core`:
  Git puts its own exec path in front of `PATH`,
  and this distribution's exec path holds a `git` that is the real Git,
  so the nested command is unguarded and the read's loss is real.
  Measured:
  `!git -C <dirty main> reset --hard` typed in the clean linked worktree ran with exit 0,
  and the main checkout's unstaged edit was gone;
  its staged entry survived,
  because the nested Git acted with the clean worktree's index.
- a system whose exec path holds no `git` leaves `PATH` to answer,
  and an installed wrapper is then that shell's `git`.
  Measured with `GIT_EXEC_PATH` naming an empty directory:
  `command -v git` printed the wrapper's link,
  and running it from the alias's shell failed with exit 126 and
  `<fixture>/bin/git: Permission denied`
  at 2026-11-28T03:07-05:00.
  This machine's AppArmor profile `git` allows executing `@{bin}/git`,
  `@{lib}/git*` and a few named programs,
  and nothing else,
  so the wrapper cannot be that shell's `git` here at all.
- what a nested wrapper judges
  when it is the one that answers,
  measured by handing it the alias shell's environment:
  the wrapper's own directory as the working directory and `GIT_DIR` the Git directory Git exported.
  It refused,
  with the linked-worktree-only finding,
  and nothing was lost;
  the same command with no `GIT_DIR` reaches the facts as the main worktree and is refused there.

The README and the SPEC now say both arms where they state the boundary,
and the boundary itself is unchanged:
whether an unread alias is refused is the owner's to decide,
and this round changed nothing about which aliases are read.

### L2: the refusal named no place

The read's words:

> the refusal text is "this worktree has uncommitted changes to tracked files" with no worktree named,
> though the dirty state is wt2's index over wt1's files.

Reproduced with real Git:
with one worktree's Git directory and another worktree's files,
`git -C <wt1> status --porcelain=v1 --untracked-files=no` answered `AD s2.txt`,
and `git -C <wt1> reset --hard` in that state dropped the staged entry of the index it kept,
so the state is dirty and the two places are real.
`rev-parse --show-toplevel` in that state prints the `-C` directory and
`--git-dir` the Git directory the shell kept,
which is why the identity the guard reads carries two places.

What changed:
`dirty_worktree_message` takes the two places and whether they are one worktree
and lives in `command/rule_linked_worktree_dirty_refusal.rs`,
the two places being read in `policy/policy_linked_worktree_the_places_it_judged.rs`,
because both files that held them would otherwise pass the linter's line limit,
and the text names the Git directory whose index and `HEAD` the form rewrites and the worktree whose files
it writes.
When they are one worktree the text says so and names that worktree;
when they are not,
it says the place is two,
names both,
and says the work that would be lost is that
index's staged entries and those files' own edits.
The function became crate-private with that signature,
and its line left
`src/native/source_guard/items_nameable_from_outside_release.txt`,
which records one name leaving and none joining.

Controls:
`rule_linked_worktree_tests.rs`'s `a_dirty_refusal_names_each_place_it_judged` holds the two shapes,
and `entry_shell_alias_judged_where_its_words_point_tests.rs`'s
`a_refusal_of_a_state_that_is_two_worktrees_names_both_of_them` builds the real two-worktree state and
asserts the refusal carries both paths.
With the places taken out of the text,
both failed;
`a_dirty_refusal_names_each_place_it_judged` failed on the path the text no longer held and the entry
control on both paths.

What the round left:
the main-worktree refusal,
`main git worktree`,
still names no path.
It is the text M1's shape gets when the state is the main checkout itself,
and naming its place is a wording choice outside L2's words,
so it is open for the owner with the cost that a person refused there must know which checkout is meant.

The three refusals issue 6641's closing comment records as open for the owner are unchanged,
and their costs are:

- `g -C <wt1> checkout -- <path>`,
  `restore -- <path>` and `checkout HEAD -- <path>` typed in a dirty
  linked worktree are refused where real Git changes nothing.
  The cost is that restoring one path from another worktree is refused,
  and the person has to copy the file or pass `--no-enforce-worktree`.
- An alias whose own `-C` names a worktree
  typed in a Git directory or in a bare repository,
  is refused with the uncommitted-changes finding where alpha.51 let Git print
  "this operation must be run in a work tree".
  The cost is a refusal that names a policy instead of Git's own reason,
  for a command Git would have failed anyway.

### The environment half of the reading

The read's sixth point was that no control gave an alias row `GIT_DIR` or `GIT_WORK_TREE` in the caller's
environment,
so the loop that reads them could be deleted with every test green.

What the round measured:
it changes nothing.
With the loop taken out,
the alias spelling `g -C <clean linked worktree> reset --hard` typed where `GIT_DIR=<the caller's Git directory>`
was in the environment is still refused,
because the facts' own Git inherits that variable and names the same state.
Measured against real Git:
an absolute `GIT_DIR` names that index over the `-C` worktree's files,
a relative `GIT_DIR=.git` is rebased by the command's own `-C` so that it acts in that worktree with no pair
at all,
and a `GIT_WORK_TREE` makes `--show-toplevel` differ from the Git directory,
which the comparison already reads as an export.

What changed:
the loop is gone,
`the_place_was_named` reads only the caller's own `--git-dir` and `--work-tree`,
and the inheritance it rests on is stated beside it,
with the gotcha that a fact request which unset either variable would make this reading name no state where
the nested command has one.
Two controls stay:
`an_alias_row_is_judged_with_the_git_dir_its_callers_environment_names` refuses the alias spelling with
`GIT_DIR` naming the caller's index over a clean worktree's files,
and `the_worktree_a_caller_names_runs_when_no_git_dir_reaches_the_facts` runs the worktree alone without it.
Both stay green without the loop,
which is the fact that took it out.

### What this round did not do

- The final-newline hints (the read's L3 to L6) belong to another round and were not touched,
  including the sentence the read says no run prints and the carve-out the direct-check hint leaves out.
- No whole container gate and no independent review ran.
