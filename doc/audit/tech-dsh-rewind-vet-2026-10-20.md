# dsh-rewind vet 2026-10-20: `SiriLee/dsh-rewind` 0.17.1 for the dsh web profile

- Status:
  validated and scored;
  conditional recommendation with one named prerequisite;
  adoption not authorized.
- Lifecycle phase:
  8 (Recommended) under the single-subject scope below;
  phase 9 (Adopted) is not requested and no product or profile change was made.
- Subject:
  `dsh-rewind`,
  npm package `dsh-rewind-plugin@0.17.1`,
  repository `SiriLee/dsh-rewind` at revision `6e627cc0fa4e9a0991db8398ca1967bc361e98f8`
  (tag `v0.17.1`).
- Scope:
  pre-install vet of the named candidate for the DeepSeek Harness web
  profile on the runtime in service (`dsh` 0.2.1-alpha.4),
  answering "is it safe and is it compatible with dsh 0.2.1-alpha.4".
- Start date:
  2026-10-20.
- Last-updated date:
  2026-10-20.
- Governing skill:
  `.agents/skills/choosing-technology/SKILL.md`,
  commit `37117e36397e30233062828581a5e38217da9458`.
- Governing skill SHA-256:
  `552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.
- Compatibility fingerprint (RFC 8785 canonical input,
  SHA-256):
  `223236c84106b9a8ea175f23cbe4d90ee6c2b6b3f5f0eb3387e4ff7e1b5567dc`.
- Active audit owner:
  `lead` (this session).
- Hard constraints:
  1.  Installs and activates on the runtime in service without a version
   exemption.
  2.  Public inspectable source under an OSI license.
  3.  No runtime network endpoints.
  4.  No runtime credential or secret access.
  5.  Session history never rewritten in place.
  6.  Web profile support.
- Base category:
  inspectable open-source local technology
  (a plugin running inside another tool).
- Overlays:
  high-trust agent plugin execution;
  sensitive data (workspace files and session history).
- Frozen criteria and weights:
  all weight 1,
  because the workspace owner named no priorities
  (see "Score arithmetic").
- Unresolved preferences:
  none for the vet itself.
  Adoption authority stays with the workspace owner,
  and the runtime-upgrade prerequisite below is theirs to authorize.
- Prior compatible report:
  none.
  `doc/audit/` holds no `dsh-rewind` report,
  so this is a new report.

## Context

The capability record
[doc/troubleshooting/dsh-web-sent-message-edit-unavailable.md](../troubleshooting/dsh-web-sent-message-edit-unavailable.md)
establishes that `dsh` itself offers no edit action on sent messages and
names community plugins as the only edit-and-rerun path.
The workspace owner chose "vet its source first",
so this report audits `SiriLee/dsh-rewind` before anything of it runs in
the harness profile.

Measured deployment facts:

- runtime in service:
  `dsh --version` prints `0.2.1-alpha.4`,
  installed from the npm `alpha` channel at
  `/var/home/user/.local/share/mise/installs/npm-deepseek-ai-dsh/alpha/v11/7256-18df34f15ddc07b6-0`;
- target profile:
  `web`
  (the DeepSeek Harness Web GUI at `http://127.0.0.1:3080`);
- host:
  Linux 7.2.9-ogc3.1.fc44.x86_64,
  x86_64;
- trust boundary:
  a third-party plugin would execute inside the harness host and web
  client,
  read workspace files the agent's tools modify,
  write a snapshot store under the dsh data directory,
  and append markers to session logs;
- incumbent:
  none
  (first-party queue edit,
  steer,
  and fork cover only part of the wanted
  behavior and are not a rewind capability).

## Decision scope and deviations

The authorizing request is a single-candidate pre-install vet,
not a selection across the plugin field.
The field was discovered and screened,
and every candidate the discovery found is listed with its screening
outcome,
but equal-depth validation was applied to the named subject only.
Consequences recorded here so nothing is silently skipped:

- No field-wide winner is claimed and unvetted alternatives are not
  ranked.
  Choosing among the field would need a separate request and equal-depth
  validation of each finalist.
- The report was committed once at synthesis,
  not after every audit phase;
  the audit ran in one pass in one session.
- The consumer-boundary step that installs the plugin into a real dsh
  profile was not run,
  because the workspace owner deferred all execution in their harness.
  Its substitutes are recorded in "Validation results".

## Query schedule and discovery

Queries were frozen after one expansion round.
Provider for repository searches is GitHub search
(`gh search repos`),
sorted by best match,
no include or exclude filters,
full result pages read.

