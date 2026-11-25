# The ZFS route's root modes as modes of the one wrapper executable

# The ZFS route's root modes as modes of the one wrapper executable

## Status

A proposal whose eleven choices the owner has answered.
It is still not a decision,
and nothing here is approved or final,
but it no longer asks anything:
each choice's section states the answer it follows,
quotes the record that fixed it,
and keeps the weighed options only as the reason the ranking ran the way it did,
so the change list, the order, the guards, the printed texts and the tests follow one option each.

The answers are recorded in
[the eleven choices record](../decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md),
answered between 2026-11-24T23:00-05:00 and 2026-11-24T23:33:55-05:00 at this design's sixth revision,
which is `a0dad3adb7`.
It records what the project decided earlier in
[the one-program record](../decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md),
what the owner answered about the ZED hook's name in
[the start-name record](../decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md),
and what the owner answered about the boot mount provider and the ordering of plan step 19 in
[the step 19 record](../decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md)
and about which checkouts the route serves and when its texts may change in
[the step 10 record](../decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md).
`doc/decision/` and `doc/handover/` are untouched by this round.

This is the seventh revision,
the round of 2026-11-24T23:55:40-05:00.
It writes the eleven answers in,
and it folds the list a same-vendor second pass made of the sixth revision,
`read-of-the-sixth-one-program-revision.md`
in `~/temp/agent/gate-review-zfs-root-modes-sixth-revision-2026-11-24/keep/`.
That pass is not an independent review:
it is a second reader of the same vendor.
It found no High item,
five Medium items before the owner was asked (its list one) and eighteen after (its list two),
and every one of the twenty-three is answered in
"What this revision does,
and the sixth read's findings it folds".

Every claim about a program this repository does not own is read from that program's source.
Every claim about this repository's code is read at `2b81674c10`,
this round's tip of `main`,
which this round merged as `8e9e3ce7c6`;
that is this round's base,
and every line this document prints or pins is that commit's file with this design's change in it.
A commit that touches `main.rs` or `root_mode_names.rs` before this design lands changes the pin in the
same commit,
which is the maintenance cost the pin's own measurement states in
"The guard shape: two files pinned whole, and lists where a pin cannot reach".
A citation given at another commit says which,
and gives this base's line beside it where the line moved.
The bare `:line` continuations that follow a named file in a list were read by hand at the same base.

A build of the root modes starts from this revision:
`main.rs` and `root_mode_names.rs` are printed whole and pinned whole,
the two new modes' words, arguments, order and refusals are printed,
the guard texts are printed,
and the change list names every other site.
The two files whose text the pins hold are the only files this design fixes line for line;
everything else it names is a site a build edits and its own tests hold.

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
  read at `2b81674c10` unless the citation says another commit;
- (upstream PATH:LINE),
  another program's source,
  read through the file the named round kept;
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

## What this revision does, and the sixth read's findings it folds

Two things happen here.
The eleven answers are written in,
which turns eleven sections that offered options into eleven sections that state what the
build does and keep the options only as the reason for the ranking.
Then the sixth read's twenty-three findings are folded,
in its own two lists and in its own order.

### The guard shape: two files pinned whole, and lists where a pin cannot reach

The last three reads each found new edits that pass the printed guards,
and each revision closed the named ones and opened others,
the sixth revision's comment stripper among them.
That is not a run of bad luck:
a guard that decides a line's meaning by a list of spellings, or by a lexer of its own,
can only be as complete as the spellings and the cases someone has already thought of,
and every read that tries a new one wins.

So the shape is the fifth revision's, and this revision changes what the shape is built from.
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
and a line that is only a comment is dropped before the comparison,
because a whole-line comment carries no behavior.

What the shape catches,
by measurement
(run keep/guard-repairs-seventh.log):

- the pinned text fails every `main` variant and every names variant a read has planted,
  `nm6` included,
  and every one of them is a changed code line,
  so every one of them is a changed pin;
- the controls that must pass do:
  a comment-only edit and a blank line in either file pass,
  because a whole-line comment and a blank line are not code.

What the shape costs a person who edits the file honestly,
measured rather than estimated:

- every honest edit to `main.rs` or to `root_mode_names.rs` edits the pin in the same commit.
  `main.rs` is 27 non-blank code lines at this base and 47 under this design,
  and the repository's history holds ten commits that touch it in the 41 days from
  2026-10-05 to 2026-11-15, the file's creation commit among them,
  seven of them in the 30 days before this round
  (run keep/pin-cost-seventh.log).
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
  1,473 release files as this round classifies them,
  where a pin per file is not maintainable;
- the typed bans in `clippy.toml`,
  which resolve the call rather than match its text;
- the pin over the release tree's lint-level attributes,
  because one `#[allow(` line can excuse a typed ban in a file no pin covers.

Two things this revision removed from the shape,
because the sixth read found a way past each and no repair of them could be shown complete:

- the scan's lexer.
  The scan's needles are matched against the lines as they are written,
  so a line can be hidden only by not existing;
  the price is that a comment spelling a needle is a finding,
  and release code holds two such lines, both in `main.rs`, declared by one row;
- the lint pin's per-line needles.
  It compares whole-attribute anchors built from the file's own text,
  with each reason's prose skipped,
  so neither a split attribute nor a macro-built lint path can step around it.

Both repairs are argued from what they catch and what they cost in
"The sixth read's list two",
and both are replayed against the read's own variants in that list's A1 to A3.

### The answers, written in

Nine of the eleven answers take the option the sixth revision recommended,
so their text stands and only its framing changes.
Two answers rewrite what a build prints,
and they are the ones to read first:

- choice 5 is C, the typed words:
  `--version` prints `monochromatic-git-policy-cli zfs-root <version>`,
  every root-mode refusal opens with `monochromatic-git-policy-cli zfs-root:`,
  and `mount-git-worktree-clone-datasets` is printed nowhere.
  The two words are read from the constants the guards already pin,
  so the line and the refusals cannot drift from choice 2's word;
- choice 9 is E, `--replace <the kernel's guid>`:
  the record mode's usage prints that token,
  the boot refusal prints it,
  and a value that is not the kernel's GUID refuses `replace-token-differs`.

The eleven, in the order the record keeps them, with what each one's section now says:

1.  choice 1, the hook-named file accepts `--help` and `--version` beside ZED's no-argument start,
    which is what `root_mode_names.rs` does as printed;
2.  choice 2, the group word is `zfs-root`,
    so `ROOT_GROUP_WORD` is that one line of the pinned module;
3.  choice 3, root's install folder holds one file under the ZED hook's name,
    so `SERVICE_INSTALL_PATH` is renamed for what it now is and no file is named for the wrapper;
4.  choice 4, the dispatch is the first statements of `main`,
    which is the block the entry pin holds;
5.  choice 5, the typed words, above;
6.  choice 6, the account's `PATH` keeps no short name,
    so `doctor` prints the root-owned path and the words and the toolchain links nothing new;
7.  choice 7, the release keeps the four test-only variables,
    so the guest arms keep testing the released bytes;
8.  choice 8, the record mode's word is `record-checkout`;
9.  choice 9, the kernel's GUID, above;
10. choice 10, a root mode installs the wrapper's payload-free panic hook,
    which is the arm's own first statement;
11. choice 11, the release compiles the deadline variable out,
    so the raised-deadline control builds the probe with a Cargo feature.

The section "The eleven decisions, and where each is written" lists the same eleven against the text
that carries each.

### The sixth read's list one: five repairs, all folded

The read's list one held no High item and five Medium ones,
all of them about text the owner read before answering.
The owner has answered,
and each of the five is folded here so the record of why stands corrected.

- B1, choice 5 gave no reason for the adjacent pair A over B.
  The pair's reason is now printed in the ranking that survives the answer:
  A's first line is unique against the wrapper's banner, while B's is a prefix of it,
  so a check on the first line cannot tell which program answered.
  It is a reason about delivery, and it is the only pair reason the answer's ranking was missing.
- B2, the modes' usage printed E's token and D's token in one block,
  because the sixth revision printed the recommended token while the sentence under it described
  the other option.
  The usage printed in choice 2 now carries E's token alone,
  and D's is stated only in the reason E ranks above it.
- B3, choice 9 stated C's and B's stale-record window as a property of the option,
  although the window is set by what the boot's refusal is allowed to name.
  The design now says what that permission is:
  a refusal names a repair only when the repair carries a value the refusal itself prints,
  which is E's and D's token and neither A's bare flag nor C's re-run of the ordinary command.
  So C's window is longer because the refusal may not teach an admit,
  which is the reason the ranking already gave for putting A and D above C.
- B4, choice 4's "B over C" was not a reason about delivery.
  B and C cost the same start-time work,
  so the pair is now stated as a tie by delivery,
  with C's parameter, which its one in-process caller cannot honestly fill, named as a cost.
  The order stands.
- B5, choice 5's separability statement was true and incomplete.
  Under C the line prints choice 2's word,
  and the design now says so where the coupling is built:
  the version line and the refusal prefix are `format!`s over `COMMAND_WORD.current` and
  `ROOT_GROUP_WORD`,
  the two constants the guards already pin,
  so a change to choice 2's word moves them together and no check reads a second copy.

### The sixth read's list two: the guard holes and the rest

The read's list two held eighteen items,
three of them about guards an editor can walk past,
two about `mount-path-descendant`,
and thirteen labels, counts, citations and mechanisms.
The three guard holes are answered by the shape they share:
a guard that reads text a hand-written lexer decided about cannot be complete,
so this revision takes the lexer out of the guards altogether.

#### A1. The stripper misread raw strings and brought every rule down with it

The read planted three files whose text the sixth revision's stripper read differently from rustc:
`br"\"` and `br#"a"b//"#` opened a string the stripper thought was ordinary,
and a raw string after a closed block comment inherited the previous word's last character.
In each case an `#[allow(clippy::disallowed_methods)]` and a `std::env::args_os()` read sat inside
what the stripper believed was a comment,
so the scan reported nothing, the lint pin passed, Clippy was silent,
and a start returned the name it was started under.

The repair is not a wider lexer, and not a better one either:
it takes the lexer out of every guard this design prints.

- the start-name scan reads each line as it is written and matches its needles against it.
  Nothing is cut, so nothing can be hidden behind a misread:
  a needle inside a string is the read it names, and the read's three files report at the same line
  as their control (run keep/guard-repairs-seventh.log).
  The cost is that a comment spelling a needle is a finding,
  and each such line is a declared row: release code holds two, both in `main.rs`,
  and one `ALLOWED` row covers them;
- the two file pins read their own files with `pinned_code_lines`,
  which cuts nothing either:
  both files are held to the whole-line comment rule the project already asks of them
  (rule TD4, `AGENTS.md`), the rule is asserted first,
  and under it a line's code is the line or, for a line that begins with `//`, nothing.
  Every byte of every literal stays, which is what a pin of a module's constants needs;
- the crate's own masker, `read_the_code_of` at
  `blocking_code_off_the_runtime_tests.rs:226`, stays where it is with the two scans that already
  read through it, and this design adds no user of it.

The sixth revision's printed `Reading` state machine goes entirely,
so the design prints no lexer at all,
and the class the read's three variants opened is closed by construction rather than by three more
cases.
What the honest editor pays is the two doc lines `main.rs` holds, which the scan now reports as
findings a row must declare.

#### A2. The lint pin held attribute lines, not the scopes they excuse

The read showed three ways past the sixth revision's lint-attribute pin:
an `#[allow(clippy` and a `::disallowed_methods)]` on two lines, which the per-line squeeze could
not join; an allow whose lint path a macro builds, which no line spells; and a read inserted under
one of three function-wide allows that already stand in release code,
which the pin cannot see because no attribute line changed.

Three repairs, each chosen by what it catches and what it costs:

- the pin now compares anchors, not lines,
  and it builds them from each file's text as it is written, cutting nothing:
  An anchor is the squeezed text from `#[` through the attribute's closing `]`,
  with each `reason = …` meta item skipped,
  found over the whole file's squeezed text, for every attribute that can set a lint level:
  `allow`, `expect`, `warn`, `deny`, `forbid` and `cfg_attr`.
  A split allow is one anchor, so the first hole closes;
  a macro-built lint path is an anchor no pinned entry names, so the second closes;
  and `cfg_attr` is included because it is the one attribute that can inject another,
  which is what twenty-six of the thirty-four anchors turn out to be:
  `#[cfg_attr(not(test), expect(dead_code))]` and its dylint sibling,
  none of which the sixth revision's pin named.
  The reason is prose, so skipping it means a reason's wording never re-pins,
  while a lint's name, a cfg predicate or an attribute's shape does.
  Release code holds thirty-four anchors today and the design's own `main.rs` adds the
  thirty-fifth (measured: run keep/guard-repairs-seventh.log).
  The thirteen `#![cfg(unix)]` rows the sixth revision pinned are not lint levels and leave the set;
  what arrives instead is every lint-level attribute the tree holds;
- the three function-wide allows are narrowed to the one statement each excuses
  (`worktree_clone_unmount/user_cache_pin.rs:162`, `staging_claim.rs:116`,
  `staged_folder_record.rs:139`).
  Measured: a statement-level `#[allow(clippy::disallowed_methods)]` excuses that statement and
  nothing above or below it,
  so a read inserted at the top of the function fails the typed ban
  (run `keep/statement-allow-scope-seventh.log`);
  the narrowing is compile-safe because `serde_json::json!`'s expansion is still inside the allow
  (same log);
- the scan gains the plain `env::args` needle,
  which is the one spelling the read's alias variant used and neither `::args(` nor `.args()` names,
  and which release code holds only on the two `args_os` lines this design already handles
  (measured: run `keep/guard-repairs-seventh.log`).

What stays is named rather than claimed closed:
a read placed inside the excused macro statement, or inside any other pinned allow's own expression,
is excused by that allow and no text guard sees it.
The three narrowed allows are the whole of that surface in release code today.

#### A3. Three redirects passed the wiring assertion, and the typed ban's own configuration went unread

The read showed that `#[cfg_attr(all(), path = "elsewhere.rs")]`, `#[ path = "elsewhere.rs" ]` and
`# [path = "elsewhere.rs"]` each passed the assertion and each compiled the other file,
that a `#[path]` on the handler folder's declaration in `lib.rs:476` redirected the whole folder
while no guard read that line,
and that deleting the two `clippy.toml` rows or setting `disallowed_methods = "allow"` in the
manifest left every read the guards exist for clean under Clippy.

Two repairs:

- the declaration assertion now reads the declaration's own attribute run.
  It requires exactly one line whose trimmed text is `pub mod <name>;`,
  collects the attribute lines immediately above it, comments and blank lines skipped,
  squeezes them, and compares the result with the pinned run:
  nothing for `worktree_clone_mount_handler/mod.rs`, and `#[cfg(target_os="linux")]` for `lib.rs`.
  Every one of the read's spellings is an attribute in that run, so every one fails,
  and the same assertion now covers `lib.rs`, which no guard read before
  (measured: run `keep/guard-repairs-seventh.log`,
  where `wv1`, `wv2`, `wv3` and `wv5` fail and `wv4` passes because it declares a different module,
  not a redirect of the pinned one);
- the typed ban's configuration is pinned.
  `typed_ban_configuration_tests.rs` requires the manifest's `[lints.clippy]` table, squeezed, to be
  the reviewed one, and every `path = "..."` value in `clippy.toml`, in file order, to be the
  reviewed set of nine.
  `disallowed_methods` at any other level, a removed row and a retargeted row all fail;
  a reason's prose does not, because only the paths are compared.

What the honest editor pays: adding a lint-level attribute, a lint level or a disallowed call
re-pins, and editing a reason does not.
That is the same cost the two file pins already charge, for the same reason.

#### A4. `dataset-not-mountable` gave the wrong reason, and two of upstream's refusals were missing

