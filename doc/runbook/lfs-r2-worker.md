# Operating the LFS R2 worker

This repo serves its Git LFS objects from Cloudflare R2 through a Worker,
not from GitHub LFS.
The repo-root `.lfsconfig` points every clone at the Worker,
so `git clone` and `git lfs pull` fetch objects from R2 (free egress)
instead of GitHub's metered LFS bandwidth.
Background and rationale live in `package/config/lfs-r2-worker/README.md`.

This runbook is the operator procedure for the parts a human runs by hand:
deploy or redeploy the Worker,
set or rotate the upload token,
prepare a machine to push new images,
confirm README images render on GitHub,
and confirm the GitHub LFS bill actually drops.
Rolling back to GitHub LFS is not supported for now;
the "Restore" section records what that means.

Bridges tried,
 so this is not an unconsidered handoff:
deploy,
 dry-run,
 and secret rotation are all wired as mise tasks and run headless
once wrangler is authenticated,
so they are scripted,
 not manual clicks.
The single step that cannot be done headless is the first `wrangler login`:
minting a scoped Cloudflare API token through the `cf` CLI failed
(`403 Unauthorized` on `tokens/permission-groups`),
so wrangler must be authorized through its interactive browser OAuth,
which also requires the account's two-factor prompt.

## Setup

Status:
TODO

Prerequisites for a fresh machine and a fresh checkout:

1. Clone the repo and trust mise so it reads templated config and runs tasks.

   ```sh
   git clone https://github.com/Aquaticat/Monochromatic
   cd Monochromatic
   mise trust
   ```

   Expected:
    `mise trust` prints the path it trusted and exits 0.

2. Provision wrangler,
    which the typed tool plan declares as `npm:wrangler` and `toolchain.lock.json` records.

   ```sh
   mise run toolchain:install
   ```

   Expected:
    the sync converges every tool,
    and `wrangler --version` prints a version from the tool exposure directory.

3. Authorize wrangler against the Cloudflare account that owns the bucket and Worker
   (account email `an@aquati.cat`).
   This opens a browser.

   ```sh
   wrangler login
   ```

   In the browser,
    complete the two-factor prompt and click **Allow** on the
   Cloudflare authorization screen.
   Expected:
    the terminal prints `Successfully logged in.`

4. Confirm the active account.

   ```sh
   wrangler whoami
   ```

   Expected:
    the output table lists the email `an@aquati.cat`.

The Worker is `monochromatic-lfs`,
 the R2 bucket is `monochromatic-lfs`,
and the public Worker URL is `https://monochromatic-lfs.aquaticat.workers.dev`.
If the bucket does not exist yet (full recreate),
create it once with `cf r2 buckets create --name monochromatic-lfs`
or `wrangler r2 bucket create monochromatic-lfs`.

## Steps

Status:
TODO

### Deploy or redeploy the Worker

1. Run the deploy task from anywhere in the repo.

   ```sh
   mise run "//package/config/lfs-r2-worker:deploy"
   ```

   Expected:
    the output lists the binding `env.BUCKET (monochromatic-lfs)`,
   then `Deployed monochromatic-lfs triggers` and the URL
   `https://monochromatic-lfs.aquaticat.workers.dev`.

2. To validate a change without shipping it,
    run the dry run instead.

   ```sh
   mise run "//package/config/lfs-r2-worker:deploy:dry-run"
   ```

   Expected:
    the output ends with `--dry-run: exiting now.` and never prints `Deployed`.

### Set or rotate the upload token

The upload token gates writes to R2.
Rotate it whenever it may have leaked,
or set it on first provisioning.

`wrangler secret put` reads the value from stdin when stdin is not a terminal,
and it stores an **empty** secret without complaint when stdin delivers nothing.
A blank `LFS_WRITE_TOKEN` used to be worse than an absent one:
`authorized()` in `package/config/lfs-r2-worker/src/authorize.ts` refused only an `undefined` secret,
so a blank one matched an empty Basic-auth password and let anyone upload objects.
It now refuses absent and blank secrets alike
(deployed 2026-10-01,
 Worker version `b0009f5d-9b00-4719-bbb4-1b6e5902867f`),
so a blank secret fails closed and every push gets `401`.
That is still a silent breakage,
so supply the value by shell redirection from a private file
and verify the live Worker before trusting the rotation.
The failure mode,
 its measurements,
 and its source trace are recorded in
`doc/troubleshooting/wrangler-secret-put-empty-stdin.md`.

