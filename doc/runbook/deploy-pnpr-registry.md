# Runbook: deploy the private pnpr registry

Deploys `package/config/pnpr/` as a Coolify Docker Compose resource,
 publishes it at `https://pnpr.c.aquati.cat/` through the Coolify host's self-managed Caddy,
 and wires Coolify to redeploy only when that package changes.
Design:
 `doc/decision/private-npm-registry.md`.

## What this proves

- DNS,
   Caddy,
   and Coolify route `https://pnpr.c.aquati.cat/` to the pnpr container.
- The registry answers anonymous reads,
   refuses unauthenticated publishes,
   and receives scoped package paths (`%2f`) intact.
- Pushes that do not touch `package/config/pnpr/` do not redeploy.

## Why a person runs this

The owner decided that no host SSH access,
 Coolify API token,
 or Njalla DNS token is shared with agents.
The bridges that would automate these steps (SSH to the Coolify host,
 the Coolify API,
 the Njalla API)
 are withheld by that authorization choice,
 not by a missing capability.
Before writing this,
 the container,
 the generated config,
 unauthenticated publish refusal,
 and `%2f` pass-through behind Caddy 2.11.4 were verified locally with podman.

## Setup

Status:
TODO | DONE

1. Have these ready:
   - A browser signed in to the Coolify dashboard as an administrator.
   - An SSH session to the Coolify host,
      with permission to edit its Caddyfile and reload Caddy.
   - A browser signed in to Njalla with access to the `aquati.cat` domain.
   - A Coolify GitHub App source that can read `Aquaticat/Monochromatic`.
      A public-repository resource cannot work here:
       Coolify renders **Watch paths** only when
       `$this->application->is_github_based() && !$this->application->is_public_repository()`
       (`resources/views/livewire/project/application/general.blade.php` in coollabsio/coolify).
   - Hetzner egress to `ghcr.io` on port 443,
      which the image build uses to pull the patched `ghcr.io/aquaticat/pnpr` base image.
      The image needs no apt or npm access.
   - A local terminal with `curl`,
      `dig`,
      and `openssl`.
2. Confirm the deployment files are on `main`:
   open `https://github.com/Aquaticat/Monochromatic/tree/main/package/config/pnpr`.
   Expected:
    the listing shows `Containerfile`,
    `compose.yaml`,
    and `config.yaml`.
3. On the Coolify host,
    run `ss --listening --tcp --numeric 'sport = :7677'`.
   Expected:
    only one line,
    the header beginning with `State`;
    no socket is listening on port 7677.
   If a row appears,
    stop and report it,
    because `compose.yaml` publishes `127.0.0.1:7677`.

## Steps

Status:
TODO | DONE

### Stage A: DNS record at Njalla

1. In Njalla,
    open the `aquati.cat` domain's DNS records.
   Expected:
    an `A` record named `garage.c` with content `135.181.104.96` is listed.
2. Click **Add record**.
   Expected:
    a record form opens.
3. Set **Type** to `A`,
    **Name** to `pnpr.c`,
    **Content** to `135.181.104.96`,
    and **TTL** to the same value the `garage.c` record uses.
4. Save the record.
   Expected:
    an `A` record named `pnpr.c` with content `135.181.104.96` is listed.
5. In the local terminal,
    run `dig +short pnpr.c.aquati.cat`.
   Expected:
    `135.181.104.96`.
   An empty result means propagation is still pending;
    rerun it after a few minutes before continuing.

### Stage B: Coolify resource

1. In Coolify,
    open the project that holds the Garage resource,
    click **+ New**,
    and choose **Resource**.
   Expected:
    the resource picker opens with a **Search resources** box.
2. Type `github` into **Search resources** and choose the private repository option that uses a GitHub App.
   Expected:
    the form shows **Choose GitHub App**.
3. Choose the GitHub App that can read `Aquaticat/Monochromatic`,
    then pick `Aquaticat/Monochromatic` under **Choose repository**.
   Expected:
    the **Build configuration** section appears with **Branch** and **Build pack**.
4. Set **Branch** to `main`.
5. Set **Build pack** to `Docker Compose`.
   Expected:
    **Base directory** and **Compose file** fields appear.
