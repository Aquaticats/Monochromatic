# The ZFS route's root modes as modes of the one wrapper executable

## Status

A proposal,
put to the owner.
It is not a decision,
and nothing here is approved or final.
It records what the project decided in
[the one-program record](../decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md),
what the owner then answered about the ZED hook's name in
[the start-name record](../decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md),
and what the owner has answered since,
about the boot mount provider and the ordering of plan step 19 in
[the step 19 record](../decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md)
and about which checkouts the route serves in
[the step 10 record](../decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md).
Every claim about a program this repository does not own is read from that program's source,
and every claim about this repository's code is read at `5f904d9fbd`,
this round's tip of `main`,
which the round merged in `11f6fa724a`;
that merge is this round's base,
and every line this document prints or pins is that commit's file with this design's change in it.
A citation given at another commit says which,
and gives this base's line beside it where the line moved;
every citation that names a path is resolved at this base in
`keep/citations-at-this-rounds-base.txt`,
which shows the line each one names,
and the bare `:line` continuations that follow a named file in a list were read by hand at the same
base.
The decision records and the handover are untouched by this round.

This is the sixth revision,
the round of 2026-11-24T21:46:53-05:00.
It replaces the text of the fifth revision
(`6681fdf4e5`),
which a same-vendor pass then read.
That pass is not an independent review:
it is a second reader of the same vendor,
and its file is
`~/temp/agent/gate-review-zfs-root-modes-fifth-revision-2026-11-24/work/keep/read-of-the-fifth-one-program-revision.md`.
It found no High,
seven Medium and thirteen Low items,
and this revision answers every one of them in
"What this revision changed,
and why".
The largest answer is the stripper both pins and the release scan read through:
it now reads the strings, raw strings and character literals of the source it strips,
and it fails with the file's name when it ends inside a comment or a literal.
The fifth revision's stripper opened a block comment at any `/*`, including one inside a glob
string, so three real release files ended inside a comment it never closed and 794 non-blank
lines of theirs were invisible to every rule that trusted it (run keep/stripper-over-the-release-tree.log).
The revision also fixes the printed `main.rs` and its pin against `main` as this round found it,
corrects what `mount-path-descendant` checks and in which order,
makes choice 9 agree with itself about the window a stale record leaves,
and prints choice 11's raised-deadline control with the words it needs.

Every claim in this design says what it rests on,
in one parenthesized label:

- (run FILE),
  a command this round ran,
  whose output is that file in this round's `keep/`;
- (replay FILE),
  an earlier round's kept transcript,
  replayed or re-analysed by this round with the same result;
- (source PATH:LINE),
  one line of this repository's code,
  read at `5f904d9fbd` unless the citation says another commit;
- (upstream PATH:LINE),
  another program's source,
  read through `gh`;
- (record PATH),
  an accepted decision record;
- (proposal),
  this design's own wording,
  where no build and no probe exists yet;
- (design),
  this proposal's reasoning:
  inference,
  with no probe,
  source line or record behind it.

## What this revision changed, and why

The fifth read's file is named in "Status".
Its seven Medium and thirteen Low findings are answered here in its own order,
and every answer names the probe, source line or record it rests on.
The round's logs are listed in the appendix,
"Appendix: where the probe transcripts are".

### The guard shape: two files pinned whole, and lists where a pin cannot reach

The last three reads each found new edits that pass the printed guards,
and each revision closed the named ones and opened others,
the fourth and fifth revisions' comment stripper among them.
That is not a run of bad luck:
a guard that lists forbidden spellings can only name the spellings someone has already
thought of,
so its completeness is unknowable,
and every read that tries a new spelling wins.

So the fifth revision changed the shape, and this one keeps it.
The whole code text of two files,
`package/git-policy/cli/src/native/main.rs` and
`package/git-policy/cli/src/native/worktree_clone_mount_handler/root_mode_names.rs`,
is pinned line for line,
in order,
in the guard that reads it.
Any code line added,
removed,
reordered,
reworded or re-spelled fails,
so any change to either file is a change to its guard,
and the guard's claim is true by construction rather than by enumeration.
The pin is compared with the leading blanks trimmed,
because indentation carries no meaning to Rust,
and comments are cut before the comparison,
because they carry no behavior.

What the shape catches,
measured this round on the fifth read's own variants and this round's
(run keep/guards-replay-sixth.log):

- the pinned text fails every `main` variant and every names variant either read named,
  `nm6` (a live function aliased to the pinned name) included,
  and every one of them is a changed code line,
  so every one of them is a changed pin;
- two of the fifth read's variants are controls that must pass (a comment-only edit and a blank
  line), and they pass;
- the fifth revision's printed lists passed some of the same variants,
  which is the fifth read's own before column
  (replay its `keep/guards-replay-this-read.log`,
  where the lists pass the variants whose prohibited spelling they do not name).

What the shape costs a person who edits the file honestly,
measured rather than estimated:

- every honest edit to `main.rs` or to `root_mode_names.rs` edits the pin in the same commit.
  `main.rs` is 27 non-blank code lines at this base and 47 under this design,
  and the repository's history holds 9 commits that touch it in the 41 days from
  2026-10-05 to 2026-11-15,
  7 of them in the 30 days before this round
  (run keep/pin-cost-sixth.log).
  So the pin costs one or two re-pins a week at this repository's pace,
  and the re-pin is one line or a few in a sidecar guard file;
- a person who pastes the guard's printed "actual text" without reading turns the pin into a
  rubber stamp,
  and no shape prevents that: a list has the same failure with less evidence.
  What the pin buys is that the same commit shows the changed code line and the changed pin line
  beside it,
  so a reviewer reads the one decision the edit makes.
  The guard prints the first differing line with both texts and the line count,
  so the update is mechanical and the diff is legible;
- a list costs an honest editor nothing,
  which is exactly why it cannot be complete.

The lists stay where a whole-file pin cannot reach,
and nowhere else:

- the scan over the release tree,
  1,472 release files as this round classifies them,
  where a pin per file is not maintainable;
- the typed bans in `clippy.toml`,
  which resolve the call rather than match its text;
- the pin over the release tree's lint attributes,
  because one `#[allow(` line can excuse a typed ban in a file no text pin covers.

Three facts about the two pinned files are outside any pin of their text,
so this revision guards them where they stand (the fifth read's M2,
answered in "M2. The pins held two files"):
the declaration that makes `root_mode_names.rs` the compiled module,
the manager's own name the module's first word is taken from,
and the `[[bin]]` `path` that makes `main.rs` the program.

### M1. The shared stripper was blind inside strings, and the lint pin's needles were not squeezed

The fifth revision moved `code_lines` into `source_guard/mod.rs` and had the scan and the
lint-attribute pin read through it.
That stripper opened a block comment at any `/*` outside a `//` line,
including one inside a string literal,
and cut a line at any `//`,
including one inside a string.
A glob string therefore opened a comment nothing closed,
and the rest of the file was invisible to every rule that trusted the stripper.

Measured this round over the 1,472 release files
(run keep/stripper-over-the-release-tree.log):

- three files end inside a comment that stripper never closed,
  and 794 non-blank lines of theirs are lost to every reader:
  424 real code and 370 comment-only.
  The strings that open it are `"refs/remotes/*/"` in `command/rule_branch_worktree.rs`,
  `"--exclude=refs/cli-git/kept/*"` in `kept_references/kept_references_reading.rs`
  and `"refs/heads/*"` in `push/manual_push_probe.rs`;