The sixth revision said `canmount=off`, `zoned=on` and `redacted` are "the three `zfs mount` answers
by returning success and mounting nothing",
and rested the refusal on libzfs's `zfs_mount()`.
That is true of libzfs, whose `zfs_is_mountable` answers false and whose caller returns 0
(upstream `lib/libzfs/libzfs_mount.c`, zfs-2.4.4, `:299`, `:270` and `:305`).
It is not true of the command the unit runs today:
an explicit `zfs mount <dataset>` prints an error and returns 1 for each of them
(`share_mount_one` in `cmd/zfs/zfs_main.c`, zfs-2.4.4, at `:7196`, `:7275` and `:7327`).
The mode's refusals stay;
their reason is now that an explicit mount fails loudly and a unit that reported success while
mounting nothing is the failure the refusal replaces.

Two refusals were missing beside them, and both are added:

- an inconsistent dataset that holds a receive resume token is refused by the same function at
  `:7313`,
  so `dataset-not-mountable` names it and says the receive can be resumed with `zfs send -t`;
- a pool imported read-only forces every mount read-only
  (upstream `libzfs_mount.c:428`),
  so the mode reads the pool's `readonly` property and passes `MOUNT_ATTR_RDONLY` with the
  attributes when it is on, rather than mounting read-write under a read-only pool.

