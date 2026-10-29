# cli-git policies that add paths to a commit

## Status

Implementation plan for the owner decision recorded 2026-09-15 in
 `doc/decision/private-npm-registry.md` ("Versions change only by manual bumps"):
 extend the cli-git commit transaction so a policy pass can add paths to the commit
 and write those fixes to the worktree when the worktree copy still matches `HEAD`.
The first consumer is the dependent-bump policy.

Implemented 2026-09-15 on `main`;
 see "Implementation state" for what landed and what remains.

## Requirements carried from the decision

- A hand bump of a workspace manifest `version` bumps every dependent by patch,
   transitively,
   in the same commit.
- An edge counts when the dependent reaches the bumped package through `dependencies`,
   `peerDependencies`,
   or `optionalDependencies`,
   or imports it from non-test source as a bundled `devDependency`.
- The dependent's manifest joins the commit,
   and its worktree copy is rewritten so `git status` stays clean.
- SPEC,
   `doc/decision/cli-git-policies-platform.md`,
   and the fixtures are updated for every commit mode.
- The changesets `version` job in `npm-release.yml` runs the same ripple (Task 8),
   so the ripple computation must be callable outside cli-git.
- No CI backstop.

## Current engine limits (measured 2026-09-15)

- Policies see only candidates:
   `LazyPolicyGitFacts` in `package/git-policy/cli/src/api/context-types.ts` exposes
   `candidates`,
   `headOid`,
   `landedCommitOid`,
   and `pushUpdates`;
   nothing lists or reads other tracked files.
- A patch must name a pass-start candidate:
   `applyPolicyPatches` (`package/git-policy/cli/src/policy-engine/apply-policy-patches.ts` lines 99 to 112)
   returns `patchTargetFailure` for any other target.
- `runCommitTransaction` (`package/git-policy/cli/src/policy-engine/commit-transaction.ts`)
   computes `candidatePaths` once before the convergence loop
   and passes the same list to `preparePostIndex` and `prepareTransactionJournal` as `selectedPaths`.
- Patches apply with `git apply --cached --3way` to the private index only;
   `SPEC.md` "Transaction protocol" says never mutate the real index or worktree while evaluating policies,
   and no commit phase writes the worktree.
- `executePreparedCommit` (`commit-transaction-finalize.ts`) journals `recordRefUpdated`,
   installs the post index,
   then `recordIndexInstalled`;
   recovery knows no worktree phase.
- Direct fix (`direct-fix.ts`,
   `direct-fix-install.ts`) already installs fixed bytes into the worktree atomically for its pathspec scope.

## Design

### Policy API

- `trackedFiles: (request: Readonly<{ pathspecs: readonly string[] }>) => Promise<readonly TrackedFile[]>`
   on `LazyPolicyGitFacts` lists index entries of the current candidate state matching Git pathspecs,
   glob magic included,
   each with an opaque `targetId`,
   `path`,
   `revision`,
   `mode`,
   `headRevision`,
   and lazy `bytes` and `headBytes`.
   It replaced a first sketch,
    `headFiles` over the `HEAD` tree,
    because `git ls-tree` rejects `:(glob)` pathspec magic
    (`fatal: ... pathspec magic not supported by this command: 'glob'`,
     measured 2026-09-15)
    and because reading the current index spares policies from merging candidates over `HEAD`.
- A `PolicyPatch` whose `targetId` names a tracked file that is not a candidate asks the engine to add that path.
- Post-commit and manual-push lifecycles return no tracked files;
   only `pre-forward` commit transactions and direct fix apply added-path patches.

### Commit transaction

1.   Precondition per added path,
      checked when the patch applies:
      the private commit index entry equals the `HEAD` entry,
      and the worktree file's bytes and mode equal the `HEAD` blob.
      Otherwise the engine fails the pass,
       naming the path and the two remedies
       (stage the path so it becomes a candidate,
        or restore or stash its worktree changes).
2.   The patch applies to the private commit index as today;
      the path then differs from `HEAD`,
      so the next pass sees it as a candidate.
