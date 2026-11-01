# actions/checkout v6.0.2 with `fetch-depth: 2` drops the pre-push head of a multi-generation push, and the cargo-publish detect step read that as a version bump for every crate

## Symptom

Run 37531066191 of `.github/workflows/cargo-publish.yml`,
 the push of `main` from `173e5fe2a` to `533574e1f` on 2026-10-06,
 was meant to publish only `monochromatic-jsonc-edit` 0.1.1.
Every detect job instead decided to publish.
The `detect` job for `forbidden-strings` logged:

```text
BEFORE_SHA: 173e5fe2ab687b48e87bcce31729eba8f01c19cc
before=
after=0.4.1
```

The jobs that followed for crates whose version had not changed then failed:

```text
publish-crate	Publish	error: crate forbidden-strings@0.4.1 already exists on crates.io index
nws-create-release	Create GitHub release with all archives	a release with the same tag name already exists: monochromatic-nested-wayland-session-v0.1.1
create-release	Create GitHub release with all archives	a release with the same tag name already exists: forbidden-strings-v0.4.1
```

What the run changed outside the repository:

- crates.io:
   only the intended upload,
   `monochromatic-jsonc-edit` 0.1.1.
   `forbidden-regex` and `monochromatic-nested-wayland-session` skipped their existing versions,
   and `forbidden-strings` 0.4.1 was refused as a duplicate.
- GitHub releases:
   nothing,
   because the repository's immutable releases refuse an existing tag.
- Build provenance:
   13 attestations for bytes that were never published,
   in addition to the intended one for `monochromatic-jsonc-edit-0.1.1.crate`.
   They cover the rebuilt `forbidden-strings` 0.4.1 archives for all eight targets,
   the rebuilt `monochromatic-nested-wayland-session` 0.1.1 archives for both targets,
   and repackaged `.crate` files of `forbidden-strings` 0.4.1,
   `forbidden-regex` 0.1.1 and `monochromatic-nested-wayland-session` 0.1.1.
   None of the three `.crate` digests equals the checksum crates.io lists for that version,
   so no consumer download can match them.

A cancel of the run was attempted and refused by the session's permission classifier,
 so the jobs ran to completion.

## Root cause

The detect step reads the crate version at `github.event.before` from the checkout.
Before 2026-10-06 it did so only when that commit was already present
 (`.github/workflows/cargo-publish.yml`, `detect` job, as of `533574e1f`):

```bash
before=''
if [ -n "${BEFORE_SHA:-}" ] \
  && [ "$BEFORE_SHA" != '0000000000000000000000000000000000000000' ] \
  && git cat-file -e "${BEFORE_SHA}:${cargo_toml}" 2>/dev/null; then
  before=$(git show "${BEFORE_SHA}:${cargo_toml}" | awk -F'"' '/^version = / { print $2; exit }')
fi
```

An absent commit left `before` empty,
 and an empty `before` differs from any `after`,
 so `publish=true`.
The checkout comment called this degrading gracefully on a first push or a force push.

The checkout is shallow.
`actions/checkout` at the pinned commit `de0fac2e4500dabe0009e67214ff5f5447ce83dd` (tag `v6.0.2`)
 defines the input in `action.yml:74`:

```yaml
  fetch-depth:
    description: 'Number of commits to fetch. 0 indicates all history for all branches and tags.'
    default: 1
```

and the run's checkout issued:

```text
/usr/bin/git -c protocol.version=2 fetch --no-tags --prune --no-recurse-submodules --depth=2 origin +533574e1f10113bff7817f4bb0e2b308790a6d37:refs/remotes/origin/main
```

`--depth=2` fetches the pushed commit and its parents,
 one generation back.
The pushed commit `533574e1f` was a merge whose parents were the branch commit `184ee5baf`
 and the local `main` head `21f2c625d`,
 which carried 16 commits another session had not pushed yet.
The pre-push head `173e5fe2a` was a parent of `184ee5baf`,
 two generations back,
 so it was not in the checkout,
 `git cat-file -e` failed,
 and every crate looked bumped.
A push that moves `main` by exactly one commit never shows this,
 because its `before` is that commit's parent.

The checkout action behaves as documented;
 the defect is the detect step's assumption that `before` is always within reach.

## Verification

Harness:
 the script below,
 saved as `detect-sim.ts` outside the repository,
 builds the same topology in a disposable repository:
 a base commit,
 two commits on local `main`,
 a bump commit in its own worktree,
 and a merge of `main` into it.
It pushes the merge to a local bare repository with `uploadpack.allowReachableSHA1InWant`,
 fetches it with `--depth=2` as the action does,
 and runs each workflow's detect step under `bash` with `BEFORE_SHA` set to the base.
Run it from the repository root,
 after `git show <old commit>:.github/workflows/cargo-publish.yml > cargo-publish.old.yml`,
 as
 `node detect-sim.ts cargo-publish.old.yml .github/workflows/cargo-publish.yml node_modules/.pnpm/yaml@2.9.1/node_modules/yaml/dist/index.js`.
It writes its sandbox under `${HOME}/temp/agent/linter-cutover-20261006/`,
 which must exist.

