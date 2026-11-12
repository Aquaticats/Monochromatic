# LFS upload token written into `lfs.url` and copied into Git config, key names, logs and backups

Status:
the owner paused this thread on 2026-11-07 ("pause the lfs thread until I say unpause lfs")
and lifted the pause on 2026-11-10 ("Unpause LFS.").
The owner's answers recorded at 2026-11-10T06:44:02-05:00 settle two of the open decisions in this file.
Decision 1 is "Per machine (Recommended)",
which is option B.
Decision 3 first drew the note
"The custom lfs settings or server should only apply to repos where this is configured."
and then "Repo + per-clone command":
the token moves to a private file,
only this repository's own Git settings point to it,
all its worktrees share that,
nothing global changes,
and one command adds the setting to a clone the person chooses.
The steps listed under "State of this machine" have not run yet.
Everything else below marked **waiting for the owner** is still undecided and unstarted.
This file holds no credential:
every value is withheld,
and every count comes from a scan that printed paths and
counts only.

## Symptom

`git config --local --get lfs.url` in the main checkout returns a URL with the Worker's upload token as the
Basic-auth password (`https://lfs:<token>@monochromatic-lfs.aquaticat.workers.dev`).
The same token is also the middle of a second key name,
`lfs.<that URL>.locksverify`,
so listing config key names
(`git config --list --name-only`) prints it too.
An agent printed the token into its transcript on 2026-11-07,
and another did the same while inventorying this
defect (the key name came out of a `--name-only` listing before the masking rule was applied;
the transcript of that
session is one of the copies listed below).
The token is treated as compromised.

## Where the token came from

The package's own instructions told operators to write it there:

- `package/config/lfs-r2-worker/README.md`,
  section "Pushing new images":
  `git config --local lfs.url "https://lfs:<LFS_WRITE_TOKEN>@monochromatic-lfs.aquaticat.workers.dev"`.
- `doc/runbook/lfs-r2-worker.md`,
  procedures "Configure a machine to push new images" and "Migrate an existing clone
  after the Worker URL changes",
  which also set `lfs.https://lfs:<TOKEN>@...locksverify`,
  and the write-path check,
  which read the token back out of `lfs.url`.
- `package/config/lfs-r2-worker/src/authorize.ts` documented the design as "git-lfs sends whatever userinfo the
  configured `lfs.url` carries".

The Worker's scheme (Basic password equals the secret,
user name free-form,
upload batches answered `401` with
`LFS-Authenticate: Basic realm="monochromatic-lfs"`) already works with Git's credential helpers;
nothing in the
Worker required the URL form.

## Why putting it in the URL spreads it

- `git-lfs` prints the URL it used in its errors and writes the same text to `.git/lfs/logs/*.log` on a failed
  batch;
  one failed push of many refs wrote one log per attempt.
