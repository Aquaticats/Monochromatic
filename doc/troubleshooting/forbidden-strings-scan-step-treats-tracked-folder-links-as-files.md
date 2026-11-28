# forbidden-strings 0.6.2 and the CI scan step: a tracked link to a folder reaches the scanner as a file, so every push to `main` failed from 2026-11-20

Status:
unapproved and non-final (2026-11-27T21:14:21-05:00).
Three rounds wrote this record:
the one merged as `fc1c84b58e`,
which stopped the read errors,
the one for issue #6633,
which makes the steps inspect the links they leave out,
and the one for issue #6640,
which adds every kept link's stored target text and the whole-tree arm's links
and gitlinks.
The owner declares completion.
The last three sections list the commits,
the checks,
the measurements and what is open.

## Symptom

The `forbidden-strings` workflow failed on every push to `main`
for eight days,
printing nothing but read errors on entries under
`uses/` folders.

Measured by the coordinating session from the workflow's own logs
on 2026-11-27:

- the last green push run is 44656350646
  (head `82d6a89596`,
  2026-11-20T01:49:06Z);
- the first failed one after it is 44656950451
  (head `e260ae257b`,
  2026-11-20T01:54:13Z),
  which scanned 79 changed entries,
  printed eight read-error lines,
  and reported no forbidden string;
- run 46079689494 (head `49ca48000e`,
  2026-11-27T22:57:21Z)
  scanned 8149 entries,
  printed 139 such lines,
  and reported no forbidden string.

One of the 139 lines,
from that run's "Scan changed files" step:

```text
package/git-policy/cli/src/native/zfs_mounted_dataset/uses/foundation: read error: Is a directory (os error 21)
```

`git ls-files --stage` on `main` lists 139 symbolic links,
every one under a `uses/` folder,
and every one pointing at a folder,
which is the layout `AGENTS.md` requires:
"every source folder inside a crate holds a `uses/` folder of links to the
folders its code uses".

The step never advanced its baseline,
because the baseline is the newest successful push run of this workflow
revision
(`.github/workflows/forbidden-strings.yml:150-175`),
and no run succeeded.
So every push rescanned the whole delta since `82d6a89596`
and met all 139 links again.

What a red run at that point was worth:
the gate said "failed" on every push,
so a push that added a real forbidden string would have looked like every
other one,
its finding one line among 139 in a run nobody expects to pass.

## Root cause

Before this change one step,
"Scan changed files",
built its list of changed entries with
`git diff --name-only --diff-filter=ACMRT -z`,
at `.github/workflows/forbidden-strings.yml:178` (the push arm)
and `:191` (the pull-request arm) on `main`.
After it,
the two `git diff --raw --no-renames --diff-filter=ACMRT -z` calls
sit in the "List changed files" step at `:184` and `:193`,
and the entry filter and the scanner run in the "Scan changed files" step
from `:201`.
`--name-only` names a tracked symbolic link like any other path,
and the list becomes the scanner's positional arguments:

```sh
forbidden-strings --builtin-rules --rules "$rules" "${changed[@]}"
```

(`.github/workflows/forbidden-strings.yml:265` after this change).

The scanner opens every positional path with `File::open`
(`package/forbidden-strings/cli/src/lib.rs:619`)
and reads it with `read_to_end` (`:623`).
On Linux,
opening a folder succeeds and the read fails with `EISDIR`,
so a link whose target is a folder answers
`Is a directory (os error 21)`.
The error becomes a finding and counts toward exit 1:

```rust
    let content_result = match read_with_binary_check(p) {
        Ok(c) => Some(c),
        Err(e) => {
            findings.push(format!("{}: read error: {}", pathname.display, e));
            None
        }
    };
```

(`package/forbidden-strings/cli/src/candidate_file_scan.rs:161-166`,
and the README states the rule it implements at
`package/forbidden-strings/cli/README.md:730-735`:
"A file that cannot be opened (broken symlink,
 permission denied,
 deleted during scan)
 produces `PATH: read error: <reason>` on stderr and counts toward the exit-1
 total.
 A secret-scanning gate must not pass silently on a file it could not
 inspect.").

The walk (`--all`) never meets this,
because it filters index entries by mode:

```rust
                    let mode = entry.mode;
                    if !mode.contains(gix_index::entry::Mode::FILE)
                        && !mode.contains(gix_index::entry::Mode::FILE_EXECUTABLE)
                    {
                        continue;
                    }
```

(`package/forbidden-strings/cli/src/walk.rs:461-466`;
the comment above it at `:432-455` explains that `Mode::COMMIT` is a
submodule gitlink and `Mode::SYMLINK` would let `fs::read` follow the link
out of the repository),
and because the directory walk does not follow links
(`package/forbidden-strings/cli/README.md:1031`:
"Symlinks NOT followed.").
So the walk leaves both kinds out and the positional path did not.

The 2026-11-01 record has the same shape
(`doc/troubleshooting/forbidden-strings-appendix-self-match.md`,
"Update 2026-11-01"):
a rule the walk applied and positional arguments did not kept the workflow
red,
and that time it was cured inside the scanner,
with the limit "The fix reaches CI only in a release that carries it".
This time the scanner stays unchanged (see "What is left for the owner"),
because a release is a separate decision:
`doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md`,
item 4.9,
allows alpha releases unasked and says "anything that is not an alpha is
asked first",
and the scanner's version is `0.6.2`.

## What was measured

Everything below is a real invocation on 2026-11-27,
from this worktree,
with `forbidden-strings` 0.6.2:

- the release asset `forbidden-strings-0.6.2-x86_64-unknown-linux-gnu.tar.gz`
  of tag `forbidden-strings-v0.6.2`,
  repository `Aquaticats/Monochromatic`
  (archive sha256
  `fff00e61b5187878affb37cbd19baa87398e1ca2d312305d817430f79f13b711d`),
  unpacked binary sha256
  `8537a0917e209b1d2c34580416d30a50c3da6606c67d4634f5bf86b799036e51`;
  `forbidden-strings --version` prints `forbidden-strings 0.6.2`;
- the scanner's behavior on each kind of positional argument;
- the workflow's own step bodies,
  run on the same delta before and after this change.

### The released scanner on each kind of positional argument

The probe is a disposable folder with a `base/` folder,
a `user/uses/` folder of links,
one regular file,
and a rules file with one content rule and one rule that matches a name
segment:

```sh
probe="$HOME/temp/agent/link-positional-probe"
mkdir --parents "$probe/base" "$probe/user/uses"
printf 'clean line\n' > "$probe/base/lib.txt"
printf 'zzq-scope-violation-sentinel inside the link target\n' > "$probe/base/sentinel-holder.txt"
ln -s ../../base "$probe/user/uses/zzqnametoken-folder-link"
ln -s ../../base/lib.txt "$probe/user/uses/link-to-file"
ln -s ../../nowhere "$probe/user/uses/broken-link"
ln -s ../../base/sentinel-holder.txt "$probe/user/uses/link-to-sentinel-file"
printf 'ordinary content\n' > "$probe/real.txt"
cat > "$probe/rules.txt" <<'EOF'
==> ci-scope-fixture-sentinel <==
zzq-scope-violation-sentinel
==> ci-scope-fixture-name <==
zzqnametoken
EOF
cd "$probe" && git init -q --initial-branch=main .
```