6. Set **Base directory** to `/package/config/pnpr`.
7. Set **Compose file** to `/compose.yaml`.
   Expected:
    the helper text shows the compose path resolving to `/package/config/pnpr/compose.yaml`.
8. Click **Continue**.
   Expected:
    the application's configuration page opens and lists the `pnpr` service.
9. Leave every domain field empty.
   Expected:
    no domain is attached,
    because Caddy on the host serves the hostname.
10. On the **General** page,
     set **Watch paths** to exactly these three lines and save:
     `package/config/pnpr/config.yaml`,
     `package/config/pnpr/Containerfile`,
     `package/config/pnpr/compose.yaml`.
    Expected:
     the field keeps all three lines after the page reloads.
    If **Watch paths** is missing,
     the resource was created from a public repository;
     delete it and restart Stage B with the GitHub App.
11. In the local terminal,
     run `openssl rand -hex 32`.
    Expected:
     one line of 64 hexadecimal characters.
12. Open **Environment Variables**.
    Expected:
     `PNPR_SECRET` is listed and marked required,
     because `compose.yaml` declares it with `${PNPR_SECRET:?...}`.
13. Set `PNPR_SECRET` to the 64-character value from step 11 and save.
    Expected:
     the variable shows a saved,
     masked value.
14. Click **Deploy**.
    Expected:
     the deployment log ends with the `pnpr` container running,
     and the container log contains `"message":"pnpr listening","listen":"0.0.0.0:7677"`.
15. On the Coolify host,
     run `curl --silent http://127.0.0.1:7677/-/ping`.
    Expected:
     `{}`.

### Stage C: Caddy site block

1. On the Coolify host,
    open the self-managed Caddyfile in an editor.
2. Append this site block after the existing `garage.c.aquati.cat` block:

   ```caddyfile
   pnpr.c.aquati.cat {
   	reverse_proxy 127.0.0.1:7677
   }
   ```

   Expected:
    the file contains exactly one `pnpr.c.aquati.cat {` block.
3. Run `caddy validate --config <path to the Caddyfile> --adapter caddyfile`.
   Expected:
    the output ends with `Valid configuration`.
4. Run `systemctl reload caddy`.
   Expected:
    the command returns with no error printed.
5. In the local terminal,
    run `curl --silent https://pnpr.c.aquati.cat/-/ping`.
   Expected:
    `{}`.
   A certificate error means Caddy has not yet obtained the certificate for the new name;
    rerun it after a minute.

### Stage D: push events

No manual repository webhook is needed:
 the GitHub App delivers push events to Coolify,
 and Coolify filters them by **Watch paths**.
Do not add a repository webhook as well,
 or matching pushes deploy twice.

## What to check

Status:
TODO | DONE

Run each command in the local terminal.

The URLs in steps 2 and 3 are the `0.1.0-alpha.11` root surface.
 On the `0.1.0-alpha.16` base and later the same paths live under `/npm`;
 the section "Move to a newer pnpr base" checks that surface.

1. `curl --silent https://pnpr.c.aquati.cat/-/ping`
   prints `{}`.
2. `curl --silent --write-out ' %{http_code}\n' 'https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw'`
   prints `Not Found 404` before the first publish run,
   or a JSON document containing `"name":"@monochromatic-dev/or-throw-core"` after it.
3. `curl --silent --request PUT --header 'content-type: application/json' --data '{}' --write-out ' %{http_code}\n' 'https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw'`
   prints `Authentication required for package "@monochromatic-dev/or-throw-core" 401`.
   The package name in that message proves the encoded slash reached pnpr through Caddy.
4. `curl --silent --write-out ' %{http_code}\n' https://pnpr.c.aquati.cat/left-pad`
   prints `Not Found 404`,
   confirming the registry proxies nothing.
5. After the next push to `main` that does not touch `package/config/pnpr/`,
   the pnpr application's **Deployment Logs** list shows no new deployment for that commit.
6. Tell the agent session that the registry is live,
   so it can dispatch the publish workflow and run the throwaway-consumer verification.

## Restore

Status:
TODO | DONE

Only if the registry must be removed.