- The Git wrapper relays and stores `git-lfs` output (`lfs_upload.rs` masks userinfo on relay),
  yet the persisted
  push records under `.git/monochromatic-git-policy-cli/push/` of the installed 0.5.34 wrapper held the output
  unmasked (3 records plus 3 copies under a worktree's `shadow` folder).
- Every copy of `.git` (clones of the folder,
  kopia snapshots,
  the planned move to another disk) copies
  `.git/config`.
- The `lfs.<url>.locksverify` key name carries the token even where the value is `false`.

### The mask hid the userinfo alone until 2026-11-11

The wrapper's mask replaced the userinfo of a `scheme://userinfo@host` URL and left the rest of the
URL as it stood,
so the form this file describes (the token as the password) was covered and two
other forms were not:

- a token written as a path segment,
  as in `https://host/<token>/lfs`;
- a token written as a query value,
  as in `https://host/lfs?token=<token>`.

The audit `alpha-10-merge-lfs-and-package-repair` (finding F7) measured both in the wrapper's
typed-push output and in the service log,
against a local stand-in LFS server whose URL carried a
canary in each position.
Since 2026-11-11 the mask hides the userinfo,
every path segment that is not a plain label,
and
every query and fragment value (`package/git-policy/cli/src/native/lfs/lfs_output_mask.rs`;
the rule is stated in `package/git-policy/cli/SPEC.md`,
"An LFS upload").
What it still cannot hide is a token made of lowercase letters alone,
which reads as a path label;
whether the whole path of an LFS URL should be hidden instead is an open question for the owner.
`git-lfs`'s own `.git/lfs/logs/*.log` files are outside the wrapper and still hold the URL it used.

## Inventory of every copy (paths and counts; nothing edited or deleted)

Scanner:
a scratch script reads the token once into memory and searches for it,
and for the base64 of `lfs:<token>`
(a Basic header),
with `rg --file -` (patterns on stdin,
never on a command line).
It prints only paths and counts,
and it was proven first on a fake value planted in a fixture,
with a positive
control (raw and base64 hits) and a negative file.
Scanned 2026-11-07.

### Git config and Git directories

- Main checkout `.git/config`:
  holds the token in `lfs.url` and in one `lfs.<url>.locksverify` key name.
  Every linked worktree shares this one file.
- `config.worktree`:
  one linked worktree has one;
  it does not hold the token.
- Linked worktrees registered with the main checkout:
  340 (including the main one),
  71 of them under
  `/var/mnt/encrypted/temp/agent`,
  246 under `.claude/worktrees`.
  All 339 linked ones have a `.git` pointer file and so read the single shared config;
  none has its own copy.
- Other clones with their own `.git` directory under `$HOME` (depth 3):
  10
  (`.config/nvim`,
  `wayland-mcp`,
  `resharp`,
  `pi-packages`,
  `skills`,
  `Monochromatic-but-every-file-0-bytes`,
  `one-among-us/data`,
  `labwc-vm-test`,
  `domain-list-community`,
  `xinyanchen-portfolio/portfolio-xinyanchen0124`):
  none holds the token.
  No `.git` directory sits within depth 3 of `/var/mnt/encrypted`;
  the 71 worktrees there are linked worktrees of the
  main checkout.
- A second search (`find ... -maxdepth 4 -name .git -type d`) under `/var/mnt/encrypted/temp/agent`,
  `~/temp/agent` and
  `~/worktrees` found 5306 `.git` directories (5354 `config` and `config.worktree` files);
  none holds the token.
  A clone nested deeper than those depth limits was not seen.
- `~/.gitconfig`,
  `~/.config/git`,
  `~/.netrc`,
  `~/.git-credentials`,
  `~/.config/gh`:
  none holds the token.
  The only credential helpers configured are `gh auth git-credential` for `github.com` and `gist.github.com`.

### Logs and records written by tools

- `.git/lfs/logs/*.log`:
  1301 files hold the token (all from 2026-11-01),
  each repeating the URL in a
  `batch response: Fatal error: Server error https://lfs:<token>@.../objects/batch from HTTP 503` line.
- `.git/worktrees/<name>/cli-git-service-<id>/shadow/lfs/logs/`:
  1301 files,
  a copy of the same logs.
- `.git/monochromatic-git-policy-cli/push/*.last-pushed.json`:
  3 records,
  plus 3 copies in the same worktree's
  `shadow` folder,
  each with the failing push output unmasked.
- `~/.config` (includes `~/.config/.wrangler/logs`):
  none holds the token.

### Backups

kopia (repository `/mnt/pcloud/rclone`) holds 53 snapshots of `/home/user/Monochromatic`,
listed with
`kopia snapshot list`.
Each snapshot contains `.git/config`;
streaming that file into memory for a yes or no check (nothing restored,
nothing written) found the token in 51 and not in the 2 June and July 2026 snapshots.
The earliest snapshot that holds it is 2026-11-01T23:52:45Z.
`.git/lfs` is inside the snapshots too (362 MiB),
so the log files above are also in them.
kopia also snapshots `~/.claude`,
`~/.pi` and `~/.config`;
those snapshots were not opened.

### CI

`gh secret list` for the repository shows 11 secrets and none for LFS:
`CLAUDE_CODE_OAUTH_TOKEN`,
`FACTORY_API_KEY`,
`FORBIDDEN_STRINGS_LIST`,
`MAVEN_CENTRAL_PASSWORD`,
`MAVEN_CENTRAL_USERNAME`,
`MAVEN_SIGNING_KEY`,
`MAVEN_SIGNING_PASSWORD`,
`MUSIC_PLAYER_KEYSTORE_BASE64`,
`MUSIC_PLAYER_KEYSTORE_PASSWORD`,
`MUSIC_PLAYER_KEY_ALIAS`,
`MUSIC_PLAYER_KEY_PASSWORD`.
Environments `copilot` and `github-pages` have no secrets;
variables are
`COPILOT_AGENT_FIREWALL_ALLOW_LIST_ADDITIONS` and `COPILOT_AGENT_FIREWALL_ENABLED`.
The account is personal (no organization secrets).
So the token was never given to Actions.

### Shell history, repository files and history

- `~/.bash_history`:
  does not hold the token.
- Repository history:
  every object in the main object database (414203 objects,
  7.5 GB,
  loose and packed,
  all
  refs including worktree branches) was streamed through `git cat-file --batch-all-objects --batch`;
  none holds the
  token
  or its Basic-header form.
  The token was never committed.
- Tracked files at the main checkout:
  21550 files searched,
  none holds the token.

### Agent transcripts and tool state

- `~/.claude/projects/-var-home-user-Monochromatic`:
  17 transcript files hold the token (2 session files and 15 subagent
  files under `<session>/subagents/`),
  written between 2026-10-05 and 2026-11-07.
  The files are the owner's data and were not edited.
  The transcript of the session that produced this inventory is one more copy:
  it printed the `locksverify` key name
  (which embeds the token) from a `--name-only` listing before any masking was applied,
  and the scan began before
  that file was complete.
- `~/.codex`:
  no file holds the token.
- `~/.local/state` and `~/.config` (including `~/.config/.wrangler/logs`):
  no file holds the token.
- Not scanned,
  outside the brief's list or too large for the time spent:
  `~/.pi` (4.2 GB),
  `~/.cache`,
  `~/.local/share` (model and tool installs),
  the kopia snapshots of `~/.claude`,
  `~/.pi` and `~/.config`
  (kopia snapshots the transcripts,
  so the 17 files above are in them from the time they were written).
  The scan script and its fake-value proof are scratch under `~/temp/agent/lfs-scan`;
  resume with
  `node scan.ts real cache` there for `~/.cache`.
runs in user unit `lfs-inventory-scan2`;
  `~/.pi` (4.2 GB) is not in the brief's list and was not scanned.

## What was done in the repository

All on branch `fix/lfs-r2-worker-credential-out-of-url`,
not yet merged.

- `package/config/lfs-r2-worker/src/git-config-credential-guard.ts` and its unit tests (all green,
  run with the
  suite):
  finds a credential in a URL (password in any scheme,
  or a user name alone on a web scheme),
  in a config key
  name,
  and in an `Authorization` header set through `http.extraHeader`;
  returns findings with credentials masked.
- Mise task `//package/config/lfs-r2-worker:check:no-credential-in-git-config`:
  runs the guard over every Git config
  scope (`git config --list --null --show-scope --show-origin`) and over tracked tool configs (`.lfsconfig`,
  `.gitmodules`,
  `.npmrc`,
  `.cargo/config.toml`,
  `.yarnrc.yml`,
  `.pnpmrc`,
  `.config/mise/**`,
  `mise.toml`).
  It prints finding locations with credentials masked and `credential-free: yes` or `credential-free: no`.
  Run against this machine's main checkout before any migration it printed the two findings (`lfs.url` value,
  `locksverify` key name) and `credential-free: no`,
  exiting with failure,
  and printed no credential.