The `git init` matters:
the scanner then uses repository-relative names,
as it does on the runner.
Each invocation below is `forbidden-strings --rules rules.txt -- <argument>`
from `$probe`.

A positional link to a folder
(`user/uses/zzqnametoken-folder-link`):

```text
{"type":"forbidden-strings/cache-warning","schemaVersion":1,"reason":"missing","recovery":"compile-from-text"}
user/uses/[REDACTED]:name:3 rule=ci-scope-fixture-name
user/uses/[REDACTED]: read error: Is a directory (os error 21)
exit 1
```

Two things at once:
the link's own path segments are name-scanned
(the rule matching `zzqnametoken` fires,
and the segment is masked as
`[REDACTED]`),
and the read fails on the folder the link points at.
The cache-warning line appears once per empty cache directory and appears
in green runs too.

A positional link to a file whose target holds the forbidden string
(`user/uses/link-to-sentinel-file`):

```text
user/uses/link-to-sentinel-file:1 rule=ci-scope-fixture-sentinel
exit 1
```

The link is read through and the target's content is reported under the
link's own path.

A positional broken link (`user/uses/broken-link`):

```text
user/uses/broken-link: read error: No such file or directory (os error 2)
exit 1
```

A positional folder (`base`):

```text
base: read error: Is a directory (os error 21)
exit 1
```

A positional regular file (`real.txt`):
exit 0.

`--all` over the same tree:

```text
base/sentinel-holder.txt:1 rule=ci-scope-fixture-sentinel
base/[REDACTED]:name:2 rule=ci-scope-fixture-name
exit 1
```

No line for any of the four links.
The walk lists neither a link's name nor its target text,
so `--all` inspects less of a link than the positional path does.
(The second line is the name of `base/zzqnametoken-base.txt`,
a regular file,
not a link.)

### `--name-path` can carry a link's own target text

Git stores a symbolic link as a blob whose content is the target text,
so a caller can materialize it:
`git cat-file blob <the 120000 entry's sha>`,
or `readlink` on the checked-out link.
Passing that materialized text as the content file and the link's path as
the logical name makes the scanner scan the target text under the link's
name:

```sh
printf '../../base/zzqnametoken-base.txt' > materialized/plain-link-target-text
forbidden-strings --rules rules.txt --name-path user/uses/plain-link materialized/plain-link-target-text
```

```text
user/uses/plain-link:1 rule=ci-scope-fixture-name input=0
exit 1
```

The bytes scanned are exactly the link's stored target text,
the finding names the link,
and `input=0` is the operand index the option
appends.
The logical name is also name-scanned in the same invocation,
so one call can cover both the link's name and its target text.
Nothing in the workflow materializes those blobs today.

### The workflow's step before and after this change, on the issue's own delta

Harness:
the workflow's own step bodies,
extracted from the workflow file exactly as
`package/forbidden-strings/cli/tests/ci_scan_scope/scan_step_command.rs`
extracts them,
and run in workflow order;
the released 0.6.2 binary on `PATH` as `forbidden-strings`;
a stand-in `gh` whose `run list` prints `82d6a8959655c863e4a28579139456f609d5d615`;
the repository's own `forbidden-strings.append.txt` as the rules file;
a disposable `RUNNER_TEMP` and scanner cache directory;
`EVENT_NAME=push`,
`HEAD_SHA=49ca48000e64f20225ac2fce93a524c99596d738`,
`BRANCH=main`.

`main`'s step body,
over `82d6a89596..49ca48000e`:

```text
Scanning 8149 changed file(s) from 82d6a8959655c863e4a28579139456f609d5d615..49ca48000e64f20225ac2fce93a524c99596d738:
  .agents/skills/choosing-technology/SKILL.md
...
package/git-policy/cli/src/native/checkout_boot_units/uses/testing: read error: Is a directory (os error 21)
package/git-policy/cli/src/native/checkout_boot_units/uses/worktree_clone_mount_handler: read error: Is a directory (os error 21)
...
step exit=1
```

139 lines of that shape,
and no other finding:
the same count,
on the same delta,
that run 46079689494 printed.

The two step bodies after this change,
same delta,
same binary:

```text
Left out 139 of 8149 changed entries the scanner cannot open as a file: folder links 139, submodule gitlinks 0. A link to a file, a broken link and an unreadable file stay in the scan.
Scanning 8010 changed file(s) from 82d6a8959655c863e4a28579139456f609d5d615..49ca48000e64f20225ac2fce93a524c99596d738:
  .agents/skills/choosing-technology/SKILL.md
...
step exit=0
```

The scanner's stderr held the cache-warning notice and nothing else.

The working tree those two runs read is this branch's
(a descendant of `main` at `50e4b07543`),
not `49ca48000e`'s tree,
so file contents could differ;
the entry list,
the counts and the read-error shape are the delta's own.
Both runs are one invocation each,
not repeats.

## The cure

The list is now read with
`git diff --raw --no-renames --diff-filter=ACMRT -z`,
whose records carry each entry's mode in the tree next to its path,
in both arms of the "List changed files" step
(`.github/workflows/forbidden-strings.yml:184` for a push and `:193` for a
pull request),
and the "Scan changed files" step filters that one list for both arms:

- mode `160000` (a submodule gitlink) leaves the list always.
  Its path is a folder when checked out and absent otherwise;
  the walk leaves `Mode::COMMIT` out for the same reason.
- mode `120000` (a symbolic link) leaves the list only when
  `[ -d "./$entry" ]` is true,
  which follows the link and is true only when its target on disk is a
  folder.
- every other entry is handed to the scanner as before:
  a link to a file is still read through,
  a broken link still produces its read error,
  a regular file the scanner cannot open still produces its read error.

Why the disk test and not the mode alone:
mode `120000` covers links to files as well,
and a link to a file is content the scanner reads today
(measured above).
Leaving every link out would stop inspecting those targets,
which the requirement forbids.
The mode alone cannot tell a link to a folder from a link to a file
without resolving the target text against the tree,
which is more shell than the distinction is worth;
`-d` answers the exact question the scanner's `fs::read` fails on.

Why not leave out every entry whose path is a folder on disk,
whatever the mode:
a tracked regular file whose on-disk path had become a folder would then
leave the list silently.
The mode test keeps such an entry in,
where the scanner reports the read error it reports today.

`--no-renames` keeps one record per entry with one path,
so the parser reads exactly two NUL-terminated fields per change and the
path set matches what `--name-only` produced
(measured:
for a rename,
`--name-only` names the destination only,
and `--raw --no-renames` reports the destination as an add and the source
as a delete,
which the filter drops).

The work sits in two steps because the repository's linter caps one shell
step at 58 code lines (`shell/max-lines`),
and `main`'s single step used all 58:
`monochromatic-lint` reports no finding on `main`'s workflow file,
and its "Scan changed files" body counts 58 code lines
(comments and blank lines excluded).
One step would have meant either an over-limit step or compacting tested
scope logic to buy room,
so the scope selection and the entry filter are separate steps,
and the tests run both bodies in workflow order,
each in its own `bash`,
as the runner does.
Nothing else in the scope logic changed.

Nothing leaves silently.
When any entry was left out the step prints one line before it scans:

```text
Left out 139 of 8149 changed entries the scanner cannot open as a file: folder links 139, submodule gitlinks 0. A link to a file, a broken link and an unreadable file stay in the scan.
```

