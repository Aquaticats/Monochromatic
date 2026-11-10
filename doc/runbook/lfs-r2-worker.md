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

1.  Clone the repo and trust mise so it reads templated config and runs tasks.

    ```sh
    git clone https://github.com/Aquaticat/Monochromatic
    cd Monochromatic
    mise trust
    ```

    Expected:
     `mise trust` prints the path it trusted and exits 0.

2.  Provision wrangler,
     which the typed tool plan declares as `npm:wrangler` and `toolchain.lock.json` records.

    ```sh
    mise run toolchain:install
    ```

    Expected:
     the sync converges every tool,
     and `wrangler --version` prints a version from the tool exposure directory.

3.  Authorize wrangler against the Cloudflare account that owns the bucket and Worker
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

4.  Confirm the active account.

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

1.  Run the deploy task from anywhere in the repo.

    ```sh
    mise run "//package/config/lfs-r2-worker:deploy"
    ```

    Expected:
     the output lists the bindings `env.INDEX (monochromatic-lfs-index)` and `env.BUCKET (monochromatic-lfs)`,
    then `Deployed monochromatic-lfs triggers` and the URL
    `https://monochromatic-lfs.aquaticat.workers.dev`.

2.  To validate a change without shipping it,
     run the dry run instead.

    ```sh
    mise run "//package/config/lfs-r2-worker:deploy:dry-run"
    ```

    Expected:
     the output ends with `--dry-run: exiting now.` and never prints `Deployed`.

### Create or rebuild the presence index

The Worker answers batches from the D1 database `monochromatic-lfs-index` (binding `INDEX`)
because one R2 `head` per object exceeds the 10 ms CPU limit of a Workers Free request
(`doc/troubleshooting/lfs-r2-worker-error-1102.md`).
The database was created and filled on 2026-11-01.
A full recreate needs these steps.
The index also heals itself (downloads confirm and record their first 10 misses per batch,
and every `PUT` records its oid),
but a stale index is not harmless:
uploads of unindexed objects are redundant but correct,
while a download batch with more than 10 unindexed stored objects answers the rest with per-object 404s
and git-lfs fails that checkout until later batches record them.
Rebuild a stale index promptly.

1.  Create the database and note the printed `database_id`
    (update `wrangler.toml` if it changed).

    ```sh
    # doc/runbook/lfs-r2-worker.md
    wrangler d1 create monochromatic-lfs-index
    ```

2.  Create the table.

    ```sh
    # doc/runbook/lfs-r2-worker.md
    mise run "//package/config/lfs-r2-worker:index:schema"
    ```

    Expected:
     wrangler reports one statement run in the remote database.

3.  Fill it from R2's own listing with a throwaway Worker bound to both resources.
    Wrangler has no R2 listing command,
    so the listing has to run inside a Worker.
    Deploy this as `lfs-index-backfill` from a scratch directory
    (`wrangler.toml` with the same `r2_buckets` and `d1_databases` blocks as the package's,
    binding names `BUCKET` and `INDEX`),
    call it until `cursor` is `null`,
    then delete it with `wrangler delete --name lfs-index-backfill`.

    ```js
    // doc/runbook/lfs-r2-worker.md
    export default {
      async fetch(request, env) {
        const cursor = new URL(request.url).searchParams.get('cursor') || undefined;
        const page = await env.BUCKET.list({ limit: 250, cursor });
        const keys = page.objects.map((object) => object.key);
        if (keys.length > 0) {
          await env.INDEX.prepare('INSERT OR IGNORE INTO present (oid) SELECT value FROM json_each(?1)').bind(JSON.stringify(keys)).run();
        }
        return Response.json({ keys: keys.length, cursor: page.truncated ? page.cursor : null });
      },
    };
    ```

    The page size of 250 keeps each request far below the CPU limit.
    Expected:
     the pages' `keys` sum to the number of objects in the bucket (2174 on 2026-11-01).

4.  After deleting an object from R2 by hand,
    delete its row too,
    because the index would keep promising the object:

    ```sh
    # doc/runbook/lfs-r2-worker.md
    wrangler d1 execute monochromatic-lfs-index --remote --command "DELETE FROM present WHERE oid = '<oid>'"
    ```

To see what a request costs,
run `wrangler tail monochromatic-lfs --format json` while probing:
each event carries `cpuTime`,
`wallTime` and `outcome`.
A healthy 100-object batch reads 1 to 8 ms;
`outcome: exceededCpu` is the Error 1102 case.

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

1.  Generate a token into a private file with no trailing newline,
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

2.  Upload it through the mise task,
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

3.  Verify the live Worker against three credentials:
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

4.  Destroy the file.

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
The token lives in a Git credential helper,
 never in a Git config value,
 a URL or a file under the repository
(the incident that ended the old practice is `doc/troubleshooting/lfs-credential-in-url.md`).

1.  Point a credential helper at the Worker host.
    The stock `git credential-store` with a file scoped to this host needs nothing installed;
    a keyring helper can replace it.

    ```sh
    # doc/runbook/lfs-r2-worker.md
    mkdir --parents --mode 700 "${HOME}/.config/monochromatic-lfs"
    git config --global credential.https://monochromatic-lfs.aquaticat.workers.dev.helper \
      "store --file ${HOME}/.config/monochromatic-lfs/credentials"
    ```

    Expected:
     no output.
    The config value holds a helper command and a path,
    never the token.

2.  Leave `lfs.url` unset locally,
    so the committed `.lfsconfig` supplies the plain URL.
    Optionally keep git-lfs from asking a Worker that has no locks API:

    ```sh
    # doc/runbook/lfs-r2-worker.md
    git config --local lfs.https://monochromatic-lfs.aquaticat.workers.dev.locksverify false
    ```

