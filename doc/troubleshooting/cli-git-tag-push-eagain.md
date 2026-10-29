# cli-git manual push of a new tag or branch fails with `spawn /usr/bin/git EAGAIN`

Status:
fixed on branch `fix/cli-git-manual-push-scans-already-published-history`
(commits `b2adca59d`,
`d4820fcec`,
`a9aa892d7`,
`ddf15690a`).
The cause was our own code in `package/git-policy/cli`,
not Git.

## Symptom

Pushing a single tag through the wrapped `git` failed three times in a row on 2026-09-06:

```text
git push origin '@monochromatic-dev/logger-core@0.1.0'
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"plugin-threw","message":"spawn /usr/bin/git EAGAIN","trigger":"manual-push","policyId":"final-newline"}
```

The tag existed locally (`git tag -a '@monochromatic-dev/logger-core@0.1.0' dfa0e0d58 -m ...` succeeded).
Commit auto-pushes through the same wrapper kept working before and after
(`git status -sb` showed `main` in sync with `origin/main` between attempts).

On 2026-09-26 the same failure crashed a Claude Code session twice
when a manual `git push` of a new ref ran inside the session's process tree:
the wrapper's process fan-out exhausted the task budget the session shares,
so the session's own process creation failed too.

Every manual push with an update that has no prior remote value triggered it:

- a new tag (`git push origin <tag>`),
  annotated or lightweight;
- a new branch (`git push origin <branch>:refs/heads/<new>`);
- a first push to any ref.

Pushes that advance an existing ref only scanned the new range and were not affected.
Post-commit auto-push uses a different lifecycle
(`post-commit-facts.ts`,
 one `git diff-tree` for the single landed commit)
and was never affected.

## Root cause

Any enabled policy with the `manual-push` trigger asks for candidates.
`final-newline` is a core policy enabled by default,
so it was the first to call `context.git.candidates()`
(`package/git-policy/cli/src/policy-engine/final-newline-policy.ts:103` at `5c4d114f0`):

```ts
// package/git-policy/cli/src/policy-engine/final-newline-policy.ts
const candidates = await context.git
  .candidates();
```

### Step 1: a new ref scanned the whole history

`pushedCommits` built the commit range with no exclusion when the destination ref was absent
(`package/git-policy/cli/src/policy-engine/manual-push-candidates.ts:111` at `5c4d114f0`):

```ts
// package/git-policy/cli/src/policy-engine/manual-push-candidates.ts
const exclusions = update.remoteOid === ABSENT_GIT_VALUE
  ? []
  : [`^${update.remoteOid}`,];
```

It then ran `git rev-list --reverse <local>` with those exclusions (lines 118 to 125)
and added the pushed tip unconditionally (line 133).
For a new tag or branch that is every commit reachable from the tip,
even though the remote already had almost all of them on `main`.
The incident tag's target `dfa0e0d58` reaches 8,493 commits
(`git rev-list --count dfa0e0d58`);
`main` at `5c4d114f0` reaches 10,494.

### Step 2: one concurrent `git diff-tree` per commit

`createManualPushCandidates` mapped every commit to its own subprocess with an uncapped `Promise.all`
(`package/git-policy/cli/src/policy-engine/manual-push-candidates.ts:190` at `5c4d114f0`):

```ts
// package/git-policy/cli/src/policy-engine/manual-push-candidates.ts
return (await Promise.all(commits.map(function commitCandidates(commit,) {
  return commitDeltaCandidates({
    gitPath,
    cwd,
    commitOid: commit,
    targetPrefix: `manual-push:${update.remoteName}:${update.remoteRef}:${commit}`,
  },);
},))).flat();
```

Each `commitDeltaCandidates` call spawned `git diff-tree`
(`package/git-policy/cli/src/policy-engine/manual-push-descriptors.ts:295` at `5c4d114f0`,
through `spawn` at line 123):

```ts
// package/git-policy/cli/src/policy-engine/manual-push-descriptors.ts
args: [
  'diff-tree',
  '--root',
  '--no-commit-id',
  '-r',
  '-z',
  '-m',
  commitOid,
],
```

So one tag push started about 8,500 `git` processes at once.
When `fork` failed,
Node reported `spawn /usr/bin/git EAGAIN`,
the rejection escaped `final-newline`'s `check`,
and `policy-stage.ts:218` to `policy-stage.ts:224` recorded it as `plugin-threw` under that policy's ID.