When every listed entry was left out,
the step says
`Every changed entry in <range> is one the scanner cannot open as a file; nothing to scan.`
and exits 0 without invoking the scanner,
so it never announces a scan it did not do.
When no entry was left out,
no such line appears;
nothing was left out.

Both arms are tested through the workflow's own two step bodies,
run in workflow order,
in `package/forbidden-strings/cli/tests/ci_scan_scope/folder_link_entry_tests.rs`:

- a push whose delta holds a folder link and no forbidden string ends
  green and prints the left-out line;
- a forbidden string in a regular file in the same push still ends it 1;
- a changed link to a file is still read through,
  and a forbidden string in its target is reported under the link's path;
- a broken link still ends the step 1 with
  `read error: No such file or directory (os error 2)`;
- a push whose every changed entry is a folder link ends 0 and says what it
  left out;
- a push whose only changed entry is a submodule gitlink ends 0 and says
  so;
- the pull-request arm leaves a folder link out and keeps its
  `origin/main...HEAD` scope,
  and still reports a forbidden string beside
  the link.

The acceptance test was red before the workflow change:
6 of the 8 cases failed,
the acceptance one printing

```text
Scanning 2 changed file(s) from a8a74f0a8438142bfbdfe778fea6b84d742c7a06..9221cfa8be1d19710fdc7e30cf1ba332465080e5:
  consumer/main.txt
  consumer/uses/base

stderr: {"type":"forbidden-strings/cache-warning","schemaVersion":1,"reason":"missing","recovery":"compile-from-text"}
consumer/uses/base: read error: Is a directory (os error 21)
```

(the two cases that passed on `main` are the two controls that must pass
there:
the broken link and the read-through of a link to a file).

## What the steps inspect now (issue #6633)

This section records the state issue #6633 left;
the section "What the steps inspect now (issue #6640)" supersedes its entry
lists.

The requirement issue #6633 states:
every entry the file scan leaves out of its positional list is still handed
to the scanner,
a tracked link by its own name and by its stored target text,
a submodule gitlink by its own name;
nothing the file scan inspects today stops being inspected;
and nothing is followed that was not followed before.

### Three steps and two scanner calls

The file scan writes each entry it leaves out,
with its raw record,
to `$RUNNER_TEMP/forbidden-strings.left-out-raw`,
and its left-out line now says what inspects them instead:
"The next step scans their own names and,
for each link,
its stored target
text."

A third step,
"Scan the entries left out of the file scan",
reads that list and hands every entry over in one scanner call:

```sh
forbidden-strings --builtin-rules --rules "$rules" \
  --name-path <entry> [--name-path <entry> ...] -- <input> [<input> ...]
```

- For a tracked link the input is the link's stored target text,
  written from the link's own blob in the tree
  (`git cat-file blob` of the new object the raw record names),
  never from the path the link points at.
  `--name-path` name-scans the link's own path segments and content-scans
  the bytes Git stores for it in the same invocation,
  which is what the 6632 round measured on `0.6.2`.
- For a submodule gitlink the input is one empty file written once,
  so the scanner name-scans the gitlink's path and content-scans nothing,
  because a gitlink's own commit is not a file of this repository.

Every finding of that call carries `input=N`,
the pair's index,
so a finding names the entry it came from even when the entry's matching
segment is redacted.

Both scanner calls now end their option list with `--`.
`git diff --raw` names repository paths,
and a path can begin with a dash,
which `forbidden-strings` `0.6.2` read as an option:
measured,
`forbidden-strings --rules rules.txt -- -dashfile.txt` reports the finding,
and the same call without `--` answers
`error: unexpected argument '-d' found` with exit 2.
A repository file with a dash-leading name would therefore have failed the
gate with a usage error instead of being scanned,
which is why both calls now end their option list with `--`.
That hardening is new with issue #6633's round;
before it,
the file scan handed its list to the scanner without one.

### Why this shape and not the others

One scanner call for everything,
with `--name-path` for every positional file:
`--name-path` requires one name per positional file,
so every finding would gain an `input=` label and the output shape every
other consumer of this gate reads would change,
and a link to a file would need two entries under one name
(its read-through and its stored target text)
to keep today's read-through.

Both calls in one step:
the repository's linter caps a shell step at 58 code lines
(`shell/max-lines`,
`.github/workflows/monochromatic-lint.config.jsonc`),
and the file scan's body is 53 of them;
building the pairs needs about twenty more,
so one step would mean either an over-limit step or compacting tested scope
logic to buy room,
which `AGENTS.md` forbids without the owner's approval.
The file scan's body is 53 code lines and the new step's is 33,
both measured by `monochromatic-lint`'s own count
(blank and comment lines excluded).

A script the workflow runs:
`AGENTS.md` rule `SCR` bans shell scripts in this repository,
and a TypeScript program would need Node,
pnpm and `task-run` in a workflow that has none,
which `doc/decision/mise-free-ci.md` makes the way CI runs a task.

A change in the scanner:
what a positional link means inside the scanner is issue #6633's separate
question,
and any scanner change reaches this workflow only in a release,
which item 4.9 of
`doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md` makes a question
for the owner because `0.6.2` is not an alpha.

Reading the link's target text with `readlink` instead of its Git blob:
both give the same bytes on a fresh checkout,
so no case in the test suite tells them apart except the one that asserts
the mechanism;
the blob is the tree's own answer,
it is already named by the raw record the step reads,
and it does not depend on the checked-out link matching the tree.

### What each kind of changed entry gets now

- A regular file:
  its path segments and its content,
  positionally,
  as before.
- A link to a file:
  its path segments,
  and the content behind it read through the link under
  the link's own name,
  as before.
  Its stored target text is not scanned
  (see "Limits of what the steps inspect now").
  Moved on 2026-11-27:
  issue #6640 scans that stored target text as well,
  in "What the steps inspect now (issue #6640)".
- A broken link:
  the scanner's read error and exit 1,
  as before.
- A link whose target on disk is a folder:
  its path segments and its stored target text,
  in the left-out step.
- A submodule gitlink:
  its path segments,
  in the left-out step.
- An entry whose mode the filter does not name,
  or whose path is missing from the raw record:
  handed over as before,
  so an unexpected tree state still fails loudly.
- Every changed entry,
  when no validated baseline exists:
  the whole-tree arm still runs `--all`,
  which inspects neither a link's name nor any target text
  (see the limits).
  Moved on 2026-11-27:
  issue #6640 hands every tracked link and gitlink of the index to the scanner
  in that arm too.

### The delta the workflow scans today, measured

Measured on 2026-11-27,
in this worktree,
with the released `forbidden-strings` `0.6.2`
(tag `forbidden-strings-v0.6.2`,
archive sha256
`fff00e61b5187878affb37cbd19baa87398e1ca2d312305d817430f79f13b711d`,
binary sha256
`8537a0917e209b1d2c34580416d30a50c3da6606c67d4634f5bf86b799036e51`),
the workflow's own four step bodies
("Materialize deny-list",
"List changed files",
"Scan changed files",
"Scan the entries left out of the file scan")
extracted the way
`package/forbidden-strings/cli/tests/ci_scan_scope/scan_step_command.rs`
extracts them and run in workflow order,
the repository's own `forbidden-strings.append.txt` as the rules file,
a `gh` stand-in answering
`82d6a8959655c863e4a28579139456f609d5d615` for `run list`,
stock Git as the runner has it,
`EVENT_NAME=push`,
`HEAD_SHA=60ecd08eefd447e7fbf397b134ae7d1b004c52ff`,
`BRANCH=main`,
a disposable `RUNNER_TEMP` and scanner cache directory.