- Rule `url-userinfo-credential` in `forbidden-strings.append.txt` (checked in,
  so CI and every clone use it):
  blocks a
  committed file with `scheme://user:password@host` (password of 12 or more characters;
  placeholders in angle
  brackets or braces and short example passwords pass).
  Before the change the built-in rules covered only MongoDB connection strings and `curl -u`;
  no generic URL
  userinfo rule existed.
  A scratch copy of the same pattern,
  run with `--all` over the tracked files of a fresh checkout,
  reported nothing.
  Nine Rust integration tests (`package/forbidden-strings/cli/tests/repository_appendix_url_userinfo.rs`) run the real
  appendix through the scanner binary:
  three positives,
  six negatives.
- The README and the runbook no longer instruct anyone to put the token in a URL;
  they describe a Git credential
  helper (`credential.<host>.helper`),
  the guard,
  and a write-path check that asks `git credential fill`.

### Credential helper flow verified against the real Worker handler

The built Worker (`dist/final/neutral/worker.mjs`,
in-memory bucket,
made-up token) was served on localhost.
A throwaway repository with a plain `lfs.url` and `credential.helper = store --file <fixture>` ran
`git lfs push origin HEAD`:
`Uploading LFS objects: 100% (1/1)`,
with the server log showing the authenticated batch
and `PUT` answered `200`.
The helper was configured in its unscoped form (`credential.helper`);
the URL-scoped form the README recommends
(`credential.<host>.helper`) was not run,
though `git-lfs` shells out to `git credential fill`,
which does the URL
match.
So the current Worker works with a credential helper and a plain URL,
with no Worker change.
A control run with no credential available failed plainly
(`fatal: could not read Username for ... terminal prompts disabled`,
then `Git credentials for ... not found`,
exit
status 2).
The server log showed the unauthenticated batch answered `401` with the `LFS-Authenticate` header and no retry,
so the helper is what supplies the credential.