### The earlier "not host exhaustion" reading was wrong

The first write-up ruled out a process limit because the user had
"305 processes ... against `ulimit -u` of 8192",
and concluded the `EAGAIN` did not come from host exhaustion.
That compared the wrong unit.
`RLIMIT_NPROC` (`ulimit -u`) and the systemd slice's `TasksMax` (cgroup `pids.max`) both count tasks,
meaning threads,
not processes.
Measured on this host on 2026-09-26:

```text
$ ps --user "$(id --user)" --no-headers | wc --lines
395
$ ps -L --user "$(id --user)" --no-headers | wc --lines
4468
$ systemctl show "user-$(id --user).slice" --property TasksMax --property TasksCurrent
TasksCurrent=4519
TasksMax=12288
```

About 400 processes already held about 4,500 tasks,
because every Node and Electron process runs many threads.
About 8,500 more concurrent `git` tasks exceed either budget,
so the fan-out did exhaust process creation for the user,
which is why a Claude Code session in the same tree failed alongside it.
The other ruled-out items stay valid:
memory was never the constraint (`free -g` showed 30 GiB available),
and retries failed identically because history size,
not transient load,
determined the fan-out.
Commit auto-pushes succeeded in between because they take the post-commit path.

## Verification

### Harness

Bounded-container reproduction through the packed wrapper,
on `node:24-slim` with Debian's `git`,
`--memory=2g --cpus=2 --pids-limit=512`,
with a 3,000-commit history already pushed to a local bare remote,
followed by a wrapped push of a new annotated tag at `main~5` and of a new branch at `main~3`.
The fixture repository contains a trusted empty `cli-git.config.mjs`;
without a trusted configuration manual-push policies do not run.
`/fixture/cli.tgz` is the output of `mise run //package/git-policy/cli:pack:npm`
for the revision under test.

```js
// /fixture/repro.mjs
import { execFile, spawn } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { promisify } from 'node:util';

const run = promisify(execFile);
const COMMITS = Number(process.env.COMMITS ?? '3000');
await run('apt-get', ['update']);
await run('apt-get', ['install', '--yes', '--no-install-recommends', 'git']);
await mkdir('/work/cli', { recursive: true });
await run('npm', ['init', '--yes'], { cwd: '/work/cli' });
await run('npm', ['install', '--ignore-scripts', '/fixture/cli.tgz'], { cwd: '/work/cli' });
const wrapper = '/work/cli/node_modules/@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs';
const repo = '/work/repo';
const remote = '/work/remote.git';
const env = { ...process.env, HOME: '/work/home', GIT_AUTHOR_NAME: 'F', GIT_AUTHOR_EMAIL: 'f@example.invalid',
  GIT_COMMITTER_NAME: 'F', GIT_COMMITTER_EMAIL: 'f@example.invalid' };
await mkdir('/work/home', { recursive: true });
await run('git', ['init', '--quiet', '--initial-branch=main', repo], { env });
await run('git', ['init', '--quiet', '--bare', remote], { env });
const stream = Array.from({ length: COMMITS }, (_u, i) => {
  const content = `revision ${i}\n`;
  const msg = `c${i}`;
  return `commit refs/heads/main\nmark :${i + 1}\ncommitter F <f@example.invalid> ${1_700_000_000 + i} +0000\n`
    + `data ${msg.length}\n${msg}\n${i === 0 ? '' : `from :${i}\n`}`
    + `M 100644 inline file-${i % 64}.txt\ndata ${content.length}\n${content}\n`;
}).join('');
await writeFile('/work/stream', stream);
await run('sh', ['-c', 'git fast-import --quiet < /work/stream'], { cwd: repo, env });
await run('git', ['reset', '--quiet', '--hard', 'main'], { cwd: repo, env });
await writeFile(`${repo}/cli-git.config.mjs`, 'export default {};\n');
await run('git', ['add', 'cli-git.config.mjs'], { cwd: repo, env });
await run('git', ['commit', '--quiet', '-m', 'config'], { cwd: repo, env });
await run(process.execPath, [wrapper, 'cli-git', 'trust', '--yes'], { cwd: repo, env });
await run('git', ['remote', 'add', 'origin', remote], { cwd: repo, env });
await run('git', ['push', '--quiet', 'origin', 'main'], { cwd: repo, env });
await run('git', ['tag', '--annotate', '--message', 'v1', 'v1', 'main~5'], { cwd: repo, env });
await run('git', ['branch', 'feature', 'main~3'], { cwd: repo, env });

function wrapped(args) {
  return new Promise((resolve) => {
    const started = performance.now();
    const child = spawn(process.execPath, [wrapper, ...args], { cwd: repo, env, stdio: ['ignore', 'pipe', 'pipe'] });
    let stderr = '';
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.stdout.resume();
    child.on('close', (code) => resolve({ args: args.join(' '), code, ms: Math.round(performance.now() - started),
      stderr: stderr.trim().slice(0, 400) }));
  });
}
for (const args of [['push', 'origin', 'refs/tags/v1:refs/tags/v1'], ['push', 'origin', 'feature:refs/heads/feature']]) {
  console.log(JSON.stringify({ commits: COMMITS, ...(await wrapped(args)) }));
}
const remoteRefs = await run('git', ['for-each-ref', '--format=%(refname)'], { cwd: remote });
console.log(JSON.stringify({ remoteRefs: remoteRefs.stdout.trim().split('\n') }));
```