`redacted`'s wording changes with it:
the property is a count, not a switch, so the refusal says "is not 0"
(this host's dataset answers `0`, run keep/host-facts-seventh.log).

#### A5. The order of `mount-path-descendant`'s proofs made one refusal unreachable and stacked a mount

Two defects, both in the order the section states:

- the sixth revision's step 2 asked for "the recorded path is a mount point, and the dataset mounted
  there is the record's dataset" before step 3 asked whether the checkout is mounted at all.
  A delayed checkout mount therefore refused `checkout-record-no-longer-matches`,
  the refusal choice 9 steers to `--replace`,
  and `checkout-not-mounted`, the fact the step 19 acceptance list needs, was unreachable.
  Step 2 now reads dataset facts by name: the recorded dataset exists, its GUID is the recorded one,
  its `mountpoint` property is the recorded path, and the container is still beside it.
  Step 3 reads the mount table and refuses `checkout-not-mounted`;
- the already-mounted answer stood after the create and the mount,
  so a re-run on a dataset the kernel already had mounted walked to the mount as its last hop and
  placed a second mount on it.
  It now stands before the walk and before any act,
  with `dataset-already-mounted-elsewhere` beside it,
  so an `ExecStart=` re-run writes nothing.

What it does to the one nested dataset at this host's boot is unchanged in outcome and now right in
order:
with `canmount=noauto` and the record written, the record agrees by name, the mount table shows the
checkout mounted at its recorded path, the dataset's properties pass, the kernel does not show the
dataset mounted, the walk opens the mountpoint beneath the checkout's mount,
and the mode mounts it with `noatime` and the five default pairs.
Every property this rests on was re-read this round
(run keep/host-facts-seventh.log).

#### A6 to A18: the change list, the citations and the counts

- A6.
  The change list kept the family name under choice 5's option A in two places while the design
  recommended C.
  Both are now written for C:
  `PROGRAM_NAME` goes, the version line and every refusal are the words,
  and the `kept` list says what the modes keep instead of the name.
- A7.
  Ten `(run keep/<earlier log>)` labels named earlier rounds' logs, three of them inside
  owner-facing blocks.
  Each now reads `(replay …)`, and this round's own logs are the ones the appendix names.
- A8.
  Three `entry.rs` citations named the wrong lines (`:348`, `:353`, `:356` are `:366`, `:371`,
  `:372` at this base), `plan:2599` is `:2598`, the `clone_slot_walk.rs:38` citation was a doc line
  about a constant and now names the walk's own `use` at `:30`, and the appendix put the two kept
  upstream libzfs files in the fifth read's folder when they are in the fourth read's `keep/`.
  Each is corrected where it stands.
- A9.
  Four relative pointers stood in prose although the document said none did.
  Each now names its heading or its line.
- A10.
  Three counts were wrong and one site was in neither list:
  `binary_zfs_step9_m_arms_tests.rs` holds one start at `:200` and two threads at `:561`,
  so three and not four or two;
  `binary_mount_git_worktree_clone_datasets_tests.rs:106` is an `ldd` list entry and now stands in
  the list of sites that only lose an array entry;
  and `release_sources_in`, which exists privately in two test files, is the copy in
  `no_self_invocation_tests.rs:52` that moves.
- A11.
  Choice 9's prose listed "the record still agrees with the kernel's dataset properties" among the
  proofs every option runs before it writes, omitted E from the options that change the setup's
  line, said the boot refusal prints the record's GUID alone, and counted two options leaving a
  named command where the order section counted three.
  All four now say one thing:
  a replace runs because the record differs in the one field the flag may replace,
  E is one of the options that changes that line,
  the refusal prints both GUIDs, and A, D and E leave a repair a refusal names.
- A12.
  The count of release lines holding a `//` inside a string is eight, not nine;
  the ninth release line the sixth round counted is a real trailing comment.
  The sentence now says so, and this revision's reader no longer needs the figure at all.
- A13.
  The mount-relevant property list was called fifteen and the log held fourteen.
  This round's log holds eighteen, because `redacted`, `inconsistent` and `receive_resume_token`
  joined the list, and the text counts eighteen.
- A14.
  `root_mode_name_tests.rs` imported the Linux-only handler module for one constant,
  which no release target compiles on another system,
  so only a developer's non-Linux `cargo test` would have noticed.
  The assertion now reads the constant's text out of `handler_install_paths.rs`,
  which is portable and keeps the tie between the installed name and the start name.
- A15.
  The residual list of ways to read a start name gains an `.init_array` constructor that receives
  `argv`, which the read measured returning the name with no needle hit
  (replay the read's `keep/init-array-constructor-reads-the-start-name.log`).
- A16.
  The costs of the whole-line comment rule are now stated where the rule is:
  a `/*` anywhere in either pinned file fails, a `//` inside a literal included,
  and the declaration assertion no longer fails on a comment,
  because it reads the declaration's own line and the attribute run above it.
- A17.
  The pin-cost figure is ten commits in the window, the file's creation commit counted, and the
  stripper's lost-line figure is gone with the stripper.
  What remains of it is the three files whose glob strings hid 794 non-blank lines from every rule
  that trusted the old reader (76, 331 and 387 lines, the opening line counted in each),
  which is why the reader is now the crate's own.
- A18.
  Two option blocks printed output with no label of what it rested on.
  Both are labelled `(proposal)` where they stand.

### What this revision left as the sixth revision wrote it

The two pinned files' text,
the scan's six needle sets other than the two this revision changes,
the readings of ZED, systemd and the kernel,
the cold-start measurements,
and the two new modes' words and arguments are the sixth revision's,
and this revision changed only what the findings above name.
The one dataset, its properties and its mountpoint were re-read here,
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
(`2b81674c10`,
whose `main.rs` the same file a commit after the fifth revision's base changed)
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

- `SPEC.md:19873`
  (`:19715` at the sixth revision's base, `:19705` at the fifth's, `:19261` at `f3d81e0910`),
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

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> The answer: "A: also --help, --version (Recommended)".

> So `<file>` alone runs the clone-mount sweep,
> `<file> --help` and `<file> --version` print,
> every other word under that name is refused with exit 2 before Git can run,
> and `monochromatic-git-policy-cli zfs-root <mode>` selects a mode on any file.

So the design follows one rule,
and the module that decides it is printed whole in "The rule,
in the entry":

- the two words `monochromatic-git-policy-cli zfs-root`,
  followed by a mode word,
  select a root mode on **any** file,
  and that is the clause a unit's `ExecStart`,
  an adoption and a person use;
- under the installed ZED history event's name,
  ZED's no argument is the sweep,
  and `--help` and `--version` are the modes' own answers;
- under that name,
  any other word is refused before the modes' grammar sees it,
  with the usage and status 2;
- every other start is the wrapper.

The name is read as the whole last path component,
which `Path::file_name` answers,
and both refusals answer status 2,
which is `EXIT_REFUSED`
(source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs:45).

### What the answer decides, case by case

Each case is what the chosen rule decides,
measured by starting the compiled module under the named `argv[0]`
(replay keep/names-starts-fourth.log,
whose module text is this design's;
the shell-scale cases are replay keep/dispatch-cases.log).
The order is the dispatch's: the words first, then the one name, then the service's words.

- ZED's own child step,
  the bare name and no argument:
  the attaching sweep.
  ZED passes one argument element and no words,
  so this is the only shape its start has
  (replay keep/zed-shape-report.txt).
- the installed path,
  the words and `boot-reconcile`,
  which is a unit's `ExecStart`:
  `boot-reconcile`.
  The words are read before the name,
  which is what lets this line work at the installed path,
  so that order is load-bearing.
- the installed path,
  `--help` and `--version`:
  the modes' own help and version,
  which is the answer's addition and the word-free check of the installed bytes.
- the installed path,
  the words and no mode word:
  a refusal naming the modes,
  never a sweep
  (section "`zfs-root` alone refuses with usage").
- the installed path,
  a bare `boot-reconcile`,
  which step 19's earlier line and a person's guess use:
  a refusal before the grammar,
  so no argument vector at that path reaches Git.
- the installed path,
  the group word misspelled,
  the mode word misspelled,
  a trailing slash,
  doubled slashes,
  `./` before it,
  `sub/../` before it:
  a refusal for the two typos,
  and the mode or the sweep for the four spellings of the path,
  because `Path::file_name` reads the last component the same way.
- the bare name through a `PATH` lookup:
  the sweep.
- five near-miss names:
  the wrapper,
  which is what a whole-component comparison buys and a contains or suffix test would not.
- a stray copy under another name:
  the wrapper.
- a stray copy under the zedlet's own name in another folder:
  the sweep with no argument,
  a refusal with a bare word,
  because the rule reads the name and not the file.
- the wrapper's own name with the words:
  the mode;
  with the words and no mode word:
  a refusal.
- an adoption's three options through the words:
  `adopt-worktree-parent` with them.
- the two new modes through the words:
  each one.
- a caller-chosen `argv[0]` through `exec -a` that is not the name:
  the wrapper for a bare word,
  and the mode for the words,
  which is the words clause at work on a file no name selects.
- an empty vector and a vector of one empty element:
  the wrapper,
  because the last component of `""` is none.
  The kernel inserts the empty element,
  so the rule decides both without indexing anything
  (section "An empty `argv[0]` and a NULL vector are one empty argument").
- a start whose name is not UTF-8:
  the wrapper,
  since a name this rule cannot compare is not the zedlet's
  (replay keep/non-utf8-starts-fourth.log).

### Why the ranking ran this way

The record asked what the file under the hook's name accepts beyond ZED's start with no argument,
and ranked one file under the hook's name above one named for the modes,
above one named for the wrapper,
above a copy or link per name;
this choice ranked the three rules `A > B > C`,
and the record's answer took A.
What each pair rested on is kept here because it is the reason the text reads as it does.

A over B:
both fail closed for the installed file,
and both let the words select a mode on any file,
which is the answer's second clause.
B refuses every word under the installed name,
`--help` and `--version` included,
which is the answer's literal minimum.
A answers those two words there,
so the release smoke's first arm and a person's check of the installed bytes are one command with no
words in them,
and the dispatch is proved positive before any sweep runs.
B proves the same dispatch with a refusal,
which answers `EXIT_REFUSED` where Git would print its version and exit 0,
so A's gain is two words fewer and a positive answer,
not the only proof.
A's cost, disclosed, is that two more spellings are accepted under the name.

B over C:
C is the first revision's rule,
where the name wins and the words pair is dropped under it,
so `boot-reconcile` alone at that path starts a mode.
That accepts a third spelling the answer never names,
it makes the words stop being what selects a mode under the recommended layout,
and it is the only one of the three that starts a mode for a bare word under a stray copy of the
zedlet's name.
B refuses every one of those.

### What the answer costs, and what it does not decide

The cost is the two spellings:
`--help` and `--version` at the installed path are root-mode starts,
so a stale line that means Git's version there answers the modes' version instead.
That is stated in the release smoke's first arm and in `doctor`'s planned check,
and it is the price of the word-free check.

The answer does not decide the layout,
the dispatch's placement,
the panic hook or the modes' text.
Each of those is its own choice,
and each is now stated with the answer it follows.

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

So under the layout choice 3's answer fixes
both spellings reach the modes,
and the words remain what selects a mode:
a unit line works whether the file it starts is the installed name or another,
and the same unit line works against a stray copy.
That is the property the answer needs and the first revision's pair-dropping did not have.

### The guard change, exactly

Five guards change or arrive,
and the shape is the fifth revision's:

- the two files that decide what a start is are pinned whole,
  so any change to either is a change to its guard;
- the scan keeps needles,
  because a pin per file over 1,473 release files is not maintainable;
- a pin holds the release tree's lint attributes,
  because one `#[allow(` line can excuse a typed ban in a file no pin covers;
- five assertions hold the wiring the text pins cannot see:
  the module's declaration and its attribute run,
  the handler folder's declaration and its attribute run,
  the words' first word,
  the installed ZED path's file name,
  and the `[[bin]]` `path`
  (this revision's answer to the sixth read's A3);
- a fifth guard reads the typed ban's own configuration,
  because a level set to `allow`, a removed row or a retargeted row silences every call the ban
  names without changing a line of source
  (this revision's answer to the same finding).

#### The hook guard on `main.rs`

`main_does_not_dispatch_on_its_start_name` becomes
`main_is_the_reviewed_handful_of_statements`,
and its body is this, whole,
with the file's code text pinned in one constant:

```rust
// package/git-policy/cli/src/native/source_guard/hook_self_invocation_tests.rs
/// The pins' reader and the comment rule: a line as written, and comments on their own line only.
use super::{assert_whole_line_comments_only, pinned_code_lines};

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
    let code: Vec<String> = pinned_code_lines(MAIN_SOURCE, "main.rs").into_iter().filter(|line| { return !line.is_empty(); }).collect();
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

#### The readers: no lexer in any guard this design prints

The sixth revision had one hand-written lexer, `code_lines`, that cut comments and could not tell a
literal from code.
The sixth read walked past it in three ways,
and the lesson is not that the lexer needed three more cases:
a reader that can disagree with rustc anywhere hides every line after the disagreement from every
rule that trusts it.
So this revision prints no lexer at all.
The three readings a guard needs are these,
and two of them need nothing more than a line and a squeeze:

- the scan reads the file's lines as they are written,
  squeezes the whitespace out of each,
  and matches its needles against that text.
  Nothing is cut, so nothing can be hidden:
  a needle inside a string is the read it looks like,
  and a needle inside a comment is a finding the `ALLOWED` row has to declare.
  The cost is exactly that second half, and it is measured in the scan's section;
- the lint-attribute pin reads the file's text as it is written,
  squeezes it,
  and builds each anchor from `#[` to the attribute's closing `]`,
  skipping each `reason = …` meta item because a reason is prose.
  Nothing is masked, so a masking mistake cannot hide an attribute,
  and no reason's wording enters the comparison;
- the two pinned files are read by `pinned_code_lines`,
  which also cuts nothing:
  both files are held to the whole-line comment rule (rule TD4, `AGENTS.md`),
  the rule is asserted first,
  and under it a line's code text is the line itself or, for a line that begins with `//`,
  nothing.
  Every byte of every literal stays, which is what a pin of a module's constants needs.

`source_guard/mod.rs` gains these three items,
all `pub(super)`:

```rust
// package/git-policy/cli/src/native/source_guard/mod.rs
/// The text with every whitespace character removed, so `args_os ()` reads as `args_os()` and
/// `clippy ::` as `clippy::`.
pub(super) fn squeezed(text: &str) -> String {
    return text.chars().filter(|character| { return !character.is_whitespace(); }).collect();
}

/// What:
///  One pinned file's code text: each line as written,
///  a line that is only a comment dropped, leading and trailing blanks trimmed.
/// Why:
///  The two files this reads are held to comments on their own line,
///  which the assertion below checks,
///  so cutting at a leading `//` is exact and no lexer decides what a line is.
///  Every byte of every literal stays, because a module's constants and its format text are behaviour.
pub(super) fn pinned_code_lines(source: &str, what: &str) -> Vec<String> {
    assert_whole_line_comments_only(what, source);
    return source.lines().map(|line| {
        let trimmed: &str = line.trim();
        return if trimmed.starts_with("//") { String::new() } else { String::from(trimmed) };
    }).collect();
}

/// Every `//` in the source begins its own line, so cutting a line at the first one hides nothing,
/// and no block comment spans lines a reader would have to reconstruct.
pub(super) fn assert_whole_line_comments_only(what: &str, source: &str) {
    assert!(!source.contains("/*"), "{what} holds a block comment, which no line-at-a-time reader can cut exactly");
    for line in source.lines() {
        if let Some(at) = line.find("//") {
            assert!(line[..at].trim().is_empty(), "a comment starts after code in {what}, and a line-at-a-time reader would cut what follows it: {line}");
        }
    }
}
```

`pinned_code_lines` returns one entry per line, blanks included,
so a line's index is its line number,
and each pin filters the blank entries out before comparing.

What the whole-line rule costs is stated rather than hidden:
a `/*` anywhere in either pinned file fails, a `/*` inside a `//` comment included,
and so does a `//` inside a literal.
Neither file holds either today,
and the project's own comment rule already asks for comments on their own line,
so the cost falls only on a future edit that wants a block comment or a trailing `//` in one of the
two files this design fixes line for line.
The alternative the fifth read offered was requiring it of the whole release tree:
63 release files hold `/*` somewhere and eight release lines hold a `//` inside a string,
so that rule would have reworded 63 files to buy nothing here
(replay keep/stripper-over-the-release-tree.log,
the sixth round's measurement of the tree at its own base).

The crate's own masker, `read_the_code_of` at `blocking_code_off_the_runtime_tests.rs:226`,
stays where it is and keeps its two users:
this design adds none,
because the scan needs literal text and the lint pin needs no masking at all.
That is the whole of the answer to the sixth read's first finding,
and it is why the design prints no `Reading` state machine and no second lexer.

#### The scan rule's needles

`no_self_invocation_rules_tests.rs` keeps its rules and gains one,
and the wrapper squeezes each line before a rule sees it,
so a call split by a space still matches.
The rules take one text, not two:
the pair the sixth revision passed beside every line was never read by any rule of this file
(every one of them matched `line` alone),
and matching a two-line join reports a needle on the line above it as well as on its own,
which is a finding a reader has to discount.
So the pair goes, and what it never caught is stated in the residual below.

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs
Rule {
    id: "argv0-dispatch",
    reason: "reads the program name the wrapper was started as, which only matters for a wrapper that is started under another name",
    matches: |line| {
        return ["args_os().next()", "args().next()", ".nth(0)", "argv[0]", "arg0(", "/proc/self/cmdline", "/proc/self/comm", "/proc/thread-self/"]
            .iter()
            .any(|needle| { return line.contains(needle); });
    },
},
Rule {
    id: "argv-vector-read",
    reason: "reads the argument vector, whose first element is the name the program was started under; the one collection belongs to main, which hands the whole vector to root_mode_names",
    matches: |line| {
        return ["env::args", "args_os", "::args(", ".args()"].iter().any(|needle| { return line.contains(needle); });
    },
},
```

and the type the rules are declared with changes to match:

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs
/// One kind of forbidden text.
pub(super) struct Rule {
    /// The name an `ALLOWED` entry gives.
    pub(super) id: &'static str,
    /// Why the text is forbidden.
    pub(super) reason: &'static str,
    /// Whether one squeezed source line holds the forbidden text.
    pub(super) matches: fn(&str) -> bool,
}
```

The scan's own reader is where the raw reading lives:

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_tests.rs
/// The squeezed text of each line of a source that holds anything at all, with its line number.
/// Why nothing is cut from a line first:
///  a forbidden read is usually a string, as `"/proc/self/cmdline"` is, so cutting literals would
///  hide exactly what this scan looks for;
///  and cutting comments would let a reader that disagrees with rustc hide a line, which is what the
///  sixth revision's lexer did.
///  A comment that spells a needle is therefore a finding, and each such line is a declared `ALLOWED`
///  row: release code holds one today, and it is in `main.rs`.
fn code_lines(source: &str) -> Vec<(usize, String)> {
    let mut code: Vec<(usize, String)> = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let squeezed_line: String = super::squeezed(line);
        if squeezed_line.is_empty() {
            continue;
        }
        code.push((index + 1, squeezed_line));
    }
    return code;
}
```

with one `ALLOWED` row and one new sample:

```rust
Allowed {
    file: "main.rs",
    rule: "argv-vector-read",
    why: "the one vector the owner's answer needs: main collects it once, and the pin on main.rs fixes the one line that names it; the same row covers the two doc lines above it that name `args_os`, because this scan cuts nothing",
},
```

(the `samples` array at `no_self_invocation_tests.rs:191` is `[(&str, &str); 6]`,
so it is written with a `7` and gains
`("argv-vector-read", "use std::env::args_os as the_vector;")`,
which `:208`'s length assertion then carries).

What the raw reading costs, measured over the release tree
(run keep/guard-repairs-seventh.log):

- four lines hold a needle where nothing is cut:
  `main.rs:52` and `mount_git_worktree_clone_datasets/main.rs:49`, the two collections this design
  replaces with one, and the two doc lines above them that name `args_os`;
- after this design lands the second program is gone and the one collection is allowed,
  so the release tree's findings are the two doc lines of `main.rs`,
  and the one `ALLOWED` row declares them;
- no other release file holds a needle in prose, a string or code.

Why these needles and not a broad `/proc/self/`:

- measured over the release tree in keep/scan-needles-fifth.log,
  `/proc/self/` stands in 46 code lines of 27 files,
  `/proc/self/cmdline` in none,
  `/proc/self/comm` in none and
  `/proc/thread-self/` in none;
- `env::args` catches an alias import, `use std::env::args as the_vector;`,
  which neither `::args(` nor `.args()` names,
  and it costs no row beyond the one the design writes;
- `args_os` stands in the two collection lines,
  `::args(` in none and
  `.args()` in none,
  and the last of those is what catches `std::env::args()` and `env::args()` written without an
  import;
- so the needles name the argument vector and the name's own files without failing today's tree
  on anything but `main.rs`'s own two lines;
- the four spellings that release code does read today stay legal:
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
through `getauxval(AT_EXECFN)`,
and through an `.init_array` constructor, which receives `argv` before `main` runs and names no
needle in its text
(replay the fifth read's `keep/other-ways-to-read-the-start-name-no-guard-names.log` and its
`keep/init-array-constructor-reads-the-start-name.log`,
which measured each of those returning the name a start was given).
`/proc/<pid>/cmdline` stays out of the scan because a needle that named `/cmdline` would fail today's
tree on the kernel command line that `health/doctor_filesystem.rs:65` reads,
and the one spelling that would not,
`}/cmdline`,
is a `format!`'s closing brace rather than a path
(replay the fifth read's `keep/cmdline-needle-reach-over-the-release-tree.log`).
A read of any of them inside `main.rs` is a pinned-line change,
and every other file that reads the name is a review, not a scan.

A call split across a line break is not caught by the scan, and that is stated here rather than
implied:
the scan reads one line at a time,
and the typed ban, which resolves the call rather than its text,
is what refuses `std::env::args` however it is spelled across lines.

Each added needle has a witness of its own,
and each file release code reads today has a control it must not fire on:

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_tests.rs
#[test]
fn each_added_needle_fires_on_its_own_witness_and_no_release_read() {
    let dispatch: &Rule = RULES.iter().find(|rule| { return rule.id == "argv0-dispatch"; }).expect("argv0-dispatch");
    let vector: &Rule = RULES.iter().find(|rule| { return rule.id == "argv-vector-read"; }).expect("argv-vector-read");
    for witness in ["std::fs::read(\"/proc/self/cmdline\")", "std::fs::read_to_string(\"/proc/self/comm\")", "std::fs::read_to_string(\"/proc/thread-self/comm\")"] {
        assert!((dispatch.matches)(witness), "argv0-dispatch does not fire on {witness}");
    }
    for witness in ["use std::env::args_os as the_vector;", "let all = std::env::args().collect::<Vec<_>>();", "let name = e::args();", "use std::env::args as the_vector;"] {
        assert!((vector.matches)(witness), "argv-vector-read does not fire on {witness}");
    }
    for control in ["std::fs::read_to_string(\"/proc/self/stat\")", "std::fs::read_to_string(format!(\"/proc/self/task/{tid}/io\"))", "std::fs::read_to_string(format!(\"/proc/{pid}/comm\"))"] {
        assert!(!(dispatch.matches)(control), "argv0-dispatch fires on {control}, which release code reads today");
    }
}
```

#### The lint-attribute pin

`source_guard/lint_attribute_tests.rs` is new,
and it pins every attribute in release code that can set a lint level,
because one such attribute can excuse a typed ban in a file no text pin covers.
It compares anchors rather than lines:
an anchor is the squeezed text from `#[` through the attribute's closing `]`,
with each `reason = …` meta item left out.

`release_sources_in` moves to `source_guard/mod.rs` as `pub(super)` so every guard classifies the
tree the same way,
and the pin reads each file's text as it is written,
through nothing at all:

```rust
// package/git-policy/cli/src/native/source_guard/lint_attribute_tests.rs
use super::{release_sources_in, squeezed};
use crate::source_scan_tests::{real_roots, source_root};
use std::path::PathBuf;

/// The attributes that name a lint level, and so can silence one.
const LINT_LEVEL_ATTRIBUTES: [&str; 6] = ["allow(", "expect(", "warn(", "deny(", "forbid(", "cfg_attr("];

/// Every release attribute that can set a lint level, with the file it stands in: the whole set.
/// An anchor is the squeezed text from `#[` through the attribute's closing `]`,
/// so an attribute split over two lines is one anchor and a reason's prose is not part of it.
const LINT_LEVELS: [(&str, &str); 35] = [
    ("foundation/call_catching_panics.rs", "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]"),
    ("foundation/scoped_start.rs", "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]"),
    ("foundation/side_by_side_threads.rs", "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]"),
    ("invocation/real_git.rs", "#[allow(clippy::disallowed_methods)]"),
    ("landing/landing_state_carried_by_its_task.rs", "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]"),
    ("landing/landing_state_carried_by_its_task.rs", "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]"),
    ("landing/service_land_finish_branch_questions.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_git_steps_and_import_stream.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_git_steps_and_import_stream.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_git_steps_and_import_stream.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_git_steps_and_import_stream.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_git_steps_and_import_stream.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_side_reference_write.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_side_reference_write.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_side_reference_write.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_side_reference_write.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_side_reference_write.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_land_side_reference_write.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_prepare_own_git_steps.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_prepare_own_git_steps.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_scratch_ref_of_a_dropped_batch.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_scratch_ref_of_a_dropped_batch.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("landing/service_work.rs", "#[cfg_attr(not(test),expect(unused_mut))]"),
    ("landing/service_work.rs", "#[cfg_attr(not(test),expect(unused_mut))]"),
    ("main.rs", "#[allow(clippy::disallowed_methods)]"),
    ("process/process_identity.rs", "#[cfg_attr(all(target_os=\"linux\",not(test)),allow(dead_code))]"),
    ("push/auto_push_reconcile.rs", "#[cfg_attr(dylint_lib=\"monochromatic_lint_closure\",expect(avoidable_closure))]"),
    ("push/bounded_query_input.rs", "#[cfg_attr(not(test),expect(dead_code))]"),
    ("worktree_clone_unmount/staged_folder_record.rs", "#[allow(clippy::disallowed_methods)]"),
    ("worktree_clone_unmount/staged_unmount.rs", "#[allow(clippy::large_enum_variant)]"),
    ("worktree_clone_unmount/staging_claim.rs", "#[allow(clippy::disallowed_methods)]"),
    ("worktree_clone_unmount/unmount_checks.rs", "#[allow(clippy::result_large_err)]"),
    ("worktree_clone_unmount/unmount_record.rs", "#[allow(clippy::result_large_err)]"),
    ("worktree_clone_unmount/unmount_record.rs", "#[allow(clippy::result_large_err)]"),
    ("worktree_clone_unmount/user_cache_pin.rs", "#[allow(clippy::disallowed_methods)]"),
];

/// Every lint-level anchor in one source, in the order the source writes them.
/// Nothing is cut from the source first:
///  the scan's needles are often literal text, and this pin's claim is about code,
///  so the two cannot share one reader that decides what a line is.
fn anchors(source: &str) -> Vec<String> {
    let flat: String = squeezed(source);
    let bytes: Vec<char> = flat.chars().collect();
    let mut found: Vec<String> = Vec::new();
    let mut at: usize = 0;
    while at < bytes.len() {
        let opened: Option<usize> = if bytes.get(at) == Some(&'#') && bytes.get(at + 1) == Some(&'!') && bytes.get(at + 2) == Some(&'[') { Some(3) } else if bytes.get(at) == Some(&'#') && bytes.get(at + 1) == Some(&'[') { Some(2) } else { None };
        let Some(start) = opened else {
            at += 1;
            continue;
        };
        let rest: String = bytes[at + start..].iter().collect();
        if !LINT_LEVEL_ATTRIBUTES.iter().any(|name| { return rest.starts_with(name); }) {
            at += 1;
            continue;
        }
        let mut anchor: Vec<char> = Vec::new();
        let mut depth: usize = 0;
        let mut index: usize = at;
        while index < bytes.len() {
            let here: String = bytes[index..].iter().take(7).collect();
            if here == "reason=" {
                if anchor.last() == Some(&',') {
                    anchor.pop();
                }
                index = after_one_meta_value(&bytes, index + here.len());
                continue;
            }
            let character: char = bytes[index];
            anchor.push(character);
            match character {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ']' if depth == 0 => {
                    index += 1;
                    break;
                }
                _ => {}
            }
            index += 1;
        }
        found.push(anchor.into_iter().collect());
        at = index;
    }
    return found;
}

/// Where the value of one skipped meta item ends: a parenthesised group, a string, or a bare word.
fn after_one_meta_value(bytes: &[char], from: usize) -> usize {
    let mut cursor: usize = from;
    match bytes.get(cursor) {
        Some('(') => {
            let mut depth: usize = 0;
            while cursor < bytes.len() {
                match bytes[cursor] {
                    '(' => depth += 1,
                    ')' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return cursor + 1;
                        }
                    }
                    _ => {}
                }
                cursor += 1;
            }
            return cursor;
        }
        Some('"') => {
            cursor += 1;
            while cursor < bytes.len() {
                if bytes[cursor] == '\\' {
                    cursor += 2;
                    continue;
                }
                if bytes[cursor] == '"' {
                    return cursor + 1;
                }
                cursor += 1;
            }
            return cursor;
        }
        _ => {
            while cursor < bytes.len() && bytes[cursor] != ',' && bytes[cursor] != ')' {
                cursor += 1;
            }
            return cursor;
        }
    }
}

#[test]
fn the_release_code_sets_a_lint_level_only_where_this_pin_names_it() {
    let root: PathBuf = source_root();
    let mut found: Vec<(String, String)> = Vec::new();
    for file in release_sources_in(&root, &real_roots()).expect("the release sources") {
        let text: String = std::fs::read_to_string(root.join(file.as_str())).expect("source file");
        for anchor in anchors(text.as_str()) {
            found.push((file.clone(), anchor));
        }
    }
    let mut named: Vec<(String, String)> = LINT_LEVELS.iter().map(|(file, anchor)| { return (String::from(*file), String::from(*anchor)); }).collect();
    found.sort();
    named.sort();
    assert_eq!(found, named, "release code holds a lint-level attribute this pin does not name, and an allow lists the lints the typed bans stop applying");
}
```

The set is measured, not guessed:
release code holds thirty-four anchors today, and this design's `main.rs` adds the thirty-fifth
(run keep/guard-repairs-seventh.log).
What the pin catches is every way past the sixth revision's line-per-attribute shape:

- an allow split across two lines is one anchor, so a new one is a new anchor;
- an allow whose lint path a macro builds is an anchor no entry names;
- an `#[allow(warnings)]` or a `#![allow(...)]` is an anchor like any other,
  so the old `(warnings`, `#![` and `clippy::` needles are no longer needed;
- a `cfg_attr` that injects a lint level is included,
  which is the one attribute that can carry another,
  and twenty-six of the thirty-four anchors are exactly that:
  the `#[cfg_attr(not(test), expect(dead_code))]` and
  `#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure))]` lines the sixth
  revision's pin never named;
- a reason's prose is skipped, so fixing it re-pins nothing,
  while changing a lint's name, a cfg predicate or an attribute's shape does;
- nothing is masked, so a masking mistake cannot hide an attribute from this pin.

The thirteen `#![cfg(unix)]` lines the sixth revision pinned are not lint levels and leave the set:
a `cfg` changes which code a build compiles, which every build shows,
and not which lint level applies to it.
The pin therefore holds thirty-five rows where the last held twenty,
and its claim is wider than the last one's on both counts:
every lint-level attribute, and no reliance on a lexer.
What it still cannot see is stated where the typed ban is:
a read placed inside an excused statement is excused by that statement's allow.

#### The one-name guard on the names module

`source_guard/root_mode_name_tests.rs` is new,
declared in `source_guard/mod.rs`,
and it pins the module whole:

```rust
// package/git-policy/cli/src/native/source_guard/root_mode_name_tests.rs
use super::{assert_whole_line_comments_only, pinned_code_lines, squeezed};
use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;
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
    let code: Vec<String> = pinned_code_lines(NAMES_SOURCE, "root_mode_names.rs").into_iter().filter(|line| { return !line.is_empty(); }).collect();
    assert_eq!(code.len(), NAMES_CODE.len(), "root_mode_names.rs has {} code line(s), and the reviewed text has {}: {code:#?}", code.len(), NAMES_CODE.len());
    for (at, (found, want)) in code.iter().zip(NAMES_CODE.iter()).enumerate() {
        assert_eq!(found.as_str(), *want, "root_mode_names.rs's code line {} is not the reviewed one", at + 1);
    }
}

/// What a declaration of one module in a source stands under:
/// whether the file declares it exactly once, and the attribute lines immediately above that
/// declaration, squeezed.
/// Why the attributes are read rather than the whole file:
///  a `#[path]` beside a declaration makes a different file the compiled module, so the pin above
///  would keep passing the old one;
///  and the attribute run, not the file, is the thing that decides which file a `mod` names,
///  which is why `lib.rs`'s own `#[path]` on another module is allowed to stand.
fn declaration_run(source: &str, declaration: &str) -> (usize, String) {
    let lines: Vec<&str> = source.lines().collect();
    let declared: Vec<usize> = lines.iter().enumerate().filter(|(_at, line)| { return line.trim() == declaration; }).map(|(at, _line)| { return at; }).collect();
    let Some(at) = declared.first().copied() else {
        return (0, String::new());
    };
    let mut run: Vec<String> = Vec::new();
    let mut cursor: Option<usize> = at.checked_sub(1);
    while let Some(line_at) = cursor {
        let trimmed: &str = lines[line_at].trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            cursor = line_at.checked_sub(1);
            continue;
        }
        if !trimmed.starts_with('#') {
            break;
        }
        run.push(String::from(trimmed));
        cursor = line_at.checked_sub(1);
    }
    run.reverse();
    let squeezed_run: String = squeezed(run.join("\n").as_str());
    return (declared.len(), squeezed_run);
}

/// The pinned file is the file the crate compiles as that module,
/// because a `#[path]`, a `#[cfg_attr]` or a second declaration would make a different file the
/// module while the pin above kept passing the old one.
#[test]
fn the_pinned_module_is_the_module_the_crate_compiles() {
    let (declarations, run): (usize, String) = declaration_run(HANDLER_MODULE_SOURCE, "pub mod root_mode_names;");
    assert_eq!(declarations, 1, "worktree_clone_mount_handler/mod.rs must declare `pub mod root_mode_names;` exactly once, and it declares it {declarations} time(s)");
    assert_eq!(run, "", "worktree_clone_mount_handler/mod.rs's declaration of root_mode_names must stand under no attribute, and it stands under {run:?}");
}

/// The folder itself is declared once, under the one attribute that makes it a Linux build's.
#[test]
fn the_handler_folder_is_declared_once_and_under_its_one_attribute() {
    let (declarations, run): (usize, String) = declaration_run(CRATE_ROOT_SOURCE, "pub mod worktree_clone_mount_handler;");
    assert_eq!(declarations, 1, "lib.rs must declare `pub mod worktree_clone_mount_handler;` exactly once, and it declares it {declarations} time(s)");
    assert_eq!(run, squeezed("#[cfg(target_os = \"linux\")]"), "lib.rs's declaration of the handler folder must stand under `#[cfg(target_os = \"linux\")]` alone, and it stands under {run:?}");
}

/// The words' first word is the program's own name,
/// taken from the one constant the crate keeps it in.
#[test]
fn the_words_first_word_is_the_programs_own_name() {
    assert_eq!(COMMAND_WORD.current, "monochromatic-git-policy-cli", "the words are `monochromatic-git-policy-cli <the group word> <mode>`, and root_mode_names.rs reads this constant for the first of them");
}

/// The installed ZED path's file name is the one start name the module compares,
/// so an install and the dispatch cannot drift apart.
/// The path is read as text, not imported, because the folder it lives in is a Linux build's and
/// this guard runs on every developer's machine.
#[test]
fn the_installed_zedlet_path_carries_the_one_start_name() {
    let installed: &str = INSTALL_PATHS_SOURCE.lines().find_map(|line| { return line.trim().strip_prefix("pub const ZEDLET_INSTALL_PATH: &str = \"") }).and_then(|rest| { return rest.split('"').next(); }).expect("ZEDLET_INSTALL_PATH's text in handler_install_paths.rs");
    let file_name: Option<&str> = Path::new(installed).file_name().and_then(|name| { return name.to_str(); });
    assert_eq!(file_name, Some("history_event-mount_git_worktree_clone_datasets"), "ZEDLET_INSTALL_PATH's file name must be ZEDLET_START_NAME, which root_mode_names.rs compares a start's last component against");
}
```

with three more constants beside `NAMES_SOURCE` and `HANDLER_MODULE_SOURCE`:

```rust
/// The crate root, which declares the folder the pinned module lives in.
const CRATE_ROOT_SOURCE: &str = include_str!("../lib.rs");

/// The paths module, read as text: its `ZEDLET_INSTALL_PATH` is one of the two spellings this
/// guard ties to the pinned start name.
const INSTALL_PATHS_SOURCE: &str = include_str!("../worktree_clone_mount_handler/handler_install_paths.rs");
```

The pin includes the module's `use` lines and both item headers,
which is what the read's `nm4` (a local `Path` shadowing the import) and `nm2` and `nm3`
(the constants' values swapped while the old literals stayed) needed and did not have.

The module's string literal set,
its function count and its `file_name()` count are gone:
they were a list's way of saying "the whole text is this",
and the whole text is now pinned.

The wiring the pins cannot see is four assertions,
and each one is shown failing on the read's own variants:
the module's declaration stands under no attribute,
the folder's declaration stands under `#[cfg(target_os = "linux")]` alone,
the words' first word is the crate's one name,
and the installed ZED path's file name is the pinned start name
(measured: run keep/guard-repairs-seventh.log,
where `wv1`'s `#[cfg_attr(all(), path = "elsewhere.rs")]`,
`wv2`'s `#[ path = "elsewhere.rs" ]`, `wv3`'s `# [path = "elsewhere.rs"]` and `wv5`'s `#[path]` on the
folder in `lib.rs` each fail on the run they produce,
and `wv4`, which declares a second module under another name, passes because it redirects nothing).

#### The typed half, and the configuration it rests on

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

Three allows that already stand in release code are narrowed from their function to the one
statement each excuses:
`worktree_clone_unmount/user_cache_pin.rs:162`,
`staging_claim.rs:116` and
`staged_folder_record.rs:139` each carry
`#[allow(clippy::disallowed_methods)]` on an `encode` method whose `serde_json::json!` expansion is
what the ban fires on.
The attribute moves to the `let object` statement the macro builds,
which is measured to excuse that statement and nothing else:
a read inserted at the top of the same function fails the typed ban,
and the narrowed form still compiles because the macro's expansion stays inside the allow
(run keep/statement-allow-scope-seventh.log).

What remains, and is stated rather than hidden:
a read placed inside the excused macro statement,
or inside any other pinned allow's own expression,
is excused by that allow and no text guard sees it.
The three narrowed statements are the whole of that surface in release code,
and each is a `let` whose value is one JSON object.

The ban's own configuration is what makes all of this a ban rather than a suggestion,
and until this revision nothing read it.
`source_guard/typed_ban_configuration_tests.rs` is new and pins it:

```rust
// package/git-policy/cli/src/native/source_guard/typed_ban_configuration_tests.rs
use super::squeezed;
use std::path::Path;

/// The crate's Clippy level table, squeezed, as the review fixed it.
/// `disallowed_methods` at `deny` is what makes the two `std::env::args` rows a ban;
/// at `allow` the same source compiles with no call refused and no pinned line changed.
const CLIPPY_LEVELS: &str = "complexity={level=\"allow\",priority=-1}disallowed_methods=\"deny\"implicit_return=\"deny\"needless_return=\"allow\"shadow_reuse=\"deny\"shadow_same=\"deny\"shadow_unrelated=\"deny\"";

/// Every disallowed call's path, in the configuration's own order.
const DISALLOWED_CALLS: [&str; 9] = [
    "std::result::Result::unwrap",
    "std::env::temp_dir",
    "tempfile::tempdir",
    "tempfile::tempfile",
    "tempfile::TempDir::new",
    "tempfile::NamedTempFile::new",
    "std::env::current_exe",
    "std::env::args",
    "std::env::args_os",
];

#[test]
fn the_typed_ban_is_configured_as_this_design_names_it() {
    let crate_root: &Path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest: String = std::fs::read_to_string(crate_root.join("Cargo.toml")).expect("the manifest");
    let mut levels: Vec<String> = Vec::new();
    let mut in_table: bool = false;
    for line in manifest.lines() {
        let trimmed: &str = line.trim();
        if trimmed.starts_with('[') {
            in_table = trimmed == "[lints.clippy]";
            continue;
        }
        if !in_table || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        levels.push(String::from(trimmed));
    }
    assert_eq!(squeezed(levels.join("\n").as_str()), CLIPPY_LEVELS, "the crate's Clippy level table is not the reviewed one, and a level changed here silences every pinned call without changing a line of source");
    let configuration: String = std::fs::read_to_string(crate_root.join("clippy.toml")).expect("the Clippy configuration");
    let calls: Vec<&str> = configuration.lines().filter_map(|line| { return line.trim().strip_prefix("{ path = \"") }).filter_map(|rest| { return rest.split('"').next(); }).collect();
    assert_eq!(calls, DISALLOWED_CALLS, "the disallowed calls are not the reviewed set, in the reviewed order");
}
```

What that catches, and what it costs:

- `disallowed_methods = "allow"` anywhere in the table fails, because the table is compared whole;
- a removed, added or retargeted call fails on the path list;
- a reason's prose is free, because only the paths are compared;
- an honest edit that adds a lint level or a disallowed call re-pins, which is the same cost the two
  file pins already charge and for the same reason:
  each of those lines decides what the build refuses, so it is a decision a reviewer reads.

(replay keep/clippy-bans.log):
a probe crate carrying those rows and the crate's own file refuses a call through a
`use std::env::args_os as ...` alias,
refuses `std::env::args()`,
and accepts the one statement that carries the allow.
The narrowing and the statement-scope behaviour are measured on this round
(run keep/statement-allow-scope-seventh.log).

#### Replayed

(run keep/guard-repairs-seventh.log),
from `keep/sources/cut_fences_seventh.py`,
which cuts every printed text out of this document by fence,
`assemble_probe_seventh.py`,
which builds the probe crate (its head lists the substitutions a binary crate forces),
and `guard_replay_seventh.py`,
which drives the verdicts and writes the log.
The first two verdicts are the document against itself:
the whole file and the whole module, both printed in "The rule, in the entry",
are read as code and compared with the two pins printed in
"The hook guard on `main.rs`" and "The one-name guard on the names module",
so the two printings cannot drift apart without this log saying so.
The log is kept whole in this round's `keep/`;
it prints the md5 of every text it replays and every variant's own verdict,
and it holds eighty-five verdicts, nineteen passes and sixty-six failures,
every pass and every failure intended.
The excerpt below shortens each path to its end,
and the log prints them whole.

```text
# the texts this log replays, cut out of the document by fence
#   main_rs.rs: md5 6d1190dee32095eee4ffd8d4fc32fe0c
#   names_rs.rs: md5 a5da89404c3cb83b3d29272d582fc7bb
#   text_module.rs: md5 42446b04314c75b2273c7ef11f48c3d7
#   hook_guard.rs: md5 1db03daaa86372461b7ab656345f52dd
#   names_guard.rs: md5 a99808b2bf3b284f2b34e03fad1c1351
#   lint_guard.rs: md5 bccb584a3fe12304e27b79819518154a
#   typed_ban_guard.rs: md5 bcc2abe65dd77eaf1ab7f77e7077cabd
#   bin_path_guard.rs: md5 aabf6004179b1eab9dfb322cfa58860c
#   scan_rules.rs: md5 3c861d2c32c0194a14112592d07c2d2b
#   scan_rule_type.rs: md5 855064f5470b0d43bccfff0000c20048
#   scan_wrapper.rs: md5 1dfecd6a852fe8980decf2a4ca2d71e3
#   scan_allowed.rs: md5 0810b025970640d7bef10684b23105b4
#   scan_witness.rs: md5 5ec8c117c84979f68777fad4e7836b53
== the document's own printed files, against the pins the document prints
ok   PASS scratch/printed/main_rs.rs
ok   PASS scratch/printed/names_rs.rs
== the positive controls: today's files must fail the pins and the settings that are not the design's
ok   FAIL package/git-policy/cli/src/native/main.rs: assertion `left == right` failed: main.rs has 27 code line(s), and the reviewed text has 47
== the wiring the pins cannot see
# the real mod.rs at this base: the module is not there yet, so the assertion fails until this design lands
ok   FAIL package/git-policy/cli/src/native/worktree_clone_mount_handler/mod.rs: worktree_clone_mount_handler/mod.rs must declare `pub mod root_mode_names;` exactly once, and it declares it 0 time(s)
ok   PASS scratch/sources/fixtures/handler_mod_declared.rs
ok   FAIL scratch/sources/fixtures/handler_mod_path_redirect.rs: worktree_clone_mount_handler/mod.rs's declaration of root_mode_names must stand under no attribute
ok   PASS package/git-policy/cli/src/native/lib.rs
ok   PASS COMMAND_WORD.current
ok   PASS package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_install_paths.rs
# the read's five wiring variants
ok   FAIL scratch/sources/read-new-variants/wv1_cfg_attr_path_redirect.rs: ... must stand under no attribute
ok   FAIL scratch/sources/read-new-variants/wv2_space_after_the_bracket.rs: ... must stand under no attribute
ok   FAIL scratch/sources/read-new-variants/wv3_space_after_the_hash.rs: ... must stand under no attribute
ok   PASS scratch/sources/read-new-variants/wv4_declaration_spelled_with_a_visibility_path.rs
ok   FAIL scratch/sources/read-new-variants/wv5_path_on_the_parent_module_in_lib.rs: lib.rs's declaration of the handler folder must stand under `#[cfg(target_os = "linux")]` alone
== the lint-level anchor pin, the whole release tree
ok   PASS lint pin, main.rs read as scratch/printed/main_rs.rs
# the same pin with today's main.rs, which holds no allow: the set is one anchor short
ok   FAIL lint pin, main.rs read as package/git-policy/cli/src/native/main.rs: release code holds a lint-level attribute this pin does not name
== the typed ban's own configuration
ok   PASS typed ban configuration, from scratch/sources/fixtures/designed-crate
ok   FAIL typed ban configuration, from package/git-policy/cli: the disallowed calls are not the reviewed set
ok   FAIL typed ban configuration, from scratch/sources/fixtures/ban-disabled: the crate's Clippy level table is not the reviewed one
ok   FAIL typed ban configuration, from scratch/sources/fixtures/row-removed: the disallowed calls are not the reviewed set
== the [[bin]] path assertion
ok   PASS release_bins_tests' path assertion
ok   FAIL release_bins_tests' path assertion: the wrapper's `[[bin]]` must compile src/native/main.rs, the one file the entry pin reads; a `path` that moves makes the pinned text guard a file the release does not build
ok   PASS release_bins_tests' path assertion
== the scan's rules over every fixture
ok   FAIL scratch/sources/variants/sc1_args_collected_with_its_own_allow.rs: reports [(3, "argv-vector-read", "letall:...
ok   FAIL scratch/sources/variants/sc2_alias_with_its_own_allow.rs: reports [(1, "argv-vector-read", "usestd::env::args_os...
ok   FAIL scratch/sources/variants/sc3_comment_closer_hides_the_read.rs: reports [(4, "argv0-dispatch", "//*/returnstd::...
ok   FAIL scratch/sources/variants/sc4_whitespace_split_read.rs: reports [(2, "argv0-dispatch", "returnstd::env::args_os()...
ok   FAIL scratch/sources/variants/sc5_proc_thread_self_comm.rs: reports [(2, "argv0-dispatch", "returnstd::fs::read_to_st...
ok   PASS scratch/sources/variants/sc6_proc_self_stat_as_release_code_does_today.rs: reports nothing
ok   FAIL scratch/sources/variants/sc7_module_alias_under_its_own_allow.rs: reports [(4, "argv0-dispatch", "returne::args...
ok   PASS scratch/sources/variants/sc8_release_control_args_of_a_command.rs: reports nothing
ok   PASS scratch/sources/variants/sc9_release_control_the_task_stats.rs: reports nothing
ok   PASS scratch/sources/variants/sc10_release_control_another_process_comm.rs: reports nothing
# the read's three stripper variants, whose allow and read the sixth revision's stripper hid
ok   FAIL scratch/sources/read-new-variants/sv1_br_raw_byte_string_hides_allow_and_read.rs: reports [(6, "argv0-dispatch", ...
ok   FAIL scratch/sources/read-new-variants/sv1c_control_same_text_without_the_br_prefix.rs: reports [(6, "argv0-dispatch", ...
ok   FAIL scratch/sources/read-new-variants/sv2_comment_then_r_in_a_macro_hides_allow_and_read.rs: reports [(6, "argv0-disp...
ok   FAIL scratch/sources/read-new-variants/sv3_br_hash_raw_string_with_inner_quote_cuts_the_line.rs: reports [(4, "argv0-d...
# the read's two allow variants, whose read the alias import spells
ok   FAIL scratch/sources/read-new-variants/lv1_allow_split_over_two_lines_and_alias_of_args.rs: reports [(1, "argv-vector...
ok   FAIL scratch/sources/read-new-variants/lv1c_control_allow_on_one_line_and_the_same_alias.rs: reports [(1, "argv-vector...
ok   FAIL scratch/sources/read-new-variants/lv2_allow_path_built_by_a_macro_from_an_ident.rs: reports [(1, "argv-vector-rea...
== the six rules over the real release tree
# reader: the raw lines, squeezed; release files: 1473; findings: 4
#   main.rs:34 argv-vector-read ///`args_os`/`vars_os`neverdecode,
#   main.rs:52 argv-vector-read letarguments:Vec<OsString>=std::env::args_os().skip(1).collect();
#   mount_git_worktree_clone_datasets/main.rs:40 argv-vector-read ///`args_os`neverdecodes,
#   mount_git_worktree_clone_datasets/main.rs:49 argv-vector-read letarguments:Vec<OsString>=std::env::args_os().skip(1).collect();
== each added needle fires on its own witness and on no release read
ok   PASS each rule fires on its own witness
```

What the excerpt says, line by line:

- the document's two printed files are the two pins' own text,
  so the two printings cannot drift;
- today's `main.rs` fails the entry pin on its line count,
  which is the positive control that the pin is not vacuous;
- the wiring assertions pass on the real `lib.rs` and the declaration fixture,
  fail on the real `mod.rs` (which does not declare the module yet),
  fail on the `#[path]` fixture and on `wv1`, `wv2`, `wv3` and `wv5`,
  and pass on `wv4`,
  which declares a second module rather than redirecting the pinned one;
- the lint-level pin passes with the design's `main.rs` and fails with today's,
  one anchor short;
- the typed-ban pin passes on a crate whose manifest and `clippy.toml` are the design's,
  and fails on the real crate (two rows not there yet), on the level changed to `allow`,
  and on one row removed;
- the `[[bin]]` assertion passes on the real manifest and on the one-bin fixture,
  and fails on the moved path;
- every scan fixture that must report does,
  every control that must stay silent does,
  and the read's three stripper variants and two allow variants all report now,
  which is the whole of A1's and A2's first two holes;
- the release tree holds four findings, which are the two collections and the two doc lines above
  them, and the one `ALLOWED` row declares `main.rs`'s;
- the witness test passes.

`SPEC.md:19873` becomes
(it was `:19715` at the sixth revision's base, `:19705` at the fifth's, `:19588` at `45ac28ae47` and
`:19261` at `f3d81e0910`):

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

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked the word every unit line,
> adoption command and person's root-mode command spells after `monochromatic-git-policy-cli`.
> Ranked `zfs-root` over `clone-mounts` over `mount` over `root`.
>
> The answer: "A: zfs-root (Recommended)".

So `ROOT_GROUP_WORD` is `"zfs-root"`,
one line of the pinned module,
and the shape of every command is the wrapper's own word,
then the group word,
then the mode's grammar,
which is the shape `service run` already has.
A unit's `ExecStart`,
an adoption's command and a person's command all read it:

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
```

```sh
sudo /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
  monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
  --checkout /home/user/Monochromatic --device 66306 --inode 1234567
```

(proposal for the lines as written,
since the file does not exist at that path on this host yet,
run keep/host-facts-seventh.log;
the decision the words make is measured,
replay keep/names-starts-fourth.log.)

### The text the modes print

The usage is the build's own text
(replay keep/handler-usage.log),
with choice 5's line for the program's name,
choice 8's word and choice 9's token where those answers put them:

```text
usage: <file> monochromatic-git-policy-cli zfs-root <mode>
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
                       `--replace`, given the GUID the kernel shows now, replaces a record whose GUID
                       a legitimate restore changed
  mount-path-descendant --dataset <name>
                       mount one nested dataset at its own mountpoint, under the recorded checkout
  --help               print this text
  --version            print this build's version
Every root mode is reached through the words `<file> monochromatic-git-policy-cli zfs-root <mode>`,
with `<file>` the root-owned file.
Plan step 19's setup installs `<file>` as /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
and links it as history_event-mount_git_worktree_clone_datasets for ZED; no release installs it yet.
The modes that make requests open /dev/zfs through this repository's binding, never a CDDL library and never the zfs program.
```

The first line names the root-owned file before the words,
so no line offers a spelling that works only through it and no line offers a mode in brackets:
the words follow whatever path starts the program,
so the words alone reach Git as an unknown command.
That is the alpha.46 release pass's answer A,
which also settles the closing sentences:
they say who installs the file and when,
because `--help` claimed install paths no release creates,
and the sweep's refusal to an administrator says the same
(`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`,
"How the root modes' usage reads").
The first line is choice 5's,
so no refusal and no usage names `mount-git-worktree-clone-datasets` any more;
the last three lines are replaced by the sentences above,
because the build is one file at one path and the words are what a person types, and the file is not installed yet.
The two `record-checkout` and `mount-path-descendant` blocks are this design's
(proposal);
everything else is the build's own text.
`--replace`'s value is choice 9's E,
which is one change to the sixth revision's printing of this block.

### Why the ranking ran this way

The record ranked `zfs-root` over `clone-mounts` over `mount` over `root`,
and the sixth revision ranked the same order,
with these pair reasons.

A over D:
both name the route,
and A names the privilege the words path exists for,
which is what a unit's `ExecStart` under root and an adoption under `sudo` are about.
D's noun covers the mounting modes and not the rest:
`start-probe`, `holder-scan`, the adoption and `record-checkout` mount nothing,
and the record the checkouts read is not a mount at all.
D's noun is also the one the record mode's reader cannot use,
so A covers the modes where D covers part of them.
The picks are a naming taste,
and this is the delivery that separates them (design).

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
while `root` collides with a Git word's meaning,
so `git monochromatic-git-policy-cli root …` invites a wrong guess about what it does.

### What the answer costs, and where it is written

The cost is the eight characters on every unit line and the pairing of the route's name with the
privilege's.
What it buys is one word for seven modes,
which is the property `service` already teaches.

If the word ever changes,
what changes is one line of `root_mode_names.rs` and the same line of its pin,
the unit texts in plan step 19 and in the four documents that carry unit lines,
the wrapper's management help line in `MANAGEMENT_USAGE` and `MANAGEMENT_HELP`,
the modes' usage's closing sentences,
every refusal that names the words,
the release smoke's second arm and `native:install`'s words check,
`SPEC.md`'s sentence about the words,
and the guest commands in `e2e/start-probe-guest.ts`.
Nothing about the one name,
the layouts or the dispatch's placement changes,
and the version line and the refusals move with it without a second copy of the word,
because both are built from `COMMAND_WORD.current` and `ROOT_GROUP_WORD`
(choice 5's section).

## Choice 3: what root's install holds

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked what `/usr/libexec/monochromatic-git-policy-cli/` holds,
> and so what a stray or mistyped start of it does as root.
> Ranked one file under the hook's name,
> over one file named `mount-git-worktree-clone-datasets`,
> over one file named for the wrapper,
> over a copy or link per name.
>
> The answer: "A: one file, hook's name (Recommended)".

> So the folder holds one root-owned regular file under the hook's name,
> ZED's entry `/etc/zfs/zed.d/<that name>` is a link to it,
> and no file there is named for the wrapper or the modes.

So the layout is one file and one link:

```text
/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ../../../usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
```

and what each start at that path does,
with today's wrapper bytes as the installed file for the "today" lines
(run keep/today-installed-name.log)
and choice 1's rule for the "after" lines
(replay keep/names-starts-fourth.log):

```text
$ .../history_event-mount_git_worktree_clone_datasets
    today:   git's usage, exit=1; a marker in the repository's `.git/cli-git-lock-holders/`
    after:   the attaching sweep
$ .../history_event-mount_git_worktree_clone_datasets boot-reconcile
    today:   git: 'boot-reconcile' is not a git command, exit=1; the same marker
    after:   a refusal with the usage, exit=2, no mode
$ .../history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
    today:   the management usage, exit=2; the same marker
    after:   boot-reconcile
$ .../history_event-mount_git_worktree_clone_datasets --version
    today:   git version ... and the wrapper's banner, exit=0; the same marker
    after:   monochromatic-git-policy-cli zfs-root <version>, exit=0
$ .../history_event-mount_git_worktree_clone_dataset --version
    today and after:   No such file or directory, exit=127
$ .../mount-git-worktree-clone-datasets boot-reconcile
    today and after:   No such file or directory, exit=127
```

The last two lines are the reason the layout is worth its odd name:
a typo,
a stale line or a copied command at either old spelling fails loudly and never runs Git as root.

### Why the ranking ran this way

A over B:
both keep one file,
but A's only root-owned file is a root-mode start whatever its words,
so a typo,
a stale line or a copied command at that path is a refusal or a mode and never Git as root,
while B's file is the wrapper for a bare word and for no argument at all,
and step 19's earlier line as the plan carries it is exactly a bare word at such a path.
The confusion A costs is a `history_event-` name in `/usr/libexec`,
a name a reader has to be told about once,
and the unit and adoption lines are generated from the constants and printed with their values
anyway.

B over C:
both fail open for a bare word at their own path,
and B's file at least names the modes it runs.
A stale line that names the wrapper's path fails with `No such file` under A and under B,
because no such file exists in either,
while under C that path exists and runs Git as root,
which is the review's finding exactly.

C over D:
C has one file and one upgrade step;
D multiplies both and adds a drift a person cannot see,
because a hard link made before an upgrade that writes a new file and renames it over the old stays
on the previous release,
so a zedlet or a unit would run old bytes without a word saying so.

What none of the four prevents,
and this design does not pretend otherwise:
a copy of the wrapper's bytes somewhere else,
started as root under a name of its own,
runs the whole wrapper,
and a start of any of these files under a caller-chosen `argv[0]` that is not the zedlet's
is the wrapper as that caller
(replay keep/dispatch-cases.log and keep/names-starts-fourth.log).
An account that can write root's install folder can replace the file,
which is why `doctor` checks its owner,
mode,
ancestor folders and the version it prints.

### What the answer costs, and what moves with it

The costs are three:
the installed path the modes' messages print is long,
a file named `history_event-…` in `/usr/libexec` reads as a zedlet copy,
and `SERVICE_INSTALL_PATH`,
whose name means "where a service call starts it",
is renamed to `ROOT_MODE_INSTALL_PATH`,
because nothing about it is a service call any more.
`/usr/libexec` does not exist on this host today
(run keep/host-facts-seventh.log),
so the step that installs the file creates its folder.

What moves with the answer, and what does not:

- the words still select every mode at that path,
  and the name adds ZED's no-argument sweep and the modes' `--help` and `--version`;
- `doctor`'s planned installed-file check (step 18) is one word-free command,
  `--version` under the installed name,
  and step 19's guest arm reads the same;
- the release smoke's first arm links the name and reads `--version` through it;
- the AppArmor rule in `/etc/apparmor.d/local/zed` names the file the zedlet resolves to,
  which under this answer is the root-owned file in `/usr/libexec`,
  so a later layout change is one line in two files.

## Choice 4: where the dispatch sits

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked where in the program's start the root-mode dispatch sits.
> Ranked the first statements of `main`,
> over after the panic hook and the environment,
> over the top of `run_process`,
> over inside `plan_invocation`.
>
> The answer: "A: first lines of main (Recommended)".

So the dispatch is the `#[cfg(target_os = "linux")]` block of "The rule, in the entry",
before the service check,
before the panic hook,
and before the environment that block itself does not need.
The three checks it holds run in one order,
and that order is load-bearing twice:

- the words are read before the one name,
  which is what lets a unit's line work at the installed path;
- both are read before the service's words,
  which is what refuses `monochromatic-git-policy-cli service run` under the zedlet's name.

Each check reads the argument vector and nothing else.
A root-mode start runs today's handler from there:
the grammar,
the compatibility gate,
the mode,
and the two log waits at exit.
A new mode's own reads are on top of that and are the mode's business,
not the dispatch's.

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

### What the answer's start does, and what the other placements cost

Option A's start,
which is the one this design builds:

```text
$ cd /home/user/Monochromatic
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   the mode's own line, exit=0 or 2
   .git/cli-git-lock-holders: no marker this start made
```

The second line is a prediction,
not a run:
the block returns before `entry::run_process` is called,
which is the only route to the lock pass,
and every start in `keep/today-installed-name.log` that reached `run_process` created the marker
(replay).

A over B:
B puts the block after the panic hook and the environment collection,
so `main` collects the whole environment for a start that reads none of it,
and its second gain is gone as well,
because the hook can be installed inside the arm,
which is what choice 10's answer does.
No probe shows any benefit over A.

B over C:
the two cost the same start-time work,
so this pair is a tie by delivery,
and the order is about where the code lives.
B keeps the decision in the executable's entry,
which is where the answer's precedent (`service run`) puts it and where the vector already is.
C moves it into `entry::run_process`,
whose one other caller cannot fill the parameter it would need:
`run_nested` (`merge/merge_second_pass_finish.rs:429`) calls `run_process` with the words of a Git
command and no program name,
so the new parameter would be "there is none" at that call site.
That is a cost, and it is why C ranks third rather than as B's equal.

C over D:
D adds the lock pass,
the resolution and the started-as-Git check before the mode,
and it is the only placement whose cost grows with the caller's working directory:
a root start there writes a marker into whatever repository holds the working directory,
which every start in `keep/today-installed-name.log` shows (replay).
`git_facts`, the configuration read and the forwarded Git are one branch away from it.

### What moves with the answer

The dispatch lives in the pinned `main.rs`,
so the pin and the block are one commit:
any placement other than A's is a changed pinned line,
which is why the choice was never free.
A's own cost is the one stated:
the entry now holds three mode checks in a fixed order before the wrapper's path.

## Choice 5: what `--version` and the refusals say

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked the line `<file> --version` prints and the name a root-mode refusal opens with.
> Ranked the typed words,
> over keeping `mount-git-worktree-clone-datasets`,
> over the wrapper's name.
>
> The answer: "C: the typed words (Recommended)".

> So `--version` prints `monochromatic-git-policy-cli zfs-root` and the version,
> every root-mode refusal opens with `monochromatic-git-policy-cli zfs-root:`,
> and the family name `mount-git-worktree-clone-datasets` is no longer printed;
> the line carries choice 2's word.

So the two printed things are these,
and this is the text the build produces:

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
monochromatic-git-policy-cli zfs-root 0.6.1-alpha.45
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root boot-reconcil
monochromatic-git-policy-cli zfs-root: `boot-reconcil` is not a mode. Nothing was done. The modes are
boot-reconcile, start-probe, shutdown-unmount, holder-scan, adopt-worktree-parent, record-checkout,
mount-path-descendant, and running the command with no mode word after `zfs-root` is refused rather
than run. Ways forward: run `monochromatic-git-policy-cli zfs-root --help` for the modes this build
accepts.
```

```text
$ /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets --version x
monochromatic-git-policy-cli zfs-root: `x` is an argument this start does not take: under this
file's own name only no argument, `--help` and `--version` are answered. Nothing was done. Ways
forward: run the file with no argument for the clone-mount sweep, with `--help` for the modes'
usage, or with `--version` alone; every mode is reached through the words, as
`monochromatic-git-policy-cli zfs-root <mode>`.
```

The version's number is whatever `env!("CARGO_PKG_VERSION")` holds,
which is `0.6.1-alpha.45` today
(replay keep/handler-usage.log,
where the built handler answers its own line with that version).

### How the two are built, so they cannot drift

`PROGRAM_NAME` (`handler_install_paths.rs:28`) goes with the family name.
The line and the refusal prefix are built from the two constants the guards already pin,
`COMMAND_WORD.current` (`foundation/wrapper_names_old_and_new.rs`)
and `ROOT_GROUP_WORD` (`root_mode_names.rs`):

```rust
// package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_install_paths.rs
/// The words every root-mode line and every root-mode refusal names:
/// the program's own word, the group word choice 2 fixes, and a colon when it opens a refusal.
/// Why the two constants and not a third spelling:
///  the guards already pin both, so a refusal cannot name words the dispatch does not accept.
pub fn root_modes_name() -> String {
    return format!("{} {}", COMMAND_WORD.current, ROOT_GROUP_WORD);
}
```

```rust
// package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_install_paths.rs
/// The line `--version` prints: the words and the release this build came from.
pub fn version_line() -> String {
    return format!("{} {}", root_modes_name(), env!("CARGO_PKG_VERSION"));
}
```

and every refusal that used to say the family name says
`format!("{}: {sentence}", root_modes_name())`,
which is where the colon comes from:
the prefix is the name,
and the sentence after it no longer repeats it.

### Why the ranking ran this way

C over A:
the two deliver the same three things a check needs.
Each is one line,
each is unique against the wrapper's banner
(`monochromatic-git-policy-cli 0.6.1-alpha.45, shadowing git at ...`),
because each continues with something the banner does not have,
and each lets `native:install`, the release smoke and `doctor` tell "this build answers the root
modes" from "this is the wrapper".
C delivers two things A cannot:

- an opening spelling a person can type.
  A's line names `mount-git-worktree-clone-datasets`,
  which after this change is no file and no command,
  so every refusal would have to name the family and also print the words and the path;
  C's line is the words,
  so the refusal's opening, the unit lines and the version line are one spelling;
- a refusals set that needs no second name:
  the opening words are the runnable spelling, and the path is the one thing left to print.

What A delivers that C does not is that its line does not move when choice 2's word is renamed,
and that four existing readers keep the string they compare today.
The second is upkeep and fidelity to what exists,
which the repository's ranking rule says to state as a cost and never to rank by:
"Score capability for the target workload, never fidelity to incumbents" (`AGENTS.md`).
What remains is the first, and it is a cost of a rename that choice 2 already decides once,
paid by one line in each of three checks that the change list rewrites under either option.
C's own cost is the longer opening in every refusal (two words and a space more than a
33-character name),
which the design already requires to carry the words and the path.

A over B:
A's first line is unique against the wrapper's banner,
and B's first line is a prefix of it,
so a check on the installed file's first line cannot tell which program answered,
and a second line would have to be read instead.
This is the pair the sixth read found missing from the sixth revision's ranking.

### What the answer costs, and where it is written

What moves with C:

- `handler_install_paths.rs`: `PROGRAM_NAME` goes and `root_modes_name` arrives,
  and `version_line` moves beside it from `handler_mode_arguments.rs`;
- `handler_mode_arguments.rs`: the four refusal messages that name the family (`:220`, `:223`,
  `:233`, `:245`), `usage()`'s first line, and the two new modes' lines;
- the dispatch's two refusal lines and every mode refusal,
  which gain the prefix through the one function;
- `handler_install_paths_tests.rs:22` and `:32`,
  whose assertions compare the installed paths against `PROGRAM_NAME`,
  now compare them against the constants they are made of;
- `handler_mode_arguments_tests.rs:128`,
  whose version assertion becomes `version_line()`'s own line;
- `bin/native-install-steps.ts:104` (`handlerProgram` becomes the words),
  `bin/native-install.ts:448` and `:458` (the check runs the words and compares
  `rootModesVersionLine`),
  `bin/stand-in-install-programs-for-tests.ts:169` to `:186` and `:335` (the stand-in answers the
  words and loses its handler),
  and `bin/native-install.unit.test.ts:417`;
- the release smoke's two arms,
  which print and compare the words' line;
- `doctor`'s planned check (step 18),
  which compares the words' line against the version `doctor` runs from;
- `SPEC.md`'s and the plan's sentences that name the family,
  and the four refusal texts that tell an administrator to run the old name
  (`health/doctor_zfs_route/identity_checks.rs:145` and `:164`,
  `worktree_clone_mount_handler/sweep_preconditions.rs:195`,
  `worktree_clone_mount/clone_slot_walk_refusals.rs:194`).

The choice is separable from the other ten in what it decides and coupled to choice 2 in one line:
the version line and the refusal prefix print choice 2's word,
which is why they are built from the constant rather than written out a second time.
Under any other answer to choice 2 that line and those prefixes change together,
and no other text does.

## Choice 6: whether the account's `PATH` keeps a short name

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked whether the account's `PATH` keeps a short name for root modes.
> Ranked none over a name the toolchain links.
>
> The answer: "A: none (Recommended)".

> So `doctor` prints the root-owned path and the words to type.

So a person types the file root's units run,
and nothing else is installed where `sudo` would find it:

```text
$ sudo /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
    --checkout /home/user/Monochromatic --device 66306 --inode 1234567
   decides: root-mode grammar="adopt-worktree-parent --checkout /home/user/Monochromatic --device 66306 --inode 1234567"
            (the compiled module under `git` with these words, run keep/names-starts-fourth.log;
            dispatch case 25 has the same verdict)
```

and a person's read-only question goes through the account's own wrapper:

```text
$ git monochromatic-git-policy-cli zfs-root --help
   decides: root-mode grammar="--help"
            (the compiled module under `git`, run keep/names-starts-fourth.log)
   prints:  the modes' usage of choice 2's section, exit=0 (proposal)
```

`doctor` prints the adoption line with the checkout's device and inode filled in,
beside the three unit lines the setup writes,
and it prints the root-owned path,
never `sudo git monochromatic-git-policy-cli …`,
whose `git` is the account's file.

Nothing named for the modes is on this host's `PATH` today:
`command -v mount-git-worktree-clone-datasets` prints nothing and exits 1,
while the control `command -v git` prints the wrapper's own path,
so removing the second program loses no name a person has
(run keep/host-facts-seventh.log).

### Why the ranking ran this way

A over B:
A keeps one file behind every name,
and the thing a person reads is the thing root runs,
so `--version` describes the bytes that will act.
B's saving is typing,
and it moves the thing a person runs away from the thing root runs,
while adding an account-writable file that a `sudo` that keeps `PATH` can execute as root:
that is one more path to an escalation this design does not need.
B would also need the toolchain's plan to express a second post-install link,
because `mpm-shadow`'s `postinstall.link` takes one source, one name and one directory for each
tool,
which is a cost to state and not the reason the ranking puts B second.

### What the answer costs, and where it is written

The cost is the long command,
and every refusal inside the modes that names a follow-up command is written to it,
which the change list already carries because the refusals change with choice 5.
No toolchain plan row,
no `native:install` step and no `PATH` entry moves with this answer,
which is why it is separable from the other ten.

## Choice 7: whether the release keeps the modes' test-only variables

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked whether the release keeps reading the start probe's test switch,
> its keep-descriptor switch,
> `MONOCHROMATIC_VERBOSE` and `MONOCHROMATIC_WARN`.
> Ranked keeping them over compiling them out.
>
> The answer: "A: keep them (Recommended)".

So the four variables stay in the release,
and the guest arms keep testing the bytes a release ships.

The modes' code reads five variables,
all compiled into the release and all inert unless a control sets them
(`src/native/start_probe/probe_test_switch.rs:29` and `:35`,
`src/native/start_probe/probe_deadline.rs:69`,
`src/native/foundation/diagnostic_log.rs:40` and `:67`).
Four of them can only skip, refuse or narrate;
the fifth is choice 11's,
because it can turn a missed deadline into a pass,
and this revision's answer compiles that one out.

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

So none of the four can turn a refusal into a pass
(design,
from those four code statements;
no arm of this round set one as root).

ZED cannot set any of them:
it builds the environment from the event and a fixed list,
and the event's own variables carry the `ZEVENT_` prefix
(replay keep/zed-shape-report.txt).
A unit could set them with `Environment=`,
and so could anyone who edits the unit,
which is root.

### What the answer keeps, in one command

A control starts the installed file with a switch set and reads the record it wrote:

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=disabled <installed file> monochromatic-git-policy-cli zfs-root start-probe
   today, as the account, the mode refuses before the switch is read:
   monochromatic-git-policy-cli zfs-root: the start-probe mode runs as root, and this process's effective user is 1000. Nothing was read, mounted
   or recorded, because the record it writes decides whether a boot's clones are admitted for attachment,
   and an account that could write it could report a boot that never passed. Ways forward: run the mode as
   root, from the boot unit or by hand.
   as root, the record's verdict is skipped with its skip reason `test-switch`.
```

(the refusing line is the built handler's own sentence with choice 5's prefix,
printed today;
the root arm is the control's and was not run here)

### Why the ranking ran this way

A over B:
keeping them preserves the one property that makes the guest arms evidence,
that the binary under test is the binary released.
B's gain is a release binary without test switches,
and its cost is a build shape whose arms test something else,
which is the same cost choice 11's answer pays for the one variable that can pass a probe.
No probe here measures either direction,
and the design says so rather than claiming a measurement it did not make:
what it shows is that ZED cannot set one of them,
which is neither direction's evidence
(replay keep/zed-shape-report.txt).

### What the answer costs, and where it is written

The cost is a privileged program with four switches nobody but a control needs,
each of which at worst skips a record, refuses later or writes a line.
Nothing in the change list moves for this answer:
no file, no lint pass and no build shape.

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
  when `--replace`'s value is not the GUID the kernel shows now,
  naming both numbers and what to do;
- `nothing-to-replace`,
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
naming the field, printing both values and naming the exact command to run,
which is choice 9's answer:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
monochromatic-git-policy-cli zfs-root: checkout-record-differs: the record's GUID is 1234567890123456789 and the kernel
shows 9876543210987654321. Nothing was written. Ways forward: run this command again with
`--replace 9876543210987654321` to record this dataset, or restore the dataset the record names, which answers the
recorded GUID again.
```

With that command it prints every field that changes,
the old value and the new,
and writes:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic --replace 9876543210987654321
replaced: /home/user/Monochromatic keeps its path, dataset, container and pool; its GUID was
1234567890123456789 and is now 9876543210987654321. The record is <the record's path>.
```

A value that is not the kernel's GUID refuses rather than writing:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic --replace 1
monochromatic-git-policy-cli zfs-root: replace-token-differs: the kernel shows 9876543210987654321, and `--replace`
was given 1. Nothing was written. Ways forward: read the kernel's GUID from this refusal and pass it whole, or run
the command without `--replace` to read the difference again.
```

(proposal:
the mode does not exist,
and the numbers are the shape this host's GUIDs take,
18 to 20 digits,
run keep/host-facts-seventh.log)

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
(run keep/host-facts-seventh.log).

What it does,
in this order,
so that every act acts on an object it has proved rather than on a path it has read:

1.  reads the checkout's record at its one path,
    and refuses `no-recorded-checkout` when there is none;
2.  proves the record against the kernel's dataset facts, by name:
    the recorded dataset exists,
    its GUID is the recorded GUID,
    its `mountpoint` property is the recorded path,
    and the container is a dataset in the same pool that is still beside the checkout,
    or refuses `checkout-record-no-longer-matches` naming the field that differs.
    This step reads no mount table and needs no mount,
    which is what leaves step 3's refusal reachable when the checkout's own mount is delayed;
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
    `setuid`,
    `nbmand`,
    `inconsistent` and
    `receive_resume_token`,
    and the pool's own `readonly`,
    which is eighteen in all
    (the seven between `atime` and `nbmand` are the pairs upstream's mount derives its options from,
    upstream lib/libzfs/libzfs_mount.c, zfs-2.4.4, `zfs_add_options` at `:355` to `:381`;
    the last four are the refusals `share_mount_one` makes for an explicit mount and the one
    behaviour libzfs adds for a read-only pool,
    upstream cmd/zfs/zfs_main.c at `:7313` and `:7327` and lib/libzfs/libzfs_mount.c at `:428`);
5.  answers before it writes when the kernel already has the dataset mounted,
    which is what makes an `ExecStart=` re-run harmless:
    mounted at its own mountpoint, the `already mounted:` line and status 0 with nothing written;
    mounted at any other path, `dataset-already-mounted-elsewhere`.
    This step stands before the walk because a walk to an occupied last component would place a
    second mount on the dataset's own mountpoint (design);
6.  walks the mountpoint's components below the checkout's opened mount,
    one descriptor at a time,
    with the crate's own walk:
    `open_chain_beneath` refuses a link, a magic link, a hop on another device and a mount
    anywhere but the last component,
    and `open_directory_at_a_mount` allows that last one
    (source package/git-policy/cli/src/native/subvolume/fs_beneath_walk_chain.rs:140 and `:187`;
    the clone walk documents the same discipline at
    `worktree_clone_mount/clone_slot_walk.rs:30`).
    A missing last component is created `0755` and reopened with `O_NOFOLLOW`;
    a missing intermediate component refuses `mountpoint-unusable`,
    because this mode creates the mountpoint and never a tree of its own above it (design);
7.  mounts with the primitive the route already uses for its clones,
    not with a started `zfs` program and not through libzfs:
    `fsopen("zfs")`,
    `fsconfig` naming the dataset as `source`,
    `fsconfig` creating the superblock,
    `fsmount` with the attributes the pairs above give,
    and `move_mount` onto the opened folder's descriptor with
    `MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH`,
    which is the call `attach_the_clone` makes for a clone
    (source package/git-policy/cli/src/native/worktree_clone_mount/attach_the_clone.rs:341 to `:359`);
8.  answers the `mounted:` line and status 0.

What the mount's attributes are,
on this host's dataset:

- upstream's mount derives one option from each of the seven pairs and skips the `relatime` pair
  when `atime` is off (`:365` to `:369`),
  which this dataset's `atime=off` and `relatime=off` make its case;
- so the resolved set is `noatime,dev,exec,rw,suid,nonbmand`,
  which is the properties this round's log measures read through upstream's order above
  (run keep/host-facts-seventh.log),
  and `dev`, `exec`, `rw` and `suid` are the kernel's defaults,
  so the one attribute this mode passes to `fsmount` is `MOUNT_ATTR_NOATIME`;
- `nbmand` has no kernel attribute on Linux,
  so the pair is read and mounted as the no-op it is,
  and the mode says so rather than passing an option nothing reads;
- a pool whose `readonly` is `on` gets `MOUNT_ATTR_RDONLY` beside `MOUNT_ATTR_NOATIME`,
  which is libzfs's own rule ("If the pool is imported read-only then all mounts must be read-only",
  upstream lib/libzfs/libzfs_mount.c:428),
  and a dataset whose `readonly` property is `on` resolves to the same attribute through its pair.

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
  It costs one comparison;
- `dataset-without-a-mountpoint`,
  when the property is `none`,
  `-` or `legacy`,
  because such a dataset is mounted by someone else's rule and this mode would be guessing;
- `dataset-already-mounted-elsewhere`,
  when the kernel shows the dataset mounted at a path other than the one its property names;
- `dataset-not-mountable`,
  naming the property and its value,
  when `canmount` is `off`,
  when `zoned` is `on` in the global zone,
  when `redacted` is not `0`,
  or when `inconsistent` is `1` and `receive_resume_token` holds a token.
  These are the answers an explicit `zfs mount` refuses,
  each with an error and status 1
  (`share_mount_one` in cmd/zfs/zfs_main.c, zfs-2.4.4: `:7196` for a dataset exported to a local
  zone, `:7275` for `canmount=off`, `:7313` and `:7315` for a partially-completed receive, and
  `:7327` and `:7333` for a redacted dataset),
  while libzfs's own mount returns success and mounts nothing for the first three
  (lib/libzfs/libzfs_mount.c `:270`, `:299` and `:305`).
  The unit runs an explicit mount,
  so the refusal keeps the behaviour the unit has today and gives it a name:
  the failure it replaces is a unit line that reports success and leaves the dataset unmounted,
  which is what the tenth revision's `zfs mount` call would have done had it been libzfs's;
  the resume-token case also names the way forward, `zfs send -t` with the token's own stream;
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
  (run keep/host-facts-seventh.log),
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
after the migration and the first record,
in the order this revision states it:
the dataset is not mounted when the unit starts (`canmount=noauto`);
the record's dataset, GUID, path and container agree with the kernel by name, so step 2 passes;
the mount table shows the checkout at its recorded path, so step 3 opens it and passes;
the dataset's eighteen properties read as this round's log records them,
its key is available and the pool is writable, so step 4 passes;
the kernel does not show the dataset mounted, so step 5 answers nothing and writes nothing;
the mountpoint's hops are the account's own plain folders, so step 6 opens the folder;
and the mode mounts it with `noatime` and the five default pairs.
No refusal fires, and the ownership clause the fifth revision printed would have refused it and left
it unmounted.

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

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked the word of the mode that records which dataset is the checkout
> (the root command of choice 1,
> option E,
> in `doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`).
> Ranked `record-checkout` over `record-route` over `checkout-record`.
>
> The answer: "A: record-checkout (Recommended)".

So the mode is `record-checkout`,
one member of `MODE_WORDS`,
one line of the usage and one of the help,
and the command is:

```text
$ git monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
   decides: root-mode grammar="record-checkout --checkout /home/user/Monochromatic"
            (measured for `record-checkout` only: run keep/names-starts-fourth.log,
            which starts the compiled module under `git` with these words)
   prints:  the `recorded:` line of "The two modes these answers add" (proposal)
```

### Why the ranking ran this way

A over B:
A names the thing that is recorded,
which is what the add,
the sweep and `mount-path-descendant` read,
and B names the route,
which the group word `zfs-root` already says.
What A delivers is a mode whose word a person can match to the record file's name (design).

B over C:
both are two words about the route or the record,
and B reads as a verb and an object,
while C reads as a noun phrase,
which the grammar's other words never are (`boot-reconcile`, `start-probe`, `holder-scan`,
`adopt-worktree-parent`).
The difference C costs is the habit a reader already has (design).

### What the answer costs, and where it is written

The word is one member of `MODE_WORDS`,
one line of the usage and one of the help,
one match arm and its refusal texts,
the step 10 build's reader,
and `doctor`'s printed line.
Nothing about the record's shape or the other modes changes,
which is why this choice is separable from the other ten.

## Choice 9: the record a legitimate restore replaces

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked what root types to replace the recorded checkout after a legitimate restore gives the
> checkout dataset a new GUID,
> while the nested dataset's boot unit refuses and leaves it unmounted.
> Ranked `--replace <the kernel's GUID>`,
> over `--replace <the record's GUID>`,
> over a bare `--replace`,
> over the ordinary command replacing,
> over deleting the record and recording again.
>
> The answer: "E: --replace <kernel's GUID> (Recommended)".

> So `record-checkout --checkout <path>` refuses `checkout-record-differs` while the record differs,
> printing both GUIDs and the exact `--replace <the kernel's GUID>` to run;
> `--replace <that GUID>` writes the record and prints the old and the new GUID;
> a value that is not the kernel's GUID is refused `replace-token-differs`.

So the mode's usage line prints that token,
the refusal prints the value,
and the three texts are the ones "The two modes these answers add" prints.
This section keeps the reasons,
because they are the whole of what E buys over the four other shapes.

### The problem the answer is for

The step 10 design's own reason for the record is that a dataset re-created under the recorded name
is refused at the GUID comparison,
and the same sentence admits its price:
a legitimate restore gives the checkout dataset a new GUID,
"and E refuses the add until root re-adopts the checkout with the command"
(source doc/planning/cli-git-zfs-step-10-open-choices.md:578 to :579).
After a restore the record names a GUID the kernel no longer shows,
so `mount-path-descendant` refuses `checkout-record-no-longer-matches` at the next boot,
and the nested dataset's unit leaves `zpcachyos/nosnap/vm-builder-output` unmounted until root
replaces the record.
A unit that refuses has no fallback,
and the step 19 record's V6 says a unit that `Requires=` the nested one fails with it.
So the answer decides what an operator types and what the ledger of that act is.

### The window each shape leaves, and the rule behind it

The design's refusals name what was not done and the way forward.
That does not make every repair printable at boot,
and this answer fixes the rule:

- a refusal names a repair only when the repair carries a value the refusal itself prints;
- a repair that has no such value is not named,
  because naming it would teach the act that admits a dataset unseen.

Under E the boot refusal prints the kernel's GUID,
so `--replace <that GUID>` is a repair it may name,
and the window lasts one command before the reboot.
D's token is the record's GUID,
which that refusal also prints,
so D's window is the same.
A's flag carries no value and C's repair is a re-run of the ordinary command,
so neither may be named,
and the window lasts until someone remembers the runbook.
B's repair is a deletion no refusal can name,
so its window is the same and longer still.

That is why the window is a reason in this ranking and not a property of an option's mechanism:
it follows from whether the option gives the refusal a value to print.

### Why the ranking ran this way

E over D:
both make the admission a distinct, printed act with one value to copy,
and both leave the journal line with the old value beside the new.
E delivers the record the command named:
the typed value is the one written,
so a dataset that moved again between the refusal and the command refuses rather than being written
unseen,
and the journal's `is now` value is the operator's own token.
D delivers the record that was read:
a record another root act replaced in the meantime refuses.
Which of the two the ranking puts first follows from what the act is for:
the act admits a dataset,
and the mode's own proof of freshness is a read of the kernel,
so the value the operator's command should fix is the kernel's.
D's protection covers a race between two root acts on one host,
which is rarer than a second restore of the same checkout,
and either race already ends in a correct record when it does not refuse.

D over A:
both name one value to copy and both print the old GUID,
and D's token proves the operator read this record,
while A's bare flag cannot be told from a remembered habit,
and A pays for its shortness with a second kind of option in a grammar whose every option takes a
value.

A over C:
both print the old GUID and both compare.
A keeps `checkout-record-differs` meaningful for every field,
while C turns the ordinary command into the admitting one,
so a re-run for another reason admits a dataset nobody looked at.

C over B:
C compares and prints the old value;
B writes the new record with nothing to compare it against,
and makes the operator's step an unlogged deletion of a root-owned file.

### What the answer costs

- the operator copies an 18 to 20 digit number at the worst moment,
  which is when a person is least willing to;
- `replace-token-differs` and `nothing-to-replace` are two more refusals, one more usage line and one
  more option in the parser;
- a record another root act replaced in the meantime answers `already recorded` rather than
  refusing,
  because the record now agrees with the kernel;
  that is the one case where D refuses and E does not;
- the refusal prints the value,
  so a paste defeats the check,
  which is the price of printing the way forward at all.

What it buys is the property the design's GUID check exists for:
the record this command writes is the dataset this command named.

### What moves with the answer

- `REPLACE_OPTION` beside the grammar's five existing option constants
  (`handler_mode_arguments.rs:50` to `:62`),
  parsed as a `--name <value>` pair like every other option;
- the `record-checkout` usage line,
  which choice 2's section prints;
- the mode's refusals:
  `checkout-record-differs`, `replace-token-differs` and `nothing-to-replace`,
  all printed in "The two modes these answers add";
- `mount-path-descendant`'s `checkout-record-no-longer-matches`,
  which prints both GUIDs so the token the repair needs is at the boot that failed;
- the step 19 setup's line for a restore,
  which the runbook states as `--replace <the kernel's GUID>`,
  while a first record never needs the flag;
- a `--replace` whose value leaves every field equal,
  which answers 0 with the `already recorded` line and writes nothing,
  because a replace that changes nothing is the ordinary command's no-op.

The step 10 build's reader does not depend on any of this:
it reads whatever record stands.

## Choice 10: the panic hook a root mode installs

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked what a root-mode start prints if the program panics,
> when a panic message can hold a dataset name or a checkout path and lands in root's journal.
> Ranked the wrapper's payload-free hook over Rust's default hook.
>
> The answer: "A: payload-free hook (Recommended)".

So a root-mode start installs the wrapper's existing hook as its arm's first statement,
and what a panic prints is the code's own line:

```text
$ sudo <file> monochromatic-git-policy-cli zfs-root mount-path-descendant --dataset <a dataset whose read panics>
monochromatic-git-policy-cli: internal error at <file>:<line>:<col>; its message is not shown because it may
contain repository content. Hint: if you were running a Git command, run it again; your files are not changed
by this notice, so `git status` shows where things stand. If it repeats, report this location.
```

(the code's own line, a pure function of the panic's location,
source package/git-policy/cli/src/native/invocation/panic_notice.rs:68 to :86;
not run)

### Why the ranking ran this way

A over B:
both print a location,
and A's line is the same length and shape;
what separates them is what else is printed.
A never prints a payload that can hold a dataset name, a checkout path or a request's fields,
while B always does,
into root's journal and any log a unit's output reaches.
B's gain is a panic's reason in the journal,
which a root mode's refusals already give for every failure it anticipates.

### What the answer costs, and what it does not decide

The cost is the hint's last sentence:
it speaks of a Git command and of `git status`,
which reads oddly from a unit,
and this design accepts that rather than keeping a second hook text true for one caller.

Installing the hook before `main`'s own work would be choice 4's placement instead,
and this choice is about what the arm installs.
The arm's two `install_payload_free_panic_hook();` lines are pinned lines of `MAIN_CODE`,
so the other answer would have deleted both and re-pinned the entry,
which is the one way this choice touches a file.

## Choice 11: whether the release honours the probe's deadline variable

### The decision

The record's answer
(record doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md):

> Asked whether the release honours `CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS`,
> which today makes any positive number of milliseconds the start probe's bound.
> Ranked compiling it out,
> over keeping it and recording a raised bound in the verdict,
> over keeping it as it is.
>
> The answer: "A: compile it out (Recommended)".

> So the release uses the shipped bound whatever the variable says,
> and the guest arms that raise it build the probe with a Cargo feature.

So the release's probe path is the shipped bound and nothing else:

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=hold-move CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=60000 <installed file> monochromatic-git-policy-cli zfs-root start-probe
   the probe uses the shipped 350 ms bound; nothing read the variable, so a held step still records
   as timed out and the mode still fails closed (design)
```

and the one control whose point is a step that finishes after the shipped bound builds the probe
with the variable compiled in,
which is the feature the answer names:

```text
$ cargo build --release --locked --features start-probe-raised-deadline
$ CLI_GIT_TEST_ONLY_START_PROBE=hold-move CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=60000 <that build> monochromatic-git-policy-cli zfs-root start-probe
   the record says the held step passed, which is what the control exists to show (design)
```

The words in the second command are choice 2's,
and they are what the sixth revision's control was missing:
a bare `start-probe` is refused under the zedlet's name and is Git under any other,
so the control never reached the probe (replay the fifth read's
`keep/choices-2-6-8-9-11-decisions-measured-on-the-printed-module.log`).

### What the answer changes in the code

`probe_deadline.rs:119` to `:127` reads the variable,
and its only non-test caller is `start_probe/probe_mode.rs:117`.
The read moves behind one Cargo feature:

```rust
// package/git-policy/cli/src/native/start_probe/probe_deadline.rs
/// The bound this build uses: the shipped one, or a control's, when this build was made for a
/// control.
/// Why a feature and not a runtime switch:
///  a variable that can raise the bound turns a missed deadline into a pass, and a unit's
///  `Environment=` line can set one; a build made for a control says so in its own manifest and
///  cannot be the build a release ships (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`).
fn the_bound() -> Duration {
    #[cfg(feature = "start-probe-raised-deadline")]
    if let Some(raised) = raised_by_the_test_variable() {
        return raised;
    }
    return SHIPPED_BOUND;
}
```

with `raised_by_the_test_variable` holding what the file holds today under
`#[cfg(feature = "start-probe-raised-deadline")]`,
`[features] start-probe-raised-deadline = []` in the crate's manifest,
and the guest arm that raises the bound building with `--features`.

### Why the ranking ran this way

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
C leaves a passed record that nobody can tell from a real one,
and a stale `Environment=` line silently turns a missed probe into a pass.

### What the answer costs, and where it is written

The cost is one more build shape,
so the control that raises the bound tests that build rather than the released bytes,
which is the weaker statement this answer accepts to remove the stronger risk.
It reaches choice 7's territory and nothing else:
the four variables choice 7 keeps are the ones that cannot pass a probe,
and this is the fifth, which could.

What moves with the answer:

- `start_probe/probe_deadline.rs`,
  its tests and the `SHIPPED_BOUND` constant's doc;
- `package/git-policy/cli/Cargo.toml`'s `[features]` table;
- `e2e/start-probe-guest.ts:196`'s build line and its raised-deadline arm,
  which names the feature build;
- the guest arm's sentence in the e2e README;
- `mise.toml`'s task for that arm,
  if it builds its own binary rather than reusing the release's.

Nothing about the dispatch, the layouts, the record or the two new modes changes.

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
- the installed path spellings of the ZED hook and the configuration;
- the two log waits at exit.

The modes' program name is not kept.
Choice 5's answer drops it:
`mount-git-worktree-clone-datasets` is printed nowhere after this change,
the version line and every refusal's opening are choice 5's typed words,
and `PROGRAM_NAME` goes with the family name.

Changed:

- `--version` and `--help` are reached through the one name or the words,
  and the line is choice 5's,
  which `doctor`'s planned check (step 18) and the release smoke read,
  and `command -v mount-git-worktree-clone-datasets` is already empty on this host
  (run keep/host-facts-seventh.log);
- `native:install`'s handler check becomes a words check,
  because no family name reaches the account's `PATH`;
- every message that tells a person to run `mount-git-worktree-clone-datasets ...`
  names the words and the installed path instead,
  and every root-mode refusal opens with `monochromatic-git-policy-cli zfs-root:`,
  built from the two pinned constants rather than written out;
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
and each line number is given at `2b81674c10`,
this round's base,
unless it says another commit,
which is one the line has not moved since.
`rg --hidden -o "mount-git-worktree-clone-datasets"` finds the name in the files this list names,
of which the crate's `doc/` records and one `perf/` measurement are history and stay as they are,
and this proposal's own mentions are not a site to edit
(run keep/occurrences-seventh.txt).
A run without `--hidden` misses `.github/workflows/cargo-publish.yml`,
which is why every count here is given with `--hidden`.

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
  `replace-token-differs` and
  `nothing-to-replace`;
- `src/native/worktree_clone_mount_handler/mount_path_descendant_mode.rs` (new):
  the mount mode's eight steps of "What it does",
  its read of the record and of the dataset's eighteen properties and the pool's own,
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
  and its reader is `pinned_code_lines`
  (`names_the_argument_vector` does not exist in the tree today,
  and nothing here needs it);
- `src/native/source_guard/release_bins_tests.rs`
  and `bin/native-release-bins.unit.test.ts`:
  the `path` reader and its assertion,
  as "The `[[bin]]` `path` the release builds" gives it,
  so the manifest's `path` is a guard and not only its `name`;
- `src/native/source_guard/mod.rs`:
  `pub(super) fn squeezed`,
  `pub(super) fn pinned_code_lines`,
  `pub(super) fn assert_whole_line_comments_only(what, source)`,
  `pub(super) fn release_sources_in`,
  and the declarations of `root_mode_name_tests`,
  `lint_attribute_tests` and
  `typed_ban_configuration_tests`;
  the sixth revision's `Reading` state and its `code_lines` do not arrive at all,
  and the crate's own masker stays where it is with its two users;
- `src/native/source_guard/root_mode_name_tests.rs` (new):
  `NAMES_CODE`, the pin of the module,
  its test,
  and the four wiring assertions
  (the module's declaration and its attribute run,
  the folder's declaration in `lib.rs` and its attribute run,
  `COMMAND_WORD.current`,
  and `ZEDLET_INSTALL_PATH`'s file name read as text);
- `src/native/source_guard/lint_attribute_tests.rs` (new):
  the thirty-five pinned lint-level anchors and the test that holds the release tree to them;
- `src/native/source_guard/typed_ban_configuration_tests.rs` (new):
  the pinned Clippy level table and the nine disallowed call paths;
- `src/native/source_guard/no_self_invocation_rules_tests.rs`:
  the three new `argv0-dispatch` needles,
  the `argv-vector-read` rule with its `env::args` needle,
  its one `ALLOWED` row,
  both rules' `squeezed_line` variable,
  which no longer shadows its own parameter,
  and the `Rule.matches` signature, which takes one text and every rule of the file follows it;
- `src/native/source_guard/no_self_invocation_tests.rs`:
  its `code_lines` becomes the seven-line reader that squeezes each line and cuts nothing,
  a seventh `samples` pair (`:191`),
  which `:208`'s length assertion carries,
  the witness test's one-argument calls,
  and the test that fires each added needle on its own witness and on no release read;
- `package/git-policy/cli/clippy.toml`:
  the two `std::env::args` rows;
- `src/native/worktree_clone_unmount/user_cache_pin.rs:162`,
  `staging_claim.rs:116` and
  `staged_folder_record.rs:139`:
  each function-wide `#[allow(clippy::disallowed_methods)]` moves to the `let object` statement its
  `serde_json::json!` builds,
  so the allow excuses that statement and nothing a later edit inserts above or below it;
- `package/git-policy/cli/Cargo.toml`'s `[features]` table:
  `start-probe-raised-deadline`, which is choice 11's one build shape;
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
  The starts that pass no argument keep their shape:
  the sweep at `binary_zfs_sweep_guest_tests.rs:230` and `:875`,
  the unmount start at `binary_zfs_unmount_guest_tests.rs:238`,
  and the start at `binary_zfs_step9_m_arms_tests.rs:200` with the two threads at `:561`,
  which is three sites and not four;
  `binary_mount_git_worktree_clone_datasets_tests.rs:106` is the `ldd` list's entry
  (`RELEASE_BINS`), not a start,
  so that file only loses an array entry;
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
  which builds the probe with `--features start-probe-raised-deadline` and names that build,
  which is choice 11's answer;
  `probe_deadline.rs`'s variable read moves behind that feature,
  `Cargo.toml` gains the `[features]` row,
  and the arm's sentence in the e2e README names the feature build.

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
  prints both GUIDs, and names the exact `--replace <the kernel's guid>` to run,
  so the repair is one command typed from a value the failed boot printed;
  until it runs the nested dataset stays unmounted,
  because a unit that refuses has no fallback and a unit that `Requires=` it fails with it
  (the step 19 record's V6);
- the migration's tested rollback
  (source doc/planning/cli-git-zfs-worktree-implementation-plan.md:2598,
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
  which under choice 3's answer is the root-owned file in `/usr/libexec`;
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

What this round measured itself:

- the two printed files against the two pins printed here,
  today's `main.rs` as the positive control that must fail,
  thirty-nine `main` and names variants from the third, fourth and fifth reads,
  the plain controls that must pass,
  the wiring fixtures and the read's five wiring variants,
  the scan's ten fixtures and the read's seven new ones,
  the lint-level anchor pin over the whole tree with the design's `main.rs` and with today's,
  the typed-ban configuration pin on the design's settings and on three broken ones,
  the `[[bin]]` path assertion on the real manifest and two fixtures,
  and the release tree's own findings:
  eighty-five verdicts, nineteen passes and sixty-six failures, every one intended
  (run keep/guard-repairs-seventh.log);
- every printed Rust text compiled and linted under the crate's own `[lints.clippy]` table and its
  `clippy.toml` with this design's two rows:
  the printed `main.rs` and `root_mode_names.rs` in a stub crate that provides the symbols they call,
  and the three printed fragments of `handler_install_paths.rs` and `probe_deadline.rs` with both
  arms of the one Cargo feature they turn on
  (run keep/clippy-printed-main-and-module-seventh.log),
  and every guard text in the probe crate that runs them, with no diagnostic at all
  (run keep/clippy-printed-guards-seventh.log);
- a statement-level `#[allow(clippy::disallowed_methods)]` excuses that statement and nothing above
  it, so a read inserted at the top of the function fails the typed ban,
  and the same statement allow keeps a `serde_json::json!` expansion compiling
  (run keep/statement-allow-scope-seventh.log);
- the host's own table, re-read for this revision:
  the fourteen datasets,
  the nested dataset's eighteen mount-relevant properties,
  its `redacted` of `0` and its empty `receive_resume_token`,
  the pool's `readonly` of `off`,
  the ownership and mode of its mountpoint and of every parent,
  the GUID digit counts (one of 18, six of 19, seven of 20),
  and the two paths and one name that do not exist here
  (run keep/host-facts-seventh.log);
- the pin's cost in commits, and the release tree's count of files the scan reads
  (run keep/pin-cost-seventh.log and keep/guard-repairs-seventh.log).

What earlier rounds measured,
replayed here and cited where it is used:

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
  and the real release tree
  (run keep/guard-repairs-seventh.log, which replaces the sixth round's run of the same shape);
- the fifth revision's printed stripper and the sixth revision's,
  over the 1,472 release files of that round's base:
  the blind regions,
  the lost code and comment lines,
  the 63 files that hold a `/*`
  and the eight lines with a `//` inside a string,
  which is why this revision stopped reading through a lexer at all
  (replay keep/stripper-over-the-release-tree.log);
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
  (run keep/host-facts-seventh.log);
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
  so the release tree's four findings are the probe's count over the crate's own classifier,
  and the two doc lines it reports are the ones the `ALLOWED` row would declare;
- the crate's own build, tests and Clippy:
  this round changes no code, and the two printed files and the guard texts are compiled and linted
  in a stub crate and a probe crate instead,
  so nothing here shows that the crate compiles with the change;
  `mise run //package/git-policy/cli:lint:types` was not run for the same reason;
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

## The eleven decisions, and where each is written

None of these is open.
Each line gives the answer, the section that states it, and the text a build edits first.

1.  Choice 1, what selects a root mode:
    the words select a mode on any file,
    and under the installed name ZED's no argument, `--help` and `--version` are answered while every
    other word is refused with status 2.
    Written in "Choice 1: what selects a root mode",
    in `root_mode_names.rs` whole,
    and pinned by `root_mode_name_tests.rs`.
2.  Choice 2, the group word:
    `zfs-root`.
    Written in "Choice 2: the group word units, adoption and a person spell",
    and in `ROOT_GROUP_WORD`, one pinned line.
3.  Choice 3, what root's install folder holds:
    one root-owned regular file under the ZED hook's name,
    with `/etc/zfs/zed.d/<that name>` a link to it.
    Written in "Choice 3: what root's install holds",
    and in `ROOT_MODE_INSTALL_PATH` and `ZEDLET_INSTALL_PATH`.
4.  Choice 4, where the dispatch sits:
    the first statements of `main`, before the service check and the environment.
    Written in "Choice 4: where the dispatch sits",
    and in the pinned `main.rs`.
5.  Choice 5, what `--version` and the refusals say:
    `monochromatic-git-policy-cli zfs-root <version>`,
    and `monochromatic-git-policy-cli zfs-root:` opening every root-mode refusal.
    Written in "Choice 5: what `--version` and the refusals say",
    and in `root_modes_name` and `version_line`.
6.  Choice 6, the account's `PATH`:
    no short name; `doctor` prints the root-owned path and the words.
    Written in "Choice 6: whether the account's `PATH` keeps a short name".
7.  Choice 7, the modes' test-only variables:
    all four stay in the release.
    Written in "Choice 7: whether the release keeps the modes' test-only variables".
8.  Choice 8, the record mode's word:
    `record-checkout`.
    Written in "Choice 8: the word of the mode that records a checkout",
    and in `MODE_WORDS`.
9.  Choice 9, what a legitimate restore does to the record:
    `--replace <the kernel's guid>`,
    with `replace-token-differs` when the value is not the kernel's.
    Written in "Choice 9: the record a legitimate restore replaces",
    and in the three refusals "The two modes these answers add" prints.
10. Choice 10, the panic hook a root mode installs:
    the wrapper's payload-free hook, inside the arm.
    Written in "Choice 10: the panic hook a root mode installs",
    and in the two pinned lines of `MAIN_CODE`.
11. Choice 11, the probe's deadline variable:
    compiled out of the release,
    and the raised-deadline control builds the probe with
    `--features start-probe-raised-deadline`.
    Written in "Choice 11: whether the release honours the probe's deadline variable",
    and in `probe_deadline.rs` and the crate's `[features]` table.

The closest pairs, kept because a reader will ask:

- choice 1's A over B,
  where B is the answer's literal minimum and A adds two accepted spellings for a word-free check of
  the installed bytes;
- choice 4's B over C,
  a tie by delivery whose order rests on C's parameter, which its one in-process caller cannot fill;
- choice 9's E over D,
  where each token protects a different race and both end in a correct record when neither refuses.

## Appendix: where the probe transcripts are

This round's folder,
`/home/user/temp/agent/zfs-root-modes-seventh-revision-2026-11-24/keep/`:

- `state-so-far.md`:
  what the round did,
  in order,
  and what it could not do;
- `guard-repairs-seventh.log`:
  the two printed files,
  the two pins,
  the pins' reader,
  the scan's wrapper, its rules and their type,
  the witness test,
  the lint-level anchor pin,
  the typed-ban configuration pin and the `[[bin]]` `path` assertion,
  cut out of this document by fence by `sources/cut_fences_seventh.py` and run as Rust by
  `sources/assemble_probe_seventh.py` and `sources/guard_replay_seventh.py`,
  over the document's own printed files,
  today's `main.rs` and the real `mod.rs` and `lib.rs`,
  every variant a read has planted,
  the read's seven new variants,
  the wiring fixtures and the manifest fixtures in `sources/fixtures/`,
  the three typed-ban fixtures,
  the scan fixtures,
  and the real release tree.
  Each verdict is the guard's own message,
  and its first lines are the md5 of every text it replays;
- `clippy-printed-main-and-module-seventh.log`:
  the printed `main.rs` and `root_mode_names.rs` compiled and linted in a stub crate under the
  crate's `[lints.clippy]` table and its `clippy.toml` with this design's two rows,
  and the printed fragments of `handler_install_paths.rs` and `probe_deadline.rs` in a second crate,
  with and without the feature choice 11 adds;
- `clippy-printed-guards-seventh.log`:
  the probe crate that holds every printed guard text, linted the same way, with no diagnostic;
- `statement-allow-scope-seventh.log`:
  two probe crates showing what a statement-level allow excuses and that the narrowed form of the
  three release allows still compiles;
- `host-facts-seventh.log`:
  the dataset table,
  the nested dataset's eighteen mount-relevant properties,
  the ownership and mode of its mountpoint and of every parent,
  both `findmnt` answers,
  the GUID digit counts,
  and the paths and name that do not exist here;
- `pin-cost-seventh.log`:
  the commits that touch `main.rs` in the window the fifth revision measured, and their count;
- `citations-at-this-rounds-base.txt`:
  every citation that names a path,
  printed with the line it names at `2b81674c10`, this round's base;
- `occurrences-seventh.txt`:
  the second program's name counted, with and without `--hidden`, for the change list's count;
- `main_rs.rs` and `names_rs.rs`:
  the two printed files as the logs compiled them,
  cut out of this document by fence;
- `sources/`:
  every file a log in this appendix was made from,
  named for what it is:
  `cut_fences_seventh.py` (the fence cutter, one file per printed text),
  `assemble_probe_seventh.py` (the probe crate, whose substitutions are listed at its head),
  `guard_replay_seventh.py` (the replay driver that writes the log),
  `variants/` (every variant and every scan fixture an earlier read planted),
  `read-new-variants/` (the sixth read's thirteen, kept as it wrote them),
  `fixtures/` (the wiring, manifest and typed-ban fixtures),
  `stub_crate_lib.rs`, `stub_crate_Cargo.toml` and `stub_crate_clippy.toml`
  (the stub crate the printed `main.rs` and module are compiled and linted in),
  and `citations.py` (the citation dump).

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

The sixth read's folder,
`/home/user/temp/agent/gate-review-zfs-root-modes-sixth-revision-2026-11-24/keep/`,
holds the document this round answers and its logs,
each cited where it is used:
`read-of-the-sixth-one-program-revision.md`,
`new-stripper-scan-and-lint-pin-variants.log` (the three files the sixth revision's stripper hid),
`new-variants-accepted-by-clippy-and-read-the-start-name.log`,
`new-wiring-variants.log`,
`redirects-compile-and-choose-the-other-file.log`,
`typed-ban-config-edits-no-guard-reads.log`,
`init-array-constructor-reads-the-start-name.log`,
`starts-of-the-sixth-revisions-printed-main-and-module.log`,
`guards-replay-this-read.log`,
`spot-checks-seeded-draw.log` and
`spot-checks-the-rest-of-the-candidates.log`,
with its `work/keep/variants/` beside them.

The fourth read's folder,
`/home/user/temp/agent/gate-review-zfs-root-modes-fourth-revision-2026-11-24/keep/`,
holds the two upstream libzfs sources this design reads:
`upstream-libzfs_mount-zfs-2.4.4.c` (the read-only-pool rule at `:428`) and
`upstream-libzfs_dataset-zfs-2.4.4.c`.
The upstream `cmd/zfs/zfs_main.c` at `zfs-2.4.4` was fetched through `gh` by the sixth read and is
`work/upstream-zfs_main-zfs-2.4.4.c` in that read's folder;
this round read it there.

This round's `scratch/` folder held the probe crates, their build trees and the two client crates
the statement-allow probe used, and is removed when the round ends.
Two logs are rebuilt by kept programs:
`guard-repairs-seventh.log` by `sources/cut_fences_seventh.py`,
`sources/assemble_probe_seventh.py` and `sources/guard_replay_seventh.py`,
and `citations-at-this-rounds-base.txt` by `sources/citations.py`.
The rest are command transcripts,
and each one's own header prints the commands that made it,
so a reader can run them again as they stand.
No guest,
no container and no ZFS-changing command ran this round,
which changes no code.