#### Prompts in test runs

The first control run,
made with only `GIT_TERMINAL_PROMPT=0`,
opened `/usr/bin/ksshaskpass` on the owner's desktop
(`Username for http://127.0.0.1:...`) and the dialog outlived the killed command.
Git's `prompt.c` and git-lfs's `creds/creds.go` try `GIT_ASKPASS`,
then `core.askPass`,
then `SSH_ASKPASS` (exported
by this desktop as `/usr/bin/ksshaskpass`) before they look at `GIT_TERMINAL_PROMPT`.
Every test Git or git-lfs command must therefore run with `GIT_ASKPASS=` (set but empty) and `GIT_TERMINAL_PROMPT=0`,
plus `-c core.askPass=` when the config may set one.
The second control run used exactly that:
it failed with `terminal prompts disabled` and `ps -eo pid,args` showed no
`ksshaskpass` afterwards.
"no credential,
no upload".

## State of this machine (unchanged on purpose)

The owner's pause means the local migration was not done.
`.git/config` of the main checkout still holds the token in `lfs.url` and in the locksverify key name,
and the Worker
secret is unchanged.
Nothing else was edited:
no copy listed above was edited or deleted.
The planned steps,
to run after the owner says to continue:

1.  Choose the helper (below),
    store the current token through `git credential approve` from stdin.
2.  Unset `lfs.url` (the committed `.lfsconfig` supplies the plain URL) and unset the `locksverify` key;
    set the plain
    `lfs.https://monochromatic-lfs.aquaticat.workers.dev.locksverify false`.
3.  Run `check:no-credential-in-git-config` and expect `credential-free: yes`;
    run `git lfs fsck --pointers` and a
    write-path check.
4.  Rotate last,
    once nothing depends on the old value:
    generate into a 0600 file,
    `wrangler secret put` from stdin,
    store in the helper from the file,
    deploy if the auth design changed,
    prove old refused and new accepted with
    bounded requests whose output is redacted,
    delete the file.

## Open decisions (waiting for the owner)

### Decision 1: authentication design

What the Worker does today:
one long-lived shared secret;
reads are anonymous by design (public repository,
GitHub's
README renderer,
free egress for clones);
writes need the secret as Basic password;
the batch response echoes the
caller's `Authorization` header into upload actions.

Facts from the Git LFS documents (fetched 2026-11-07 from `git-lfs/git-lfs` `docs/api/authentication.md` and
`docs/api/batch.md`):
for HTTP remotes the credentials come from Git's credential system,
SSH remotes use
`git-lfs-authenticate` headers,
and a URL-embedded credential is "not recommended for security reasons".
A `401` should carry `LFS-Authenticate`;
the client then asks the credential helper.
`git-lfs` 3.8.1's `creds/creds.go` declares the credential-helper capability `authtype`,
and `lfsapi/auth.go` builds
an `Authorization: <authtype> <credential>` header from the helper's answer,
so a helper can return a Bearer token as
well as a Basic pair.
GitHub Actions can mint an OIDC token per run (needs `permissions: id-token: write`),
verifiable against the issuer
`https://token.actions.githubusercontent.com`;
this repository's `package/config/pnpr/config.yaml` already trusts those
tokens by issuer,
audience,
subject,
`repository_id`,
`repository_owner_id` and `workflow_ref`
(`doc/decision/private-npm-registry.md`).
No workflow in `.github/workflows` mentions LFS,
and `actions/checkout` defaults to `lfs: false`,
so CI uses no LFS
credential today.

