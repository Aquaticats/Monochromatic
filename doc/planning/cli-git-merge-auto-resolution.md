# Automatic resolution of "non-conflicting" merge conflicts in the Git wrapper

Status:
decided and implemented on `feat/git-policy-cli-word-level-second-merge-pass` (see the last section,
"Implemented:
the word-level second merge pass").
The first part of this document is the research that preceded the decisions and is kept as it was written;
"Owner decisions of 2026-11-03" and the sections after "merge-code options measured" record what the owner chose.
Scope:
`package/git-policy/cli` (crate `monochromatic-git-policy-cli`,
licence `LGPL-3.0-or-later` in its `Cargo.toml`),
the wrapper's handling of `git merge` and of the merges it performs itself.
Recorded 2026-11-03T01:13:58-05:00.

## The question and who decides

The owner asked,
on 2026-11-03:
"What is going on with all these conflicts?
Shouldn't our wrapper auto resolve non-conflicting conflicts?"

The coordinator (the agent that handed this research to the document writer) asked the owner to choose between the
approaches in "Candidate approaches".
The owner's answers are in "Owner decisions of 2026-11-03".

The research behind this document is one same-vendor agent's findings (rule IRW).
First-person statements in the fact sections ("I measured",
"I did not read") are that research agent's.
It is not an independent review:
no reviewer from another model vendor has read it.
Where this document corrects the research text,
the correction is stated in "Corrections made while writing this document".

Evidence for the claims lives in the scratch directory `${HOME}/temp/agent/mergiraf-eval/` (scripts `run*.sh`,
raw outputs `out*.txt`,
replayed fixtures under `real/`) and in clones and downloads under `${HOME}/temp/agent/`.
Scratch does not survive a reboot reliably,
and none of it is in the repository.
The two conflicted hunks that carry the central claims are therefore copied into this document,
in
"The two conflicts that prompted the question".

## Owner decisions of 2026-11-03

The coordinator relayed these answers on 2026-11-03;
the option labels are quoted as the owner chose them.

### Merges a person runs

Chosen:
"Word-level second pass (Recommended)".
Meaning as relayed:
after Git stops,
re-merge only the conflicted parts word by word (the method of Jujutsu);
accept a file only when every part resolves;
list each merged part on stderr and in the journal.
This matches the word-level pass in the research agent's suggestion (see "Suggested ranking").

### The wrapper's own unattended merges

Chosen:
"Second pass there too".
Meaning as relayed:
the same pass for auto-push reconciliation with the remote and for commit replay,
logging every part merged,
with the commit.
The research agent's suggestion for this case was different:
keep Git's result only,
or allow a second pass only with a
printed list of the hunks it merged (see "Caveats on the ranking").
The owner's choice follows the logged-second-pass form of that suggestion,
and goes against the keep-Git's-result-only form.
The coordinator describes the offered recommendation as keeping Git's result there;
both positions are recorded.

### Append-only documents

Chosen:
"union for named append-only files".
Meaning as relayed:
a `.gitattributes` line marks chosen files,
for example `doc/handover/*.md`,
as `union`.
The coordinator describes the offered ranking as having no union anywhere.
The research text's own ranking listed union for explicitly named append-only files second,
so the choice agrees
with that list and differs from the offered one.
The facts that bear on this choice are in "The built-in union driver" and "The built-in union driver for append-only
documents";
one of them is a risk to weigh when naming files (see "Open design points").

### Where the merge code comes from (still open)

The question:
port Jujutsu's algorithm on top of `similar` or `imara-diff`,
or link a Jujutsu crate.
The owner said "Re-ask this specific question when the agent returns",
meaning after the source check of
`files::try_merge` that this document contains.
The finding of that check is in "Source check of the Jujutsu merge function";
in short,
the published `jj-core` 0.46.0
does not expose `try_merge`,
the published `jj-lib` 0.46.0 does,
and an unreleased `jj-core` source revision does.
The facts for each branch of the question are in "Jujutsu" and "Source check of the Jujutsu merge function".

### Open design points

These follow from the three answers and from the facts in this document;
none is decided.

- A `.gitattributes` line is read by the wrapper's `merge-tree` calls only when the attribute file is visible to Git
  without a worktree (see "Does git merge-tree honour a merge driver").
  `union` is built in,
  so no `-c merge.<name>.driver` setting is needed,
  only visibility of the attribute.
- The wrapper's unattended merges read objects,
  not a worktree,
  so the attribute source (which tree supplies
  `.gitattributes`) is a design question (see "Limits and unknowns").
- Union keeps both sides silently on a true overlap and loses the blank line between appended sections
  (fixture `md2`);
  whether `doc/handover/*.md` is a set of files where that is acceptable is the owner's call.
- The word-level pass can produce text neither side wrote (`scale(10, 20)`);
  the guard options are in
  "Built-in word-level resolver inside the wrapper".
- Logging:
  stderr,
  the journal,
  and the commit are the places the owner named;
  their exact format is not decided.

## Short answer

### Why Git reports a conflict where the two sides changed different lines

Git's line merge treats two changes that touch (one ends where the other begins) as overlapping.
In `xdiff/xmerge.c` (Git v2.56.0) the two "independent change" tests use a strict `<`:
line 548 `if (xscr1->i1 + xscr1->chg1 < xscr2->i1)` and line 563 `if (xscr2->i1 + xscr2->chg1 < xscr1->i1)`.
A change that ends exactly where the other starts fails both tests and becomes a conflict hunk.
This is why "edit a line,
insert a line right after it" conflicts,
and why "both append at the end" conflicts
(two insertions at one point cannot be ordered by a line merge).
It is Git's documented-by-code behaviour,
not a bug in the wrapper.

### Why the wrapper does not resolve them

The wrapper forwards `git merge` to real Git,
so Git's merge decides.
Its own merges (auto-push reconcile,
commit replay) call `git merge-tree --write-tree`,
which is the same merge engine
(merge-ort,
then `xdl_merge`),
so they conflict on exactly the same shapes.
Nothing in the wrapper currently tries a second,
smarter resolution pass.

### Why Markdown makes it frequent

The Markdown rule MD1 ("Break lines at semantic boundaries") explains the README case:
paragraphs are several lines,
so editing the last line and adding lines after it touches.
The same edits on a one-line paragraph with a blank line between them merge cleanly in Git (measured,
fixture `md1d`).

## The two conflicts that prompted the question


The research agent rebuilt both conflicts from the repository itself:
the `rerere` cache holds the conflict hunks,
and the merge commits
give the three versions and the owner's resolution.
Method:
`git merge-base` of the two parents of each merge,
then `git show` of that base and both parent versions of every
file the merge commit's combined diff lists,
written under `${HOME}/temp/agent/mergiraf-eval/real/` (read-only on the
repository).
Each file was merged with `git merge-tree --write-tree` in a bare clone inside the bounded container
(see "Measured behaviour on the owner's shapes" for the bounds,
here with `IOWeight=10` and `--blkio-weight 10`),
once per tool,
and compared with `diff` against the blob the merge commit holds (the owner's resolution).
Outputs:
`out_real.txt` and `out_realj.txt` in the evidence directory.

Files replayed:
planning document,
README,
`SPEC.md`,
`lib.rs` and `worktree_ladder_finish.rs` for merge `14601b28d3`;
planning document,
`SPEC.md` and `lib.rs` for merge `78928da1f1`.
Plain Git `merge-tree` conflicts in only the two files named in the next two headings (one hunk each);
every other listed file merges cleanly in
Git and equals the owner's committed content.
The `rerere` cache entries from 2026-11-03
(`.git/rr-cache/d6a27937.../preimage` and `.git/rr-cache/4ff65605.../preimage`)
show the same two hunks with markers.

### README hunk

`package/git-policy/cli/README.md` in merge `14601b28d3` (committed 2026-11-03T00:36:57-05:00).
Its parents are `83adc0b2ab` (2026-11-03T00:35:10-05:00) and `ddd1371a1f` (2026-11-03T00:26:14-05:00).

The hunk as `git merge-file --diff3` prints it for the replayed fixtures,
copied from
`${HOME}/temp/agent/m1-diff3.txt` lines 203 to 220 (the long lines are the file's own):

```text
# ${HOME}/temp/agent/m1-diff3.txt lines 203 to 220, from mergiraf-eval/real/m1_readme.{ours,base,theirs}
<<<<<<< m1_readme.ours
   and run records aside in a trash directory under `.claude/` and the commit service empties it afterwards.
  The line reads like `cli-git: this worktree was not made as a snapshot, so it took longer, because the main checkout has
   uncommitted changes in 2 paths (for example notes.md). Commit or stash them, and the next add is a snapshot that
   takes seconds.` It tells uncommitted changes, a running Git command, an unfinished operation, an index that cannot yet
   vouch for its files (`git status` once fixes it), a source that is no subvolume, and a snapshot that started and failed;
   `journalctl --identifier=cli-git` finds the same entry afterwards, with the fields `CLI_GIT_WORKTREE`,
   `CLI_GIT_SLOW_CAUSE` and `CLI_GIT_SLOW_DETAIL`. On a filesystem that is not btrfs it says nothing.
||||||| m1_readme.base
   and run records aside in a trash directory under `.claude/` and the commit service empties it afterwards.
=======
   and run records aside in a trash directory under `.claude/` and the commit service empties it afterwards, deleting
   only the directories the snapshot itself moved there (recorded with their identities) and never following a link.
   It does decline when a registered nested worktree holds a file the main checkout tracks. A trash that cannot be
   emptied is reported with its path and the reason, and its work item stays for the service. Another process that
   replaces or relinks directories of the new worktree while this runs is a risk the person accepts by using the
   command: only the cheap checks (no followed links, identities compared on opening) guard against it.
>>>>>>> m1_readme.theirs
```

Reading of the hunk (re-read from the fixtures for this document):
the side the fixtures call `ours` left the sentence line as it was and inserted lines after it;
the side called `theirs` appended the text `, deleting only the directories ...` to that line
and also inserted lines after it.
Both sides therefore insert at one point,
and one of them also edits the line before that point.
The owner's committed resolution keeps the extended line,
then the lines `theirs` inserted,
then the lines `ours` inserted.

Results (number of lines differing from the owner's resolution):
- Git:
  conflict (16).
- mergiraf:
  conflict (18).
- weave:
  conflict (17).
- jj at line level:
  conflict (18).
- union:
  clean,
  but the stale line is kept (13).
- jj at word level:
  clean,
  **identical to the owner's resolution (0)**.

### Planning-document hunk

`doc/planning/cli-git-main-worktree-ignored-copy.md` in merge `78928da1f1` (committed 2026-11-03T00:20:36-05:00).
Its parents are `752964ff51` (2026-11-03T00:16:50-05:00) and `11d8edaf6a` (2026-11-03T00:15:33-05:00).

Both branches appended a section titled "Round 3" (different content) after "Round 2";
the `theirs` side also added "Round 4".
Marker lines and headings of the hunk,
and the heading lines of the owner's resolution,
copied from
`${HOME}/temp/agent/m2-diff3.txt` (lines 751,
752,
812,
813,
814 and 864) and from `m2_planning.resolved`:

```text
# ${HOME}/temp/agent/m2-diff3.txt and mergiraf-eval/real/m2_planning.resolved, marker and heading lines only
<<<<<<< m2_planning.ours
### Round 3: review findings on the leave-out and its disposal
||||||| m2_planning.base
=======
### Round 3: the conversion tool refused clean worktrees (0.5.18)
>>>>>>> m2_planning.theirs
# resolved file (the owner's resolution), heading lines only:
### Round 2: merge of main, the snapshot message, and the earlier-release service
### Round 3: the conversion tool refused clean worktrees (0.5.18)
### Round 4: the cross-vendor reviews of the content proof and the conversion
### Round 5: review findings on the leave-out and its disposal
```

Results (number of lines differing from the owner's resolution):
- Git:
  conflict (104).
- union,
  mergiraf and weave:
  clean (101,
  102,
  102),
  each output holding both "Round 3" sections and then "Round 4".
- jj at line level and at word level:
  conflict (107).

The owner's resolution renumbered the sections to rounds 3,
4 and 5 and put the earlier-landed branch's rounds first.
So for this shape a clean automatic result is a wrong result:
the headings collide and the order differs.
Here the conflict was the correct signal.

Run time,
single samples inside the container and not repeated (so no comparison is claimed):
Git 6 to 18 ms per file,
mergiraf 53 to 1624 ms (the README took 1624 ms),
weave 30 to 157 ms.
Mergiraf resolved the real planning-document hunk although it conflicted on the small `md2` and `md6` fixtures;
its Markdown behaviour depends on the structure,
so the fixtures do not predict it.

## Facts, with citations

All Git source paths are in a shallow clone of tag v2.56.0 at `${HOME}/temp/agent/git-2026-11-03`.
Mergiraf source is at `${HOME}/temp/agent/mergiraf-2026-11-03` (HEAD committed 2026-11-01T20:54:45+01:00).
Weave source is at `${HOME}/temp/agent/weave-2026-11-03` (HEAD committed 2026-11-02T15:01:15-08:00).
Jujutsu source is at `${HOME}/temp/agent/jj-2026-11-03` (HEAD `08edabb3be19c67dab3ebd46c05b056b19d56d4c`,
committed
2026-11-03T02:07:53Z).

### Mergiraf

- Licence:
  `GPL-3.0-only` (`Cargo.toml` line 16;
  `LICENSE.txt` is the GPLv3 text).
  The wrapper crate is `LGPL-3.0-or-later` (`package/git-policy/cli/Cargo.toml` line 6).
- Latest release v0.20.0,
  2026-09-26 (Codeberg API `repos/mergiraf/mergiraf/releases/tags/v0.20.0`);
  earlier releases 0.19.1 (2026-09-01),
  0.19.0 (2026-08-16),
  0.18.0 (2026-07-14):
  active.
  Published on crates.io as `mergiraf` 0.20.0,
  25862 downloads (crates.io API).
- Languages (`doc/src/languages.md`):
  Rust,
  TypeScript (`*.ts *.tsx *.mts *.cts`),
  JSON (`*.json`),
  TOML,
  YAML,
  plus many more.
  Markdown is not in that doc page,
  but `src/supported_langs.rs` line 1214 defines a `Markdown` profile for `*.md`,
  and the v0.20.0 release binary lists `Markdown (*.md)` in `mergiraf languages` (measured).
  `.jsonc` is not a listed extension;
  open issue 826 asks for JSON5/JSONC support ([issue 826][mergiraf-826]).
  Measured:
  a `.jsonc` file falls back to the plain line merge and conflicts.
- The Markdown profile is nearly empty:
  `src/supported_langs.rs` lines 1229 to 1233 make only `link_reference_definition`
  children of a `section` commutative (`restricted_to`),
  plus code-fence injections.
  Paragraph text is an atomic `inline` node,
  so edits inside a paragraph cannot be split.
  Headings and sections are not commutative,
  so appended sections still conflict.
- How it works (`doc/src/architecture.md`,
  "Fast mode"):
  it runs Git-style line merge first and returns it if clean;
  otherwise it runs a tree-sitter based three-way tree merge (GumTree matching,
  PCS triples,
  as in Spork) on the
  conflicted region.
  Documented checks:
  delete/modify check,
  duplicate-signature check,
  parse error falls back to the line merge,
  commutative merging only for configured "commutative parents".
- Driver registration (`doc/src/usage.md`):
  `git config --global merge.mergiraf.driver 'mergiraf merge --git %O %A %B -s %S -x %X -y %Y -p %P -l %L'`
  and `* merge=mergiraf` in attributes;
  `mergiraf=0` in the environment disables it.
  It also supports `mergiraf solve` on a file already containing conflict markers (needs `diff3` style),
  and `mergiraf review` for results it produced (a cache of attempts).
- Library use:
  `src/lib.rs` lines 12 to 15 say "Mergiraf is not designed to be used as a library so far,
  the Rust API is
  therefore not meant to be stable".
  The public entry point is `merge::line_merge_and_structured_resolution` (`src/merge.rs` lines 40 to 52):
  eleven parameters,
  owned `Cow<'static, str>` contents,
  a `&'static Path`,
  `DisplaySettings<'static>`,
  a `Duration`
  timeout.
  `Cargo.toml` has 45 `tree-sitter-*` dependency lines (grammar crates,
  C code compiled by `cc`),
  `tree-edit-distance`
  and `diffy-imara`,
  and its profile sets `panic = "abort"` (a root-crate setting;
  the wrapper's own profile would
  govern if linked).
  The release binary is 82271264 bytes (measured,
  musl build).
- Verified issue reports of wrong clean results (all open as of 2026-11-03):
  - [issue 853][mergiraf-853]:
    TOML merge emits two `[workspace]` tables,
    exit 0,
    invalid TOML
    (`Cargo.toml` with more than about 43 members).
  - [issue 829][mergiraf-829]:
    TS/JS edit applied to the wrong object property,
    exit 0,
    valid code,
    wrong value;
    `git merge-file` conflicts.
  - [issue 769][mergiraf-769]:
    TOML delete/modify of a table merged into a different table by position.
  - [issue 805][mergiraf-805]:
    YAML merge dropped one of two added pairs.
  - [issue 780][mergiraf-780]:
    bad merge with Rust `use` items.
  - [issue 825][mergiraf-825]:
    invented an invalid combination around a renamed annotation parameter.
- Its own docs state the order assumption openly:
  `doc/src/conflicts.md` "Strictly speaking,
  the order of declaration of
  class attributes or methods in Java can have an influence on program execution ... this tool assumes that such
  conflicts can be resolved without human intervention",
  and formatting merges are "best-effort".

### Does git merge-tree honour a merge driver

Yes,
with conditions.

- Content merge goes through `merge_3way` in `merge-ort.c` (lines 2108 to 2175),
  which calls `ll_merge(...)` (line 2161)
  for each path that both sides changed.
  `ll_merge` is in `merge-ll.c` (the successor name of `ll-merge.c`):
  line 414 `load_merge_attributes`,
  line 429
  `git_check_attr(istate, path, check)` reads the `merge` and `conflict-marker-size` attributes,
  line 439 `find_ll_merge_driver`,
  line 448 runs the driver.
- Driver definitions come from configuration only (`merge.<name>.driver`,
  `Documentation/gitattributes.adoc`
  lines 1135 to 1190),
  never from the tree.
  A repository cannot ship the driver definition;
  each clone,
  or the wrapper (`git -c merge.<name>.driver=...`),
  must
  supply it.
  An attribute naming a driver that is not defined silently falls back to the built-in text merge
  (`merge-ll.c` lines 363 to 390,
  "default to the 3-way");
  measured:
  no warning on stderr,
  exit `1`.
- An external driver is run through the shell with three temporary files:
  `ll_ext_merge`,
  `merge-ll.c` lines 191 to 270
  (`child.use_shell = 1`,
  line 247).
  Exit `0` is clean,
  `1` to `128` is a conflict (the result file `%A` is still read,
  so markers stored in the tree),
  above `128` is an error.
  So any external driver is a spawned process per conflicted file,
  even under `merge-tree`.
- Where attributes are read from without a worktree:
  - `attr.c` lines 841 to 872 (`read_attr`):
    with an attribute source tree it reads `.gitattributes` blobs from that tree;
    without one,
    in a non-bare repository it reads the checked-out worktree file (direction `GIT_ATTR_CHECKIN`,
    the zero
    value of the enum in `attr.h` lines 223 to 227),
    then the index;
    in a bare repository it reads neither.
    The system file,
    the global file and `$GIT_DIR/info/attributes` are also read (`attr.c` lines 419 to 422 and 874 to 934).
  - `merge-ort.c` lines 2042 to 2105:
    the fake `attr_index` it builds holds `.gitattributes` only when `renormalize` is
    set,
    so it does not rescue the bare case.
  - The attribute source is `git --attr-source=<tree-ish>` (`Documentation/git.adoc` lines 230 to 234),
    `GIT_ATTR_SOURCE`,
    or `attr.tree` (`Documentation/config/attr.adoc`;
    the latter is ignored when the first two are
    given).
    `attr.c` lines 1194 to 1242 implement all three;
    a bad value dies with `bad --attr-source or GIT_ATTR_SOURCE`.
  - Measured with Git 2.56.0 (`out3.txt`;
    fixture:
    `.gitattributes` with `* merge=mergiraf`,
    struct edit versus field
    insert):
    - non-bare,
      `.gitattributes` checked out:
      driver used (exit 0,
      clean)
    - non-bare,
      file deleted from the worktree but in both commits,
      no source:
      driver not used (exit 1)
    - same with `--attr-source=main`:
      driver used (exit 0)
    - bare clone,
      committed `.gitattributes`,
      no source:
      driver not used (exit 1)
    - bare clone with `--attr-source=main`,
      `GIT_ATTR_SOURCE=main`,
      `-c attr.tree=main`:
      driver used (exit 0)
    - bare clone,
      `info/attributes` containing `* merge=<driver>`:
      driver used (all `none/union/mergiraf` runs in `out.txt`)
    - source tree without the file (`theirs` after deleting it):
      driver not used (exit 1)
  - Consequence for the wrapper:
    its `merge-tree` calls run from objects alone,
    so a per-path driver is honoured only if
    the wrapper passes `--attr-source` (which tree?
    ours tip,
    theirs tip,
    or a merged `.gitattributes` is a design
    decision) or writes `info/attributes`,
    and supplies `-c merge.<name>.driver=...` for an external driver.

### The built-in union driver

- `Documentation/gitattributes.adoc` lines 1125 to 1132:
  "Run 3-way file level merge for text files,
  but take lines from
  both versions,
  instead of leaving conflict markers.
  This tends to leave the added lines in the resulting file in random
  order and the user should verify the result.
  Do not use this if you do not understand the implications."
- Source:
  `merge-ll.c` lines 149 to 166 (`ll_union_merge` sets `variant = XDL_MERGE_FAVOR_UNION`,
  calls `ll_xdl_merge`);
  `xdiff/xmerge.c` lines 286 to 313:
  a conflict hunk (`m->mode == 0`) is rewritten to the favour mode,
  and mode `3` emits
  the postimage of side 1 and then side 2.
  Binary or oversized input still takes the binary driver (`merge-ll.c` lines 114 to 125),
  which conflicts.
- Only true overlaps are affected:
  it does not change non-conflicting hunks,
  but for a true overlap it keeps both versions
  silently,
  with exit status clean and no marker.
- Measured (`out.txt`,
  `out2.txt`):
  - "both change the same line differently":
    both lines kept one after the other (`md3`,
    `rs4`,
    `wl1`,
    `wl2`)
  - JSON append:
    `"b": 2` and `"c": 3` kept with no comma between them,
    so invalid JSON (`json1`,
    `jsonc1`)
  - Rust statement edit plus insert:
    both `let a = 10;` and `let a = 1;` kept (`rs1`),
    duplicate struct field `size` (`rs2`)
  - Markdown appended sections:
    the blank line between the two sections is lost (`md2`)
  - order-dependent statements both kept (`rs8`:
    `v += 1;` then `v *= 10;`)
  - TS and Rust appended functions:
    the closing `}` of the first function is lost,
    so the file no longer parses
    (`ts2`,
    `rs3`)

  Union is therefore only appropriate for files where every line stands alone
  (see "The built-in union driver for append-only documents").

### Measured behaviour on the owner's shapes

Environment:
Git 2.56.0 from the repository's test image
`localhost/git-policy-native-test:gate71-snapshot-trash-review-findings`.
Container bounds for every run:
`podman run --rm --network=none --cpus 2 --memory 2g --pids-limit 256 --blkio-weight 30 --user 0`,
inside `systemd-run --user --scope` with `CPUQuota=200% MemoryMax=2G TasksMax=256 IOWeight=30`
(`IOWeight` 30 was within the cap in force when those runs were started;
the cap is now 10 and later work will use 10 or
less).
Each case builds a base commit and two branches,
clones it bare,
and runs `git merge-tree --write-tree` with the tool as
driver via `info/attributes`.
Tools:
mergiraf 0.20.0 musl release archive,
weave-driver 0.7.1 (x86_64 gnu),
jj 0.46.0 (x86_64 musl).
Verification of archives:
weave and jj sha256 equal the publisher digest (GitHub release API `digest`,
and weave
`checksums.txt`);
mergiraf publishes no checksum file (the Codeberg release lists only the seven archives),
so its
archive hash `fc77469be936b02ab9021b984a3aebc9250a21ae3d264f2ef2198eb8b31b2a7b`
is recorded but is not verified against a
publisher value.
It ran only inside the bounded,
network-less container.
Mergiraf and weave were run as merge drivers;
jj was run as `jj new main theirs` in a colocated repository with
`merge.hunk-level` set to `line` and to `word`.

Legend:
C means a conflict was left;
ok means resolved cleanly (form defects are noted in the line).
The fixture names (`md1b`,
`rs1`,
and so on) are the names of the throwaway fixtures in the evidence directory.

#### Edit-then-insert shape: edit a line and insert right after it

- Markdown,
  lines in one paragraph (`md1b`,
  the likely owner case):
  Git C,
  union ok (keeps the stale line too),
  mergiraf C,
  weave C,
  jj line C,
  jj word ok ("Flags are optional,
  see below."
  then "Exit code is zero on success.",
  correct).
- Markdown list item edit plus new item (`md1c`):
  Git C,
  mergiraf ok,
  weave C,
  jj word ok.
- Markdown one-line paragraph edit plus new paragraph after a blank line (`md1d`):
  Git ok already.
- Plain text (`txt1`,
  the probe shape):
  Git C,
  mergiraf C (not a known language),
  weave C,
  jj word ok.
- Rust statement edit plus insert (`rs1`):
  Git C,
  mergiraf ok (`let a = 10; let c = 3;`),
  weave C,
  jj word ok.
- Rust struct field edit plus new field (`rs2`):
  Git C,
  mergiraf ok,
  weave ok,
  jj word ok.
- The histogram variant of the owner's probe is not helped (owner's measurement,
  not repeated).
- One unchanged line between the changes (`md4`):
  Git ok,
  all tools ok.

#### Both-append shape: both sides append different text at the end

- Markdown sections (`md2`,
  `md6`):
  Git C,
  mergiraf C,
  weave ok (both sections kept,
  but theirs placed before ours,
  and in
  `md6` no blank line before the second heading),
  jj C.
- Markdown link reference definitions (`md5`):
  mergiraf ok (its one commutative case),
  weave C,
  jj C.
- Rust functions (`rs3`):
  Git C,
  mergiraf ok (no blank line between the two functions),
  weave ok (theirs first),
  jj C.
- TypeScript functions or constants (`ts2`,
  `ts3`):
  Git C,
  mergiraf C,
  weave ok.
- TypeScript imports (`ts1`):
  mergiraf ok,
  weave ok.
- TOML new tables (`toml1`):
  mergiraf ok (no blank line between tables),
  weave ok;
  new keys (`toml2`):
  mergiraf ok,
  weave ok,
  jj C.
- JSON new keys (`json1`):
  mergiraf ok,
  weave ok,
  jj C;
  JSONC (`jsonc1`):
  mergiraf C (unsupported extension),
  weave C.
- YAML list items (`yaml1`):
  mergiraf C,
  weave C (both treat a sequence as ordered).

#### Same-line overlap shape: true overlap on one line

Fixtures `md3`,
`rs4`,
`rs6`,
`rs8`:
- mergiraf:
  always C.
  weave:
  C.
  jj:
  C.
  union:
  silently keeps both.
  Git:
  C.

Same line,
different words (`wl1` prose,
`wl2` Rust call `scale(1, 2)` to `scale(10, 2)` versus `scale(1, 20)`):
- Git C;
  mergiraf C for prose but **ok for the call** (`scale(10, 20)`);
  weave C for both;
  jj word **ok for both** (`Run the fast tool on a large file.`,
  `scale(10, 20)`).

So the answer to "does any tool produce a clean result where a human would need to choose":
yes for mergiraf (code
arguments) and for jj word level (any text),
no for weave on these fixtures.

### Other candidates

#### Weave

- Repository ([weave][weave-repo]):
  Rust,
  `MIT OR Apache-2.0` (`crates/*/Cargo.toml`),
  v0.7.1 released 2026-11-02,
  1344 stars,
  pushed 2026-11-02,
  a small team that "reviews and adapts" PRs
  (README "Contributing").
- Entity-level (functions,
  classes,
  keys,
  Markdown headings) merge on tree-sitter via `sem-core`;
  38 languages including
  Markdown,
  TOML,
  YAML,
  JSON.
  `weave-core` is on crates.io (0.7.0,
  1429 downloads) with `pub use merge::{entity_merge, ...}`
  (`crates/weave-core/src/lib.rs`).
- Its refusals are explicit:
  the conflict marker carries a `// refused_by: ...` line inside the file (measured;
  in
  Markdown that comment line stays visible in the text),
  and it ships `weave check`,
  `weave explain`,
  and `weave land`
  with PROVEN / VERIFIED / REFUSED labels plus `--verify-cmd`;
  its README states these labels "describe merge checks,
  not
  proof that the program behaves correctly".
- The benchmark tables in its README are the vendor's own (31 scenarios it wrote,
  replay of merge commits),
  not
  independent.
- It declines to merge reordering-sensitive unions (decorators,
  lists,
  statement sequences) per README "Declaring a
  container a set".
  It does not resolve the likely owner case (`md1b`) or statement edit plus insert (`rs1`),
  but does resolve appended
  Markdown sections.

#### Jujutsu

- Repository ([jj][jj-repo]):
  `Apache-2.0` (`Cargo.toml` line 21 at the source revision),
  Rust,
  v0.46.0 (2026-10-07),
  very
  active.
- `merge.hunk-level = "word"` (`docs/config.md` lines 2142 to 2152;
  `CHANGELOG.md` line 1931).
- The algorithm:
  line-level hunks,
  then each hunk that is still a conflict is re-diffed by word and kept only if the
  word-level result is fully resolved.
  In the source revision this is `merge_hunk_by_word` at `core/src/files.rs` line 351,
  with the comment on why partial
  results are not used at lines 358 to 362 ("Partially-merged conflicts would be hard to review because they would have
  mixed contexts").
- It is language-blind:
  a data-model-free text algorithm,
  so it also acts on prose,
  which fits Markdown.
  It does not solve both-append shapes,
  and it never reorders.
- Its diff is its own code,
  not `similar` or `imara-diff`:
  `core/src/diff.rs` (1804 lines at the source revision,
  with
  `find_word_ranges` at line 56 and `ContentDiff::by_word` at line 774) uses `hashbrown`,
  `bstr`,
  `itertools` and
  `smallvec`;
  `core/src/files.rs` is 1132 lines.
- jj also lists `mergiraf` as a built-in external merge tool (`docs/config.md` line 1381),
  that is,
  it spawns it.
- Whether the merge function is available from a published crate is in "Source check of the Jujutsu merge function".

#### Source check of the Jujutsu merge function

The research text said that `jj-core` 0.46.0 is on crates.io with `pub mod files`
and that `jj_core::files::try_merge` is
public.
That was not checked against the published crate.
Checked for this document:

Source revision (`${HOME}/temp/agent/jj-2026-11-03`,
HEAD `08edabb3be19c67dab3ebd46c05b056b19d56d4c`,
2026-11-03T02:07:53Z,
workspace version still 0.46.0):
- `core/src/files.rs` line 328:
  `pub fn try_merge<T: AsRef<[u8]>>(inputs: &Merge<T>, options: &MergeOptions) -> Option<BString>`.
- `core/src/lib.rs` line 44:
  `pub mod files;`,
  and `lib/src/lib.rs` line 46:
  `pub use jj_core::files;`.
- `MergeOptions` at `core/src/files.rs` line 281.
- So at this revision the function is public in `jj-core`,
  but this layout is unreleased.

Published `jj-core` 0.46.0 (crates.io tarball unpacked at `${HOME}/temp/agent/jj-core-crate-2026-11-03/jj-core-0.46.0`;
Apache-2.0,
created 2026-10-07T22:33:23Z):
- `src/lib.rs` declares no `files` module and the tarball has no `src/files.rs`;
  a search of `src` for `try_merge`,
  `MergeOptions` and `hunk_level` finds nothing.
- So the published `jj-core` does not expose `files::try_merge`.

Published `jj-lib` 0.46.0 (tarball at `${HOME}/temp/agent/jj-lib-crate-2026-11-03/jj-lib-0.46.0`):
- `src/lib.rs` line 46:
  `pub mod files;`;
  `src/files.rs` line 305:
  `pub fn try_merge`;
  `merge_hunk_by_word` at line 328,
  its
  partial-result comment at line 335.
- `MergeOptions` at `src/tree_merge.rs` line 57 (`pub mod tree_merge` at `src/lib.rs` line 93) with fields
  `hunk_level: FileMergeHunkLevel` (`Line` or `Word`,
  `src/files.rs` line 268) and `same_change`.
- So today `try_merge` is reachable from a published crate only through `jj-lib`.

Dependencies from the crates.io API (queried 2026-11-03;
saved as `${HOME}/temp/agent/deps-*.json` and
`jj-core-deps.json`):
- `jj-core` 0.46.0:
  22 required non-optional normal dependencies:
  `async-trait`,
  `blake2`,
  `bstr`,
  `chrono`,
  `clru`,
  `digest`,
  `either`,
  `futures`,
  `globset`,
  `hashbrown`,
  `indexmap`,
  `itertools`,
  `jj-core-proc-macros`,
  `ref-cast`,
  `regex`,
  `same-file` (Windows only),
  `serde`,
  `smallvec`,
  `tempfile`,
  `thiserror`,
  `tracing`,
  `winreg` (Windows only).
  `rayon` is not among them;
  it appears in the unpacked `Cargo.lock` only through the `criterion` development dependency.
- `jj-lib` 0.46.0:
  36 required non-optional normal dependencies,
  including `jj-core`,
  `rayon`,
  `pest`,
  `pest_derive`,
  `prost`,
  `gix-ignore`,
  `toml_edit`,
  `rand`,
  `rustix` (Unix only) and `windows-sys` (Windows only).
- `imara-diff` 0.2.0:
  Apache-2.0,
  2 dependencies (`hashbrown`,
  `memchr`).
- `similar` 3.2.1:
  Apache-2.0,
  no dependencies.
- None of `jj-core`,
  `jj-lib`,
  `imara-diff` or `similar` is in `package/git-policy/cli/Cargo.lock`
  (searched 2026-11-03).
  Neither `imara-diff` nor `similar` is a dependency of `jj-core`,
  so a port on either would reimplement jj's
  `ContentDiff` and the hunk resolution rather than reuse them,
  and its results could differ from jj's.

Still untested:
whether `try_merge` is usable by a crate other than jj (only that it is `pub` where stated),
and the
dependency tree beyond the first level.

#### Others

- Spork (`ASSERT-KTH/spork`):
  MIT,
  Java only,
  last push 2025-02-25,
  JVM start per file;
  not applicable to these file types
  (mergiraf `doc/src/related-work.md`).
- jdime (`se-sic/jdime`):
  LGPL-2.1,
  Java only.
  IntelliMerge (`Symbolk/IntelliMerge`):
  MIT,
  Java only,
  last push
  2021-07-21.
  Not applicable.
  Related academic tools (FSTMerge,
  s3m,
  jsFSTMerge,
  automerge-ptm) are Java or JavaScript ES5 only (same related-work
  page).
- git-mediate (`Peaker/git-mediate`):
  Haskell,
  GPL-2.0,
  active (push 2026-09-16).
  It only resolves a conflict that has become trivial after a person edits it (one side unchanged relative to base);
  it
  does not solve any measured shape by itself (README).
- Mercurial:
  `internal:merge`,
  `internal:merge3`,
  `internal:union` exist ([merge tools][hg-merge-tools],
  fetched);
  `:union` "will use both local and other sides for conflict regions by adding local on top of other",
  the same silent
  union as Git's.
  A search summary mentions an experimental `relaxed-block-sync-merge` setting that stops neighbouring edits from
  conflicting;
  I could not read the changeset (the Mercurial site blocked the fetch with an access-denied page),
  so treat
  that as unverified.
- Pijul and Darcs:
  patch-based systems whose manuals claim better handling of reorderable changes
  ([why Pijul][pijul-why]);
  they are different version control systems,
  not embeddable resolvers,
  and I found no source
  for how either treats the measured adjacent-line shape.
  Not evaluated.
- Mergiraf's related-work page links `jelmer/awesome-merge-drivers`;
  I read that list:
  Mergiraf,
  Weave,
  Plume-lib merging
  and `smartmerge` are the "all-in-one" entries;
  the rest are single-format drivers (Cargo.lock,
  pnpm lock,
  notebooks).

### Risk side: documented wrong clean results and the checks these tools apply

- Textual and semantic conflicts are different things:
  a clean line merge of non-adjacent edits can already be wrong
  (measured `rs5`:
  one side renames `helper` to `helper2`,
  the other adds a call to `helper`;
  every tool,
  including plain
  Git,
  merges clean).
- Cases produced here by tools that resolved something Git would have stopped on:
  - `rs7`:
    one side adds a parameter to `f` and updates a call,
    the other adds a new call `f(2)` right after the old one.
    Git conflicts (adjacent),
    mergiraf and jj word return clean with `f(2)` still using the old arity.
    I did not compile it;
    a type error is expected (inference).
  - `rs9`:
    both sides add an enum variant with the same explicit value `2`.
    Mergiraf,
    weave and union return clean
    (a compile error is expected;
    Git conflicts).
  - `wl2` (mergiraf and jj word):
    `scale(10, 20)` from two argument edits in one call;
    may be intended or not.
  - Appending in an order-sensitive place:
    mergiraf and weave refused `rs8` (statements) and yaml sequences;
    union did not.
- Published results about false negatives (wrong clean merges):
  the LastMerge paper compares jDime,
  Spork,
  LastMerge and
  Mergiraf ([arXiv 2507.19687][lastmerge-paper]);
  it exists,
  but I did not read its numbers in full,
  so no figure is
  quoted here.
- Checks each tool applies:
  - Git `merge-tree`:
    none beyond the line algorithm;
    exit status clean or not.
  - mergiraf:
    delete/modify detection,
    duplicate-signature detection on commutative parents,
    parse-error fallback to the
    line merge,
    a timeout parameter,
    `mergiraf review` of its own results,
    and "err on the side of caution"
    (mergiraf.org front page).
    It does not re-parse its output to prove validity ([issue 853][mergiraf-853] shows invalid TOML at exit 0);
    I did not
    find such a check in `src/merge.rs`.
  - weave:
    refuses reorder-sensitive unions,
    verifies unions ("an entry edited,
    deleted or moved beside the insertion,
    one
    key with two values,
    or an answer that does not parse stays a conflict",
    README),
    `weave check` against the three
    stages,
    and `weave land` with a user-supplied `--verify-cmd`.
  - jj word level:
    only keeps a word merge when every sub-hunk resolves;
    no syntax check.
- Repository-level evidence that these checks matter:
  this repository's TOML (`Cargo.toml` with workspace members) is
  exactly the class [issue 853][mergiraf-853] describes.

## Assumptions

These are not verified;
they are stated so they can be challenged.

- Section appends in the repository's planning documents commonly come from parallel branches,
  so the "Round 3 twice"
  outcome is likely to recur;
  I verified it for one merge only.
- Mergiraf and weave quality on this repository's real files (long Rust files,
  `Cargo.toml`,
  Markdown docs of several
  hundred lines) will resemble the small fixtures.
  Not measured at scale;
  the fixtures are tiny.
- Linking a `GPL-3.0-only` library into an `LGPL-3.0-or-later` binary makes the combined binary subject to
  `GPL-3.0-only` terms;
  the repository's licence policy decides whether that is acceptable.
  This is not legal advice and I did not read the repository's licence decision documents.
- A merge result that a person did not see (auto-push reconcile) needs a stricter standard than one shown in a working
  tree where the person reviews it.

## Candidate approaches

Common baseline for all:
Git's own result stays the first pass;
a second pass runs only on paths Git left conflicted.
That keeps "Git decides what it can decide" and limits the blast radius to files that would otherwise stop the person.

### Keep Git's behaviour and only explain the conflict

Pros:
no new merge semantics,
no licence or dependency change,
never a wrong clean result,
fits "never discard user data".
Can be cheap:
the wrapper already reads `merge-tree` output and conflicted paths (`--name-only`,
or stage lines
`<mode> <object> <stage> <filename>`,
`Documentation/git-merge-tree.adoc` lines 125 to 140).
It could classify each conflicted hunk ("touching edit and insertion",
"both appended at end",
"same line") and name the
resolution.
Cons:
the person still resolves by hand;
does nothing for auto-push reconcile,
which stops.
Resolves of the measured shapes:
none (explains all).

### Built-in word-level resolver inside the wrapper

This is the approach the owner chose for merges a person runs and for the wrapper's unattended merges
(see "Owner decisions of 2026-11-03").

Idea:
when Git leaves a hunk conflicted,
re-merge only that hunk at finer granularity and accept only results fully
resolved,
the algorithm jj uses (see "Jujutsu").
Candidate building blocks:
the jj merge function (published only in `jj-lib` today,
in `jj-core` at an unreleased
revision;
see "Source check of the Jujutsu merge function"),
or a small word tokenizer plus an existing diff crate
(`similar` or `imara-diff`);
the wrapper already links in-process tools instead of spawning (`Cargo.toml` comments for the
scanner and linter).
The choice between these is the open decision.

Pros:
- no spawn,
  no attribute or driver configuration (the wrapper post-processes the stage blobs itself);
- works for Markdown,
  TOML,
  YAML,
  TS,
  Rust and unknown types alike,
  and resolves the likely owner case;
- Apache-2.0 is the permissive licence least likely to raise a question against the LGPL crate (the repository owner
  decides);
- small surface the owner controls.

Cons:
- language-blind,
  so it can combine two edits of one line into a result neither side wrote (`scale(10, 20)`,
  `wl2`);
- it does not resolve both-append;
- writing the result back needs object writes (the wrapper must write blobs and a tree without spawning,
  or accept
  `git hash-object`,
  `git mktree`,
  `git commit-tree` spawns it already uses).

Possible guard:
restrict to prose files (`.md`),
or require an equality check that every output line comes from base,
ours,
or theirs and no old line from either side disappears;
ask the person to confirm for code.

Resolves:
edit-then-insert yes for Markdown,
Rust,
plain text (measured on the real README hunk:
identical to the owner's
resolution);
both-append no (the real planning-document hunk stays a conflict,
which was the right outcome there);
same-line overlap no (conflict stays) except for different-word edits in a line.
Plain statement:
word-level merging can emit text that neither side wrote (`Run the fast tool on a large file.`,
`scale(10, 20)`).
It loses no side's characters ("never discard user data" holds),
but it does not guarantee that the combination means what
either author intended.

### Mergiraf as a spawned merge driver

Pros:
mature (v0.20.0,
active,
many languages),
handles Rust struct-field and statement shapes,
TOML/JSON/Rust
both-append,
TS imports,
delete/modify check,
works for a person's `git merge` and the wrapper's `merge-tree` calls once
configured.

Cons:
- spawns per conflicted file (contrary to "invoke real git as rarely as possible" in spirit,
  and requires installing the
  binary through `toolchain.lock.json`);
- requires config and attribute plumbing (driver config is not in the tree;
  the wrapper must supply
  `-c merge.mergiraf.driver` and `--attr-source` or `info/attributes`;
  see "Does git merge-tree honour a merge driver");
- a missing config degrades silently to a text conflict;
- misses the likely owner case (Markdown paragraph lines),
  Markdown section appends,
  TS function appends,
  YAML lists,
  JSONC;
- documented silent wrong results in TOML and TS ([issue 829][mergiraf-829],
  [issue 853][mergiraf-853]) hit this
  repository's file types;
- `GPL-3.0-only` is only an execution licence issue here (spawned process,
  no linking).

Resolves:
edit-then-insert for Rust only (and Markdown lists),
both-append for Rust,
TOML,
JSON,
TS imports;
not Markdown
prose,
not Markdown sections.

### Mergiraf linked as a library

Pros:
no spawn,
no attribute plumbing (the wrapper calls it for the conflicted stage blobs).

Cons:
everything in "Mergiraf as a spawned merge driver",
plus:
- the maintainers say the API is unstable and not designed for library use (`src/lib.rs` lines 12 to 15);
- an eleven-parameter entry point with `'static` borrows;
- many C grammar crates in the dependency graph (45 `tree-sitter-*` lines in `Cargo.toml`) and a large binary (the
  standalone release is 82271264 bytes);
- `panic = "abort"` in its own profile contradicts the wrapper's need for panic unwinding at its catch boundaries
  (`Cargo.toml` comment on `forbidden-strings`);
- `GPL-3.0-only` linked into an `LGPL-3.0-or-later` crate changes the licence of the distributed binary;
- each upgrade is a source-level API risk.

Not recommended while a spawned form exists.
Resolves:
same as the spawned form.

### The built-in union driver for append-only documents

This is the approach the owner chose for named append-only files (see "Owner decisions of 2026-11-03").

Pros:
zero dependencies,
one attribute line,
honoured by every `merge-tree` call once attributes are visible
(see "Does git merge-tree honour a merge driver"),
and the only option here that resolves Markdown section appends with a
one-line change (but see Cons).

Cons:
silent for every true overlap in the file (see "The built-in union driver"),
including after a person edited the same
sentence;
lost blank lines between sections (`md2`);
Git warns "do not use this if you do not understand the implications".

Safe scope:
files in which every line is an independent record and order does not matter,
such as a `.gitignore`,
a
changelog of bullet lines,
or a log.
Planning documents with prose are not such files:
a true overlap would produce two near-duplicate paragraphs,
without
marker.
The research text judged it unsafe for the planning documents.
Whether handover documents (the owner's example `doc/handover/*.md`) fit the safe scope is not measured here.

Resolves:
both-append for any file (with defects),
edit-then-insert for any file but keeps the stale line (`md1b`,
`rs1`),
same-line overlap by keeping both.

### Weave as a spawned driver

Not in the original list;
included because it solves the planning-document shape.

Pros:
Apache-2.0 or MIT;
resolves both-append for Markdown sections,
Rust,
TS,
JSON,
TOML;
explicit refusal reasons;
check
tools.

Cons:
- very young (v0.7.x,
  first-party benchmarks only,
  a pace of releases that may break behaviour);
- spawn per file;
- ordering of union results is "theirs first" (a surprising order for a planning document);
- marker lines contain extra `// refused_by` text inside Markdown;
- no resolution for the likely owner case;
- `weave-core` is a published crate but pulls the `sem-core` grammar set.

Resolves:
both-append (Markdown,
Rust,
TS,
TOML,
JSON),
edit-then-insert only for struct fields.
On the real planning-document hunk it returned a clean file with two "Round 3" headings that the owner had to renumber
(see
"Planning-document hunk"),
so resolving appended sections is not an improvement for documents whose headings are numbered
or ordered by time.

## Suggested ranking

This ranking is the research agent's suggestion made before the owner answered;
it is not a decision.
Ranking by value for the repository's file mix and the owner's stated preferences (embed,
spawn rarely,
never lose user data).

Ranking:
keep Git's result as first pass plus explanation plus the word-level pass as an opt-in second pass,
then
`union` for named append-only files,
then mergiraf as a spawned driver for Rust and TOML,
then weave as a spawned driver,
then mergiraf linked as a library,
because of the adjacent-pair reasons below.

- Explanation plus word-level pass over `union`:
  the word-level pass needs no attribute or driver plumbing and resolves the
  most probable owner case (adjacent lines in hard-wrapped Markdown) with a rule the owner can read,
  while union resolves
  it only by keeping a stale line and,
  in a prose file,
  turns every true overlap into silent duplication.
  (Plain explanation alone is the zero-risk floor;
  it is ranked together with the word-level pass because the pass can be
  introduced behind it.)
- `union` over mergiraf spawned:
  union is dependency-free and honoured by `merge-tree` once attributes are visible,
  and its
  failure is bounded to files the owner names;
  mergiraf adds a binary,
  a spawn,
  config plumbing and known wrong-clean cases
  in TOML and TS,
  and still misses the Markdown shapes that caused the question.
- Mergiraf spawned over weave spawned:
  mergiraf is the more mature (Codeberg lists releases back to v0.13.0 on
  2025-07-23;
  I did not find weave's first release date) and its Rust and TOML handling resolves shapes the repository hits
  often;
  weave resolves the append shape for Markdown,
  which mergiraf does not,
  but its benchmarks are the vendor's own,
  its union order is surprising,
  and its pace of change is high.
- Weave spawned over mergiraf linked:
  both are heavy,
  but linking adds a second layer of risk (unstable API,
  `GPL-3.0-only` linking,
  `panic = "abort"` profile) on top of the same resolution quality as the spawned mergiraf;
  a
  spawned weave carries none of these.

### Caveats on the ranking

- The real planning-document conflict argues against resolving both-append automatically in documents:
  three tools
  returned a clean file whose content was wrong for the owner (colliding "Round 3" headings,
  wrong order).
  The research text therefore ranked append resolution (union,
  weave) below the word-level pass,
  which leaves that
  conflict alone.
  If the owner still wants it,
  the least risky form is a rule that resolves only when the two appended headings are
  distinct and unnumbered,
  which was not evaluated.
- Unlike the planning document,
  the README conflict is resolved identically to the owner's choice by the word-level pass;
  that is one sample,
  and `wl1` and `wl2` show the same pass can also combine same-line edits into text neither side wrote.
- Any option that returns a clean result changes what a person sees:
  a conflict is currently the only signal.
  For auto-push reconcile,
  where nobody reviews the merge before it is pushed,
  the research text would keep Git's result
  only,
  or allow a second pass only with a printed list of hunks it merged.
  That is a suggestion,
  not a measured requirement;
  the owner chose the second pass there,
  with logging.

## Limits and unknowns

- The fixtures in "Measured behaviour on the owner's shapes" are small and hand-written.
  The only real data is the replay of two merge commits (eight files,
  two conflicted hunks);
  no wider corpus replay of this
  repository's history was done.
  The weave and mergiraf vendor corpora are theirs;
  the figures in that section are counts of the research fixtures,
  not
  rates.
- I did not run mergiraf `solve` (post-hoc resolution of existing markers) or `--compact`,
  nor the `diff3` conflict style
  recommended by its docs;
  the driver path was used.
- Time and memory of each tool per file were not measured in repeated or loaded runs,
  per the load rule;
  the single
  samples under "Planning-document hunk" are not a comparison.
- The mergiraf archive hash could not be verified against a publisher checksum because none is published;
  the container
  had no network and ran only fixture files.
- jj was run as a separate version control binary on a colocated repository;
  whether `try_merge` is usable by a crate other
  than jj was not tested,
  and the dependency lists in `Source check of the Jujutsu merge function` are first-level only.
- Mercurial behaviour,
  Pijul and Darcs were not tested;
  the Mercurial changeset claim is unverified.
- The LastMerge paper's false-negative figures were not read in full.
- Compile results for `rs7`,
  `rs9` and `rs5` outputs were not checked with a compiler;
  the expected errors are inference.
- Which tree should supply `--attr-source` for a wrapper merge of two arbitrary branches is a design question;
  Git offers
  only a single tree,
  and a `.gitattributes` that exists on one side only (measured:
  not found when absent from that tree)
  is not merged first.
- Git's `merge-ort` also applies to renames and directory renames;
  a driver sees only paths where both sides changed the
  blob,
  after rename detection.
  Rename-heavy conflicts were not tested.
- The repository's licence decision documents were not read.

## Corrections made while writing this document

The research text was compared with sources and fixtures when this document was written.
These statements changed:
- `try_merge` location:
  the research text placed `files::try_merge` in published `jj-core` 0.46.0;
  it is in published
  `jj-lib` 0.46.0 and,
  unreleased,
  in `jj-core` at the source revision (see "Source check of the Jujutsu merge function").
  Its cited line range for the word-level function (`core/src/files.rs` lines 305 to 379,
  comment 357 to 362) matches
  neither source;
  the lines given in this document were read from the files.
- README hunk sides:
  the research text said the `ours` side extended the sentence line and the `theirs` side kept it;
  the
  fixtures show the reverse,
  and that both sides insert lines after the line.
- Planning-document hunk:
  the research text said both branches added "Round 3";
  the `theirs` side added "Round 3" and
  "Round 4",
  and the owner's resolution holds rounds 3,
  4 and 5.

The dependency count for `jj-core` was not measured in the research text;
it is measured here.

## Evidence files

Output files from the research:
`${HOME}/temp/agent/mergiraf-eval/` (scripts and `out*.txt`),
downloaded archives in
`${HOME}/temp/agent/*-dl-2026-11-03/`,
extracted binaries in `${HOME}/temp/agent/*-bin-2026-11-03/`,
source clones in
`${HOME}/temp/agent/*-2026-11-03/`.
Added for this document:
`${HOME}/temp/agent/jj-core-crate-2026-11-03/`,
`${HOME}/temp/agent/jj-lib-crate-2026-11-03/`,
`${HOME}/temp/agent/m1-diff3.txt`,
`${HOME}/temp/agent/m2-diff3.txt` and the crates.io API responses
`${HOME}/temp/agent/jj-core-deps.json`,
`deps-jj-lib-0.46.0.json`,
`deps-imara-diff-0.2.0.json`,
`deps-similar-3.2.1.json`.
None are in the repository,
and scratch may not survive a reboot.

[mergiraf-769]: https://codeberg.org/mergiraf/mergiraf/issues/769
[mergiraf-780]: https://codeberg.org/mergiraf/mergiraf/issues/780
[mergiraf-805]: https://codeberg.org/mergiraf/mergiraf/issues/805
[mergiraf-825]: https://codeberg.org/mergiraf/mergiraf/issues/825
[mergiraf-826]: https://codeberg.org/mergiraf/mergiraf/issues/826
[mergiraf-829]: https://codeberg.org/mergiraf/mergiraf/issues/829
[mergiraf-853]: https://codeberg.org/mergiraf/mergiraf/issues/853
[weave-repo]: https://github.com/Ataraxy-Labs/weave
[jj-repo]: https://github.com/jj-vcs/jj
[hg-merge-tools]: https://www.mercurial-scm.org/help/topics/merge-tools
[pijul-why]: https://pijul.org/manual/why_pijul
[lastmerge-paper]: https://arxiv.org/abs/2507.19687

## 2026-11-03T02:08:29-05:00: merge-code options measured, and the owner's last two decisions

Asked where the merge code should come from,
the owner first answered "Do the measurements first then ask.".
A same-vendor agent (rule IRW:
a second pass,
not an independent review) measured three options against the
`jj` 0.46.0 binary as the reference.
Evidence,
in scratch that may not survive a reboot:
`~/temp/agent/merge-code-options/`
(`results/correctness.txt`,
`differential.txt`,
`builds.jsonl`,
`bench.log`,
`bench-extra.txt`;
arms `arm-a`,
`arm-b`,
`arm-c`,
`arm-c-myers`).

### Correctness

- 44 inputs:
  the two real conflicted files of this repository,
  the measured shapes,
  and the Rust,
  Markdown,
  TypeScript,
  TOML,
  YAML,
  JSON and JSONC fixtures.
  Every option gave the reference's verdict on all 44,
  and every resolved output was byte-identical to it:
  23 resolve and 21 stay conflicted.
- Word level resolves 11 inputs that line level leaves conflicted,
  among them the README conflict of `14601b28d3`,
  whose merged output equals the committed resolution byte for byte.
- 600 seeded random edit pairs on real README and `lib.rs` text:
  the port of Jujutsu and the `similar` version
  with the Myers diff agreed on all 600;
  the `similar` version with the Patience diff differed in one verdict.

### Cost

- Port of Jujutsu's code:
  about 1270 lines (943 of code) from `diff.rs`,
  `files.rs` and `merge.rs`,
  7 small crates
  (no new package name in the wrapper's lockfile),
  +107,200 bytes stripped,
  about 2 s clean build;
  its upstream
  tests are not ported.
- `jj-lib` 0.46.0:
  127 compiled crates,
  111 new lockfile entries (71 new names),
  about 60 s clean build,
  +447,648 bytes stripped.
- Own merge on `similar` 3.2.1 with Myers:
  about 150 lines,
  1 new package name,
  about 10 s clean build,
  +141,560 bytes stripped,
  merge about 2.5 to 3 times faster than the other two.
- Every option adds about 30 to 55 microseconds to a process start and exit,
  below the run-to-run spread of the
  baseline (interquartile range about 100 to 180 microseconds);
  a merge of a 170 KB file takes 0.5 to 1.6 ms.
- Licence:
  Jujutsu and `similar` are Apache-2.0 and neither ships a NOTICE file;
  a port must keep the copyright
  headers and mark modified files (Apache-2.0 section 4).

### Decisions

- Where the merge code comes from,
  owner's choice (question tool,
  2026-11-03):
  "Own merge on similar
  (Recommended)",
  that is,
  a word-level three-way merge of the wrapper's own on `similar` 3.2.1 with the Myers diff,
  kept equal to Jujutsu's results on the measured corpus by tests.
- Which files get the union driver,
  owner's choice:
  "doc/handover/*.md only (Recommended)".
  Done in `.gitattributes` (`8ecbb0b188`);
  `git check-attr merge` shows `union` for a handover log and
  `unspecified` for this planning document.
- With the earlier answers (a word-level second pass for merges a person runs,
  and the same pass in the wrapper's
  unattended merges with every merged part logged),
  every decision of this proposal is now made.
  Still to design inside the implementation:
  which tree supplies attributes to `git merge-tree` in the wrapper's
  object-only merges,
  so that the union line also applies there.
- What a fully resolved merge does when the second pass settles every conflicted part,
  owner's choice (question tool,
  2026-11-03):
  "Finish like a clean merge (Recommended)",
  that is,
  the merge,
  rebase,
  cherry-pick,
  revert or `git pull` ends as a clean run of that command would,
  honouring `--no-commit`,
  `--squash`,
  `--edit` and similar options.
  Adopted from that answer without a further question:
  the finish runs the hooks a clean merge runs
  (`pre-merge-commit`,
  not `pre-commit`),
  respects a person's `attr.tree` as Git does,
  and derives the candidate size bounds from Git's own constants.

## 2026-11-03T03:39:36-05:00: implemented, the word-level second merge pass

Implemented in `package/git-policy/cli` on `feat/git-policy-cli-word-level-second-merge-pass` as the owner decided
(question tool,
2026-11-03):
the pass for merges a person runs,
the same pass in the wrapper's unattended merges,
the merge code of the wrapper's own on `similar` 3.2.1 with the Myers diff,
and the `union` line of `doc/handover/*.md`
applying in the merges from objects.
The rules are in `SPEC.md` ("Word-level second merge pass") and the user's view in `README.md`.

### What was built

- `src/native/word_merge.rs`:
  the prototype of the measurement (`arm-c-myers`) with the report of the merged regions
  (line spans in base,
  ours and theirs,
  and the merged text).
  The 41 inputs of the stored corpus (`src/native/word_merge_corpus/`:
  the 44 of the measurement without the three large
  benchmark inputs `large_spec_*`,
  whose 520 KB of text a test does not need) get Jujutsu 0.46.0's verdict and,
  when resolved,
  its bytes;
  the README conflict of merge `14601b28d3` resolves to the committed blob;
  both-append stays a conflict.
  The 600 seeded random edit pairs of the measurement are not repeated:
  they compared two programs,
  and the `jj` binary is not
  in the tests;
  a generated property (no inserted word and no base word is dropped,
  both verdicts occur) replaces them.
- `src/native/merge_second_pass.rs`:
  which conflicted files may be re-merged (all three stages,
  one plain-file mode,
  no NUL
  byte in the first 8000 bytes,
  at most 1 MiB,
  `merge` attribute unspecified,
  set or `text`),
  the re-merge,
  the blob and
  tree writing,
  and the report words.
  The 8000 bytes are Git's own binary probe;
  the 1 MiB is this implementation's bound on the cost of a diff of two
  unrelated large files,
  not a measured value.
- `src/native/merge_second_pass_worktree.rs`:
  merge,
  rebase,
  cherry-pick and revert forwarded to Git.
  `src/native/index_writer_forward.rs` reads the unmerged paths in process before Git starts and again when Git exits with
  a status other than 0;
  the paths unmerged only afterwards are the pass's,
  so a file a person is resolving by hand is never replaced.
- `src/native/merge_second_pass_tree.rs`:
  the pass for merges from objects,
  used by `merge_isolation.rs` (the isolated merge),
  `auto_push_reconcile.rs` (the reconcile merge) and `replay_merge.rs` with `service_replay.rs` (the commit replay),
  and the attribute source.

### Where the second pass runs, with file and function

- A person's merge,
  rebase,
  cherry-pick,
  revert:
  `run_coordinated_forward` in `index_writer_forward.rs`,
  after Git's status
  is known (`watch_before` and `resolve_after`).
  `git pull` is not included:
  its merge runs inside Git and the owner named four commands.
- The isolated merge (`git merge` with another session's staged index):
  `merge_tree` in `merge_isolation.rs`.
- The reconcile merge:
  `commit_merge` in `auto_push_reconcile.rs`.
- The commit replay:
  `merge_captured` in `service_replay.rs`,
  last,
  after the subsumption rules and their second merge
  (`word_pass`),
  so a captured file that already holds the landed edit is still settled by subsumption first.

### Which tree supplies the attributes

`GIT_ATTR_SOURCE` is set to the first side's tip for the three merges from objects (the isolated merge's `HEAD`,
the reconcile
merge's branch tip,
the landed tip of a replay),
unless the environment already sets it.
Reasons:
the planning measurement showed that a bare or worktree-less `git merge-tree` reads no `.gitattributes` without a
source;
the first side is the branch whose checkout would apply its attributes;
Git takes one tree only,
so a `.gitattributes`
that only the second side has is not read;
the variable beats `attr.tree`,
so a person's `attr.tree` is overridden (a person's own
`GIT_ATTR_SOURCE` is kept).
The same source makes the second pass's `git check-attr` agree with the merge.
The controls:
a both-append in `doc/handover/log.md` merges by `union` in the reconcile merge and in the replay merge when the
checkout holds no `.gitattributes`,
which conflicts without the source.

### Logging

- Standard error:
  one line per part,
  `cli-git: word-level merge: <path>: part <n> of <m>: base <lines>, ours <lines>, theirs <lines>
  became "<text>"`,
  and for a person's merge a closing line that names the staged files.
- Journal:
  one `warning` entry per merged file (`SYSLOG_IDENTIFIER=cli-git`),
  fields `CLI_GIT_WORD_MERGE_FILE`,
  `CLI_GIT_WORD_MERGE_OPERATION` and `CLI_GIT_WORD_MERGE_PARTS`.
- "With the commit" for the unattended merges,
  as implemented:
  the reconcile merge commit's message ends with the part lines;
  the replayed commit's message is the person's and is not changed,
  so the replay reports the parts in a new `word-merged` event
  next to the commit's other events (`SPEC.md`,
  "Word merged event").

### Measurements

Method:
the unchanged baseline executable (`e27dd206de`,
built with `mise run //package/git-policy/cli:build`) and the new one
(`5d7d068382`'s code;
the executable is 224632 bytes larger,
12526832 against 12751464) run alternately,
with a second copy of
the baseline in every round as the same-binary control that gives the run-to-run spread (rule QNB),
on the shared host at a load
average between 14 and 21 (nobody waits for a quiet host).
Scratch scripts,
not in the repository:
`${HOME}/temp/agent/wlmp/merge-perf.ts` and the raw samples in `merge-perf-*.json`,
`nperf-*.json`;
they do not survive a reboot reliably.

- A merge Git completes by itself (`git merge --no-edit`,
  400 tracked files,
  60 rounds):
  medians 20.80 ms (baseline),
  21.34 ms
  (second baseline copy),
  21.18 ms (new),
  16.57 ms (real Git without the wrapper).
  New minus baseline:
  +0.38 ms;
  baseline copy minus baseline:
  +0.54 ms;
  the interquartile range of one arm is about 1.7 ms.
  The difference is inside the spread.
- The same with 20000 tracked files (30 rounds):
  340.9,
  360.2,
  358.9 and 338.3 ms;
  new minus baseline +18.0 ms,
  baseline copy
  minus baseline +19.3 ms,
  interquartile range about 100 ms.
  The in-process read of the index before Git starts is inside the spread there too;
  it is not separated from it.
- A merge Git stops on,
  with an edit and an insertion right after it (400 files,
  60 rounds):
  the baseline stops with the conflict
  at 18.44 ms;
  the new executable resolves it in 24.85 ms,
  +6.41 ms for the five Git starts of the pass (`check-attr`,
  `cat-file --batch`,
  `hash-object`,
  `update-index`,
  `checkout-index`).
  At 20000 files the difference (+15.3 ms) is inside that run's spread (+7.7 ms between the two baseline copies).
- `mise run //package/git-policy/cli:native:perf` (30 samples per scenario),
  eight runs of each executable,
  alternating
  (baseline,
  new,
  baseline,
  new,
  then new,
  baseline,
  baseline,
  new,
  new,
  baseline,
  baseline,
  new,
  new,
  baseline,
  baseline,
  new):
  commit through the service,
  paired delta median:
  baseline 41.3 ms (run medians 40.3 to 57.0),
  new 43.0 ms (40.5 to 76.2),
  +1.7 ms;
  signed commit:
  baseline 51.4 ms,
  new 54.6 ms,
  +3.3 ms.
  The forwarded commands' added p95 stays about 2 ms (baseline medians of the run p95s 1.8 to 2.1 ms,
  new 2.0 to 2.5 ms).
  The largest paired commit delta seen was 876 ms for the baseline and 3878 ms for the new executable (one loaded run),
  against the bound of 5000 ms;
  the signed maximum was 1328 ms and 767 ms.
  So a commit through the new executable is about 1.7 ms (4 percent) slower in the median in these runs,
  more than the 0.4 ms
  between two baseline runs but inside the 40 to 45 ms range of the baseline's run medians;
  the cause was not isolated
  (the commit path has no new code except one `Option` in the preparation;
  the executable is 1.8 percent larger).
  Every bound of the tool held.
- Tests at `7b26167ecd` (the last code change;
  later commits add one control and remove a duplicate guard):
  the whole library
  suite 3340 passed and 5 skipped,
  the whole binary suite 557 passed and 1 skipped,
  Clippy clean.
- Guard removal (each guard removed in turn,
  the controls that must fail run,
  the file restored):
  every guard listed in the commit
  messages failed at least one control,
  after two additions (the isolated merge refused whole,
  and the redundant early driver skip removed).


### Needs the owner's decision

1.  After a person's merge where every conflicted file resolved,
    the wrapper stages the files and keeps Git's exit status
    (the operation is not finished:
    `git commit`,
    `--continue`).
    Finishing the operation itself (`git commit --no-edit`)
    was not done;
    it is the broader reading of "accept a file".
2.  `git pull` (its inner merge or rebase) is not covered.
3.  The attribute source is the first side's tip;
    a merged `.gitattributes` of both sides is not built,
    and `attr.tree` is overridden.
4.  The merged blobs and trees are written with `git hash-object`,
    `read-tree`,
    `update-index` and `write-tree` (the spawns the
    crate already uses for this),
    not in process;
    that is only on the conflict path.
    Writing loose objects in process is possible
    (the crate has an inflate and a SHA-1 and SHA-256 already,
    and `miniz_oxide` can deflate) and would remove those starts.
5.  The 1 MiB and 8000-byte bounds of the candidate test are this implementation's choices.
6.  The replay's journal entries are sent when the merge is used,
    before the policies run again on the merged tree,
    so a commit
    that a policy then refuses still has its entries.
7.  The entries carry the identifier `cli-git` (the wrapper's),
    also the ones the replay sends from inside `cli-git-service`;
    the service's own entries carry its own identifier.
8.  A commit through the new executable is about 1.7 ms slower in the median (see "Measurements");
    the owner's bound
    (five seconds longer than native Git) holds by three orders of magnitude,
    but the cause of the 1.7 ms is not isolated.
9.  The corpus holds 41 of the 44 measured inputs and no random-edit comparison against the `jj` binary (see "What was built").

## 2026-11-03T04:16:06-05:00: the finish like a clean run, pull, the attribute tree precedence, Git's bounds, the service's identifier

Follow-ups the owner asked for after the first implementation,
on the same branch (merged with
`integrate/git-policy-cli-conversion-and-lock-free-add` at `9f44842b11` first,
with no conflict).
The red controls came first (`ab31df93c5`:
with the finish disabled 11 of the 19 binary controls failed);
the code is `2373ad42bc`
and `d5f0bae69a`.

### Finish like a clean merge

The owner chose "Finish like a clean merge (Recommended)" on 2026-11-03:
when every conflicted file resolves,
the operation ends
as Git ends it after a clean run,
`--no-commit`,
`--squash`,
`--edit` and similar stop as Git would,
and the continuing step goes
through the wrapper's own command path.
The Git 2.56.0 source says what a clean run does (file and line,
in `SPEC.md`,
"Word-level second merge pass"):
a merge commits at `builtin/merge.c` line 1828 or,
with `--no-commit`/`--squash`,
prints "Automatic merge went well;
stopped before
committing as requested" and exits 0 (lines 1867 to 1873);
cherry-pick and revert commit with the original message,
edit only on
request (and a revert on a terminal),
and Git's own `--continue` is `git commit --no-edit --cleanup=strip` for a conflicted pick
(`sequencer.c` lines 5354 to 5378);
a rebase continues with `git rebase --continue`,
which edits the message of the conflicted pick
(`commit_staged_changes`,
line 5381) where a clean pick does not.
Built in `src/native/merge_second_pass_finish.rs`,
called from `resolve_after` in `merge_second_pass_worktree.rs`:
the continuing command is `run_process` of the same process (`entry.rs`),
so a later conflicting pick of a rebase gets the pass again
(controls:
a rebase with two conflicting picks;
a cherry-pick range) and the commit goes through the policies (control:
a
`forbidden-strings` policy rejects the ending commit and leaves the merge in progress).
Findings from the work:
- A plain `git commit` in a rebase conflict loses the original author (measured),
  so the rebase is continued with `git rebase --continue`
  and its editor switched off by an in-process variable (`CLI_GIT_CHILD_SET_GIT_EDITOR`,
  `child_environment.rs`);
  a todo list with
  `reword`,
  `squash` or `fixup -c` is not finished,
  because the same setting would switch off the editor those steps want.
- The hint `# Conflicts:` that Git appends to `MERGE_MSG` is cut before a merge commit (`without_conflicts_hint`).
- A concluding `git commit` runs `pre-commit` where a clean merge runs `pre-merge-commit`;
  external hooks are switched off in the
  wrapper's Git starts either way (`external-hooks-unsupported`).
- What Git printed when it stopped (`CONFLICT ...`,
  "Automatic merge failed") is already out;
  the final status and output are the
  finishing command's,
  plus one line per part.
- Options are read by exact spelling (Git takes abbreviations);
  an unknown option,
  `--autostash`,
  a `branch.<name>.mergeOptions` setting
  or a nesting deeper than 64 finishes (this implementation's choice) leaves the operation stopped with the files staged,
  as before.

### Pull

`git pull` is covered:
it is an index writer the wrapper already stays alive around,
its merge or rebase is a child of Git
(`builtin/pull.c` lines 525 and 750) that the wrapper never sees,
and the state files left after its exit tell which one stopped.
Impossible or declined modes,
from the source:
`--autostash` and `branch.<name>.mergeOptions` (the finish cannot see `--no-commit` or
`--squash` in a setting);
a pull whose options this reader does not know exactly.

### The attribute tree

Git's precedence is `--attr-source`,
then `GIT_ATTR_SOURCE`,
then the configuration `attr.tree`,
and an `attr.tree` that is no tree is
ignored (`attr.c` lines 1202 to 1231).
The wrapper names the first side's tip only when none of the three applies
(`attribute_tree_variables`).
The configuration is read in process;
the in-process readers decline an environment that holds a `GIT_*`
variable they do not know (`GIT_EDITOR` is one),
and then Git is asked once for `attr.tree`.

### The bounds

- The binary probe is Git's `FIRST_FEW_BYTES`,
  8000 (`xdiff-interface.c` line 197).
- Git's size limit is `MAX_XDIFF_SIZE`,
  1023 MiB (`xdiff-interface.h` line 14,
  `merge-ll.c` lines 117 to 122).
  It cannot bound this pass:
  texts that
  share no line took 0.08 s and 45 MB at 1 MiB,
  0.73 s and 352 MB at 8 MiB,
  4.12 s and 1320 MB at 32 MiB (release build,
  one core).
  The pass takes the smaller of Git's limit and 8 MiB,
  which is this implementation's choice backed by that measurement.

### The service's journal identifier

The auto-push runs inside `cli-git-service`;
its entries carry `cli-git-service`.
The commit replay and a push a person typed run
in the wrapper and keep `cli-git`.

### Tests and guards

Full library suite 3442 passed and 5 skipped,
full binary suite 595 passed and 1 skipped,
Clippy clean,
each run once.
Guards removed in turn,
each failing at least one control:
unknown options,
`--no-commit`/`--squash` stop,
the rebase editor,
the todo list that wants an editor,
the conflict hint cut,
the rest of a sequence,
the `mergeOptions` decline,
the nesting limit,
the three
attribute-tree precedences,
the service identifier,
the pull rebase state,
and `--no-commit` of a pick.

### Needs the owner's decision

1.  The ending commit of a merge runs through `git commit`,
    so `pre-commit` is the hook name where a clean merge runs `pre-merge-commit`.
2.  The 8 MiB bound and the 64 nesting levels are this implementation's choices (measured for the first,
    not for the second).
3.  One extra Git start for `attr.tree` in an environment the in-process readers decline.
4.  Declining the finish for `pull --autostash` and for `branch.<name>.mergeOptions`.

## 2026-11-03T04:45:20-05:00: hooks parity, autostash and mergeOptions, a derived nesting bound, plain environments

Follow-ups on the same branch,
after the container gate began on a copy.
Red controls first (`e95e54d9a0`),
code in `8bc4b70b7a`.

### Hooks

The premise that a finished run must run the hooks a clean run runs does not change the code,
because the wrapper runs none:
every Git it starts has every hook event switched off (`hooks_off.rs`;
`SPEC.md`,
the hook section),
so a user's hooks,
traditional or
configured,
see nothing on a clean merge,
rebase,
cherry-pick or revert and nothing on a finished one.
The control `user_hooks_see_the_same_on_a_finished_run_as_on_a_clean_run` records ten hook events for both,
compares them,
and has
real Git run directly as the positive control (it fires them),
so an empty log means something.
It was green from the start;
the
only red it can show is a finish that bypasses the wrapper's Git starts.
What Git would run if hooks were run,
from the source:
a clean merge `pre-merge-commit`,
`prepare-commit-msg`,
`commit-msg` (`builtin/merge.c`
lines 931,
966 and 975) and `post-merge` (line 537);
a concluding `git commit` `pre-commit`,
`prepare-commit-msg`,
`commit-msg` and
`post-commit` (`builtin/commit.c` lines 780,
1116,
1133 and 1966);
a clean pick `prepare-commit-msg` and `post-commit`
(`sequencer.c` lines 1367 and 1699);
a rebase `pre-rebase` (`builtin/rebase.c` line 1839) and `post-rewrite` (`sequencer.c` lines 1329
and 5328).
If hooks are ever supported,
the finish would run `git commit` hooks where the clean merge runs the merge ones;
that
difference then needs a decision.
The earlier SPEC sentence that a concluding commit "runs `pre-commit` where a clean merge runs
`pre-merge-commit`" was true of Git and not of the wrapper,
and is rewritten.

### Autostash and mergeOptions

Nothing needs declining.
A merge's `--autostash` and `merge.autoStash` are applied by the ending `git commit` (`builtin/commit.c` line 1982) as
the clean merge's `finish` applies them (`builtin/merge.c` line 1848);
a rebase's `--autostash` and `rebase.autoStash` by the continuing step
(`sequencer.c` lines 4980 and 5330);
a pull hands both to its child.
Controls:
merge flag and config,
rebase flag and config,
pull.
The rebase pair was already green:
`rebase --continue` does the work.
One real defect was found by the merge control:
the wrapper's commit builds the ending commit in a shadow repository that held a **link**
to the real `MERGE_AUTOSTASH` pseudoref,
so Git's removal after applying it removed only the link and the real pseudoref stayed;
the
next merge's concluding commit would have applied the stash again.
`MERGE_AUTOSTASH` is now a conclusion state file with a private copy in the
shadow,
and landing removes the real one (`conclusion_cleanup.rs`).
A stash whose apply conflicts is stored by Git in the shadow's refs
and is not carried back;
that is the wrapper's existing behaviour for a person's own `git commit` after `merge --autostash` too,
and
is not changed here.
`branch.<name>.mergeoptions` is split as Git splits it (`alias.c` lines 126 to 180) and parsed before the command line
(`builtin/merge.c` lines 1407 to 1409),
for the branch that is checked out;
controls:
`--no-commit` there stops the finished merge,
a quoted `-m` is its message.

### The nesting bound

The fixed 64 is gone.
The budget is the remaining todo length plus one for a rebase;
for a cherry-pick or revert the sequencer todo length
(one for a single pick),
because every pick of a range can conflict and each is one finish;
one for a merge (the coordinator's "one
finish").
The controls pin the three values and the comparison.

### Variables the in-process readers tolerate

`GIT_EDITOR` was already tolerated (`repo_read_layout.rs`);
the decline seen in this shell's unit tests came from the real user-level
configuration,
not from it.
The list now also tolerates the `GIT_` names that only trace,
word output,
reach a network or a tool,
or set
a limit or a write-time choice,
each with the file and line of Git 2.56.0's reader in the comment on `PLAIN_ENVIRONMENT_PREFIXES`;
every name that can
move a configuration,
attribute tree,
layout,
index,
object,
ref or pathspec answer,
every `GIT_TEST_*` and any name not known stays a decline.
The attribute-tree probe now reads with the variables its children get,
so a person's usual environment costs no Git start (control with
a runner that refuses every start).

### Tests

Clippy clean;
library suite 3448 passed and 5 skipped;
binary suite 598 passed,
1 failed,
1 skipped,
the failure being
`commit_timing::a_commit_that_waits_for_a_foreign_index_holder_is_attributed_to_that_wait`,
a timing attribution that passed alone straight
afterwards (14 of 14 of its module).
Guards removed in turn,
each failing a control:
mergeOptions before the command line,
its quoting,
the three budget values and the comparison,
`--autostash`,
the landing removal of `MERGE_AUTOSTASH`,
`GIT_TRACE` and a moving variable in the
plain list,
the in-process attribute probe.

### Needs the owner's decision

1.  If hooks are ever supported:
    the finished merge runs the hooks of `git commit`,
    a clean merge those of `git merge` (the hook list under `Hooks`).
2.  The edit to `conclusion_cleanup.rs` is outside the merge paths (a commit-conclusion file);
    it is small and fixes a defect,
    but another agent may be editing near it.
3.  A stash whose apply conflicts inside the shadow is not carried back to the real repository.

## 2026-11-03T11:45:26-05:00: the 0.5.28 base merged in, autostash and mergeOptions verified, a pull that merges in isolation

### The merge of the 0.5.28 base

`integrate/git-policy-cli-plain-commit-no-quiet-index-in-process` (tip `9763599f3a`) was merged into this branch.
Six files conflicted;
four were additive and kept both sides:
the imports of `auto_push_reconcile.rs`,
 the test modules of
`binary_tests.rs`,
the tests of `auto_push_reconcile_tests.rs`,
and this document
 (the base's decision bullet joined the list
that ends in "every decision of this proposal is now made",
and this branch's sections follow it).
`README.md` kept this branch's "Word-level second merge pass" section between the base's merge subsections
 and its commit
section,
and the base's "A conflict" paragraph,
which said the pass "would plug in" at `merge_tree`,
now says
 where it runs.
The one semantic choice is in `merge_tree` of `src/native/merge_isolation.rs`:
 the base forwards a conflict with a clean index to
native Git (`Some(1) if !staged_work`),
this branch runs the pass on every conflict.
Both are kept:
 with another session's staged
work the pass runs from objects and the merge is refused whole if a file does not resolve;
 with a clean index the conflict stays
native Git's,
which stops and writes the markers,
and the pass then runs on the stopped merge.
 The base's order was kept,
not
"resolve first,
forward when unresolved",
because the stopped path finishes through the wrapper's commit path,
 so the repository's
commit policies judge the merge commit
 (the finish control `the_ending_commit_is_judged_by_the_repositorys_policies` pins
that for the stopped path);
whether a merge completed from objects is judged by the same policies was not measured.
 Where the intents truly conflicted the base's behavior won;
nothing else did.
The merge left one duplicate import,
removed in a separate commit;
compile,
library
 and binary controls of the touched modules passed.

### Autostash and mergeOptions, verified

The controls from `8bc4b70b7a` were read and run:
`a_merge_autostash_is_applied_by_the_ending_commit` (flag and
`merge.autoStash`),
`a_rebase_autostash_is_applied_when_the_rebase_ends` (flag and `rebase.autoStash`),
`a_pull_autostash_is_applied_by_the_ending_commit` and `a_merge_options_setting_is_applied_to_the_finish`
 (`--no-commit` and a
quoted `-m` in `branch.<name>.mergeoptions`).
They are sound for a stash that applies cleanly.
One defect they do not reach:
when the stash conflicts with the merged file,
 Git's apply runs in the commit's scratch repository,
so
`stash store` writes the scratch repository's private `refs/stash` and the unmerged entries stay in the private index.
 The real
repository ends with an empty stash list and the file marked modified,
where Git ends with `stash@{0}: autostash`
 and the file
unmerged;
Git's words,
which the person sees,
promise the stash.
The conflict markers are in the worktree file,
 so the lines are not
gone,
but the stash entry is.
 `a_merge_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it` states Git's result (run
against real Git on a twin repository) and is `#[ignore]`d with this reason;
it fails when run with `--run-ignored all`.
A fix has to copy the stash commit and its objects out of the scratch repository,
store it in the real reflog,
 and carry the unmerged
entries into the real index under the landing's lock;
that is the commit service's landing path,
 which this branch does not touch.
This is item 3 of the earlier "Needs the owner's decision" lists,
now with a measured case.

### Pull

A `git pull` that merges did not go through the isolated merge:
 `run_merge` in `builtin/pull.c` starts `git merge ... FETCH_HEAD` as
a child of Git (`cmd.git_cmd = 1`),
so the merge never reached the wrapper,
and the control
`a_pull_that_merges_is_completed_over_staged_work_and_equals_native_gits` was red before the build (native Git:
 "Your local changes to
the following files would be overwritten by merge ... Merge with strategy ort failed.",
status 2).
`src/native/pull_isolation.rs` now splits a pull that is stated to merge and read exactly into the fetch and the merge
(`pull_step`,
called from `run_coordinated_forward`);
the isolated merge reads `FETCH_HEAD` for its head and its message
(`read_fetch_head` in `merge_isolation.rs`).
The decisions of `cmd_pull` that are mirrored:
the unmerged-index
 and mid-operation
refusals (left to native Git),
`orig_head` against `curr_head` after the fetch (`read-tree -u -m`,
 as `checkout_fast_forward`),
the
head count of `FETCH_HEAD`,
`rebase_unspecified` with no `--ff`/`--no-ff` (Git's advice stays Git's),
 and `GIT_REFLOG_ACTION`.
What is left to native Git's whole pull and why:
a rebase (a separate machine,
not in scope),
`--ff-only`
 (its refusal text),
`--autostash`
and `pull.autostash` (the stash round trip around the merge),
`pull.ff` (Git rewrites it when `--no-rebase` is typed),
 submodules
(`update_submodules` after the merge),
a refspec with a destination,
every option not spelled exactly,
detached
 and unborn heads.
Controls,
each equal to native Git on a clean copy in output,
diagnostics,
tree,
parents and message:
 over staged work,
into a clean
index,
the pulls left to native Git (advice,
`--ff-only`,
`--rebase`,
an abbreviation),
 and a fetch that moves the checked-out branch.
The guard that carries the working tree along was removed once and the last control failed,
then restored.
Not measured:
the cost of the extra Git starts of the split (a `config`,
a `symbolic-ref`,
two `rev-parse`,
 an `ls-files`,
the
`fetch`),
against one `git pull`;
the held-lock wait of a pull is the typed merge's,
tested for the typed merge only.

### Needs the owner's decision

1.  How the stash of a conflicted autostash apply and its unmerged entries are carried back (the ignored control above).
2.  Whether a pull with nothing stated,
    and `--ff-only`,
    should also merge in isolation.
    Git refuses the first with advice and the
    second with a status;
    the owner's rule "Auto merging should never fail."
    could be read to cover both,
    but each refusal is a
    deliberate Git choice,
    so they stay Git's here.

## 2026-11-03T11:59:41-05:00: the coordinator's picks built; the pull split measured

Picks made by the coordinator under the owner's delegation,
each open to the owner's veto.

### Autostash that would conflict: decline the finish (pick B)

`autostash_blocks_finish` (`merge_second_pass_finish.rs`) tries the apply before the finish commits:
 `write-tree` for the tree the
ending commit will have,
then `merge-tree --write-tree --merge-base=<stash>^1 <tree> <stash>`,
 the three-way merge `git stash apply`
makes.
A conflict,
or any failure to try,
leaves the merge stopped as Git leaves it,
with the resolved files staged
 and the work in
`MERGE_AUTOSTASH`.
Red before the code:
`a_merge_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it`
 (the wrapper
finished and exited 0);
green after,
with the clean-apply controls (`a_merge_autostash_is_applied_by_the_ending_commit`,
`a_pull_autostash_is_applied_by_the_ending_commit`) still green,
and the unit control
`an_autostash_that_would_conflict_or_cannot_be_tried_blocks_the_finish`.
Carry-back A is not built
 (the commit service's landing code
is being rewritten);
its control
`a_commit_that_applies_a_conflicting_autostash_keeps_the_stash_as_git_keeps_it` failed on the empty
stash list before it was ignored,
with the reason "fixed by carry-back A,
queued after the index writer's third round".
The dry run is not exact where the working tree holds other unsaved changes to files the stash touches
 (Git's apply refuses
those differently);
that case is not covered.

### The working tree update waits for the index lock (pick A)

`pull_step` waits for the commit service's hold and for any holder of `index.lock` (bounded,
naming it)
 before `read-tree -u -m`.
Red before:
`a_pull_whose_fetch_moved_the_branch_waits_for_a_held_index_lock` ended with status 128.
 The lock harness moved from
`binary_merge_never_fails_tests.rs` to the shared `binary_held_lock_support.rs`.

### Measurement of the split against native Git

Debug build of the wrapper (the test profile),
nine rounds in turns on fresh copies of one repository,
 each with a remote of its own
(a first run without that was wrong:
the wrapper pushes a merge it completes,
 which turned the later pulls into fast-forwards),
on a
host with a load average of 9 to 11.
Times in milliseconds,
sorted;
the median is the fifth.
Merging pull:
native first 13 to 16 (median 16),
native second 14 to 16 (14),
wrapper split 340 to 380 (367),
 wrapper leaving the
pull to native Git (`--autostash`) 91 to 122 (105),
the wrapper's typed `git merge origin/main` 223 to 493 (302).
Fast-forwarding pull:
native first 12 to 14 (12),
native second 11 to 13 (11),
wrapper split 118 to 140 (128),
 wrapper leaving it to
native 24 to 57 (42),
the typed merge 72 to 106 (89).
The spread of one arm on unchanged input (native first against native second) is 2 ms or less,
 so every difference above is real.
The split costs about 350 ms more than native Git for a merging pull and about 115 ms more for a fast-forward,
 in a debug build,
far
under the 5 s bound the owner named but 20 and 10 times native Git's time.
 The merging pull's cost is the isolated merge itself
(the typed merge costs 302 ms) plus about 65 ms for the fetch and the pull's queries;
the fast-forward's 86 ms over the
wrapper's native route is the queries (`config`,
`symbolic-ref`,
three `rev-parse`,
`ls-files`,
`merge-base`),
 which an in-process