3.   `candidatePaths` becomes the union of the selected paths and every added path,
      recomputed after each changed pass.
4.   Explicit-path mode passes the union as `selectedPaths` to `preparePostIndex`,
      so the post-commit index takes added paths from the landed tree.
      Index mode already copies the private commit index.
5.   The journal records each added path with its original and intended blob OIDs and mode before real Git runs.
6.   After `recordIndexInstalled`,
      the engine rewrites each added path's worktree file
      (same-directory temporary file plus rename,
       mode preserved),
      re-checking that the file still holds the original bytes first,
      before the transaction directory is removed.
      No separate marker exists:
       the journal staying behind is what tells startup recovery to finish.
7.   Read-only selection modes (`--interactive`,
      `--patch`,
      `--include`) keep refusing automatic patches,
      so a needed bump blocks with direct-fix guidance.

### Recovery

A journal left behind after the index install checks each added path:
 intended bytes means done,
 original bytes means write the intended bytes,
 and anything else is kept with a warning naming the path,
 as in the normal path.

### Dependent-bump policy

- Lives in `package/git-policy/repository` as `dependent-version-bump`,
   registered by `repositoryPolicyPlugin`,
   so the repository's `mono` namespace enables it at its default `error` severity without a `cli-git.config.ts` change.
- Publishable manifests are the names listed in `package/config/pnpr/config.yaml`
   (the publish workflow's own source of truth);
   their directories come from `package/*/*/package.json`.
- Bumped packages are publishable manifests whose candidate `version` differs from the `HEAD` version.
- Edges come from the merged current manifests;
   a bundled `devDependency` edge needs an import of the package name,
    or a subpath of it,
    in a file under the dependent's `src/` that is not a `*.test.*` file.
- The policy computes the whole transitive closure in one pass
   and patches every dependent whose current `version` still equals its `HEAD` version,
   so the second pass is stable instead of spending one pass per graph level against the eight-pass limit.
- A patch bump increments the third component;
   no workspace manifest used a prerelease version on 2026-09-15 (0 of 155),
   and 8 had no `version` field and are skipped.
- The pure closure computation is a separate exported module,
   so the `npm-release.yml` changesets job can run it through a package task (Task 8).

### Adopted sub-decisions

- The engine,
   not the policy,
   checks that an added path is unchanged,
   because policies cannot see the real index or the worktree.
- A worktree copy edited while the commit runs is kept
  with a warning instead of blocking every later Git command through recovery.
- A dependent whose worktree manifest differs from `HEAD` and is not staged blocks the commit.
   This follows from the rejected option in the decision
   (adding paths without updating the worktree leaves reverted versions in `git status`).
- Only publishable manifests are bumped;
   edges still pass through non-publishable packages,
   so a publishable package bundling an unpublished one that depends on the bumped package still bumps.

## Fixtures to add

For each of ordinary staged commit,
 explicit-path commit,
 `--no-only`,
 amend,
 allow-empty,
 and merge,
 cherry-pick,
 and revert conclusions:
 a hand bump adds a clean dependent manifest,
 and the test asserts exact ref,
 index,
 and worktree bytes.
Also:

- dirty dependent worktree manifest blocks with the named remedies;
- staged dependent manifest is patched as a candidate,
   not added;
- `--interactive` and `--patch` block with direct-fix guidance;
- transitive chain longer than eight levels settles in two passes;
- interruption after index install and before worktree install recovers each of the three byte states;
- direct fix adds and writes a clean dependent manifest.

## Implementation state

Landed:

- `trackedFiles` in `package/git-policy/api` and its cli-git copy,
   implemented by `commit-transaction-tracked-files.ts` for private-index facts.
- `applyPolicyPatches` resolves tracked targets through `commit-transaction-added-paths.ts`,
   and `runCommitTransaction` carries added paths into later passes,
    the post-commit index,
    the journal,
    finalize,
    and recovery.
- `mono/dependent-version-bump` in `package/git-policy/repository`,
   with pure helpers shared by Task 8
   (`dependent-version-bump.ts`,
    `manifest-text.ts`,
    `source-imports.ts`,
    `publishable-names.ts`).
- `SPEC.md` and `doc/decision/cli-git-policies-platform.md` updated.
- Packed fixtures,
   run in `node:24-slim` through the packed shim:
   `built-autofix-added-paths-consumer.ts`
    (explicit-path,
     `--no-only`,
     amend,
     unstaged and staged conflicts,
     `--include` refusal;
     it fails when the worktree install is removed)
   and `built-dependent-version-bump-consumer.ts`
    (runtime and bundled dependents bumped,
     test-only importer untouched,
     clean status).

Remaining:

1.   Closed 2026-09-15 by owner decision ("Implement it now"):
      direct-fix convergence passes added-path context with lifecycle `direct-fix` (`b20d65fc5`),
      tracks added paths through `createAddedPathTracker`,
      shared with commit transactions (`921062c87`,
       which also brought `commit-transaction.ts` back under `max-lines`),
      and installs added paths' converged bytes into the worktree with their verified `HEAD` bytes as originals.
      `built-autofix-added-paths-direct-fix.ts` (`9b9129e15`) covers a clean unselected path,
      a refused dirty one,
      and the same path once selected;
      removing the context failed it with `git cli-git fix -- version.txt expected 0, got 2`,
      and removing the added originals failed it with `Direct-fix candidate became unavailable: dependent.txt`.
      `dependentVersionBump` also declares `direct-fix` (`5d2519e85`,
       owner decision "Enable it for fix"),
      so `git cli-git fix -- <bumped manifest>` rewrites dependent manifests in the worktree
      and leaves the real index untouched;
      `built-dependent-version-bump-consumer.ts` covers it.
2.   Closed in `a37690a0d`:
      `built-autofix-added-paths-recovery.ts` kills the wrapper from `pre-commit` and `post-commit` hooks
      and simulates an installed index,
      then asserts recovery leaves an uncommitted dependent alone
      and writes the committed dependent to index and worktree.
      Ablating either `installAddedWorktreeFiles` call in `commit-transaction-recovery.ts`
      made the packed fixture fail
      (`commit-created recovery worktree added path: expected bumped, got stale`
       and `index-already-installed recovery worktree added path: expected bumped, got stale`).
3.   Closed in `a37690a0d`:
      `built-autofix-added-paths-conclusion.ts` covers merge,
      cherry-pick,
      and revert conclusions.
4.   Closed 2026-09-15:
      the repository's trusted bundle predated the policy
      (`git cli-git check --policy mono/dependent-version-bump --all` reported
       `Unknown built-in policy ID: mono/dependent-version-bump`).
      The owner delegated re-trust;
      `git cli-git trust --yes` succeeded,
      and with it the owner-configured recursive stage covering repositories beneath the repository root.
      Afterwards the same check exited 0,
      while `--policy mono/no-such-policy` still failed with `Unknown built-in policy ID`,
      so the bundle registers the policy.
      Its firing on a real hand bump in this repository is not yet observed;
      `check` has no revision-range mode,
      so observing it needs a modified manifest,
      and the packed `built-dependent-version-bump-consumer.ts` fixture is the functional evidence.

Task 8 landed:
 root task `changeset:version` runs `mise run //package/git-policy/repository:bump:dependents`
 between `changeset version` and the lockfile refresh.
The runner reads the worktree through `bump-dependents-worktree.ts`
 and plans through `dependent-bump-workflow.ts`,
 the module the policy also uses.
`bump-dependents-worktree.unit.test.ts` covers a runtime and a bundled dependent in a disposable repository,
 plus the no-bump case.
A disposable clone of `main` with `@monochromatic-dev/or-throw-core` hand-bumped planned 19 dependents
 and changed only their version lines.
The CI job itself has not run the ripple yet;
 the first Version Packages pull request after a pending changeset is its first real exercise.
