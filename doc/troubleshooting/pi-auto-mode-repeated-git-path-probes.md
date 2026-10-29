# Pi auto-mode 0.0.1 read events emit repeated missing-Git PATH probe logs

Status:
 resolved.

## Resolution

`@monochromatic-dev/git-executable-core` now owns real-Git selection for git-policy-cli and auto-mode.
The resolver promotes common platform paths exposed by `PATH`,
scans remaining candidates sequentially,
recognizes Windows `PATHEXT`,
skips every known cli-git self-shim form,
rejects non-regular filesystem candidates without blocking,
rejects non-native scripts above 64 KiB,
and caches 16 least-recently-used successful resolutions by effective absolute candidate sequence.
Expected absent candidates no longer emit one diagnostic each.

Auto-mode deleted its duplicate resolver but keeps `git worktree list` and `rev-parse` metadata queries fresh for every
read event.
A same-process test creates another linked worktree after initial discovery and verifies
that the next discovery includes
it.

The post-change verbose unit harness recorded one `/usr/bin/git` resolver success across seven linked-worktree metadata
collections and no `git not executable at` records.
A Pi SDK host verification loaded the built auto-mode extension and executed two built-in `read` calls.
It recorded one `/usr/bin/git` resolution,
two fresh worktree metadata collections,
and no missing-candidate records.
The resolver successes are positive controls proving that both verbose harnesses captured resolver diagnostics.
The shared package,
git-policy-cli,
and auto-mode unit suites passed after migration.
Artifact-level cache tests observed one scanner invocation for concurrent equal calls and confirmed least-recently-used
eviction after a refreshed key.

Implementation commits are recorded in
[`doc/planning/pi-auto-mode-real-git-resolution.md`](../planning/pi-auto-mode-real-git-resolution.md).

The root-cause excerpts in this document describe the pre-fix revision and remain as historical diagnostic evidence.

## Symptom

A Pi `read` event can add debug records such as:

```text
[auto-mode] [git-worktree-read-allowlist] [resolveGitCandidate] git not executable at /var/home/user/Monochromatic/package/pi-shared/model-selection/node_modules/.bin/git: Error: ENOENT: no such file or directory, access '/var/home/user/Monochromatic/package/pi-shared/model-selection/node_modules/.bin/git'
```

The named package directory is expected to be on `PATH`.
Pi needs the repository's broad workspace-bin `PATH` for normal operation.
The diagnostic does not mean Pi expected that specific package to provide Git,
and it does not mean real Git resolution failed.

The repeated records come from filesystem executable checks.
Auto-mode does not spawn each missing candidate.

## Root cause

### The broad PATH is intentional input

`file-enforcer.config.ts:651-680` generates `mise.toml` with every materialized workspace bin directory.
Its own comment states that this membership changes with installation state and is expected:

```ts
/**
 * Generates mise.toml from mise.no-env.toml with a dynamic [env] section
 * containing _.path entries for all workspace package bin directories.
 */
async function generateMiseToml(): Promise<void> {
  // ...
  const envSection = `[env]
  // ...