```ts
// detect-sim.ts
// Reproduces the 2026-10-06 cargo-publish incident in a disposable repository and checks the
// patched detect step: a push whose pre-push head lies outside a depth-2 checkout.
// Usage: node detect-sim.ts <old-workflow.yml> <new-workflow.yml> <yaml-module-path>
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const [oldWorkflowPath, newWorkflowPath, yamlModulePath] = process.argv.slice(2);
if (oldWorkflowPath === undefined || newWorkflowPath === undefined || yamlModulePath === undefined) {
  throw new Error('usage: detect-sim.ts <old-workflow.yml> <new-workflow.yml> <yaml-module-path>');
}
const { parse } = await import(pathToFileURL(yamlModulePath).href);

function git(cwd: string, args: readonly string[]): string {
  return execFileSync('git', args, { cwd, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}

function detectScript(workflowPath: string, job: string): string {
  const workflow = parse(readFileSync(workflowPath, 'utf8'));
  const step = workflow.jobs[job].steps.find(function isDetect(candidate: { id?: string }) {
    return candidate.id === 'detect';
  });
  if (step === undefined) throw new Error(`no detect step in ${job}`);
  return step.run;
}

const root = mkdtempSync(join(process.env.HOME ?? '', 'temp/agent/linter-cutover-20261006/detect-sim-'));
const work = join(root, 'work');
mkdirSync(join(work, 'package/jsonc-edit/core-rust'), { recursive: true });
mkdirSync(join(work, 'package/forbidden-strings/cli'), { recursive: true });
git(root, ['init', '--quiet', '--initial-branch=main', work]);
git(work, ['config', 'user.email', 'sim@example.invalid']);
git(work, ['config', 'user.name', 'sim']);
const subject = 'package/jsonc-edit/core-rust/Cargo.toml';
const bystander = 'package/forbidden-strings/cli/Cargo.toml';
writeFileSync(join(work, subject), 'version = "0.1.0"\n');
writeFileSync(join(work, bystander), 'version = "0.4.1"\n');
git(work, ['add', '--', subject, bystander]);
git(work, ['commit', '--quiet', '--message', 'base', '--', subject, bystander]);
const base = git(work, ['rev-parse', 'HEAD']);
// Local main moves two generations past origin's head, as the unpushed desktop-app-ide commits did.
for (const label of ['side one', 'side two']) {
  writeFileSync(join(work, 'side.txt'), `${label}\n`);
  git(work, ['add', '--', 'side.txt']);
  git(work, ['commit', '--quiet', '--message', label, '--', 'side.txt']);
}
// The bump lives on a branch in its own worktree, then merges local main, as the landing did.
const feat = join(root, 'feat');
git(work, ['worktree', 'add', '--quiet', '-b', 'feat', feat, base]);
writeFileSync(join(feat, subject), 'version = "0.1.1"\n');
git(feat, ['add', '--', subject]);
git(feat, ['commit', '--quiet', '--message', 'bump', '--', subject]);
git(feat, ['merge', '--quiet', '--no-ff', '--no-edit', 'main']);
const pushed = git(feat, ['rev-parse', 'HEAD']);

const bare = join(root, 'bare.git');
git(root, ['init', '--quiet', '--bare', bare]);
git(bare, ['config', 'uploadpack.allowReachableSHA1InWant', 'true']);
git(work, ['push', '--quiet', bare, `${pushed}:refs/heads/main`]);

const clone = join(root, 'clone');
git(root, ['init', '--quiet', clone]);
git(clone, ['remote', 'add', 'origin', pathToFileURL(bare).href]);
git(clone, ['fetch', '--quiet', '--no-tags', '--depth=2', 'origin', `+${pushed}:refs/remotes/origin/main`]);
git(clone, ['checkout', '--quiet', '--detach', pushed]);
const baseInClone = spawnSync('git', ['cat-file', '-e', `${base}^{commit}`], { cwd: clone }).status === 0;
console.log(`base ${base.slice(0, 9)} present in depth-2 clone before detect: ${baseInClone}`);

function runDetect(label: string, workflowPath: string, job: string, beforeSha: string): void {
  const output = join(root, `${label}.out`);
  writeFileSync(output, '');
  const result = spawnSync('bash', ['-c', detectScript(workflowPath, job)], {
    cwd: clone,
    encoding: 'utf8',
    env: { ...process.env, BEFORE_SHA: beforeSha, EVENT_NAME: 'push', INPUT_CRATE: '', GITHUB_OUTPUT: output },
  });
  const printed = result.stdout.trim().replaceAll('\n', ' ');
  const decided = readFileSync(output, 'utf8').trim().replaceAll('\n', ' ');
  console.log(`${label}: exit=${result.status} ${printed} -> ${decided || '(no output)'}`);
}

runDetect('old-je', oldWorkflowPath, 'je-detect', base);
runDetect('old-forbidden-strings', oldWorkflowPath, 'detect', base);
// The old run leaves the clone untouched, so the new script sees the same depth-2 state.
runDetect('new-forbidden-strings', newWorkflowPath, 'detect', base);
runDetect('new-je', newWorkflowPath, 'je-detect', base);
runDetect('new-missing-before', newWorkflowPath, 'detect', '1111111111111111111111111111111111111111');
console.log(`sandbox: ${root}`);
```