```sh
podman run --memory=2g --cpus=2 --pids-limit=512 --rm \
  --volume <tarball>:/fixture/cli.tgz:ro,Z --volume <repro.mjs>:/fixture/repro.mjs:ro,Z \
  --tmpfs /work:rw,size=1g docker.io/library/node:24-slim node /fixture/repro.mjs
```

Never run the unfixed wrapper's manual push of a new ref on a large history outside such a container:
it exhausts the invoking user's task budget.

### Pre-fix failures at `5c4d114f0`

Both pushes reproduce the incident message verbatim,
and the remote keeps only `main`:

```text
{"commits":3000,"args":"push origin refs/tags/v1:refs/tags/v1","code":2,"ms":5085,"stderr":"{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"plugin-threw\",\"message\":\"spawn /usr/bin/git EAGAIN\",\"trigger\":\"manual-push\",\"policyId\":\"final-newline\"}"}
{"commits":3000,"args":"push origin feature:refs/heads/feature","code":2,"ms":5878,"stderr":"{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"plugin-threw\",\"message\":\"spawn /usr/bin/git EAGAIN\",\"trigger\":\"manual-push\",\"policyId\":\"final-newline\"}"}
{"remoteRefs":["refs/heads/main"]}
```

### Post-fix results at `a9aa892d7`

```text
{"commits":3000,"args":"push origin refs/tags/v1:refs/tags/v1","code":0,"ms":175,"stderr":"To /work/remote.git\n * [new tag]         v1 -> v1"}
{"commits":3000,"args":"push origin feature:refs/heads/feature","code":0,"ms":167,"stderr":"To /work/remote.git\n * [new branch]      feature -> feature"}
{"remoteRefs":["refs/heads/feature","refs/heads/main","refs/tags/v1"]}
```

### Process counts on one fixture

In-process spawn counting around `createManualPushCandidates`
(the `node:child_process` `spawn` export replaced,
then `syncBuiltinESMExports()`)
on a 300-commit linear history over 32 files:

- range above the root commit before the fix:
  303 spawns,
  peak 299 concurrent
  (`diff-tree` 299,
  `cat-file` 2,
  `rev-parse` 1,
  `rev-list` 1);
- range above the root commit after the fix:
  5 spawns,
  peak 2 concurrent
  (`cat-file` 2,
  `for-each-ref` 1,
  `rev-list` 1,
  `diff-tree` 1);
- empty remote before the fix:
  304 spawns,
  peak 300 concurrent,
  300 candidates from walking every commit;
- empty remote after the fix:
  4 spawns,
  peak 2 concurrent
  (`cat-file` 2,
  `for-each-ref` 1,
  `ls-tree` 1),
  32 candidates from the tip tree.

### Regression tests

`package/git-policy/cli/src/policy-engine/manual-push-scope.unit.test.ts` pins the scope and the bound.
At `b2adca59d` (tests only,
 unfixed source) seven of its eight tests failed;
the mixed-history parity test passed,
proving it characterizes the old per-commit semantics the refactor had to keep.

Cases that scan nothing or only new commits:

- a new branch pushed to a local bare remote scans only its two unpublished commits;
- an annotated tag on an already-published commit scans nothing and spawns no `diff-tree`;
- a prior destination value of one update bounds another update to the same remote;
- a remote value Git never fetched is skipped while tracking refs still bound the range.

