# rclone bisync 1.76.0 aborts every run while Path1 holds a name that a ZFS `normalization=formD` Path2 refuses

Status: written by an agent on 2026-11-17, not reviewed or approved by the owner.
Nothing has been filed upstream,
and the readiness check in "Upstream filing decision" has not been run.

When a file appears on Path1 whose name the local ZFS dataset cannot create,
`rclone bisync` fails the whole run with exit status 7 and prints `Must run --resync to recover`.
On this machine's Garage sync (`doc/handover/garage-file-sync.md`) that means the hourly unit fails every hour
until the name is changed.
With `--recover` and backup listings present, no `--resync` is actually needed.

## Symptom

Each run while the file exists logs, verbatim apart from the shortened paths:

```text
ERROR : Music/Tenjin_Kotone/<name>: Failed to copy: open /home/user/Plain/Music/Tenjin_Kotone/<truncated name>.f8de7791.partial: operation not supported
ERROR : Local file system at /home/user/Plain: not deleting files as there were IO errors
NOTICE: copy1to2: Received error: open <same path>: operation not supported - retrying as --resilient is set. Retry 1/3
ERROR : Bisync critical error: open <same path>: operation not supported
ERROR : Bisync aborted. Must run --resync to recover.
NOTICE: Failed to bisync with 2 errors: last error was: bisync aborted
```

The name that triggered it here,
a yt-dlp download,
was 249 bytes as stored (NFC) and 264 bytes once decomposed (NFD):
`【歌ってみた (Cover)】ぼうけんのしょがきえました！ (The Adventure Log Disappeared!) _ じーざすP (Ji-zasuP)【アメノセイ (Amenosei) × Kotone(天神子兎音 (Tenjin Kotone)) coverコラボ (Collab)】 [x-jiNeS9-W4].opus`.
Kana with voicing marks (`ぼ`, `じ`, `ざ`) grow from 3 to 6 bytes when decomposed;
Hangul syllables grow more.
A plain `cp` of the same file fails the same way:
`cp: cannot create regular file '…': Operation not supported`.

What the following runs do depends on whether bisync has backup listings (`<workdir>/<pair>.path1.lst-old`
and `.path2.lst-old`, written by every successful run that was not a `--resync`):

- With backups present and `--recover` set,
  every later run logs `Listings not found. Reverting to prior backup as --recover is set.`,
  copies other new files normally,
  then fails on the same file with the same lines and exit status 7.
- With no backups (for example when the only successful run so far was a `--resync`),
  every later run logs only
  `Bisync critical error: cannot find prior Path1 or Path2 listings, likely due to critical error on prior run`
  and `Bisync aborted. Error is retryable without --resync due to --resilient mode.`,
  exits 7,
  and syncs nothing at all.

## Root cause

Two tools contribute.

### OpenZFS checks the normalized name against a buffer sized for the original name