Options,
with pros,
cons and why each ranks where it does:

#### Option C: per-client revocable tokens plus GitHub OIDC for CI (recommended)

- Each client (this machine,
  any other machine) gets its own random token (at least 128 bits),
  stored only as a hash
  in the existing D1 database (`clients`:
  id,
  label,
  scope,
  hash,
  created,
  expires,
  revoked).
  The Basic user name is the client id,
  the password the token,
  so lookup is one row and the compare is constant time.
- CI sends its GitHub OIDC token as the Basic password through an inline credential helper that reads it from the job
  environment;
  the Worker verifies signature (RS256 via WebCrypto against the cached JWKS),
  issuer,
  audience (the Worker
  URL),
  expiry,
  and the same claim set pnpr uses.
  No secret is stored in GitHub,
  each token lives minutes and is bound to repository,
  ref and workflow.
- Scopes:
  `write` for uploads;
  reads stay anonymous (see Decision 2).
- Pros:
  per-client revocation takes effect at once;
  a leaked machine token exposes one client;
  CI holds no stored
  secret;
  matches the pnpr precedent;
  the credential lives only in a helper or keyring.
- Cons:
  a clients table and a small admin path to add and revoke rows (no endpoint is needed:
  rows can be written with
  `wrangler d1 execute`);
  a JWT verifier in the Worker (about 100 lines plus tests;
  RSA verify is a public-key
  operation,
  small next to the 10 ms CPU limit,
  which should be measured before shipping);
  JWKS fetch on a cold isolate;
  more code to maintain for CI that uses no LFS today.

#### Option B: per-client revocable tokens only

- The same table without the OIDC verifier;
  CI,
  if it ever pushes LFS,
  would need a stored per-client token.
- Pros:
  all of the per-client benefit with less code;
  same Basic flow on the client.
- Cons:
  the first CI use brings back a stored long-lived secret in GitHub,
  the class of problem this incident is about.
- Ranks below C because of that CI gap,
  but it is a strict subset:
  build it first,
  add OIDC when CI needs LFS.

#### Option D: short-lived signed tokens from an exchange endpoint, custom credential helper

- A machine keeps one long-lived key,
  a small helper exchanges it for one-hour signed tokens (the helper answers
  `authtype=Bearer`),
  the Worker verifies the signature statelessly.
- Pros:
  tokens expire on their own;
  nothing long-lived travels with each request.
- Cons:
  a helper program to ship and keep working on every machine;
  a signing key the Worker must hold;
  the long-lived
  exchange key still exists and still needs revocation,
  so Option B's table is needed anyway.
- Ranks below B because B already gives per-client revocation with no new client software.

#### Option A: keep the shared secret, move it into a helper, rotate

- Pros:
  no Worker change;
  done quickly;
  fixes this incident's exposure.
- Cons:
  no per-client credentials,
  no scopes,
  no expiry;
  any one leaked copy exposes all writes;
  CI would need the
  shared secret.
- Ranks below D because D adds expiry and revocation,
  which A lacks entirely,
  but it stays above F and E because it
  exposes only LFS writes and works with the stock client.

#### Option F: verify the caller's GitHub token against the GitHub API

- The Worker would call the GitHub API with the supplied token and allow the upload if the token can push to the
  repository;
  CI would use `GITHUB_TOKEN`.
- Pros:
  identity and revocation follow GitHub access;
  no second credential system.
- Cons:
  the Worker would receive GitHub tokens that are worth far more than an LFS write;
  dependence on GitHub API
  availability and rate limits.
- Ranks below A because A leaks only LFS writes,
  F would hand account-level tokens to a third-party Worker.

#### Option E: Cloudflare Access service tokens or mutual TLS in front of uploads

- Pros:
  managed per-token revocation and audit.