```text
::notice::FORBIDDEN_STRINGS_LIST secret not set; baseline + shared appendix scan.
Left out 139 of 8158 changed entries the scanner cannot open as a file: folder links 139, submodule gitlinks 0. The next step scans their own names and, for each link, its stored target text. A link to a file, a broken link and an unreadable file stay in the scan.
Scanning 8019 changed file(s) from 82d6a8959655c863e4a28579139456f609d5d615..60ecd08eefd447e7fbf397b134ae7d1b004c52ff:
Scanning 139 left-out entries: each one by its own name, and each link by its stored target text:
```

All four steps exited 0,
and neither stream held a line with `rule=` or `read error`:
under the repository's own rules and the embedded baseline,
the 139 links' own names and stored target texts are clean,
and so are the 8019 files the file scan read.

The run read this branch's tree,
which is `main` plus this round's changes,
so the content of the six entries this round edited or added
(the workflow file and five files under
`package/forbidden-strings/cli/tests/ci_scan_scope/`)
is this branch's and not `60ecd08eef`'s.
The entry list,
the counts,
and every link's stored target text,
come from the delta's own Git objects and are unaffected.

The left-out step left 140 files in
`$RUNNER_TEMP/forbidden-strings.link-target-text`:
139 stored target texts and the one empty file a gitlink would use,
which this delta does not need.
The first stored target text is `../../testing`
for `package/git-policy/cli/src/native/checkout_boot_units/uses/testing`,
and the record file beside it held 139 raw records.

Two plants show the harness can report a finding on that delta,
each reverted after its run:

- a rule for `../`,
  which every `uses/` link's stored target text holds,
  made the left-out step exit 1 with 139 findings,
  the first being
  `package/git-policy/cli/src/native/checkout_boot_units/uses/testing:1 rule=control-left-out-target-text input=0`;
- a rule for `validated-tree-watermark-v1` made the file scan exit 1 with
  three findings,
  the first being
  `.github/workflows/forbidden-strings.yml:129 rule=control-file-content`.

## What the steps inspect now (issue #6640)

Issue #6640 asks for two inspections the round of issue #6633 left out:
every changed link's stored target text,
including a link to a file that stays in the file scan,
and every tracked link and gitlink of the index in the whole-tree arm.
Both are closed in the workflow,
with no release of the scanner.

### Four steps and the scanner calls they make

The step that scanned the entries the file scan leaves out is now two steps,
and both carry `if: ${{ !cancelled() }}`:

- "Write each link's stored target text to a file"
  lists the links and gitlinks to inspect and writes each one,
  in the raw-record shape the next step parses,
  to `$RUNNER_TEMP/forbidden-strings.link-and-gitlink-raw`.
  A delta's list comes from the raw changed-entry list the scope step wrote;
  a whole-tree run has no such list and takes every `120000` and `160000`
  entry of the index with `git ls-files --stage -z`.
  A whole-tree run's line names what it hands over:

```text
Whole-tree scan: the --all walk leaves link and gitlink entries out, so the next step scans each tracked link and gitlink by its own name and each link by its stored target text: tracked links 139, submodule gitlinks 0.
```
- "Scan each link's stored target text and each gitlink's name"
  reads that list and hands every entry to one scanner call as a
  `--name-path` pair,
  with the link's blob from `git cat-file blob` as the input,
  or one empty file for a gitlink.

The file scan hands over what it handed over before:
a regular file,
a link to a file,
a broken link and an unreadable file go to
it positionally;
a folder link and a gitlink do not.
Its left-out line gained one clause,
and the count it states fills the last `%d`,
from `.github/workflows/forbidden-strings.yml:263`:

```text
Left out %d of %d changed entries the scanner cannot open as a file: folder links %d, submodule gitlinks %d. The next step scans their own names and, for each link, its stored target text. A link to a file, a broken link and an unreadable file stay in the scan; the stored target text of every changed link is scanned there too, %d in all.
```

### What each kind of entry gets now

- A regular file:
  its path segments and its content,
  positionally,
  as before.
- A link to a file:
  its path segments,
  the content behind it read through the link under the link's own name,
  and now its stored target text under the same name in the same run.
- A broken link:
  the scanner's read error and exit 1,
  as before,
  and now its stored target text under its own name,
  which the tree still holds though the path it names does not exist.
- A folder link:
  its path segments and its stored target text,
  as issue #6633 left it.
- A submodule gitlink:
  its path segments,
  as issue #6633 left it.
- Every entry of a whole-tree run:
  every regular file the `--all` walk finds,
  plus every link and gitlink of the index,
  each by its own name and,
  for a link,
  by its stored target text.

Nothing the steps scanned before stops being scanned.
The file scan's positional list is the same list;
every entry in it is still name-scanned and read as before;
a delta's links are added to the second call beside it;
and the whole-tree arm gains the index's links and gitlinks beside the walk
instead of losing the walk.

### What a person sees when the content behind a link and its stored text both match

Two findings,
in one run,
under one link name.
From a workflow-test fixture
(`push_of_a_link_whose_content_and_stored_target_text_both_match_prints_both`):

```text
consumer/uses/lib:2 rule=ci-scope-fixture-sentinel
consumer/uses/lib:1 rule=ci-scope-fixture-sentinel input=0
```

The first comes from the file scan's read-through,
at line two of the file
behind the link,
and carries no `input=` index;
the second comes from the link's stored target text,
at line one of it,
labelled with the pair index `0`.
Both appear because the two later steps carry `if: ${{ !cancelled() }}`:
the runner skips the rest of a job after a failing step,
and without the check the file scan's finding would hide the link's.
`!cancelled()` and not `always()`,
because every later push cancels the run in flight
(`concurrency` with `cancel-in-progress: true`).

The same name is scanned twice for a link to a file,
once by the file scan and once by the pair,
so a forbidden path segment in such a link's name prints two name findings,
`PATH:name:N rule=X` and `PATH:name:N rule=X input=N`.
That is the cost of keeping the read-through and adding the pair.

### The whole-tree arm measured on this repository

Measured on 2026-11-27,
in this worktree,
with the released `forbidden-strings` `0.6.2`
(tag `forbidden-strings-v0.6.2`,
archive sha256
`fff00e61b5187878affb37cbd19baa87398e1ca2d312305d817430f79f13b711d`,
binary sha256
`8537a0917e209b1d2c34580416d30a50c3da6606c67d4634f5bf86b799036e51`),
the workflow's four step bodies
extracted from the workflow file exactly as
`package/forbidden-strings/cli/tests/ci_scan_scope/scan_step_command.rs`
extracts them,
the repository's own `forbidden-strings.append.txt` as the rules file,
a stand-in `gh` that answers no run at all,
stock Git,
`EVENT_NAME=push`,
`BRANCH=main`,
a disposable `RUNNER_TEMP` and one warm scanner cache directory:

```text
::notice::No validated scan baseline; scanning the current tree.
Whole-tree scan: the --all walk leaves link and gitlink entries out, so the next step scans each tracked link and gitlink by its own name and each link by its stored target text: tracked links 139, submodule gitlinks 0.
Scanning 139 link and gitlink entries: each one by its own name, and each link by its stored target text:
```