1. Generate a token into a private file with no trailing newline,
   so the value never reaches terminal scrollback or shell history.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   umask 077
   openssl rand -hex 32 | tr -d '\n' > "${HOME}/temp/lfs-write-token.txt"
   ```

   Expected:
    the file holds 64 hexadecimal characters and no newline.
   Record the same value in your password manager;
   every pushing machine needs it.

2. Upload it through the mise task,
   redirecting the file into the task's stdin.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   mise run "//package/config/lfs-r2-worker:secret:write-token" < "${HOME}/temp/lfs-write-token.txt"
   ```

   The task runs `wrangler secret put LFS_WRITE_TOKEN`.
   Expected:
    `✨ Success! Uploaded secret LFS_WRITE_TOKEN`.
   That message reports only that Cloudflare accepted a value,
   not that the value was the one you intended,
   so the verification step is mandatory.

3. Verify the live Worker against three credentials:
   the new token,
   an empty password,
   and a wrong token.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   node --input-type=module -e '
   import { readFileSync } from "node:fs";
   const token = readFileSync(process.argv[1], "utf8").trim();
   const body = JSON.stringify({ operation: "upload", transfers: ["basic"], objects: [{ oid: "0".repeat(64), size: 1 }] });
   for (const password of [token, "", "definitely-wrong-token"]) {
     const response = await fetch("https://monochromatic-lfs.aquaticat.workers.dev/objects/batch", {
       method: "POST",
       headers: {
         Accept: "application/vnd.git-lfs+json",
         "Content-Type": "application/vnd.git-lfs+json",
         Authorization: `Basic ${Buffer.from(`lfs:${password}`).toString("base64")}`,
       },
       body,
     });
     console.log(response.status);
   }
   ' "${HOME}/temp/lfs-write-token.txt"
   ```

   Expected,
    in order:
   `200`,
   `401`,
   `401`.
   Three `401` responses mean the stored secret is blank or differs from the file;
   repeat the upload from a file that provably holds the token.
   A `200` for the empty password means the deployed Worker predates the blank-secret guard;
   redeploy it.
   The probe asks for an upload action but never sends a `PUT`,
   so it stores nothing.

4. Destroy the file.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   shred -u "${HOME}/temp/lfs-write-token.txt"
   ```

   A rotated token invalidates every machine's previously stored push credential,
   so update each pushing machine (next procedure) after rotating.

   Expected after updating a pushing machine:
    a test push (below) succeeds again.

### Configure a machine to push new images

Downloads are anonymous,
 so only machines that add images need this.
The token lives in local git config,
 never in the committed `.lfsconfig`.

1. Point local git LFS at the Worker with the token embedded as Basic-auth userinfo.
   Replace `<TOKEN>` with the current `LFS_WRITE_TOKEN`.

   ```sh
   git config --local lfs.url "https://lfs:<TOKEN>@monochromatic-lfs.aquaticat.workers.dev"
   ```

   Expected:
    no output;
   `git config --get lfs.url` then prints the URL.

2. Add an image and push it normally.
   The pre-push hook runs `git lfs push` against the Worker.

   ```sh
   git add path/to/new-image.png
   git commit -m "add new image"
   git push
   ```

   Expected:
    the push log includes a line like
   `Uploading LFS objects: 100% (1/1), ... done`.

### Migrate an existing clone after the Worker URL changes

A committed `.lfsconfig` change does not reach a clone that carries a local `lfs.url` override,
because git-lfs prefers the local value.
Every pushing machine has such an override,
since that is where its token lives.
Download-only clones need no migration:
a plain `git pull` picks up the new `.lfsconfig`.

Never print `lfs.url` or the `lfs.<url>.locksverify` key name unredacted;
both embed the token.

1. List the local LFS keys with the credential masked.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   git config --local --get-regexp '^lfs\.' | cut -d' ' -f1 | sed -E 's#//[^@]*@#//<credential>@#'
   ```

   Expected:
    `lfs.url` carrying the old host,
   plus one `lfs.<url>.locksverify` key that embeds the same old URL.

2. Drop the stale `locksverify` key without echoing it.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   git config --local --get-regexp '^lfs\.https://.*\.locksverify$' \
     | cut -d' ' -f1 \
     | xargs --no-run-if-empty -n1 git config --local --unset-all
   ```

   The key name passes through the process table as an argument,
   so run this on a machine whose process list you trust.