reader could answer without a Git start.
A fast-forward or a merge of nothing now skips the isolated attempt
 (it was 166 ms before
`merge-base` told them apart).
A release build was not measured.

### Needs the owner's decision

1.  Whether the pull's queries are to be answered in process to bring a fast-forwarding pull toward native Git's time.

## 2026-11-03T12:13:49-05:00: the conflict warning reads the content, and the conflicted autostash is carried back

### The conflict-marker warning

Seen live in the shipped 0.5.27:
a path resolved by editing and not staged,
committed by name during a merge,
 was reported as
still holding conflict markers although the committed blob had none,
and the remedy given (another commit of the path)
 never
finishes the merge.
`conflict_markers.rs` now reads the committed blob and the `conflict-marker-size` attribute
 (seven by
default) and claims markers only for a blob that has a line of equal `<`
 or `>` characters of that size followed by white
space,
as Git's `is_conflict_marker` reads it (a line of `=` alone underlines a heading and is not a claim).
 A commit of named
paths that leaves a merge or a cherry-pick open now says so in one line with the commands that finish it,
whether
 or not markers
were committed.
Red before the code:
 `a_resolved_but_unstaged_path_committed_by_name_makes_no_marker_claim_and_says_how_to_finish`,
`a_path_committed_by_name_with_marker_lines_keeps_the_claim_and_says_the_merge_is_open` and
`the_claim_follows_the_conflict_marker_size_attribute`;
green after,
with the five existing controls of
`binary_commit_during_merge_tests.rs` and four unit controls.

