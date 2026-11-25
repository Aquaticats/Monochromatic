# rclone bisync with --use-server-modtime records the source mtime in its baseline but compares the S3 LastModified, so every uploaded file reads back as changed and a file edited across two runs loses its name to a false conflict

A file edited locally during two consecutive hourly bisync runs ends up with its original name
deleted on **both** sides,
 replaced by `<name>.conflict1` (the remote's copy) and `<name>.conflict2` (the local copy).
No data is lost,
 but the file the user was working on no longer exists under the name they use,
 and an application that reopens it by path sees nothing.

This is not a deletion,
 not Seafile,
 and not a Garage fault.
It is rclone bisync's documented conflict resolution firing on a conflict that is half phantom.

Observed against the hourly `rclone-bisync-garage.service` user unit described in
[rclone-double-encode-phantom-keys.md](rclone-double-encode-phantom-keys.md),
 which is where `--use-server-modtime` was introduced,
 for the reason recorded there:
 dropping the per-object HEAD.
Six occurrences,
 always on whichever files were being actively edited that session,
 never on the roughly 9900 untouched files in the same tree.
Paths in this document are synthetic;
 the mechanism does not depend on them.

## Symptom

From one run,
 with `-v`:

```text
INFO  : Path1 checking for diffs
INFO  : - Path1    File changed: time (newer) - DIR/file-a.html
INFO  : Path1:    4 changes:    0 new,    4 modified,    0 deleted
INFO  : Path2 checking for diffs
INFO  : - Path2    File changed: size (larger), time (newer) - DIR/file-a.html
INFO  : Path2:    6 changes:    1 new,    5 modified,    0 deleted
INFO  : Applying changes
INFO+2: - WARNING  New or changed in both paths       - DIR/file-a.html
INFO+2: - Path1    Renaming Path1 copy                - garage:.../DIR/file-a.html.conflict1
INFO  : DIR/file-a.html: Copied (server-side copy) to: DIR/file-a.html.conflict1
INFO  : DIR/file-a.html: Deleted
INFO+2: - Path2    Renaming Path2 copy                - /local/.../DIR/file-a.html.conflict2
INFO  : DIR/file-a.html: Moved (server-side) to: DIR/file-a.html.conflict2
```

The asymmetry in those two diff lines is the whole finding.
Path2 changed `size (larger), time (newer)`:
 a real edit.
Path1 changed `time (newer)` with **no size change**:
 the remote bytes are identical to what bisync already had.
Only one side really changed,
 but bisync counted two and took the conflict branch.

## Root cause

### The same remote object reports two different mtimes

`--use-server-modtime` selects the S3 `LastModified` instead of the mtime rclone stores in
object metadata on upload.
For a byte-identical object those disagree:

```bash
rclone lsl garage:.../DIR/                        # metadata mtime (X-Amz-Meta-Mtime)
#     8362 2026-08-11 20:53:46.620927602 file-b.md
rclone lsl garage:.../DIR/ --use-server-modtime   # S3 LastModified
#     8362 2026-08-11 21:01:58.234000000 file-b.md
```

Same size,
 same bytes,
 two timestamps eight minutes apart.
The nanosecond precision of the first identifies it as the stored source mtime;
 `LastModified` is second-granularity and equals the moment rclone uploaded the object.

### bisync's baseline records the mtime it did not compare with

The listing under `~/.cache/rclone/bisync/*.path1.lst`,
 written at 21:02 by the very run that uploaded that object at 21:01:58:

```text
-     8362 - - 2026-08-12T00:53:46.620927602+0000 "DIR/file-b.md"
```

That is the source mtime (20:53:46 local),
 not the `LastModified` (21:01:58) the next run will read.

The precise claim,
 and no broader:
 **for a file bisync has just uploaded,
 it records the source's mtime in the baseline rather than re-reading the destination's
`LastModified`.**
Objects bisync did not upload,
 such as ones it created by server-side copy,
 record `LastModified` in the baseline and match on the next run;
 those show no phantom.
So the defect is specific to the upload path,
 which is why it tracks edited files exactly.

Consequence:
 every file bisync uploads is guaranteed to read back as
`Path1 File changed: time (newer)` on the following run,
 with no content change.

### Why that usually costs nothing, and occasionally costs the filename

A phantom alone is one-sided.
Path1 appears changed,
 Path2 does not,
 so bisync queues a copy from remote to local.
No bytes move.
The sizes match and only the times differ,
 so rclone satisfies the queued copy by stamping the local file's mtime with the
`LastModified` it compared against:

```text
INFO  : - Path1    Queue copy to Path2       - /local/.../DIR/file-a.html
INFO  : DIR/file-a.html: Updated modification time in destination
```

The two sides then agree and the phantom clears itself.
That silent self-heal is why this does not fire on every file every hour.
It is also why an unhealed phantom costs almost nothing on its own:
 a run that heals one transfers no data,
 which is what makes the destructive case easy to miss until it lands.

An earlier draft of this document said bisync downloads the file and stamps the downloaded
copy.
It does not.
The distinction matters for anyone reasoning about cost or about bandwidth-triggered alerts.

It becomes destructive only when the local file **also** changes before that heal runs:

1.  Edit the file.
     Run N uploads it.
     Remote `LastModified` becomes run N's clock.
2.  Keep editing.
     The local file changes again,
     genuinely.
3.  Run N+1 sees Path1 newer (phantom) and Path2 newer and larger (real).
    Both sides changed,
     so bisync applies `--conflict-resolve` (default `none`),
     renames both copies,
     and the original name exists on neither side.

An editing session spanning two consecutive hourly runs meets that condition every time.

### The phantom also reverses a local deletion

Deleting a file locally while its remote counterpart carries a phantom does not delete it.
bisync sees a change on Path1 and a deletion on Path2,
 resolves in favour of the change,
 and copies the remote object back down.
The local deletion is undone and never reaches the remote.

One run supplies both halves of the comparison at once.
Eight files had been deleted locally in the same directory:
 four whose remote objects bisync had uploaded,
 and four whose remote objects bisync had created by server-side copy while resolving an
earlier conflict.
Only the uploaded four carried a phantom:

```text
INFO  : - Path1    File changed: time (newer) - DIR/file-c.md.conflict2
INFO  : - Path2    File was deleted          - DIR/file-c.md.conflict1
INFO  : - Path2    File was deleted          - DIR/file-c.md.conflict2
INFO  : - Path1    Queue delete              - garage:.../DIR/file-c.md.conflict1
INFO  : - Path1    Queue copy to Path2       - /local/.../DIR/file-c.md.conflict2
```

Same directory,
 same run,
 same local deletion,
 opposite outcomes.
The server-side-copied object took the delete;
 the uploaded object was restored to the local disk.
The two differ in nothing but provenance,
 which makes this a natural experiment on the upload-path claim rather than another
observation consistent with it.

Operational consequence:
 conflict artifacts cannot be cleaned up locally.
Deleting `<name>.conflict1` or `<name>.conflict2` on one side only leaves whichever copy the
phantom protects,
 and the next run puts it back.
Delete on both sides in the same window,
 then confirm both listings are empty.

Confirmed rather than assumed:
 four artifacts deleted from both sides between two runs were reported by the next run as
`File was deleted` on Path1 and Path2 together,
 with no copy queued in either direction and no resurrection.
The same run logged the phantom on two unrelated files and healed both without a transfer,
 so the deletions held while the phantom was demonstrably still active.

## Verification

Versions:
 rclone `v1.75.0` via the mise shim
 (`rclone version`;
 the mise shim resolves to `rclone-v1.75.0-linux-amd64`,
 so this is a later rclone than the `v1.74.3` recorded in the sibling document),
 Garage `dxflrs/garage:v2.3.0`,
 single node,
 bucket `files`.
Tree under sync:
 9948 files,
 77.9 GiB.

### The two clocks disagree on an unchanged object

The two `rclone lsl` invocations above,
 differing only in `--use-server-modtime`,
 against the same object.
Read-only.

### The baseline disagrees with what the next run will read

```bash
grep 'DIR/file-b.md"' ~/.cache/rclone/bisync/*.path1.lst
rclone lsl garage:.../DIR/ --use-server-modtime | grep file-b.md
```

The first prints the source mtime,
 the second the upload time.
Any difference predicts a phantom on the next run.

### Standing prediction, confirmed

A byte-identical,
 locally untouched file whose baseline and `LastModified` disagree
**must** be logged as `Path1 File changed: time (newer)` by the next run,
 with no size change,
 and must not conflict.
If a run logs no such line for such a file,
 this analysis is wrong and should be reopened.
This doubles as the positive control:
 the phantom has to be observable on a file nobody edited,
 otherwise the mechanism above is not what is happening.

The next scheduled run logged six such lines,
 none of them a conflict.
Two were files nobody had opened or edited in that session,
 which is the positive control the prediction demanded:
 the phantom appears on untouched files,
 so it is a property of how the baseline is written and not a side effect of editing.
The remaining four are the deletion case described above.

The prediction stays in this document as written.
It is cheap to re-run against any later log and is the fastest way to find out that something
about the upload path has changed.

### The fix reproduced and proved on a throwaway fixture

Run against two local directories under `~/temp/agent`,
 never against the real bucket.
Twenty files,
 so that changing one stays under bisync's "all files were changed" safety abort;
 an earlier single-file fixture tripped that guard and proved nothing.
A genuine conflict is created by changing `f3.txt` on **both** sides,
 with the Path2 copy given the later mtime by `touch -d`:

```text
DEFAULT (current behaviour, no flag)
  files named f3*: f3.txt.conflict1 f3.txt.conflict2
  f3.txt: ABSENT (original name destroyed)

WITH --conflict-resolve newer
  files named f3*: f3.txt f3.txt.conflict1
  f3.txt content: FROM-B-newer
```

The first block reproduces the reported symptom exactly.
The second shows the flag doing the two things that matter:
 the original filename survives and holds the newer bytes,
 and the losing copy is retained rather than deleted.

### The cost that motivated the flag

Listing all 9948 objects,
 warm,
 `--fast-list --checkers 32`:

- with `--use-server-modtime`:
   3.08 s
- without it (per-object HEAD):
   19 min 27 s

Both returned the same object count,
 so the difference is the per-object HEAD and nothing else.
That is a factor of roughly 380.
The flag is not a micro-optimisation,
 and the sibling document's "took minutes" understates it.

This settles the ranking below rather than merely informing it.
The timer fires hourly,
 so a 19-minute listing would spend a third of every hour in a run that currently takes
seconds,
 and `TimeoutStartSec=3000` leaves under 31 minutes of margin before a stalled run is reaped.
Any fix that reintroduces the per-object HEAD pays that on every run,
 forever,
 to correct a comparison that a single flag already resolves correctly.

## Candidate fixes

Ranked for an operator whose priority is never silently losing the newest version of a file
they are actively editing.

Option 1 is now deployed,
 chosen by the operator after the measurement above.
Adding `--conflict-resolve` does **not** require a `--resync`,
 unlike a filter change;
 verified on a throwaway fixture by adding the flag mid-stream between two runs,
 which completed normally with exit 0.

1.  **`--conflict-resolve newer`,
     on its own.**
    Cheapest correct-in-practice fix,
     keeps the 3.1 s listing.
    Leave `--conflict-loser` at its default,
     which `rclone bisync --help` gives as `num`:
     the winner then keeps the original filename and only the loser is renamed to
     `<name>.conflict1`.
    That is the property that matters here,
     because the reported harm is the filename disappearing,
     not the bytes going missing.
    Pro:
     resolves the false conflict in favour of the genuinely newer local edit,
     preserves the working filename,
     and still keeps the losing copy on disk.
    Con:
     it resolves a conflict that should never have been raised,
     so it is a guard rather than a cure,
     and it leaves a `.conflict1` file behind to be cleaned up.
    Do **not** pair it with `--conflict-loser delete`:
     that buys tidiness by destroying the copy that would be the fallback if `newer` ever
     chooses wrong.
    Ranked above dropping the flag because it costs nothing per run and the phantom cannot
    make the remote win in a single-writer setup:
     the remote's inflated time is the upload of an edit that is already in the local file,
     so a later local edit always carries a later mtime.
    That reasoning depends on there being one writer,
     and would need rechecking before a second machine writes to this bucket.
2.  **Drop `--use-server-modtime`.**
    The only option that removes the phantom at its source rather than resolving it.
    Pro:
     baseline and comparison then use the same clock,
     and no false conflict is ever raised.
    Con:
     reintroduces the per-object HEAD measured above,
     and the sibling document notes the phantom-key filter `- **‛**` became load-bearing
     precisely because `--use-server-modtime` removed the old HEAD-time self-heal;
     that interaction needs rechecking before changing it.
    Ranked below the first because correctness is bought with a per-run cost of minutes,
     and the first option already prevents the observed loss.
3.  **`--conflict-resolve path2`,
     local always wins.**
    Pro:
     simple,
     and correct for a single-writer setup.
    Con:
     silently discards genuine remote-side changes,
     so it converts a visible mess into an invisible one the moment a second machine writes.
    Ranked below `newer` only because `newer` degrades more gracefully under that same event.
4.  **`--compare size,checksum`.**
    Pro:
     immune to clock semantics entirely.
    Con:
     the run already reports `"SlowHashDetected": true`,
     and Garage multipart ETags are not plain MD5,
     so this needs proving before it is trusted.
    Ranked below the above for being unproven here,
     not for being wrong in principle.
5.  **Leave it and keep restoring from git.**
    Ranked last:
     it works only because the affected files happen to be in a git repository that lives
     outside the synced tree,
     which is a property of one directory and not of the sync.

Rejected outright:
 `--conflict-suffix` changes,
 which only rename the wreckage.

## What this analysis got wrong on the way

`--conflict-resolve newer` was first rejected on the grounds that the phantom would make the
stale remote copy win.
That was wrong,
 and the log disproves it.
The reasoning had used the `LastModified` of the `.conflict1` object,
 21:01:55,
 but that object was created by the server-side copy **during** the run being diagnosed.
What Path1 actually presented at diff time was the original object's upload time from the
previous run,
 20:01:57,
 against a local mtime of 20:52:28.
Local was genuinely newer,
 and `newer` would have chosen correctly.

The general lesson is narrow and worth keeping:
 when reconstructing a conflict,
 the timestamps on the conflict artifacts are stamped by the resolution itself and are not
evidence about the state that triggered it.
Read the previous run's upload line instead.

## Upstream report draft (not filed)

Reproduction checked at 2026-11-25T18:02:38-05:00 by the coordinating session,
on a throwaway Garage v2.3.0 in podman with rclone v1.76.0:
the steps as first drafted failed at step 2 with
`Empty prior Path1 listing. Cannot sync to an empty directory` (exit status 7),
so step 1 now creates a seed file;
with that change every step produced the output the draft states,
except that Garage reported the upload time to the millisecond (`23:02:16.022`),
which step 3 now says.
Output of both attempts:
`~/temp/agent/garage-bisync-on-change-grilling-2026-11-25/throwaway-garage-test/keep/upstream-draft-reproduction-as-written-and-corrected.txt`.

Status:
 agent-drafted 2026-11-25T17:48:32-05:00,
 never filed,
 never posted,
 never reviewed,
 not approved by the owner.
Nothing was created on GitHub,
 the rclone forum or anywhere else while researching this:
 only searches and reads ran.
The issue body under [Draft issue body](#draft-issue-body) is a proposal for the owner to
approve,
 rewrite or discard,
 not a record of anything sent.

### What the upstream search found

No existing report and no existing fix.
No rclone issue,
 pull request,
 commit,
 changelog entry or forum thread describes this upload-path behaviour,
 and the code that writes the baseline is unchanged between v1.76.0,
 the latest release,
 and the default branch.

The behaviour above was recorded on v1.75.0 (see Verification).
The queries,
 line numbers and code below are against v1.76.0,
 which `rclone version` on this machine also reports now.
`WriteResults` (`cmd/bisync/queue.go`) and the listing updates (`cmd/bisync/listing.go`) are
 the same in both releases apart from the sub-second `put` rule PR #10012 removed and
 unrelated refactors,
 so the line numbers below describe the behaviour that was recorded.

Every query below was run once on 2026-11-25 against rclone's tracker,
 its forum and its changelog.
Counts are what each tool reported,
 and every hit named as relevant was opened and read.

#### How to read the counts

`gh search issues` and `gh search prs` send a multi-word query as one quoted phrase:
`GH_DEBUG=api` shows `bisync use-server-modtime` going out as
`q=( "bisync use-server-modtime" )`,
which matches nothing.
Splitting the terms into separate arguments sends them unquoted and does return results,
but GitHub then matches loosely:
`bisync use-server-modtime` by that route returns #118,
whose body and comments never mention the flag.
So counts from `gh search` say which reports exist at all,
never that a hit is about this behaviour.
Phrase-precise counts come from
`gh api --method GET search/issues --raw-field q='repo:rclone/rclone ...'`
with the phrase quoted inside the query string.

#### GitHub: issues and pull requests

Phrase queries through the search API,
both states unless noted:

- `bisync "use-server-modtime"`:
   3 hits,
   all read,
   none about the upload path:
   #9150 (one log line carrying the flag, plus an unrelated bisync critical error on OneDrive),
   #8629 (a comment asks whether another backend could use the flag),
   PR #8941 (a backend review checklist mentioning bisync).
- `"use-server-modtime"` alone:
   33 hits;
   the three above are the only ones that also mention bisync.
- `bisync "time (newer)"`:
   5 hits,
   all log excerpts or unrelated reports:
   PR #10163,
   #8841,
   #7555,
   #3232,
   PR #7716.
- `bisync "Updated modification time in destination"`:
   1 hit,
   #8101 (`--refresh-times` on an encrypted OneDrive remote),
   a different defect.
- `"Updated modification time in destination"` alone:
   19 hits,
   the same #8101 being the only bisync one.
- `bisync "baseline"`:
   7 hits,
   none about listing baselines for uploads:
   #9897,
   #9416,
   #2697,
   #9939,
   #9961,
   #9620,
   #8873.
   #9939 is covered under [adjacent reports](#adjacent-reports-and-why-none-of-them-is-this-defect).
- `bisync "phantom"`:
   3 hits:
   #9622,
   #10012,
   #8886.
   Only PR #10012 is bisync,
   and it fixes a different listing-modtime rule.
- `bisync LastModified`:
   0 hits.
- `bisync "server-modtime"`:
   7 hits,
   the same three as above plus #10236,
   #10163,
   #8886 and #7282,
   all backend proposals or feature threads where the flag's name appears in a comment,
   none a report of this behaviour.
- `"use-server-modtime" in:title`:
   2 hits,
   #3629 (a 2019 docs fix) and #2214 (the flag's introduction in 2018).

Loose queries through `gh search`,
listed for completeness,
each term a separate argument,
open plus closed counts in parentheses:

- `bisync use-server-modtime`:
   12 (4 open, 8 closed);
   none about the upload path.
- `bisync modtime baseline`:
   2 (0 open, 2 closed);
   #9939 and #8873.
- `bisync phantom`:
   0.
- `bisync File changed time newer`:
   11 (2 open, 9 closed);
   all incidental log lines or unrelated reports.
- `bisync upload modtime listing`:
   10;
   none about the upload path.
- `bisync re-upload`:
   14;
   the nearest is #249,
   a feature request for syncing on filesystem events.
- `prs bisync use-server-modtime`:
   15 (1 open, 14 closed);
   none relevant.
- `prs bisync modtime`:
   28 (1 open, 27 closed);
   the adjacent ones are PR #10012 and PR #10001.

Whole-corpus sweep,
to catch reports that never use the flag's name:

- Every issue labelled `bisync` (61: 11 open, 50 closed) and every one
  with `bisync` in the title (71) was downloaded whole,
  `number`, `title`, `state` and `body`: 77 distinct items.
  None of the 77 contains `server-modtime` in its body,
  so no bisync report names the flag in its own text,
  and none describes a file reading back as changed after bisync uploads it.
  Four of the 77 mention `baseline`,
  `time (newer)` or `phantom` at all:
  #7555,
  #9416,
  #9620 and
  #9939,
  each in an unrelated sense or as a log excerpt.

#### rclone forum

Discourse search (`https://forum.rclone.org/search.json?q=...`,
read only) and a search restricted to `forum.rclone.org`:

- `bisync use-server-modtime`:
   2 topics;
   46636#27,
   where a maintainer lists `--use-server-modtime` among the S3 modtime quirks to rule out,
   and 53502#1,
   release notes.
- `"use-server-modtime"`:
   50 posts,
   one of them bisync (46636#27).
- `bisync server modtime listing`:
   21 posts,
   14 of them in bisync threads;
   nearest are 46636#7 and 54015#1.
- `bisync "Updated modification time in destination"`:
   2 posts,
   54015#1.
- `bisync "time (newer)" use-server-modtime`:
   0.
- `bisync baseline modtime upload`:
   0.
- `bisync LastModified listing`:
   0.
- Web search, restricted to the forum:
   `bisync "use-server-modtime" baseline listing modtime conflict` (11 hits),
   `bisync "File changed: time (newer)" phantom upload` (2 hits),
   `bisync uploaded file modtime changes next run server modtime` (5 hits).
   Every hit was a post of thread 46636 or 51468,
   or an unrelated thread (54100,
   53496).

#### Releases and the default branch

- `gh release list` puts v1.76.0 (2026-11-06) as the latest release,
  so there is no changelog section after v1.76.0 to search.
- v1.76.0's changelog has two bisync blocks
  (`docs/content/changelog.md:55` and `:136`);
  neither mentions an upload mtime,
  and the bug-fix list's
  "Fix listings ignoring a modtime less than 1s older" is PR #10012,
  a different rule.
- `diff -r` over `cmd/bisync` between the v1.76.0 tag and the default branch (`5e2a70f`)
  reports only `lockfile.go` and the test files as different,
  so `listing.go` and `queue.go` are identical and nothing has been fixed after the release.
- `gh api repos/rclone/rclone/commits?path=cmd/bisync/queue.go&since=2026-11-06`
  returns nothing,
  and no commit search hit mentions bisync and the flag together
  (`gh search commits --repo rclone/rclone use-server-modtime`).

#### Adjacent reports, and why none of them is this defect

- #9939
  (closed 2026-10-14 as completed by PR #10001),
  "a one-sided delete is reverted when the paths share no hash type and `--ignore-checksum` is set":
  a deleted file comes back,
  which is one of the consequences measured for this defect,
  but the cause there is a file that never enters the listings at all.
  Its reporter names forum thread 46636 as the closest match and says it did not diagnose
  the consequence they reported.
- PR #10012
  (merged 2026-10-14),
  "fix listings ignoring a modtime less than 1s older":
  `fileList.put` kept an older entry when the new time was less than a second earlier,
  so the listing recorded a time the file did not have.
  That body calls the result "a phantom change on the next run",
  the same shape of harm reached by a different route;
  it does not touch the mtime bisync takes from the source of a copy.
- #8359
  (closed 2025-02-11),
  "modtime of latest/updated file gets changed matching modtime of the previous revision",
  over SFTP and a crypt remote:
  a stale modtime propagated between machines.
  No S3,
  no `--use-server-modtime`.
- Forum thread 46636,
  "Avoiding Bisync 'times are equal' conflicts with newer?",
  a bisync user on IDrive (S3) chasing unexpected conflicts,
  with a maintainer replying:
  the diagnosis ends at modtimes an application did not update,
  and the maintainer asks the reporters to rule the flag out rather than reporting it.
- Forum thread 54015,
  "Bisync remote conflicting with itself",
  over SFTP:
  `File changed: time (older)` on the local side,
  then `Updated modification time in destination`.
  The self-heal line matches,
  the trigger does not:
  no S3, no flag,
  and the delta is older, not newer.
- Forum thread 51468,
  "Using update and use-server-modtime when syncing causes a few files to be constantly
  reuploaded":
  a plain `sync`,
  and the cause was local mtimes set in 2049.
- Commit `9f9569ad` (2026-02-26)
  "docs: note that --use-server-modtime only works on some backends" and #6977:
  the flag's documentation is scoped to `sync` and `copy`,
  and `docs/content/bisync.md` never mentions it,
  which is why nothing warns a bisync user.

### Source pointers in rclone v1.76.0

All paths are relative to the rclone repository at the `v1.76.0` tag (`e982ff3`).

The write side,
in the order the values flow:

1.  `fs/operations/operations.go:1788-1794`
    is where bisync first hears about a file it is about to copy,
    before the copy happens,
    with no destination object to describe:

    ```go
    func NeedTransfer(ctx context.Context, dst, src fs.Object) bool {
    	ci := fs.GetConfig(ctx)
    	logger, _ := GetLogger(ctx)
    	if dst == nil {
    		// ...
    		logger(ctx, MissingOnDst, src, nil, nil)
    ```

    A successful copy never reaches the logger again:
    `operations.Copy` does not call it,
    and the `s.logger` calls in `fs/sync/sync.go` report errors or directories only.
    So the result bisync records is a prediction made before the upload.
2.  `fs/operations/logger.go:203-220`
    says so in its own words,
    `Winner predicts which side (src or dst) should end up winning out on the dst.`,
    and lists the cases it cannot predict.
3.  `cmd/bisync/queue.go:109-146`
    (`WriteResults`) turns that call into two JSON result lines,
    one per side,
    and takes each side's time from that side:

    ```go
    result.IsSrc = i == 0
    result.IsDst = i == 1
    result.Flags = "-"
    if side != nil {
    	result.Size = side.Size()
    	if !b.queueOpt.ignoreListingModtime {
    		result.Modtime = side.ModTime(ctx).In(TZ)
    	}
    ```

    `result.IsWinner = result.Winner.Obj == side` (line 146) marks the source line,
    because for both a new file and a changed one the winner is the source object.
    The destination side contributes no winning entry,
    so its own time, where it has one (the pre-copy object of a changed file),
    never reaches the listing.
4.  `cmd/bisync/listing.go:521-531`
    builds the winner entries from those results,
    the destination's from the winning result's time,
    which is the source's:

    ```go
    if result.IsSrc && result.Src != "" && ... {
    	srcWinners.put(result.Name, result.Size, result.Modtime, ...)
    }
    if result.IsWinner && result.Winner.Side != "none" && ... {
    	dstWinners.put(result.Name, result.Size, result.Modtime, ...)
    }
    ```

5.  `cmd/bisync/listing.go:568`
    copies that entry into the destination's listing
    (`list.put(queueFile, new.size, new.time, ...)`),
    and `cmd/bisync/listing.go:250-293`
    writes `fi.time` verbatim into the `.lst` file.
    Nothing between steps 3 and 5 reads the destination object.

The read side:

1.  `cmd/bisync/operations.go:337-346`
    compares each side against its own prior listing,
    `b.findDeltas(fctx, b.fs1, b.listing1, b.march.ls1, "Path1")` and the same for Path2,
    where `b.march.ls1` was built during this run's march.
2.  `cmd/bisync/march.go:161-170`
    takes the time from the object as this run listed it,
    `modtime = o.ModTime(b.march.marchCtx).In(TZ)`.
3.  `backend/s3/s3.go:4293-4296`
    is what that returns under the flag:

    ```go
    func (o *Object) ModTime(ctx context.Context) time.Time {
    	if o.fs.ci.UseServerModTime {
    		return o.lastModified
    	}
    ```

    `lastModified` is the S3 `LastModified`,
    the moment of the upload,
    second-granularity,
    not the mtime rclone did not store because of the flag.
4.  `cmd/bisync/deltas.go:204-214`
    compares the two values and names the difference:

    ```go
    if b.opt.Compare.Modtime {
    	if timeDiffers(fctx, old.getTime(file), now.getTime(file), f, f) {
    		if old.beforeOther(now, file) {
    			// ... "time (newer)"
    ```

    `old` is the prior listing, so `old.getTime` is the time the write side recorded,
    which is the source mtime, and `now.getTime` is the upload time.
    The comparison is not wrong about the two numbers;
    the numbers were never on the same clock.
5.  `cmd/bisync/compare.go:271-273`
    is the only other place bisync touches the flag,
    forcing it on when modtime comparison is off,
    which is why a `--compare` without `modtime` shows no phantom:
    `cmd/bisync/queue.go:194` then sets `ignoreListingModtime`,
    and no time reaches the listing.

One piece of code already does what a fix would need:
 `cmd/bisync/listing.go:731-762`
 (`recheck`) lists both sides again after an uncertain result and records each side's own
 `obj.ModTime(ctxRecheck)` (line 759).
It runs only for files bisync could not predict,
 via `filterRecheck.AddFile` at line 572 and the rename branches,
 never for a successful upload.

A reading of the code,
 offered as a hypothesis and not as a measurement:
 the `.lst` line for a conflict artifact carries a correct time because the artifact is made by
 a server-side move of an object this run already listed
 (`operations.MoveFile` in `cmd/bisync/resolve.go:320`),
 and `cmd/bisync/listing.go:596-625` copies that entry to the new name after
 line 568 has refreshed the old name from the march listing.
The upload path has no equivalent refresh.
The document's own measurement,
 the four server-side-copied objects that took their delete while four uploaded ones were
 restored,
 is the evidence; this paragraph only says where the difference sits.

### Draft issue body

The text in the block is a draft for rclone's tracker.
It is not filed,
 and it is not approved.
Paths,
 remote names,
 bucket names and timestamps in it are synthetic;
 the indented lines become code blocks when pasted into an issue.

```text
Title: bisync: with --use-server-modtime, a file bisync uploads is recorded at the source
mtime, so the next run reports it as changed

### What is the problem you are having with rclone?

With --use-server-modtime against an S3 remote, bisync records the mtime of the local source
in its listing for a file it has just uploaded, but the next run compares the object's S3
LastModified. The two never agree for an uploaded file, so every file bisync uploads is
logged as changed on the following run even though nothing changed.

The damage is not the extra line. It is what happens when the local file also changes before
that phantom is healed:

- A run that sees the phantom alone queues a copy remote to local, moves no bytes, and
  stamps the local mtime to the remote's LastModified, so the phantom clears itself.
- If the local file changes again before that run, both sides look changed, so a conflict
  applies. With no --conflict-resolve there is no winner and both copies are renamed:
  <name>.conflict1 and <name>.conflict2, with the original name left on neither side. With
  --conflict-resolve newer the winner keeps the name and only the loser is renamed.
- A local delete made while the phantom is live is reverted: bisync prefers the "changed"
  remote object over the deleted local file and copies it back.

### What is your rclone version

v1.76.0 (latest release), Linux amd64. Also seen on v1.75.0.

### Which cloud storage system are you using?

S3, a self-hosted S3-compatible server (reproduced on Garage v2.3.0). The mechanism does not
depend on the provider: it needs a backend where --use-server-modtime makes ModTime return
the server's upload time instead of the stored mtime.

### The command you were trying to run

    rclone bisync sandbox:sandbox-bucket/notes /srv/sync-test/notes \
      --use-server-modtime --workdir /srv/sync-test/bisync-workdir -v

### Minimal reproduction

1. Prepare an empty bucket path and a local directory holding one seed file, then resync
   (bisync refuses to start from two empty sides:
   "Empty prior Path1 listing. Cannot sync to an empty directory"):

    mkdir -p /srv/sync-test/notes
    printf 'seed\n' > /srv/sync-test/notes/seed.txt
    rclone bisync sandbox:sandbox-bucket/notes /srv/sync-test/notes \
      --use-server-modtime --workdir /srv/sync-test/bisync-workdir --resync -v

2. Create one file and run again, so bisync uploads it:

    printf 'one\n' > /srv/sync-test/notes/a.txt
    rclone bisync sandbox:sandbox-bucket/notes /srv/sync-test/notes \
      --use-server-modtime --workdir /srv/sync-test/bisync-workdir -v

3. Compare what the listing recorded with what the server will report:

    grep 'a.txt"' /srv/sync-test/bisync-workdir/*.path1.lst
    rclone lsl sandbox:sandbox-bucket/notes --use-server-modtime | grep a.txt
    stat -c %y /srv/sync-test/notes/a.txt

   Path1 is the remote. The listing line carries the local file's mtime, to the
   nanosecond; the server reports the upload time (to the millisecond on Garage v2.3.0).

4. Run again without touching the file:

    rclone bisync sandbox:sandbox-bucket/notes /srv/sync-test/notes \
      --use-server-modtime --workdir /srv/sync-test/bisync-workdir -v

### Expected behaviour

After an upload, the listing records the mtime the next run will compare against, which under
--use-server-modtime is the object's LastModified. The fourth step reports no changes.

### Actual behaviour

Step 2 records the source mtime. Step 4 reports:

    INFO  : - Path1    File changed: time (newer) - a.txt
    INFO  : Path1:    1 changes:    0 new,    1 modified,    0 deleted
    INFO  : - Path1    Queue copy to Path2       - /srv/sync-test/notes/a.txt
    INFO  : a.txt: Updated modification time in destination

No size change, no hash change, no transfer: the only difference is the clock the two runs
used.

### Where it comes from

The logger bisync installs on sync reports a file before the copy, not after it.
operations.NeedTransfer logs MissingOnDst with no destination object
(fs/operations/operations.go:1788-1794), and a successful operations.Copy never reaches the
logger again, so fs/operations/logger.go:203-220 describes its own output as a prediction of
what the destination will look like.

bisync then takes the time off whichever side it is logging
(cmd/bisync/queue.go:132-136) and puts the winning result's time into both listings
(cmd/bisync/listing.go:521-531, then line 568). For a new or changed file the winner is the
source object, so the destination's entry carries the source mtime, and
cmd/bisync/listing.go:250-293 writes it to the .lst file unchanged.

The next run takes the time from the object as listed (cmd/bisync/march.go:161-170), which
under the flag is the S3 LastModified (backend/s3/s3.go:4293-4296), and compares the two
clocks in cmd/bisync/deltas.go:204-214.

### Suggested fix

After the copies in one direction, refresh the destination entries for the files that were
actually copied from the destination itself, when fs.GetConfig(ctx).UseServerModTime is set,
instead of writing the source's time into the destination listing.

The pattern for this already exists in the same file: recheck (cmd/bisync/listing.go:731-762)
re-lists both sides after an uncertain result and records each side's own ModTime. It lists
whole trees, which a fix must not do; a per-file lookup of the copied names would cost one
request per uploaded file per run, proportional to the number of changed files rather than to
the size of the tree, which is the cost --use-server-modtime is meant to avoid. A narrower
variant records the destination time only when it is knowable without a new request, for
example when the copy result already carries the created object.

The alternative shape touches more code: have sync log the object it created, so the result
carries the destination's real mtime. That changes fs/sync and every backend's returned
object, and it also changes --dest-after output.

Either way, recording the value the next run will read is the invariant worth stating, since
the current code records what the destination was expected to contain.

### Workarounds

- --conflict-resolve newer keeps the newer bytes under the original name when the phantom
  collides with a real local edit, at the cost of a conflict copy.
- A second identical pass right after the uploading pass clears the phantom before a further
  local edit can make it a conflict.
- Dropping --use-server-modtime removes the mismatch entirely and costs one HEAD per object
  per listing.
```

### What the owner decides

- Whether to file it at all,
  and where (an issue, or a discussion first).
- Whether the wording above is the wording to send.
- Whether the suggested fix is the one to propose,
  or whether the report should describe the defect and leave the shape of the fix open.
- Whether the consequence list in the body (conflicts,
  reverted deletes) belongs in an issue a maintainer can reproduce in one run,
  or whether the body should be shortened to the phantom alone.