Output on 2026-10-06:

```text
base 2a2368ea8 present in depth-2 clone before detect: false
old-je: exit=0 before= after=0.1.1 -> should_publish=true version=0.1.1
old-forbidden-strings: exit=0 before= after=0.4.1 -> should_publish=true version=0.4.1
new-forbidden-strings: exit=0 before=0.4.1 after=0.4.1 -> should_publish=false
new-je: exit=0 before=0.1.0 after=0.1.1 -> should_publish=true version=0.1.1
new-missing-before: exit=128  -> (no output)
```

### Pushes the old detect step handles

- A push of one commit,
   merge or not,
   whose parent is the pre-push head.
- A first push with `before` all zeros,
   which publishes by design.

### Pushes the old detect step misreads

- A push of two or more generations,
   such as a merge whose other parent carries unpushed commits,
   or a plain push of several commits:
   every crate in the workflow is treated as bumped.

## Verified workarounds

The detect steps now fetch the missing commit before reading it:

```bash
before=''
if [ -n "${BEFORE_SHA:-}" ] \
  && [ "$BEFORE_SHA" != '0000000000000000000000000000000000000000' ]; then
  if ! git cat-file -e "${BEFORE_SHA}^{commit}" 2>/dev/null; then
    git fetch --no-tags --depth=1 origin "$BEFORE_SHA"
  fi
  if git cat-file -e "${BEFORE_SHA}:${cargo_toml}" 2>/dev/null; then
    before=$(git show "${BEFORE_SHA}:${cargo_toml}" | awk -F'"' '/^version = / { print $2; exit }')
  fi
fi
```

Tradeoffs:
 one extra fetch of a single commit when a push spans generations;
 a `before` that cannot be fetched,
 for example after a force push that GitHub has already collected,
 now fails the detect job instead of publishing,
 so such a push needs a manual dispatch.
A crate whose manifest did not exist at `before` still counts as bumped,
 which is the first-publication case.

A landing procedure can also avoid the trigger:
 when local `main` holds unpushed commits,
 push it on its own before fast-forwarding it to a merge,
 so each push spans one generation.

## What does not work

- `fetch-depth: 0`:
   fetches all history for a decision that needs one file at one commit,
   and the repository's history is large.
- Comparing against `HEAD^1`:
   for a merge landed by fast-forward the first parent is the branch,
   not the old `main`,
   so a bump made on the branch would be missed.
- A partial clone (`--filter=blob:none`) as a reproduction:
   a missing commit is fetched lazily from the promisor remote,
   so the absence never shows.
   The first attempt at this reproduction made that mistake and reported the base commit as present.

## Upstream filing decision

1. Upstream's fault:
   no.
   `actions/checkout` fetches the documented number of commits;
   the assumption that `before` is within them was this repository's.
2. Upstream can fix it:
   not applicable.
3. Supported use case:
   not applicable.
4. Contribution welcome:
   not checked,
   since nothing is filed.
5. Likely to fix:
   not applicable.
6. Prototyped fix:
   the fix is in this repository's workflow,
   verified by the harness.

`.out-of-scope/` was not consulted because no upstream filing is proposed.
Nothing to file.

## Depot preparation cannot inspect its immutable base after default shallow checkout

On 2026-10-16,
Depot run `1v4l9tglbt` stopped before dependency preparation or repository checks.
The trial task asked Git to verify its fixed source base:

```text
# trial/depot-complete-ci-20261016, commit 50fbf6983171067eda7199bc21f5c31edfcbc5ce
git merge-base --is-ancestor bae6ef28a183039a6c4ba509179dc4baaedeac2e HEAD
fatal: Not a valid commit name bae6ef28a183039a6c4ba509179dc4baaedeac2e
```

The workflow used the same pinned checkout action documented in this report,
but omitted `fetch-depth`.
The action's `action.yml` was fetched again at
`de0fac2e4500dabe0009e67214ff5f5447ce83dd`;
its default remains `1`.
The trial required an ancestor object that the workflow had not requested.
This is a trial configuration error,
separate from the publishing decision described in this report.

Trial commit `3fda67d325bc646f1c6b2cc5385542c4863bac78`
adds `fetch-depth: 0` to `.depot/workflows/depot-complete-ci.yml`.
Its tradeoff is retrieving repository history for the ancestry check.
Unlike the publishing check,
this task explicitly verifies ancestry before comparing immutable inputs.
The corrected environment-only run is `tg907s5kfs`;
it completed successfully.
The preparation record at `2026-10-16T02:26:38.274Z`
reports matching hashes for all four immutable inputs,
`elapsedMs: 22986`,
and `checksExecuted: false`.
The repository tracking issue is
[1057](https://github.com/Aquaticats/Monochromatic/issues/1057).
The original failed run executed no quality,
build,
or test checks and must not count as CI coverage.

No upstream filing is justified:
the checkout action applied its documented default.
This incident requires a repository trial-harness correction,
not an upstream patch.