# Every workspace bin dir, globbed from what is installed on disk rather than
# read from workspace metadata. So this list changes with install state: a fresh
# clone, a pruned package, or a pnpm install that adds a dependency all move it.
# That is expected, and the regenerated list is committed as file-enforcer
# writes it.
_.path = [
${
    [
      ...new Set([
        'node_modules/.bin',
        ...(await Array.fromAsync(glob('package/*/*/node_modules/.bin',),)).toSorted(),
      ]),
    ]
```

The inherited shell `PATH` measured during diagnosis had
`package/pi-shared/model-selection/node_modules/.bin` at position 37 and `/usr/bin` at position 122.
The user's Pi runtime `PATH` was not sampled separately,
but the supplied log proves that the model-selection bin directory reached auto-mode.
That broad ordering is valid and is not the defect.

### Every read recomputes the linked-worktree allowlist

`package/pi-plugin/auto-mode/src/index.ts:431-458` invokes linked-worktree discovery while handling each `read`,
before deciding whether the event needs guard evaluation:

```ts
const readAllowlistedDirs: readonly string[] = event.toolName === 'read'
  ? [
    ...trustedAgentTempDirs,
    ...(await linkedWorktreeReadAllowlistedDirs({ cwd: ctx.cwd, },)),
    ...currentSkillReadDirs,
  ]
  : [];

const flagged = await shouldFlag({
  event,
  ctx: signalCtx,
  readAllowlistedDirs,
  bashAllowlistedDirs,
},)
  || (denialInPreviousTurn && isRelevantTool(event,));
```

Every `read` event pays this cost,
including an event that `shouldFlag()` later accepts without review.
`linkedWorktreeReadAllowlistedDirs()` calls `resolveRealGit()` each time.
There is no process-level or session-level resolved-path cache in this call chain.

### The resolver launches checks for every PATH directory

`package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts:211-237` splits the complete `PATH`,
starts one candidate check per directory with `Promise.all()`,
and only then selects the first usable result:

```ts
export async function resolveRealGit({
  pathEnv = process.env
    .PATH
    ?? '',
}: ResolveRealGitOptions = {},): Promise<string> {
  moduleLogger.debug('resolving real git for linked worktree read allowlist',);

  const pathDirs = pathEnv.split(delimiter,);
  const candidateResults = await Promise.all(
    pathDirs.map(function resolveGitCandidateForDir(dir,) {
      return resolveGitCandidate(dir,);
    },),
  );
  const gitPath = candidateResults.find(function candidateIsAvailable(
    candidate,
  ): candidate is string {
    return candidate !== GIT_CANDIDATE_UNAVAILABLE;
  },);
```

`candidateResults.find()` preserves lookup order,
but it cannot cancel checks that `Promise.all()` already started.
Every package-local bin directory is therefore probed even after another candidate can resolve to real Git.

`package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts:260-286` builds `<PATH entry>/git`,
runs `fs.promises.access(..., X_OK)`,
and logs every handled miss:

```ts
const candidatePath = resolve(
  dir === ''
    ? process.cwd()
    : dir,
  'git',
);

try {
  await access(
    candidatePath,
    constants.X_OK,
  );
}
catch (error) {
  const innerL = tagged({
    tag: resolveGitCandidate.name,
    l: moduleLogger,
  },);
  innerL.debug(`git not executable at ${candidatePath}: ${String(error,)}`,);
  return GIT_CANDIDATE_UNAVAILABLE;
}
```

An absent `model-selection/node_modules/.bin/git` is thus an expected rejected candidate,
not an exceptional operation failure.

### Two refactors turned short-circuit lookup into visible fan-out

Commit `a0b7880365bb6142f10d3b39125a47003eaf9b31` introduced linked-worktree reads.
Before commit `d7f462ff08ab736a3929155b74816dafbe4a2487`,
its `packages/pi/auto-mode/src/git-worktree-read-allowlist.ts:223-248` implementation used a synchronous
`for...of` loop.
It stopped at the first usable real Git and silently continued past absent candidates:

```ts
for (const dir of pathDirs) {
  const candidatePath = resolve(
    dir === ''
      ? process.cwd()
      : dir,
    'git',
  );

  try {
    accessSync(
      candidatePath,
      constants.X_OK,
    );
  }
  catch {
    continue;
  }
```

Commit `d7f462ff08ab736a3929155b74816dafbe4a2487`
converted the filesystem work to asynchronous APIs and replaced that loop with `Promise.all()`.
Commit `facb9b1ee1e359bd52f861b0ff90b550ea8680e5`
then bound and logged handled catch values,
making every ordinary missing candidate visible.

The broad `PATH` makes the resolver's fan-out and per-candidate logging conspicuous,
but remains valid input.

### Resolution still succeeds

`package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts:239-241` returns the selected candidate:

```ts
if (gitPath !== undefined) {
  moduleLogger.debug(`resolved real git at ${gitPath}`,);
  return gitPath;
}
```

The reproduction resolved `/usr/bin/git`,
discovered linked worktrees,
and passed the affected unit suite.

`package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts:461-501` also shows that each read event runs
`git worktree list --porcelain`,
then runs a `git rev-parse` classification for each reported worktree:

```ts
const gitPath = await resolveRealGit();
const worktreeList = await readGitStdout({
  gitPath,
  cwd,
  args: [
    'worktree',
    'list',
    '--porcelain',
  ],
},);
// ...
const linkedRootDecisions = await Promise.all(
  worktreeRoots.map(function classifyWorktreeRoot(worktreeRoot,) {
    return isLinkedWorktreeRoot({
      gitPath,
      worktreeRoot,
    },);
  },),
);
```

`package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts:383-399` shows that each classification is a
`rev-parse` subprocess:

```ts
async function isLinkedWorktreeRoot({
  gitPath,
  worktreeRoot,
}: IsLinkedWorktreeRootOptions,): Promise<boolean> {
  const metadata = await readGitStdout({
    gitPath,
    cwd: worktreeRoot,
    args: [
      'rev-parse',
      '--path-format=absolute',
      '--is-inside-work-tree',
      '--git-dir',
      '--git-common-dir',
    ],
  },);
```

The observed impact is repeated candidate filesystem probing,
repeated real-Git metadata subprocesses,
and diagnostic noise.
It is not failure to locate Git.

### A better resolver already exists inside git-policy-cli

`package/git-policy/cli/src/resolve-git.ts:289-417` owns a more complete real-Git resolver.
It supports Windows `PATHEXT`,
recognizes additional self-shim forms,
prioritizes common platform Git paths that are present in `PATH`,
and scans sequentially:

```ts
/**
 * Locates real Git by prioritizing common platform paths present in PATH,
 * skipping candidates that delegate back into this package as detected by
 * {@link isShimForSelf}.
 *
 * Sequential scanning is intentional: we need first preferred match and can
 * stop immediately, so parallelizing would waste work.
 */
export async function resolveGit({
  // ...
}: ResolveGitOptions = {},): Promise<string> {
  // ...
  const candidates = new Set([
    ...exposedCommonGitPaths,
    ...pathCandidates,
  ],);

  for (const candidate of candidates) {
    try {
      await access(
        candidate,
        constants.X_OK,
      );

      if (await isShimForSelf(candidate,)) {
        rl.debug(`skipping self at ${candidate}`,);
        continue;
      }

      rl.debug(`resolved real git at ${candidate}`,);
      return candidate;
    }
```

Auto-mode duplicates git-policy-cli's package markers instead of using that owner boundary.
Direct reuse is not currently available through the package API:
`package/git-policy/cli/package.json:10-16` exports package root and `./ts`,
but `./ts` points only to `src/authoring.ts`.
Auto-mode also has no declared dependency on git-policy-cli.

Reuse is not behavior-neutral.
Auto-mode currently selects the first usable real Git in `PATH` order,
while git-policy-cli prioritizes exposed common platform paths.
A correction should expose or extract the owner resolver,
add the required workspace dependency or shared package,
and test the intended multiple-real-Git selection semantics.

## Verification

### Versions and source identities

- Workspace-installed Pi was `@earendil-works/pi-coding-agent` 0.84.3.
  The user's running Pi version was not sampled separately.
- Auto-mode was `@monochromatic-dev/pi-plugin-auto-mode` 0.0.1.
- Node.js was v26.7.0.
- The reproduction selected Git 2.55.0 at `/usr/bin/git`.
- mise was 2026.7.0.
- Repository commit was `05116cb7d058f2c53e13040980da96e201857908`.
- Auto-mode resolver source blob was `f3fa507a13b0c4090cefa35e18e52d94f530f1cc`.

### Runnable harness

From repository root:

```bash
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
MONOCHROMATIC_VERBOSE=true mise run //package/pi-plugin/auto-mode:test:unit -- \
  package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.unit.test.ts \
  2>&1 | tee "${HOME}/temp/agent/pi-auto-mode-git-probes.log"
rg --count 'git not executable at' \
  "${HOME}/temp/agent/pi-auto-mode-git-probes.log"
rg --count 'resolving real git|resolved real git at|skipping cli-git wrapper shim' \
  "${HOME}/temp/agent/pi-auto-mode-git-probes.log"
```

The 2026-08-27 run passed every test.
Its log contained 810 missing-candidate records,
five resolver entries,
five successful `/usr/bin/git` resolutions,
and ten self-shim skips.
These counts describe that exact mise task harness rather than every Pi process.
The test did not capture its effective `PATH` separately,
so candidate multiplicity and duplicate entries were not attributed further.

### Patterns that work cleanly

Without verbose console output,
the same unit task emitted only suite-level pass records.

With verbose output enabled,
these behaviors still completed successfully:

- cli-git wrapper candidates were identified and skipped;
- `/usr/bin/git` was selected;
- a linked worktree root was discovered;
- the main worktree root was excluded;
- reads into linked worktrees remained allowed while writes and secret-looking reads stayed guarded.

### Patterns that emit noise

- A `PATH` entry without a `git` executable emits one handled `ENOENT` debug record per resolver call.
- A package-manager shim delegating to git-policy-cli emits a skip record.
- Every `read` event enters linked-worktree discovery again and repeats real-Git resolution.
- Every successful discovery also starts real-Git worktree-list and per-worktree classification subprocesses.
- Parallel candidate checks make later `PATH` entries observable even though an earlier candidate ultimately wins.

No functional failure variant reproduced.
The diagnostic wording looks like a failed Git operation,
but the emitting operation is an executable-candidate `access()` check.

## Historical workaround

Leaving `MONOCHROMATIC_VERBOSE` unset suppresses these debug records from the console sink.
The non-verbose harness passed and omitted the candidate records from console output.
The tradeoff is loss of all debug and trace console visibility,
and the underlying filesystem checks still run.
Other configured logger sinks can still retain debug records.

There is no valid `PATH`-narrowing workaround.
The broad workspace-bin `PATH` is expected and needed for normal Pi operation.

The durable remediation was applied at the resolver ownership boundary.
The user confirmed common-path priority,
and `@monochromatic-dev/git-executable-core` now separates executable cache lifetime from mutable worktree metadata.

## What does not work

- Removing workspace package bin directories from Pi's `PATH` breaks intended command discovery.
  It misidentifies correct environment setup as the cause.
- Treating one `ENOENT` record as overall resolution failure ignores the later
  `resolved real git at /usr/bin/git` record.
- Changing only `Promise.all()` to a sequential scan still reaches many earlier package-bin entries because
  `/usr/bin` is intentionally late in `PATH`.
  Common-path prioritization or an owner-provided real-Git capability is needed to avoid those checks while
  preserving broad `PATH`.
- Suppressing the per-candidate log alone removes noise but leaves repeated filesystem work.
- Caching only the linked-worktree list forever risks stale results after worktrees are added or removed.
  Real-Git executable resolution and worktree metadata need separate cache lifetimes.
- Importing `@monochromatic-dev/git-policy-cli/ts` does not expose `resolveGit()`;
  that subpath currently exports policy authoring only.

## Upstream filing artifact

### Duplicate search

Open and closed issue and pull request searches in `Aquaticat/Monochromatic` for
`auto-mode git PATH` returned no matching report on 2026-08-27.

The `.out-of-scope/` directory has no matching Pi auto-mode or real-Git resolver exemption.

### Upstream filing decision

1. **Is it really upstream's fault?**
   No.
   Pi supplies the intended environment.
   This repository's auto-mode extension owns the repeated probe behavior.
2. **Can upstream fix it?**
   Pi upstream should not change the repository's required `PATH` or extension-local resolver.
   This repository can correct the resolver boundary.
3. **Are they supporting this use case?**
   Pi supports extension tool-call handlers,
   but real-Git selection for this guardrail is repository-owned behavior.
4. **Would the repo welcome our contribution?**
   The owning repository is already this repository.
   No external contribution policy is involved.
5. **Will they likely fix it?**
   Not applicable to Pi upstream because no upstream change is requested.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   No source fix was requested or applied.
   The existing git-policy-cli resolver proves that the repository has a sequential and platform-aware design,
   but its selection semantics differ and its package export does not expose that implementation.

Nothing should be filed against Pi upstream.
There is no external issue draft or additive comment to send.
