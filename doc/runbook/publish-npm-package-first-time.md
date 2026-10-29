# Publish a workspace package to npm for the first time

## What this proves

A package that has never existed on the npm registry gets its first version published under the owner's own account,
then a trusted publisher pointing at `.github/workflows/npm-release.yml`,
so every later version publishes from CI with provenance and no stored token.
Decision:
`doc/decision/npm-publishing.md`.
Written for `@monochromatic-dev/logger-core` 0.1.0;
substitute another package name and version for the next first release.

Bridges tried before writing this runbook:

- Trusted publishing (OIDC) for the first version:
   impossible,
   `npm trust` requires the package to already exist on the registry
   (<https://docs.npmjs.com/cli/v11/commands/npm-trust/>).
- A granular npm token stored as a repository secret:
   rejected by the decision record;
   npm's own guidance steers CI publishing away from Bypass-2FA tokens.
- Scripting the publish with `expect`:
   the one-time password comes from the owner's authenticator app,
   so no automation can supply it.

Everything up to the one-time password is a command you run;
the password prompt is the only manual input.

## Setup

Status:
TODO

Prerequisites:

- An npm account that owns the `@monochromatic-dev` scope,
   with two-factor authentication enabled and the authenticator app at hand.
- A clone of `https://github.com/Aquaticats/Monochromatic` with `mise` installed and activated
   and the toolchain provisioned (`mise run toolchain:sync`),
   so `mise exec -- npm --version` prints `11.19.0` or newer (`npm trust` needs 11.15.0 or newer)
   and `pnpm --version` prints `11.21.0` or newer.
- `gh` signed in to GitHub with access to the repository:
   `gh auth status` prints `Logged in to github.com`.
- The "Version Packages" pull request opened by the `npm-release` workflow,
   titled `chore(*): version packages`,
   still open.
   Land the changeset in the same push as the manifest change that drops `private`:
   a public manifest reaching `main` without a pending changeset makes the workflow try to publish the pre-release
   version through trusted publishing,
   which fails with `E404` because no trusted publisher exists yet
   (harmless,
   seen for `module-fs-path` on 2026-09-07).
   It bumps the package to its first version and carries the changelog and lockfile changes.
   If it was merged already,
   check out `main` instead of the pull request branch in step 1 and skip steps 12 and 13.

1. In the clone,
   run `git switch main` and `git pull`.
   Expected:
   `Already up to date.` or a fast-forward.

## Steps

Status:
TODO

1.   Find the version pull request:
     `gh pr list --search 'chore(*): version packages' --json number,headRefName`.
     Expected:
     one entry whose `headRefName` is `changeset-release/main`.
2.   Check it out:
     `gh pr checkout <number>`.
     Expected:
     `git branch --show-current` prints `changeset-release/main`.
     When the main worktree carries other sessions' uncommitted changes,
     do not switch its branch;
     check the branch out in a linked worktree instead
     (`git fetch origin changeset-release/main` then
     `git worktree add "${HOME}/temp/agent/<package>-release" origin/changeset-release/main`,
     `mise trust` inside it,
     and run steps 4 to 9 there;
     remove it with `git worktree remove` at the end).
     Done that way for `module-fs-path` on 2026-09-07.
3.   Confirm the version:
     `jq -r .version package/logger/core/package.json`.
     Expected:
     `0.1.0`.
4.   Install the workspace:
     `pnpm install --frozen-lockfile`.
     Expected:
     the last line starts with `Done in`.
5.   Link every direct dependency into the package
     (workaround for pnpm issue 9566,
     see `doc/troubleshooting/pnpm-pack-dedupe-direct-deps.md`):
     `env pnpm_config_dedupe_direct_deps=false pnpm install --force --filter @monochromatic-dev/logger-core`.
     pnpm 12.3.4 ignores the `--config.dedupe-direct-deps=false` form.
     Expected:
     `ls package/logger/core/node_modules/@monochromatic-dev` lists `config-typescript`.
6.   Build the package:
     `mise run //package/logger/core:build`.
     Expected:
     two lines ending in `Finished in` and `package/logger/core/dist/final/node/index.mjs` exists.
7.   Remove any stale compiler cache left from before `tsBuildInfoFile` was redirected:
     `rm -f package/logger/core/dist/final/types/tsconfig.tsbuildinfo`.
     Expected:
     no output.
8.   Pack:
     `mkdir --parents "${HOME}/temp/agent/npm-publish"` then
     `pnpm --filter @monochromatic-dev/logger-core pack --pack-destination "${HOME}/temp/agent/npm-publish"`.
     Expected:
     the output ends with a path ending in `monochromatic-dev-module-logger-0.1.0.tgz`.
9.   Inspect the packed manifest:
     `tar -xOzf "${HOME}/temp/agent/npm-publish/monochromatic-dev-module-logger-0.1.0.tgz" package/package.json | jq -c '{version, exports, dependencies}'`.
     Expected:
     `{"version":"0.1.0","exports":{".":{"types":"./dist/final/neutral/index.d.mts","node":"./dist/final/node/index.mjs","default":"./dist/final/neutral/index.mjs"}},"dependencies":{}}`
     with no `./ts` key and no `workspace:` string.
10.  Sign in to npm:
     `mise exec -- npm login`.
     Expected:
     a browser opens for the login,
     then the terminal prints `Logged in on https://registry.npmjs.org/.`;
     `mise exec -- npm whoami` prints your npm user name.
11.  Publish the tarball without provenance (provenance needs a CI runner):
     `mise exec -- npm publish "${HOME}/temp/agent/npm-publish/monochromatic-dev-module-logger-0.1.0.tgz" --access public --no-provenance`.
     Expected:
     a prompt `This operation requires a one-time password.`;
     type the six digits from the authenticator app and press **Enter**;
     the output ends with `+ @monochromatic-dev/logger-core@0.1.0`.
     From a shell without a terminal (an agent's command tool),
     pass the code instead:
     `--otp=<six digits>`;
     the code is good for about 30 seconds,
     so run the command right after reading it.
12.  Register the release workflow as the trusted publisher:
     `mise exec -- npm trust github @monochromatic-dev/logger-core --repo Aquaticats/Monochromatic --file npm-release.yml --allow-publish`.
     Expected:
     a confirmation prompt,
     answer **y** (or pass `--yes`),
     then a line confirming the trusted publisher for `Aquaticats/Monochromatic` and `npm-release.yml`.
     Since 2026-10-29 the owner releases through `.github/workflows/npm-publish.yml` instead
     (one package per manual dispatch, input `package`, for example `logger/core`);
     `logger-core` and `fs-path-core` trust that workflow,
     registered with the same command and `--file npm-publish.yml`.
     A package can list up to 10 trusted publishers
     (<https://docs.npmjs.com/trusted-publishers>),
     so registering both workflows is allowed.
     This command and `npm trust list` open a browser for npm's web authentication every time,
     even with a logged-in token,
     so they must run from your own terminal;
     an agent's command tool only prints
     `Open this URL in your browser to authenticate`.
13.  Wait for the registry index to replicate:
     `npm view @monochromatic-dev/logger-core version` until it prints `0.1.0`.
     Expected:
     a brand-new package's version document appears at once,
     but the package index (`https://registry.npmjs.org/@monochromatic-dev%2fmodule-logger`) can answer 404 for several
     minutes.
14.  Merge the version pull request:
     `gh pr merge <number> --squash`.
     Expected:
     `✓ Squashed and merged pull request #<number>` and the branch is deleted automatically.
15.  Watch the release run that the merge triggers:
     `gh run list --workflow npm-release.yml --limit 1 --json databaseId --jq '.[0].databaseId'`
     then `gh run watch <id> --exit-status`.
     Expected:
     the run succeeds but does nothing for this version.
     With the version already on the registry and no changeset pending,
     `select-mode` reports nothing to version or publish,
     so the pack and publish jobs are skipped and no tag is created.
     (If the index had not replicated yet,
     the run instead plans a fresh publish that no-ops against the existing version;
     the outcome is the same.)
16.  Create the bootstrap version's tag and release through the GitHub API,
     in the shape the publish job uses for later versions
     (a plain `git push` of the tag fails in the cli-git wrapper,
     see `doc/troubleshooting/cli-git-tag-push-eagain.md`):
     `gh release create '@monochromatic-dev/logger-core@0.1.0' --target "$(git rev-parse <merge commit of the version pull request>)" --title '@monochromatic-dev/logger-core@0.1.0' --notes-file <file holding the 0.1.0 section of package/logger/core/CHANGELOG.md>`.
     Expected:
     the release URL is printed,
     and `gh release view '@monochromatic-dev/logger-core@0.1.0' --json tagName --jq .tagName` prints the tag name.
17.  Fetch the tag GitHub created:
     `git fetch --tags origin`.
     Expected:
     `git tag --list '@monochromatic-dev/logger-core@*'` prints `@monochromatic-dev/logger-core@0.1.0`.

## What to check

Status:
TODO

- `npm view @monochromatic-dev/logger-core version` prints `0.1.0`.
- `npm view @monochromatic-dev/logger-core dist.tarball` prints a `registry.npmjs.org` URL.
- `mise exec -- npm trust list @monochromatic-dev/logger-core` lists one GitHub entry with
   `Aquaticats/Monochromatic` and the workflow registered in step 12.
- `gh release view '@monochromatic-dev/logger-core@0.1.0' --json tagName --jq .tagName`
   prints `@monochromatic-dev/logger-core@0.1.0`.
- In a disposable directory:
   `mkdir -p /tmp/logger-consumer && cd /tmp/logger-consumer && npm init -y >/dev/null && npm install @monochromatic-dev/logger-core`
   ends with `added 1 package`,
   and
   `node --input-type=module -e "import { logger } from '@monochromatic-dev/logger-core'; logger.info('hello from the registry'); await logger.flush();"`
   prints a line ending in `hello from the registry`.
- The next version publishes from CI:
   add a changeset (`mise run changeset:add`),
   merge it,
   merge the resulting version pull request,
   and the `publish` job log for that run contains `+ @monochromatic-dev/logger-core@0.1.1`
   (or whatever version the changeset produced) with a provenance attestation shown by
   `npm view @monochromatic-dev/logger-core dist.attestations.url`.
   Verified on 2026-09-06 with 0.2.0:
   the `publish` job succeeded,
   `npm view @monochromatic-dev/logger-core@0.2.0 dist.attestations.url` printed the attestation URL,
   and the workflow itself created the tag and GitHub release `@monochromatic-dev/logger-core@0.2.0`,
   so the manual tag step in the steps section is bootstrap-only.

## Restore

Status:
TODO

1.   Return the workspace to its default dependency layout:
     from the repository root run `pnpm install`.
     Expected:
     the last line starts with `Done in`.
2.   Remove the packed tarball:
     `rm -rf "${HOME}/temp/agent/npm-publish"`.
     Expected:
     no output.
3.   Remove the disposable consumer:
     `rm -rf /tmp/logger-consumer`.
     Expected:
     no output.
4.   Optionally sign out of npm on this machine:
     `mise exec -- npm logout`.
     Expected:
     no output,
     and `mise exec -- npm whoami` prints `ENEEDAUTH`.