### Carry-back A

Git's apply of a merge's autostash runs inside the commit's scratch repository.
When it conflicts,
 its `git stash store` writes the
scratch repository's `refs/stash` and the conflict stages stay in the commit's private index.
 The scratch repository writes its
objects into the real object store,
so nothing is copied:
`autostash_carry.rs` stores the stash in the repository
 (once,
with the
`autostash` message) from `remove_conclusion_state`,
and puts the stages into the index the landing installs,
 in `follow_up`,
under
the lock the landing holds.
`a_commit_that_applies_a_conflicting_autostash_keeps_the_stash_as_git_keeps_it` was red
 (empty stash
list,
`M` for `UU`) and is green with its ignore removed;
removing either half fails it (the stash list,
 then the status).
The finish's own decline of a conflicting autostash (pick B) is kept:
with carry-back it could be retired,
 and the finish would
then end like Git's,
which is the owner's choice to make.

## 2026-11-03T12:28:44-05:00: option B retired, the finish applies a conflicting autostash as Git does

The coordinator's pick under the owner's delegation (veto open),
because of "Auto merging should never fail.":
 with carry-back A
the finish needs no dry run.
`autostash_blocks_finish`
 and its unit control are removed from `merge_second_pass_finish.rs`,
`merge_second_pass_worktree.rs` and `merge_second_pass_finish_tests.rs`;
nothing else used the dry run.
Red first,
with B in place:
`a_merge_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it` and
`a_pull_autostash_that_conflicts_on_apply_keeps_the_work_as_git_keeps_it`,
 rewritten to compare the end state with a twin
