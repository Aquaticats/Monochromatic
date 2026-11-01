# Git 2.39.5 trusts cached stat in a copied index with a fresh mtime and misses same-size edits

## Symptom

`git cli-git fix`,
`git add` checks,
and commit transactions build private indexes by copying the real index,
and they install prepared indexes by writing bytes into `index.lock` and renaming it.
Before #544 each copy and install got the current time as its mtime.
A tracked file edited in place without changing its size,
in the same second Git last cached its stat,
then looked unmodified to Git reading the private or installed index.

No error is emitted.
The failure
that started the investigation was the packed `built-dependent-version-bump-consumer.ts` direct-fix scenario:
`git cli-git fix -- package/module/base/package.json` exited 0 with empty stdout
and rewrote no dependent manifest.
A missed hand bump from `1.1.0` to `1.2.0` (same byte length) produces that output.
The same tarball then passed three times,
so a build without the `direct-fix` trigger is excluded.
The rate was then measured by looping that fixture 20 times per build in one container,
rebuilding cli-git with the index copy and install sources from `372168ae0^`:
the pre-fix build failed 4 of 20 with the same message,
and the fixed build failed 0 of 20.
If the fix made no difference,
all four failures landing in the pre-fix half has probability 0.053.
That single original run kept no timestamps,
so it is covered by the rate rather than by its own record.

Surfaces that could miss the edit:

- `git cli-git fix` and `git cli-git check` (private index from `add-policy-facts.ts`),
   so no candidate and no fix;
- `git add` policy checks (same private index),
   so no finding;
- commit transactions that install a prepared index,
   so the real index afterwards hides the edit from `git status` and `git diff`.

## Root cause

Git records the index file's own mtime when it reads an index.
`read-cache.c:2371-2372` at Git `v2.39.5` (`cc7d11c16782041a6bb73e2fb56417b7d4c6d186`):

```c
// read-cache.c (do_read_index)
istate->timestamp.sec = st.st_mtime;
istate->timestamp.nsec = ST_MTIME_NSEC(st);
```

An entry is racy when its cached mtime is not older than that timestamp,
`read-cache.c:360-373`:

```c
// read-cache.c
static int is_racy_stat(const struct index_state *istate,
			const struct stat_data *sd)
{
	return (istate->timestamp.sec &&
#ifdef USE_NSEC
		 /* nanosecond timestamped files can also be racy! */
		(istate->timestamp.sec < sd->sd_mtime.sec ||
		 (istate->timestamp.sec == sd->sd_mtime.sec &&
		  istate->timestamp.nsec <= sd->sd_mtime.nsec))
#else
		istate->timestamp.sec <= sd->sd_mtime.sec
#endif
		);
}
```

Only a racy entry whose stat still matches is compared by content,
`read-cache.c:441-446` in `ie_match_stat`:

```c
// read-cache.c (ie_match_stat)
if (!changed && is_racy_timestamp(istate, ce)) {
	if (assume_racy_is_modified)
		changed |= DATA_CHANGED;
	else
		changed |= ce_modified_check_fs(istate, ce, st);
}
```

Git protects its own index writes by truncating the cached size of racily clean entries that differ,
`read-cache.c:2956-2957` in `do_write_index`,
so a later reader sees a size mismatch:

```c
// read-cache.c (do_write_index)
if (!ce_uptodate(ce) && is_racy_timestamp(istate, ce))
	ce_smudge_racily_clean_entry(istate, ce);
```

`Documentation/technical/racy-git.txt` states the assumption this relies on:
"index entries that can be racily clean are limited to the ones that have the same timestamp as the index file itself."

A byte copy breaks that assumption.
`copyFile` in `add-policy-facts.ts` and `commit-transaction-index.ts`
gave the copy a later mtime than the entries' cached stat,
so `is_racy_stat` returned false,
`ie_match_stat` trusted the stat,
and `git add --all -- <pathspecs>` against the private index skipped the edited file.
Installing through a freshly written lock
(`installIndex` in `commit-transaction-workspace.ts`,
 `installRecoveredIndex` in `commit-transaction-recovery-files.ts`)
had the same effect on the real index.

An earlier fixture draft was wrong about which paths it covered.
Its commit scenarios committed a file that needed no fix,
and ablating the commit-index copies and the lock install still passed.
Wrapping `/usr/bin/git` to log `GIT_INDEX_FILE` showed why:
with no patch to apply,
cli-git hands the commit to real `git commit` on the real index,
which reads the pinned timestamp itself and smudges the entry
(`ls-files --debug` size went from 15 to 0).
Only commits that apply a fix install a prepared index,
so the commit scenarios now stage a file missing its final newline.

## Verification

Git under test:
Debian `git version 2.39.5` from `docker.io/library/node:24-slim`,
source tag `v2.39.5` (commit `cc7d11c16782041a6bb73e2fb56417b7d4c6d186`).

Same-second probe,
run with `podman run --memory=1g --cpus=1 --rm` in that image:

```ts
// racy-probe.ts: copy the index, rewrite a file in place with the same size, stage it through the copy
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, statSync, utimesSync, writeFileSync } from 'node:fs';
execFileSync('apt-get', ['update'], { stdio: 'ignore' });
execFileSync('apt-get', ['install', '--yes', '--no-install-recommends', 'git'], { stdio: 'ignore' });
const git = (cwd: string, args: string[], env: NodeJS.ProcessEnv = {}) =>
  execFileSync('git', args, { cwd, encoding: 'utf8', env: { ...process.env, ...env } }).trim();
function trial(label: string, preserveMtime: boolean, iterations: number): void {
  let missed = 0;
  for (let i = 0; i < iterations; i += 1) {
    const repo = `/r/${label}-${i}`;
    mkdirSync(repo, { recursive: true });
    git(repo, ['init', '--quiet']);
    writeFileSync(`${repo}/f.json`, 'version 1.1.0\n');
    git(repo, ['add', 'f.json']);
    git(repo, ['-c', 'user.name=x', '-c', 'user.email=x@example.invalid', 'commit', '--quiet', '-m', 'base']);
    writeFileSync(`${repo}/f.json`, 'version 1.2.0\n');
    const privateIndex = `${repo}/.git/private-index`;
    copyFileSync(`${repo}/.git/index`, privateIndex);
    if (preserveMtime) {
      const s = statSync(`${repo}/.git/index`);
      utimesSync(privateIndex, s.atime, s.mtime);
    } else {
      const later = new Date(Date.now() + 2000);
      utimesSync(privateIndex, later, later);
    }
    git(repo, ['add', '--all', '--', 'f.json'], { GIT_INDEX_FILE: privateIndex });
    if (git(repo, ['show', ':f.json'], { GIT_INDEX_FILE: privateIndex }) !== 'version 1.2.0') missed += 1;
  }
  console.log(`${label}: missed ${missed} of ${iterations}`);
}
trial('copy-with-later-mtime', false, 20);
trial('copy-preserving-mtime', true, 20);
```

```text
copy-with-later-mtime: missed 20 of 20
copy-preserving-mtime: missed 0 of 20
```

Deterministic packed fixtures,
run in the same image by `mise run //package/git-policy/cli:test:built:trust`:
`package/git-policy/cli/src/trust/fixture/built-racy-index-consumer.ts`
and `built-racy-index-install.ts`,
with helpers in `built-racy-index-helpers.ts`.
They set `core.trustctime=false`,
pin a file's mtime and the real index's mtime to one second 100 seconds in the past,
refresh the index,
and rewrite the file with same-size bytes at that second.
Scenarios that edit before the command first assert the positive control:
`git diff-files --quiet` exits 1 against the real index
and 0 against a plain copy.

Cases that expose the edit:

- `git diff-files` against the real index whose mtime equals the entry's cached second;
- a private index copied with the source's access and modification times;
- an installed index whose lock carries the prepared index's times.

Cases that hide the edit:

- `git diff-files` or `git add` against a byte copy with a later mtime;
- an index installed through a lock written at the current time,
   when the prepared index shared the entry's cached second.

## Verified workarounds

`package/git-policy/cli/src/policy-engine/index-file-timestamps.ts` (commit `372168ae0`):

```ts
// index-file-timestamps.ts
await copyFile(sourcePath, destinationPath);
const { atime, mtime } = await stat(sourcePath);
await utimes(destinationPath, atime, mtime);
// and, for installs, after writing the prepared bytes into the open lock:
await handle.utimes(atime, mtime);
```

Tradeoffs:

- Git re-hashes every entry whose cached mtime is not older than the carried timestamp,
   exactly as it would against the source index,
   so the cost matches real Git rather than exceeding it.
- An edit that backdates a file's mtime below the index timestamp and keeps its size stays hidden;
   real Git has the same limit.

Removing the timestamps from each site in turn failed the packed fixture,
except the commit-to-post copy in index mode:
that copy follows `git apply --cached`,
which rewrites the commit index and truncates differing racy entries itself,
so only an edit in the same second as that write could reach it,
and no hook runs between the two steps.
Per-site results are recorded in #544.

## What does not work

- Relying on commit fixtures that need no fix:
   cli-git forwards those commits to real Git,
   so they never exercise a private index install
   (see "Root cause").
- Pinning without `core.trustctime=false`:
   rewriting a file updates its ctime to the current second,
   which differs from the cached ctime and exposes the edit by stat alone,
   so such a fixture passes even with the bug.
   Probe in the same image,
   rewriting one second after the pin and reading through a fresh-mtime copy:
   `core.trustctime=true` gave `diff-files` exit 1 (edit visible),
   `core.trustctime=false` gave exit 0 (edit hidden).

## Upstream filing decision

- `.out-of-scope/` has no Git entry.
- Constraint 1 fails:
   Git behaves as `Documentation/technical/racy-git.txt` specifies.
   The defect was cli-git giving index copies and installs a newer timestamp than the stat data they carry.
- Constraints 2 to 6 are moot once constraint 1 fails.

Nothing to file upstream,
and no draft is kept.