3. Set both keys against the new host,
   replacing `<TOKEN>` with the current `LFS_WRITE_TOKEN`.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   git config --local lfs.url "https://lfs:<TOKEN>@monochromatic-lfs.aquaticat.workers.dev"
   git config --local --add \
     "lfs.https://lfs:<TOKEN>@monochromatic-lfs.aquaticat.workers.dev.locksverify" false
   ```

   Expected:
    no output.
   Re-run the listing command from this procedure and confirm both keys name the new host.

4. Confirm the write path end to end with the write-path check in "What to check".

   Expected exact output:
   `oid match: true; authenticated PUT: 200; unauthenticated PUT: 401`.
   A `401` for the authenticated `PUT` means the endpoint credential no longer matches the Worker secret.

## What to check

Status:
TODO

1. The Worker serves a known object with intact bytes.
   This oid is `wolf-s.png` (320 bytes).

   ```sh
   curl --silent \
     "https://monochromatic-lfs.aquaticat.workers.dev/8a2f3dfd12cbaf3aa59a65937584ce25070bf3be5156dcbc14f0b4920626c0b8" \
     | sha256sum
   ```

   Expected exact output:
   `8a2f3dfd12cbaf3aa59a65937584ce25070bf3be5156dcbc14f0b4920626c0b8  -`

2. The Worker serves the same object under a path suffix with an image content type,
   which is the URL shape README images use.

   ```sh
   curl --silent --head \
     "https://monochromatic-lfs.aquaticat.workers.dev/8a2f3dfd12cbaf3aa59a65937584ce25070bf3be5156dcbc14f0b4920626c0b8/package/ssg/aquati.cat/src/content/wolf-s.png"
   ```

   Expected:
    the headers include `HTTP/2 200`,
   `content-type: image/png`,
   `content-length: 320`,
   and `cache-control: public, max-age=31536000, immutable`.

3. Every Worker URL embedded in repository Markdown resolves to the right bytes.

   ```sh
   mise run "//package/config/lfs-r2-worker:check:markdown-urls"
   ```

   Expected:
    one `ok` line per URL,
   no `FAIL` line,
   and a final `N/N Worker object URLs verified` where both numbers match.
   Then open `https://github.com/Aquaticat/Monochromatic/blob/main/package/music-player/README.md`
   in a logged-out browser and confirm every gallery image shows a picture,
   not pointer text.

4. The write path accepts the configured push credential and refuses everyone else.
   This needs a machine whose local `lfs.url` carries the token,
   and it re-uploads an object that already exists,
   so it stores nothing new:
   objects are content-addressed,
   and putting identical bytes under the same oid is a no-op.

   ```sh
   # doc/runbook/lfs-r2-worker.md
   node --input-type=module -e '
   import { createHash } from "node:crypto";
   import { execFileSync } from "node:child_process";
   const endpoint = new URL(execFileSync("git", ["config", "--local", "--get", "lfs.url"], { encoding: "utf8" }).trim());
   const authorization = `Basic ${Buffer.from(`${decodeURIComponent(endpoint.username)}:${decodeURIComponent(endpoint.password)}`).toString("base64")}`;
   const oid = "8a2f3dfd12cbaf3aa59a65937584ce25070bf3be5156dcbc14f0b4920626c0b8";
   const bytes = new Uint8Array(await (await fetch(`${endpoint.origin}/${oid}`)).arrayBuffer());
   const digest = createHash("sha256").update(bytes).digest("hex");
   const put = await fetch(`${endpoint.origin}/${oid}`, { method: "PUT", headers: { Authorization: authorization }, body: bytes });
   const anonymous = await fetch(`${endpoint.origin}/${oid}`, { method: "PUT", body: bytes });
   console.log(`oid match: ${digest === oid}; authenticated PUT: ${put.status}; unauthenticated PUT: ${anonymous.status}`);
   '
   ```

   Expected exact output:
   `oid match: true; authenticated PUT: 200; unauthenticated PUT: 401`.
   An authenticated `401` means the local credential no longer matches the Worker secret,
   or that the secret is blank,
   since a blank secret fails closed;
   the three-credential probe in "Set or rotate the upload token" tells those apart.

5. A fresh clone resolves LFS from the Worker and verifies clean.

   ```sh
   git clone --depth 1 https://github.com/Aquaticat/Monochromatic /tmp/lfs-check
   cd /tmp/lfs-check && git lfs fsck
   ```

   Expected exact output:
    `Git LFS fsck OK`.

6. The GitHub LFS bandwidth bill drops over the following days.
   In GitHub,
    open **Settings**,
    then **Billing and licensing**,
    then the
   **Git LFS** usage detail,
    and read the daily bandwidth figures.
   Expected:
    the anonymous and unauthenticated bandwidth line trends toward
   `0 GB` per day after the cutover date,
   since raw clones now read `.lfsconfig` and fetch from R2.
   Objects that predate the cutover remain in GitHub LFS storage (well under the free `10 GB`),
   so storage stays billed at `$0`.
   The web UI renders README images from the Worker URLs,
   never from GitHub LFS (issue #476).

## Restore

Status:
Not supported for now

Rolling back to GitHub LFS is not supported for now.
The decision is recorded in `doc/planning/lfs-readme-image-rendering.md`.
Two facts make the old "delete `.lfsconfig`" procedure insufficient:
objects pushed since the cutover exist only in R2,
and committed Markdown names this Worker's object URLs,
so removing the redirect alone would leave clones missing objects and READMEs pointing at the Worker.
If a rollback is ever needed,
treat it as a planned project (re-upload every object to GitHub LFS,
rewrite the Markdown links,
then drop `.lfsconfig`),
not as a runbook step.
Never tear down the Worker or the bucket while `.lfsconfig` and the Markdown links still name them.
