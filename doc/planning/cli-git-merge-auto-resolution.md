# Automatic resolution of "non-conflicting" merge conflicts in the Git wrapper

Status:
proposal.
The owner answered three of four decisions on 2026-11-03 (recorded in "Owner decisions of 2026-11-03");
the fourth,
where the merge code comes from,
is open and is to be asked again with the source check in this document.
Nothing is implemented and nothing in `package/git-policy/cli` was changed.
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
  and the dependency lists above are first-level only.
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