1. In Coolify,
    open the pnpr application's **Danger Zone**,
    delete the resource,
    and select the option that also deletes volumes.
   Expected:
    the application and the `pnpr-registry-storage` volume disappear,
    along with the empty `pnpr-storage` volume if the resource predates the patched image.
   Published packages are gone with the volume;
    recovery republishes current versions.
2. On the Coolify host,
    remove the `pnpr.c.aquati.cat` block from the Caddyfile and run `systemctl reload caddy`.
   Expected:
    `curl --silent --write-out '%{http_code}\n' --output /dev/null https://pnpr.c.aquati.cat/-/ping` no longer prints `200`.
3. In Njalla,
    delete the `pnpr.c` `A` record.
   Expected:
    `dig +short pnpr.c.aquati.cat` prints nothing once caches expire.

## Operations

- pnpr moves only when the `Containerfile` `FROM` digest changes;
   `package/config/pnpr/README.md` "Deploy and operate" lists the steps,
   and committing the new digest triggers the Watch paths redeploy.
- The switch to the patched image on 2026-09-15 renamed the storage volume to `pnpr-registry-storage`,
   because the new image runs as user `pnpr` instead of `node`.
   The old `pnpr-storage` volume held no packages;
   delete it from the application's **Storages** page once the new deployment answers `{}` on `/-/ping`.
- `config.yaml` changes only through `mise run //package/config/pnpr:generate:config`
   (file-enforcer does not write it; `check:config` reports a drift);
   committing its regenerated file triggers the Watch paths redeploy.

## Move to a newer pnpr base

Moves the deployed registry onto a newer `pnpr` base
 (for example `0.1.0-alpha.16`),
 including the store conversion that base requires
 and the URL move it brings.
 Repository-side steps live in
 `package/config/pnpr/README.md` "To move to a newer pnpr";
 measured hazards and evidence live in
 `doc/troubleshooting/pnpr-base-move.md`.
 The container CLI and the registry's own `pnpr upgrade-storage` subcommand were exercised here
 against throwaway stores with podman;
 the Coolify host itself stays with the owner
 (bridges named in "Why a person runs this"),
 so a person runs these steps.
 Expect the registry to answer `502` or refuse connections from the first step
 until the last check passes.

### Setup

Status:
TODO | DONE

1. Confirm the image to move to exists in the registry.
    Read the **pnpr image** notice
    (`ghcr.io/aquaticat/pnpr:<version>-oidc-workload-discovery.<sha>@sha256:<digest>`)
    from the `.github/workflows/pnpr-patched-image.yml` run that built it.