Cases that scan a full tree once:

- a first push of 300 commits to an empty remote scans the tip tree once
  (one `ls-tree`,
   no `rev-list`,
   no `diff-tree`,
   at most 8 spawns and 2 concurrent);
- tracking refs of another remote do not count as published.

Cases that keep exact per-commit semantics:

- a range over an octopus merge with an unrelated root,
  a second root commit,
  a rename,
  a deletion,
  a symlink-to-file type change,
  an executable,
  and gitlink updates equals an independent one-`diff-tree`-per-commit reference;
- a 300-commit range spawns exactly one `diff-tree`,
  at most 8 processes,
  and at most 2 concurrently.

## Verified workarounds

### Upgrade to the fixed wrapper

The fix is the workaround.
The published set and fallback are specified in `package/git-policy/cli/SPEC.md`,
section "Manual push":

- `manual-push-published.ts` peels every pushed and prior object in one `git cat-file --batch-check`
  and lists `refs/remotes/` once with `git for-each-ref`,
  matching `refs/remotes/<remote name>/` as a literal prefix.
  A commit is already published when it is reachable from those tracking commits
  or from any locally present prior destination value of an update to the same remote.
- `manual-push-candidates.ts:185` scans the pushed tip's tree once when that set is empty,
  and otherwise lists only unpublished commits with `git rev-list --reverse --stdin`
  (exclusions on stdin,
   line 124).
  Per-update work runs through `mapBounded` with `UPDATE_LANES = 4` (line 37),
  so `git push --tags` cannot fan out either.
- `manual-push-descriptors.ts:334` reads every range commit's delta through one
  `git diff-tree --stdin --root -r -z -m`,
  parsed per commit header by `parseRawDiffCommitStream` in `raw-diff-records.ts`
  with the same first-wins path retention as the old single-commit parse.

Tradeoffs:

- A first push to an empty remote scans only the final tree,
  so content added and removed in intermediate commits before that first push is not seen.
  The repository owner chose this over walking history.
  `manual-push.integration.unit.test.ts` now publishes a baseline before asserting transient-blob blocking.
- Remote-tracking refs are a cached view.
  After a remote history rewrite without a fetch,
  a stale tracking ref can exclude commits the remote no longer has.
- Pushing history unrelated to every published commit still lists that whole range,
  though through a fixed number of processes.

### Create the tag through the GitHub API on an unfixed wrapper

Verified during the 2026-09-06 incident:

```text
gh release create '<tag>' --target <full commit SHA> --title '<tag>' --notes-file <notes>
```

`--target` needs the full 40-character SHA;
the short form is rejected with `Release.target_commitish is invalid`.
GitHub creates a lightweight tag at that commit and the release in one call.
Afterwards delete the local annotated tag (`git tag -d '<tag>'`) before `git fetch --tags`,
or the fetch refuses to clobber it.
Tradeoff:
the tag is lightweight rather than annotated,
and no manual-push policy runs on it.

`doc/runbook/publish-npm-package-first-time.md` uses this path for the bootstrap version's tag.

## What does not work

- Excluding only `^<remote OID>`:
  a new ref has no remote OID,
  which is the bug.
- Copying Git's sample pre-push hook:
  the Git 2.55 template (`/usr/share/git-core/templates/hooks/pre-push.sample`)
  does not exclude anything for a new ref either;
  it sets `range="$local_oid"` under the comment `# New branch, examine all commits`.
  The assumption that the sample uses `--not --remotes=<remote>` does not hold for this version.
- `git rev-list --remotes=<name>`:
  treats the remote name as a glob pattern,
  and a pre-push hook receives a URL as its name for unnamed remotes.
  The fix matches a literal prefix instead.
- Capping the `Promise.all` with bounded lanes while keeping one `diff-tree` per commit:
  still thousands of processes over time;
  `--stdin` keeps exact semantics in one process.

## Upstream filing decision

Not filed.
The defect is in `package/git-policy/cli`,
this repository's own code;
Git behaved as documented throughout.

1. Upstream's fault:
   no;
   the unbounded range and the fan-out are ours.
2. Upstream can fix:
   not applicable.
3. Upstream supports the use case:
   not applicable.
4. Contribution welcome:
   not applicable.
5. Likely to fix:
   not applicable.
6. Prototyped fix:
   the fix itself landed in this repository.

`.out-of-scope/` has no Git entry;
it is moot because there is nothing to file.