All four steps exited 0,
and neither stream held a line with `rule=` or `read error`:
under the repository's own rules and the embedded baseline,
this tree's 139 links and 0 gitlinks are clean by name and by stored target
text.

The clean result is not a silent pass.
A positive control added the literal rule `..`,
which every `uses/` link's stored target text holds,
to the same rules file and ran the listing and the link scan over the same
tree:
the scan exited 1 with 139 findings,
the first being
`package/git-policy/cli/src/native/checkout_boot_units/uses/testing:1 rule=control-link-target-parent-segments input=0`.
(A first attempt wrote the rule as `\.\./` and found nothing:
a bare rule line is a case-sensitive literal
(`package/forbidden-strings/cli/README.md`,
"Legacy format (one rule per line)"),
not a regex,
so that line looked for the five bytes `\.\./`.)

### What the arm costs

Three rounds,
the two arms interleaved in each round with the order
alternating,
so the control is measured beside the change:

- round one,
  without then with:
  82.164 s against 46.802 s,
  ratio 0.570;
- round two,
  with then without:
  50.620 s against 77.370 s,
  ratio 0.654;
- round three,
  without then with:
  47.118 s against 84.308 s,
  ratio 1.789.

The ratio's range is 0.570 to 1.789,
and it says nothing about the hand-over:
the unchanged `--all` walk alone took 44.116 s to 82.155 s across those six
runs,
a spread of nearly two times,
which is this machine under other agents' load (rule `NQH`).
The added cost is the two added steps' own time,
measured in the same runs:

- the index listing,
  one `git cat-file blob` per link:
  1.737 s to 1.845 s;
- the link scan,
  139 `--name-path` pairs in one call:
  0.562 s to 0.835 s;
- together:
  2.299 s to 2.680 s.

That is the figure to compare against a walk of 44 s to 82 s on this tree,
and the pair of steps is too small for a whole-arm ratio to resolve here.

### Why this shape and not the others

- One scanner call for everything,
  with `--name-path` for every positional file:
  `--name-path` requires one name per file,
  so every finding would gain an `input=` label and the output shape every
  other consumer of this gate reads would change,
  and a link to a file would need two entries under one name to keep its
  read-through.
- Building the pair list in the file-scan step:
  the repository's linter caps one shell step at 58 code lines
  (`shell/max-lines`),
  and the file scan's body is 51 after this change;
  the pair building is about thirty more.
  The list has one owner instead,
  and the file scan lost the writing of the old left-out list,
  which the new step does for both arms.
- Reading the target text with `readlink`:
  both give the same bytes on a fresh checkout,
  so no case tells them apart except the one pinning the mechanism;
  the blob is the tree's own answer,
  it is already named by the record the step reads,
  and a whole-tree run has no checked-out copy of a link it may trust either.
- A change in the scanner that treats a positional link as a link:
  what a positional link means inside the scanner is issue #6633's separate
  owner question,
  and any scanner change reaches this workflow only in a release.
- No status check,
  running the later steps as before:
  a file scan's finding would hide a link's,
  and the both-match pair would never be seen together,
  because the link leaves the delta once the content behind it is fixed.
- `always()` instead of `!cancelled()`:
  `always()` also runs in a cancelled run,
  and every later push cancels the run in flight,
  so the two steps would scan a tree nobody is waiting for.
- Running the link scan before the file scan:
  the same hiding,
  mirrored:
  the link's finding would then hide the file scan's.

## Limits of what the steps inspect now

- A delta scans the links its changed entries name.
  A link that did not change is not rescanned by a delta;
  the whole-tree arm is what covers it,
  on a first scan and after a baseline is lost.
- A submodule gitlink has no content of this repository behind it,
  so only its own name is inspected;
  the commit it records is not scanned,
  by this workflow or by the walk.
- A link to a file has its name scanned twice,
  once by the file scan's pathname pass over the positional path and once by
  the `--name-path` pair,
  so a forbidden segment in such a link's name prints two findings.
  Its content behind and its stored target text are different bytes,
  so their findings are not duplicates;
  the name findings are.
- The two later steps run after a failure but not after a cancellation,
  so a run cancelled by a later push reports nothing from them;
  the next run rescans what it covers.
  Each step reports its own exit 1 and its own findings.
- Both later steps read each target text from the tree's blob,
  so they are right even when a working-tree link differs from the tree,
  and in a whole-tree run the index is the list.
  They need those blobs to be present:
  a shallow or partial checkout fails the step loudly
  (`set -euo pipefail` around `git cat-file blob`)
  instead of skipping an entry,
  and the workflow checks out with `fetch-depth: 0`.
- A name holding a line break cannot be matched by a single-line rule,
  and `forbidden-strings` `0.6.2` answers it with
  `[REDACTED]: unsupported pathname line break`,
  which counts toward exit 1.
  Such a name fails the gate whether or not it holds a forbidden string,
  which is the scanner's answer today and not a choice these steps make.
- The link scan is one scanner call,
  so a rule-file error ends it for every entry at once;
  a per-file matcher fault still becomes that file's finding through the
  scanner's own `catch_unwind` boundary.
- The pair list and the materialized target texts stay in `$RUNNER_TEMP`,
  where the runner discards them with the job,
  like the rules file and the two lists beside them.
- The target texts are not removed by the step:
  no deletion runs over a path this step builds,
  which `AGENTS.md` rule `RLW` keeps free of variables.
- The whole-tree arm reads the index with one `git cat-file blob` per link,
  measured at 1.737 s to 1.845 s for this repository's 139 links.
  `git cat-file --batch` would read them in one process;
  it is the shape to take if a tree with thousands of links ever makes that
  time matter.
- The whole-tree arm's `--all` walk and its index listing are two reads of one
  tree,
  so an entry that changed between them would be inspected as one or the
  other;
  a push run scans a checkout nobody edits during the job.

## Limits of the issue #6633 cure

The state after issue #6640 is in "Limits of what the steps inspect now".

- A link's own name was inspected before that change,
  by the scanner's pathname pass over the positional path
  (`package/forbidden-strings/cli/src/candidate_file_scan.rs:155-160`),
  and was not inspected for a link the step left out
  (measured:
  the name rule fired on
  `user/uses/zzqnametoken-folder-link` before the reader failed).
  Issue #6633's round closed that:
  the step that scans the left-out entries name-scans every one of them.
- A link's target text was scanned by neither the walk nor the step before
  that round.
  The left-out step now scans it for every link the file scan leaves out;
  a link to a file is still read through without its stored target text
  being scanned,
  and the whole-tree arm is unchanged.
  See "Limits of what the steps inspect now".
  Moved on 2026-11-27:
  issue #6640 scans a link to a file's stored target text too,
  and the whole-tree arm lists the index's links and gitlinks.
