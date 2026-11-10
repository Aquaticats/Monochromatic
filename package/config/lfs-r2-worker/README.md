# lfs-r2-worker

Git LFS server for this repository,
backed by Cloudflare R2 and deployed as a Cloudflare Worker.
It has two jobs:

- Serve LFS objects to clones from free-egress R2 instead of GitHub's metered LFS bandwidth.
- Serve Markdown-embedded images to GitHub's README renderer,
  which reads LFS objects only from GitHub's own store (issue #476).

## Why this exists

The repo is public and gets cloned thousands of times per day,
almost all of it by bots (clone traffic dwarfs human views by roughly three thousand to one).
Each clone with default git config smudges LFS and pulls the object set,
and on GitHub that download is billed to the repo owner.
Routing LFS through R2 keeps that traffic on a backend whose egress is free.

The repo-root `.lfsconfig` points every clone at this Worker:

```ini
# .lfsconfig
[lfs]
	url = https://monochromatic-lfs.aquaticat.workers.dev
```

GitHub never consults `.lfsconfig`.
Its web UI renders an LFS-tracked image only when the object sits in GitHub's own LFS store,
so a README image written as a relative link renders as pointer text once objects live here.
That is why Markdown links to LFS-tracked images are rewritten to this Worker's object URLs;
see the "README images" section.

Rolling back to GitHub LFS is not supported for now.
The decision and its context are recorded in `doc/planning/lfs-readme-image-rendering.md`.

## How it works

- Download is anonymous,
  so public `git clone` and `git lfs pull` need no credentials and `.lfsconfig` carries no secrets.
- Upload is gated by the `LFS_WRITE_TOKEN` Worker secret.
  git-lfs sends it as HTTP Basic auth (any username,
  password is the token).
- Objects live in the `monochromatic-lfs` R2 bucket,
  keyed by their 64-hex sha256 oid,
  reached through the Worker's `BUCKET` binding.
  No R2 access keys are stored in the repo or in the Worker source.
- Which oids exist is also recorded in the `monochromatic-lfs-index` D1 database (`INDEX` binding),
  so a batch needs one query instead of one R2 lookup per object;
  see "Presence index".

Routes:

- `POST /objects/batch`:
  the Git LFS batch API;
  returns per-object download or upload actions.
- `GET /<oid>` and `GET /<oid>/<repo-relative path>`:
  stream an object from R2 (anonymous).
  The path suffix is never looked up;
  it only selects the `Content-Type` from its extension
  (`png`,
   `jpg`,
   `jpeg`,
   `gif`,
   `webp`,
   `avif`,
   `heif`,
   `jxl` map to `image/*`,
  anything else is `application/octet-stream`).
  Responses carry `Cache-Control: public, max-age=31536000, immutable` and `ETag: "<oid>"`,
  and a matching `If-None-Match` returns `304`.
  Bytes are content-addressed,
  so a URL never changes meaning.
- `HEAD` on the same paths returns the headers without a body.
- `PUT /<oid>`:
  store an object in R2 (requires the upload token).
  A path suffix is rejected on `PUT`.

## Presence index

A batch names up to hundreds of objects,
and one R2 `head` per object costs 0.25 to 0.45 ms of CPU,
which exceeds the 10 ms a Workers Free request may use (Cloudflare Error 1102).
The Worker therefore asks a D1 database (`monochromatic-lfs-index`,
binding `INDEX`,
schema in `schema.sql`) which oids exist,
in one query per 500 oids.
R2 stays the source of truth:

- an index row means the object exists,
  and a missing row proves nothing;
- an upload batch gives an upload action to every oid the index lacks,
  and `PUT` is idempotent;
- a download batch confirms its first 10 index misses with `head`,
  records the ones R2 holds,
  and reports further misses absent (a stored object can then get a per-object 404
  until later batches record it,
  so rebuild a stale index promptly);
- `PUT /<oid>` records the oid once R2 has accepted the bytes;
- without the binding,
  or when a query fails,
  every object is checked with `head`,
  which is correct but too costly for a batch of 100.

Deleting an object from R2 by hand leaves a stale row,
so delete the row too (`DELETE FROM present WHERE oid = '<oid>'`).
The incident,
its measurements and the rebuild procedure are in
`doc/troubleshooting/lfs-r2-worker-error-1102.md` and `doc/runbook/lfs-r2-worker.md`.

## README images

The `markdown/lfs-image-url` rule of `monochromatic-lint` (`package/linter/monochromatic-lint`)
rewrites a Markdown image whose target is an LFS-tracked file to
`https://monochromatic-lfs.aquaticat.workers.dev/<oid>/<repo-relative path>`.
The `markdown/autofix` cli-git policy (native wrapper `package/git-policy/cli`,
configured in the repository's settings file,
`monochromatic-git-policy-cli.config.jsonc`)
applies that rewrite inside every commit,
so authors keep writing relative links and the landed commit carries absolute ones.
`package/ssg/` is excluded because those MDX pages resolve images through the site build.

The oid in the URL is the sha256 of the bytes,
so replacing an image produces a new URL and the rule refreshes stale ones on the next commit.

Verify every embedded URL against production:

```sh
mise run //package/config/lfs-r2-worker:check:markdown-urls
```

