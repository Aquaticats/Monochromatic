# forbidden-strings 0.6.2 and the CI scan step: a tracked link to a folder reaches the scanner as a file, so every push to `main` failed from 2026-11-20

Status:
unapproved and non-final (2026-11-27T18:53:04-05:00).
Two rounds wrote this record:
the one merged as `fc1c84b58e`,
which stopped the read errors,
and the one for issue #6633,
which makes the steps inspect what they leave out.
The owner declares completion.
The last two sections list the commits,
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

## Limits of what the steps inspect now

- A link to a file still gets no scan of its stored target text.
  It stays in the positional list,
  where the scanner reads the content behind it,
  and its stored target text is a path,
  so scanning it would report one problem under two labels and hand the same
  name to the scanner twice.
- The whole-tree arm inspects neither a link's name nor any target text,
  because it runs `--all`,
  whose walk leaves `Mode::SYMLINK` and `Mode::COMMIT` out.
  That was true before this round and is true after it.
  A link that entered before a baseline tree was validated is covered by the
  delta scan only until a whole-tree run replaces that baseline.
- The runner stops the job at the first failing step,
  so a push whose file scan finds something does not run the left-out step,
  and a finding there appears on the next run.
  Each step reports its own exit 1 and its own findings.
- The left-out step reads each target text from the tree's blob,
  so it is right even when a working-tree link differs from the tree.
  It needs that blob to be present:
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
- The left-out step's scan is one scanner call,
  so a rule-file error ends it for every left-out entry at once;
  a per-file matcher fault still becomes that file's finding through the
  scanner's own `catch_unwind` boundary.
- The left-out list and the materialized target texts stay in `$RUNNER_TEMP`,
  where the runner discards them with the job,
  like the rules file and the two lists beside them.
- The target texts are not removed by the step:
  no deletion runs over a path this step builds,
  which `AGENTS.md` rule `RLW` keeps free of variables.

## Limits of the cure

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
2. A link's own target text is now scanned for a link the file scan leaves
   out,
   and not for a link to a file,
   which stays in the positional list and is read through as before.
   Two choices remain with the owner:
   whether the second kind should be scanned twice,
   once read through and once by its stored text,
   which costs one more pair per such link and two findings labels for one
   problem;
   and whether the whole-tree arm should list the index's links at all,
   which is a coverage change to the full scan rather than a fix for what
   the delta scan leaves out.
3. Whether the 139 `uses/` links should exist as links at all,
   given that a content scan checks neither a link's name nor its target
   text,
   and only a file's own path segments carry a folder's name into the scan.
   This record does not decide that;
   it only stops the read errors.

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
