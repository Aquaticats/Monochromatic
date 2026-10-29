# npm publishing for workspace packages

## Status

Accepted 2026-09-06.
Owner-authorized during the module-logger release grilling recorded in
`doc/planning/module-logger-release.md`.
Supersedes the "bundle private helpers into public artifacts rather than publishing them"
direction in `doc/handover/cli-git-policies-platform.md` and the framing of issue #338.

## Context

No package under `@monochromatic-dev` has ever been published to npm.
Five public manifests depend on the private `module-logger`,
so any of them published as-is would point consumers at a package that does not exist.
The old direction was to bundle the logger into each consumer's artifact.
That leaves the `dependencies` entries in place,
which pnpm rewrites to a version nobody can install,
and it makes every consumer carry a private copy of shared code.
Publishing the shared modules is the direction that scales.

## Decisions

### Publish shared modules; module-logger goes first

`module-logger` is the first package released,
as groundwork.
Its consumers in `module-test`,
`module-toml-edit`,
and `dev-script-watch-restart` keep working through workspace links;
`module-test` stays blocked on the private `module-fs-path` until that package gets the same treatment.
`module-fs-path` followed on 2026-09-07 (`doc/planning/module-fs-path-release.md`),
 after a platform split that mirrors the logger's;
`module-test` then waits on `module-async-time`,
 `module-caught-value`,
 `module-const`,
 and `module-numeric-format`.

### Versioning and publishing tool: changesets

`@changesets/cli` with the `changesets/action` sub-actions,
vetted in `doc/audit/tech-changesets-release-tooling-vet-2026-09-06.md`.
Alternatives discovery was waived by the owner;
the fallback if changesets fails at the publish boundary is a hand-run publish from a pnpm-packed tarball.
`.changeset/config.json` is written by hand (`changeset init` is interactive with no flag),
with `access: public`,
`baseBranch: main`,
and every other option at its default,
so private packages are neither versioned nor tagged.

### Authentication: trusted publishing, never a stored token

npm trusted publishing (OIDC) authenticates every CI publish.
No `NPM_TOKEN` secret exists or will be created;
npm's own access-token guidance steers CI publishing away from Bypass-2FA tokens,
and `gh secret list` confirms none is stored.
Only the publish job holds `id-token: write`.

### Bootstrap: the first publish of each package is local and interactive

`npm trust` requires the package to already exist on the registry,
so trusted publishing cannot bootstrap.
The first version of each package is packed with `pnpm pack`
(which rewrites `workspace:` specifiers and applies `publishConfig`)
and published with `npm publish <tarball> --access public` with provenance disabled,
authenticated by the owner's one-time password.
Immediately afterwards `npm trust github` registers the release workflow.
That first version has no provenance;
every later version does.
Runbook:
`doc/runbook/publish-npm-package-first-time.md`.

### The never-run `publish.yml` is deleted

It authenticated with a token that does not exist,
had no build step,
and listed ten packages nobody had readiness-checked.
The changesets release workflow replaces it.
Issue #307 closes by construction;
issue #306's build and tarball checks move to the release workflow.
Issue #287's stale `module-numeric-const` filter
(the package renamed to `module-const` in `9e571fc5c4` on 2026-05-28)
disappears with the file;
`changeset:build` fails on any publish-plan name the workspace does not carry.

### Published surface is the built artifact; `./ts` is stripped at publish

Node refuses `.ts` files under `node_modules`,
so a published `./ts` subpath serves bundler users only and is never exercised by this repo's tests.
`publishConfig.exports` overrides `exports` at publish time without the `./ts` entries;
in-repo `/ts` imports are unaffected because workspace links read the real `exports` field.
Workspace dependencies that the dist bundles move to `devDependencies`.

### Test files ship; compiler state does not

Test files stay in tarballs (mixed ecosystem practice:
zod and pino ship them,
fast-check does not).
`tsconfig.tsbuildinfo` leaves every tarball by redirecting `tsBuildInfoFile`
outside `dist/final` in the shared TypeScript config.

### Runtime floor: `engines.node >=24`

Built artifacts call `Error.isError`,
which arrived with V8 13.6 in Node 24.0.0.
Declaring the floor beats a polyfill:
a Node 22 consumer fails at install with a clear message instead of at the first caught error.

### First version 0.1.0

Produced by a `minor` changeset from 0.0.1.
Caret ranges on 0.x treat every minor as breaking,
which matches the state of the contract.

### The release workflow runs only when a release input changes (2026-09-06)

`npm-release.yml` used to run on every push to `main`.
With a changeset pending,
 every run regenerated the `changeset-release/main` branch
and force-pushed one fresh `chore(*): version packages` commit
onto the open Version Packages pull request,
and GitHub mailed the owner a push notification each time:
198 commits landed on `main` on 2026-09-06 and the workflow ran at least 40 times that day.
The owner ruled one run per commit unsustainable.

The `push` trigger is now filtered to the paths that can change what the flow does:
`.changeset/**` (a new or consumed changeset opens,
 updates,
 or closes the Version Packages pull request),
`**/package.json` and `**/CHANGELOG.md` (what a merged Version Packages pull request edits,
 so publishing still triggers),
and the workflow file itself.
`workflow_dispatch` stays for a manual run.
Accepted consequence:
the Version Packages branch is no longer rebased on every unrelated `main` commit;
the action rebases it on its next relevant run,
and the branch only ever touches manifests,
 changelogs,
 and changeset files.

### Releases run through `npm-publish.yml` (2026-10-29)

After the category moves renamed `module-logger` and `module-fs-path` to `logger-core` and `fs-path-core`,
the owner directed:
"just do new releases through npm-publish.yml now".
`.github/workflows/npm-publish.yml` is a manual dispatch that builds,
tests,
packs,
and publishes one package (input `package`) with provenance.
Both renamed packages list it as their trusted publisher
and published `logger-core` 0.4.6 and `fs-path-core` 0.2.2 through it.
`npm-release.yml` (changesets) still runs on its path filter;
its `select-mode` failed on every run until `33b6a407a5` ignored the `translation-repair-*` family,
and with no changeset pending it versions and publishes nothing.
A package may list up to 10 trusted publishers
(<https://docs.npmjs.com/trusted-publishers>),
so moving a package back to the changesets flow only needs `npm trust github` with `--file npm-release.yml`.

## Consequences

- Public packages may depend on published workspace modules;
   the graph check proposed in issue #338 should reject only edges to packages still marked private.
- Every first publish is a manual owner step;
   every later publish is a merged "Version Packages" pull request.
- Consumers outside the workspace get the built artifact only.