- The disk test assumes the step runs at the repository root,
  which the runner guarantees (`actions/checkout` into `GITHUB_WORKSPACE`
  and the step's default working directory).
  If it did not,
  `[ -d "./$entry" ]` would be false for every link and the
  step would keep failing closed with the read error,
  never skip an entry silently.
- A mode the filter does not name,
  or a path absent from the raw record,
  keeps the entry,
  so an unexpected tree state still fails loudly.
- The filter decides by the entry's mode at the head commit.
  An entry that changes from a regular file to a link to a folder leaves
  the list in the same push,
  which is right for content scanning (nothing readable sits behind it in
  the new tree) and loses the old blob's content,
  as the walk and the read-error path did before.

## What is left for the owner

1. What a positional argument that is a link should mean inside the
   scanner.
   Today,
   measured:
   a link to a folder is name-scanned and then fails with
   `Is a directory`;
   a link to a file is read through and its target's content is reported
   under the link's path;
   a folder passed positionally fails the same way a link to a folder does.
   Three ways:
   - leave it as today:
     costs one read error per link to a folder in any list that still
     contains one (the commit guard's `SCANNER_SELF_MATCH_PATHS` list and
     the walk do not,
     but any other caller would);
   - skip links as files under positional arguments,
     matching the walk:
     costs the read-through of a link to a file,
     which some caller may
     rely on,
     and needs a release of the scanner to reach CI;
   - scan a link's own name and target text without following it:
     costs a new code path,
     a new output shape for "the link itself",
     and a
     release,
     and it changes what a security gate reports.
   A scanner-side cure reaches CI only in a release,
   and item 4.9 of
   `doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md` makes a
   non-alpha release a question for the owner.
   Issue #6633's round works around this for the workflow alone:
   the steps hand each left-out link over as a `--name-path` pair,
   which `0.6.2` already supports.
2. The two choices this item held were taken by the round of issue #6640:
   a link to a file is read through and scanned by its stored target text,
   and the whole-tree arm lists every link and gitlink of the index.
   Issue #6640 states the ask and the coordinating session's ranking,
   that scanning both beats leaving either at one more scanner input per link;
   the owner's veto on that ranking is open,
   and a veto puts the two steps back to a delta's left-out entries only,
   losing a kept link's stored target text and the whole-tree arm's links.
3. Whether the 139 `uses/` links should exist as links at all,
   given that a content scan checks neither a link's name nor its target
   text,
   and only a file's own path segments carry a folder's name into the scan.
   This record does not decide that;
   it only stops the read errors.
   Moved on 2026-11-27:
   issue #6640 now scans every link's name and stored target text,
   so this item no longer rests on their being unscanned.

## What does not work, and why

- Handing the link to the scanner anyway and filtering its read error:
  a gate that hides one class of read error cannot tell a real "cannot
  inspect" from noise,
  and the README's rule
  ("A secret-scanning gate must not pass silently on a file it could not
  inspect") is the reason the error exists.
- Leaving every `120000` entry out,
  whatever it points at:
  measured above,
  a link to a file is content the scanner reads today.
  The test
  `changed_link_to_a_file_still_reads_its_target_content` fails under that
  rule,
  and so does the broken-link control.
- Testing the runner's disk for every entry and dropping every path that is
  a folder:
  it would drop a tracked regular file whose path had become a folder,
  and silently rather than with the read error the scanner prints today.
- Curing it inside the scanner only:
  see the 2026-11-01 record's limit,
  "The fix reaches CI only in a release that carries it",
  and item 4.9's non-alpha question.
  It would also leave every other caller of the workflow's list shape with
  the same read error.
- `git ls-tree` over the head tree and a mode lookup per path:
  it works,
  but it is a second whole-tree read and an associative array in
  shell,
  where `git diff --raw` already carries each entry's mode next to the path
  the step needs.

## What the issue #6632 round did, checked and left open

Done,
in this worktree,
in four commits:

- `d20e1d6b22b88725425c7b06c43fc6a19d49cf79`
  (2026-11-27T18:18:22-05:00):
  `test(forbidden-strings): pin which changed entries the scan step hands over`,
  the eight cases and the fixture helpers
  (`link`,
  `gitlink`,
  `commit_index`).
- `db985a58ada907fed614311f5083dfa499127fe9`
  (2026-11-27T18:18:34-05:00):
  `fix(forbidden-strings): leave folder links and submodule gitlinks out of the scan`,
  the first single-step cure,
  which the linter rejected as an over-limit shell step
  and which `6d6581a11c` replaces.
- `6d6581a11c7b9ecb23c9859de7835a730563e875`
  (2026-11-27T18:26:28-05:00):
  `fix(forbidden-strings): split the scan step so the list filter fits the step limit`,
  the two-step shape and the harness that runs both bodies.
- `65435033f67a2536e384dc7a0acde9b1d01ac33a`
  (2026-11-27T18:28:38-05:00):
  `style(forbidden-strings): format the workflow-test cases with cargo fmt`,
  the 13 formatting diffs `cargo fmt --check` reported in the two test files.

Checked in `package/forbidden-strings/cli`,
`cargo fmt --check`,
`cargo check --locked --offline --all-targets`,
`cargo clippy --locked --offline --all-targets -- -D warnings`,
and `cargo nextest run --locked --offline --test ci_scan_scope`
(`14 tests run: 14 passed, 0 skipped`);
the package's own test task,
`mise run //package/forbidden-strings/cli:test`,
printed `Summary [ 22.003s] 373 tests run: 373 passed, 0 skipped` and exited 0.
All of those were run again after `origin/main` was merged into this branch
(the merge is `47ddc11eea`,
2026-11-27T18:31:04-05:00),
with the same results:
every command exited 0,
`cargo fmt --check` reported no diff,
and the two test runs printed those summary lines.
Before the merge the same task printed
`Summary [ 21.184s] 373 tests run: 373 passed, 0 skipped`.

The tests were seen red first:
before the workflow change 6 of the 8 new cases failed,
and after it 8 of 8 passed;
the two-step form was run again and 14 of 14 passed.
Three plants show the controls can fail,
each reverted after its run:
leaving out every symlink fails the link-to-file and broken-link cases;
leaving out every entry fails the positive control with
`a forbidden string in a regular file must still fail the step; got status 0`;
removing the left-out line fails six cases on the missing sentence.
`monochromatic-lint` reports no finding on any file this change writes,
before and after the merge:
the workflow file,
the three workflow-test files,
and this record.
The last planted mutant was reverted;
the committed workflow file's sha256 is
`03e842a2691ab578b252edb888371b49d1e8fde789363a9bdae241c8922211e8`,
the same file the linter and the tests read.

Open:
the workflow side takes effect at the merge,
and a green push run on `main` is what closes issue #6632,
which the coordinating session reads after merging.
The three questions in "What is left for the owner" are undecided,
and no release,
workflow dispatch,
container task or mutation task was run.

## What the issue #6633 round did, checked and left open

Done,
in this worktree,
in two commits before this record's own:

- `ae0f5c185328283302833749d1680afb98bcb432`
  (2026-11-27T18:49:08-05:00):
  `test(forbidden-strings): pin what the scan step hands over for each entry
  it leaves out`,
  the fixture's pathname rules,
  the extraction of the left-out step's body,
  eight new cases and two rewritten assertions
  (`package/forbidden-strings/cli/tests/ci_scan_scope/left_out_entry_tests.rs`,
  `folder_link_entry_tests.rs`,
  `fixture_inputs.rs`,
  `scan_step_command.rs`,
  `ci_scan_scope.rs`).
- `f2b1b09f4a7022b37d4cb7295145b82029354333`
  (2026-11-27T18:50:10-05:00):
  `fix(forbidden-strings): scan each entry the file scan leaves out by name
  and target text`,
  the left-out list in the file scan,
  the third step,
  the left-out line's new sentence,
  and `--` on both scanner calls.

The two new cases were red first:
before the workflow change the module ran 22 cases and
`9 failed, 13 passed`,
the two acceptance cases failing with

```text
a forbidden string in a left-out folder link's own name must fail the step; got status 0
stdout: Left out 1 of 2 changed entries the scanner cannot open as a file: folder links 1, submodule gitlinks 0. A link to a file, a broken link and an unreadable file stay in the scan.
Scanning 1 changed file(s) from 178a3642a71a488f9ca5c474aef684cb9cd141bd..820c570480fcd4be866c2bc7104acf043e1a76b1:
  consumer/main.txt
```

and

```text
a forbidden string in a left-out folder link's stored target text must fail the step; got status 0
stdout: Left out 1 of 2 changed entries the scanner cannot open as a file: folder links 1, submodule gitlinks 0. A link to a file, a broken link and an unreadable file stay in the scan.
Scanning 1 changed file(s) from fc596c9e361b62039d56e0a094a0c03dd4eca0fd..0f15e462d1c0aadf3e561648824f7e0e2b78064a:
  consumer/main.txt
```

The 13 that passed there are today's coverage:
the clean folder link,
the link to a file read through,
the broken link's read error,
the submodule entry's own left-out line,
both pull-request cases,
and the five push-scope cases.

Nine plants show each control can fail,
each written into the workflow file,
run,
and restored from the committed file,
so no plant was ever committed
and the workflow file's sha256 matched the committed one after every run
(`098a13b002e5f3749590724645557f64b16904d6bd2c3d80822e41c82eb386f9`).
The test module ran 22 cases in each run:

- leaving a folder link in the positional list:
  8 cases failed,
  among them the clean folder link,
  the folder-links-only delta,
  and the link whose target folder holds a forbidden string,
  each printing `must pass` with `got status 1`;
- leaving every `120000` entry out:
  `broken_link_still_ends_the_step_with_its_read_error` and
  `changed_link_to_a_file_still_reads_its_target_content` failed,
  the broken link answering `got status 0`;
- dropping the submodule gitlink's record:
  only `push_of_a_submodule_gitlink_whose_own_name_is_forbidden_ends_the_step_1`
  failed,
  with `got status 0`;
- dropping a folder link's record:
  6 cases failed,
  among them both pull-request left-out cases,
  the special-byte case and both acceptance cases;
- unquoting the name handed to `--name-path`:
  `push_hands_over_a_link_whose_name_holds_a_space_a_newline_and_a_comma`
  failed with `got status 2`;
- dropping the sentence that says what scans the left-out entries:
  the two cases that assert it failed,
  printing `the file-scan step must say what scans the entries it left out`;
- reading the path a link points at instead of its Git object:
  9 cases failed,
  each printing `cat: ./<link>: Is a directory`;
- ignoring the left-out step's findings:
  the five cases that expect a left-out entry's finding failed,
  each with `got status 0`;
- renaming the left-out step:
  8 cases failed,
  the declaration case printing
  ``the workflow must declare its `- name: Scan the entries left out of the file scan` step``.

One further plant replaced the Git blob read with `readlink`:
21 of 22 cases passed and only the declaration case failed,
which is the honest result,
because both give the same bytes on a fresh checkout.
The declaration case is what pins the mechanism.

Checked in `package/forbidden-strings/cli`:

- `cargo fmt --check`:
  the first run reported one diff in `folder_link_entry_tests.rs`
  (a line break this round had removed),
  `cargo fmt` fixed it and the next run exited 0;
- `cargo check --locked --offline --all-targets`:
  exit 0;
- `cargo clippy --locked --offline --all-targets -- -D warnings`:
  the first run exited 101 on `clippy::implicit-return` at
  `package/forbidden-strings/cli/tests/ci_scan_scope/scan_step_command.rs:115`
  (the new helper's tail expression),
  a `return` fixed it and the next run exited 0;
- `cargo nextest run --locked --offline --test ci_scan_scope`:
  `Summary [   0.976s] 22 tests run: 22 passed, 0 skipped`;
- the package's own task,
  `mise run //package/forbidden-strings/cli:test`:
  `Summary [  23.281s] 381 tests run: 381 passed, 0 skipped`,
  exit 0.
  The 6632 round's branch ran 373,
  and this round adds eight cases.

`monochromatic-lint` reports no finding on any file this round changes:
the workflow file,
the five files under
`package/forbidden-strings/cli/tests/ci_scan_scope/`,
and this record.

What no check covers,
and what this round did not run:

- No container task,
  no mutation task and no release task ran,
  and nothing in the scanner changed,
  so no release is owed for this work.
- No workflow was dispatched and no run of the workflow exists for these
  changes;
  the first push run on `main` after the merge is what shows the new step in
  a live run,
  and issue #6633 closes on a green one.
- The plant runs and the delta measurement used the released `0.6.2`
  binary and stock Git,
  because the workflow downloads exactly that binary and the runner has
  stock Git;
  no scanner was built from this branch.

Merged and re-checked.
`origin/main` moved twice while this round ran,
so it was merged twice:
`e032823a1e` (2026-11-27T18:49:55-05:00) in the merge commit
`02e9db640f` (2026-11-27T18:57:44-05:00),
and then `25dec7b462` (2026-11-27T18:58:21-05:00) in
`e76afec9a2` (2026-11-27T18:58:42-05:00).
Neither merge touched the workflow file or any test file this round changes.

After the second merge every check above ran again,
in `package/forbidden-strings/cli`,
and each printed the same result:
`cargo fmt --check` exit 0,
`cargo check --locked --offline --all-targets` exit 0,
`cargo clippy --locked --offline --all-targets -- -D warnings` exit 0,
`cargo nextest run --locked --offline --test ci_scan_scope`
with `Summary [   1.079s] 22 tests run: 22 passed, 0 skipped`,
and `mise run //package/forbidden-strings/cli:test`
with `Summary [  26.566s] 381 tests run: 381 passed, 0 skipped`.
`monochromatic-lint` again reported no finding on any changed file,
the workflow file and this record among them.

The delta measurement was run again on `82d6a89596..e032823a1e`,
the tip this branch merged first,
with the same rules,
stand-in and binary:

```text
Left out 139 of 8159 changed entries the scanner cannot open as a file: folder links 139, submodule gitlinks 0. The next step scans their own names and, for each link, its stored target text. A link to a file, a broken link and an unreadable file stay in the scan.
Scanning 8020 changed file(s) from 82d6a8959655c863e4a28579139456f609d5d615..e032823a1ed89c182d7895688186051728483ed5:
Scanning 139 left-out entries: each one by its own name, and each link by its stored target text:
```

All four steps exited 0,
neither stream held a line with `rule=` or `read error`,
and the left-out step left 140 files in its target-text folder again.
Those figures differ from the earlier run's only by the one entry and the one
file `main` gained between the two tips.

The coordinating session merges this branch,
and issue #6633 closes on a green push run on `main` that prints the new
left-out line,
which no run can show before that merge.
No workflow was dispatched.

Open:

- The two owner questions in "What is left for the owner" item 2 stay
  open:
  whether a link to a file should also have its stored target text scanned,
  and whether the whole-tree arm should list the index's links.
- The workflow file's left-out step is one scanner call over every left-out
  entry,
  so a push whose file scan fails reports only the file findings first;
  the left-out step runs on the next push.
  That ordering is the runner's,
  and no test covers a live job's step skipping.

## What the issue #6640 round did, checked and left open

Done,
in this worktree,
in three commits before this record's own:

- `398621c31dfd1af409c0731fcbd1dd3934280ce6`
  (2026-11-27T20:44:11-05:00):
  `test(forbidden-strings): pin that each link's stored target text and each
  gitlink is scanned`,
  the new whole-tree cases,
  the kept-link cases,
  the fixture that runs every declared step body in workflow order,
  and the rename of `left_out_entry_tests.rs` to `link_target_text_tests.rs`.
- `2d770061bd84578a0806a33693eeb707b4bca2cc`
  (2026-11-27T20:47:08-05:00):
  `fix(forbidden-strings): scan each changed link's stored target text, and the
  whole tree's links and gitlinks`,
  the two later steps,
  the index listing,
  and `if: ${{ !cancelled() }}` on both.
- `257306395ca2df1a6d14b7665e8bb25ecb84e2a8`
  (2026-11-27T20:49:23-05:00):
  `test(forbidden-strings): read the status check the workflow declares instead
  of assuming it`,
  without which a plant of the check's removal passed all 33 cases.

### The reds before the cure

With the tests in place and the workflow as `main` had it,
`cargo nextest run --locked --offline --test ci_scan_scope`
printed `33 tests run: 15 passed, 18 failed, 0 skipped`.
The cases the requirement names failed with:

```text
a forbidden string in a kept link's stored target text must fail the step; got status 0
stdout: Scanning 2 changed file(s) from fd865ae8eee2682de1e67e6d903aadb1f56006ac..38ee9422d7aed773fdcc2d063b8fe5315597c8b3:
  consumer/main.txt
  consumer/uses/lib
```

```text
a forbidden string in a whole-tree-scanned link's own name must fail the step; got status 0
stdout: ::notice::No validated scan baseline; scanning the current tree.
```

```text
a forbidden string in a whole-tree-scanned link's stored target text must fail the step; got status 0
stdout: ::notice::No validated scan baseline; scanning the current tree.
```

```text
a forbidden string in a whole-tree-scanned gitlink's own name must fail the step; got status 0
stdout: ::notice::No validated scan baseline; scanning the current tree.
```

```text
the stored target text must report its own line under the same link name
stdout: Scanning 2 changed file(s) from b7ac7bcce362204bab3ca131d9604f9b5af54bef..5995a406ca0a486580f6e1d44756096ba0e6bad5:
  consumer/main.txt
  consumer/uses/lib
```

and the pull-request case the same way:

```text
the pull-request arm must report a forbidden string in a kept link's stored target text; got status 0
stdout: Scanning 2 changed file(s) from origin/main...HEAD:
  consumer/main.txt
  consumer/uses/lib
```

`whole_tree_scan_still_reports_a_regular_file` passed on `main`,
which is the control that the whole-tree arm's walk still has to work after
the change.

### The plants

Seven plants show each control can fail,
each written into the workflow file,
run,
and restored from the committed file,
so no plant was ever committed:

- the whole-tree arm listing nothing:
  5 cases failed,
  every whole-tree case among them;
- the delta listing only folder links:
  4 cases failed,
  the kept link's target text,
  the both-match pair,
  the pull-request case and the clean link's hand-over;
- reading the path the link points at instead of its blob:
  16 cases failed;
- dropping the whole-tree line's counts:
  3 cases failed,
  the three that assert what the arm says it handed over;
- unquoting the name handed to `--name-path`:
  2 cases failed,
  one per arm,
  on the space and the comma;
- dropping `if: ${{ !cancelled() }}`:
  `push_of_a_link_whose_content_and_stored_target_text_both_match_prints_both`
  failed,
  and no other case did;
- leaving every link out of the positional list:
  5 cases failed,
  among them `changed_link_to_a_file_still_reads_its_target_content`,
  `broken_link_still_ends_the_step_with_its_read_error` and the clean link's
  read-through.

The workflow file's sha256 after every plant was
`5e85cacd3f0639d16256745f638e0b7324abaaa55d97fcd75cd67ccf65a32d62`,
the committed file's own.

### Checked

In `package/forbidden-strings/cli`:

- `cargo fmt --check`:
  exit 0.
- `cargo check --locked --offline --all-targets`:
  exit 0.
- `cargo clippy --locked --offline --all-targets -- -D warnings`:
  the first run exited 101 on `clippy::shadow-reuse` at
  `package/forbidden-strings/cli/tests/ci_scan_scope.rs:206`
  (a `body` binding shadowed by the match arm's own `body`),
  a rename fixed it and the next run exited 0.
- `cargo nextest run --locked --offline --test ci_scan_scope`:
  `Summary [   1.286s] 33 tests run: 33 passed, 0 skipped`.
- the package's own task,
  `mise run //package/forbidden-strings/cli:test`:
  `Summary [  20.823s] 392 tests run: 392 passed, 0 skipped`,
  exit 0.
  The 6633 round's branch ran 381,
  and this round adds eleven cases.

`monochromatic-lint` reports no finding on any file this round changes:
the workflow file,
the six files under
`package/forbidden-strings/cli/tests/ci_scan_scope/`,
and this record.
Its `markdown/semantic-line-breaks` rule rewrote this round's new prose once
(`--fix`),
because every comma in a paragraph needs a line break after it.

### Merged and re-checked

`origin/main` was 28 commits ahead when this round finished,
so it was merged in `5edf62d28d` (2026-11-27T21:20:18-05:00).
Neither the workflow file nor any test file this round changes was touched by
the merge,
and the index still holds 139 links and 0 gitlinks.

After the merge every check above ran again,
in `package/forbidden-strings/cli`,
and each printed the same result:
`cargo fmt --check` exit 0,
`cargo check --locked --offline --all-targets` exit 0,
`cargo clippy --locked --offline --all-targets -- -D warnings` exit 0,
`cargo nextest run --locked --offline --test ci_scan_scope`
with `Summary [   1.350s] 33 tests run: 33 passed, 0 skipped`,
and `mise run //package/forbidden-strings/cli:test`
with `Summary [  24.001s] 392 tests run: 392 passed, 0 skipped`.
`monochromatic-lint` again reported no finding on any changed file,
and the workflow file's sha256 is still
`5e85cacd3f0639d16256745f638e0b7324abaaa55d97fcd75cd67ccf65a32d62`.

The coordinating session merges this branch,
and issue #6640 closes on a green push run on `main` that prints the new
steps' lines,
which no run can show before that merge.
No workflow was dispatched.

### What this round did not run, and open

- No container task,
  no mutation task and no release task ran,
  and nothing in the scanner changed,
  so no release is owed for this work.
- No workflow was dispatched and no run of the workflow exists for these
  changes;
  the first push run on `main` after the merge is what shows the new steps in a
  live run,
  and issue #6640 closes on a green one.
- The whole-tree measurement used the released `0.6.2` binary and stock Git,
  because the workflow downloads exactly that binary and the runner has stock
  Git;
  no scanner was built from this branch.
- The measurement's whole-arm ratio is dominated by the `--all` walk's own
  run-to-run spread on this machine,
  so the added cost is reported as the two added steps' own times.
- The owner's veto on issue #6640's ranking (scan both,
  not either) is open,
  and so is the ranking of issue #6633's question about what a positional link
  means inside the scanner.