- `dsh rewind`
  (GitHub repositories):
  10 results,
  9 new candidates
  (`XSJUSTC/dsh-rewind`,
  `Anionex/dsh-turn-rewind`,
  `extracurricular-ai/dsh-filesnap`,
  `L0251314/dsh-rewind`,
  `dpskh/dsh-rewind`,
  `Locietta/dsh-rewind-retry`,
  `ddll8023/dsh-rewind`,
  `jiangr100/dsh-rewind`,
  `wyq09/dsh-rewind`).
- `deepseek harness plugin`
  (GitHub repositories):
  10 results,
  ecosystem indexes discovered
  (`awesome-dsh-plugin/awesome-dsh-plugin`,
  `imsai-sh/awesome-deepseek-harness-plugins`,
  `losebird/dsh-plugin-market`),
  1 new candidate class
  (plugin directories,
  used for corroboration only).
- `dsh edit prompt`
  (GitHub repositories):
  0 results.
- `dsh checkpoint rewind`
  (GitHub repositories,
  expansion round):
  3 results,
  3 new candidates
  (`PerryLink/dsh-checkpoint-rewind`,
  `skkaczsh/dsh-checkpoint-rewind`,
  `tmpdot/dsh-checkpoint-diff`).
- GitHub topic `dsh-plugin`
  (`gh search repos --topic dsh-plugin`),
  20 results,
  ecosystem corroboration,
  no additional rewind candidates.
- Discussions search
  (`search(type: DISCUSSION)`,
  `edit message` in `deepseek-ai/deepseek-harness`,
  10 results):
  candidates and community reports already recorded in the
  troubleshooting record,
  including `SiriLee/dsh-rewind` announced by its author and the
  `Myoontyee/deepseek-harness` fork prototype.
- This repository's incumbent and prior records
  (`doc/audit/`,
  `doc/decision/`,
  `doc/troubleshooting/`):
  no prior rewind-plugin evaluation.

New taxonomy terms recorded without further query rounds:
  `rewind`,
`checkpoint`,
`redo`,
`change ledger`,
`file snap`.

Terminal discovery result:
  saturated with more than two survivors.

## Candidate ledger

Each entry is one subsection:
  candidate,
  discovery source,
  base category,
  overlays,
  screening result.

### `SiriLee/dsh-rewind` (subject)

- Source:
  `dsh rewind` search and discussion 9422.
- Category:
  inspectable open-source local plugin (MIT).
- Overlays:
  high-trust plugin execution,
  sensitive data.
- Screening:
  survives;
  promoted to the audit subject by the authorizing request.

### `Locietta/dsh-rewind-retry`

- Source:
  `dsh rewind` search.
- Category:
  inspectable open-source local plugin.
- Overlays:
  high-trust plugin execution.
- Screening:
  survives on metadata
  (rewind by forking the session,
  edit-prompt and retry actions).
  Not deep-validated under this scope,
  so it is named as the leanest visible alternative and left unranked.

### `Anionex/dsh-turn-rewind`

- Source:
  `dsh rewind` search.
- Category:
  inspectable open-source local plugin.
- Overlays:
  high-trust plugin execution,
  sensitive data.
- Screening:
  survives on metadata
  (conversation and workspace rewind via a persistent change ledger).
  Not deep-validated under this scope.

### `extracurricular-ai/dsh-filesnap`

- Source:
  `dsh rewind` search.
- Category:
  inspectable open-source local plugin with a Rust core.
- Overlays:
  high-trust plugin execution,
  native artifact boundary,
  sensitive data.
- Screening:
  survives on metadata,
  but the native overlay requires source-to-artifact mapping for its Rust
  core,
  which was not inspected in this scope.
  Left unranked with that gate pending.

### `XSJUSTC/dsh-rewind`

- Source:
  `dsh rewind` search.
- Category:
  inspectable open-source local plugin.
- Overlays:
  high-trust plugin execution.
- Screening:
  survives on metadata.
  Not deep-validated under this scope.

### `wyq09/dsh-rewind`

- Source:
  `dsh rewind` search.
- Category:
  inspectable open-source local plugin.
- Overlays:
  high-trust plugin execution.
- Screening:
  survives on metadata.
  Not deep-validated under this scope.

### `L0251314/dsh-rewind`

- Source:
  `dsh rewind` search.
- Category:
  inspectable open-source local plugin.
- Overlays:
  high-trust plugin execution.