- Cons:
  service tokens are headers (`CF-Access-Client-Id` and `-Secret`),
  which `git-lfs` can only send from
  `http.extraHeader`,
  so the secret lands in Git config again;
  mutual TLS and Access policies need a custom domain,
  and
  the anonymous download route would need an exemption.
- Ranks last:
  it does not fit the credential-helper flow.

Ranking:
C > B > D > A > F > E.
C > B because OIDC removes the only remaining stored secret (CI) at the price of a verifier;
B > D because per-client rows give revocation without new client software;
D > A because A has neither expiry nor revocation;
A > F because A exposes only LFS writes while F hands GitHub tokens to the Worker;
F > E because F works with a credential helper and E cannot.

### Decision 2: do reads stay anonymous?

The owner asked for "read separately from write".
Reads are anonymous today on purpose:
GitHub's README renderer fetches
`<base>/<oid>/<path>` without credentials,
bots clone without credentials,
and the repository is public.
Gating reads would break README images and anonymous clones.
If reads stay anonymous,
scopes reduce to `write`.
If a private-read tier is wanted,
the table gains a `read` scope and the batch download branch must check it.

### Decision 3: credential helper on this machine

No helper serves this host yet (`gh auth git-credential` answers only for GitHub hosts).
Available here:
`git credential-store` (plaintext file with mode 0600 on the encrypted volume),
`git credential-cache`
(memory,
expires),
and a Secret Service (`ksecretd` on D-Bus,
with `secret-tool` installed) but no
`git-credential-libsecret` binary.
Recommended:
`credential-store --file ~/.config/monochromatic-lfs/credentials` now (needs nothing installed,
file is
outside every `.git`,
mode 0600,
scoped to this host),
and a keyring-backed helper later if one is installed.
A keyring helper is better at rest and costs an install plus a unit of trust in that binary.

### Decision 4: what happens to each class of copy

Nothing was touched.
Rotation makes the old token useless in all of them,
so the question is only whether the
copies are cleaned for hygiene.

- Crash logs under `.git/lfs/logs` (1301) and their `shadow` copies,
  and the 6 push records:
  tool logs the owner may
  delete.
- `.git/config` key and value:
  removed by the planned migration.
- kopia snapshots (51 of 53):
  immutable history;
  rotation neutralizes them;
  deleting snapshots is the owner's call.
- Agent transcripts and tool state:
  see the inventory;
  rotation neutralizes them.
- Shell history:
  clean.

### Decision 5: wire the guard into the Git wrapper

The guard is a mise task and a committed scanner rule.
Nothing runs it automatically on push.
Refusing a push while any config scope carries a URL credential belongs in the wrapper's pre-push policy
(`package/git-policy/cli`,
where `uses_lfs` already reads the config snapshot),
a Rust change that was not started.

## Every legitimate client and how it will be verified

- This machine (main checkout and 339 linked worktrees share one `.git/config`):
  pushes through the wrapper's pre-push
  `git lfs push`;
  verify with `check:no-credential-in-git-config` printing `credential-free: yes`,
  `git lfs fsck --pointers`,
  and the runbook's write-path check printing
  `oid match: true; authenticated PUT: 200; unauthenticated PUT: 401`.
- CI:
  no workflow mentions LFS and `actions/checkout` runs with its default `lfs: false`,
  so nothing authenticates to
  the Worker.
  Verify by `rg -i lfs .github` (no hit) after any change;
  under Option C add one workflow that requests the OIDC
  token and runs the write-path check.
- Anonymous clones and the README renderer (bots,
  `git clone`,
  GitHub):
  unauthenticated reads;
  verify with
  `mise run //package/config/lfs-r2-worker:check:markdown-urls`.
- Depot VMs and checkouts that smudge LFS from the Worker (`doc/troubleshooting/depot-historical-checkout-bootstrap.md`)
  read anonymously;
  they need no credential.
- Other machines named in the docs:
  `doc/planning/repository-on-the-860-drive.md` plans to copy `.git` to another disk
  of this machine,
  which would copy the config;
  the migration should run first.
  No other pushing machine is named anywhere in the repository.

## What could not be done

- The local migration,
  the rotation and any Worker auth change:
  paused by the owner.
- Opening the kopia snapshots of `~/.claude`,
  `~/.pi` and `~/.config`.