3.  Add an image and push it normally.
    The pre-push hook runs `git lfs push` against the Worker.
    On the first upload `git-lfs` asks `git credential fill` for the credential,
    finds none,
    prompts at the terminal for a user name (any value) and the token,
    and hands the pair to the helper once the Worker accepts it.

    ```sh
    git add path/to/new-image.png
    git commit -m "add new image"
    git push
    ```

    Expected:
     the push log includes a line like
    `Uploading LFS objects: 100% (1/1), ... done`.

4.  Confirm that no Git config scope carries a credential.

    ```sh
    # doc/runbook/lfs-r2-worker.md
    mise run "//package/config/lfs-r2-worker:check:no-credential-in-git-config"
    ```

    Expected exact last line:
     `credential-free: yes`.
    A `no` lists each offending key with the credential masked;
    remove the key and move the credential into the helper.

### Move a token out of an existing clone's `lfs.url`

A clone configured before 2026-11-07 may hold the token in `lfs.url` and in the name of a
`lfs.<url>.locksverify` key.
Never print either key unmasked:
the key name embeds the token.
Rotating the token (above) makes the old value useless,
and the check in the previous procedure names the keys that remain.
The status of this machine's migration is recorded in `doc/troubleshooting/lfs-credential-in-url.md`.

### Migrate an existing clone after the Worker URL changes

A committed `.lfsconfig` change does not reach a clone that carries a local `lfs.url` override,
because git-lfs prefers the local value.
With the credential in a helper,
no clone needs such an override:
unset `lfs.url` and the `lfs.<url>.locksverify` key of the old host,
and a plain `git pull` picks up the new `.lfsconfig`.

1.  List the local LFS keys with any URL userinfo masked.

    ```sh
    # doc/runbook/lfs-r2-worker.md
    git config --local --get-regexp '^lfs\.' | cut -d' ' -f1 | sed -E 's#//[^@/]*@#//<credentials>@#'
    ```

    Expected:
     keys for the plain host only.

2.  If the output still names the old host,
    unset the stale keys with the masked listing's names (an unmasked name passes through the process table).

3.  Run the write-path check in "What to check".

    Expected exact output:
    `oid match: true; authenticated PUT: 200; unauthenticated PUT: 401`.
    A `401` for the authenticated `PUT` means the credential in the helper no longer matches the Worker secret.

## What to check

Status:
TODO

1.  The Worker serves a known object with intact bytes.
    This oid is `wolf-s.png` (320 bytes).

    ```sh
    curl --silent \
      "https://monochromatic-lfs.aquaticat.workers.dev/8a2f3dfd12cbaf3aa59a65937584ce25070bf3be5156dcbc14f0b4920626c0b8" \
      | sha256sum
    ```

    Expected exact output:
    `8a2f3dfd12cbaf3aa59a65937584ce25070bf3be5156dcbc14f0b4920626c0b8  -`

2.  The Worker serves the same object under a path suffix with an image content type,
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

3.  Every Worker URL embedded in repository Markdown resolves to the right bytes.

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

4.  The write path accepts the configured push credential and refuses everyone else.
    This needs a machine whose credential helper holds the token,
    and it re-uploads an object that already exists,
    so it stores nothing new:
    objects are content-addressed,
    and putting identical bytes under the same oid is a no-op.
    The script asks `git credential fill` for the credential in memory and prints no credential.

    ```sh
    # doc/runbook/lfs-r2-worker.md
    node --input-type=module -e '
    import { createHash } from "node:crypto";
    import { execFileSync } from "node:child_process";
    const origin = "https://monochromatic-lfs.aquaticat.workers.dev";
    const filled = execFileSync("git", ["credential", "fill"], { input: "protocol=https\nhost=monochromatic-lfs.aquaticat.workers.dev\n\n", encoding: "utf8", env: { ...process.env, GIT_TERMINAL_PROMPT: "0" } });
    const fields = Object.fromEntries(filled.split("\n").filter((line) => line.includes("=")).map((line) => [line.slice(0, line.indexOf("=")), line.slice(line.indexOf("=") + 1)]));
    const authorization = `Basic ${Buffer.from(`${fields.username}:${fields.password}`).toString("base64")}`;
    const oid = "8a2f3dfd12cbaf3aa59a65937584ce25070bf3be5156dcbc14f0b4920626c0b8";
    const bytes = new Uint8Array(await (await fetch(`${origin}/${oid}`)).arrayBuffer());
    const digest = createHash("sha256").update(bytes).digest("hex");
    const put = await fetch(`${origin}/${oid}`, { method: "PUT", headers: { Authorization: authorization }, body: bytes });
    const anonymous = await fetch(`${origin}/${oid}`, { method: "PUT", body: bytes });
    console.log(`oid match: ${digest === oid}; authenticated PUT: ${put.status}; unauthenticated PUT: ${anonymous.status}`);
    '
    ```

    Expected exact output:
    `oid match: true; authenticated PUT: 200; unauthenticated PUT: 401`.
    An authenticated `401` means the credential in the helper no longer matches the Worker secret,
    or that the secret is blank,
    since a blank secret fails closed;
    the three-credential probe in "Set or rotate the upload token" tells those apart.

5.  A fresh clone resolves LFS from the Worker and verifies clean.

    ```sh
    git clone --depth 1 https://github.com/Aquaticat/Monochromatic /tmp/lfs-check
    cd /tmp/lfs-check && git lfs fsck
    ```

    Expected exact output:
     `Git LFS fsck OK`.

6.  The GitHub LFS bandwidth bill drops over the following days.
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