repository on which real Git stops,
the resolution is made as the word pass makes it,
and `git commit` concludes
 (stash list,
status,
merged file,
the second file,
and the commit's exit status);
the wrapper had left the merge stopped (status 1,
 not the
commit's 0).
Green after the removal,
 with `a_commit_that_applies_a_conflicting_autostash_keeps_the_stash_as_git_keeps_it` and
the clean-apply controls unchanged.
The case B's dry run did not model,
unsaved changes in the worktree to a file the stash touches,
is compared with Git in
`a_commit_over_unsaved_changes_to_a_stash_file_keeps_the_stash_as_git_keeps_it`:
 a file the stash touches is written again after
the merge stopped,
Git's apply refuses and keeps the whole stash,
and the wrapper's commit ends identically.
 That control passed
before the removal too,
because it goes through the person's commit,
which carry-back A already covered;
 the finish makes the same
commit,
so it is the same end.
 A finish cannot meet such unsaved changes unless another program writes the file in the moment
between the pass and the commit;
that interleaving is not constructed.

## Measured again on 2026-11-10: the line level and the refusals

Recorded 2026-11-10T18:20:00-05:00.
The owner wrote that day,
verbatim:
"there are still ways you haven't tried to make this happen less often,
for example,
using a better algorithm or something."
The decision of 2026-11-03 stands:
the pass stays,
and its wrong answers are to become rarer.
This section is the measurement that answered the owner,
and the change it chose.

### The test set

Every two-parent merge reachable from `main`,
3,194 merges,
and the files both sides changed that are text,
3,862 files.
Plain line merging conflicts on 722 of those,
which is the input domain of the pass,
because a file Git resolves never reaches it.
The set is those 722 files plus three named ones:
merge `ef43afe9f9` with `package/git-policy/cli/src/native/policy/policy_forbidden_strings.rs`
(the doubled `return`),
merge `31f72ecf12` with `doc/handover/questions-for-the-owner.md`
(the record both sides wrote),
and merge `b971df44f5` with the same path,
which the owner named.
That last merge carried the record rather than producing it:
its base and its other side are the same bytes,
so the merge had nothing to decide,
and on the three stages of `31f72ecf12` the pass leaves three conflicts,
so the text `b971df44f5` carried was written by whoever resolved it by hand.
That is a finding of this measurement:
the record was not the pass's own output,
although the shape it holds is the shape the pass must never write.

Every candidate is the same all-or-nothing pass:
a file is resolved only when every region of it resolves,
and a file that is not resolved keeps exactly what Git left.
The candidates differ in the line level (the pass's own in-process line merge with the `similar`
crate's Myers algorithm,
or Git's own line merge with `--zdiff3` or `--diff3` and one diff algorithm)
and in whether the two refusals are on.

Each result is scored by mechanical signals,
which the report of the change names:
a silent result is one without a conflict marker,
a doubled word is one that neither side writes twice in a row,
a line no side holds is one no side's file holds,
an empty heading is a Markdown heading whose body no side left empty,
a dropped line is a line both sides hold that the result lost,
and a duplicated block is three or more consecutive lines that appear twice although each side holds them once.
The committed blob is printed for context;
it is not the verdict,
because in this range the wrapper's own pass wrote many of those blobs.

### The counts

723 triples,
every candidate on all of them.
`silent` is files resolved without a marker,
`defect` counts files whose silent result writes a doubled word,
an empty heading or a line both sides kept and the merge lost,
`conflicts` counts conflict regions,
and `invented` counts lines no side holds.

- The pass as it shipped:
  silent 142,
  defect 2,
  conflict regions 2,807 in 581 files,
  invented 92.
- The pass with the refusals,
  its own line merge:
  silent 108,
  defect 0,
  conflict regions 3,002 in 615 files,
  invented 21.
- Git's `--zdiff3` line merge (default algorithm),
  no refusals:
  silent 164,
  defect 2,
  conflict regions 2,753 in 559 files,
  invented 91.
- Git's `--zdiff3` line merge with the refusals,
  chosen:
  silent 130,
  defect 0,
  conflict regions 2,948 in 593 files,
  invented 21.
- Git's `--zdiff3` with `--diff-algorithm=histogram`,
  with the refusals:
  silent 129,
  defect 0,
  conflict regions 2,951 in 594 files,
  invented 21.
- Git's `--zdiff3` with `--diff-algorithm=patience`,
  with the refusals:
  silent 130,
  defect 0,
  conflict regions 2,948 in 593 files,
  invented 21.
- Git's `--zdiff3` with `--diff-algorithm=minimal`,
  with the refusals:
  silent 130,
  defect 0,
  conflict regions 2,948 in 593 files,
  invented 21.
- Git's `--diff3` with `--diff-algorithm=histogram`,
  with the refusals:
  silent 108,
  defect 0,
  conflict regions 3,002 in 615 files,
  invented 21.
- Git's `--diff3` with `--diff-algorithm=patience`,
  with the refusals:
  silent 109,
  defect 0,
  conflict regions 2,999 in 614 files,
  invented 21.
- Of the files each candidate resolved,
  the number whose bytes differ from the committed blob:
  52 of 142 as shipped,
  31 of 108 with the refusals and its own line merge,
  28 of 130 with `--zdiff3` and the refusals.

### What the counts decided

The refusals remove every silent wrong answer the measurement can name,
in every line level:
the two doubled words,
and 71 lines no side holds that 34 resolved files had joined across lines,
leaving 21 lines,
each the merge of a single line all three texts held,
which is the shape the owner accepted on 2026-11-03.
They cost conflicts:
34 files that the pass resolved are left to the person.

The diff algorithm is not what the owner's question was about:
Git's `patience`,
`histogram` and `minimal` change at most one file of 723 against `default`.
The conflict style is:
`--zdiff3` keeps the lines both sides wrote identically out of a conflict,
the word pass therefore sees smaller regions,
and 22 of the 34 files the refusals cost are resolved again,
with the same refusals and the same 21 lines no side holds.
`--diff3` gives the pass's own line merge's counts to the file (108 and 3,002),
because it keeps the matched lines inside the conflict.
So the line level became `git merge-file --zdiff3`,
and Git's default diff algorithm stays.

The word level stays as it was,
because 41 stored inputs of the Jujutsu corpus give the same verdict and the same bytes under
Git's `--zdiff3` line merge as under the pass's own,
so the corpus control still holds the word pass to Jujutsu 0.46.0's answers.

### Evidence

The scripts and the 723-file test set are in `${HOME}/temp/agent/git-merge-pass/`
(`build-corpus3.ts`,
`metrics.ts`,
`run-matrix.ts`,
`score-dir.ts`,
`corpus/`,
`out/`),
with a scratch harness that dumped each candidate's answer for every triple
(`examples/merge_eval_dump.rs` in the crate while the choice was made,
deleted before the commit).
Scratch does not survive a reboot reliably,
and none of it is in the repository;
the counts above are the durable record,
and `src/native/merge/word_merge_refusals/` holds the two real shapes as controls.
