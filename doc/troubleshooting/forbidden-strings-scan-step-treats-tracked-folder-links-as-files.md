# forbidden-strings 0.6.2 and the CI scan step: a tracked link to a folder reaches the scanner as a file, so every push to `main` failed from 2026-11-20

Status:
unapproved and non-final (2026-11-27T18:31:59-05:00).
The workflow side is written,
its tests pass,
and the checks were re-run after merging `origin/main`;
the owner declares completion.
The last section lists the commits,
the checks and what is open.

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

## Limits of the cure

- A link's own name was inspected before,
  by the scanner's pathname pass over the positional path
  (`package/forbidden-strings/cli/src/candidate_file_scan.rs:155-160`),
  and is not inspected now for a link the step leaves out
  (measured above:
  the name rule fired on
  `user/uses/zzqnametoken-folder-link` before the reader failed).
  The name is still inspected whenever any file inside the folder it
  points at is scanned,
  because that file's path carries the folder's name as a segment,
  and the walk never inspected a link's name at all.
  What is lost is the case where the link is the only changed entry and
  its folder name alone would have matched a rule.
- A link's target text is scanned by neither the walk nor the step.
  That was true before this change and is true after it,
  and the owner may want it changed (see below).
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
   measured above:
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
2. Whether a link's own target text should be scanned at all,
   and if so whether under the link's path or its target's.
   `--name-path` shows the mechanism exists
   (materialize the 120000 blob,
   pass the link path as the logical name),
   and costs the workflow one `git cat-file` per link plus a temporary file,
   or a scanner option that reads the link's blob itself.
   Cost of not doing it:
   a forbidden string that appears only in a link's target text is
   undetected,
   which is true today and was true before this change.
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

## What was done, what was checked, what is open

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