The task performs a `HEAD` request for each `<oid>/<path>` URL found in repository Markdown and asserts status `200`,
an `image/*` content type,
the URL oid equal to the local pointer's oid,
and `Content-Length` equal to the pointer size.

## Deploy

Deploy,
dry run,
and secret rotation are mise tasks;
each needs a prior `wrangler login`.
The operator procedure is `doc/runbook/lfs-r2-worker.md`.

```sh
mise run //package/config/lfs-r2-worker:deploy
mise run //package/config/lfs-r2-worker:deploy:dry-run
mise run //package/config/lfs-r2-worker:secret:write-token < "${HOME}/temp/lfs-write-token.txt"
```

`wrangler secret put` stores an **empty** secret without complaint when its stdin delivers no value.
`authorized()` refuses an absent or blank `LFS_WRITE_TOKEN`,
so a blank secret fails closed and every push gets `401`;
before that guard landed on 2026-10-01,
a blank secret accepted an empty Basic-auth password and opened uploads to any anonymous caller.
Redirect the value from a private file,
then verify the live Worker against three credentials (correct,
 empty,
 wrong) before trusting a rotation.
`doc/runbook/lfs-r2-worker.md` carries the procedure;
`doc/troubleshooting/wrangler-secret-put-empty-stdin.md` carries the measurements and the source trace.

The R2 bucket is created once with `cf r2 buckets create --name monochromatic-lfs`
(or `wrangler r2 bucket create monochromatic-lfs`).

## Pushing new images

A machine that adds images needs the upload token in a Git credential helper,
never in a Git config value.
The committed `.lfsconfig` already names the plain Worker URL,
so no local `lfs.url` is needed.
`git-lfs` sends the token as the Basic-auth password when the Worker answers an upload batch with
`401` and `LFS-Authenticate: Basic`;
it asks `git credential fill` for it,
and a credential helper answers.

```sh
# package/config/lfs-r2-worker/README.md
mkdir --parents --mode 700 "${HOME}/.config/monochromatic-lfs"
git config --global credential.https://monochromatic-lfs.aquaticat.workers.dev.helper \
  "store --file ${HOME}/.config/monochromatic-lfs/credentials"
```

On the first push `git-lfs` prompts for the user name (any value) and the password (the token) and hands them to the
helper,
so the token never passes through a command line,
a Git config value or a URL.
`doc/runbook/lfs-r2-worker.md` lists the helper choices and the verification.
The helper above is the stock `git credential-store` with a file scoped to this host;
a keyring-backed helper is a drop-in replacement.

### Git commands that may need a login

AKP:
 Git and `git-lfs` commands this package's scripts, runbooks or agents start that may reach a login set
 `GIT_ASKPASS` to the empty string and `GIT_TERMINAL_PROMPT=0`.

Set only `GIT_TERMINAL_PROMPT=0` and the desktop's password dialog still opens on a host whose `SSH_ASKPASS` names one:
Git 2.56.0 and git-lfs 3.8.1 try `GIT_ASKPASS`, then `core.askPass`, then `SSH_ASKPASS`, and only then read
`GIT_TERMINAL_PROMPT` ([why](../../../doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md)).
A probe or a test that uploads writes `GIT_ASKPASS= GIT_TERMINAL_PROMPT=0 git lfs push ...` in front of the command.
The wrapper's own Git children get the pair from it when their process has no controlling terminal, so an agent session
is covered without this rule; a shell command typed by a person is not.

With that set,
the normal flow uploads to R2 automatically,
because git's pre-push hook runs `git lfs push`:

```sh
git add path/to/new-image.png
git commit -m "..."
git push
```

Never put the token in `lfs.url`,
in a remote URL,
in a `lfs.<url>.*` key name,
or in `http.extraHeader`:
`git-lfs` echoes the URL it uses in its error messages and crash logs (`.git/lfs/logs`),
and every copy of `.git` carries Git config.
Guards that fail on it:

- `mise run //package/config/lfs-r2-worker:check:no-credential-in-git-config` checks every Git config scope and the tracked tool configs
  and prints only the key names (credentials masked) and `credential-free: yes` or `no`.
- The `url-userinfo-credential` rule in `forbidden-strings.append.txt` blocks a committed file that holds
  `scheme://user:password@host`.

The incident that produced these guards,
with the inventory of copies and the open design decisions
(per-client credentials,
read and write scopes,
CI through GitHub OIDC),
is in `doc/troubleshooting/lfs-credential-in-url.md`.

## CI note

CI in this repo checks out with `lfs: false`,
so no workflow pulls or pushes LFS objects and none needs the token.
If a future workflow commits images (for example a screenshot regenerator),
give it a credential through a Git credential helper (see "Pushing new images"),
never through `lfs.url`.
The open decision on how CI authenticates is in `doc/troubleshooting/lfs-credential-in-url.md`.

## Develop

```sh
mise run //package/config/lfs-r2-worker:build
mise run //package/config/lfs-r2-worker:test:unit
mise run //package/config/lfs-r2-worker:lint
```

`src/worker.ts` is the wrangler entry and exports only the `fetch` handler,
because workerd rejects non-handler named exports on the main module.
`src/index.ts` is the library surface the tests import,
including an in-memory object store.