- Screening:
  exits on platform fit.
  The description scopes it to the DeepSeek Harness desktop client
  ("桌面端插件"),
  while the deployment target here is the web profile.

### `dpskh/dsh-rewind`

- Source:
  `dsh rewind` search.
- Category:
  inspectable open-source local plugin.
- Overlays:
  high-trust plugin execution.
- Screening:
  exits on category fit.
  Its behavior is fold-since-checkpoint reporting,
  not message rewind or edit-and-rerun.

### `ddll8023/dsh-rewind`

- Source:
  `dsh rewind` search.
- Category:
  derivative fork of the subject.
- Overlays:
  high-trust plugin execution.
- Screening:
  exits as a stale derivative
  ("Modified dsh-rewind plugin",
  last update 2026-08-15 against the subject's 2026-10-19 release),
  so it adds no independent code path to audit.

### `jiangr100/dsh-rewind`

- Source:
  `dsh rewind` search.
- Category:
  prompt-level skill replica
  ("Replica of the Claude Code /rewind skill").
- Overlays:
  none beyond prompt execution.
- Screening:
  exits on category fit for this request
  (the request is for an installable plugin with code to audit).

### `PerryLink/dsh-checkpoint-rewind`, `skkaczsh/dsh-checkpoint-rewind`

- Source:
  `dsh checkpoint rewind` expansion query.
- Category:
  inspectable open-source local plugins,
  git-first or git-worktree checkpointing.
- Overlays:
  high-trust plugin execution,
  git state boundary.
- Screening:
  exit on the deployment shape.
  Both require a git-managed workspace and restore through git or linked
  worktrees,
  while the wanted behavior must also cover non-git workspaces
  (the subject's file store is git-independent).

### `tmpdot/dsh-checkpoint-diff`

- Source:
  `dsh checkpoint rewind` expansion query.
- Category:
  inspectable open-source local plugin.
- Screening:
  exits on category fit
  (read-only diff visualization over another plugin's checkpoints).

### `Myoontyee/deepseek-harness` fork prototype

- Source:
  discussion 9127.
- Category:
  customized harness fork,
  not an installable plugin.
- Screening:
  exits on category fit for an install-and-run request
  (adopting it replaces the harness runtime,
  a different decision).

## Hard-gate outcomes

### Category fit

Pass.
  The subject is a dsh plugin package
  (`package.json` `dsh.bundle.patch` and `dsh.client` declarations)
  built to mount in a harness profile.

### Open-source precedence

Pass.
  MIT licensed,
  public source,
  no proprietary alternative in the set.

### License and terms

Pass.
  `LICENSE` is MIT;
  npm metadata `licenseInfo.key` is `mit`.

### Provenance and inspectability

Pass.
  npm Trusted Publishing via GitHub OIDC
  (`publish.yml` sets `id-token: write` and publishes with no
  `NODE_AUTH_TOKEN`),
  the published artifact carries SLSA provenance
  (`dist.attestations.provenance.predicateType`
  `https://slsa.dev/provenance/v1`) and two npm signatures,
  and a local rebuild of tag `v0.17.1` reproduces the published tarball
  byte-identically
  (see "Validation results").

### Security boundary

Pass for the named constraints,
  with the inherent feature risk named in "Pros and cons".
  Runtime code has no network calls and no credential reads;
  the only network use in the repository is a CI badge script
  (`scripts/update-badge.mjs`,
  GitHub gist update with `GIST_TOKEN`) that is not part of
  `npm run check` and is not shipped
  (`package.json` `files` ships `lib`,
`cordis.patch.yml`,
`README*`,
  `SECURITY.md`,
`LICENSE`,
`locale/*.json`,
`assets/icon.svg`).

### Native, Wasm, and prebuilt artifacts

Not applicable to the subject.
  The runtime artifact is plain JavaScript
  (host half ESM,
  client half CJS),
  built by esbuild and `tsc` from the audited sources;
  no native or Wasm code is shipped.

### Platform fit

Pass.
  `dsh.client.platform` declares `web`;
  CI runs Node 22.19.0 and 24 on ubuntu-latest;
  `engines.node` is `^22.19.0 || >=24.0.0`.

### Target-runtime compatibility (hard constraint 1)

Fail on the runtime in service,
  pass after the named prerequisite.

- Fail evidence:
  `package.json` `engines.dsh` is `>=0.2.1-rc.1 <0.3.0-0`,
  and the runtime in service is `0.2.1-alpha.4`,
  which is lower in semver order and in publish time
  (npm `time`:
  `0.2.1-alpha.4` at 2026-10-15T13:08:08Z,
  `0.2.1-rc.1` at 2026-10-17T03:17:24Z).
  The built client half requires `@deepseek-ai/dsh-client-ui-primitives`
  at runtime,
  and no such package exists in the installed alpha.4 tree.
- Prerequisite:
  upgrade the harness to `>= 0.2.1-rc.1`
  (npm `latest` is `0.2.1-rc.2`,
  which is also the exact version line the plugin's dev dependencies pin),
  then the engine range matches and the module graph contains the
  required client package.

## Evidence records

### E1 engine range excludes the runtime in service

- Claim:
  `dsh-rewind-plugin@0.17.1` cannot satisfy `engines.dsh` on
  `dsh` 0.2.1-alpha.4.
- Relevance:
  hard constraint 1.
- Status:
  fail on the current runtime.
- Primary source:
  `SiriLee/dsh-rewind` `package.json` at `6e627cc`,
  `engines.dsh`;
  npm registry `dsh-rewind-plugin` metadata and
  `@deepseek-ai/dsh` `time`,
  accessed 2026-10-20.

```json
// package.json:engines
"engines": {
  "node": "^22.19.0 || >=24.0.0",
  "dsh": ">=0.2.1-rc.1 <0.3.0-0"
}
```

- Corroboration:
  the project's own issue history shows the same coupling
  (issue 43 "需要适配v0.2.0-rc.1",
  issue 40 schemastery API break at import time,
  open issue 52 "0.2.0-rc2无法正常启用").

### E2 the client half requires a module the runtime tree lacks

- Claim:
  the built client bundle calls
  `require("@deepseek-ai/dsh-client-ui-primitives")`,
  and that package is absent from the installed alpha.4 tree.
- Relevance:
  hard constraint 1,
  and the practical load failure behind it.
- Status:
  fail on the current runtime.
- Commands:
  `grep` of the container-built `lib/client.js` for external requires;
  `ls` of
  `/var/home/user/.local/share/mise/installs/npm-deepseek-ai-dsh/alpha/v11/7256-18df34f15ddc07b6-0/node_modules/.pnpm`
  filtered for `primitives` and `slots`.
- Output excerpt:

```text
require("@deepseek-ai/dsh-client-ui-primitives")
require("react")
require("react-dom")
require("react/jsx-runtime")
# pnpm store filter for primitives|slots: no results
```

- Counter-evidence weighed:
  `@deepseek-ai/dsh-client-ui-primitives` does publish `0.2.1-alpha.4`
  on npm,
  so the absence is a packaging choice of the alpha.4 app graph,
  not a missing upstream package.
  Either way the web profile cannot resolve the module id on alpha.4.

### E3 every other imported symbol exists in the alpha.4 dist

- Claim:
  the plugin's remaining `@deepseek-ai/*` imports
  (`Session`,
  `SessionId`,
  `FileSystem`,
  `resolveDshHome`,
  `createUserMessage`,
  `CommandInvocation`,
  `PostToolDecision`,
  `WorkingDirectoryService`,
  `Agent`,
  `ISessions`,
  `SessionFace`,
  `CommandUiContract`,
  `UserMessageNode`,
  `ClientSessionContext`,
  `ConfigForm`,
  `SettingsFormModel`,
  `Volatile`,
  and the rest)
  all resolve in the installed alpha.4 type and runtime sources.
- Relevance:
  scopes E1 and E2 to the one real gap.
- Status:
  pass.
- Command:
  `node ${HOME}/temp/agent/compat-probe.mjs`
  (extracts `@deepseek-ai/*` imports from `src/` and `src/client/`,
  greps each symbol against the installed `lib/types` and `lib` of the
  matching pnpm package).
- Output excerpt:
  `--- symbols NOT found in installed alpha.4 dist --- (none)`.
- Note:
  `@deepseek-ai/dsh-client-ui-slots` is imported type-only
  (`import type` plus `declare module`),
  so its absence from the alpha.4 tree is not a runtime defect,
  only a typecheck-time dependency satisfied by the plugin's own dev
  dependencies.

### E4 no runtime network, no credentials, no process execution

- Claim:
  runtime code opens no sockets,
  reads no secrets,
  and spawns nothing.
- Relevance:
  hard constraints 3 and 4.
- Status:
  pass.
- Command:
  `grep -rn -E 'child_process|spawn\(|execFile|exec\(|fetch\(|node:net|node:tls|process\.env|eval\(|new Function' src scripts`.
- Output excerpt:
  the only hits are `scripts/build.mjs`
  (`execSync('npx tsc ...')`,
  build-time),
  `scripts/update-badge.mjs`
  (`process.env.GIST_TOKEN`,
  `fetch(https://api.github.com/gists/...)`,
  CI-only),
  and regex `.exec()` matches that are not calls to `exec`.

### E5 session history is appended, never rewritten

- Claim:
  the rewind works by appending producer-marked events and cutting the
  visible surface,
  not by editing the log.
- Relevance:
  hard constraint 5.
- Status:
  pass.
- Source:
  `src/marker.ts:1-23`
  (declares the `dsh-rewind` `MessageSourceMap` kind),
  `src/snapshot.ts:1-40`
  (checkpoint store design),
  `scripts/verify-host.mjs:1-45`
  (verified behaviors 3,
  12 to 15:
  "cuts the surface in-place (log untouched)",
  compaction transactions stay balanced,
  no dangling `step/start` or `turn/start` frames).

```ts
// src/marker.ts:14-16
declare module '@deepseek-ai/dsh-llm' {
  interface MessageSourceMap {
    'dsh-rewind': { kind: 'dsh-rewind' }
  }
}
```

### E6 file effects are bounded and git-independent

- Claim:
  the snapshot store backs up the before-state of tracked writes under the
  dsh data directory,
  keeps the newest 100 anchor groups per session,
  restores the earliest recorded before-state per path,
  and never writes through symlinks or hard links.
- Relevance:
  sensitive-data overlay;
  the feature's inherent destructiveness.
- Status:
  pass with the named tradeoff.
- Source:
  `src/snapshot.ts:32-40`
  (restore semantics),
  `src/snapshot.ts:42-50`
  (store root `resolveDshHome()` plus `rewind-snapshots`),
  `tests/snapshot-bytes.test.ts`
  (byte-exact 1 MiB binary round-trip),
  `tests/parent-guard.test.ts`
  (path traversal guards).

### E7 the plugin manager reproduces the incompatibility

- Claim:
  `dsh plugin add` refuses `dsh-rewind-plugin@0.17.1` on
  `dsh` 0.2.1-alpha.4 and accepts it on `dsh` 0.2.1-rc.2.
- Relevance:
  hard constraint 1;
  turns E1 from static range arithmetic into observed behavior.
- Status:
  fail on the runtime in service,
  pass on the prerequisite runtime.
- Commands:
  `DSH_HOME=/work/dsh-home-alpha dsh plugin --profile vet add dsh-rewind-plugin`
  under `dsh` 0.2.1-alpha.4,
  and the same call under `dsh` 0.2.1-rc.2 with
  `DSH_HOME=/work/dsh-home-rc`,
  both in the manifested container image with `pnpm` added per the
  manifest update below.
- Output excerpt:

```text
dsh: installation rejected: Plugin dsh-rewind-plugin@0.17.1 is incompatible with dsh 0.2.1-alpha.4: requirements {"engines.dsh":">=0.2.1-rc.1 <0.3.0-0"}. Running it may cause crashes or data loss. Update the plugin or install a plugin version compatible with this dsh runtime. ... Exact-version exemption: not active.
dsh: nothing was installed.
```

```text
dependencies:
+ dsh-rewind-plugin 0.17.1
Done in 1.1s using pnpm v12.14.0
```

- Exits:
  0.2.1-alpha.4 exit 1,
  0.2.1-rc.2 exit 0.
- Logs:
  `${HOME}/temp/agent/dsh-rewind-run/add-alpha2.log` and
  `add-rc2.log`.

## Execution manifest

Recorded before running third-party code.

- Candidate and revision:
  `SiriLee/dsh-rewind` at `6e627cc0fa4e9a0991db8398ca1967bc361e98f8`
  (tag `v0.17.1`),
  cloned with
  `gh repo clone SiriLee/dsh-rewind ${HOME}/temp/agent/dsh-rewind-2026-10-20 -- --depth 50`.
- Artifact checksums:
  npm `dsh-rewind-plugin@0.17.1`
  `integrity sha512-Cq0VuqkITUUWQ7WEQsD26370twbwiwn/A5EycE9CrHu9QPe5ewmNXkZLq9OQVept5X/d6y4LFcmqIL8D7Ijkng==`,
  `shasum 6fc7a017c7bd128e67f094e5bee24cf279736029`.
- Top-level commands:
  `npm ci --no-audit --no-fund`
  then `npm run check`
  (= `npm run typecheck`,
  `npm run build`,
  `npm test`,
  `node scripts/verify-host.mjs`,
  `npm pack --dry-run`).
- Statically discovered command tree:
  `scripts/build.mjs` runs `npx tsc -p tsconfig.build.json` and
  `npx tsc -p tsconfig.client.json` via `execSync`
  (lines 60 to 61);
  `vitest run` executes 38 test files under jsdom and node environments;
  `scripts/verify-host.mjs` boots the built plugin on a real cordis
  context with real `@deepseek-ai/dsh-*` packages and writes only under
  `mkdtemp(join(tmpdir(), 'dsh-rewind-verify-'))`
  (line 75);
  `scripts/update-badge.mjs` is outside `npm run check` and was not run.
- Files inspected to establish the tree:
  `package.json`,
  `scripts/build.mjs`,
  `scripts/verify-host.mjs`,
  `scripts/update-badge.mjs`,
  `.github/workflows/ci.yml`,
  `.github/workflows/publish.yml`,
  `cordis.patch.yml`,
  `src/` and `tests/` listings.
- Expected reads and writes:
  reads of the cloned tree and the npm registry cache;
  writes confined to the work directory
  (`lib/`,
  `node_modules/`,
  `.npm-cache/`,
  `.tmp/`,
  and the pack dry-run listing);
  no writes to the host dsh directory or the repository.
- Network endpoints:
  npm registry during the fetch phase only.
- Container and limits:
  `podman run --rm --memory=2g --cpus=2`,
  image `docker.io/library/node:24`,
  repo digest
  `sha256:14078204b6bb268c107ceff6d3c43f45a888105e5c2b18b14c32b3409d93a1b0`,
  host Linux x86_64;
  work directory bind-mounted from
  `${HOME}/temp/agent/dsh-rewind-run`
  (a copy of the clone with `.git` removed).
- Credential,
  environment,
  home,
  mount policy:
  no ambient credentials passed,
  no home directory mounted,
  no repository mounted,
  npm cache inside the work directory,
  network disabled for the validation phase
  (`--network none`).
- Success,
  failure,
  cleanup,
  stop conditions:
  success is `npm run check` exit 0;
  failure keeps the logs for inspection;
  containers are `--rm`;
  the scratch copy is disposable and separate from the audit clone.

Execution-protocol deviation to note:
  the dependency fetch and the validation ran as two container
  invocations in one background shell,
  not as two separately manifested sessions.
  The install-gate demo also revealed an undeclared subprocess:
  `dsh plugin add` invokes `pnpm`,
  which the first demo image lacked
  (exit 127,
  `dsh: pnpm was not found`).
  The manifest was updated with `npm install -g pnpm` inside the same
  bounded image,
  and the demo was rerun to completion as recorded in E7.

## Validation results

All on the pinned revision in the manifested container.

- `npm ci`:
  exit 0,
  log `${HOME}/temp/agent/dsh-rewind-run/ci.log`;
  `prepare` ran `npm run build` as expected.
- `npm run typecheck`:
  pass
  (three tsconfig projects,
  host,
  client,
  client-test).
- `npm run build`:
  pass;
  host half bundles to ESM with `@deepseek-ai/*` external,
  client half to CJS wrapped in
  `window.__ModuleLoader__.load`.
- `npm test`:
  pass,
  38 of 38 test files,
  duration 16.45 s,
  including `crash-safety` (23 tests),
  `downgrade-safety` (11),
  `snapshot-bytes` (30,
  byte-exact 1 MiB binary round-trip),
  `parent-guard` (15),
  `compat-invariants` (24),
  `session-integration` (5).
- `node scripts/verify-host.mjs`:
  `verify-host: all checks passed`,
  covering 16 named behaviors
  (command registration,
  `/rewind` withdrawal,
  surface cut with the log untouched,
  checkpoint commit and deny paths,
  file restore and created-file deletion,
  running-agent force-stop,
  cancel timeout,
  subagent edit exclusion,
  real `/compact` interop,
  compaction-boundary refusal,
  no dangling frames,
  per-file size cap).
- `npm pack --dry-run`:
  28 files,
  `shasum 6fc7a017c7bd128e67f094e5bee24cf279736029`,
  `integrity sha512-Cq0VuqkITUUWQ[...]`,
  which matches the published npm artifact exactly,
  so the audited source is the published source.
- Consumer boundary:
  not exercised in a real dsh profile,
  per the owner's deferral.
  Substitutes used:
  `verify-host.mjs` boots the plugin against real harness packages,
  and the built artifacts' external-module graph was checked against the
  installed alpha.4 tree
  (E2,
  E3).

## Human-auditability measurements

- Runtime code:
  17 source files,
  8,907 lines
  (`src/` and `src/client/`).
- Tests:
  41 files,
  10,139 lines,
  which outnumber the runtime code.
- Runtime dependencies:
  zero
  (`package.json` declares no `dependencies`).
- Peer dependencies:
  17,
  all optional,
  all resolved from the harness installation itself
  (`@deepseek-ai/*` plus `@deepseek-ai/schemastery`),
  so no third-party runtime package is introduced.
- Dev dependencies:
  build and test only
  (esbuild,
  typescript,
  vitest,
  jsdom,
  react,
  plus the harness packages
  pinned at 0.2.1-rc.2).
- Boundaries:
  one bundle-patch row
  (`cordis.patch.yml` inserts the single id `dsh-rewind-plugin`),
  one host command (`/rewind`),
  tool hooks (`tools/execute`,
  `tools/post-execute`),
  one settings card,
  client portals and one popover.
- Concentration:
  credentials none;
  network none at runtime;
  process execution none at runtime;
  filesystem activity concentrated in `src/snapshot.ts`
  (1,300 lines),
  which reads workspace files,
  writes the snapshot store under `resolveDshHome()/rewind-snapshots`,
  and performs restores
  (including deletions of files created after the rewind target).

Feature breadth versus verifiability:
  the plugin is deliberately narrow
  (one command,
one UI affordance,
one store),
  and its risk concentrates in the restore path,
  which is also where its tests are densest.

## Score arithmetic

Criteria and weights were frozen before any candidate rating.
The owner named no priorities,
  so every weight is 1.

Criteria:

1.  Target-runtime compatibility
  (hard constraint 1 evaluated in context B,
  after the prerequisite upgrade).
2.  Supply-chain provenance and reproducibility.
3.  Runtime safety
  (network,
    credentials,
    log integrity,
    file effects).
4.  Feature fit
  (edit-and-rerun plus optional file restore in one window).
5.  Human auditability
  (size,
    dependency surface,
    test depth).
6.  Maintenance responsiveness.

Ratings for `dsh-rewind-plugin` 0.17.1
  (context B,
  after upgrading dsh to 0.2.1-rc.2):

- Target-runtime compatibility:
  3,
  confidence medium.
  Engine range matches and the module graph is the one CI tests;
  the live profile load is unexercised here.
- Supply-chain provenance:
  4,
  confidence high.
  OIDC trusted publishing,
  SLSA provenance,
  and a byte-identical local rebuild of the published tarball.
- Runtime safety:
  3,
  confidence medium.
  No network,
  no credentials,
  append-only history,
  bounded store,
  symlink guards;
  the restore path is destructive by design and only statically and
  unit-tested here.
- Feature fit:
  4,
  confidence high.
  It implements exactly the wanted UX
  (withdraw message and tail,
  refill composer,
  regenerate,
  optional file restore),
  as its verify-host behaviors show.
- Human auditability:
  3,
  confidence high.
  8.9k runtime lines against 10.1k test lines,
  zero runtime dependencies,
  one patch row;
  the snapshot store is the one dense module.
- Maintenance responsiveness:
  3,
  confidence medium.
  Release v0.17.1 on 2026-10-19,
  external pull requests merged,
  issues triaged with fixes
  (file-size cap after issue 39,
  command-name collision handling after issue 49);
  the harness's churn forces repeated adaptation
  (issues 40,
  43,
  52),
  and concentration rests mainly on one maintainer.

```text
# .agents/skills/choosing-technology/SKILL.md
earned   = 3 + 4 + 3 + 4 + 3 + 3 = 20
maximum  = 6 * 4 = 24
score    = 20 / 24 * 100 = 83.3
```

Context A
  (the runtime as it stands,
  dsh 0.2.1-alpha.4)
  has no scored candidate,
  because hard constraint 1 fails before arithmetic
  and hard gates cannot be offset by points.

## Sensitivity

With one validated subject there is no adjacent order to defend against a
peer,
  so the sensitivity matrix tests whether any single input flips the
  recommendation between the subject and the no-plugin baseline.

- Weights raised one at a time from 1 to 5:
  the subject stays above the baseline in context B,
  because the baseline scores 0 on feature fit and the subject scores at
  least 3 on every criterion.
- Medium-confidence ratings
  (compatibility,
  safety,
  maintenance)
  moved one step down and up within 0 to 4:
  the one-step-down case for all three still leaves the subject above the
  baseline;
  the only conclusion that changes is the strength of the recommendation,
  not its direction.
- Low-signal ranges:
  none were recorded,
  so no endpoint test applies.

Stability statement:
  no one-at-a-time change reverses the recommendation in context B.
  This does not cover simultaneous multi-input changes.
  Context A is unchanged by any rating,
  because the gate is hard.

## Pros and cons

### `dsh-rewind-plugin` 0.17.1 (after the prerequisite)

Pros:

- Delivers the wanted behavior in one window:
  withdraw a sent user message and its tail from the model context and the
  view,
  refill the composer,
  regenerate.
- Optional file restore covers the side effects first-party tooling leaves
  behind,
  without requiring git.
- Append-only history:
  rewind markers,
  never a rewritten log,
  which matches the durability boundary in the troubleshooting record.
- Supply chain is unusually strong for a community plugin:
  OIDC publishing,
  SLSA provenance,
  byte-identical rebuild,
  zero runtime dependencies.
- Test depth is high
  (38 files,
  crash and downgrade safety,
  byte-exact backups,
  real `/compact` interop).

Cons:

- Not installable on the runtime in service:
  engines range and the missing `dsh-client-ui-primitives` module both
  require a harness upgrade to `>= 0.2.1-rc.1` first.
- The restore path deletes files created after the rewind target,
  which is inherent to the feature and still destructive when a rewind is
  chosen carelessly.
- It chases a fast-moving harness API,
  and its issue history shows import-time and enablement breakage across
  harness prereleases.
- Community-maintained:
  one main maintainer,
  no vendor support,
  and the compatibility contract is whatever the latest release declares.
- Subagent edits are deliberately not tracked
  (Claude Code alignment),
  so a rewind does not restore files a subagent changed.

### No plugin (first-party queue edit, steer, branch only)

Pros:

- No third-party code,
  no upgrade requirement,
  no restore risk.
- Keeps the session model exactly as the troubleshooting record documents.

Cons:

- Delivers none of the wanted behavior:
  a sent message still cannot be edited or withdrawn.

## Ranking and adjacent reasons

Ranking in context B:
  `dsh-rewind-plugin` 0.17.1
  >
  no plugin.

- Reason for the order:
  the subject supplies the capability the owner asked for with verified
  provenance and an append-only history model,
  while the baseline supplies none of it;
  the subject's residual risks are named,
  bounded,
  and manageable after the prerequisite.

Ranking in context A:
  no plugin
  >
  anything installable today,
  because every candidate that needs the newer module graph is
  un-installable without a version exemption that the harness itself
  flags as crash and data-loss risk.

The plugin field is unranked by design
  (see "Decision scope and deviations").

## Confidence and evidence limits

- The engine-range finding
  (E1)
  is static range arithmetic corroborated by npm publish times,
  and its user-visible consequence is reproduced
  (E7),
  while the module-graph finding
  (E2)
  stays static:
  the client bundle's `require` was read from the built artifact,
  not executed in a browser.
- The compatibility rating in context B rests on the plugin's own CI
  matrix and pinned dev dependencies at 0.2.1-rc.2,
  not on a live profile load.
- Runtime safety is evidenced by source reading and the plugin's test
  suite,
  not by a monitored production run.
- Maintenance evidence covers the last month of issues and pull
  requests
  (the tracker holds a small total),
  so long-term stewardship is a judgment,
  not a measurement.
- Nothing in this audit ran inside the owner's harness profile.

## Recommendation

Conditional recommendation for the subject,
  nothing else.

1.  On the runtime in service
  (`dsh` 0.2.1-alpha.4):
  do not install.
  Hard constraint 1 fails,
  and a version exemption would be papering over a real module-graph gap
  the harness warns about.
2.  If the owner upgrades the harness to `>= 0.2.1-rc.1`
  (npm `latest` is `0.2.1-rc.2`):
  `dsh-rewind-plugin@0.17.1` is recommendable,
  scoring 83.3 with the recorded medium-confidence areas.
  A post-install smoke in the real profile
  (rewind one disposable message,
  confirm the composer refill and the impact list)
  closes the one unexercised validation step.

Adoption is not authorized by this report.
  Installing the plugin or upgrading the harness is a separate action
  request,
  and no decision record is written until then.