The pool here was created by the CachyOS installer with `-O normalization=formD` (from `zpool history`),
which every dataset inherits;
`normalization` can only be set when a dataset is created.
OpenZFS stores names unmodified but hashes and compares the normalized form.
The normalization buffer is chosen from the length of the name as given,
in `module/zfs/zap_micro.c:334-336`
([source](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/zap_micro.c#L334-L336)):

```c
// module/zfs/zap_micro.c (zfs-2.4.4)
	size_t key_len = strlen(key) + 1;
	zap_name_t *zn = zap_name_alloc(zap, (key_len > ZAP_MAXNAMELEN));
```

`zap_name_alloc` gives that name a 256-byte buffer unless the original key exceeds 256 bytes,
`module/zfs/zap_micro.c:260`
([source](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/zap_micro.c#L260)),
with the sizes from `include/sys/fs/zfs.h:84-85`
([source](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/include/sys/fs/zfs.h#L84-L85)):

```c
// module/zfs/zap_micro.c (zfs-2.4.4)
	zn->zn_normbuf_len = longname ? ZAP_MAXNAMELEN_NEW : ZAP_MAXNAMELEN;
// include/sys/fs/zfs.h (zfs-2.4.4)
#define	ZAP_MAXNAMELEN 256
#define	ZAP_MAXNAMELEN_NEW 1024
```

`zap_name_init_str` then normalizes into that buffer and turns any failure into `ENOTSUP`,
`module/zfs/zap_micro.c:304-306`
([source](https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/zap_micro.c#L304-L306)):

```c
// module/zfs/zap_micro.c (zfs-2.4.4)
		if (zap_normalize(zap, key, zn->zn_normbuf,
		    zap->zap_normflags, zn->zn_normbuf_len) != 0)
			return (SET_ERROR(ENOTSUP));
```

A 249-byte name whose NFD form needs 265 bytes with its terminator gets the 256-byte buffer,
the normalization does not fit,
and the create fails with `EOPNOTSUPP` ("Operation not supported").

An earlier reading was wrong:
"setting `longname=on` lifts the limit to 1023 bytes,
so the name will fit".
On a throwaway pool with `normalization=formD` and `longname=on`,
`stat -f` reported `namemax=1023` and the same `touch` still failed with `Operation not supported`.
The source explains it:
the long buffer is chosen only when the original name is longer than 256 bytes,
so `longname` never helps a name that is short as stored but long once normalized.

The name rclone actually creates first is a temporary one.
In the observed failure rclone wrote `<name with its last 17 bytes cut>.f8de7791.partial`,
the same 249 bytes as the original,
and that `open` is what ZFS refused.
How rclone shortens partial names in general was not traced.

### rclone bisync treats the failed copy as critical, then recovers from backups anyway

A failed copy marks the run critical and not retryable.
At the end of the run,
`cmd/bisync/operations.go:214-227`
([source](https://github.com/rclone/rclone/blob/e982ff36668f5130d94b4f70409babb3d964df42/cmd/bisync/operations.go#L214-L227))
renames the listings aside and prints the lockout message,
whatever `--recover` is set to:

```go
// cmd/bisync/operations.go (rclone v1.76.0)
	if b.critical {
		if b.retryable && b.opt.Resilient && !b.opt.Resync {
			fs.Errorf(nil, Color(terminal.RedFg, "Bisync critical error: %v"), err)
			fs.Error(nil, Color(terminal.YellowFg, "Bisync aborted. Error is retryable without --resync due to --resilient mode."))
		} else {
			if bilib.FileExists(b.listing1) {
				_ = os.Rename(b.listing1, b.listing1+"-err")
			}
			if bilib.FileExists(b.listing2) {
				_ = os.Rename(b.listing2, b.listing2+"-err")
			}
			fs.Errorf(nil, Color(terminal.RedFg, "Bisync critical error: %v"), err)
			fs.Error(nil, Color(terminal.RedFg, "Bisync aborted. Must run --resync to recover."))
		}
```

On the next run the missing listings send it to the `--recover` branch,
`cmd/bisync/operations.go:296-300`
([source](https://github.com/rclone/rclone/blob/e982ff36668f5130d94b4f70409babb3d964df42/cmd/bisync/operations.go#L296-L300)),
which reverts to the `-old` backups and carries on:

```go
// cmd/bisync/operations.go (rclone v1.76.0)
	if !bilib.FileExists(b.listing1) || !bilib.FileExists(b.listing2) {
		if b.opt.Recover && bilib.FileExists(b.listing1+"-old") && bilib.FileExists(b.listing2+"-old") {
			...
			fs.Log(nil, Color(terminal.YellowFg, "Listings not found. Reverting to prior backup as --recover is set. \n")+errTip)
```

Without backups the same check falls through to `cmd/bisync/operations.go:312-322`
([source](https://github.com/rclone/rclone/blob/e982ff36668f5130d94b4f70409babb3d964df42/cmd/bisync/operations.go#L312-L322)),
whose own comment reads "the prior listings are renamed to .lst-err to lock out further runs",
and which marks the run critical and retryable and returns `cannot find prior Path1 or Path2 listings`;
with `--resilient`,
the end-of-run branch at `operations.go:214-217` then prints "retryable without --resync",
although no later run can succeed until a `--resync` writes new listings.

So with `--recover` and backups,
"Must run --resync to recover" is not true:
the next run recovers by itself once the name is fixed.
Without backups,
"retryable without --resync" is not true either.
rclone's own documentation scopes `--recover` to "a sudden interruption or other un-graceful shutdown"
(`docs/content/bisync.md:786-787` in the v1.76.0 tree),
so the fallback after a critical abort goes beyond what is documented.

Where this was measured:
between two local folders,
without the Garage unit's `--use-server-modtime`,
`--fast-list` and filter.
With `--use-server-modtime`,
files synced during the failing runs look "new or changed on both sides" after the revert
and may resolve as conflicts,
which `--conflict-resolve newer` settles by keeping the newer copy and renaming the other to `.conflictN`;
see `doc/troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md`.
That was not measured.

An earlier reading was wrong too:
"one bad name stops the whole sync until a `--resync`".
That came from a first throwaway test whose only successful run was the `--resync` baseline,
so no backups existed.
The second test,
with one normal run before the bad name,
showed other files syncing during the failing runs and recovery without `--resync`.

## Verification

Versions under test:

- rclone v1.76.0,
  the Arch `rclone` package (`/usr/bin/rclone`);
  source read at tag `v1.76.0`,
  commit `e982ff36668f5130d94b4f70409babb3d964df42`.
- OpenZFS 2.4.4 (`zfs-2.4.4-1`,
  `zfs-kmod-2.4.4-1`);
  source read at tag `zfs-2.4.4`,
  commit `71a9f9578616a90c3c14bb59629fb4d31bfd68d1`.
- Linux 7.3.4-1-cachyos,
  systemd 262.

The harness builds a file-backed throwaway pool,
never touching the real pool,
with a `normalization=none` root for Path1 and a `normalization=formD` child dataset for Path2,
then replays a realistic history.
It needs passwordless `sudo` for `zpool`/`zfs` and the long-named sample file at the path in `sample`.

```ts
// ~/temp/agent/bisync-recover-from-old-listings-2026-11-17/run.ts (condensed)
import { spawnSync, } from 'node:child_process'
import { copyFileSync, mkdirSync, readdirSync, renameSync, rmSync, truncateSync, writeFileSync, } from 'node:fs'
import { join, } from 'node:path'

const base = `${process.env.HOME}/temp/agent/bisync-recover-from-old-listings`
const [vdev, mnt, work,] = ['vdev.img', 'mnt', 'bisync-workdir',].map((name,) => join(base, name,))
const [src, dst,] = [join(mnt, 'src',), join(mnt, 'dst',),]
const sample = '/path/to/a/file/whose/NFD/name/exceeds/255/bytes.opus'
const longName = sample.split('/',).at(-1,) ?? ''
const pool = 'bisyncrecovertest'
const run = (cmd: string, args: string[],) => spawnSync(cmd, args, { encoding: 'utf8', },)
const bisync = (label: string, extra: string[],) => {
  const result = run('rclone', ['bisync', src, dst, '--workdir', work, '--conflict-resolve', 'newer',
    '--resilient', '--recover', '-v', ...extra,],)
  const lines = `${result.stdout}${result.stderr}`.split('\n',)
    .filter((line,) => /ERROR|critical|aborted|successful|Reverting|Copied/.test(line,))
  console.log(`--- ${label}: exit ${result.status}\n${[...new Set(lines,),].join('\n',)}`,)
}

mkdirSync(mnt, { recursive: true, },)
writeFileSync(vdev, '',)
truncateSync(vdev, 512 * 1024 * 1024,)
try {
  run('sudo', ['-n', 'zpool', 'create', '-o', 'cachefile=none', '-O', 'normalization=none', '-m', mnt, pool, vdev,],)
  run('sudo', ['-n', 'zfs', 'create', '-o', 'normalization=formD', `${pool}/dst`,],)
  run('sudo', ['-n', 'chown', '-R', `${process.getuid?.()}:${process.getgid?.()}`, mnt,],)
  mkdirSync(src,)
  writeFileSync(join(src, 'ok.txt',), 'baseline\n',)
  bisync('baseline --resync', ['--resync',],)
  writeFileSync(join(src, 'second.txt',), 'second\n',)
  bisync('normal run (writes .lst-old)', [],)
  copyFileSync(sample, join(src, longName,),)
  bisync('run 1 after the long name appears', [],)
  writeFileSync(join(src, 'later.txt',), 'later\n',)
  bisync('run 2 with an unrelated new file too', [],)
  renameSync(join(src, longName,), join(src, 'short-name.opus',),)
  bisync('renamed on Path1, plain run', [],)
  console.log('dst holds:', readdirSync(dst,).sort().join(', ',),)
} finally {
  run('sudo', ['-n', 'zpool', 'destroy', pool,],)
  rmSync(base, { recursive: true, force: true, },)
}
```

Run it with `node run.ts`;
Node 26 runs TypeScript files directly.

### Patterns that work

- The shortened name on `normalization=formD`,
  229 bytes NFC and 244 bytes NFD:
  downloaded with `Copied (new)` on the real sync.
  Names between 245 and 255 bytes NFD were not tried.
- The 249-byte name on a dataset with `normalization=none`:
  `cp -a` created it.
- The same name on a dataset with `normalization=formC`:
  created;
  `café` written once precomposed and once decomposed still produced a single directory entry,
  so the look-alike protection stays.
- With backups and `--recover`,
  renaming the file on Path1:
  the next plain run logged `Reverting to prior backup`,
  `short-name.opus: Copied (new)`,
  `Bisync successful`,
  exit 0.
- Without backups,
  `--resync --dry-run` (exit 0,
  listing the pending copies) followed by `--resync` (exit 0),
  after which a plain run exited 0.

### Patterns that fail, by variant

`operation not supported`,
exit 7,
`Must run --resync to recover`:

- The long name on `normalization=formD`,
  with or without `longname=on`.
- Every later run while the name is still on Path1 and backups exist;
  other new files are copied in the same run (`later.txt: Copied (new)`).

`cannot find prior Path1 or Path2 listings`,
exit 7,
"retryable without --resync":

- Every run after the first failure when no backups exist,
  including after the name is fixed;
  nothing is copied.

## Verified workarounds

### Rename the file on the remote

```bash
rclone moveto 'garage:files/Plain/<dir>/<long name>' 'garage:files/Plain/<dir>/<short name>'
# 244 bytes in NFD is the measured-safe size; the exact limit depends on rclone's temporary
# .partial name, whose naming rule was not traced, so stay well under 255:
python3 -c 'import sys,unicodedata as u;print(len(u.normalize("NFD",sys.argv[1]).encode()))' '<short name>'
```

With backups present the next hourly run recovers by itself.
Tradeoff:
every client sees the rename,
and it fixes one file at a time.
This is what was done on 2026-11-17,
dropping ` (Ji-zasuP)` and ` (Collab)` from the name in "Symptom".

### Give the local folder its own dataset with `normalization=formC` or `none`

Verified on throwaway pools only.
Tradeoff:
the property is create-time only,
so the data must be copied into the new dataset,
and the new dataset needs its own snapshot policy;
`formC` can still expand rare characters;
`none` drops the look-alike protection.

### Resync after fixing the name, when no backups exist

Run the unit's own bisync command with `--resync --dry-run`,
read what it would copy,
then run it with `--resync`.
Tradeoff:
a resync takes the union of both sides,
so a file deleted on one side since the last good run comes back.

### Detect it

On this machine an `OnFailure=` drop-in on `rclone-bisync-garage.service` starts a template handler
that reads the failed run's journal and raises a critical desktop notification naming the file and this doc.
Tradeoff:
a failure while nobody is logged in to the desktop shows nothing until the next hourly run fails again.

## What does not work

- `longname=on`:
  the normalization buffer is chosen from the original name's length (see "Root cause").
- Waiting:
  a name that cannot be created fails on every run.
- `--resilient` alone:
  a copy failure is not classified retryable,
  and without backups the "retryable" retry can never succeed.
- Renaming the file without backups:
  later runs still fail on the missing listings until a `--resync`.

## Upstream filing decision

Checked on 2026-11-17.
`.out-of-scope/` has no entry for OpenZFS or rclone.
`agent-browser` is not installed on this machine,
so none of the source links was load-checked as the troubleshooting-doc skill asks.

Filing posts under a real person's account,
and some projects run trust or blocklist systems that can follow a poster across projects;
no trust-system search was done for either repository.
Whether to post anything is the poster's decision,
and this doc makes no recommendation.

### OpenZFS

A matching issue exists:
[openzfs/zfs#9965](https://github.com/openzfs/zfs/issues/9965),
"normalization=formD fails to allow writing of some valid utf8 filenames",
open since 2020.
The older [openzfs/zfs#2234](https://github.com/openzfs/zfs/issues/2234) was closed by the stale bot without a fix.
A comment there from 2026-02-15 reports that `longname` does not help and asks whether that is expected.

1.  Upstream's fault:
    yes;
    a name valid as stored cannot be created,
    and the error is `ENOTSUP` rather than `ENAMETOOLONG`.
2.  Can upstream fix it:
    yes;
    the buffer could be sized for the normalized length.
3.  Supported use case:
    yes;
    `normalization` is a documented dataset property.
4.  Contributions welcome:
    not checked (`CONTRIBUTING.md` and AI policy not read).
5.  Likely to fix:
    no maintainer response in the thread;
    absence of signal is not a fail.
6.  Prototype:
    not done.
    It needs a patched kernel module,
    which cannot be loaded on this host while its root pool runs on ZFS;
    a throwaway VM would be the place.

Not a new issue:
the contribution would be an additive comment answering the 2026-02-15 question.
Do not post as-is:
constraints 4 and 6 are open.

~~~md
The reason `longname` does not help here: `zap_name_alloc_str` picks the normalization buffer from the
length of the name as given, not of its normalized form
(`module/zfs/zap_micro.c:334-336` at zfs-2.4.4,
https://github.com/openzfs/zfs/blob/71a9f9578616a90c3c14bb59629fb4d31bfd68d1/module/zfs/zap_micro.c#L334-L336):

    zap_name_t *zn = zap_name_alloc(zap, (key_len > ZAP_MAXNAMELEN));

so any name of at most 255 bytes gets the 256-byte buffer even with `longname=on`, and
`zap_name_init_str` returns `ENOTSUP` when the NFD form does not fit
(`zap_micro.c:304-306`). Reproduced on 2.4.4 with a 249-byte NFC name that is 264 bytes in NFD:
`normalization=formD` fails with or without `longname=on` (`namemax` reports 1023), and
`normalization=formC` and `none` accept it.
~~~

### rclone

No matching issue:
`gh search issues --repo rclone/rclone` for `bisync "Must run --resync" recover`,
`bisync critical error single file copy resync lockout` and `bisync recover resync` found none.
Related:
[rclone/rclone#9352](https://github.com/rclone/rclone/issues/9352),
closed as not planned;
the bisync maintainer explained on the forum that the reporter's `--resync` run itself was failing,
and the reporter called the "retryable without --resync" line misleading.

1.  Upstream's fault:
    yes,
    in wording at least:
    both abort messages misstate what the next run needs when `--recover` is set.
2.  Can upstream fix it:
    yes;
    the message branch can check `b.opt.Recover` and the `-old` files,
    as the `--recover` branch already does.
3.  Supported use case:
    yes;
    the docs present `--resilient` with `--recover` for scheduled background runs.
4.  Contributions welcome:
    yes with conditions;
    `CONTRIBUTING.md` "AI-assisted contributions" (v1.76.0 tree) welcomes AI-assisted pull requests
    if the author understands every line,
    `go build` and `make quicktest` pass,
    and comments are trimmed.
5.  Likely to fix:
    no won't-fix signal found.
6.  Prototype:
    done on 2026-11-17 in a disposable clone of tag `v1.76.0`
    (`origin` `https://github.com/rclone/rclone.git`,
    commit `e982ff36668f5130d94b4f70409babb3d964df42`),
    built with `go build` (go1.27.4).

```diff
--- a/cmd/bisync/operations.go
+++ b/cmd/bisync/operations.go
@@ -223,7 +223,11 @@ func Bisync(ctx context.Context, fs1, fs2 fs.Fs, optArg *Options) (err error) {
 				_ = os.Rename(b.listing2, b.listing2+"-err")
 			}
 			fs.Errorf(nil, Color(terminal.RedFg, "Bisync critical error: %v"), err)
-			fs.Error(nil, Color(terminal.RedFg, "Bisync aborted. Must run --resync to recover."))
+			if b.opt.Recover && !b.opt.Resync && bilib.FileExists(b.listing1+"-old") && bilib.FileExists(b.listing2+"-old") {
+				fs.Error(nil, Color(terminal.YellowFg, "Bisync aborted. The next run will retry from the backup listings as --recover is set."))
+			} else {
+				fs.Error(nil, Color(terminal.RedFg, "Bisync aborted. Must run --resync to recover."))
+			}
 		}
 		return ErrBisyncAborted
 	}
```

Verification of the prototype,
with the harness in "Verification" run against both binaries on throwaway pools:

- Installed v1.76.0,
  with backups:
  runs 1 to 3 print `Bisync aborted. Must run --resync to recover.`;
  after the rename,
  a plain run reverts to the backups and ends `Bisync successful` (exit 0).
- Patched,
  with backups:
  the same runs print `Bisync aborted. The next run will retry from the backup listings as --recover is set.`;
  the outcome of every run is unchanged.
- Without backups,
  both binaries print the same lines run for run,
  and both still need a `--resync`,
  so the patch changes only the case where the old message was untrue.
- `go test -count=1 -v ./cmd/bisync/ -remote local` with `RCLONE_CONFIG=/notfound`:
  every test passed and none failed.
  `TestBisyncRemoteRemote` ran all 33 golden cases (one per `cmd/bisync/testdata/test_*` folder);
  `TestBisyncRemoteLocal` and `TestBisyncLocalRemote` skip when both paths are local.
  Without `-remote local` all three golden suites skip,
  so a bare `go test ./cmd/bisync/` proves nothing about this change.
- Not run:
  `make quicktest`,
  which rclone's `CONTRIBUTING.md` requires before a pull request,
  and `golangci-lint`.

All six constraints hold.
The second misleading message ("retryable without --resync" when no backups exist) is outside this patch.

Readiness check,
not run on any item:
reproduction on the latest release and other maintained lines,
prior pull requests touching these lines,
the issue and pull request templates,
the tracker's automation,
a same-day duplicate and work-in-progress search,
a trust-system search,
and a public reproduction repository with a CI workflow.
No recommendation to file;
that is the poster's decision,
with the risk described at the start of this section.

~~~md
Title: bisync: "Must run --resync to recover" is printed even when --recover will recover on the next run

With `--resilient --recover`, a run that hits a non-retryable copy error (here a local filesystem refusing a
file name with `operation not supported`) ends with `Bisync aborted. Must run --resync to recover.`
(`cmd/bisync/operations.go:226`). The next plain run takes the `--recover` branch
(`operations.go:297-300`), reverts to the `-old` listings and succeeds once the cause is gone; no `--resync`
is needed. Conversely, without `-old` listings, later runs print "retryable without --resync" although they
cannot succeed until a `--resync`.

Suggested fix: in the non-retryable branch, print a message saying the next run will retry from the backup
listings when `b.opt.Recover` is set and both `-old` files exist.
~~~
