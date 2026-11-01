# Retry Coolify's GitHub clock check so pnpr deployments survive an api.github.com blip

## What this proves

A pnpr deployment that ends with

```text
Deployment failed: cURL error 28: Connection timed out after 10000 milliseconds (see https://curl.se/libcurl/c/libcurl-errors.html) for https://api.github.com/zen
```

did not fail in the clone.
The timing-out request is Coolify's own clock check,
which runs before the clone command.
`assertGithubClockInSync()` in coollabsio/coolify
(`bootstrap/helpers/github.php`,
line 153 in v4.4.6,
line 155 on main at `9355728`)
runs `Http::GitSource($apiUrl)->get("{$apiUrl}/zen")` once,
with no retry,
before it mints any GitHub App installation token.
Laravel's HTTP client defaults `connect_timeout` to 10 seconds,
so one dropped TCP connection fails the deployment.
Commit `00678a12f0` (Oct 15) adds retries for the installation-token POST
and for a clone that lost SSH (exit 255),
but not for this precheck.
Upstream tracks the same failure in [#9785][].

The outbound allowlist is not the first suspect:
the applied `package/config/tofu` state covers
the GitHub API addresses sampled on 2026-10-18,
and `tofu plan` refreshes `api.github.com/meta`.
This procedure does not change the firewall.

This procedure installs a script into `/etc/entrypoint.d`
that retries the check three times
and treats a final transport failure as advisory,
then mounts the script for every container start through `docker-compose.custom.yml`,
which Coolify's `scripts/upgrade.sh` keeps across upgrades.
Skipping a failed check is safe:
the check only reports clock skew above 50 seconds,
and a wrong clock still fails the token request with GitHub's own error.
Upgrading Coolify does not replace this procedure yet:
the check is still single-shot on main as of `9355728`.
The `/etc/entrypoint.d` script convention comes from the image's `serversideup/php` base
([start-up scripts][startup]).

The live host was not touched from this repository.
[`deploy-pnpr-registry.md`](deploy-pnpr-registry.md) records that host SSH,
the Coolify API,
and DNS credentials are not shared with agents.
The anchor line and the replacement below were checked
against the v4.4.6 and main copies of `github.php`;
the patch has not been run on the live host.

This patch covers the check that failed.
The token-POST and clone retries need a Coolify build that contains `00678a12f0`,
or a separate patch.

## Setup

Status:
TODO | DONE

Prerequisites:

- Root shell on the host that runs the `coolify` container.
- Coolify's compose files in `/data/coolify/source`:
  `docker-compose.yml`,
  `docker-compose.prod.yml`,
  and `.env`.
- Docker Compose plugin:
  `docker compose version` prints a version.
- Access to the Coolify dashboard as an administrator.
- No Coolify upgrade or deployment in progress,
  because recreating the container restarts the queue workers.

1. Open a terminal on the Coolify host.
2. Run `sudo -i`.
   Expected:
    the prompt ends with `#`.
3. Run `docker inspect --format '{{.Config.Image}}' coolify`.
   Expected:
    one line prints,
   for example `docker.io/coollabsio/coolify:4.4.6`.
   Note the text after the last `:`,
    it is the tag the recreate step keeps.
4. Run `docker exec coolify test -d /etc/entrypoint.d && echo supported`.
   Expected:
    `supported` prints.
   No output means the image does not use the `serversideup/php` entrypoint layout;
    stop here.
5. Run `docker exec coolify grep -n 'Http::GitSource($apiUrl)->get' /var/www/html/bootstrap/helpers/github.php`.
   Expected:
    one line prints,
   numbered 153 or 155,
   containing `->get("{$apiUrl}/zen")`.
   No line means a newer Coolify moved the check;
    stop here.

## Steps

Status:
TODO | DONE

1. Create the patch directory and confirm its mode:

   ```sh
   install -d -m 700 /data/coolify/source/patches
   stat -c '%a' /data/coolify/source/patches
   ```

   Expected:
    `700` prints.
2. Create the entrypoint script.
   The script runs as `www-data`,
   which owns the target file,
    and does nothing when the anchor line or the marker is absent:

   ```sh
   cat > /data/coolify/source/patches/99-github-clock-check-retry.sh <<'SCRIPT'
   #!/bin/sh
   set -eu

   file=/var/www/html/bootstrap/helpers/github.php
   [ -f "$file" ] || exit 0
   grep -q 'clock check is advisory' "$file" && exit 0

   php <<'PHP' || true
   <?php
   $file = '/var/www/html/bootstrap/helpers/github.php';
   $source = file_get_contents($file);
   $needle = '    $response = Http::GitSource($apiUrl)->get("{$apiUrl}/zen");';
   $replacement = '    try {
           $response = Http::GitSource($apiUrl)->retry(3, 1000)->get("{$apiUrl}/zen");
       } catch (\Throwable) {
           // The clock check is advisory. The installation-token request reports real failures.
           return;
       }';
   if (! str_contains($source, $needle)) {
       fwrite(STDERR, "[github-clock-check] anchor not found, file left unchanged\n");
       exit(0);
   }
   if (file_put_contents($file, str_replace($needle, $replacement, $source)) === false) {
       fwrite(STDERR, "[github-clock-check] write failed, file left unchanged\n");
       exit(0);
   }
   fwrite(STDERR, "[github-clock-check] retry patch applied\n");
   PHP
   SCRIPT
   ```

   Expected:
    the heredoc ends and the prompt returns without an error.
3. Make the script executable and check its first line:

   ```sh
   chmod 755 /data/coolify/source/patches/99-github-clock-check-retry.sh
   head -1 /data/coolify/source/patches/99-github-clock-check-retry.sh
   ```

   Expected:
    `#!/bin/sh` prints.
4. Confirm the script received the patch code:

   ```sh
   grep -c 'clock check is advisory' /data/coolify/source/patches/99-github-clock-check-retry.sh
   ```

   Expected:
    `1` prints.
5. Check whether a compose override already exists:

   ```sh
   test -f /data/coolify/source/docker-compose.custom.yml && echo exists
   ```

   Expected:
    `exists` prints when the file is present,
    nothing prints when it is absent.
6. When step 5 printed `exists`,
    open `/data/coolify/source/docker-compose.custom.yml` in an editor
    and add this line under the existing `coolify` service's `volumes:` list:

   ```yaml
   - /data/coolify/source/patches/99-github-clock-check-retry.sh:/etc/entrypoint.d/99-github-clock-check-retry.sh:ro
   ```

   When step 5 printed nothing,
    create the file with exactly:

   ```yaml
   services:
     coolify:
       volumes:
         - /data/coolify/source/patches/99-github-clock-check-retry.sh:/etc/entrypoint.d/99-github-clock-check-retry.sh:ro
   ```

   Expected:
    no editor error prints,
    and other entries in an existing file stay unchanged.
7. Confirm the override names the script:

   ```sh
   grep -n '99-github-clock-check-retry' /data/coolify/source/docker-compose.custom.yml
   ```

   Expected:
    one line prints,
   containing the `/etc/entrypoint.d/` mount.
8. Recreate the `coolify` container with its current image tag:

   ```sh
   cd /data/coolify/source
   image="$(docker inspect --format '{{.Config.Image}}' coolify)"
   LATEST_IMAGE="${image##*:}" docker compose --env-file .env \
     -f docker-compose.yml -f docker-compose.prod.yml -f docker-compose.custom.yml \
     up -d coolify
   ```

   Expected:
    the command exits with status 0,
    and `docker ps --filter name=coolify --format '{{.Status}}'` prints a line starting with `Up`.
   The dashboard is unavailable for a few seconds while the container restarts.
9. Inspect the new container's startup log:

   ```sh
   docker logs --since 5m coolify 2>&1 | tail -20
   ```

   Expected:
    `[github-clock-check] retry patch applied` prints once,
    with no `anchor not found` or `write failed` line and no `Fatal error`.
10. Verify the patch reached the file:

    ```sh
    docker exec coolify php -r 'var_dump(str_contains(file_get_contents("bootstrap/helpers/github.php"), "clock check is advisory"));'
    docker exec coolify php -l bootstrap/helpers/github.php
    ```

    Expected:
     `bool(true)` and `No syntax errors detected in bootstrap/helpers/github.php`.
11. Open the Coolify dashboard,
     open the project and environment that hold the `pnpr` resource,
     and open **pnpr**.
    Expected:
     the resource page opens on **General**.
12. Click **Deploy**,
     or **Redeploy** when that is the shown label.
    Expected:
     a deployment appears in **Deployment Logs**,
     and the log ends with `New container started.` without a `cURL error 28` line.
13. Confirm the registry still answers:

    ```sh
    curl --silent https://pnpr.c.aquati.cat/-/ping
    ```

    Expected:
     `{}` prints.

## What to check

Status:
TODO | DONE

1. `docker exec coolify php -r 'var_dump(str_contains(file_get_contents("bootstrap/helpers/github.php"), "clock check is advisory"));'`
   prints `bool(true)`.
2. `docker exec coolify php -l bootstrap/helpers/github.php`
   prints `No syntax errors detected in bootstrap/helpers/github.php`.
3. `docker ps --filter name=coolify --format '{{.Status}}'`
   prints a line starting with `Up`.
4. The pnpr deployment log contains `New container started.`
   and does not contain `cURL error 28`.
5. After each Coolify upgrade,
   repeat the `bool(true)` check in item 1.
   `bool(false)`,
   or `[github-clock-check] anchor not found` in `docker logs coolify`,
   means the patch is inactive.
   `bool(false)` means the mount or script is gone;
   `anchor not found` means upstream moved the anchor line and this procedure needs an update.

## Restore

Status:
TODO | DONE

1. Remove the volume line added in step 6 from `/data/coolify/source/docker-compose.custom.yml`.
   Delete the file when this procedure created it and it has no other entries.
2. Delete the script:

   ```sh
   rm /data/coolify/source/patches/99-github-clock-check-retry.sh
   ```

3. Recreate the container with the command from step 8.
   Expected:
    `docker ps` shows `Up`,
    and the `bool(true)` check prints `bool(false)`.
4. Remove this procedure once upstream retries the check.
   Track [#9785][].

[#9785]: https://github.com/coollabsio/coolify/issues/9785
[startup]: https://serversideup.net/open-source/docker-php/docs/customizing-the-image/adding-your-own-start-up-scripts