- the fifth read's planted variant shows what that buys an editor:
  an `#[allow(clippy::disallowed_methods)]` and a `std::env::args_os()` read appended to
  `kept_references_reading.rs` pass both the scan and the lint pin,
  and the same text placed before the glob string fails both
  (replay the fifth read's `keep/guards-replay-this-read.log`);
- the lint pin's needle was `clippy::`, matched against the line as written,
  so `#[allow(clippy ::disallowed_methods)]` (one space before the `::`) passed it
  while Clippy accepted the allow.

The repair is the stripper's own reading,
not a wider ban:

- `code_lines(source, what)` now tracks what it is reading:
  code,
  a block comment at its nesting depth,
  a `"..."` or `b"..."` string,
  an `r"..."` or `r#"..."#` raw string at its hash count,
  or a `'...'` character literal,
  and it decides between a character literal and a lifetime the way Rust does
  (an escape or a closing quote after one character).
  It keeps every byte of a literal, because the pins compare the line as written;
- it fails with `what` when the source ends inside a comment, a string or a character literal.
  A file that really ends there does not compile, so a source that trips this is a source the
  stripper misread,
  and the message names it: the scan reads 1,472 files through this one function;
- the scan's own reader is the wrapper it printed no code for in the fifth revision:
  it numbers the shared stripper's lines,
  drops the empty ones,
  and joins each to the next line that holds code,
  which is what its two-text rules read;
- the lint-attribute pin's needles are matched against the line with its whitespace removed,
  through the same `squeezed` the scan's rules use, so the two cannot disagree again.

What the honest editor pays for this is nothing:
the stripper's reading changes only for text inside a literal,
and neither pinned file holds a `//` or `/*` inside one
(the whole-line assertion in "The hook guard on `main.rs`" still holds both files to comments on
their own line).

The repair the fifth read offered as the smallest was this one;
the alternative is worth stating because it is cheaper to read and would cost the tree far more:
if the guards instead required every release file to hold comments on their own line and no block
comment at all,
`code_lines` would need no lexer,
and 63 release files hold `/*` somewhere while 9 release lines hold `//` after non-blank text
(8 of them inside a string literal, and one a real trailing comment in
`landing/service_amend_replayed.rs`)
(run keep/stripper-over-the-release-tree.log).
That is 63 files to reword before the guard could pass,
and it forbids `//` inside a string in release code for good,
which the project's own rule about comments (rule TD4, `AGENTS.md`) does not ask.

### M2. The pins held two files, and nothing held the wiring that made them the compiled text

Both pins read their file's text.
Nothing read the declaration that makes `root_mode_names.rs` the compiled module,
the `[[bin]]` `path` that makes `main.rs` the program,
or `COMMAND_WORD.current`,
which is the words' first word and lives in `foundation/wrapper_names_old_and_new.rs`.

The fifth read ran the variant:
a `#[path = "elsewhere.rs"] pub mod root_mode_names;` line in
`worktree_clone_mount_handler/mod.rs` left both pinned files byte-identical,
and a start under a name the pinned module would refuse decided a root mode.
That is the one new variant it ran against the pins.

Three assertions close it,
each in the guard that already owns the question,
and the folder's declaration file holds no `#[path]` attribute today:

- `root_mode_name_tests.rs` reads `worktree_clone_mount_handler/mod.rs` and requires exactly one
  `pub mod root_mode_names;` line and no `#[path]` attribute in that file,
  so the pinned text is the text the crate compiles;
- the same test asserts `COMMAND_WORD.current` is `monochromatic-git-policy-cli`,
  because the pinned module takes the first word from that constant and the constant is in a file
  no pin covers;
- `release_bins_tests.rs` reads each `[[bin]]` table's `path` as well as its name and requires the
  wrapper's to be `src/native/main.rs`,
  so the pinned entry is the entry the release builds.

Cost to the honest editor:
an edit to any of those three lines fails a guard,
which is right,
because each one decides what runs,
and none of them is expected to move often.

### M3. The printed scan rules failed the crate's own `shadow_reuse`

`package/git-policy/cli/Cargo.toml` sets `shadow_reuse`, `shadow_same` and `shadow_unrelated` to
deny,
and the crate's Clippy task runs with `-D warnings` over test code.
The fifth revision's two printed rules both began
`matches: |line, _joined| { let line: String = squeezed(line); ... }`,
which shadows `line` inside its own arm.
The fifth revision said its Rust was never compiled, and that was true.

The repair is the variable's name:
`squeezed_line`, in both rules.
The replayed test passes and the printed text now compiles and passes Clippy under the crate's own
lint table and the two rows this design adds (run keep/clippy-printed-texts-sixth.log).

### M4. `main` moved after the fifth revision merged, so the printed entry and its pin were stale

`558bd72198` (committed 19:41:33, merged as `ac6fce8e3b` at 21:03:54, after the fifth revision merged
at 20:53:00) gives `StartDirectory::read_from_the_process()` two lines in `main.rs`,
passes a third argument to `entry::run_process`,
and adds a module to `source_guard/mod.rs`,
which this design's change list edits.
`main.rs` is 27 code lines now, and `run_process`'s signature moved from `entry.rs:342` to `:356`.

- The printed `main.rs` is that file at this round's base with this design's block in it,
  47 code lines,
  and `MAIN_CODE` is its code text;
- the status sentence now names the commit it is true at, and says what a later move costs:
  a commit that touches `main.rs` before this design lands changes the pin in the same commit,
  which is the maintenance cost the pin's own measurement states;
- every `entry.rs` line this document cites is this base's
  (`run_process` `:356`, `runtime_probe::is_requested` `:362`, `hooks_off::activate` `:366`,
  the lock pass `:371`, `process_resolution_inputs` `:372`,
  `refusal_when_started_as_git` `:374`),
  and the change list says the commit that lands the text re-reads `main.rs` first.

### M5. `mount-path-descendant`: what agreed, what did not, and the order it now runs in

What the fifth revision said that agrees with this host and the records:
`zpcachyos/nosnap/vm-builder-output` has mountpoint
`/home/user/Monochromatic/package/vm-builder/cli/output`,
is mounted,
is `aes-256-gcm` with `keystatus=available`,
and is in pool `zpcachyos`;
the checkout is not a dataset today
(its mount is `/home`);
so the strictly-under rule,
the same-pool rule and the `none`/`-`/`legacy` refusal all pass the one dataset the step 19 record
names,
and the unit line is the one that record and its design give
(run keep/host-facts-sixth.log).

What did not agree, and what this revision does about each:

- the `mountpoint-unusable` clause refused a mountpoint that "is not root's".
  This host's mountpoint folder is `user:user 755`,
  and so is every parent of it,
  because the folder is the root of the mounted dataset and the plan's migration copies with
  "the copy that preserves ownership" (run keep/host-facts-sixth.log,
  source doc/planning/cli-git-zfs-worktree-implementation-plan.md:2591).
  The clause would therefore have refused the one dataset at every boot and left it unmounted,
  since a unit that refuses has no fallback.
  It is replaced by the route's own discipline:
  the mode opens the mountpoint's components beneath the checkout's mount,
  one descriptor at a time, refusing a link (the walk `clone_slot_walk.rs` already uses,
  source package/git-policy/cli/src/native/worktree_clone_mount/clone_slot_walk.rs:38
  and `subvolume/fs_beneath_walk_chain.rs:140`),
  and places the mount on the opened folder itself with `MOVE_MOUNT_T_EMPTY_PATH`,
  which is the primitive `attach_the_clone` uses for a clone
  (source package/git-policy/cli/src/native/worktree_clone_mount/attach_the_clone.rs:357).
  Ownership is then not the safety property,
  and the account owning its own mountpoint stops being a refusal;
- the mode compared a canonicalised text and then `mkdirp`ed a path.
  It now says in which order it proves what,
  and every step that acts acts on a descriptor (the section "What it does");
- nothing proved, before the mount, that the recorded checkout dataset is mounted at its recorded
  path now.
  "The record agrees with the kernel" is not "it is mounted",
  and the mode now refuses `checkout-not-mounted` when the mount table does not show the recorded
  dataset at the recorded path,
  which is what the step 19 acceptance list needs when a mount is delayed or fails;
- three of `zfs mount`'s refusals were missing,
  and the mode now refuses each by name:
  `canmount=off` (upstream lib/libzfs/libzfs_mount.c, zfs-2.4.4, `:299`),
  `zoned=on` in the global zone (`:270`),
  and a non-empty mountpoint when `overlay` is off (`:521` to `:538`,
  the "directory is not empty" refusal at `:538`),
  plus `redacted=on`, which `zfs_is_mountable` also refuses (`:305`);
- the key refusal was worded for every dataset.
  Upstream tests `encryption` first and then `keystatus` (`:455` to `:498`),
  and an unencrypted dataset's `keystatus` is `-`,
  so `encryption-key-not-loaded` now applies only to an encrypted dataset whose key is not
  `available`;
- "one option for each" of seven properties was wrong for this dataset:
  `zfs_add_options` adds the `relatime`/`strictatime` pair only when `atime` is not off
  (upstream `:365` to `:369`),
  so a dataset with `atime=off` resolves to six pairs,
  which is this host's case (`noatime,dev,exec,rw,suid,nonbmand`).

What it does to the one nested dataset at boot,
after the migration and the record:
the dataset is not mounted at boot (`canmount=noauto`),
every proof passes on this host's table,
the mountpoint is opened by descriptor beneath the checkout's mount,
and the mode mounts it with `noatime` and the five default pairs.
The refusal the old ownership clause would have produced is gone.

### M6. Choice 9 agreed with itself nowhere, and the rollback paragraph cited the plan for what the plan does not say

- Choice 9 said "options A, C and D leave one command whose word or GUID the boot's own refusal
  prints", while its own option C said the refusal names no repair.
  Option C's repair is the ordinary command, which the boot's refusal does not name,
  so the window lasts until someone re-runs it;
  only A and D leave one command the refusal prints.
  The sentence now says so,
  and the window length is a pair reason in the ranking;
- the rollback paragraph said the plan's step 19 rollback "puts that unit back to
  `/usr/bin/zfs mount <dataset>`" and labelled it (source).
  The plan's step 19 offers only "the tested rollback" of the checkout migration
  (source doc/planning/cli-git-zfs-worktree-implementation-plan.md:2599),
  and its step 19 design mentions no rollback of the nested dataset's unit at all
  (`rg -n -i rollback` over that design finds nothing).
  Restoring the unit's earlier line and removing the record first is this design's requirement,
  so the paragraph is labelled (design) and names the plan for the migration only;
- the reason given for restoring `canmount=on` ("so the line it swaps back has a dataset to mount")
  does not hold: an explicit `zfs mount` mounts a `noauto` dataset,
  and with `canmount=on` the step 19 unit's `ConditionPathIsMountPoint=!` skips it when the
  provider mounted first (source
  doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md:974 to :1000).
  Restoring `on` returns the dataset to the state the setup changed,
  which is what a rollback of that setup does;
- `--replace` was described with a "nineteen-digit" GUID.
  This host's 14 datasets answer GUIDs of 18, 19 and 20 digits
  (one of 18, six of 19 and seven of 20, run keep/host-facts-sixth.log),
  and the fifth revision's own token was the record's old GUID,
  not the dataset being admitted;
  choice 9 now weighs both and prints the digit range it measures.

### M7. Choice 11's control was a bare word, and the change list was silent about the starts that already pass a mode word

The printed raised-deadline control was
`CLI_GIT_TEST_ONLY_START_PROBE=hold-move CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=60000 <that build> start-probe`,
which under this design is Git or a refusal,
never the probe:
under the ZED file's name a bare word is refused,
and under any other name a word-free start is the wrapper
(replay the fifth read's `keep/choices-2-6-8-9-11-decisions-measured-on-the-printed-module.log`).
The control now carries the words,
as every root-mode start must,
and so does every existing start that passes a mode word,
which the change list now says.

Two statements in the change list were silent where they should not be:

- the ten test sites that start the built program each gain the words at the sites that pass a mode
  word (`adopt-worktree-parent` at `binary_zfs_adoption_guest_tests.rs:118`,
  `holder-scan --dataset` at `binary_zfs_holder_scan_guest_tests.rs:290`,
  `start-probe` at `binary_zfs_sweep_guest_tests.rs:245`),
  and only the starts that pass no argument stay as they are
  (the sweep at `binary_zfs_sweep_guest_tests.rs:230`,
  the unmount start at `binary_zfs_unmount_guest_tests.rs:238`,
  and the four starts at `binary_zfs_step9_m_arms_tests.rs:200` and `:561`
  that name no mode word);
- `HANDLER` in `e2e/start-probe-guest.ts:42` is a bare path today,
  and it becomes that path plus the words,
  so every `${HANDLER} start-probe` line keeps its shape,
  including the two `pkill --full "^${HANDLER} start-probe$"` patterns at `:415` and `:447`.

### The Low findings

All thirteen hold against this round's base and this host,
and each is answered where it lives:

- 31 of 45 `(run …)` labels named logs of earlier rounds.
  Ten were the fifth round's own and keep the label;
  the rest now read `(replay …)`,
  and every log they name is in the round folder the appendix names;
- the ledger text said the fourth revision's lists "failed" seven `main` and five module variants,
  which they passed,
  and called the sixth module variant a control that must pass,
  while the log and "The one-name guard on the names module" say it fails the comment rule.
  The sentence now says what the fifth read's `guards-replay-this-read.log` shows:
  the lists passed the variants that were not changed lines and failed every changed one,
  which is exactly the count the fifth revision's own paragraph gives;
  `nm6`, which was missing from that replay, is rebuilt on the sixth revision's text and fails
  the pin (run keep/guards-replay-sixth.log);
- the warm-pair Low holds and the fifth revision was wrong to deny it:
  the sentence set the pair's "120 of the pair's 120" against the control's "66 of its 119".
  Same-basis pairs are 120 of 120 against 67 of 120,
  or 119 of 119 against 66 of 119
  (replay keep/warm-reanalysis-fourth.log, lines 11, 13, 17 and 19);
- the citations that landed on doc lines are fixed:
  `handler_install_paths.rs`'s two constants are at `:38` (`ZEDLET_INSTALL_PATH`) and `:42`
  (`CONFIGURATION_PATH`) at this base,
  `attach_the_clone.rs`'s three calls are at `:257` (`fsopen`), `:273` (`fsconfig` source) and
  `:283` (the create),
  and `process/thread_quiet.rs` reads `/io` at `:68` and `/stat` at `:93`.
  `lib.rs`'s module line is `:476` at this base, `:457` at the fifth read's;
- two relative pointers remain in quoted logs and are left as the logs print them;
  every one in this document's own prose now names a heading, a tag or a path.
  The bold-label bullets in "The conflicts the pair carries" are headings now;
- the residual list is completed:
  a release file can still read the start name through `std::env::argv[0]` itself,
  `/proc/<own pid>/cmdline` and `/proc/self/task/<tid>/comm`,
  glibc's `program_invocation_name` and `program_invocation_short_name`,
  and `getauxval(AT_EXECFN)`,
  and none of those names a scan needle or a typed ban
  (replay the fifth read's `keep/other-ways-to-read-the-start-name-no-guard-names.log`).
  `/proc/<pid>/cmdline` stays out of the scan for the reason the kernel command-line read gives,
  and the pin on `main.rs` is what closes the hole where the decision is made.
  `/proc/self/stat` and `/proc/self/status` do carry the process name,
  `/proc/self/statm` does not;
- the three separability sentences that were false under the whole-file pin are corrected in
  choices 3, 4 and 10:
  a block that moves inside `main` is a changed pinned line,
  so option B of choice 4 and option B of choice 10 each change the pin,
  and choices 3's B and C need the words in `doctor` and in step 19's guest arm;
- choice 4's option D cited the retired guard's "three vector lines".
  It cites this design's own pin now:
  `main` names the vector on one line, and the pinned text is what a fourth line fails on
  (run keep/guards-replay-sixth.log);
  the same correction removes the sentence that today's `main` fails a guard "on four vector
  lines";
- the two spellings of the ZED file's name are tied by one assertion:
  `ZEDLET_INSTALL_PATH`'s file name must be `ZEDLET_START_NAME`,
  in `root_mode_name_tests.rs`;
- the change list keyed on the second program's name alone.
  It now names the sites that count executables without naming it:
  `SPEC.md:251`, `:262` and `:290`;
  `README.md:14`, `:1921`, `:1928`, `:7333`, `:8202` and `:8448`;
  `bin/native-install-steps.ts:96`; `bin/native-install.ts:443`;
  the doc of `source_guard/release_bins_tests.rs`; and plan lines 234 to 247, 3442 to 3448 and
  3521;
- `--replace` with every field already equal is now stated:
  it answers 0 with the `already recorded` line and writes nothing,
  because a replace that changes nothing is the ordinary command's no-op;
- the `(warnings` needle stays and its reach is stated:
  `#[allow(warnings)]` cannot excuse `disallowed_methods`, which is `deny`
  (replay the fifth read's `keep/clippy-planted-release-variants-with-the-proposals-rows.log`),
  so the needle is there because the pin's claim is every line that names a lint, a group or an
  inner attribute, and a reviewer sees that line;
- the fifth revision's claim that `/proc/self/stat` and `/proc/self/status` carry the name is
  true, and `statm` does not, which the table in "The residual" now says.

### The two rankings this revision changes

Choice 5 is re-weighed by what each option delivers, and it now ranks C > A > B.
The fifth revision ranked A first because A keeps the expected line in four existing readers and
keeps it stable when choice 2's word is renamed.
Those are upkeep and fidelity to what exists,
which the repository's ranking rule (quoted in choice 5) says to state as a cost and never to
rank by.
On delivery, C is the only line that spells something a person can type,
so it is the only one that names a program that exists,
and it makes the refusals' opening words and the version line one spelling.

Choice 9 gains one option it did not weigh,
offered by the fifth read and no record:
a token that names the dataset being admitted.
Weighed by what it delivers, it is the only token that fixes the record written to the dataset the
operator's command named,
and it ranks first (choice 9).

The remaining nine rankings are the fifth revision's,
and each is restated with a reason for every adjacent pair in its own section.

### What this revision left as the fifth revision wrote it

The pins' shape,
the scan's needles and the lint-attribute pin's set,
the two new modes' words and refusals,
the four option blocks of choices 1 to 4 and 6 to 8,
and the readings of ZED, systemd and the kernel are the fifth revision's,
and this revision changed only what the findings above name.
The one dataset, its mountpoint and its properties are re-read here,
so every number in the mode's section and in "What else the modes keep" is this round's.

## What the decision requires

The one-program record answers "One program only,
ever",
and its "What follows" is the requirement this design starts from
(record doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md):

- the release holds one executable,
  and the root modes
  (`boot-reconcile`,
  `shutdown-unmount`,
  `start-probe`,
  `adopt-worktree-parent`
  and the no-argument sweep the ZED hook makes)
  become modes of `monochromatic-git-policy-cli`;
- step 5's release changes are undone
  (the second `[[bin]]`,
  the release table rows,
  the binstall row,
  the lock's `bins` row
  and the archive's second program);
- the wrapper still links no OpenZFS library and starts where OpenZFS is absent;
- how root reaches the modes,
  what the ZED hook and the units name,
  and how a root mode keeps the wrapper's Git,
  LFS and network paths from running,
  are this design's to propose,
  and the start-name record has since answered the first part of that.

The record's list is four modes and the sweep.
The grammar of the program this design folds in has five modes,
`holder-scan` beside the four
(source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_arguments.rs:41),
and `holder-scan` comes along unchanged.
Two more modes are added by the answers since,
which is the count this design's change list carries:
seven modes and the sweep.

The start-name record's answer has two clauses,
and everything in choice 1 follows from the pair
(record doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md):

- only the zedlet's file name selects a root mode,
  compared as the whole last path component,
  and it is the only exception to the ban on reading a start name;
- the boot and shutdown units,
  adoption and a person reach the root modes by words in the wrapper's own namespace,
  as `service run` does.

What makes the problem hard is unchanged:
the wrapper is the `git` a person runs,
so its arguments are Git's arguments,
and an argument alone cannot mean "root mode" without taking a word away from Git.
And ZED hands a zedlet no arguments at all.

## What the code holds today

### The two entries

`package/git-policy/cli/src/native/main.rs` is the wrapper's entry.
It collects the arguments once,
skipping the program name (`main.rs:52`),
answers the service mode when the leading words are
`monochromatic-git-policy-cli service run` (`main.rs:56`,
`invocation::service_main::is_service_start`),
installs the payload-free panic hook,
collects the environment,
and hands everything to `invocation::entry::run_process` (`main.rs:76`),
then waits for the log lines it handed to their own threads
(`end_after_the_log`).
Every line number here is re-read at this round's base
(`5f904d9fbd`,
whose `main.rs` a commit after the fifth revision's base changed)
(source package/git-policy/cli/src/native/main.rs).

`package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs` is the second program's entry,
deleted by this design:
`args_os().skip(1)` (`:49`),
`worktree_clone_mount_handler::run`,
the same two log waits,
and the exit.
Its grammar (`worktree_clone_mount_handler/handler_mode_arguments.rs:319`) takes
one leading word out of the five `MODE_WORDS` (`:41`),
`--help` and `--version`;
an empty argument list is `AttachingSweep`;
anything else is a refusal that names the word and the modes (`:223`).
`handler_mode_dispatch.rs:317` to `:338` gives every mode the same two answers,
a line and status 0 (`EXIT_DONE`,
`:40`),
or a refusal line and status 2 (`EXIT_REFUSED`,
`:45`),
and reads the compatibility gate before a mode that will make a request
(`run_behind_at` at `:317`).
`main.rs:56`, `:57` and `:76` read the arguments after the program name,
which this design hands them as one slice,
and `main.rs:52` collects that vector with `.skip(1)` today,
which this design replaces with the whole vector and one `handed` slice
(the section "The rule, in the entry").
`handler_install_paths.rs:28` holds the program's name,
`:32` holds the root-owned installed path,
`:38` the ZED event's path,
and `:42` the configuration path.

The service mode is the precedent this design follows,
and the answer says so:
a mode chosen inside `main`,
before the panic hook,
before the environment is collected,
and before anything of the wrapper's own start runs.
It differs from the words path in three ways this design must state,
because they are what a reader will compare:

- `service run` also answers the earlier command word `cli-git`
  (`service_main.rs:74`,
  `EARLIER_SERVICE_WORDS`);
  the root words answer only `monochromatic-git-policy-cli`,
  because no earlier release has a root-mode unit to keep working;
- `service` needs a third word,
  `run`;
  so does the words path need a mode word,
  which is why `zfs-root` alone refuses (choice 1);
- `service` has no no-argument form at all,
  while the installed name's no argument is ZED's own start.

### The release's lists, and what reads them

`package/git-policy/cli/Cargo.toml:35` to `:48` declares two `[[bin]]` tables,
and its `[package.metadata.binstall]` table (`:56` to `:58`) names `{ bin }` (`:58`),
so one archive unpacks both programs.
Every list that counts them must agree with the manifest,
and `bin/native-release-bins.unit.test.ts` is the guard that says so:
the release job's `cargo build --bin` flags (`.github/workflows/cargo-publish.yml:2666`)
and its `bin=` copy lines (`:2695` to `:2698`),
`package/crate-release/cli/releasable-crates.json:30`'s `checkBins` for the crate,
`toolchain.lock.json`'s `bins` row for `cargo:monochromatic-git-policy-cli`,
and `src/native/source_guard/release_bins_tests.rs:31`.
`toolchain.plan.jsonc` holds the same row as intent (`:271` to `:274`),
and `toolchain.lock.json` is written from the plan:
`mpm-shadow`'s update keeps every other row as it is and gives it "the plan's current bins",
so a changed plan row moves the lock row through the tool,
offline (`toolchain:lock:check` runs
`install --lockfile-only --frozen-lockfile`).
The crate's `pinFiles` row holds only the fuzz crate,
so the decision's "pinFiles row" has nothing left to undo
(source package/crate-release/cli/releasable-crates.json:29).

`bin/native-install-steps.ts:104` holds `handlerProgram` and `:121` `handlerVersionLine`.
`bin/native-install.ts:448` runs the handler by bare name through `PATH`
and `:458` refuses unless its `--version` line names the release.
`bin/stand-in-install-programs-for-tests.ts:169` to `:186` writes the stand-in handler those tests install,
and `:335` lists it beside the stand-in wrapper.

Six Rust test files start the second program by
`env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets")`,
at ten sites:

- `binary/binary_mount_git_worktree_clone_datasets_tests.rs:106`;
- `binary/binary_zfs_adoption_guest_tests.rs:118`;
- `binary/binary_zfs_holder_scan_guest_tests.rs:290` and `:892`;
- `binary/binary_zfs_step9_m_arms_tests.rs:200` and `:561`;
- `binary/binary_zfs_sweep_guest_tests.rs:230`,
  `:243` and `:875`;
- `binary/binary_zfs_unmount_guest_tests.rs:238`.

Each is a binary-level control under `src/native/binary/binary_tests.rs`.
`e2e/start-probe-guest.ts:42` names the built binary in its `HANDLER` constant
and its guest commands run `${HANDLER} start-probe`;
`e2e/mount-git-worktree-clone-datasets-guest.ts` drives the second program with the same built path.
`package/git-policy/cli/mise.toml:268` is the e2e task named for it.

### The ZFS code already in the wrapper, and the gate it runs

The wrapper already carries the ZFS code.
`Cargo.toml` depends on
`monochromatic-zfs-request-to-openzfs-kernel-device` for the typed `/dev/zfs` requests,
and the wrapper runs the same compatibility gate before its first request,
which plan step 7 added:
`worktree_clone_add_route/add_zfs.rs:69` calls `LiveGate.verdict()` inside `OpenedDevice::open()`,
reached from `add_preflight.rs:139` at this base
(`:113` at `f3d81e0910`,
where the file has changed since),
both release code.
The call arrived in `b01af8c3bd` (2026-11-23T20:54:55-05:00),
which was not yet merged into the line the first gate review read
(`c2701d58a5`,
2026-11-23T21:45:28-05:00),
which is why that review found no gate and why its `rg` was right for its commit.
So a root mode inside the wrapper adds no dependency and no library:
the decision's "links no OpenZFS library and starts where OpenZFS is absent" check stays as it is,
and `worktree_clone_mount_handler` is already `#[cfg(target_os = "linux")]`
(source package/git-policy/cli/src/native/lib.rs:476 and
:477,
whose paragraph is `:462` to `:475`;
the fourth revision cited `:429`,
which is another module's `#[cfg]` line at this base).

### The sentence and the two guards

Three places say the wrapper never reads its start name,
and the answer changes all three for exactly one name:

- `SPEC.md:19715`
  (`:19705` at the fifth revision's base, `:19261` at `f3d81e0910`),
  in "#### External hooks":
  "and the executable does not read the name it was started under".
- `src/native/source_guard/hook_self_invocation_tests.rs:74`,
  `main_does_not_dispatch_on_its_start_name`,
  which forbids `args_os().next()`,
  `start_name`,
  `current_exe`,
  `arg0`,
  `file_name`,
  `hook_dispatch` and `hook_entry` in `main.rs`,
  requires `args_os()` exactly once (`:80`),
  requires `args_os().skip(1)` (`:81`)
  and requires `entry::run_process(`.
- `src/native/source_guard/no_self_invocation_rules_tests.rs:26`,
  the `argv0-dispatch` scan rule,
  whose needles are `args_os().next()`,
  `args().next()`,
  `.nth(0)`,
  `argv[0]` and `arg0(`
  (`:29`).

Beside them,
`src/native/main.rs:53` explains the service mode as chosen
"never by the name this program was started as",
and `binary/binary_external_hook_tests.rs:174` proves that a start under each Git hook event's name behaves as `git`.
Both stay true for every name but the one,
which is what the guard change has to express.

The scan's `ALLOWED` rows are keyed by file and rule
(`no_self_invocation_rules_tests.rs`,
`unexplained` and `stale`),
so a row excuses every needle of its rule in its file.
That is why the argument-vector read needs a rule of its own (section "The guard change,
exactly").

### The wrapper's management help, which has no line for the words

`src/native/invocation/management_arguments.rs:46` holds `MANAGEMENT_USAGE` and `:59` `MANAGEMENT_HELP`,
and neither names a root mode.
A person who types `git monochromatic-git-policy-cli --help` today reads:

```text
Usage: git monochromatic-git-policy-cli <command> [options]

Run repository policies directly.

Commands:
  check    Check policies over an explicit scope.
  fix      Apply policy fixes over an explicit scope.
  service  status: whether the commit service answers (it sets itself up when a commit needs it).
  ...
```

and a person who types the words is answered by the management arm's refusal,
because the root-mode arm does not exist yet
(replay keep/today-lines.log):

```text
$ ./target/release/monochromatic-git-policy-cli monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
Usage: git monochromatic-git-policy-cli check (--all | -- <pathspec>...) [--policy <id>]...
       git monochromatic-git-policy-cli fix (--all | -- <pathspec>...) [--policy <id>]...
       git monochromatic-git-policy-cli doctor
       git monochromatic-git-policy-cli service (status | settle-earlier-batches | stop-units-of-removed-worktrees)
       git monochromatic-git-policy-cli kept-references [--prune]
       git monochromatic-git-policy-cli migrate-to-subvolume [--dry-run | --rollback | --remove-old]
       git monochromatic-git-policy-cli convert-worktree-to-subvolume [--dry-run | --abandon] (--all | <worktree>...)
       git monochromatic-git-policy-cli --help
exit=2
```

So the change needs one line in each of those two constants,
and the tests that assert them
(`invocation/management_arguments_tests.rs`,
and `invocation/entry_command_word_tests.rs:108` and `:113`,
which compare the usage line)
move with it.
Without that line the words work and nothing teaches them.

## What a launcher hands a start

### ZED runs a zedlet by path, with no arguments

From OpenZFS `zfs-2.4.4`
(upstream cmd/zed/zed_exec.c):

- `cmd/zed/zed_exec.c:141` builds `path` as `"<zedlet_dir>/<prog>"`;
- `:161` to `:166` point descriptors 0,
  1 and 2 at `/dev/null`
  when ZED runs in the foreground,
  which the host's unit does (`ExecStart=/usr/bin/zed -F`);
- `:167` puts the event descriptor on 3;
- `:168` is `execle(path, prog, NULL, env)`:
  the argument vector is the program name alone,
  and `argv[0]` is `prog`,
  the bare file name with no directory;
- `:421` matches a zedlet against the event class and requires a non-letter straight after it;
- `cmd/zed/zed_conf.c:373` to `:405` accept a file only when `stat` succeeds,
  it is a regular file,
  root owns it (unless `-f`),
  its owner execute bit is set,
  and it is not group- or other-writable;
- `cmd/zed/zed_conf.c:51` caps ZED at 16 concurrent zedlet jobs (`zcp->max_jobs = 16`);
- `cmd/zed/zed.c:238` to `:241` set `umask(0)` and then `chdir("/")`,
  so no zedlet ever starts in a repository.

`cmd/zed/zed_event.c:1007` to `:1022` build the child's whole environment,
and it is not inherited:
the event's nvlist,
a restricted list (`IFS`,
`PATH` from `_PATH_STDPATH`,
`ZDB`,
`ZED`,
`ZFS`,
`ZINJECT`,
`ZPOOL`,
`ZFS_ALIAS`,
`ZFS_VERSION`,
`ZFS_RELEASE`),
`TZ` when the parent has it,
`ZED_PID`,
`ZED_ZEDLET_DIR`,
`ZEVENT_SUBCLASS`
and the `ZEVENT_TIME_*` values.

This round's replay of that child step,
with a fixture writing down what it was handed
(from `keep/zed_replay.c` and `keep/replay_fixture.rs`;
replay keep/zed-shape-report.txt):

```text
PROBE argv0=Some("history_event-mount_git_worktree_clone_datasets")
PROBE last_component=Some("history_event-mount_git_worktree_clone_datasets")
PROBE argc=0
PROBE cwd=Ok("/")
PROBE env PATH=Some("/usr/bin:/bin:/usr/sbin:/sbin")
PROBE env HOME=None
PROBE env TZ=Some("UTC")
PROBE env IFS=Some(" \t\n")
PROBE env ZED_PID=Some("4242")
PROBE env ZED_ZEDLET_DIR=Some("/etc/zfs/zed.d")
PROBE env ZEVENT_SUBCLASS=Some("\"history_command\"")
PROBE fd0=/dev/null
PROBE fd1=/dev/null
PROBE fd2=/dev/null
PROBE fd3=~round/scratch/probes/names/fx3/zed.d/zevent.nvlist
PROBE decided=root-mode: the attaching sweep (no word)
```

The fixture's `argc=` counts the arguments after the program name,
so `0` is the empty vector `execle(path, prog, NULL, env)` builds;
the program still has one element,
its `argv[0]`,
the bare name with no directory.

Consequences a design cannot talk its way out of:

- a mode selector that needs an argument cannot be reached by the ZED hook,
  because ZED passes none;
- `argv[0]` is the bare file name,
  so a name rule must read the last path component and must work with no slash at all;
- ZED offers no way to add a variable of this repository's own to that environment,
  so a variable cannot select a mode;
- ZED starts a zedlet from `/`,
  so nothing a zedlet start does can be about a repository unless a mode goes and finds one.

`current_exe()` is not the name either:
under a symlink it answers the link's target,
so it cannot carry the installed name ZED passes.

### systemd runs what `ExecStart` spells

Measured this round on this host's user manager,
with a transient service and a transient scope
(replay keep/unit-argv0-fourth.log):

```text
Running as unit: zfs-root-modes-fourth-service.service
PROBE argc=2
PROBE argv[0]=/home/user/temp/agent/zfs-root-modes-fourth-revision-2026-11-24/scratch/names-probe/argv_view
PROBE argv[1]=boot-reconcile
          Finished with result: success

Running as unit: zfs-root-modes-fourth-scope.scope
PROBE argc=2
PROBE argv[0]=/home/user/temp/agent/zfs-root-modes-fourth-revision-2026-11-24/scratch/names-probe/argv_view
PROBE argv[1]=boot-reconcile
```

So a unit passes words,
and its `argv[0]` is the path as written,
which the one name rule reads.
A system manager's start is not measured here,
because that needs root;
the user manager and the system manager both read `ExecStart` with the same parser,
which is the one thing this round does not prove
(design).

### A caller can choose `argv[0]`, and not when ZED or systemd starts it

`exec -a` sets anything,
and ZED's `execle` and systemd's `ExecStart` fix the value.
A start whose `argv[0]` a lower-privileged caller chose as the zedlet's name is a root-mode start as that caller,
which is harmless for an account and is why none of this design's launchers passes a caller's choice through
(cases 26 and 34;
run keep/dispatch-cases.log).

### An empty `argv[0]` and a NULL vector are one empty argument

`execve` with an empty vector,
and `execve` with one empty element,
both reach the program as a single empty argument.
Measured on this kernel with `keep/exec_empty_argv.c` and `keep/argv_view.c`
(replay the fourth read's `keep/dispatch-cases-35-rerun-this-read.log`,
whose `argv_view` run reports argc 1 and an empty `argv[0]` for both vectors).
The dispatch probe's own case 35 is no evidence for this:
it takes the start name as its own second argument,
so with an empty vector the name under test is absent and it decides the wrapper for lack of a
name,
and a debug build of it panics on `collected.len() - 2`.
What the compiled module decides on both vectors is `keep/names-starts-fourth.log`'s,
printed by the start itself,
and it is the same answer for the same reason.

```text
    start: exec_empty_argv argc0 ./dispatch-probe   (execv with argv = {NULL})
PROBE decided=wrapper (the service check, then entry::run_process)
    start: exec_empty_argv empty0 ./dispatch-probe  (execv with argv = {"", NULL})
PROBE decided=wrapper (the service check, then entry::run_process)
```

The kernel inserts the empty element
(the `bprm->argc == 0` block of `fs/exec.c`,
upstream source),
so the design's rule decides both without indexing anything:
the last component of `""` is none,
which is not the zedlet's name,
so both are the wrapper.
ZED never does either:
its `execle` passes one element.

### The bytes must fit the launcher's libc

A root-owned file that a guest cannot start is no evidence,
so this design's measurements read the floor each program links against:
the wrapper needs at most `GLIBC_2.39` and the handler `GLIBC_2.34`
(`readelf --version-info`),
and the Ubuntu 24.04 guest the second revision measured in has 2.39,
so both host-built binaries start there
(replay keep/cold-start-analysis-rerun.txt).
That is what made a guest measurement possible at all in the second revision round,
where the first revision round's Fedora guest refused every route in.

## Choice 1: what selects a root mode

### The three rules probed

Each of the three rules carries the same answer to the second revision's Medium finding:
the words with no mode word after them refuse with the usage,
as `service` needs `run`,
so `zfs-root` alone never starts a sweep.
They differ in what a word does under the installed name.

Option A,
the rule this design recommends:

- the two words `monochromatic-git-policy-cli zfs-root`,
  followed by a mode word,
  select a root mode on **any** file;
- under the installed ZED history event's name,
  ZED's no argument is the sweep,
  and `--help` and `--version` are the modes' own answers;
- under that name,
  any other word is refused before the modes' grammar sees it,
  with the usage and status 2;
- every other start is the wrapper.

Option B:

- as A for the words;
- under the installed name only ZED's no argument selects a root mode;
- every word there,
  `--help` and `--version` included,
  is refused before the grammar.

Option C,
the first revision's rule:

- the installed name selects a root mode and the modes' grammar reads whatever follows,
  with the words pair dropped when it leads them;
- the words select a root mode on any file.

All three read the name as the whole last path component,
which `Path::file_name` answers,
and refuse with status 2.

### What each decides

`keep/dispatch-cases.log` (replay) is the three rules and nothing else,
from `keep/dispatch_probe.rs`,
run over thirty-five starts.
Cases 1 to 26 are the second revision's twenty-six and give its log's seventy-eight decisions,
line for line
(the two decision lists,
`keep/kept-first-78.txt` and `keep/mine-decided.txt`,
compared line for line,
replay).
The cases that decide the question:

- ZED's own child step,
  the bare name and no argument (case 1):
  A,
  B and C all decide the attaching sweep.
- the installed path,
  the words and `boot-reconcile`,
  which is a unit's `ExecStart` (case 3):
  all three decide `boot-reconcile`.
  Under A this works because the words are read before the name;
  a name-first reading of A's own refusal would refuse this line,
  which is what makes the order load-bearing.
- the installed path,
  a bare `boot-reconcile`,
  which step 19's earlier line and a person's guess use (case 4):
  A and B refuse before the grammar;
  C decides `boot-reconcile`.
- the installed path,
  `--help` and `--version` (cases 5 and 6):
  A decides the grammar's help and version;
  B refuses both;
  C decides both.
- the installed path,
  the words and no mode word (case 7):
  all three refuse.
- the installed path,
  the namespace word misspelled (case 8):
  A and B refuse the first word;
  C hands the grammar `monochromatic-git-policy-cli`,
  which the grammar refuses as no mode.
- the installed path,
  the mode word misspelled (case 9):
  all three hand the grammar `boot-reconcil`,
  which refuses it.
- the installed path with a trailing slash (case 10),
  doubled slashes (11),
  `./` before it (12)
  and `sub/../` before it (13):
  all three decide the sweep,
  because `Path::file_name` reads the last component the same way.
- the bare name through a `PATH` lookup (case 14):
  all three decide the sweep.
- five near-miss names (cases 15 to 19):
  all three decide the wrapper,
  and a contains test would have taken two of them and a suffix test one.
- a stray copy under another name with a bare word (case 20):
  all three decide the wrapper.
- a stray copy under the zedlet's own name in another folder,
  no argument (case 21):
  all three decide the sweep;
  with a bare word (case 22):
  A and B refuse,
  C decides that mode.
- the wrapper's own name with the words (case 23):
  all three decide the mode;
  with the words and no mode word (case 24):
  all three refuse.
- an adoption's three options through the words (case 25):
  all three decide `adopt-worktree-parent` with them.
- a caller-chosen `argv[0]` through `exec -a` (case 26):
  A and B refuse the bare word,
  C decides the mode.
- the two new modes through the words (cases 27 and 28):
  all three decide them.
- the record mode written under the installed name (case 29):
  A and B refuse the word;
  C hands the grammar `record-checkout`.
- the installed name reached through links named `git`,
  `monochromatic-git-policy-cli` and `mount-git-worktree-clone-datasets` (cases 30 to 33):
  all three decide the wrapper,
  because none of those names is the zedlet's.
- a caller-chosen `argv[0]` that is not the name,
  with the words and a mode (case 34):
  all three decide the mode,
  which is the words clause at work on a file no name selects.
- an empty vector and a vector of one empty element (case 35):
  all three decide the wrapper.

The real kernel starts of the compiled module agree:
the installed file under a link named `git` decides the wrapper,
and the same file under its own name refuses a bare word
(replay keep/names-starts-fourth.log).

### What each option decides, in one command each

Options A's and B's lines are the module's decision,
measured by starting the compiled module under the named `argv[0]`
(replay keep/names-starts-fourth.log;
option B's rule is the same module with only ZED's no argument accepted,
compiled and started the same way).
Option C's lines are the dispatch probe's decision for the first revision's rule,
which that probe models as its `name-wins` rule
(replay keep/dispatch-cases.log,
case 4).
The text a build prints is this design's (proposal).

Option A:

```text
$ <the installed file>
   decides: root-mode grammar=[]  (ZED's own sweep)
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   decides: root-mode grammar="boot-reconcile"
$ <the installed file> --version
   decides: root-mode grammar="--version"
$ <the installed file> boot-reconcile
   decides: refused word-under-name "boot-reconcile"
```

Option B:

```text
$ <the installed file> --version
   decides: refused word-under-name "--version"
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

Option C:

```text
$ <the installed file> boot-reconcile
   decides: root-mode grammar="boot-reconcile"
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

### Ranking of choice 1

Ranking:
A > B > C.

A over B:
both fail closed for the installed file,
and both let the words select a mode on any file,
which is the answer's second clause.
A answers `--help` and `--version` under the installed name,
so the release smoke's first arm and a person's check of the installed bytes are one command with no
words in them,
and that is a positive check of the name dispatch before any sweep runs.
B can prove the same dispatch with a refusal instead:
a start under the name with `--version` decides `refused word-under-name "--version"` in the compiled
module,
and the dispatch answers `EXIT_REFUSED`, which is 2,
where Git would print its version and exit 0
(replay keep/names-starts-fourth.log,
the section "option B's rule, compiled";
the status is the dispatch's own,
EXIT_REFUSED = 2,
source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs:45).
So A's gain is a positive check with two words fewer,
not the only proof;
B is the answer's literal text,
and its gain is a rule with one fewer arm,
which is a smaller rule,
not a stronger one.
A's cost,
disclosed,
is two more spellings under the name:
`--help` and `--version`.

B over C:
C accepts a third spelling the answer never names,
a bare mode word under the installed name,
and it drops the words pair there,
so under the recommended layout the words would not be what selects a mode at all
(the second revision's High finding).
C is also the only one of the three that decides a bare word under a stray copy of the zedlet's name (case 22),
where A and B refuse.

The choice is separable from the other ten in what it decides,
and not in what it edits.
If the owner picks B,
the module's decision function loses the two help words,
which is a change to `root_mode_names.rs` and to the pin that holds it,
and the release smoke's first arm becomes a refusal check.
If the owner picks C,
the words pair is dropped under the name and the unit lines still work,
at the price of the property stated in "B over C",
and the module and its pin change the same way.
Three more things move with B or C,
and the fourth revision's separability paragraph named none of them:

- `doctor`'s planned installed-file check (step 18), which reads the installed file's `--version`
  under the name: under A it is one word-free command, under B a refusal check, and under C the
  same word-free command as A;
- step 19's guest arm, which starts a unit by that path and reads `--version` through it,
  with the same three answers;
- the example commands of choice 3's option blocks and choice 5's option blocks,
  which show a start under the installed name with and without the two words.

The layouts,
the dispatch's placement and the pin's shape do not change under any of the three.

### The rule, in the entry

The whole argument vector goes to one module and one function,
so `main` reads no element and compares no name.
The file, whole,
which is the text "The hook guard on `main.rs`" pins line for line,
comments aside:

```rust
// package/git-policy/cli/src/native/main.rs
//! What:
//!  The thin native monochromatic-git-policy-cli executable.
//! Why:
//!  Process facts are gathered once here;
//!  every decision lives in the library.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exit(runProcess(process.argv.slice(2), Object.entries(process.env), startDirectory));
//! ```

/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments and environment values reach Git exactly as the caller gave them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.argv / process.env, but byte-preserving.
/// ```
use std::ffi::OsString;
/// The wait at the end of this process for the log lines it handed to their own thread.
use git_policy_cli::service_log_lines_off_the_calling_thread::{WAIT_BEFORE_AN_EXIT, write_what_waits};
/// The wait at the end of this process for the diagnostic lines it handed to their own thread.
use git_policy_cli::diagnostic_lines_off_the_calling_thread::write_what_waits as write_every_diagnostic_line_that_waits;
/// The one module that decides a root-mode start, in a Linux build only.
#[cfg(target_os = "linux")]
use git_policy_cli::worktree_clone_mount_handler::root_mode_names::{self, RootModeStart};

/// What:
///  The program entry point:
///  collect arguments and environment,
///  run,
///  exit.
/// Why:
///   `args_os`/`vars_os` never decode,
///  so non-UTF-8 arguments neither panic nor
///       change.
///  A forwarded command replaces this process and never reaches the exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function main(): never { const code = runProcess(argv, env, startDirectory); writeWhatWaits(WAIT_BEFORE_AN_EXIT); process.exit(code); }
/// ```
fn main() {
    // The one argument-vector read the owner's answer needs; `clippy.toml` bans the call in every
    // other file, and this allow is the one it names.
    #[allow(clippy::disallowed_methods)]
    let arguments: Vec<OsString> = std::env::args_os().collect();
    // The handler is the crate's Linux-only module, like the mount table it reads, so the root
    // modes exist only in a Linux build and every other target keeps the wrapper's one mode.
    #[cfg(target_os = "linux")]
    {
        match root_mode_names::root_mode_start(arguments.as_slice()) {
            RootModeStart::Wrapper => {}
            RootModeStart::Arguments(mode_arguments) => {
                git_policy_cli::panic_notice::install_payload_free_panic_hook();
                let code: i32 = git_policy_cli::worktree_clone_mount_handler::run(mode_arguments);
                end_after_the_log(code);
            }
            RootModeStart::Refused(misuse) => {
                git_policy_cli::panic_notice::install_payload_free_panic_hook();
                let code: i32 = git_policy_cli::worktree_clone_mount_handler::refuse_root_mode_misuse(misuse);
                end_after_the_log(code);
            }
        }
    }
    // The service check and the wrapper's own entry both read the arguments after the program name,
    // and the whole vector is what the root-mode decision reads, so the name is dropped once here.
    let handed: &[OsString] = arguments.get(1..).unwrap_or_default();
    // The service mode is chosen by the leading words (`monochromatic-git-policy-cli service run`), never by the name
    // this program was started as, and before anything else of the wrapper runs: the service's process setup is its
    // own, and the wrapper's start never takes it (doc/planning/cli-git-resident-service.md, "One executable").
    if git_policy_cli::service_main::is_service_start(handed) {
        let code: i32 = match git_policy_cli::service_main::service_options(handed) {
            Ok(options) => git_policy_cli::service_main::run(options.as_slice()),
            Err(word) => {
                git_policy_cli::service_error!("an argument of the service is not text: {}", word.to_string_lossy());
                git_policy_cli::service_main::EXIT_REFUSED
            }
        };
        end_after_the_log(code);
    }
    // The linked scanner's catch boundaries run after the panic hook, so a hook that never
    // prints a panic's message is installed before anything can scan.
    git_policy_cli::panic_notice::install_payload_free_panic_hook();
    // `vars_os` yields every environment entry as an undecoded name/value pair.
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    // The one read of the directory this command was started in: every reader of it is handed this value instead of
    //  asking the process again, so no second read can answer with a directory another thread of this process moved.
    let start_directory: git_policy_cli::start_directory::StartDirectory =
        git_policy_cli::start_directory::StartDirectory::read_from_the_process();
    // `i32` is the signed 32-bit type of process exit codes.
    let code: i32 = git_policy_cli::entry::run_process(handed, environment.as_slice(), &start_directory);
    end_after_the_log(code);
}

/// What:
///  Waits,
///  bounded,
///  for the lines this process handed to its log thread,
///  then ends it.
/// Why:
///   A line written by a thread of its own is not written yet when the exit arrives,
/// and `std::process::exit` ends that thread where it stands:
/// the refusals of 2026-11-14 printed no byte at all in most runs,
/// so a person saw a status and no reason (`service_log_lines_off_the_calling_thread.rs`).
/// The bound keeps the promise a stream nobody drains was given in the first place:
/// no caller waits for the reader,
/// and the end of the process waits at most this once.
/// Nothing is started or waited for when no line was ever handed over,
/// which is every ordinary wrapper command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endAfterTheLog(code: number): never { writeWhatWaits(WAIT_BEFORE_AN_EXIT); process.exit(code); }
/// ```
fn end_after_the_log(code: i32) -> ! {
    // What: whatever each writer thread has not reached its stream yet, waited for once each.
    // Why:  a diagnostic and a service log line are both written by a thread of their own, and a process that ends at once
    //       would lose the last of them; each wait is bounded by the same figure, and a process that said no line starts no
    //       thread and waits for nothing.
    let _diagnosed: bool = write_every_diagnostic_line_that_waits(WAIT_BEFORE_AN_EXIT);
    let _written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);
    std::process::exit(code);
}
```

`src/native/worktree_clone_mount_handler/root_mode_names.rs` is the one place the name,
the words and the grammar's argument list meet.
It holds three things:

- the constant `ZEDLET_START_NAME`,
  the installed ZED history event's own name;
- the constant `ROOT_GROUP_WORD`,
  choice 2's word;
- the function `root_mode_start`,
  which answers one of three things:
  the arguments the grammar reads,
  a misuse it refuses before the grammar,
  or the wrapper.

The group word compares against `foundation/wrapper_names_old_and_new.rs`'s `COMMAND_WORD.current`
and never against a literal,
so the program's own word stays in one constant.
The folder already links `foundation/` in its `uses/` set,
so no link is added,
and the folder's own top doc gains the file first,
before `handler_install_paths.rs`,
because it uses only `foundation` and the standard library.

The module,
whole,
which is the text "The one-name guard on the names module" pins line for line,
comments aside:

```rust
// package/git-policy/cli/src/native/worktree_clone_mount_handler/root_mode_names.rs
use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;

/// The installed ZED history event's own file name: the one start name that selects a root
/// mode, compared as the whole last path component.
pub const ZEDLET_START_NAME: &str = "history_event-mount_git_worktree_clone_datasets";

/// The namespace word the root modes sit under, beside `service`.
pub const ROOT_GROUP_WORD: &str = "zfs-root";

/// What a start under the root modes' own spellings asked for and cannot have.
pub enum RootModeMisuse<'arguments> {
    /// The root-mode words with no mode word after them.
    NoModeWordAfterTheGroupWord,
    /// A word under the installed name that is neither ZED's none nor `--help` nor `--version`.
    WordUnderTheZedletName(&'arguments OsStr),
}

/// What one start is.
pub enum RootModeStart<'arguments> {
    /// A root-mode start, with the words the modes' grammar reads.
    Arguments(&'arguments [OsString]),
    /// A start refused before the grammar, so no mode runs and Git is never reached.
    Refused(RootModeMisuse<'arguments>),
    /// The wrapper, unchanged.
    Wrapper,
}

pub fn root_mode_start(arguments: &[OsString]) -> RootModeStart<'_> {
    let (start_name, handed): (Option<&OsStr>, &[OsString]) = split_the_name(arguments);
    if starts_with_the_group_words(handed) {
        let mode_arguments: &[OsString] = &handed[2..];
        if mode_arguments.is_empty() {
            return RootModeStart::Refused(RootModeMisuse::NoModeWordAfterTheGroupWord);
        }
        return RootModeStart::Arguments(mode_arguments);
    }
    if !names_the_zedlet(start_name) {
        return RootModeStart::Wrapper;
    }
    if handed.is_empty() || (handed.len() == 1 && (handed[0] == OsStr::new("--help") || handed[0] == OsStr::new("--version"))) {
        return RootModeStart::Arguments(handed);
    }
    return RootModeStart::Refused(RootModeMisuse::WordUnderTheZedletName(word_that_cannot_be_honoured(handed)));
}

fn starts_with_the_group_words(handed: &[OsString]) -> bool {
    return handed.len() >= 2 && handed[0] == OsStr::new(COMMAND_WORD.current) && handed[1] == OsStr::new(ROOT_GROUP_WORD);
}

fn names_the_zedlet(start_name: Option<&OsStr>) -> bool {
    return start_name.and_then(last_component) == Some(ZEDLET_START_NAME);
}

fn last_component(start_name: &OsStr) -> Option<&str> {
    return Path::new(start_name).file_name().and_then(OsStr::to_str);
}

fn split_the_name(arguments: &[OsString]) -> (Option<&OsStr>, &[OsString]) {
    let Some((start_name, handed)) = arguments.split_first() else {
        return (None, &[]);
    };
    return (Some(start_name.as_os_str()), handed);
}

/// The first word this start cannot honour: `--help x` and `--version x` name `x`, because the
/// first word alone is accepted and the second is what the name does not allow.
fn word_that_cannot_be_honoured(handed: &[OsString]) -> &OsStr {
    let first: &OsStr = handed[0].as_os_str();
    if handed.len() > 1 && (first == OsStr::new("--help") || first == OsStr::new("--version")) {
        return handed[1].as_os_str();
    }
    return first;
}
```

with its `//region` markers as the folder's other files carry them,
a `root_mode_names_tests.rs` beside it covering the table in "What each decides",
and with `refuse_root_mode_misuse` in `handler_mode_dispatch.rs`,
where every other refusal is written:
it says the misuse and then the usage,
and answers `EXIT_REFUSED`.

A refusal that names the whole start,
because a person who typed `--version x` reads why:

```text
$ /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets --version x
`x` is an argument this start does not take: under this file's own name only no argument,
`--help` and `--version` are answered. Nothing was done. Ways forward: run the file with no argument for
the clone-mount sweep, with `--help` for the modes' usage, or with `--version` alone; every mode is reached
through the words, as `monochromatic-git-policy-cli zfs-root <mode>`.
```

The words of this refusal are this revision's,
and no build prints them yet (design).

The service check follows the block,
so the order of the three checks is:
the words,
the one name,
the service's words.
That order is load-bearing twice over:
the words are read before the name,
which is what lets a unit's line work at the installed path,
and both are read before the service's words,
which is what refuses `monochromatic-git-policy-cli service run` under the zedlet's name
(the `service words` line;
run keep/names-starts-fourth.log).

### `zfs-root` alone refuses with usage

The words path needs a mode word,
so `git monochromatic-git-policy-cli zfs-root` is a refusal that the usage follows,
not a sweep.
Measured today,
the same line reaches the management arm,
which answers its usage and status 2,
and writes a lock-holder record into the repository it stands in
(replay keep/today-lines.log);
the second revision's `keep/probe-install-layouts.log` counted the record per start);
with this change it is the root-mode arm's own refusal,
and its text names the modes and the installed path.

Only `start-probe` and `adopt-worktree-parent` check the effective user today
(`handler_mode_dispatch.rs:174` and `adopt_worktree_parent_mode.rs:315`),
and the third revision's claim that before this change a person who typed the group word and forgot
the mode started a real sweep as root described the second revision:
today the words reach the management arm,
which refuses them and writes a lock-holder record
(replay keep/today-lines.log).
The refusal costs nothing a sweep needs:
ZED's start is the no-argument one,
and a unit always names its mode.

### What the name selects, and what the words select

The answer's two clauses are kept apart in the design like this:

- the words select a root mode on any file,
  which is what a unit,
  an adoption and a person use;
- the installed name adds ZED's no-argument sweep,
  and `--help` and `--version`,
  and turns every other word at that path into a refusal.

So under the recommended install layout (choice 3's option A)
both spellings reach the modes,
and the words remain what selects a mode:
a unit line works whether the file it starts is the installed name or another,
and the same unit line works against a stray copy.
That is the property the answer needs and the first revision's pair-dropping did not have.

### The guard change, exactly

Four guards change or arrive,
and the shape is the fifth revision's:

- the two files that decide what a start is are pinned whole,
  so any change to either is a change to its guard;
- the scan keeps needles,
  because a pin per file over 1,472 release files is not maintainable;
- a pin holds the release tree's lint attributes,
  because one `#[allow(` line can excuse a typed ban in a file no pin covers;
- three assertions hold the wiring the text pins cannot see:
  the module's declaration, the words' first word, and the `[[bin]]` `path`
  (this revision's answer to the fifth read's M2).

#### The hook guard on `main.rs`

`main_does_not_dispatch_on_its_start_name` becomes
`main_is_the_reviewed_handful_of_statements`,
and its body is this, whole,
with the file's code text pinned in one constant:

```rust
// package/git-policy/cli/src/native/source_guard/hook_self_invocation_tests.rs
/// The shared stripper and the comment rule, which every guard of a file's text reads through.
use super::{assert_whole_line_comments_only, code_lines};

/// The executable entry point, whole.
const MAIN_SOURCE: &str = include_str!("../main.rs");

/// `main.rs`'s code text, whole and in order: comments cut,
/// leading and trailing blanks trimmed.
/// A change to any line here is a change to what `main` may do with the argument vector,
/// so the pin and the code change in one commit,
/// and a reviewer reads the two diffs together.
const MAIN_CODE: [&str; 47] = [
    "use std::ffi::OsString;",
    "use git_policy_cli::service_log_lines_off_the_calling_thread::{WAIT_BEFORE_AN_EXIT, write_what_waits};",
    "use git_policy_cli::diagnostic_lines_off_the_calling_thread::write_what_waits as write_every_diagnostic_line_that_waits;",
    "#[cfg(target_os = \"linux\")]",
    "use git_policy_cli::worktree_clone_mount_handler::root_mode_names::{self, RootModeStart};",
    "fn main() {",
    "#[allow(clippy::disallowed_methods)]",
    "let arguments: Vec<OsString> = std::env::args_os().collect();",
    "#[cfg(target_os = \"linux\")]",
    "{",
    "match root_mode_names::root_mode_start(arguments.as_slice()) {",
    "RootModeStart::Wrapper => {}",
    "RootModeStart::Arguments(mode_arguments) => {",
    "git_policy_cli::panic_notice::install_payload_free_panic_hook();",
    "let code: i32 = git_policy_cli::worktree_clone_mount_handler::run(mode_arguments);",
    "end_after_the_log(code);",
    "}",
    "RootModeStart::Refused(misuse) => {",
    "git_policy_cli::panic_notice::install_payload_free_panic_hook();",
    "let code: i32 = git_policy_cli::worktree_clone_mount_handler::refuse_root_mode_misuse(misuse);",
    "end_after_the_log(code);",
    "}",
    "}",
    "}",
    "let handed: &[OsString] = arguments.get(1..).unwrap_or_default();",
    "if git_policy_cli::service_main::is_service_start(handed) {",
    "let code: i32 = match git_policy_cli::service_main::service_options(handed) {",
    "Ok(options) => git_policy_cli::service_main::run(options.as_slice()),",
    "Err(word) => {",
    "git_policy_cli::service_error!(\"an argument of the service is not text: {}\", word.to_string_lossy());",
    "git_policy_cli::service_main::EXIT_REFUSED",
    "}",
    "};",
    "end_after_the_log(code);",
    "}",
    "git_policy_cli::panic_notice::install_payload_free_panic_hook();",
    "let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();",
    "let start_directory: git_policy_cli::start_directory::StartDirectory =",
    "git_policy_cli::start_directory::StartDirectory::read_from_the_process();",
    "let code: i32 = git_policy_cli::entry::run_process(handed, environment.as_slice(), &start_directory);",
    "end_after_the_log(code);",
    "}",
    "fn end_after_the_log(code: i32) -> ! {",
    "let _diagnosed: bool = write_every_diagnostic_line_that_waits(WAIT_BEFORE_AN_EXIT);",
    "let _written: bool = write_what_waits(WAIT_BEFORE_AN_EXIT);",
    "std::process::exit(code);",
    "}",
];

#[test]
fn main_is_the_reviewed_handful_of_statements() {
    assert_whole_line_comments_only("main.rs", MAIN_SOURCE);
    let code: Vec<String> = code_lines(MAIN_SOURCE, "main.rs").into_iter().filter(|line| { return !line.is_empty(); }).collect();
    assert_eq!(code.len(), MAIN_CODE.len(), "main.rs has {} code line(s), and the reviewed text has {}: {code:#?}", code.len(), MAIN_CODE.len());
    for (at, (found, want)) in code.iter().zip(MAIN_CODE.iter()).enumerate() {
        assert_eq!(found.as_str(), *want, "main.rs's code line {} is not the reviewed one", at + 1);
    }
}
```

The constant is the whole file, comments aside:
the three plain `use` lines and the Linux-only `use` of `root_mode_names`,
the one `#[allow(clippy::disallowed_methods)]`,
the collection,
the `#[cfg(target_os = "linux")]` block with its two arms and the hook each arm installs,
the `handed` line the service check and the wrapper's entry both read,
today's service block,
the environment collection,
the entry call,
and the log wait.
Nothing in the file is outside it,
so no spelling,
no attribute,
no alias,
no second decision call and no reordered arm can be added without changing a pinned line.

This is the change that makes the guard's claim true and not merely wider:
a fourth vector line fails on the count,
`std::env::args()` fails on its line,
an inner `#![allow]` fails on the count,
a `/proc` read fails on its line,
a deleted `end_after_the_log` fails on the count,
and the messages name the first differing line with both texts,
so the person who re-pins reads the difference line by line.

#### The `[[bin]]` `path` the release builds

The manifest names the file each `[[bin]]` compiles, and `release_bins_tests.rs` read only the
names,
so a `path` that moved left the entry pin reading a file the release does not build.
The reader and the assertion are these:

```rust
// package/git-policy/cli/src/native/source_guard/release_bins_tests.rs
/// The `[[bin]]` tables of a manifest as `(name, path)` pairs, in the order they are written.
fn manifest_bin_tables(manifest: &str) -> Vec<(String, String)> {
    let mut tables: Vec<(String, String)> = Vec::new();
    let mut name: Option<String> = None;
    let mut path: Option<String> = None;
    let mut in_bin: bool = false;
    for line in manifest.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') {
            if in_bin {
                tables.push((name.take().unwrap_or_default(), path.take().unwrap_or_default()));
            }
            in_bin = trimmed == "[[bin]]";
            continue;
        }
        if !in_bin {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name = \"") {
            name = Some(rest.split('"').next().unwrap_or_default().to_string());
        }
        if let Some(rest) = trimmed.strip_prefix("path = \"") {
            path = Some(rest.split('"').next().unwrap_or_default().to_string());
        }
    }
    if in_bin {
        tables.push((name.take().unwrap_or_default(), path.take().unwrap_or_default()));
    }
    return tables;
}

/// The wrapper's `[[bin]]` compiles the file the entry pin holds.
#[test]
fn the_wrappers_bin_compiles_the_pinned_main() {
    let manifest: String = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).expect("the manifest");
    let wrapper: Vec<(String, String)> = manifest_bin_tables(manifest.as_str()).into_iter().filter(|(name, _path)| { return name == "monochromatic-git-policy-cli"; }).collect();
    assert_eq!(wrapper, vec![(String::from("monochromatic-git-policy-cli"), String::from("src/native/main.rs"))], "the wrapper's `[[bin]]` must compile src/native/main.rs, the one file the entry pin reads; a `path` that moves makes the pinned text guard a file the release does not build");
}
```

#### The shared stripper

`code_lines` lives in `source_guard/mod.rs` as `pub(super)`,
and the two pins and the scan all read through it.
It reads what it is reading:

```rust
// package/git-policy/cli/src/native/source_guard/mod.rs
/// What the stripper is reading at one point of one source.
#[derive(Clone, Copy)]
enum Reading {
    /// Code: a comment may start here.
    Code,
    /// A block comment, at its nesting depth: `/* /* */ */` closes at the second `*/`.
    BlockComment(usize),
    /// A `"..."` string or a `b"..."` byte string, where a backslash escapes the character after it.
    Text,
    /// An `r"..."` or `r#"..."#` raw string, at the number of `#` its opening used, where a backslash is the text it is.
    RawText(usize),
    /// A `'...'` character literal.
    Character,
}

/// The code lines of one source named `what`: block comments removed, `//` comments cut, blanks trimmed,
/// and every byte of a string, raw string or character literal kept as the text it is.
/// Why the literals are read rather than cut through:
///  the scan reads 1,472 release files through this, and a glob or a URL in a string holds `//` or `/*`,
///  which read as a comment hides every line after it from every rule that trusts this.
///  A file that really ends inside a comment or a literal does not compile,
///  so one that does is a file this misread, and the failure names it.
pub(super) fn code_lines(source: &str, what: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut reading: Reading = Reading::Code;
    for raw in source.lines() {
        let mut kept: String = String::new();
        let mut characters = raw.chars().peekable();
        while let Some(character) = characters.next() {
            match reading {
                Reading::BlockComment(depth) => {
                    if character == '/' && characters.peek() == Some(&'*') {
                        characters.next();
                        reading = Reading::BlockComment(depth + 1);
                    } else if character == '*' && characters.peek() == Some(&'/') {
                        characters.next();
                        reading = if depth <= 1 { Reading::Code } else { Reading::BlockComment(depth - 1) };
                    }
                    continue;
                }
                Reading::Text => {
                    kept.push(character);
                    if character == '\\' {
                        if let Some(escaped) = characters.next() {
                            kept.push(escaped);
                        }
                    } else if character == '"' {
                        reading = Reading::Code;
                    }
                    continue;
                }
                Reading::RawText(hashes) => {
                    kept.push(character);
                    if character == '"' {
                        let mut look = characters.clone();
                        let mut closed: bool = true;
                        for _ in 0..hashes {
                            if look.next() != Some('#') {
                                closed = false;
                                break;
                            }
                        }
                        if closed {
                            for _ in 0..hashes {
                                characters.next();
                                kept.push('#');
                            }
                            reading = Reading::Code;
                        }
                    }
                    continue;
                }
                Reading::Character => {
                    kept.push(character);
                    if character == '\\' {
                        if let Some(escaped) = characters.next() {
                            kept.push(escaped);
                        }
                    } else if character == '\'' {
                        reading = Reading::Code;
                    }
                    continue;
                }
                Reading::Code => {}
            }
            if character == '/' && characters.peek() == Some(&'/') {
                break;
            }
            if character == '/' && characters.peek() == Some(&'*') {
                characters.next();
                reading = Reading::BlockComment(1);
                continue;
            }
            if character == '"' {
                kept.push(character);
                reading = Reading::Text;
                continue;
            }
            if character == '\'' {
                kept.push(character);
                let mut look = characters.clone();
                look.next();
                if characters.peek() == Some(&'\\') || look.peek() == Some(&'\'') {
                    reading = Reading::Character;
                }
                continue;
            }
            if character == 'r' || character == 'b' {
                let continues_a_word: bool = kept.chars().next_back().is_some_and(|previous| { return previous.is_alphanumeric() || previous == '_'; });
                let mut look: std::iter::Peekable<std::str::Chars<'_>> = characters.clone();
                let mut hashes: usize = 0;
                while look.peek() == Some(&'#') {
                    look.next();
                    hashes += 1;
                }
                if !continues_a_word && look.peek() == Some(&'"') && (character == 'r' || hashes == 0) {
                    kept.push(character);
                    for _ in 0..hashes {
                        characters.next();
                        kept.push('#');
                    }
                    if let Some(quote) = characters.next() {
                        kept.push(quote);
                    }
                    reading = if character == 'r' { Reading::RawText(hashes) } else { Reading::Text };
                    continue;
                }
            }
            kept.push(character);
        }
        lines.push(String::from(kept.trim()));
    }
    match reading {
        Reading::Code => {}
        Reading::BlockComment(_) => panic!("{what} ends inside a block comment, so the stripper's reading of it cannot be trusted"),
        Reading::Text | Reading::RawText(_) | Reading::Character => panic!("{what} ends inside a string or a character literal, so the stripper's reading of it cannot be trusted"),
    }
    return lines;
}

/// The text with every whitespace character removed, so `args_os ()` reads as `args_os()` and `clippy ::` as `clippy::`.
pub(super) fn squeezed(text: &str) -> String {
    return text.chars().filter(|character| { return !character.is_whitespace(); }).collect();
}

/// Every `//` in the source begins its own line, so cutting a line at the first one hides nothing,
/// and no block comment spans lines a reader would have to reconstruct.
pub(super) fn assert_whole_line_comments_only(what: &str, source: &str) {
    assert!(!source.contains("/*"), "{what} holds a block comment, which the stripper would have to reconstruct");
    for line in source.lines() {
        if let Some(at) = line.find("//") {
            assert!(line[..at].trim().is_empty(), "a comment starts after code in {what}, and the stripper would cut what follows it: {line}");
        }
    }
}
```

`code_lines` keeps every line,
blank ones included,
so a line's index is its line number;
each reader filters the blank ones out before comparing.
The two pins call `assert_whole_line_comments_only` over their own file,
which the project's rule about comments already asks of them
(rule TD4, `AGENTS.md`);
the scan does not,
because the release tree holds 63 files with a `/*` and 9 lines with a `//` inside a string,
and the stripper now reads those correctly rather than forbidding them
(run keep/stripper-over-the-release-tree.log).

The scan's own reader is the wrapper the fifth revision printed no code for:

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_tests.rs
/// The lines of a source named `what` that are code,
/// as the line number, the line, and the line with the next code line joined so a call split over
/// two lines still reads.
fn code_lines(text: &str, what: &str) -> Vec<(usize, String, String)> {
    let stripped: Vec<String> = super::code_lines(text, what);
    let mut code: Vec<(usize, String, String)> = Vec::new();
    for (index, line) in stripped.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        let next: &str = stripped.iter().skip(index + 1).find(|following| { return !following.is_empty(); }).map_or("", |following| { return following.as_str(); });
        code.push((index + 1, line.clone(), format!("{line} {next}")));
    }
    return code;
}
```

That is measured to leave today's verdicts alone:
the six rules over the real release tree find the same lines under the old reader
(drop the lines that start with `//`) and under this one
(run keep/stripper-over-the-release-tree.log).
The difference the stripper makes is what it now sees:
the three files that used to end inside a comment yield 424 more code lines to the rules,
and none of them matches a needle
(run keep/stripper-over-the-release-tree.log).

#### The scan rule's needles

`no_self_invocation_rules_tests.rs` keeps its rules and gains one,
and both squeeze the whitespace out of a line before comparing it,
so a call split by a space still matches:

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs
Rule {
    id: "argv0-dispatch",
    reason: "reads the program name the wrapper was started as, which only matters for a wrapper that is started under another name",
    matches: |line, _joined| {
        let squeezed_line: String = crate::source_guard::squeezed(line);
        return ["args_os().next()", "args().next()", ".nth(0)", "argv[0]", "arg0(", "/proc/self/cmdline", "/proc/self/comm", "/proc/thread-self/"]
            .iter()
            .any(|needle| { return squeezed_line.contains(needle); });
    },
},
Rule {
    id: "argv-vector-read",
    reason: "reads the argument vector, whose first element is the name the program was started under; the one collection belongs to main, which hands the whole vector to root_mode_names",
    matches: |line, _joined| {
        let squeezed_line: String = crate::source_guard::squeezed(line);
        return ["args_os", "::args(", ".args()"].iter().any(|needle| { return squeezed_line.contains(needle); });
    },
},
```

`crate::source_guard::squeezed` is the one the lint-attribute pin reads through too,
so the two guards cannot disagree about what a squeezed line is.

with one `ALLOWED` row and one new sample:

```rust
Allowed {
    file: "main.rs",
    rule: "argv-vector-read",
    why: "the one vector the owner's answer needs: main collects it once, and the pin on main.rs fixes the one line that names it",
},
```

(the `samples` array at `no_self_invocation_tests.rs:191` is `[(&str, &str); 6]`,
so it is written with a `7` and gains
`("argv-vector-read", "use std::env::args_os as the_vector;")`,
which `:208`'s length assertion then carries).

Why these needles and not a broad `/proc/self/`:

- measured over the release tree in keep/scan-needles-fifth.log,
  `/proc/self/` stands in 46 code lines of 27 files,
  `/proc/self/cmdline` in none,
  `/proc/self/comm` in none and
  `/proc/thread-self/` in none;
- `args_os` stands in two lines (this `main.rs` and the second program's, deleted here),
  `::args(` in none and
  `.args()` in none,
  and the second of those is what catches `std::env::args()`, `env::args()` and a module alias's
  `e::args()` alike;
- so the needles name the argument vector and the name's own files without failing today's tree,
  and the four spellings that release code does read today stay legal:
  `/proc/self/stat`,
  `/proc/self/status`,
  `/proc/self/task/<tid>/stat` and
  `/proc/<pid>/comm`;
- `/proc/self/stat` and `/proc/self/status` do carry the process name, in the second field and the
  `Name:` line,
  and release code reads both for other fields,
  while `/proc/self/statm` does not carry it
  (replay the fifth read's `keep/cmdline-needle-reach-over-the-release-tree.log`).

What no needle and no typed ban reaches is disclosed rather than claimed closed,
and the pin on `main.rs` is what holds where the decision is made:
a release file can still read the start name through `std::env::argv[0]` itself,
through `/proc/<own pid>/cmdline` and `/proc/self/task/<tid>/comm`,
through glibc's `program_invocation_name` and `program_invocation_short_name`,
and through `getauxval(AT_EXECFN)`
(replay the fifth read's `keep/other-ways-to-read-the-start-name-no-guard-names.log`,
which measured each of the last four returning the name a start was given).
`/proc/<pid>/cmdline` stays out of the scan because a needle that named `/cmdline` would fail today's
tree on the kernel command line that `health/doctor_filesystem.rs:65` reads,
and the one spelling that would not,
`}/cmdline`,
is a `format!`'s closing brace rather than a path
(replay the fifth read's `keep/cmdline-needle-reach-over-the-release-tree.log`).
A read of any of them inside `main.rs` is a pinned-line change,
and every other file that reads the name is a review, not a scan.

Each added needle has a witness of its own,
and each file release code reads today has a control it must not fire on:

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_tests.rs
#[test]
fn each_added_needle_fires_on_its_own_witness_and_no_release_read() {
    let dispatch: &Rule = RULES.iter().find(|rule| { return rule.id == "argv0-dispatch"; }).expect("argv0-dispatch");
    let vector: &Rule = RULES.iter().find(|rule| { return rule.id == "argv-vector-read"; }).expect("argv-vector-read");
    for witness in ["std::fs::read(\"/proc/self/cmdline\")", "std::fs::read_to_string(\"/proc/self/comm\")", "std::fs::read_to_string(\"/proc/thread-self/comm\")"] {
        assert!((dispatch.matches)(witness, witness), "argv0-dispatch does not fire on {witness}");
    }
    for witness in ["use std::env::args_os as the_vector;", "let all = std::env::args().collect::<Vec<_>>();", "let name = e::args();"] {
        assert!((vector.matches)(witness, witness), "argv-vector-read does not fire on {witness}");
    }
    for control in ["std::fs::read_to_string(\"/proc/self/stat\")", "std::fs::read_to_string(format!(\"/proc/self/task/{tid}/io\"))", "std::fs::read_to_string(format!(\"/proc/{pid}/comm\"))"] {
        assert!(!(dispatch.matches)(control, control), "argv0-dispatch fires on {control}, which release code reads today");
    }
}
```

#### The lint-attribute pin

`source_guard/lint_attribute_tests.rs` is new,
and it pins every release line that can excuse a typed ban:
a line naming a clippy lint,
an inner attribute,
or a lint group.
`release_sources_in` moves to `source_guard/mod.rs` as `pub(super)` so both guards classify the tree
the same way.

```rust
// package/git-policy/cli/src/native/source_guard/lint_attribute_tests.rs
use super::{code_lines, release_sources_in, squeezed};
use crate::source_scan_tests::{real_roots, source_root};
use std::path::PathBuf;

/// Every release line that names a clippy lint, an inner attribute or a lint group,
/// with the file it stands in: the whole set.
const LINT_ATTRIBUTES: [(&str, &str); 20] = [
    ("invocation/real_git.rs", "clippy::disallowed_methods,"),
    ("lfs/lfs_pointer_scan.rs", "#![cfg(unix)]"),
    ("main.rs", "#[allow(clippy::disallowed_methods)]"),
    ("push/auto_push_trigger.rs", "#![cfg(unix)]"),
    ("repo_read/object_abbreviation.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_diff.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_odb.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_pack.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_query.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_query_refs.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_refs.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_tree.rs", "#![cfg(unix)]"),
    ("repo_read/repo_read_tree_cached.rs", "#![cfg(unix)]"),
    ("worktree_clone_unmount/staged_folder_record.rs", "#[allow(clippy::disallowed_methods, reason = \"the JSON macro builds one object of every field; no fallible read is made here\")]"),
    ("worktree_clone_unmount/staged_unmount.rs", "#[allow(clippy::large_enum_variant, reason = \"the retained alternative names the step, words and row a person acts on\")]"),
    ("worktree_clone_unmount/staging_claim.rs", "#[allow(clippy::disallowed_methods, reason = \"the JSON macro builds one object of every field; no fallible read is made here\")]"),
    ("worktree_clone_unmount/unmount_checks.rs", "#[allow(clippy::result_large_err, reason = \"the refusal carries the mount row a person needs, and the call chain is not hot\")]"),
    ("worktree_clone_unmount/unmount_record.rs", "#[allow(clippy::result_large_err, reason = \"the outcome carries the mount row and the step a person acts on\")]"),
    ("worktree_clone_unmount/unmount_record.rs", "#[allow(clippy::result_large_err, reason = \"the outcome carries the mount row and the step a person acts on\")]"),
    ("worktree_clone_unmount/user_cache_pin.rs", "#[allow(clippy::disallowed_methods, reason = \"the JSON macro builds one object of every field; no fallible read is made here\")]"),
];

#[test]
fn the_release_code_carries_only_the_named_lint_attributes() {
    let root: PathBuf = source_root();
    let mut found: Vec<(String, String)> = Vec::new();
    for file in release_sources_in(&root, &real_roots()).expect("the release sources") {
        let text: String = std::fs::read_to_string(root.join(file.as_str())).expect("source file");
        for line in code_lines(text.as_str(), file.as_str()) {
            let squeezed_line: String = squeezed(line.as_str());
            if ["clippy::", "#![", "(warnings"].iter().any(|needle| { return squeezed_line.contains(needle); }) {
                found.push((file.clone(), line));
            }
        }
    }
    let mut named: Vec<(String, String)> = LINT_ATTRIBUTES.iter().map(|(file, line)| { return (String::from(*file), String::from(*line)); }).collect();
    found.sort();
    named.sort();
    assert_eq!(found, named, "release code carries a lint attribute this pin does not name, and an allow lists the lints the typed bans stop applying");
}
```

That is what answers the fifth read's M1 for the typed ban without relying on `main.rs` alone:
a new `#[allow(clippy::all)]`, a `#![allow(warnings)]` or a second allow in a file that has one
all fail here,
and so does an allow written with a space before its `::`,
because the needle is matched against the squeezed line.
The set is measured rather than guessed
(run keep/stripper-over-the-release-tree.log).
What the `(warnings` needle does and does not catch is stated in "The Low findings":
`disallowed_methods` is `deny`, so a `warnings` allow cannot excuse it,
and the needle is there because the pin's claim covers every line that names a lint group.

#### The one-name guard on the names module

`source_guard/root_mode_name_tests.rs` is new,
declared in `source_guard/mod.rs`,
and it pins the module whole:

```rust
// package/git-policy/cli/src/native/source_guard/root_mode_name_tests.rs
use super::{assert_whole_line_comments_only, code_lines};
use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;
use crate::worktree_clone_mount_handler::ZEDLET_INSTALL_PATH;
use std::path::Path;

/// The one module allowed to read the name this program was started under.
const NAMES_SOURCE: &str = include_str!("../worktree_clone_mount_handler/root_mode_names.rs");

/// The file that declares that module, and so decides which file the crate compiles as it.
const HANDLER_MODULE_SOURCE: &str = include_str!("../worktree_clone_mount_handler/mod.rs");

/// `root_mode_names.rs`'s code text, whole and in order: comments cut, blanks trimmed.
/// A change to any line here is a change to what the module decides, so the pin and the code
/// change in one commit.
const NAMES_CODE: [&str; 53] = [
    "use std::ffi::{OsStr, OsString};",
    "use std::path::Path;",
    "use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;",
    "pub const ZEDLET_START_NAME: &str = \"history_event-mount_git_worktree_clone_datasets\";",
    "pub const ROOT_GROUP_WORD: &str = \"zfs-root\";",
    "pub enum RootModeMisuse<'arguments> {",
    "NoModeWordAfterTheGroupWord,",
    "WordUnderTheZedletName(&'arguments OsStr),",
    "}",
    "pub enum RootModeStart<'arguments> {",
    "Arguments(&'arguments [OsString]),",
    "Refused(RootModeMisuse<'arguments>),",
    "Wrapper,",
    "}",
    "pub fn root_mode_start(arguments: &[OsString]) -> RootModeStart<'_> {",
    "let (start_name, handed): (Option<&OsStr>, &[OsString]) = split_the_name(arguments);",
    "if starts_with_the_group_words(handed) {",
    "let mode_arguments: &[OsString] = &handed[2..];",
    "if mode_arguments.is_empty() {",
    "return RootModeStart::Refused(RootModeMisuse::NoModeWordAfterTheGroupWord);",
    "}",
    "return RootModeStart::Arguments(mode_arguments);",
    "}",
    "if !names_the_zedlet(start_name) {",
    "return RootModeStart::Wrapper;",
    "}",
    "if handed.is_empty() || (handed.len() == 1 && (handed[0] == OsStr::new(\"--help\") || handed[0] == OsStr::new(\"--version\"))) {",
    "return RootModeStart::Arguments(handed);",
    "}",
    "return RootModeStart::Refused(RootModeMisuse::WordUnderTheZedletName(word_that_cannot_be_honoured(handed)));",
    "}",
    "fn starts_with_the_group_words(handed: &[OsString]) -> bool {",
    "return handed.len() >= 2 && handed[0] == OsStr::new(COMMAND_WORD.current) && handed[1] == OsStr::new(ROOT_GROUP_WORD);",
    "}",
    "fn names_the_zedlet(start_name: Option<&OsStr>) -> bool {",
    "return start_name.and_then(last_component) == Some(ZEDLET_START_NAME);",
    "}",
    "fn last_component(start_name: &OsStr) -> Option<&str> {",
    "return Path::new(start_name).file_name().and_then(OsStr::to_str);",
    "}",
    "fn split_the_name(arguments: &[OsString]) -> (Option<&OsStr>, &[OsString]) {",
    "let Some((start_name, handed)) = arguments.split_first() else {",
    "return (None, &[]);",
    "};",
    "return (Some(start_name.as_os_str()), handed);",
    "}",
    "fn word_that_cannot_be_honoured(handed: &[OsString]) -> &OsStr {",
    "let first: &OsStr = handed[0].as_os_str();",
    "if handed.len() > 1 && (first == OsStr::new(\"--help\") || first == OsStr::new(\"--version\")) {",
    "return handed[1].as_os_str();",
    "}",
    "return first;",
    "}",
];

#[test]
fn the_one_start_name_is_written_in_one_place() {
    assert_whole_line_comments_only("root_mode_names.rs", NAMES_SOURCE);
    let code: Vec<String> = code_lines(NAMES_SOURCE, "root_mode_names.rs").into_iter().filter(|line| { return !line.is_empty(); }).collect();
    assert_eq!(code.len(), NAMES_CODE.len(), "root_mode_names.rs has {} code line(s), and the reviewed text has {}: {code:#?}", code.len(), NAMES_CODE.len());
    for (at, (found, want)) in code.iter().zip(NAMES_CODE.iter()).enumerate() {
        assert_eq!(found.as_str(), *want, "root_mode_names.rs's code line {} is not the reviewed one", at + 1);
    }
}

/// The pinned file is the file the crate compiles as that module,
/// because a `#[path]` attribute beside the declaration would make a different file the module
/// while the pin above kept passing the old one.
#[test]
fn the_pinned_module_is_the_module_the_crate_compiles() {
    let declarations: usize = HANDLER_MODULE_SOURCE.lines().filter(|line| { return line.trim() == "pub mod root_mode_names;"; }).count();
    assert_eq!(declarations, 1, "worktree_clone_mount_handler/mod.rs must declare `pub mod root_mode_names;` exactly once, and it declares it {declarations} time(s)");
    assert!(!HANDLER_MODULE_SOURCE.contains("#[path"), "worktree_clone_mount_handler/mod.rs must hold no `#[path]` attribute, so no declaration there can point the compiled module at another file");
}

/// The words' first word is the program's own name,
/// taken from the one constant the crate keeps it in.
#[test]
fn the_words_first_word_is_the_programs_own_name() {
    assert_eq!(COMMAND_WORD.current, "monochromatic-git-policy-cli", "the words are `monochromatic-git-policy-cli <the group word> <mode>`, and root_mode_names.rs reads this constant for the first of them");
}

/// The installed ZED path's file name is the one start name the module compares,
/// so an install and the dispatch cannot drift apart.
#[test]
fn the_installed_zedlet_path_carries_the_one_start_name() {
    let installed: Option<&str> = Path::new(ZEDLET_INSTALL_PATH).file_name().and_then(|name| { return name.to_str(); });
    assert_eq!(installed, Some("history_event-mount_git_worktree_clone_datasets"), "ZEDLET_INSTALL_PATH's file name must be ZEDLET_START_NAME, which root_mode_names.rs compares a start's last component against");
}
```

The pin includes the module's `use` lines and both item headers,
which is what the read's `nm4` (a local `Path` shadowing the import) and `nm2` and `nm3`
(the constants' values swapped while the old literals stayed) needed and did not have.

The module's string literal set,
its function count and its `file_name()` count are gone:
they were a list's way of saying "the whole text is this",
and the whole text is now pinned.

#### The typed half

The text guards cannot see a call that is resolved rather than spelled,
so the typed ban stays and grows.
`package/git-policy/cli/clippy.toml` gains two rows beside the `current_exe` ban:

```toml
  { path = "std::env::args", reason = "the program name this process was started under is one decision in one module (doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md); every other file that wants the arguments takes them as a parameter" },
  { path = "std::env::args_os", reason = "the program name this process was started under is one decision in one module (doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md); main's one collection is the allow this row names" },
```

and the allows are four:
the one collection in `main`,
and one in each test file that starts the test binary,
which the crate's other `current_exe` allows are the precedent for:

- `src/native/binary/binary_supervisor_tests.rs:755`;
- `src/native/push/child_watch_tests.rs:1000`;
- `src/native/landing/service_durable_tests.rs:241`.

(replay keep/clippy-bans.log):
a probe crate carrying those rows and the crate's own file refuses a call through a
`use std::env::args_os as ...` alias,
refuses `std::env::args()`,
and accepts the one statement that carries the allow.

#### Replayed

(run keep/guards-replay-sixth.log),
from `keep/sources/cut_fences.py`,
which cuts every printed text out of this document by fence,
`sources/assemble_probe.py`,
which builds the probe crate (its head lists the four substitutions a binary crate forces),
and `sources/guard_replay.py`,
which drives the verdicts and writes the log.
The first two verdicts are the document against itself:
the whole file and the whole module, both printed in "The rule, in the entry",
are read as code and compared with the two pins printed in
"The hook guard on `main.rs`" and "The one-name guard on the names module",
so the two printings cannot drift apart without this log saying so.
The log is kept whole in this round's `keep/`;
it prints the md5 of every text it replays,
every variant's own verdict,
and the release tree's two findings under both readers.
The excerpt below shortens each path to its end,
and the log prints them whole.

```text
# the two texts this log replays, cut out of the document by fence
#   main_rs.rs: md5 6d1190dee32095eee4ffd8d4fc32fe0c
#   names_rs.rs: md5 a5da89404c3cb83b3d29272d582fc7bb
== the document's own printed file, against the pin the document prints
ok   PASS scratch/printed/main_rs.rs
ok   PASS scratch/printed/names_rs.rs
== the positive control: today's main.rs must fail the entry pin
ok   FAIL package/git-policy/cli/src/native/main.rs: assertion `left == right` failed: main.rs has 27 code line(s), and the reviewed text has 47
== the wiring the pins cannot see
# the real worktree_clone_mount_handler/mod.rs at this base: the module is not there yet, so the declaration assertion fails until this design lands
ok   FAIL package/git-policy/cli/src/native/worktree_clone_mount_handler/mod.rs: worktree_clone_mount_handler/mod.rs must declare `pub mod root_mode_names;` exactly once, and it declares it 0 time(s)
ok   PASS keep/sources/fixtures/handler_mod_declared.rs
ok   FAIL keep/sources/fixtures/handler_mod_path_redirect.rs: worktree_clone_mount_handler/mod.rs must hold no `#[path]` attribute, so no declaration there can point the compiled module at another file
ok   PASS COMMAND_WORD.current
ok   PASS ZEDLET_INSTALL_PATH's file name
== the lint-attribute pin, the whole release tree
ok   PASS lint pin, main.rs read as scratch/printed/main_rs.rs
# the same pin with today's main.rs, which holds no allow: the set is one line short, and the pin says so
ok   FAIL lint pin, main.rs read as package/git-policy/cli/src/native/main.rs: release code carries a lint attribute this pin does not name
== the [[bin]] path assertion
ok   PASS release_bins_tests' path assertion
ok   FAIL release_bins_tests' path assertion: the wrapper's `[[bin]]` must compile src/native/main.rs, the one file the entry pin reads; a `path` that moves makes the pinned text guard a file the release does not build |   left: [("monochromatic-git-policy-cli", "src/native/elsewhere.rs")]
ok   PASS release_bins_tests' path assertion
== the scan's two rules over its fixtures
ok   FAIL sc1_args_collected_with_its_own_allow.rs: reports [(3, "argv-vector-read", "let all: Vec<String> = std::env::args().collect();")]
ok   FAIL sc3_comment_closer_hides_the_read.rs: reports [(4, "argv0-dispatch", "return std::env::args_os().next();")]
ok   PASS sc6_proc_self_stat_as_release_code_does_today.rs: reports nothing
ok   PASS sc9_release_control_the_task_stats.rs: reports nothing
== the six rules over the real release tree, under both readers
# reader: the printed string-aware stripper; release files: 1472; findings: 2
#   main.rs:52 argv-vector-read let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
#   mount_git_worktree_clone_datasets/main.rs:49 argv-vector-read let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
# reader: today's (drop the lines that start with `//`); release files: 1472; findings: 2
#   main.rs:52 argv-vector-read let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
#   mount_git_worktree_clone_datasets/main.rs:49 argv-vector-read let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
== each added needle fires on its own witness and on no release read
ok   PASS each rule fires on its own witness
```

The fifth round's variants keep their names and their verdicts,
and every one of them fails on a changed line or on the comment rule,
as the whole log prints.
Three of this round's results are worth naming here:

- `nm6`,
  which the fifth read rebuilt and the fifth revision's log lacked,
  fails on its added lines (run keep/guards-replay-sixth.log);
- the two wiring fixtures show the assertion working in both directions:
  the declaration fixture passes,
  and the `#[path]` redirect fails with the attribute's own message;
- the scan's fixtures and the release tree give the same verdicts under the string-aware stripper
  and under today's reader,
  and the release tree's only two findings are the two `args_os` collections the design removes and
  the `ALLOWED` row excuses.

`SPEC.md:19715` becomes
(it was `:19705` at the fifth revision's base, `:19588` at `45ac28ae47` and `:19261` at
`f3d81e0910`):

> Git never starts the wrapper for a hook:
> the wrapper writes no hook entry,
> passes no hooks path of a directory it made,
> and reads the name it was started under in one module only,
> the installed ZED history event's,
> where that name selects the clone-mount sweep and the modes' own `--help` and `--version`,
> and nothing else a Git command means.

`binary_external_hook_tests.rs:174` keeps proving that every Git hook event's name still behaves as `git`,
which is now a claim about every name but one.

## Choice 2: the group word units, adoption and a person spell

The answer fixes the shape of the words,
because `service run` is the precedent:
the wrapper's own word,
then a namespace word,
then the modes' grammar.
Which namespace word it is,
is this choice's,
and a unit's `ExecStart` line,
an adoption's command and a person's command all read it.

What the modes print does not depend on the word,
except in the usage's closing sentences,
so it is printed once here.
The text is the build's,
copied from a run of the built handler,
with the two new modes' lines where they will stand
(replay keep/handler-usage.log):

```text
usage: mount-git-worktree-clone-datasets [mode]
  (no argument)        run the clone-mount sweep the installed ZED event starts
  boot-reconcile       reconcile clone mounts after boot; the boot unit's ExecStart
  start-probe          run the start probe's arms and write this boot's result
  shutdown-unmount     unmount the verified clone mounts at shutdown; the boot unit's ExecStopPost
  holder-scan --dataset <name> [--nonce <32 hex digits>]
                       report which mount namespaces hold that clone, and every read the scan could not complete
  adopt-worktree-parent --checkout <path> --device <count> --inode <count>
                       record an administrator-identified replacement .claude/worktrees folder
  record-checkout --checkout <path> [--replace <the kernel's guid, which the refusal prints>]
                       record this checkout's path, dataset, GUID, container and pool for the route;
                       --replace, given the record's own guid, replaces a record whose GUID a
                       legitimate restore changed
  mount-path-descendant --dataset <name>
                       mount one nested dataset at its own mountpoint, under the recorded checkout
  --help               print this text
  --version            print this build's version
The same build answers these modes through the words monochromatic-git-policy-cli zfs-root <mode>,
is installed as <the root-owned file choice 3 names>, and answers ZED by the name
<the installed ZED history event>.
The modes that make requests open /dev/zfs through this repository's binding, never a CDDL library and never the zfs program.
```

The two added lines are this design's (design),
and no build prints them yet,
and those closing sentences are the change that makes the words discoverable
(choice 5 decides what the first line and every refusal name).
Everything else is the build's own text,
which the second revision's version of this block abbreviated;
the second line of `holder-scan` and the two closing paragraphs are quoted here because they exist.

Every option in this choice is shown with the start it produces.
The words are not in the build yet,
so the `decides` line is the fixture's
(cases 3, 7, 24, 27 and 28;
run keep/dispatch-cases.log),
and the `prints` line is what the modes print today
(replay keep/handler-usage.log).

### Option A: `zfs-root`

```text
$ git monochromatic-git-policy-cli zfs-root --help
   decides: root-mode grammar="--help"
   prints:  the usage printed at the head of this choice, with the words and the root-owned file
```

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
```

```sh
sudo /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
  monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
  --checkout /home/user/Monochromatic --device 66306 --inode 1234567
```

```text
$ git monochromatic-git-policy-cli zfs-root
   decides: refused-before-the-grammar word="monochromatic-git-policy-cli zfs-root"
   today:   the management arm's usage, exit=2
```

Pros:

- names the route and the privilege,
  which is what the mode family is called everywhere else in this route's writing;
- no Git word it could be mistaken for,
  and long enough that running it is a deliberate act;
- one word for seven modes,
  so a reader who knows `service` knows this.

Cons:

- the second longest of the four,
  and an adoption's line already carries three options;
- it pairs the route's name with the privilege's,
  so a reader has to learn which half is which.

### Option B: `mount`

```text
$ git monochromatic-git-policy-cli mount boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

```ini
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli mount boot-reconcile
```

Pros:

- the shortest that still says the work,
  and the family is the clone mounts (the design's own name for it and the route's `…-mounts` file names);
- a person types it more easily than the longer ones.

Cons:

- names only the mounting modes,
  while `start-probe`,
  `holder-scan` and `record-checkout` are not mounts;
- `mount` is a word any later mode family would also want,
  so it claims a broad name for a narrow set.

### Option C: `root`

```text
$ git monochromatic-git-policy-cli root boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

Pros:

- shortest,
  and honest about the privilege the modes need.

Cons:

- reads as a Git concept (a root commit,
  the root tree),
  so `git monochromatic-git-policy-cli root …` invites a wrong guess about what it does;
- says nothing about the ZFS route,
  so a person who does not already know the route learns nothing from it.

### Option D: `clone-mounts`

```text
$ git monochromatic-git-policy-cli clone-mounts boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

```ini
# the unit is already named this, so the word and the unit would agree
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli clone-mounts boot-reconcile
```

Pros:

- the route's own noun,
  which its configuration file (`git-worktree-clone-mounts.jsonc`)
  and its unit (`reconcile-git-worktree-clone-mounts.service`) already carry;
- names the work rather than the privilege,
  so a person reading a unit learns what runs;
- no Git word and no other family's word.

Cons:

- the noun describes the mounting modes and not the others:
  `mount-path-descendant` mounts a dataset that is not a clone,
  and `start-probe`,
  `holder-scan`,
  the adoption and `record-checkout` do not mount at all,
  which is the same objection as option B's,
  narrower;
- the longest of the four to type,
  twelve characters against `zfs-root`'s eight;
- says nothing about the privilege,
  which is what an adoption's line is about.

### Ranking of choice 2

Ranking:
A > D > B > C.

A over D:
both name the route,
and A names the privilege the words path exists for,
which is what a unit's `ExecStart` under root and an adoption under `sudo` are about;
D names the work and its noun covers the mounting modes,
while A's word covers the modes and the probes by naming where they run.
D's noun is also the one the record mode's new reader does not use:
the checkout record is not a mount,
so the word A covers it and D does not.
The picks are a naming taste,
and this reason is the delivery that separates them (design).

D over B:
both are about mounts,
and D's noun is this route's own,
already carried by its unit (`reconcile-git-worktree-clone-mounts.service`) and its configuration
file,
where B claims a word any later mount-shaped family would want,
while B's shortness is paid for by a name that will collide.

B over C:
both are shorter than A and D,
but `mount` at least says what the family does,
while `root` collides with a Git word's meaning.

The choice is separable from every other one here.
If the owner prefers a different word,
what changes is:

- `ROOT_GROUP_WORD` in `root_mode_names.rs`,
  which is one line of that module and the same line in its pin,
  and the fixture's cases;
- the unit text in plan step 19,
  the nested dataset's unit,
  and the four documents that carry unit lines;
- the wrapper's management help line in `MANAGEMENT_USAGE` and `MANAGEMENT_HELP`;
- the modes' usage's closing sentences,
  and every refusal that names the words:
  the group-word refusal (`refuse_root_mode_misuse`),
  the grammar's unknown-mode message,
  the usage's trailing sentence,
  and `doctor`'s printed adoption line;
- the release smoke's second arm and `native:install`'s words check;
- `SPEC.md`'s sentence about the words;
- the guest commands in `e2e/start-probe-guest.ts`.

Nothing about the one name,
the layouts or the dispatch's placement changes,
and the pin's lines that name `ROOT_GROUP_WORD` stand,
because a new word changes one pinned line of the module and one of the pin,
which is the review surface choice 2's cost is paid on.

## Choice 3: what root's install holds

The review asked that root's regular file carry a mode's name,
not the wrapper's,
so that a root start under any other name fails with `No such file`.
The answer says units and adoption reach the modes by words.
Both hold at once,
and the four layouts are what fits them together.

What selects a mode under each layout,
with choice 1's recommended rule:

- under the ZED name in `/etc/zfs/zed.d`,
  ZED's own start selects a root mode in every layout,
  because ZED passes the link's name as `argv[0]` and that name is the zedlet's;
- under **option A's** file in `/usr/libexec`,
  the words select a mode,
  the name selects ZED's no-argument sweep and the two answers,
  and every other word is a refusal,
  so no argument vector at that path reaches Git;
- under **options B and C's** file in `/usr/libexec`,
  only the words select a mode,
  and any other argument vector at that path is the wrapper,
  so Git runs as root;
- under **option D**,
  each link carries its own name,
  so a link named for the zedlet behaves as option A and a link named otherwise as option B or C.

The name rule reads `argv[0]`,
not the file,
so what a layout buys is stated per start name and never per file.
The evidence is the compiled module under real starts:
the installed-name file,
reached through links named `git`,
`monochromatic-git-policy-cli` and `mount-git-worktree-clone-datasets`,
decides the wrapper in every case,
and the same file under its own name refuses a bare word
(replay keep/names-starts-fourth.log).
A copy,
a link or an `exec -a` with another name is outside every name rule's reach,
and that is true of all four layouts.

Each layout was run as the account in a throwaway `git init` repository,
with today's wrapper's bytes as the installed file
(`keep/today-installed-name.log`,
run),
because the dispatch itself is choice 1's and this choice is about names on disk.
Today's bytes carry no dispatch,
so each option's block shows what a start does now,
what the layout's name makes it do under this revision's rule,
and what it writes into the repository it stands in.

### Option A: one regular file under the ZED hook's own name

```text
/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ../../../usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
    today:   git's usage, exit=1; a marker in the repository's `.git/cli-git-lock-holders/`
    after:   the attaching sweep
$ .../history_event-mount_git_worktree_clone_datasets boot-reconcile
    today:   git: 'boot-reconcile' is not a git command, exit=1; the same marker
    after:   a refusal with the usage, exit=2, no mode
$ .../history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
    today:   the management usage, exit=2; the same marker
    after:   boot-reconcile
$ .../history_event-mount_git_worktree_clone_datasets --version
    today:   git version 2.56.1 and the wrapper's banner, exit=0; the same marker
    after:   choice 5's line, exit=0
$ .../history_event-mount_git_worktree_clone_dataset --version
    today:   No such file or directory, exit=127
$ .../mount-git-worktree-clone-datasets boot-reconcile
    today:   No such file or directory, exit=127
```

Pros:

- every start **under that name** is a root-mode start,
  whatever its words,
  so a word,
  a typo,
  a stale line or a copied command at that path is a mode or a refusal,
  and never Git as root;
- the wrapper's own name is absent from root's folder,
  which is the review's request,
  and so is the modes' name,
  so both old spellings fail loudly;
- ZED follows the symlink and passes the link's name as `argv[0]`,
  which is the name rule's input;
- one file to verify the digest of and one to replace on upgrade;
- the unit line's words select the mode (choice 1's recommended rule),
  so the answer's second clause holds there.

Cons:

- a file named `history_event-…` in `/usr/libexec` reads as a zedlet copy,
  and a unit starting it reads oddly until one knows the rule;
- the installed path that the modes' messages print is long;
- `SERVICE_INSTALL_PATH`,
  whose name means "where service calls start it",
  is renamed (to `ROOT_MODE_INSTALL_PATH`),
  because nothing about it is a service call any more.

### Option B: one regular file named for the modes

```text
/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ...        symlink
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
    today and after: git: 'boot-reconcile' is not a git command, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets
    today and after: git's usage, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets monochromatic-git-policy-cli zfs-root boot-reconcile
    after: boot-reconcile
```

Pros:

- the file's name,
  the version line and every refusal's program name agree;
- a unit's path reads for what it does.

Cons:

- a bare mode word at that path,
  which is exactly step 19's earlier line as the plan carries it,
  runs the whole wrapper as root:
  Git,
  its configuration,
  its lock pass,
  and a marker in whatever repository the caller stands in.
  That is the fail-open the review named;
- a start under that name with a word a person guessed (`boot-reconcil`)
  does the same,
  because neither that file's name nor a misspelled word selects a root mode;
- the words are the only way in at that path,
  so a unit line that loses them runs Git as root.

### Option C: one regular file named for the wrapper

```text
/usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ...    symlink
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli boot-reconcile
    today and after: git: 'boot-reconcile' is not a git command, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli
    today and after: git's usage, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli --version
    today and after: git version 2.56.1 and the wrapper's banner, exit=0; a marker
```

Pros:

- a unit's path reads as the wrapper a person knows.

Cons:

- any root start under that name is the whole wrapper:
  no root mode at all,
  and Git as root,
  including a stale unit line that names the file and no words;
- it puts the wrapper's own bytes,
  reachable as a plain `git`,
  in root's install folder,
  which is the review's finding exactly.

### Option D: a name per file, by copy or by hard link

The same names as the first three options,
each a copy or a hard link of the one program.

```text
$ cp <the wrapper> /usr/libexec/monochromatic-git-policy-cli/holder-scan
$ /usr/libexec/monochromatic-git-policy-cli/holder-scan --dataset tank/checkout
    today and after: unknown option: --dataset, then git's usage, exit=129, because that name selects nothing
$ /usr/libexec/monochromatic-git-policy-cli/holder-scan
    today and after: git's usage, exit=1
```

This round measured it (`keep/holder-scan-name.log`,
run):
Git's unknown-option branch prints the line and calls `usage()`,
which exits 129,
and the no-argument form exits 1.
The second revision's version of this block gave exit 1,
and was not run.

Pros:

- no link can dangle,
  with hard links.

Cons:

- a copy per name costs a copy per upgrade and can drift;
- a hard link made before an upgrade that writes a new file and renames it over the old stays on the previous release,
  so a zedlet or a unit would run old bytes without a word saying so,
  which is the failure this route can least afford;
- the names multiply the fail-open:
  every name that is not the zedlet's is a Git start under root until the words are given.

### Ranking of choice 3

Ranking:
A > B > C > D.

A over B:
both keep one file,
but A's only root-owned file is a root-mode start whatever its words,
so a typo,
a stale line or a copied command at that path is a refusal or a mode and never Git as root,
while B's is Git as root for a bare word and for no argument at all.
The confusion A costs is a name a reader has to be told about once,
and the units' and adoption's lines are generated from the constants and printed with their values anyway.

B over C:
both fail open for a bare word at their own path,
and B's file at least names the modes it runs.
A stale line that names the wrapper's path fails with `No such file` under A and under B,
because no such file exists in either,
while under C that path exists and runs Git as root.
So B's fail-open is the arguments at its own path,
and C's is every start under its own name,
which B shares,
plus a path that exists where the wrapper's name is expected.

C over D:
C has one file and one upgrade step;
D multiplies both and adds a drift a person cannot see.

What none of them prevents,
and this design does not pretend otherwise:
a copy of the wrapper's bytes somewhere else,
started as root under a name of its own,
runs the whole wrapper
(cases 20 and the stray other name;
run keep/dispatch-cases.log and keep/names-starts-fourth.log),
and a start of any of these files under a caller-chosen `argv[0]` that is not the zedlet's
is the wrapper as that caller.
An account that can write root's install folder can replace any of these files,
which is why `doctor` checks owner,
mode,
the ancestor folders and the version the installed file prints.

The choice is separable in what it decides and not in what it edits.
If the owner prefers B or C,
the unit lines keep one shape and the path in them changes;
what is lost is the fail-closed property the review asked for,
and choice 1's ranking would have to say so for that option.
Two more things move with B or C, which the fifth revision's sentence missed:
a word-free `--version` at a file not named for the zedlet answers the wrapper, so `doctor`'s
planned installed-file check (step 18) and step 19's guest arm need the words,
and the release smoke's first arm becomes a words arm.
An AppArmor rule is what the layout makes necessary either way,
and "Steps 18 and 19" states it:
the rule in `/etc/apparmor.d/local/zed` names the file the zedlet resolves to,
so a layout that changes that file's path moves one line with it.

## Choice 4: where the dispatch sits

What a placement costs root is every start-time action after it that runs.

### Every start-time action of today's wrapper

In the order the code takes them,
with what a root-mode start does at choice 4's option A for each:

1.  `args_os` collection in `main` (`main.rs:52`).
    Read:
    the vector is what a root mode is decided from.
2.  The words check and the one-name check (this design's,
    in `root_mode_names`).
    Read:
    they are the decision.
3.  `service_main::is_service_start` (`main.rs:56`),
    which reads the arguments only.
    Skipped:
    the root-mode arm answers first.
4.  `panic_notice::install_payload_free_panic_hook`.
    Installed inside the root-mode arm under every placement,
    with which hook it is decided in "Choice 10: the panic hook a root mode installs".
5.  `vars_os` collection.
    Skipped:
    a root mode reads no variable through this collection (`entry`'s own arm would).
6.  `runtime_probe::is_requested` (`entry.rs:362`),
    a test-only variable.
    Skipped.
7.  `hooks_off::activate` (`entry.rs:366`),
    a process-wide switch.
    Skipped.
8.  `abandoned_locks::meet_the_locks_of_the_working_directory` (`entry.rs:371`):
    the working directory,
    `GIT_DIR`,
    one ancestor walk for a Git directory,
    then a record written into the common Git directory
    and any lock a proof says is abandoned removed.
    Skipped,
    and this is the one that writes:
    today's bytes create `.git/cli-git-lock-holders/` and a marker on every start of every layout,
    `--version` included
    (replay keep/today-installed-name.log).
9.  `process_resolution_inputs` (`entry.rs:372`):
    `current_exe`,
    `current_dir`,
    `PATH`,
    `PATHEXT`,
    `XDG_RUNTIME_DIR`,
    the common Git paths.
    Skipped.
10. `refusal_when_started_as_git`:
    the chain marker in the environment,
    this start's record file,
    and the process table for the parent.
    Skipped.
11. `plan_invocation`'s first check,
    the forwarded-to-self marker:
    one environment value and one file identity.
    Skipped.
12. `plan_invocation`'s control stripping and global layout,
    arguments only.
    Skipped.
13. The management arm,
    `plan_management_with_section`:
    the repository configuration,
    the hooks scan,
    and the service.
    Skipped.
14. `repository_link_stop`:
    `HOME`,
    the working directory,
    and the identity of the 860 drive link.
    Skipped.
15. `resolve_real_git`:
    the `PATH` scan,
    file identities,
    and the service unit's recorded Git.
    Skipped.
16. `invocation_overlay` and `announced_query`.
    Skipped.
17. `git_facts`,
    which starts Git.
    Skipped.
18. `shipped_checks`,
    which reads `monochromatic-git-policy-cli.config.jsonc`.
    Skipped.
19. `run_wrapped_command`:
    policies,
    the service socket,
    Git LFS,
    the linked-worktree copy.
    Skipped.
20. `forward_action` and `stack_if_push`:
    the worktree copy,
    the auto-push,
    the LFS upload,
    the network.
    Skipped.

So before it knows the mode,
the entry reads the argument vector and nothing else:
no variable of its own,
no file,
and no descriptor.
The loader's page-in of the program and of libc is the only other read,
and "What else the modes keep,
and what they lose" measures it.

What a root-mode start runs instead is today's handler:
the grammar,
the compatibility gate,
the mode,
and the two log waits at exit.
A new mode's own reads are on top of that and are the mode's business,
not the dispatch's.

### Option A: the first statements of `main`

The `#[cfg(target_os = "linux")]` block of "The rule,
in the entry",
before the service check,
before the panic hook,
and before the environment.

```text
$ cd /home/user/Monochromatic
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   the mode's own line, exit=0 or 2
   .git/cli-git-lock-holders: no marker this start made
```

The second line is a prediction,
not a run:
the block returns before `entry::run_process` is called,
which is the only route to the lock pass at `entry.rs:371`,
and every start in `keep/today-installed-name.log` that reached `run_process` created the marker (replay).

Pros:

- reaches none of the other start-time actions,
  only the first,
  so nothing of the wrapper's own start can fail before root's work;
- the checks are the two the answer names,
  so no other placement can claim to follow it;
- it is where the service mode already stands,
  which is a consistency with the entry's existing shape rather than a gain of
  this placement (design).

Cons:

- the entry now has three mode checks before the wrapper's path,
  and their order must be stated:
  the words,
  the one name (both inside `root_mode_names`),
  the service's words,
  each reading arguments only.

### Option B: after the panic hook and the environment

```text
$ <the installed file> monochromatic-git-policy-cli zfs-root --help
   the modes' usage, exit=0
   the environment: collected for a start that reads none of it
```

Pros:

- a panic in a root mode would print the wrapper's payload-free message without a second installation;
- the environment is already collected if a mode ever wants it.

Cons:

- `main` collects the whole environment for a start that reads none of it;
- no probe shows any benefit over A,
  and the hook can be installed inside the arm instead,
  which is what "The panic hook a root mode installs" recommends.

### Option C: at the top of `entry::run_process`

```text
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   the mode's own line, exit=0; `run_process` must be handed the program name too
```

"At the top" is exactly one statement:
before `runtime_probe::is_requested` at `entry.rs:362`,
and so before `hooks_off::activate` (`:348`),
the lock pass (`:353`)
and `process_resolution_inputs` (`:356`).

What that costs:

- what option B costs:
  `main` has already installed the hook and collected the environment by the time `run_process` is called,
  so a check at its top saves none of that;
  the lock pass and everything after it are skipped,
  which is option B's shape too;
- a parameter the library does not have:
  `run_process` is handed the arguments after the program name
  (`entry.rs:356`,
  `main.rs:76`),
  so a dispatch there needs the vector whole or the name beside it,
  and its second caller cannot supply either.
  `merge/merge_second_pass_finish.rs:429` (`run_nested`) calls `run_process` with the words of a Git command
  (["commit",
  ...]) and no program name,
  so the entry would need a parameter that one call site can only fill with "there is none".
  A root-mode start never re-enters there,
  so that is new state to keep honest for no measured gain.

Pros:

- one place decides everything,
  and `main` stays as thin as it is.

Cons:

- the cost is B's cost,
  not the lock pass's,
  which the first revision's text said wrongly;
- `run_process`'s signature changes,
  and the one in-process re-entry has no honest value for the new parameter;
- the entry would hand a start name into the library for a decision only the executable's own start needs.

### Option D: inside `plan_invocation`

A check made here,
where the arguments are already planned,
costs every action before it.

```text
$ <the wrapper> monochromatic-git-policy-cli zfs-root boot-reconcile
   Usage: git monochromatic-git-policy-cli check (--all | -- <pathspec>...) [--policy <id>]...
          ...
   exit=2
   .git/cli-git-lock-holders: a marker this start made
```

That block is today's output with today's bytes,
not an option D build:
the words reach the management arm,
which refuses them,
and the start has already written into the repository it stands in
(replay keep/today-lines.log).
Under D the same start would print the mode's own line instead,
after the actions of "Every start-time action of today's wrapper" had run (design).

Pros:

- `run_process` keeps its signature,
  so no other caller moves.

Cons:

- the runtime probe,
  the hook switch,
  the lock pass,
  the resolution and the started-as-Git check all run first,
  so root writes into whatever repository holds the working directory
  (replay keep/today-installed-name.log);
- `plan_invocation` reads the arguments after the program name,
  so the decision needs the program name handed down to it:
  `main` must pass a slice or a name it does not pass today,
  which is one more pinned line,
  and the entry pin fixes the one line that names the vector today
  (source package/git-policy/cli/src/native/main.rs:52,
  and this round's replay shows every added line failing the pin,
  run keep/guards-replay-sixth.log);
- `git_facts`,
  the configuration read and the forwarded Git are one branch away.

### The panic hook is a choice of its own

The hook a root-mode start installs is not part of this placement's ranking:
it is asked on its own in
"Choice 10: the panic hook a root mode installs",
which prints both hooks' lines and ranks them.
Every option here installs whichever hook that choice picks,
as the arm's first statement.

### Ranking of choice 4

Ranking:
A > B > C > D.

A over B:
B collects an environment nothing reads,
and the hook is installed inside the arm under every placement
("Choice 10: the panic hook a root mode installs"),
so B's only gain is gone.

B over C:
they cost the same start-time work,
so the ranking is about where the code lives:
B keeps the decision in the executable's entry,
which is where the answer's precedent (`service run`) puts it and where the vector already is,
while C moves it into the library on a parameter that its one in-process caller cannot fill.

C over D:
D adds the lock pass,
the resolution and the started-as-Git check before the mode,
and it is the only option whose cost grows with the caller's working directory.

The choice is separability-limited in one direction and clean in the other,
and the fifth read was right that it is not clean under B.
A and B both put the call in `main`,
but the pin compares the file's code lines in order,
so B's moved block is a changed pinned line:
A and B re-pin the same file,
and B is not a free move.
C puts the call in `run_process`,
so the vector must reach the library and `main`'s pinned text changes as it does under D:
`plan_invocation` and `run_process` are handed the arguments after the program name,
so a check there needs the program name passed down,
which is a fourth vector line in `main`.
That is a change to the pin under C, not only under D.
What D would still need is a reason the lock pass and the started-as-Git check run for a root start.

## Choice 5: what `--version` and the refusals say

The modes' grammar,
its refusals and its version line name the program
`mount-git-worktree-clone-datasets`
(`handler_install_paths.rs:28`).
After the change no root-owned file need carry that name,
while these compare a line:
`handlerVersionLine` (`bin/native-install-steps.ts:121`),
`native:install`'s comparison,
the release smoke's `grep`,
the stand-in handler's line,
the modes' own refusal texts,
and `doctor`'s check, which reads no version line today
(replay keep/host-facts-fourth.log;
`rg -- '--version|version_line|handlerVersionLine' src/native/health/` finds nothing,
and `handler_checks.rs:229` reads owner and mode).

### Option A: keep the modes' own name

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
mount-git-worktree-clone-datasets 0.6.1-alpha.45
```

(The line the built handler prints today:
`./handler --version` answers `mount-git-worktree-clone-datasets 0.6.1-alpha.45`,
exit 0,
(replay keep/handler-usage.log);
no installed file answers it yet,
and `keep/today-installed-name.log` shows (replay)
today's bytes answering `--version` with Git's own line instead.)

Pros:

- the line does not move when choice 2's word is renamed,
  because it names the mode family rather than the words;
- the line is unique against the wrapper's own banner,
  which is `monochromatic-git-policy-cli 0.6.1-alpha.45, shadowing git at /usr/bin/git`,
  so a check can tell "this build has the root modes" from the wrapper.

Cons:

- a person reads a name no command and no file has,
  so every refusal must also print the words and the installed path,
  and the version line names nothing anyone can run;
- four existing readers keep their string,
  which is the upkeep the ranking states as a cost and does not rank by.

### Option B: print the wrapper's name and a modes line

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
monochromatic-git-policy-cli 0.6.1-alpha.45
root modes: boot-reconcile, shutdown-unmount, start-probe, holder-scan, adopt-worktree-parent, record-checkout, mount-path-descendant, the sweep
```

Pros:

- the first line names the program a person runs.

Cons:

- its first line collides with the wrapper's own `git --version` banner,
  which already begins `monochromatic-git-policy-cli 0.6.1-alpha.45,`,
  so a check on the installed file's first line could no longer tell which program answered;
- `native:install`,
  `doctor` and the smoke all need new text,
  and the modes' refusals need a new program name.

### Option C: print the words and the version

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
monochromatic-git-policy-cli zfs-root 0.6.1-alpha.45
```

The line is written and not run (design),
and no build answers the words yet.

Pros:

- the line is a spelling a person can type,
  so the answer's "a name no command has" objection to A is answered;
- it is unique:
  the wrapper's banner continues with the version and a comma,
  and this line continues with the group word,
  so one `grep` tells them apart;
- a refusal can begin with the same words and stay findable.

Cons:

- the group word (choice 1's rule,
  choice 2's pick) becomes part of the version line,
  so renaming it changes the one line three checks read;
- every refusal's opening grows from one 33-character family name to two words and a space,
  and each refusal is already two sentences long;
- `handlerVersionLine`,
  `native:install`'s check,
  the smoke's `grep`,
  the stand-in handler and
  `doctor`'s planned check all compare a new string,
  which the change list rewrites under every option anyway.

### Ranking of choice 5

Ranking:
C > A > B,
by what each line delivers.

C over A:
the two deliver the same three things a check needs.
Each is one line,
each is unique against the wrapper's banner
(`monochromatic-git-policy-cli 0.6.1-alpha.45, shadowing git at ...`)
because each continues with something the banner does not have,
and each lets `native:install`, the release smoke and `doctor` tell "this build answers the root
modes" from "this is the wrapper".
C delivers two things A cannot:

- an opening spelling a person can type.
  A's line names `mount-git-worktree-clone-datasets`,
  which after this change is no file and no command;
  C's line is the words,
  so a person who reads it in a refusal can run what it names,
  and the refusal's opening words, the unit lines and the version line are one spelling;
- a refusals set that needs no second name.
  Under A every refusal must open with the family name and also print the words and the installed
  path, because the family name alone names nothing runnable;
  under C the opening words are the runnable spelling and the path is the one thing left to print.

What A delivers that C does not is that its line does not move when choice 2's word is renamed,
and that four existing readers compare the same string they compare today.
The second of those is upkeep and fidelity to what exists, which the repository's ranking rule
says to state as a cost and never to rank by:
"Score capability for the target workload, never fidelity to incumbents",
and "Rank options by what each delivers for the target workload; effort to build, raw system calls
or a new code path are costs to state, never reasons to rank an option lower" (`AGENTS.md`).
What remains is the first, and it is a cost of a rename that choice 2 already decides once,
paid by one line in each of three checks that the change list rewrites under either option.
C's cost is the longer opening in every refusal (two words and a space more than a 33-character
name),
which the design already requires to carry the words and the path.

C over B:
B's first line is a prefix of the wrapper's own banner,
so a check on the installed file's first line cannot tell which program answered,
and a second line would have to be read instead.
C's first line carries a word the banner never has.

The choice is separable.
Whichever option wins, the change list rewrites `native:install`'s check,
drops the stand-in handler,
adds the two smoke arms and writes `doctor`'s planned check;
under C those four compare the words' line rather than the family's,
which is the same edit in four places and not a change to the dispatch,
the layouts, the guards or the modes.

## Choice 6: whether the account's `PATH` keeps a short name

Before the change the toolchain installed the second program,
and this host's `PATH` no longer holds it:
`command -v mount-git-worktree-clone-datasets` prints nothing and exits 1,
while the control `command -v git` prints the wrapper's own path
(replay keep/host-facts-fourth.log).
So nothing named there is lost by removing the second program,
and after the change the account's toolchain installs one program.
A root mode needs the words in any case.

### Option A: the installed path and the words, printed by `doctor`

```text
$ sudo /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
    --checkout /home/user/Monochromatic --device 66306 --inode 1234567
   decides: root-mode grammar="adopt-worktree-parent --checkout /home/user/Monochromatic --device 66306 --inode 1234567"
            (the compiled module under `git` with these words, run keep/names-starts-fourth.log;
            dispatch case 25 has the same verdict)
```

and a person's read-only questions through the account's own wrapper:

```text
$ git monochromatic-git-policy-cli zfs-root --help
   decides: root-mode grammar="--help"
            (the compiled module under `git`, run keep/names-starts-fourth.log,
            and dispatch case 6 through the words)
   prints:  the modes' usage, the words' spelling, and the root-owned path (proposal)
```

`doctor` prints the adoption line with the checkout's device and inode filled in,
beside the three unit lines the setup writes,
and it prints the root-owned path,
never `sudo git monochromatic-git-policy-cli …`,
whose `git` is the account's file.

Pros:

- the path a person types is the file root's units run,
  so `--version` describes the bytes that will act;
- `doctor` already prints concrete commands it never runs;
- no other package is involved,
  and a `sudo` that preserves `PATH` finds no root-mode file there.

Cons:

- the long command,
  and every refusal inside the modes that names a follow-up command must be rewritten to it.

### Option B: the toolchain links a short name beside `git`

```text
$ sudo zfs-root monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
    --checkout /home/user/Monochromatic --device 66306 --inode 1234567
   decides: root-mode grammar="adopt-worktree-parent --checkout /home/user/Monochromatic --device 66306 --inode 1234567"
```

(the link is the account's;
the words are still needed,
which is why they appear twice)

Pros:

- shorter than the installed path,
  so a person types fewer characters.

Cons:

- the link points at the account's copy,
  so the version a person reads describes a different file from the one root would run;
- a `sudo` that preserves `PATH` runs an account-writable file as root,
  which is one more path to an escalation this design does not need;
- the toolchain's plan expresses one post-install link per tool
  (`mpm-shadow`'s `postinstall.link` takes `sourceBin`,
  `linkName` and `linkDir`),
  so a second link needs that package's schema to change first,
  which is a cost to state and not the reason the ranking puts B second.

### Ranking of choice 6

Ranking:
A > B.

A over B:
A keeps one file behind every name,
and the thing a person reads is the thing root runs;
B's saving is typing,
and it moves the thing a person runs away from the thing root runs,
while adding an account-writable file that a `sudo` can execute.
B should come back as its own proposal if the long path bites in use,
and it should then link to root's copy.

The choice is separable.
If the owner prefers B,
one toolchain plan row and one `native:install` step change,
and nothing about the mode words,
the layouts or the guards does;
what changes is which file a person's command names.

## Choice 7: whether the release keeps the modes' test-only variables

The modes' code reads five variables,
all compiled into the release and all inert unless a control sets them
(`src/native/start_probe/probe_test_switch.rs:29` and `:35`,
`src/native/start_probe/probe_deadline.rs:69`,
`src/native/foundation/diagnostic_log.rs:40` and `:67`).
Four of them can only skip, refuse or narrate;
the fifth is asked on its own in
"Choice 11: whether the release honours the probe's deadline variable",
because it can turn a missed deadline into a pass.

The four this choice is about:

- `CLI_GIT_TEST_ONLY_START_PROBE`,
  which can disable the probe or make it hold a step;
- `CLI_GIT_TEST_ONLY_START_PROBE_KEEP_DESCRIPTOR`;
- `MONOCHROMATIC_VERBOSE` and
  `MONOCHROMATIC_WARN`,
  which change what is logged and nothing that is decided.

A sixth,
`CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS`,
is read only by the lazy service runtime,
which no root mode builds today.

What each of the four can do when a unit or an operator sets it,
from the code's own text:

- `disabled` records a skipped probe and enables nothing
  (`probe_test_switch.rs:5` to `:14`);
- `hold-<step>` makes the worker hold a step past the deadline,
  so the mode records a timeout and fails closed (`:56` to `:60`);
- keeping the descriptor makes the unmount arm see `EBUSY`,
  which refuses the unmount rather than completing it;
- the two verbosity variables change which lines are written and nothing else
  (`diagnostic_log.rs:40` and `:67`).

So none of the four can turn a refusal into a pass.
That is the direction the fourth revision's pro claimed for all five,
and it is true of these four
(design,
from the four code statements in this choice;
no arm of this round set one as root).

ZED cannot set any of them:
it builds the environment from the event and a fixed list,
and the event's own variables carry the `ZEVENT_` prefix
(replay keep/zed-shape-report.txt).
A unit could set them with `Environment=`,
and so could anyone who edits the unit,
which is root.

### Option A: keep them

A control starts the installed file with a switch set and reads the record it wrote:

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=disabled <installed file> monochromatic-git-policy-cli zfs-root start-probe
   today, as the account, the mode refuses before the switch is read:
   the start-probe mode runs as root, and this process's effective user is 1000. Nothing was read, mounted
   or recorded, because the record it writes decides whether a boot's clones are admitted for attachment,
   and an account that could write it could report a boot that never passed. Ways forward: run the mode as
   root, from the boot unit or by hand.
   as root, the record's verdict is skipped with its skip reason `test-switch`.
```

(printed today by the built handler;
the root arm is the control's and is not run here)

Pros:

- the modes' controls keep running against the release build,
  which is what makes their guest arms evidence;
- each of the four at worst skips a record, refuses later or writes a line.

Cons:

- a privileged program has switches nobody but a control needs.

### Option B: compile them out of the release

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=disabled <installed file> monochromatic-git-policy-cli zfs-root start-probe
   the probe runs as it always does; the record carries no skip reason, because nothing read the variable
```

The line is written and not run (design),
and the switch is what a feature-less build would ignore.

Pros:

- the release binary holds no switch a test names.

Cons:

- a second build shape,
  so the guest arms would test a different binary from the released one,
  which is the reason the ranking puts B second;
- the crate already has one such precedent (`--features measurement-routes`),
  and a second would need its own tasks and lint passes,
  which is a cost to state beside that reason.

### Ranking of choice 7

Ranking:
A > B.

A over B:
keeping them preserves the one property that makes the guest arms evidence,
that the binary under test is the binary released;
B's gain is a release binary without test switches,
and its cost is a build shape whose arms test something else.
No probe here measures either direction,
and the design says so rather than claiming a measurement it did not make
(it shows ZED cannot set one,
which is neither direction's evidence;
replay keep/zed-shape-report.txt).

The choice is separable,
in one direction,
and it is separable from choice 11.
Keeping the variables changes nothing but the release build's contents,
so it can be decided on its own at any time.
Compiling them out is not separable from the guest arms:
each guest arm sets a switch,
so B needs `e2e/start-probe-guest.ts` to build and test the feature build instead,
and the arms' evidence changes from the release's bytes to a second build's.
The deadline variable's answer moves that one switch's arms either way,
and nothing of these four changes with it.

## The two modes these answers add

Two accepted answers need modes this design did not have when the second revision was written.
Both are modes of the fold this design makes,
so their words,
arguments and refusals are this design's to state
(record doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md
and doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md).

### What the grammar promises, and what these two keep

The grammar's principle,
as the build keeps it
(source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_arguments.rs:319):

- one leading word names the work,
  and the words are a closed list whose refusal names every member;
- every input is `--name <value>`,
  each option as a pair,
  in any order,
  which is what `parse_adoption` and `parse_holder_scan` do
  (source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_arguments.rs:393
  and :457),
  and a missing value,
  a repeated option or an unreadable argument refuses rather than guesses;
- an empty argument list is the attaching sweep,
  which is ZED's own start and nothing else;
- a mode that did its work answers 0,
  and every refusal answers 2 (`EXIT_REFUSED`),
  after a line saying what was not done and every way forward.

Both new modes meet that principle.
They also meet a rule the old modes did not have to:
each one reads or writes one root-owned record,
and the second reads what the first wrote.

### `record-checkout --checkout <path> [--replace <the kernel's guid, which the refusal prints>]`

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
recorded: /home/user/Monochromatic is <its dataset> (GUID <number>), in pool <pool>,
beside container <container>; the record is <the record's path>
```

The line's words are this revision's,
and no build prints it (proposal).
What the module decides is measured:
the words hand the grammar exactly
`["record-checkout", "--checkout", "/home/user/Monochromatic"]`
(replay keep/names-starts-fourth.log),
and cases 27 and 28 of the dispatch probe show the same for both modes
(replay keep/dispatch-cases.log).

What it records:

- the checkout's path;
- its dataset's name,
  read from the kernel's table;
- that dataset's GUID,
  read from the kernel's own statistics for it;
- the container dataset the configuration names,
  or the one beside the checkout when the configuration leaves it out;
- the pool,
  which is the dataset name's first component;
- the record format's version,
  which is this design's one field that step 10's build may change.

The record's one path is
`/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json`,
named here once and cited by name everywhere else.

So the record is one root-owned JSON object of six fields,
and the third revision left its fields to "step 10's build".
The reader still lands in step 10's build,
which owns the version bump when it adds a field;
this design owns the meaning of those six fields,
because its mode is the writer.

The file is a root-owned `0644` file,
which is the mode the step 10 design names
(source doc/planning/cli-git-zfs-step-10-open-choices.md:613,
and the reader `read_file_only_root_may_write_owned_by(path, 0)` at its "What E costs").
Its folder is `/var/lib/monochromatic-git-policy-cli`,
beside the slot identities (source
package/git-policy/cli/src/native/worktree_clone_mount/slot_identity_store.rs:44),
which this host does not have yet
(replay keep/host-facts-fourth.log).
The mode creates the folder `0755`,
root's,
when it is absent,
and refuses a folder that exists and is not root's or is group- or other-writable;
step 19's setup runs the same `install -d` line before the first record,
so a host that never runs the mode still has the folder a runbook creates.

What it proves before it writes:

- `--checkout` is text and an absolute path,
  or the grammar's own refusals answer;
- the path is a mount point of a ZFS dataset,
  and that dataset's `mountpoint` property is the path itself,
  which is the precondition step 10's design states
  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
  "The checkout path must be its dataset's own mount point");
- the container is a dataset in the same pool,
  and neither contains the other:
  the checkout's dataset is not inside the container,
  and the container is not inside the checkout
  (the route places the container beside the checkout,
  and each direction is proved and refused separately);
- the caller is root,
  through the same refusal `start-probe` and the adoption answer a caller with
  (`handler_mode_dispatch.rs:174`,
  `adopt_worktree_parent_mode.rs:315`);
- the record's folder and file are as "What it records" states them,
  or the mode makes the folder;
- the compatibility gate answers before the first `/dev/zfs` request (`run_behind_at`,
  `handler_mode_dispatch.rs:317`).

Its refusals,
each naming what was not done and the way forward:

- `checkout-not-a-mount-point`,
  because the add's gate would compare the record against a folder rather than a mount;
- `checkout-not-on-zfs`,
  because the route serves ZFS and a record for anything else would make the gate pass on a checkout
  it cannot clone;
- `container-not-beside-the-checkout`,
  naming the direction the proof found:
  a container inside the checkout,
  or a container that holds the checkout;
- `checkout-record-differs`,
  when the record already exists and a proved field differs,
  naming that field and the flag that replaces the record;
- `checkout-record-unwritable`,
  when the folder or the file cannot be root's;
- `replace-token-differs`,
  under choice 9's recommended option E,
  when `--replace`'s value is not the GUID the kernel shows,
  naming both numbers and what to do;
- `nothing-to-replace`,
  under the same option,
  when `--replace` is given and no record stands,
  because then there is nothing to replace and the ordinary command records one.

A second run over the same checkout,
with every field equal,
answers 0 with an
`already recorded: <the record's path>, and every field agrees with the kernel` line and writes
nothing,
so a runbook run twice is harmless,
and a `--replace` whose value leaves every field equal answers the same line and writes nothing.
A run whose proved field differs refuses with `checkout-record-differs`,
naming the field and the exact `--replace` command;
with that command it prints every field that changes,
the old value and the new,
and writes.
Which value the flag takes,
and whether it should need a flag at all,
is choice 9.

### `mount-path-descendant --dataset <name>`

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root mount-path-descendant --dataset zpcachyos/nosnap/vm-builder-output
mounted: zpcachyos/nosnap/vm-builder-output at /home/user/Monochromatic/package/vm-builder/cli/output
```

```text
$ … mount-path-descendant --dataset zpcachyos/nosnap/vm-builder-output
already mounted: zpcachyos/nosnap/vm-builder-output at /home/user/Monochromatic/package/vm-builder/cli/output
```

No build prints either line (proposal).
The dataset and its mountpoint are the ones the step 19 record names
(record doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md),
and this host's table shows them as the record does
(run keep/host-facts-sixth.log).

What it does,
in this order,
so that every act acts on an object it has proved rather than on a path it has read:

1.  reads the checkout's record at its one path,
    and refuses `no-recorded-checkout` when there is none;
2.  proves the record against the kernel:
    the recorded path is a mount point,
    the dataset mounted there is the record's dataset,
    its GUID is the recorded GUID,
    and the container is the record's,
    or refuses `checkout-record-no-longer-matches` naming the field that differs;
3.  proves from the mount table that the recorded dataset is mounted at the recorded path now,
    and opens that folder:
    `open_root` of the recorded path,
    then `mount_id_of` and `observed_path` of the descriptor,
    so the folder it holds is that mount's root and not a folder an account renamed onto the path
    (source package/git-policy/cli/src/native/subvolume/fs_beneath.rs:195,
    `subvolume/fs_beneath_mount_facts.rs:38` and `:97`).
    A path the table does not show mounted refuses `checkout-not-mounted`:
    a record that agrees with the kernel's dataset properties is not a mounted checkout,
    and this is the proof the step 19 acceptance list needs when a mount is delayed or fails;
4.  reads the dataset's own properties and refuses before it mounts anything:
    `mountpoint`,
    `mounted`,
    `canmount`,
    `zoned`,
    `redacted`,
    `overlay`,
    `encryption`,
    `keystatus`,
    `atime`,
    `relatime`,
    `devices`,
    `exec`,
    `readonly`,
    `setuid` and
    `nbmand`
    (the last seven are the pairs upstream's mount derives its options from,
    upstream lib/libzfs/libzfs_mount.c, zfs-2.4.4, `zfs_add_options` at `:355` to `:381`);
5.  walks the mountpoint's components below the checkout's opened mount,
    one descriptor at a time,
    with the crate's own walk:
    `open_chain_beneath` refuses a link, a magic link, a hop on another device and a mount
    anywhere but the last component,
    and `open_directory_at_a_mount` allows that last one
    (source package/git-policy/cli/src/native/subvolume/fs_beneath_walk_chain.rs:140 and `:187`;
    the clone walk documents the same discipline at
    `worktree_clone_mount/clone_slot_walk.rs:38`).
    A missing last component is created `0755` and reopened with `O_NOFOLLOW`;
    a missing intermediate component refuses `mountpoint-unusable`,
    because this mode creates the mountpoint and never a tree of its own above it (design);
6.  mounts with the primitive the route already uses for its clones,
    not with a started `zfs` program and not through libzfs:
    `fsopen("zfs")`,
    `fsconfig` naming the dataset as `source`,
    `fsconfig` creating the superblock,
    `fsmount` with the attributes the pairs above give,
    and `move_mount` onto the opened folder's descriptor with
    `MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH`,
    which is the call `attach_the_clone` makes for a clone
    (source package/git-policy/cli/src/native/worktree_clone_mount/attach_the_clone.rs:341 to `:359`);
7.  answers the `mounted:` line and status 0,
    or,
    when the mount table already shows the dataset at its own mountpoint,
    answers the `already mounted:` line and status 0 without writing anything,
    which is what makes an `ExecStart=` re-run harmless.

What the mount's attributes are,
on this host's dataset:

- upstream's mount derives one option from each of the seven pairs and skips the `relatime` pair
  when `atime` is off (`:365` to `:369`),
  which this dataset's `atime=off` and `relatime=off` make its case;
- so the resolved set is `noatime,dev,exec,rw,suid,nonbmand`,
  which is the properties `host-facts-sixth.log` measures read through upstream's order above,
  and `dev`, `exec`, `rw` and `suid` are the kernel's defaults,
  so the one attribute this mode passes to `fsmount` is `MOUNT_ATTR_NOATIME`;
- `nbmand` has no kernel attribute on Linux,
  so the pair is read and mounted as the no-op it is,
  and the mode says so rather than passing an option nothing reads.

What it refuses,
each naming what was not done and the way forward:

- `no-recorded-checkout`,
  because without the record the mode cannot prove the mountpoint descends from a served checkout;
- `checkout-record-no-longer-matches`,
  naming the field that differs and printing both values when the field is the GUID,
  when the record's path,
  dataset,
  GUID or container no longer agrees with the kernel,
  because every reader of the record refuses one that is stale and this mode reads it whole;
- `checkout-not-mounted`,
  when the mount table does not show the recorded dataset at the recorded path now,
  which is a different fact from the record's own agreement with the kernel;
- `dataset-outside-the-checkout`,
  which is the refusal the step 19 design names
  (`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`:1009 to `:1013`):
  the dataset's `mountpoint`,
  canonicalised,
  is not strictly under the recorded checkout's path,
  or the dataset is the recorded checkout's own dataset.
  "Canonicalised" means the path's `.` and `..` components are resolved and its repeated slashes
  folded before the comparison,
  and a mountpoint whose own directory does not exist yet is compared as the text it is,
  then created by step 5's `mkdirat`:
  a missing directory is created, never guessed at.
  The refusal names the dataset,
  as that design asks.
  The dataset's name is not compared against the recorded dataset's name at all:
  the step 19 record's dataset is a sibling of the checkout's dataset in the same pool,
  and a rule that asked for descent by name refused exactly the dataset the owner's answer names;
- `dataset-in-another-pool`,
  which this design adds:
  the dataset's pool is not the record's pool.
  The checkout and every dataset under it are in one pool,
  and a foreign pool's dataset mounted under the checkout would put another pool's subtree inside a
  served checkout.
  It costs one comparison,
  and the owner may drop it with one sentence;
- `dataset-without-a-mountpoint`,
  when the property is `none`,
  `-` or `legacy`,
  because such a dataset is mounted by someone else's rule and this mode would be guessing;
- `dataset-already-mounted-elsewhere`,
  when the kernel shows the dataset mounted at a path other than the one its property names;
- `dataset-not-mountable`,
  naming the property and its value,
  when `canmount` is `off`
  or `zoned` is `on` in the global zone
  or `redacted` is `on`,
  which are the three `zfs mount` answers by returning success and mounting nothing
  (upstream lib/libzfs/libzfs_mount.c `:299`, `:270` and `:305`):
  a unit whose line reports success and leaves the dataset unmounted is the failure this refusal
  replaces;
- `encryption-key-not-loaded`,
  when the dataset is encrypted and its `keystatus` is not `available`,
  because loading a key is a separate act and a unit has nobody to prompt.
  Upstream tests `encryption` before `keystatus` (`:455` to `:498`),
  and an unencrypted dataset's `keystatus` is `-`,
  so the refusal applies to an encrypted dataset only;
- `mountpoint-not-empty`,
  when `overlay` is off and the mountpoint holds anything,
  which is upstream's own "directory is not empty" refusal (`:521` to `:538`);
- `mountpoint-unusable`,
  when a hop of step 5's walk is a link or a folder on another device,
  when an intermediate component is missing,
  or when the last component exists and is not a directory.
  Ownership is deliberately not one of its conditions:
  the mountpoint of this host's nested dataset is `user:user 755`
  (run keep/host-facts-sixth.log),
  as the plan's migration leaves it,
  because that migration copies "the copy that preserves ownership"
  (source doc/planning/cli-git-zfs-worktree-implementation-plan.md:2591),
  and a clause that demanded root's ownership would refuse the one dataset the step 19 record
  names and leave it unmounted at every boot.
  The safety comes from step 5's descriptors instead:
  the mount lands on the folder the walk opened beneath the checkout's own mount,
  not on a path an account could rename.

It changes no other dataset,
mounts nothing under a dataset it was not given,
and unmounts nothing (that is `shutdown-unmount`'s work).

What it does to the one nested dataset at this host's boot,
after the migration and the first record:
the dataset is not mounted when the unit starts (`canmount=noauto`),
the record agrees with the kernel,
the checkout's mount is open and the record is its root,
the dataset is in the record's pool and its mountpoint lies strictly under the checkout's,
its key is available,
the mountpoint's hops are the account's own plain folders,
so no refusal fires and the mode mounts it with `noatime`.
The ownership clause the fifth revision printed would have refused it and left it unmounted.

### The question the step 19 design leaves to this one

That design's "What each part waits on" says the mode needs this design to say
"whether a mode may change a dataset's state by name at all"
(`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
What each part waits on).

The answer is yes,
bounded by the record:

- a mode may mount one dataset by name when the dataset's own `mountpoint` lies strictly under the
  recorded checkout's path,
  the dataset is not the checkout's own dataset,
  the dataset is in the record's pool,
  the record still matches the kernel,
  and the recorded checkout's dataset is mounted at its recorded path now,
  so the mount lands inside a mount the route can place it under rather than inside a folder an
  account could have renamed;
- a mode may not unmount,
  destroy,
  snapshot or change a property of a dataset by name,
  and nothing in this design's two modes does;
- the record is the only thing that makes "under the checkout" a proof rather than a guess,
  which is why this mode refuses before `record-checkout` has run.

### The conflicts the pair carries

#### `--checkout` on two modes

`adopt-worktree-parent --checkout <path>` records the replacement `.claude/worktrees` folder's identity;
`record-checkout --checkout <path>` records the checkout's route facts.
Both name the same path in practice,
and the mode word is what says which proof is made.
The usage line of each names its own record,
and `doctor` prints the adoption's line for an adoption and the record's line for the record,
so a person copying the wrong one reads a line about the other job.

#### `--dataset` on two modes

`holder-scan --dataset <name>` reports which mount namespaces hold a clone;
`mount-path-descendant --dataset <name>` mounts one nested dataset.
Both take a full dataset name and both refuse a malformed one through `ArgumentRefusal::DatasetName`,
and the refusal's sentence is per mode now,
because the shared one said "the clone's full dataset name" while the nested dataset is not a
clone
(source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_arguments.rs:239).

#### Two sources for the checkout

The configuration file can still name `cloneMounts.checkout` today
(source package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_configuration.rs).
Once the record exists,
the record is what the route serves,
and a configuration value that disagrees is a refusal naming both,
not a silent choice.
That reader change is step 10's build's,
with the format.

#### A mode that needs another mode's output

`mount-path-descendant` refuses until `record-checkout` has run.
That is a new ordering between modes,
and it is stated in both usage lines and in `doctor`'s output,
so a boot that runs the nested dataset's unit before the record exists reads `no-recorded-checkout`
rather than mounting a dataset it cannot place.

## Choice 8: the word of the mode that records a checkout

The word is the one thing here the answers do not fix,
so it is posed like choice 2.
The three candidates are the same command under a different name.

Option A,
`record-checkout`:

```text
$ git monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
   decides: root-mode grammar="record-checkout --checkout /home/user/Monochromatic"
            (measured for `record-checkout` only: run keep/names-starts-fourth.log,
            which starts the compiled module under `git` with these words)
   prints:  the `recorded:` line of "The two modes these answers add" (proposal)
```

Pros:

- names the thing that is recorded, which the add, the sweep and `mount-path-descendant` read;
- matches the record's own file name and the option it takes (`--checkout`);
- reads as a verb and an object, like every other mode word.

Cons:

- none beyond the habit of reading `record` as the route's word rather than the file's.

Option B,
`record-route`:

```text
$ git monochromatic-git-policy-cli zfs-root record-route --checkout /home/user/Monochromatic
   decides: root-mode grammar="record-route --checkout /home/user/Monochromatic"
            (the module hands the grammar everything after the group word, so this is the decision
            measured for `record-checkout`, not a run of this spelling)
   prints:  the same line, with the word `record-route` (proposal)
```

Pros:

- names the whole route, so a reader who knows the route's writing needs no second noun.

Cons:

- repeats what the group word `zfs-root` already says;
- does not match the record's own file name.

Option C,
`checkout-record`:

```text
$ git monochromatic-git-policy-cli zfs-root checkout-record --checkout /home/user/Monochromatic
   decides: root-mode grammar="checkout-record --checkout /home/user/Monochromatic"
            (same label as option B's)
   prints:  the same line, with the word `checkout-record` (proposal)
```

Pros:

- names the file's own subject first, which is what a person looks for when reading a folder of
  records.

Cons:

- a noun phrase, where every other mode word is verb-led.

Ranking:
A > B > C.

A over B:
A names the thing that is recorded,
which is what the add,
the sweep and `mount-path-descendant` read,
and B names the route,
which the group word `zfs-root` already said.
What A delivers is a mode whose word a person can match to the record file's name (design).

B over C:
both are two words about the route or the record,
and B reads as a verb and an object,
while C reads as a noun phrase,
which the grammar's other words never are (`boot-reconcile`,
`start-probe`,
`holder-scan`,
`adopt-worktree-parent`).
The difference C costs is the habit a reader already has (design).

The choice is separable:
the word is one member of `MODE_WORDS`,
one line of the usage and one of the help,
one match arm and its refusal texts,
the step 10 build's reader,
and `doctor`'s printed line.
Nothing about the record's shape or the other modes changes.

## Choice 9: the record a legitimate restore replaces

The step 10 design's own reason for the record is that a dataset re-created under the recorded name
is refused at the GUID comparison,
and the same sentence admits its price:
a legitimate restore gives the checkout dataset a new GUID,
"and E refuses the add until root re-adopts the checkout with the command"
(source doc/planning/cli-git-zfs-step-10-open-choices.md:578 to :579).
So the command must be able to replace a record,
and this choice decides what an operator does to replace one,
and what the ledger of that act is.

What a stale record costs,
which every option here pays in a different coin:
after a restore the record names a GUID the kernel no longer shows,
so `mount-path-descendant` refuses `checkout-record-no-longer-matches` at the next boot,
and the nested dataset's unit leaves
`zpcachyos/nosnap/vm-builder-output` unmounted until root replaces the record.
A unit that refuses has no fallback,
and the step 19 record's V6 says a unit that `Requires=` the nested one fails with it.
So each option states what it shortens that window to,
and "The order the nested dataset's unit changes in" states where the repair belongs.

Every option runs the same proofs before it writes:
the path is the dataset's own mount point,
the container is beside it in the same pool,
the caller is root,
the record still agrees with the kernel's dataset properties,
and the folder and file are root's.
They differ in what the operator types,
in what a wrong or remembered value does,
and in what is written down about the swap.

A GUID on this host is 18,
19 or 20 digits
(run keep/host-facts-sixth.log),
not the nineteen the fifth revision printed.

Option E,
a replacement that names the dataset being admitted,
`--replace <the kernel's guid>`:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
checkout-record-differs: the record's guid is 1234567890123456789 and the kernel shows 9876543210987654321.
Nothing was written. Ways forward: run this command again with `--replace 9876543210987654321` to
record this dataset, or restore the dataset the record names, which answers the recorded GUID again.
   the refusal names the flag and the value the kernel shows (proposal)

$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic --replace 9876543210987654321
replaced: /home/user/Monochromatic keeps its path, dataset, container and pool; its GUID was
1234567890123456789 and is now 9876543210987654321. The record is <the record's path>.
   the typed value is the one written, and the old one is printed beside it (proposal)

$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic --replace 1
replace-token-differs: the kernel shows 9876543210987654321, and `--replace` was given 1.
Nothing was written. Ways forward: read the kernel's guid from this refusal and pass it whole,
or run the command without `--replace` to read the difference again.
   a replacement typed from memory refuses (proposal)
```

Pros:

- the token is the value the act writes,
  so the record this command writes is the dataset this command named:
  a dataset that changed again between the refusal and the command refuses,
  and the journal's `is now` value is the operator's own token rather than whatever the kernel
  showed at run time;
- it binds the act to the thing the design's GUID check exists for,
  a dataset a restore re-created,
  and the refusal that prints the token is a statement about the kernel,
  which is where the difference was found;
- it fits the grammar's `--name <value>` pairs like every other option,
  and a `--replace` with no record at all refuses `nothing-to-replace`.

Cons:

- the operator copies an 18 to 20 digit number at the worst moment,
  which is when a person is least willing to;
- `replace-token-differs` and `nothing-to-replace` are two more refusals and one more usage line;
- a record another root act replaced in the meantime answers `already recorded` rather than
  refusing,
  because the record now agrees with the kernel;
  that is the one case where option D refuses and this one does not.

Its stricter form,
if the owner wants both sides named,
is `--replace <the record's guid> --with <the kernel's guid>`:
two tokens,
both compared,
so a change in either the record or the dataset between the refusal and the command refuses.
It delivers that extra refusal for a race whose outcome is already correct under the single token,
and it doubles what an operator copies (design).

What it makes of the stale-record window:
the repair is one command before the reboot,
whose token the boot's own refusal prints,
so the window is as short as option D's.

Option D,
a replacement that names the value it replaces,
`--replace <the record's guid>`:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
checkout-record-differs: the record's guid is 1234567890123456789 and the kernel shows 9876543210987654321.
Nothing was written. Ways forward: run this command again with `--replace 1234567890123456789` to
replace this record, or restore the dataset the record names, which answers the recorded GUID again.
   the refusal names the flag and the record's own value (proposal)

$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic --replace 1234567890123456789
replaced: /home/user/Monochromatic keeps its path, dataset, container and pool; its GUID was
1234567890123456789 and is now 9876543210987654321. The record is <the record's path>.
   every other field equal, the one that changed printed old then new (proposal)

$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic --replace 1
replace-token-differs: the record's guid is 1234567890123456789, and `--replace` was given 1.
Nothing was written. Ways forward: read the record's guid from this refusal and pass it whole,
or run the command without `--replace` to read the difference again.
   a replacement typed from memory refuses (proposal)
```

Pros:

- the token is the record's own value,
  so the act is a compare-and-swap against the record being replaced:
  a record somebody else replaced in the meantime refuses,
  and a `--replace` typed from memory cannot admit anything;
- the ledger keeps the old GUID beside the new one,
  and the refusal prints the exact word to type,
  so the way forward is one copy;
- every option in this grammar is a `--name <value>` pair,
  and this one is too.

Cons:

- the operator copies an 18 to 20 digit number at the moment of a restore;
- the value written is whatever the kernel shows when the command runs,
  so a second restore between the refusal and the command writes a dataset the operator never saw,
  and the journal's `is now` value is not the one the command named;
- `replace-token-differs` and `nothing-to-replace` are two more refusals and one more usage line.

What it makes of the stale-record window:
the repair is one command before the reboot,
whose token the boot's own refusal prints.

Option A,
a named replacement,
`--replace` as a bare flag:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
checkout-record-differs: the record's guid is 1234567890123456789 and the kernel shows 9876543210987654321.
Nothing was written. Ways forward: run this command again with `--replace` to record this dataset, or
restore the dataset the record names, which answers the recorded GUID again.
   the remove-and-re-record was refused, and the refusal names the flag (proposal)

$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic --replace
replaced: /home/user/Monochromatic keeps its path, dataset, container and pool; its GUID was
1234567890123456789 and is now 9876543210987654321. The record is <the record's path>.
   every other field equal, the one that changed printed old then new (proposal)
```

Pros:

- the operator's act is deliberate and named,
  so a swap and a restore are told apart by what was typed;
- the shortest of the named forms,
  and the printed line is the same ledger as D's and E's.

Cons:

- a bare flag is a second kind of option in a grammar whose every option takes a value,
  so the grammar, the usage and the parser each need an exception for it;
- a person who restores often types the same flag from memory,
  and nothing distinguishes a replace of this record from a replace of whatever record stands.

What it makes of the stale-record window:
the same one command,
with the same weakness as its typing: nothing proves the operator read this record.

Option C,
the same path replaces without a flag:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
replaced: /home/user/Monochromatic keeps its path, dataset, container and pool; its GUID was
1234567890123456789 and is now 9876543210987654321. The record is <the record's path>.
   the same line as D's second block, without the flag (proposal)
```

Pros:

- a restore needs one command, which is the one the step 10 design already names;
- the line still prints the old GUID beside the new one.

Cons:

- the same command now does two different things depending on what the kernel shows,
  and `checkout-record-differs` stops existing for every field but the path;
- the act that admits a different dataset is a re-run of the ordinary command,
  so a person who re-records after a restore and a person who re-records after a swap type the same
  thing and read the same line;
- an operator who re-runs the ordinary command for another reason admits a different dataset without
  ever seeing that it differs.

What it makes of the stale-record window:
the repair is the ordinary command,
which the boot's refusal does not name,
so the window lasts until someone happens to re-run it.

Option B,
remove the file and record again:

```text
$ sudo rm -- /var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
recorded: /home/user/Monochromatic is <its dataset> (GUID <number>), in pool <pool>,
beside container <container>; the record is <the record's path>
   no comparison and no old value, because there was no record to compare with (proposal)
```

Pros:

- nothing to build:
  the mode keeps the rule that an absent record is written and a differing one refuses.

Cons:

- the operator's step is a file deletion,
  which no refusal's "ways forward" can name without teaching a root file to be removed by hand;
- nothing compares the old record with the new dataset,
  and the printed line holds no old GUID,
  so the journal cannot tell a restore from a re-created dataset;
- a person who deletes the file and then fails a proof has destroyed the record and got nothing.

What it makes of the stale-record window:
the repair is a deletion and a re-record,
and nothing at boot names either,
so the window lasts until an operator remembers the runbook.

Ranking:
E > D > A > C > B.

E over D:
both make the admission a distinct, printed act with one value to copy,
and both leave the journal line.
E delivers the record that the command named:
the typed value is the one written,
so a dataset that moved again refuses rather than being written unseen.
D delivers the record that was read:
a record another root act replaced in the meantime refuses.
Which of the two the design ranks first follows from what the act is for:
the act admits a dataset,
and the mode's own proof of freshness is a read of the kernel,
so the value that should be fixed by the operator's command is the kernel's.
D's protection covers a race between two root acts on one host,
which is rarer than a second restore of the same checkout,
and either race already ends in a correct record when it does not refuse.

D over A:
both name one value to copy and both print the old GUID;
D's token proves the operator read this record,
while A's bare flag cannot be told from a remembered habit,
and A pays for its shortness with a second kind of option in the grammar.

A over C:
both print the old GUID and both compare;
A keeps `checkout-record-differs` meaningful for every field,
while C turns the ordinary command into the admitting one,
so a re-run for another reason admits a dataset nobody looked at.

C over B:
C compares and prints the old value;
B writes the new record with nothing to compare it against,
and makes the operator's step an unlogged deletion of a root-owned file.

The choice is separable from every other one,
with one interaction and one dependency.
It needs choice 8's word.
Whichever wins, `mount-path-descendant`'s `checkout-record-no-longer-matches` prints the record's
GUID and the kernel's,
so every option's repair has the number it needs printed at the boot that failed.
E, D, A and C change the record mode's parse,
its usage line and its refusals;
E and D add `replace-token-differs`,
E and D add `nothing-to-replace`,
and E's stricter two-token form adds `--with` and its own refusal.
A and D change `record-checkout`'s command line in the step 19 setup only for a restore,
which the runbook states as the repair,
while a first record never needs the flag.
A `--replace` whose value leaves every field equal answers 0 with the `already recorded` line and
writes nothing,
because a replace that changes nothing is the ordinary command's no-op.
B changes nothing in the code,
so it would be decided by writing the runbook's deletion line and nothing else.
The step 10 build's reader does not depend on any of them:
it reads whatever record stands.

## Choice 10: the panic hook a root mode installs

The wrapper's hook prints one line naming the source location and never the panic's message,
because the message can hold bytes of a scanned file
(source package/git-policy/cli/src/native/invocation/panic_notice.rs:68 to :86).
A root mode does not scan,
but its panic message can hold a dataset name,
a checkout path,
or a `ZFS` request's fields,
and it lands in root's journal under a unit.

The choice is what a root-mode start installs before its mode runs.

Option A,
the wrapper's existing payload-free hook,
installed inside the root-mode arm:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root mount-path-descendant --dataset <a dataset whose read panics>
monochromatic-git-policy-cli: internal error at <file>:<line>:<col>; its message is not shown because it may
contain repository content. Hint: if you were running a Git command, run it again; your files are not changed
by this notice, so `git status` shows where things stand. If it repeats, report this location.
   the code's own line, which is a pure function of the panic's location (source panic_notice.rs:68 to :86)
```

Pros:

- no new text and no second hook to keep true;
- a dataset name,
  a checkout path or a request's fields never reach root's journal through a panic;
- one call inside the arm,
  which is the arm's own first statement.

Cons:

- the hint speaks of a Git command and of `git status`,
  which reads oddly from a unit,
  and this design accepts that rather than keep a second hook text true.

Option B,
the default hook:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root mount-path-descendant --dataset <a dataset whose read panics>
thread 'main' (430836) panicked at panic_default.rs:4:5:
the mount of zpcachyos/nosnap/vm-builder-output at /home/user/Monochromatic was refused by the kernel
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
   measured with a five-line program whose panic message holds a dataset name and a checkout path
   (replay keep/panic-hook-choice.log)
```

Pros:

- no code at all,
  and a panic's message is where a defect's reason usually is.

Cons:

- the message is printed,
  and it is exactly the text that can hold a dataset name,
  a checkout path or a request's fields,
  into root's journal and any log a unit's output reaches;
- the backtrace note invites a variable a unit does not set.

Ranking:
A > B.

A over B:
both print a location,
and A's line is the same length and shape;
what separates them is what else is printed,
and A never prints a payload that can hold a path,
while B always does.
B's gain is a panic's reason in the journal,
which a root mode's refusals already give for every failure it anticipates.

The choice is separable in what it decides and not in what it edits.
It changes one call inside the root-mode arm,
and nothing about the dispatch's placement,
the layouts or the modes;
but the arm's two `install_payload_free_panic_hook();` lines are pinned lines of `MAIN_CODE`,
so option B (the default hook) deletes both and re-pins the entry,
which the fifth revision's sentence denied.
Installing the hook before `main`'s own work would be choice 4's placement instead,
and this choice is about what the arm installs.

## Choice 11: whether the release honours the probe's deadline variable

`CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS` is the one test-only variable that can turn a failure
into a pass.
Any positive number of milliseconds in it becomes the probe's bound
(source package/git-policy/cli/src/native/start_probe/probe_deadline.rs:119 to :127,
the only non-test caller being `start_probe/probe_mode.rs:117`),
and the constant's own doc says why it exists:
so a held step records as passed rather than timed out (:65 to :69).
A run that missed the shipped 350 ms bound and would have failed closed therefore passes under a
larger value,
and a unit's `Environment=` line can set it
(design,
from those two lines and the parse they name;
this round ran neither arm).

The variable is not fail-safe,
and the code says so in another file:
`probe_test_switch.rs`'s header calls a variable that turns a refusal into a pass "the force flag
the design forbids" (`:5` to `:14`).
This choice is what the release does about that one variable,
and it is asked apart from choice 7 because its answer changes what a guest arm builds,
not what the release contains in general.

Option A,
compile it out of the release:

```text
$ CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=60000 <installed file> monochromatic-git-policy-cli zfs-root start-probe
   the probe uses the shipped 350 ms bound; nothing read the variable, so a held step still records
   as timed out and the mode still fails closed (design)
```

and the raised-deadline control,
whose whole point is a step that finishes after the shipped bound,
builds the probe with the variable compiled in and runs that build:

```text
$ cargo build --release --locked --features start-probe-raised-deadline
$ CLI_GIT_TEST_ONLY_START_PROBE=hold-move CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=60000 <that build> monochromatic-git-policy-cli zfs-root start-probe
   the record says the held step passed, which is what the control exists to show (design)
```

Pros:

- the release cannot report a passed probe it did not pass,
  whatever a unit's `Environment=` says;
- the release's probe path loses a branch,
  which is one fewer shape to read at the root boundary.

Cons:

- a second build shape for one control,
  the same cost choice 7's option B states,
  and the guest arm that raises the bound tests that build rather than the released bytes;
- the arm's evidence changes from "the release fails closed after a held step" to "a build with the
  variable compiled in records a pass",
  which is a weaker statement about the release.

Option B,
keep it and make a raised bound visible in the record:

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=hold-move CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=60000 <installed file> monochromatic-git-policy-cli zfs-root start-probe
   the record's verdict reads passed and carries `deadline: 60000 ms, raised by a test variable`;
   the route's gate refuses that record, so a boot cannot admit clones on it (proposal)
```

Pros:

- the control keeps running the released bytes;
- a raised bound can never be mistaken for a shipped pass,
  because the record says which bound it used and every reader can refuse it.

Cons:

- the release still honours a variable that can pass a probe,
  and everything now rests on every reader checking the new field;
- it adds a field to a record the route's gate reads,
  which is a format change in step 19's build;
- the control's own record is refused by the gate,
  so the arm asserts on the record's text rather than on a boot that admits.

Option C,
keep it as it is:

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=hold-move CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=60000 <installed file> monochromatic-git-policy-cli zfs-root start-probe
   the record reads passed, and nothing in it says the bound was raised
   the code's own behaviour (source probe_deadline.rs:119 to :127), not run
```

Pros:

- nothing to build, and the control keeps the released bytes.

Cons:

- a copy-pasted unit line or a stale `Environment=` silently turns a missed probe into a pass,
  which is the one thing the probe's refusal exists to prevent.

Ranking:
A > B > C.

A over B:
both stop a raised bound from meaning a shipped pass,
and both cost the control something.
A removes the possibility from the release,
so no reader and no unit line has to be trusted;
B keeps it and moves the whole guarantee into a field every reader must check,
which is a new way for the guarantee to be lost.
A's cost is a build shape for one control;
B's is a record field in a format another step owns, plus the same discipline in every reader.

B over C:
B at least makes a raised bound visible and refusable;
C leaves a passed record that nobody can tell from a real one.

The choice is separable from choices 1 to 10,
with one cost that reaches choice 7 and the guest arms:
whichever option wins, the raised-deadline arm names the build it runs,
and that sentence changes in `e2e/start-probe-guest.ts` and in
`src/native/start_probe/probe_deadline.rs`.
Nothing about the dispatch,
the layouts, the record or the two new modes changes.

## What else the modes keep, and what they lose

Kept,
because the modes are the same code and the same grammar:

- the mode words,
  the adoption's three options,
  the holder scan's two,
  the usage text and the refusals;
- exit status 0 for a mode that did its work and 2 for a refusal;
- the compatibility gate before the first raw request,
  and the probe's skipped-record arm;
- the installed path spellings of the ZED hook and the configuration,
  and the modes' program name in messages and `--version` (choice 5);
- the two log waits at exit.

Changed:

- `--version` and `--help` are reached through the one name or the words,
  and the line is choice 5's,
  which `doctor`'s planned check (step 18) and the release smoke read,
  and `command -v mount-git-worktree-clone-datasets` is already empty on this host
  (run keep/host-facts-sixth.log);
- `native:install`'s handler check becomes a words check,
  because no family name reaches the account's `PATH`;
- every message that tells a person to run `mount-git-worktree-clone-datasets ...`
  names the words and the installed path instead,
  while the program's name in the opening sentence of a refusal stays (choice 5,
  option A);
- the `--help` paragraph that claims both install paths exist says who creates them and when;
- a root mode no longer passes the lock sweep,
  so a mode started from inside a repository writes nothing into that repository
  (measured in "Every start-time action of today's wrapper");
- the modes' usage's closing sentences name the words and the one root-owned file;
- two modes join the list,
  and the checkout record is the one thing they add to what a boot must have.

### The two programs' sizes, at one commit

Built from the third round's merge commit,
`package/git-policy/cli/target/release/monochromatic-git-policy-cli` is 29,168,472 bytes
and `.../mount-git-worktree-clone-datasets` is 3,418,256,
which is 8.53 times
(the log also holds the build's own `ls`;
replay keep/handler-usage.log;
this round built neither program,
and its merge changed no code).

### The cold start, measured four ways

What the numbers are,
said exactly,
because the first revision's labels were wrong and its two arms did different work:

- every measured start but the last is a **page-cache-cold** start of the program's own file.
  On this host `/home` and the build directory are ZFS,
  so a start after `POSIX_FADV_DONTNEED` on that file alone is **ARC-warm** for the file's own blocks;
- only the first start after such an eviction is cold.
  A per-event start on a running system finds the pages in memory,
  which is the warm arm in "The cold start,
  measured four ways";
- the units are **major page faults**,
  not pages read;
  a major fault brings in at least one page and often more,
  and the guest's in-block counts are the byte measure.

The commands each arm ran,
and what each did:

- arm `w`:
  the wrapper,
  `./wrapper monochromatic-git-policy-cli service run status`,
  which is the wrapper's main-level mode start,
  exiting 3 after the service's own setup and log wait;
- arm `h`:
  the handler,
  `./handler --version`,
  which is the handler's whole main,
  grammar and version answer,
  exiting 0;
- the warm arm `w` to `h` is the same pair with nothing evicted between runs.

No arm is the proposed start.
A root-mode start runs `w`'s dispatch and then `h`'s mode,
and this round did not build that start,
so the pairs and ratios in "The cold start, measured four ways" are what two programs cost,
never a bound or an estimate of the third.
The arithmetic that would make them one is what the second revision stated,
and this revision withdrew it (design).

Host,
page-cache-cold and ARC-warm,
41 rounds,
arms interleaved and alternating
(the second revision's host arms,
`keep/host-wrapper-service-against-handler.txt` and
`keep/host-spread-handler-against-itself.txt`,
analysed in `keep/cold-start-analysis-rerun.txt`,
replay).

- `w`:
  median 5.69 ms,
  421 major faults;
- `h`:
  median 2.05 ms,
  78 major faults;
- `w - h` per round:
  median **+3.61 ms**,
  range 2.57 to 8.60 ms;
- `w / h` per round:
  median **2.81**,
  range 1.84 to 5.06;
- the control,
  `h` against itself:
  median -0.01 ms,
  ratio 1.00 (0.79 to 1.72),
  so the method resolves a difference of this size and not much smaller;
- a second pair as context:
  the same wrapper started with the words as they behave today
  (`./wrapper monochromatic-git-policy-cli zfs-root boot-reconcile`,
  exit 2 after the management arm's refusal)
  gave median 5.94 ms and 439 major faults,
  ratio 2.97.

Guest,
Ubuntu 24.04 (kernel 6.8.0-147,
glibc 2.39,
4 CPUs),
the whole page cache dropped before every run,
21 rounds,
created and destroyed through the repository's own guest driver
(`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts create` and `mvm destroy`)
and measured by `keep/coldstart_drop.c`,
which syncs and writes 3 to `/proc/sys/vm/drop_caches` before every run
(the second revision's guest arms,
`keep/guest-disk-cold-wrapper-service-against-handler.txt` and
`keep/guest-disk-cold-spread-handler-against-itself.txt`,
replay).

- `w`:
  median 12.82 ms,
  37 major faults,
  11,384 in-block reads;
- `h`:
  median 7.45 ms,
  15 major faults,
  3,760 in-block reads;
- `w - h` per round:
  median **+6.06 ms**,
  range 0.50 to 27.60 ms;
- `w / h` per round:
  median **1.87**,
  range 1.05 to 4.00;
- the control,
  `h` against itself:
  median +0.46 ms,
  ratio 1.06 (0.46 to 2.96),
  whose range overlaps the arm's,
  so the guest's method resolves the direction and not the size;
- the difference in disk reads is 3,760 blocks of 512 bytes against 11,384,
  which is 1.9 MB against 5.8 MB,
  so a disk-cold start of the wrapper reads about 3.9 MB more than the handler's.

The same guest measured with only each file evicted (no `drop_caches`),
which is page-cache-cold and disk-cold for that file but warm for the loader and libc,
gave `w` 11.91 ms and `h` 4.44 ms,
ratio 2.91
(replay keep/guest-page-cache-warm-wrapper-service-against-handler.txt).

The warm per-event start,
which the second revision left unmeasured,
was measured in the third round on this host with the same harness and nothing evicted,
120 rounds,
arms interleaved,
and it is re-analysed this round from the same raw files
(the third round's
`keep/host-warm-wrapper-service-against-handler.txt` and
`keep/host-warm-spread-handler-against-itself.txt`,
re-analysed in keep/warm-reanalysis-fourth.log,
replay;
the log states its quantile convention,
so every number in the warm-start list is recomputable from the two files).

- `w`:
  median 1.16 ms,
  no major faults;
- `h`:
  median 0.64 ms,
  no major faults;
- `w - h` per round:
  median **+0.48 ms**,
  5th to 95th percentile 0.19 to 0.80 ms,
  lower quartile 0.38 and upper 0.64,
  maximum 1.36 without round 1;
- `w / h` per round:
  median **1.72**,
  5th to 95th percentile 1.23 to 2.42;
- the control,
  `h` against itself:
  median +0.01 ms,
  5th to 95th percentile -0.24 to 0.30 ms,
  and 66 of its 119 rounds after the first positive.
- round 1 is not a warm round:
  it took 416 and 72 major faults,
  and it is the pair's 3.67 ms maximum.

What that means for the decision:

- the method does resolve a gap of about half a millisecond here:
  the pair's lower quartile, 0.38 ms, stands above the control's 95th percentile, 0.30 ms,
  and on the same basis the pair is 120 positive rounds of its 120 against the control's 67 of its
  120,
  or,
  dropping each arm's first round, 119 of 119 against 66 of 119
  (replay keep/warm-reanalysis-fourth.log, lines 11, 13, 17 and 19);
- so a warm per-event start,
  which is what each history event pays,
  costs the wrapper about 0.48 ms more than the handler at the median,
  with the pair's 5th to 95th percentile stated in the same list;
  that is a resolved and small difference,
  and it is not an upper bound;
- a root start after boot,
  or the first after an eviction,
  is where the difference lives:
  the wrapper's cold start is 2.81 times the handler's at the median on this host,
  with the arm's own range of 1.84 to 5.06 beside it,
  and 1.87 times it in the guest,
  whose ratio range the guest's own control overlaps;
- the start-path cost is the decision's accepted price;
  if it ever bites,
  the fix is a smaller early path,
  not a second program.

One sample of the earlier round is kept apart rather than folded into a median:
the handler's first run took 12.26 ms and read 1,744 blocks,
while its other 24 runs read none
(replay keep/cold-start-analysis-rerun.txt).

This round's host numbers differ from the first revision round's
(wrapper 9.78 ms,
handler 3.57 ms,
ratio 2.68,
pairs 1.46 ms and 7.65 ms for its controls)
because the host's load differed between the rounds,
which is why only the interleaved pair and the ratio are reported here,
and why the shape of the difference,
not its milliseconds,
is the finding.

### Lost

- the smaller program:
  8.53 times the bytes,
  and one more file in every page cache;
- a release that can pin or roll back the root side separately from the wrapper,
  which is the accepted cost of the placement the project already chose.

## What changes, and in which step

Every path under "What changes, and in which step" is re-read with `rg`,
and each line number is given at `5f904d9fbd`,
this round's base,
unless it says `f3d81e0910`,
which is the commit the third revision read and the line has not moved since.
`rg --hidden -o "mount-git-worktree-clone-datasets"` finds 157 occurrences in 56 files at this base,
of which the crate's `doc/` records and one `perf/` measurement are history and stay as they are,
and this proposal contributes 24 of them;
without it the count is 133 in 55 files
(run keep/occurrences-sixth.txt).
A run without `--hidden` finds 151 in 55,
because it skips `.github/workflows/cargo-publish.yml`.

### One commit: the release ships one program

These cannot be separated,
because the guard reads the manifest and the release lists in one test,
and `toolchain:lock:check` compares the plan and the lock offline.
`rg --hidden -c "mount-git-worktree-clone-datasets"` is not the only key:
the sites that count executables without naming the second one move in the same commit,
and the fifth read found them missing from this list.
They are
`SPEC.md:251`, `:262` and `:290` (under "## The two executables and shipped policies"),
`README.md:14`, `:1921`, `:1928`, `:7333`, `:8202` and `:8448`,
`bin/native-install-steps.ts:96`,
`bin/native-install.ts:443`,
the doc of `source_guard/release_bins_tests.rs`,
and plan lines 234 to 247, 3442 to 3448 and 3521.

- `package/git-policy/cli/Cargo.toml`:
  remove the second `[[bin]]` (`:46` to `:48`) and its comment (`:39` to `:45`),
  and correct the comment that counts executables (`:54`);
- `package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs`:
  deleted,
  with its folder;
- `package/git-policy/cli/src/native/lib.rs:462` to `:475` (the paragraph) and `:476` to `:477`
  (the module):
  the module's doc paragraph,
  which says "the crate's second executable" (`:468` at this base;
  `:415` to `:428`, "second executable" at `:421`, at `f3d81e0910`);
- `src/native/source_guard/release_bins_tests.rs`:
  its doc (`:1` to `:22`) and `RELEASED_BINS` (`:31`);
- `bin/native-release-bins.unit.test.ts`:
  the doc comment;
  its planted-second-bin control still requires its four disagreements and needs no change of logic;
- `package/crate-release/cli/releasable-crates.json:30`:
  `checkBins` holds one name;
- `.github/workflows/cargo-publish.yml`:
  the job comment (`:2430` to `:2443`),
  the `--bin` flags (`:2666`),
  the smoke step (`:2671` to `:2680`),
  and the second `bin=` copy line (`:2695` to `:2698`);
- `toolchain.plan.jsonc`:
  the row's `bins` holds one name (`:271` to `:274`),
  and `toolchain.lock.json` takes it through the tool that writes the lock offline,
  which is the command `toolchain:lock:check` runs without `--frozen-lockfile`,
  never by hand;
- `package/git-policy/cli/mise.toml`:
  the descriptions that count two executables (`:88`,
  `:102`,
  `:158`)
  and the e2e task named for the second program (`:268` to `:270`);
- `package/git-policy/cli/README.md`:
  `:11`,
  `:1925`,
  `:7325`,
  `:7332` and `:8205`
  (`:11`,
  `:1912`,
  `:7188`,
  `:7195` and `:8038` at `f3d81e0910`);
- `package/git-policy/cli/e2e/README.md`:
  `:833`,
  `:863`,
  `:1089`,
  `:1090` and `:1093`,
  where the second program's guest task is described;
- `package/zfs/request-to-openzfs-kernel-device/e2e/README.md:539`,
  which says the fixture waits for step 5's binary;
- `package/git-policy/cli/SPEC.md`:
  "## The two executables and shipped policies" (`:249`) becomes
  "## The single executable and shipped policies" and says how the root modes are reached;
  `:255` and `:266` name the family through the words or the one file;
  `:8023`,
  `:22256` and `:22436` name the modes' commands and become the words
  (`:7652`,
  `:21812` and `:21992` at `f3d81e0910`);
  the sentence "and the executable does not read the name it was started under" is at `:19705`
  (`:19588` at `45ac28ae47`,
  `:19261` at `f3d81e0910`) and is rewritten as "The guard change,
  exactly" gives it;
- `package/git-policy/cli/doc/step-5-second-executable-2026-11-20.md`:
  a note under its status that the second program was removed by this decision,
  with the date,
  keeping the record's own text;
- `package/git-policy/cli/doc/holder-scan-across-mount-namespaces-2026-11-22.md`
  and `package/git-policy/cli/doc/slot-identity-store-write-cost.md`:
  historical measurements that name the program as it was,
  so each keeps its text and gains a note giving the current spelling and the date;
- `package/git-policy/cli/perf/sweep-cost-2026-11-23-statmount-interleaved.json`:
  unchanged,
  for the same reason;
- `doc/decision/` and `doc/handover/`:
  untouched by this design.

### The same commit: the entry, the names module, the guards, and every test that started the second program

Deleting the `[[bin]]` breaks these at compile time,
so they land with the release lists:

- `src/native/main.rs`:
  the whole file is the pin printed in "The hook guard on `main.rs`",
  so the commit is that text:
  the `#[cfg(target_os = "linux")]` block of "The rule,
  in the entry",
  its `#[allow(clippy::disallowed_methods)]` on the one collection,
  the comment at `:53` corrected to name the one name and the words,
  the hook installed inside the arm,
  today's three `arguments.as_slice()` uses at `:56`,
  `:57` and `:76` written as `handed`,
  which is why the `handed` line exists,
  and today's `start_directory` lines at `:73` to `:76` kept as they are.
  The first thing this bullet's commit does is re-read `main.rs` at the tip it lands on:
  the fifth revision's printed file was stale against `main` within eleven minutes of its own
  merge (this read's M4),
  and the pin turns every later move of the file into a red guard in the commit that makes it;
- `src/native/worktree_clone_mount_handler/root_mode_names.rs` (new):
  the word that cannot be honoured,
  the two constants and `root_mode_start`,
  with its controls,
  and the folder's top doc listing it first;
- `src/native/worktree_clone_mount_handler/root_mode_names_tests.rs` (new):
  the cases in "What each decides" as unit tests,
  including the two the shell cannot spell;
- `src/native/worktree_clone_mount_handler/mod.rs`:
  `pub mod root_mode_names;` first in the list,
  `refuse_root_mode_misuse` added to the `pub use handler_mode_dispatch::{…}` line (`:183`,
  which re-exports `EXIT_DONE`, `EXIT_REFUSED` and `run` today),
  and the top doc's "the second executable" paragraph (`:2` to `:12`) rewritten;
- `src/native/worktree_clone_mount_handler/handler_mode_arguments.rs`:
  `MODE_WORDS` (`:41`) gains `record-checkout` and `mount-path-descendant`,
  the `HandlerMode` doc's "no argument names a folder to mount or a dataset to change"
  (`:112` to `:115`) is rewritten,
  because the two new modes do both,
  `parse` (`:319`) gains their arms and options,
  with `REPLACE_OPTION` beside the five constants at `:50` to `:62` and
  choice 9's value-taking `--replace` parsed like every other pair,
  the four refusal messages that name the program (`:220`,
  `:223`,
  `:233`,
  `:245`),
  `usage()` (`:265`,
  whose first line at `:267` names it,
  and whose closing sentences name both
  installed paths and become the one file and the words),
  and its tests;
- `src/native/worktree_clone_mount_handler/handler_install_paths.rs`:
  `SERVICE_INSTALL_PATH` (`:32`) becomes `ROOT_MODE_INSTALL_PATH` and holds the one root-owned file's path,
  `PROGRAM_NAME` (`:28`) goes with the family name choice 5's C drops,
  and the checkout record's path joins them as one constant;
- `src/native/worktree_clone_mount_handler/handler_install_paths_tests.rs`
  and `handler_mode_arguments_tests.rs`,
  where the same strings are asserted;
- `src/native/worktree_clone_mount_handler/record_checkout_mode.rs` (new):
  the record mode's proof,
  refusal list and line,
  with its controls,
  in the shape `adopt_worktree_parent_mode.rs` has,
  and choice 9's refusals:
  `checkout-record-differs`,
  and, under the recommended option E,
  `replace-token-differs` and `nothing-to-replace`;
- `src/native/worktree_clone_mount_handler/mount_path_descendant_mode.rs` (new):
  the mount mode's seven steps of "What it does",
  its read of the record and of the dataset's fifteen properties,
  its `open_root`/`mount_id_of`/`observed_path` proof that the checkout is mounted,
  its `open_chain_beneath`/`open_directory_at_a_mount` walk and `mkdirat` of a missing mountpoint,
  its `fsopen`/`fsconfig`/`fsmount`/`move_mount` mount onto the opened folder,
  its idempotence,
  its refusals (`dataset-outside-the-checkout`,
  `dataset-in-another-pool`,
  `dataset-without-a-mountpoint`,
  `no-recorded-checkout`,
  `checkout-record-no-longer-matches`,
  `checkout-not-mounted`,
  `dataset-already-mounted-elsewhere`,
  `dataset-not-mountable`,
  `encryption-key-not-loaded`,
  `mountpoint-not-empty`,
  `mountpoint-unusable`),
  and its line,
  with its controls;
- `src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs`:
  `refuse_root_mode_misuse`,
  which says the misuse,
  the usage and answers `EXIT_REFUSED`,
  and the two new modes' dispatch arms;
- `src/native/source_guard/hook_self_invocation_tests.rs`:
  the test renamed,
  its body replaced by the pin and its two assertions,
  as "The hook guard on `main.rs`" gives it,
  `MAIN_CODE` beside it,
  and `code_lines` moved out to `mod.rs`
  (`names_the_argument_vector` does not exist in the tree today,
  and nothing here needs it);
- `src/native/source_guard/release_bins_tests.rs`
  and `bin/native-release-bins.unit.test.ts`:
  the `path` reader and its assertion,
  as "The `[[bin]]` `path` the release builds" gives it,
  so the manifest's `path` is a guard and not only its `name`;
- `src/native/source_guard/mod.rs`:
  the `Reading` state, `pub(super) fn code_lines(source, what)`,
  `pub(super) fn squeezed`,
  `pub(super) fn assert_whole_line_comments_only(what, source)`,
  `pub(super) fn release_sources_in`,
  and the declarations of `root_mode_name_tests` and `lint_attribute_tests`;
- `src/native/source_guard/root_mode_name_tests.rs` (new):
  `NAMES_CODE`, the pin of the module,
  its test,
  and the three wiring assertions
  (the declaration in `mod.rs`, `COMMAND_WORD.current`, `ZEDLET_INSTALL_PATH`'s file name);
- `src/native/source_guard/lint_attribute_tests.rs` (new):
  the twenty pinned lint attributes and the test that holds the release tree to them;
- `src/native/source_guard/no_self_invocation_rules_tests.rs`:
  the three new `argv0-dispatch` needles,
  the `argv-vector-read` rule,
  its one `ALLOWED` row,
  and both rules' `squeezed_line` variable,
  which no longer shadows its own parameter;
- `src/native/source_guard/no_self_invocation_tests.rs`:
  its `code_lines` becomes a wrapper over the shared one,
  a seventh `samples` pair (`:191`),
  which `:208`'s length assertion carries,
  and the test that fires each added needle on its own witness and on no release read;
- `package/git-policy/cli/clippy.toml`:
  the two `std::env::args` rows;
- `src/native/binary/binary_supervisor_tests.rs:755`,
  `src/native/push/child_watch_tests.rs:1000` and
  `src/native/landing/service_durable_tests.rs:241`:
  one `#[allow(clippy::disallowed_methods, reason = "…")]` each,
  matching the crate's existing allows;
- `src/native/binary/binary_external_hook_tests.rs`:
  the doc comment of `a_start_name_that_used_to_be_an_entry_behaves_as_git` (`:174`)
  says the one exception,
  and its body stays as it is;
- `src/native/testing/root_mode_alias.rs` (new):
  one writer of the installed name in a fixture folder,
  beside `testing/fixture_executable.rs`,
  so each test that starts a mode names one link and not one copy of the rule;
- the six test files whose `CARGO_BIN_EXE_mount-git-worktree-clone-datasets` lines
  "The release's lists,
  and what reads them" lists:
  each starts `CARGO_BIN_EXE_monochromatic-git-policy-cli` through the fixture link,
  its assertions on the output stay as they are,
  and every start that passes a mode word gains the two words:
  `adopt-worktree-parent` at `binary_zfs_adoption_guest_tests.rs:118`,
  `holder-scan --dataset` at `binary_zfs_holder_scan_guest_tests.rs:290` and `:892`,
  and `start-probe` at `binary_zfs_sweep_guest_tests.rs:245`.
  The starts that pass no argument stay as they are:
  the sweep at `binary_zfs_sweep_guest_tests.rs:230` and `:875`,
  the unmount start at `binary_zfs_unmount_guest_tests.rs:238`,
  and the two mode-less starts at `binary_zfs_step9_m_arms_tests.rs:200` and `:561`;
- `src/native/binary/binary_service_mode_tests.rs:6`:
  the comment that counts two executables;
- `src/native/binary/binary_root_mode_name_tests.rs` (new):
  the refusal path's binary control,
  which no source text can be.
  It makes a throwaway repository (`git init` under the round's temporary folder),
  links the built `CARGO_BIN_EXE_monochromatic-git-policy-cli` under the installed name through
  `testing/root_mode_alias.rs`,
  and starts that link from the repository's folder with one bare word
  (`boot-reconcile`).
  It asserts three things:
  status 2,
  the misuse line and the usage on the combined output,
  and that `.git/cli-git-lock-holders/` does not exist.
  Its green arm is the same link answering `--help` and
  `monochromatic-git-policy-cli zfs-root --version`, both status 0;
  its control is a second link named `git`, whose same bare word reaches the wrapper and writes the
  marker, which proves the marker check can see one.
  No arm starts the link with no argument,
  because that is the sweep;
- `src/native/invocation/management_arguments.rs`:
  one line in `MANAGEMENT_USAGE` (`:46`) and one in `MANAGEMENT_HELP` (`:59`)
  for `monochromatic-git-policy-cli zfs-root <mode>`,
  and the tests that assert those texts
  (`invocation/management_arguments_tests.rs`,
  `invocation/entry_command_word_tests.rs:108` and `:113`);
- `src/native/health/doctor_zfs_route/identity_checks.rs:145` and `:164`:
  the two messages that tell an administrator to run the old program name,
  and two release refusal texts that do the same:
  `worktree_clone_mount_handler/sweep_preconditions.rs:195`
  (which names `mount-git-worktree-clone-datasets start-probe`) and
  `worktree_clone_mount/clone_slot_walk_refusals.rs:194`
  (which names `mount-git-worktree-clone-datasets adopt-worktree-parent`);
- `src/native/health/doctor_zfs_route/handler_checks.rs`:
  `PLACEHOLDER_EXECUTABLE_PATH` (`:60`) becomes the settled root-mode path,
  `PLACEHOLDER_ZEDLET_PATH` (`:70`) becomes the installed name
  (which is also today's ZED hook name,
  so the two stand-ins become one),
  and their controls (`route_test_facts.rs:304`,
  `handler_checks_tests.rs`) move with them;
- `e2e/start-probe-guest.ts:42`:
  `HANDLER` is a bare path to the built second program today,
  and it becomes that path plus the two words
  (`monochromatic-git-policy-cli zfs-root`),
  so every `${HANDLER} start-probe` line keeps its shape,
  including the `pkill --signal KILL --full "^${HANDLER} start-probe$"` at `:415` and the
  `PAT='^${HANDLER} start-probe$'` at `:447`;
- `e2e/mount-git-worktree-clone-datasets-guest.ts`:
  its guest build loses the second program
  (the tarball excludes `target` at `:330`,
  and the guest compiles what it runs at `:340`),
  and the guest creates the installed-name link before the adoption arm runs;
- `e2e/start-probe-guest.ts:196`,
  whose `cargo build --release --locked --bins` builds one binary fewer,
  and its raised-deadline arm,
  which under choice 11's option A builds the probe with the deadline variable compiled in and
  names that build,
  and under options B and C keeps the released bytes.

The two new modes' own files are in this commit because the grammar,
the dispatch and the record's writer are one shape,
and the readers land in step 10's build with the record's format.
Until then `record-checkout` writes a record nothing reads and
`mount-path-descendant` refuses `no-recorded-checkout`,
which is a safe half.

### The order the nested dataset's unit changes in

The step 19 record's answer is that the nested dataset's unit is replaced by
`mount-path-descendant` "once the mode exists"
(record doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md),
and the step 19 design says the unit "runs `/usr/bin/zfs mount` now and the wrapper's
`mount-path-descendant` mode once it exists"
(source doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md:2382 to :2386).
The mode's existence is necessary and not sufficient,
and the order is this:

1.  the checkout migration of plan step 19 runs,
    which is the act that makes the checkout a dataset of its own.
    On this host the checkout is a subtree of `zpcachyos/ROOT/cos/home` today,
    with no dataset mounted at `/home/user/Monochromatic`
    (replay keep/host-facts-fourth.log),
    and the step 10 record says so before the migration
    (record doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md,
    "What the owner was told first").
    The same step's setup runs the `canmount=noauto` change on the nested dataset,
    which is the owner's act on an existing dataset in the step 19 record,
    and the property is `on` and the dataset mounted today
    (replay keep/host-facts-fourth.log);
2.  `record-checkout --checkout /home/user/Monochromatic` runs as root.
    It cannot run before the migration:
    it refuses `checkout-not-a-mount-point` for a path that is not a dataset's own mount point,
    which is the precondition step 10's design states;
3.  only then does the nested dataset's unit's `ExecStart=` become one line naming the mode,
    replacing `/usr/bin/zfs mount <dataset>`:
    `ExecStart=<root-owned file> monochromatic-git-policy-cli zfs-root mount-path-descendant --dataset <dataset>`.
    The unit's `[Unit]` block and its ordering after the gate stay as step 19 wrote them,
    and the swap is that one line.

Where this rests:
on the mode's `no-recorded-checkout` refusal,
which is this design's,
and on the record's own precondition,
which is step 10's design's.
The cost of doing it in the other order is a boot whose nested unit fails with
`no-recorded-checkout` and leaves the nested dataset unmounted,
because a unit that refuses has no fallback.

So the step 19 build changes that unit only after the migration and the first record,
and this design's own commit lands the mode that refuses until that record exists.

Two later acts move that unit again,
and the runbook carries both:

- a legitimate restore of the checkout dataset gives it a new GUID,
  which is the sequence the step 10 design names
  (source doc/planning/cli-git-zfs-step-10-open-choices.md:578 to :579).
  Before the reboot that follows a restore,
  root replaces the record (choice 9),
  and that boot then finds a record the kernel agrees with.
  Skipped, that boot's nested unit refuses `checkout-record-no-longer-matches`,
  names the field that differs and prints the record's GUID,
  and the nested dataset stays unmounted until the replacement runs,
  because a unit that refuses has no fallback and a unit that `Requires=` it fails with it
  (the step 19 record's V6).
  What choice 9's options make of that window is stated there:
  options A, D and E leave one command whose flag the boot's own refusal prints,
  option C leaves the ordinary command,
  which that refusal does not name,
  and option B leaves a deletion and a re-record that no refusal names;
- the migration's tested rollback
  (source doc/planning/cli-git-zfs-worktree-implementation-plan.md:2599,
  which is the plan's only word on it)
  puts the checkout back.
  That the nested dataset's unit goes back to `/usr/bin/zfs mount <dataset>` with it is this
  design's requirement,
  not the plan's or step 19's design's:
  the plan offers "the tested rollback" of the migration and nothing about that unit,
  and `rg -n -i rollback` over the step 19 design finds nothing (design).
  The `ExecStart=` line is swapped back first,
  and the record is removed in the same step,
  in that order,
  because a boot that still named the mode with the record gone refuses
  `no-recorded-checkout` and leaves the nested dataset unmounted just as the stale record
  does.
  The rollback also puts the nested dataset's `canmount` back to `on`,
  which is the property the setup changed,
  so the rollback restores the state the setup was run from (design,
  from step 19's setup line and the property this host shows as `on` today,
  run keep/host-facts-sixth.log);
  that is not what makes the swapped-back line able to mount,
  because an explicit `zfs mount` mounts a `noauto` dataset,
  and with `canmount=on` the unit's `ConditionPathIsMountPoint=!` skips it when the provider mounted
  first (source
  doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md:974 to :1000).
  After the rollback no record stands,
  and `record-checkout` must run again before any unit names `mount-path-descendant`.

### The same commit: what `native:install` and the release smoke check instead

`native:install`'s handler check becomes the words' check,
which is the contradiction's resolution:

```ts
// bin/native-install.ts, where the handler's check stands today (:444 to :465)
const modesReported = (await runGateCommandOrThrow({
  program: gitProgram,
  args: ['monochromatic-git-policy-cli', 'zfs-root', '--version',],
  deadlineMilliseconds: gitDeadlineMilliseconds,
  environment: {
    ...process.env,
    ...promptlessGitSettings,
  },
  workingDirectory: repository,
})).stdout;
printLine(modesReported.trimEnd());
if (!modesReported.includes(rootModesVersionLine({ version, }))) {
  throw new NativeGateError(`${gitProgram} monochromatic-git-policy-cli zfs-root --version does not name ${version}, so this build does not answer the root modes the release asked for. What it printed: ${modesReported.trim() || '(nothing)'}.`);
}
```

`rootModesVersionLine` is choice 5's C's line,
`monochromatic-git-policy-cli zfs-root <version>`,
built from the same two constants the usage is;
`handlerProgram` goes,
and `bin/native-install-steps.ts:104` holds the two words and the group word instead of the family
name.
`bin/native-install.unit.test.ts:417`'s test
("refuses when the release archive moved only the wrapper")
expects the words' line instead of the program's,
and `bin/stand-in-install-programs-for-tests.ts` (`:169` to `:186` and `:335`)
teaches its stand-in `git` to answer the words and loses its stand-in handler.

The release smoke gains the two arms,
so a release cannot ship with either dispatch broken
(`.github/workflows/cargo-publish.yml:2671` to `:2680`):

```yaml
    - name: Smoke-run the binaries
      shell: bash
      env:
        TARGET: ${{ matrix.target }}
      run: |
        set -euo pipefail
        wrapper="package/git-policy/cli/target/${TARGET}/release/monochromatic-git-policy-cli"
        help=$("$wrapper" monochromatic-git-policy-cli --help)
        printf '%s\n' "$help" | grep -q '^Usage: git monochromatic-git-policy-cli'
        links=$(mktemp -d "${RUNNER_TEMP}/root-modes.XXXXXX")
        ln -s "$PWD/$wrapper" "$links/history_event-mount_git_worktree_clone_datasets"
        "$links/history_event-mount_git_worktree_clone_datasets" --version | grep -q '^monochromatic-git-policy-cli zfs-root '
        "$wrapper" monochromatic-git-policy-cli zfs-root --version | grep -q '^monochromatic-git-policy-cli zfs-root '
```

The first arm proves the one name selects the modes;
the second proves the words do;
both print the version line `doctor` and `native:install` compare.

### The plan and the design text

One commit,
after the entry and its tests:

- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`:
  step 5 (`:929`) is rewritten as the undo,
  keeps its release list and loses the second executable from it,
  and its "What it builds" gains the one name and the words;
  `:1000`'s "The gate both binaries run" becomes one gate;
  step 10 gains the record's reader and its format;
  step 18's `doctor` (`:2464`) gains the installation check "Steps 18 and 19" states;
  step 19 (`:2583`) names the one file,
  the symlink and the words,
  and its acceptance arm links the name and reads `--version`;
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`:
  "Binding crate and compatibility" loses the separate-binary paragraph and keeps the "Keep inside cli"
  placement with this design's answer,
  "ZED reconciliation and mount safety" keeps its two installed paths and says they are one file and its name,
  and "Decided defaults" replaces the second binary's cost with the numbers measured here;
- `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
  `cli-git-zfs-start-probe-design.md` and
  `cli-git-zfs-private-propagation-after-replacement-mount-design.md`
  keep their commands and gain a sentence that the name at the head of each is the one root-owned file
  (their `ExecStart`,
  `ExecReload` and probe lines name the modes' own name today,
  so each line changes to that file's path plus the words);
- `doc/planning/cli-git-no-self-invocation.md`,
  "The rule":
  the `argv[0]` bullet gains the one exception,
  citing the start-name record;
- `doc/planning/cli-git-resident-service.md:3324`,
  the sentence that rejects a name-based dispatch:
  it stays true for the service and gains "except the ZED hook's installed name";
- `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
  "What each part waits on":
  the two entries that point at this design's modes gain their words and arguments,
  and the nested unit's `ExecStart=` gains the mode's line.

### Steps 18 and 19

They land in their own steps,
and nothing waits on them for the release to hold one program.

Step 18's `doctor` gains,
in one place:

- the installed file exists at the root-mode path,
  is a regular file through `stat` (so the zedlet's symlink passes with its target's values),
  is root's,
  has its owner execute bit,
  and has no group or other write bit;
- the zedlet's name is one ZED's scan accepts (the class prefix and a non-letter after it);
- every ancestor folder from `/` down exists,
  is a directory,
  is root's and is not group- or other-writable,
  and a missing one prints the `install -d -o root -g root -m 0755` command that creates it
  (`/usr/libexec` is missing on this host today:
  `ls -d /usr/libexec` answers `No such file or directory`,
  run keep/host-facts-fourth.log);
- the installed file's `--version` names the same version as the build `doctor` runs from,
  and a mismatch prints both and the lock row's version;
- the checkout's record exists at its one path,
  is root-owned and `0644`,
  names a path,
  dataset and GUID the kernel agrees with,
  and names a container in the same pool that is neither inside the checkout nor holds it,
  because every later step that names a checkout reads it;
  a missing record prints `record-checkout`'s line and names the folder the mode creates;
- its findings say what a broken or missing name costs (a sweep that never runs)
  and print the command that recreates it,
  through the root-owned path and never through the account's `git`;
- the three unit lines it prints,
  the nested dataset's line,
  the adoption's line and the record's line
  carry the words and the checked-in constants.

Step 19 prints the one file and its name,
and its guest arm starts a unit by that path with the words and reads `--version` through it.
Its runbook carries the order "The order the nested dataset's unit changes in" states:
the migration,
then `record-checkout`,
then the nested dataset's one `ExecStart=` line.
It says where the bytes come from and who runs it:

- the bytes are the lock row's `artifacts.<target>.url`,
  verified against that row's `sha256` before the file is installed,
  never a copy of the account's `~/.cache/monochromatic-git-policy-cli-bin/`;
- the step runs as root,
  by hand or by the runbook,
  and what root trusts is the lock file in a reviewed commit of this repository
  plus GitHub's TLS,
  which is the same trust the account's toolchain already places in that row;
  the design does not claim that the account's copy is safe to read as root.

Step 19 also writes the AppArmor rule,
which the owner's answer puts in `/etc/apparmor.d/local/zed`
(record doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md).
What that file must hold,
and why the design cannot leave it implicit:

- this host has no `local/zed` yet
  (replay keep/host-facts-fourth.log,
  and the profile's lines are in the third round's `keep/apparmor-and-folders.txt`,
  replayed),
  and `/etc/apparmor.d/zed` ends with `include if exists <local/zed>`,
  so every rule the hook needs is one this step adds;
- the profile gives `/etc/zfs/zed.d/{,*} r,` and no execute permission,
  so a zedlet cannot be started at all until a rule grants it;
  the rule therefore names the file AppArmor resolves the zedlet's path to,
  which under choice 3's option A is the root-owned file in `/usr/libexec`;
- AppArmor's documented default is to resolve symlinks before matching a rule,
  and this round did not measure that,
  so the ZED-enable step proves the rule with the audit log before ZED is enabled,
  and a rule that names the symlink instead would fail closed and be visible there;
- the rule carries what the sweep needs on top of exec:
  `capability dac_read_search`,
  and the mount rules for `/home/user/Monochromatic` and the clone container,
  because ZED's profile holds `capability sys_admin` and `/dev/zfs rw` already
  (replay keep/apparmor-and-folders.txt,
  which shows both lines);
- the file's name and the rule's target move together with choice 3,
  so a later layout change is one line in two files,
  and `doctor`'s step 18 check reads whether the rule names the file that exists.

## What the probes showed, and what they did not

### Shown

- ZED's exec shape,
  `argv[0]`,
  argument count,
  working directory,
  descriptors and built environment,
  by a replay of the source's own child step
  (replay keep/zed-shape-report.txt);
- a transient user service's and a user scope's `argv[0]`,
  both printing the path as `ExecStart` wrote it
  (replay keep/unit-argv0-fourth.log);
- the three dispatch rules' decisions on thirty-five starts,
  including ZED's own,
  a unit's line,
  a bare mode word,
  `--help` and `--version`,
  the words with no mode word,
  two typos,
  a trailing slash and three dot forms of `argv[0]`,
  a `PATH` lookup,
  five near-miss names,
  a stray copy under another name and one under the zedlet's own name,
  an adoption's options,
  the two new modes' words,
  the record mode under the installed name,
  links named `git`,
  the wrapper and the modes' family,
  a caller-chosen `argv[0]`,
  an empty `argv[0]`,
  and the two vectors a shell cannot spell
  (replay keep/dispatch-cases.log);
- the compiled names module under real starts by path,
  through links,
  with `exec -a`,
  through the ZED child step,
  with an empty `argv[0]`,
  and with a vector a shell cannot spell,
  each decision printed by the start itself
  (replay keep/names-starts-fourth.log,
  the section "option B's rule, compiled" included;
  the module's text is the fifth revision's and this revision did not change it);
- a start whose name is not UTF-8,
  made with `exec -a` because this host cannot hold such a file name
  (replay keep/non-utf8-starts-fourth.log,
  beside the name rule the host sets in keep/host-facts-sixth.log);
- the two pins,
  the shared string-aware stripper,
  the scan's wrapper,
  its two rules and their witness test,
  the lint-attribute pin and the `[[bin]]` `path` assertion,
  cut out of this document by fence and run as Rust against:
  the document's own printed files,
  today's `main.rs` (the positive control that must fail),
  every `main` and names variant the third,
  fourth and fifth reads named plus `nm6` rebuilt,
  the controls that must pass,
  ten scan fixtures,
  the wiring fixtures (a declaration, and the fifth read's `#[path]` redirect),
  two manifest fixtures,
  and the real release tree under both readers
  (run keep/guards-replay-sixth.log);
- every Rust text this document prints,
  compiled and linted under the crate's own `[lints.clippy]` table and its `clippy.toml` with the
  two `std::env::args` rows this design adds:
  the printed `main.rs` and `root_mode_names.rs` against a stub crate that provides the symbols they
  call,
  and the guards against the crate's own classifier;
  the same stub crate with the typed ban's three positive controls fails all three
  (run keep/clippy-printed-texts-sixth.log);
- the fifth revision's printed stripper and this revision's,
  over the 1,472 release files:
  the blind regions,
  the lost code and comment lines,
  the 63 files that hold a `/*`,
  the 9 lines with a `//` after code,
  and the 19 lint-attribute lines
  (run keep/stripper-over-the-release-tree.log);
- the fifth revision's printed guards pass the variants they were written for and this round's
  `nm6` fails them,
  which is the before column of the replay
  (replay the fifth read's `keep/guards-replay-this-read.log`);
- the typed bans in a probe crate carrying the crate's own `clippy.toml` and the two new rows,
  through a `use ... as` alias and a direct `std::env::args()`
  (replay keep/clippy-bans.log);
- today's bytes under the installed name in a throwaway repository,
  with the status of each start and the marker the lock pass leaves
  (replay keep/today-installed-name.log);
- a copy named `holder-scan` answering `unknown option: --dataset` and exit 129
  (replay keep/holder-scan-name.log);
- both programs' cold starts on this host with the command each arm ran,
  page-cache-cold and ARC-warm,
  with the method's own spread measured in the same way
  (`keep/host-*.txt` and `keep/cold-start-analysis-rerun.txt`,
  replay);
- both programs' warm starts on this host,
  nothing evicted,
  with the same spread control,
  re-analysed with the quantiles its conclusion needs
  (replay the third round's `keep/host-warm-*.txt`,
  analysed in keep/warm-reanalysis-fourth.log);
- both programs' cold starts in an Ubuntu 24.04 guest,
  the whole page cache dropped before every run,
  with the same spread control,
  and the guest destroyed through `mvm`
  (replay keep/guest-*.txt);
- the two programs' sizes in the third revision's build,
  and `readelf --version-info`'s glibc floor for each
  (replay keep/handler-usage.log and keep/cold-start-analysis-rerun.txt);
- the two hooks a root-mode start can install,
  the default one with a panic message holding a dataset name and a path,
  and the wrapper's line as its source writes it
  (replay keep/panic-hook-choice.log);
- the host facts every choice of this design rests on:
  the dataset table,
  the nested dataset's fifteen mount-relevant properties,
  the ownership and mode of its mountpoint and of every parent,
  the GUID digit counts,
  the absence of a dataset at the checkout's path,
  and the three paths and one name that do not exist here
  (run keep/host-facts-sixth.log;
  the fifth round's `host-facts-fourth.log` is replayed where a line has not moved);
- the pin's cost,
  counted from the commits that touch `main.rs` in the window the fifth revision measured
  (run keep/pin-cost-sixth.log).

### Not shown

- ZED itself:
  it needs root,
  `/dev/zfs` and a pool,
  so the replay stands in for its child step only.
  The environment and descriptor values come from the replay's own `execle`,
  not from a running ZED;
- a system unit start:
  the fourth round measured a transient user service and a user scope under the user manager,
  so a system manager's `argv[0]` is asserted from the shared `ExecStart` parse and not measured;
- any mode touching ZFS:
  this round created no guest and ran no arm that mounts,
  unmounts or adopts,
  so the modes' own work is unchanged code and is not exercised here;
  the two new modes do not exist,
  so their lines and refusals are this design's words (proposal);
- the checkout record's readers and format:
  they land in step 10's build,
  and nothing here has read or written one.
  Its fields are this design's list,
  and its file is not on this host
  (run keep/host-facts-sixth.log);
- the proposed root-mode start's cold or warm cost:
  no arm runs it,
  and this design claims no figure for it;
- a disk-cold start on the host:
  the host cannot drop its page cache without root,
  which is why the guest arm exists;
- a comparison of ARC-cold starts:
  the guest has no ZFS,
  so its numbers are a disk-cold floor on ext4,
  not a ZFS ARC measurement;
- whether AppArmor resolves the zedlet's symlink before matching the `local/zed` rule:
  the ZED-enable step proves it with the audit log;
- whether `doctor`'s future installation check catches a link that a package manager renames instead of deleting;
- the release smoke arm itself:
  it is written here and not run,
  because this round changes no workflow;
- the root-mode refusal's and the two new modes' exact texts:
  `refuse_root_mode_misuse`,
  `record_checkout_mode` and
  `mount_path_descendant_mode` do not exist yet,
  so their wording is this design's (proposal),
  not a build's;
- `record-checkout --replace <the kernel's guid>` and the other replacement shapes:
  no build answers the flag,
  and choice 9's five blocks are this design's wording;
- the guards as parts of the crate:
  they are cut out of this document by fence and compiled in a probe crate,
  so the compile and the Clippy run prove the printed text compiles and lints,
  not that the crate's own test binary runs it;
  the crate's own source scan is not run with the printed rules,
  so the release tree's two findings are the probe's count over the crate's own classifier;
- the wiring assertions over the real files:
  `root_mode_names.rs` does not exist at this base,
  so the declaration assertion is shown failing on the real `mod.rs` and passing on a fixture,
  and the `[[bin]]` assertion is shown on the real manifest and on two fixtures;
- what a re-pin feels like:
  the pin's cost is counted in commits, not timed (design);
- the refusal-path binary control:
  it is written in the change list and not run,
  because the dispatch and the refusals do not exist yet;
- choice 11's two arms:
  the raised-deadline control and the compiled-out release are this design's wording,
  and the variable's own behaviour is read from its source, not run as root;
- the nested dataset's mount through `fsopen`:
  the properties and upstream's option derivation are measured and read,
  and the mode that mounts does not exist,
  so no arm mounts it;
- a system manager's start of a unit:
  the fourth round measured the user manager's service and a scope only,
  so the two managers' shared `ExecStart` parse is read from systemd's documentation,
  not measured (design).

## The choices this proposal puts to the owner

Each is ranked in its own section,
with a reason for every adjacent pair by what each option delivers,
and each section says what it is separable from.
Item 1 is where the answer's two clauses meet;
items 2 to 7 are the second revision's other choices;
item 8 is the word of the mode the step 10 answer adds;
items 9 to 11 are the last three revisions' new questions,
posed because a legitimate restore needs a replacement path (the third read's M1),
because the panic hook was a list item inside choice 4,
and because one test-only variable can turn a missed deadline into a pass (the fourth read's M7);
item 9 gained a fifth option from the fifth read.

1.  What selects a root mode:
    the words on any file,
    and under the installed name ZED's no argument,
    `--help` and `--version`,
    with every other word there refused (choice 1,
    A);
    the same without `--help` and `--version` under the name (B);
    or the first revision's rule,
    where the name wins and drops the words (C).
    The design recommends A.
2.  The group word the units and people spell:
    `zfs-root` (choice 2,
    A),
    `clone-mounts` (D),
    `mount` (B),
    or `root` (C).
3.  What root's install holds:
    one file under the ZED hook's name and nothing under the wrapper's (choice 3,
    A),
    one file named for the modes (B),
    one file named for the wrapper (C),
    or a copy or hard link per name (D).
    Under A the name selects ZED's start and the words select every mode;
    under B and C only the words select,
    and a stray argument at that path runs Git as root.
4.  Where the dispatch sits:
    the first statements of `main` (choice 4,
    A),
    after the panic hook and the environment (B),
    at the top of `run_process` (C),
    or inside `plan_invocation` (D).
    B and C rank on where the code lives,
    not on start-time work:
    they cost the same.
    What the arm installs is item 10.
5.  What `--version` and the refusals say:
    the words and the version (choice 5,
    C),
    the modes' own name (A),
    or the wrapper's name and a modes line (B).
    The design recommends C,
    by what each line delivers rather than by which existing comparisons keep their string.
6.  Whether the account's `PATH` keeps a short name:
    the installed path and the words,
    printed by `doctor` (choice 6,
    A),
    or a second toolchain link (B).
7.  Whether the release keeps the modes' test-only variables:
    keep them (choice 7,
    A),
    or compile them out (B).
8.  The words of the mode that records a checkout:
    `record-checkout` (choice 8,
    A),
    `record-route` (B),
    or `checkout-record` (C).
9.  What a legitimate restore does to the record (choice 9):
    `--replace <the kernel's guid>`,
    so the token names the dataset being admitted (E);
    `--replace <the record's guid>`,
    so the token names the record being replaced (D);
    `--replace` as a bare flag (A);
    the same path replaced by the ordinary command, printing the old GUID (C);
    or the record file deleted by hand and recorded again (B).
    E's stricter two-token form,
    `--replace <the record's guid> --with <the kernel's guid>`,
    is the same option with both sides named.
10. The panic hook a root-mode start installs (choice 10):
    the wrapper's payload-free hook (A),
    or the default hook, which prints the panic's message (B).
11. Whether the release honours the probe's deadline variable (choice 11):
    compile it out of the release (A),
    keep it and record a raised bound so every reader can refuse that record (B),
    or keep it as it is (C).

This proposal's own ranking is A in every choice but two:
C in choice 5,
by what the version line delivers rather than by which comparisons keep their string,
and E in choice 9,
by which side of the swap the operator's token fixes.
The closest pairs are choice 1's A over B,
because B is the owner's own literal text and A adds two accepted spellings,
and choice 9's E over D,
where each token protects a different race and both end in a correct record when neither refuses.

## Appendix: where the probe transcripts are

This round's folder,
`/home/user/temp/agent/zfs-root-modes-sixth-revision-2026-11-24/keep/`:

- `state-so-far.md`:
  what the round did,
  in order,
  and what it could not do;
- `guards-replay-sixth.log`:
  the printed files,
  the two pins,
  the shared stripper,
  the scan's wrapper and its two rules,
  the witness test,
  the lint-attribute pin and the `[[bin]]` `path` assertion,
  cut out of this document by fence by `sources/cut_fences.py` and run as Rust by
  `sources/assemble_probe.py` and `sources/guard_replay.py`,
  over the document's own printed files,
  today's `main.rs` and the real `mod.rs`,
  every variant in `sources/variants/`,
  the two wiring fixtures and the two manifest fixtures in `sources/fixtures/`,
  the ten scan fixtures,
  and the real release tree under both readers.
  Each verdict is the guard's own message,
  and the first ten lines are the md5 of every text the log replays;
- `stripper-over-the-release-tree.log`:
  the fifth revision's printed stripper and this revision's over the 1,472 release files:
  the three files the old one ends inside,
  the 794 non-blank lines it loses (424 code, 370 comment-only),
  the 63 files with a `/*`,
  the 9 lines with a `//` after code,
  and the 19 lint-attribute lines under the new reader;
- `clippy-printed-texts-sixth.log`:
  every printed Rust text compiled and linted under the crate's `[lints.clippy]` table and its
  `clippy.toml` with the design's two rows,
  the typed ban's three positive controls failing in the same crate,
  and no diagnostic naming a printed file;
- `host-facts-sixth.log`:
  the dataset table,
  the nested dataset's mount-relevant properties,
  the ownership and mode of its mountpoint and of every parent,
  both `findmnt` answers,
  the GUID digit counts,
  and the paths and name that do not exist here;
- `pin-cost-sixth.log`:
  the commits that touch `main.rs` in the window the fifth revision measured,
  their count,
  and the two commits this round's base moved through;
- `citations-at-this-rounds-base.txt`:
  every citation that names a path,
  printed with the line it names at `5f904d9fbd`, this round's base,
  and `citation-drift-sixth.log`,
  which compares that dump with the fifth round's and shows every citation that moved;
- `occurrences-sixth.txt`:
  the second program's name counted, with and without `--hidden`, for the change list's count;
- `sources/`:
  every file a log in this appendix was made from,
  named for what it is:
  `cut_fences.py` (the fence cutter, one file per printed text),
  `assemble_probe.py` (the probe crate, whose substitutions are listed at its head),
  `guard_replay.py` (the replay driver that writes the log),
  `proposed_main_code.rs` and `proposed_root_mode_names.rs` (the two texts, cut from the document),
  `make_variants.py` (the fifth round's variant maker, kept beside the log, with this round's
  `nm6` added),
  `variants/` (every variant and every scan fixture),
  `fixtures/` (the wiring and manifest fixtures the wiring assertions are shown on),
  `citations.py` (the citation dump),
  `stripper_probe_main.rs` and `stripper_probe_mod.rs` (the round's own probe for the stripper log:
  the fifth revision's printed stripper beside this revision's, both copied into one crate with the
  crate's classifier),
  and `stub_crate_lib.rs`, `stub_crate_Cargo.toml` and `stub_crate_clippy.toml`
  (the stub crate the printed `main.rs` and module are compiled and linted in,
  whose `lib.rs` is the fifth read's stub with the third argument and the start directory added).

The rounds this one replays hold the logs it cites,
and each is cited as a replay where it is used:
the fifth round's
`guards-replay-fifth.log`,
`scan-needles-fifth.log`,
`nested-dataset-properties-fifth.log`,
`pin-cost-fifth.log`,
`units-left-fifth.log`,
`occurrences-fifth.txt` and
`citations-at-this-rounds-base.txt`;
the fourth round's
`guards-replay-fourth.log`,
`names-starts-fourth.log`,
`non-utf8-starts-fourth.log`,
`unit-argv0-fourth.log`,
`warm-reanalysis-fourth.log`,
`host-facts-fourth.log`,
`panic-hook-choice.log`,
`dispatch-cases.log` (thirty-five starts),
`zed-shape-report.txt` (ZED's child step replayed),
`clippy-bans.log` and `clippy.toml`,
`today-lines.log`,
`today-installed-name.log`,
`holder-scan-name.log`,
`handler-usage.log`,
`apparmor-and-folders.txt`,
`cold-start-analysis-rerun.txt`,
`host-wrapper-service-against-handler.txt`,
`host-spread-handler-against-itself.txt`,
`host-wrapper-words-against-handler.txt`,
`guest-disk-cold-wrapper-service-against-handler.txt`,
`guest-disk-cold-spread-handler-against-itself.txt`,
`guest-page-cache-warm-wrapper-service-against-handler.txt`,
`host-warm-wrapper-service-against-handler.txt`,
`host-warm-spread-handler-against-itself.txt`,
`dispatch_probe.rs`,
`zed_replay.c`,
`replay_fixture.rs`,
`coldstart_drop.c` and
`analyse_cold_start.ts`;
and the third round's variant logs.

The fifth read's folder,
`/home/user/temp/agent/gate-review-zfs-root-modes-fifth-revision-2026-11-24/work/keep/`,
holds the logs this round answers,
each cited where it is used:
`read-of-the-fifth-one-program-revision.md`,
`guards-replay-this-read.log` (the planted allow the stripper hid is in it),
`stripper-blind-regions-over-the-release-tree.log`,
`clippy-printed-scan-rules-shadow-reuse.log`,
`clippy-planted-release-variants-with-the-proposals-rows.log`,
`other-ways-to-read-the-start-name-no-guard-names.log`,
`choices-2-6-8-9-11-decisions-measured-on-the-printed-module.log`,
`cmdline-needle-reach-over-the-release-tree.log`,
`stripper-blind-regions-over-the-release-tree.log`,
and its kept upstream sources
`upstream-libzfs_mount-zfs-2.4.4.c` and
`upstream-libzfs_dataset-zfs-2.4.4.c`.

This round's `scratch/` folder held the probe crates and their build trees,
and is removed when the round ends.
Two logs are rebuilt by kept programs:
`guards-replay-sixth.log` by `sources/cut_fences.py`, `sources/assemble_probe.py` and
`sources/guard_replay.py`,
and `stripper-over-the-release-tree.log` by the `tree` mode of the round's own probe crate.
The rest are command transcripts,
and each one's own header prints the commands that made it,
so a reader can run them again as they stand.
No guest,
no container and no ZFS-changing command ran this round,
which changes no code.