2. On the Coolify host,
    have the container CLI ready
    (`docker`;
     substitute `podman` if that is what the host runs),
    and read the exact storage volume name from the running container's mounts
    with `docker inspect --format '{{json .Mounts}}' <container>`
    (`docker ps --all --filter name=pnpr` lists candidates).
   Expected:
    one mount whose `Source` ends in `pnpr-registry-storage`;
    use that `Source` name in every `--volume` below,
    never the bare compose name.
   If `command -v docker podman nerdctl ctr crictl` prints nothing,
    the shell is not on the machine that runs the container,
    or the CLI sits outside this shell's `PATH`.
   A `command not found` for a command that exists also comes from an invisible character
    pasted before the name
    (a non-breaking space;
     bash ignores an ordinary leading space);
    retype the command before concluding the CLI is absent.
   Confirm the machine first:
    `curl --silent --write-out '%{http_code}\n' --output /dev/null http://127.0.0.1:7677/-/ping`
    prints `200` only on the host,
    because compose binds the registry to `127.0.0.1:7677`.
   Expected on a host that can run the steps:
    `command -v docker` prints a path
    and `ls /var/run/docker.sock` lists the daemon socket.
   If the socket exists without any CLI,
    install that runtime's CLI on the host
    (the operator's call)
    or run the steps from a shell that already has it;
    never continue without a CLI that can run the conversion image.
3. Read the `PNPR_SECRET` value from the Coolify environment variables.
   Expected:
    the value the running application uses,
    at least 16 bytes.

### Steps

Status:
TODO | DONE

1. Stop the pnpr application
    (Coolify **Stop**,
    or `docker stop <container>`).
   Expected:
    `curl --silent http://127.0.0.1:7677/-/ping` prints nothing and exits non-zero.
    The registry serves nothing from here until the last check passes.
2. Convert the store.
   Keep the standing no-backup decision in `package/config/pnpr/compose.yaml`
    (owner decision 2026-10-21:
     no pre-conversion snapshot,
     recovery republishes current versions),
    even though the startup error says `Back it up` and the conversion is one-way for the old base:

   ```bash
   docker run --rm --env PNPR_SECRET=<secret> --volume <VOLUME>:/pnpr/storage:z <IMAGE> --storage /pnpr/storage upgrade-storage
   ```

   Expected:
    `Moved N documents`,
    `Moved N other files`,
    and `Removed N listing-index and upload files`.
    The layout marker is written last,
    so an interrupted run resumes on re-run.
    A refusal naming `layout <n>` means the store is already converted
    or converted by another layout,
    and this procedure does not apply.
   Preview first with `upgrade-storage --dry-run`
    if you want the counts before anything moves
    (`Would move N documents`).
3. Re-run step 2 as a control.
   Expected:
    `The hosted store already has a current layout.json; nothing to do.`
4. Push the lockstep commit
    (`Containerfile` `FROM` pin,
    the `/npm` URL moves in `package/config/pnpr/src/` and the docs).
   Expected:
    Coolify redeploys through its Watch Paths setting
    and the container stays up instead of restarting.

### What to check

Status:
TODO | DONE

1. `curl --silent --write-out '%{http_code}\n' --output /dev/null https://pnpr.c.aquati.cat/-/ping`
   prints `200`.
2. `curl --silent https://pnpr.c.aquati.cat/-/ready`
   prints `{"status":"ok","storage":"ok","auth":"ok"}`.
3. `curl --silent https://pnpr.c.aquati.cat/-/metrics`
   contains `pnpr_build_info{version="0.1.0-alpha.16"}`
   (the version of the base you moved to).
4. `curl --silent --write-out ' %{http_code}\n' 'https://pnpr.c.aquati.cat/npm/~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw'`
   prints a JSON document containing `"name":"@monochromatic-dev/or-throw-core" 200`.
5. `curl --silent --request PUT --header 'content-type: application/json' --data '{}' --write-out ' %{http_code}\n' 'https://pnpr.c.aquati.cat/npm/~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw'`
   prints `Authentication required for package "@monochromatic-dev/or-throw-core" 401`.
6. `curl --silent --write-out '%{http_code}\n' --output /dev/null 'https://pnpr.c.aquati.cat/~monochromatic-dev/@monochromatic-dev%2fmodule-or-throw'`
   prints `404`:
    the root npm surface is gone on this base.
7. The consumer route reads
   `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/npm/~monochromatic-dev/`
   in `.npmrc`,
   and consumer lockfiles re-resolved their tarball URLs.

### Restore

Status:
TODO | DONE

Only if the converted base must be rolled back.
 Under the standing no-backup decision there is no store snapshot,
 so the earlier base gets an empty store
 and the publish workflow republishes current versions
 (the same recovery the top-level Restore section uses).

1. Stop the pnpr application.
2. Empty the converted store,
    because `0.1.0-alpha.11` cannot read it
    and a leftover `layout.json` would make the store look current while its data sits elsewhere.
   Remove exactly what the conversion adds
    (`layout.json`,
    the `default`,
    `orgs`,
    and `.pnpr-cache` trees),
    leaving user records such as `htpasswd` in place:

   ```bash
   docker run --rm --entrypoint /bin/sh --volume <VOLUME>:/pnpr/storage:z <IMAGE> -c 'rm -rf /pnpr/storage/layout.json /pnpr/storage/default /pnpr/storage/orgs /pnpr/storage/.pnpr-cache'
   ```

   Expected:
    `ls /pnpr/storage` inside a throwaway container shows no package directories and no `layout.json`.
3. Revert the lockstep commit
    (the `Containerfile` `FROM` pin and the URL moves move back together)
    and let Coolify redeploy,
    or start the previous image.
   Expected:
    `curl --silent --write-out '%{http_code}\n' --output /dev/null https://pnpr.c.aquati.cat/-/ping` prints `200`.
4. Dispatch the `pnpr-publish` workflow
    (empty **only** input publishes every missing version).
   Expected:
    the run reports the republished versions
    and the root packument URL answers `200` again.
