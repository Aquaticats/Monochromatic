# Coolify Penpot 2.17.2 restart leaves backend unhealthy and frontend exited

A Coolify Penpot service using floating Penpot image tags can stop serving after a restart
when it pulls Penpot `2.17.2` but retains a Node.js backend healthcheck.
The current registry value of `penpotapp/backend:latest` does not contain `node`.
The live container version and readiness result must be checked before applying this diagnosis to an incident.

Investigated on 2026-09-02.

## Symptom

Coolify shows these states after restarting or redeploying the service:

```text
Frontend          Exited
Penpot Backend    Running (unhealthy)
Penpot Exporter   Running (healthy)
Mailpit           Running (healthy)
Postgres          Running (healthy)
Redis             Running (healthy)
```

The triggering Compose combination is:

```yaml
services:
  frontend:
    image: penpotapp/frontend:latest
    depends_on:
      penpot-backend:
        condition: service_healthy

  penpot-backend:
    image: penpotapp/backend:latest
    healthcheck:
      test:
        - CMD
        - node
        - -e
        - "require('http').get({host:'127.0.0.1', port:6060, path:'/readyz'}, res => process.exit(res.statusCode===200 ? 0 : 1)).on('error', () => process.exit(1));"
```

Running that healthcheck command in the current registry image produces:

```text
/bin/bash: line 1: node: command not found
```

The command exits with status `127`.

## Root cause

### The floating tag now selects Penpot 2.17.2

On 2026-09-02,
`skopeo inspect docker://docker.io/penpotapp/backend:latest` resolved to manifest
`sha256:770b55f6e51bfcee49152b30858ca6a47143256de8d43953a50b952b5c60bb55`.
Its OCI labels identify bundle version `2.17.2` and source revision
`1d2c37e52c733f74017d90b0fd1ae2d074a5c33d`.
Frontend and exporter `latest` also identified themselves as `2.17.2`.

This registry lookup does not prove which digest an existing Coolify container runs.
Confirm the live backend's `hint="welcome to penpot"` log entry and readiness result first.
A restart that pulls floating tags can change the Penpot version without any Compose edit.
Penpot's [Docker guide][penpot-docker-guide] recommends setting a version for more control
and recommends upgrading in small increments.

### Penpot 2.17.2 installs curl but not Node.js in the backend runtime image

The release source for
[`docker/images/Dockerfile.backend:50-100`][penpot-backend-dockerfile]
installs the runtime packages.
The list includes `curl` and `python3`,
but not Node.js:

```dockerfile
FROM ubuntu:26.04 AS image

RUN set -ex; \
    useradd -U -M -u 1001 -s /bin/false -d /opt/penpot penpot; \
    apt-get -qq update; \
    apt-get -qq dist-upgrade; \
    apt-get -qqy --no-install-recommends install \
        ca-certificates \
        curl \
        fontconfig \
        fontforge \
        python3 \
        python3-tabulate \
        tzdata \
        woff-tools \
        woff2
```

The image then starts only the Penpot Java application at
[`docker/images/Dockerfile.backend:102-112`][penpot-backend-command]:

```dockerfile
COPY --from=build /opt/jre /opt/jre

USER penpot:penpot
WORKDIR /opt/penpot/backend
CMD ["/bin/bash", "run.sh"]
```

The published image confirms the source reading:

```text
$ podman run --rm --entrypoint /bin/bash penpotapp/backend:2.17.2 \
    -lc 'printf "node="; command -v node || true; printf "curl="; command -v curl || true'
node=curl=/usr/bin/curl
```

By comparison,
the published `penpotapp/backend:2.11.1` image prints:

```text
node=/opt/node/bin/node
curl=/usr/bin/curl
```

The healthcheck was valid for `2.11.1`,
but it is not valid for `2.17.2`.

### The failed healthcheck blocks the frontend

Coolify's current Penpot template still couples the frontend to backend health at
[`templates/compose/penpot.yaml:13-17`][coolify-template-dependency]:

```yaml
depends_on:
  penpot-backend:
    condition: service_healthy
  penpot-exporter:
    condition: service_healthy
```

The same template's backend healthcheck invokes `node` at
[`templates/compose/penpot.yaml:58-62`][coolify-template-healthcheck]:

```yaml
healthcheck:
  test: ['CMD', 'node', '-e', "require('http').get({host:'127.0.0.1', port:6060, path:'/readyz'}, res => process.exit(res.statusCode===200 ? 0 : 1)).on('error', () => process.exit(1));"]
  interval: 10s
  timeout: 30s
  retries: 15
```

The current Coolify template avoids the incompatibility by pinning all Penpot images to
`2.11.1` at
[`templates/compose/penpot.yaml:10,28,65`][coolify-template-tags].
The affected service instead combines the Node.js healthcheck with floating `latest` tags.

The backend process can remain running while Docker labels its container unhealthy.
Because the frontend requires `service_healthy`,
Compose does not keep the frontend running.
When the live backend is `2.17.2`,
`node` is absent,
and the curl readiness probe succeeds,
this accounts for the exact combination of `Running (unhealthy)` and `Exited`.
A different live version or failed curl probe requires backend-log diagnosis instead.

## Verification

### Versions and source under test

- Published backend image `penpotapp/backend:2.17.2`,
  manifest `sha256:770b55f6e51bfcee49152b30858ca6a47143256de8d43953a50b952b5c60bb55`.
- Penpot release tag `2.17.2`,
  commit `1d2c37e52c733f74017d90b0fd1ae2d074a5c33d`.
- Published comparison image `penpotapp/backend:2.11.1`.
- Coolify source commit `b8866b87e8e855e041c21330352ca615521afed3`.

### Runnable probe

```bash
skopeo inspect docker://docker.io/penpotapp/backend:latest \
  | jq '{Digest, Created, Labels}'

podman run --rm --pull=always --entrypoint /bin/bash \
  docker.io/penpotapp/backend:2.17.2 \
  -lc 'printf "node="; command -v node || true; printf "curl="; command -v curl || true'

podman run --rm --entrypoint /bin/bash \
  docker.io/penpotapp/backend:2.17.2 \
  -lc "node -e \"require('http').get({host:'127.0.0.1', port:6060, path:'/readyz'}, res => process.exit(res.statusCode===200 ? 0 : 1)).on('error', () => process.exit(1));\""

podman run --rm --entrypoint /usr/bin/curl \
  docker.io/penpotapp/backend:2.17.2 \
  --version
```

A disposable Compose stack then ran PostgreSQL `15`,
Redis `7-alpine`,
and backend `2.17.2` with a `curl` healthcheck.
The containers were bounded to `2` CPUs and less than `2` GiB combined memory,
and the complete stack was removed with its disposable volumes after the probe.

```bash
podman-compose --project-name penpot-healthcheck-verification \
  --file compose.yaml up --detach

podman exec penpot-healthcheck-verification_backend_1 \
  curl --fail --silent --show-error http://127.0.0.1:6060/readyz

podman inspect --format '{{.State.Health.Status}}' \
  penpot-healthcheck-verification_backend_1
```

Observed output:

```text
OK
healthy
```

The original Node.js command against the same ready backend failed independently of application readiness:

```text
Error: crun: executable file `node` not found in $PATH: No such file or directory
node-exit=127
```

### Commands that work

- `command -v curl` in backend `2.17.2` returns `/usr/bin/curl`.
- `curl --fail --silent --show-error http://127.0.0.1:6060/readyz`
  returns `OK` and status `0` against the disposable backend.
- Docker reports `healthy` with that curl command configured as the healthcheck.
- `command -v node` in backend `2.11.1` returns `/opt/node/bin/node`.
- The backend source registers `/readyz` at
  [`backend/src/app/http/debug.clj:971`][penpot-ready-route].

### Commands that fail

- `command -v node` in backend `2.17.2` prints no path.
- The affected `node -e` healthcheck in backend `2.17.2` prints
  `node: command not found` and exits with status `127`.
- Repeating the affected healthcheck cannot make the container healthy,
  regardless of whether the Java server is ready.

## Verified workarounds

### Use curl for the backend readiness check

Replace only the backend healthcheck with:

```yaml
healthcheck:
  test:
    - CMD
    - curl
    - --fail
    - --silent
    - --show-error
    - http://127.0.0.1:6060/readyz
  interval: 10s
  timeout: 30s
  retries: 15
```

This uses the executable present in the published `2.17.2` runtime image
and checks Penpot's registered readiness route.

Tradeoff:
the check now depends on `curl` remaining in future backend images.
Pinning the image version makes that dependency explicit and reviewable.

### Pin every Penpot component to the confirmed live release

First confirm that the running backend logs report `version="2.17.2"`
and that its curl readiness probe returns `OK`.
Only then use the same explicit release for frontend,
backend,
and exporter:

```yaml
services:
  frontend:
    image: penpotapp/frontend:2.17.2

  penpot-backend:
    image: penpotapp/backend:2.17.2

  penpot-exporter:
    image: penpotapp/exporter:2.17.2
```

Tradeoff:
security and bug-fix releases no longer arrive implicitly.
Upgrades become an intentional edit and redeploy.

If the live backend is not already `2.17.2`,
do not use this block as an incidental upgrade.
Diagnose its actual healthcheck and application logs,
then plan a backed-up upgrade separately.
Do not downgrade a database that may already have run newer Penpot migrations.
If recovery requires an older release,
restore a matching pre-upgrade PostgreSQL and asset recovery set captured while Penpot writes were quiesced.
Separately timed database and asset backups are not guaranteed to represent one cross-store instant.

### Replace deprecated storage variable names after recovery

Penpot still translates the old `PENPOT_ASSETS_STORAGE_BACKEND=assets-fs` value.
The compatibility path is explicit at
[`backend/src/app/storage.clj:28-36`][penpot-legacy-storage]:

```clojure
(when-let [name (cf/get :assets-storage-backend)]
  (l/wrn :hint "using deprecated configuration, please read 2.11 release notes"
         :href "https://github.com/penpot/penpot/releases/tag/2.11.0")
  (case name
    :assets-fs :fs
    :assets-s3 :s3
    nil))
```

After service recovery is verified,
use the current names in a separate Compose edit:

```yaml
- PENPOT_OBJECTS_STORAGE_BACKEND=fs
- PENPOT_OBJECTS_STORAGE_FS_DIRECTORY=/opt/data/assets
```

Keep the existing `penpot-assets:/opt/data/assets` mount unchanged.

Tradeoff:
none for the filesystem backend when the directory and volume mount remain unchanged.
This is configuration cleanup rather than the cause of the unhealthy state.

## What does not work

- **Assuming the remote `latest` value proves the live version.**
  Coolify may still run a previously pulled digest;
  check the backend's `welcome to penpot` version log first.
- **Restarting without changing the Compose file.**
  Every recreated `2.17.2` backend executes the same absent `node` command.
- **Increasing healthcheck retries or timeout.**
  Status `127` means the executable is absent;
  more time cannot make it appear.
- **Disabling the healthcheck.**
  This can bypass the frontend dependency gate,
  but it also lets Coolify route or report the service without a readiness signal.
- **Changing the PostgreSQL or Redis volumes.**
  Both dependency services are already healthy,
  and the failure occurs before the healthcheck can make an HTTP request.
- **Blindly downgrading to Coolify's pinned `2.11.1`.**
  The restarted backend may already have applied `2.17.2` database migrations.
  A downgrade is safe only with data restored from a matching pre-upgrade backup.
- **Treating the deprecated storage variables as this incident's cause.**
  Penpot `2.17.2` still contains the compatibility translation quoted in
  "Replace deprecated storage variable names".

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` contains no Coolify,
Penpot,
Docker healthcheck,
or service-template exemption.

Tracker searches covered open and closed Coolify issues and pull requests for
`penpot healthcheck`,
`penpot node curl`,
and `penpot command not found`.
Related pull request
[coollabsio/coolify#6272][coolify-pr-6272]
changed the template from `curl` to `node` when Penpot `2.8.0` temporarily lacked curl.
Penpot issue
[penpot/penpot#6900][penpot-issue-6900]
records that curl was later reintroduced.
No report for the current inverse mismatch was found.

The filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
    No.
    The affected service combines a floating image tag with a healthcheck tied to an older image's toolset.
    Coolify's current template pins `2.11.1`,
    while the shown service uses `latest`.
2.  **Can upstream fix it?**
    Coolify can update its template to a newer pinned Penpot release and a curl healthcheck,
    but that would not retroactively edit this persisted service definition.
3.  **Are they supporting this use case?**
    Yes.
    Coolify ships a Penpot service template,
    and Penpot documents Docker Compose self-hosting.
4.  **Would the repo welcome our contribution?**
    Yes with conditions.
    `CONTRIBUTING.md:221` permits verified AI-assisted contributions,
    and `.github/pull_request_template.md:25-34` requires disclosure.
    The merged related fix in pull request `#6272` is another positive signal.
5.  **Will they likely fix it?**
    Not applicable to this persisted configuration.
    The current template already prevents floating-tag drift by pinning `2.11.1`.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    No upstream patch is warranted because constraint 1 fails.
    The consumer-side Compose replacement was verified with a disposable full stack:
    curl returned `OK`,
    Docker reported `healthy`,
    and the Node.js command failed with status `127` against the same ready backend.

The decision is not to file an upstream issue or prepare a patch.
The recovery belongs in the affected Coolify service configuration.

### Draft (do not file as-is)

~~~md
Title: Penpot service can combine a Node.js backend healthcheck with an image that lacks Node.js

## Description

A persisted or customized Penpot service using `penpotapp/backend:latest`
can resolve to Penpot 2.17.2 while retaining Coolify's Node.js `/readyz` healthcheck.
The 2.17.2 backend image contains `/usr/bin/curl` but no `node`,
so the healthcheck exits with status 127 and blocks the frontend's
`condition: service_healthy` dependency.

Coolify's current template pins Penpot 2.11.1,
so this is not reproducible from the current template without changing its image tags to `latest`.
Do not file this as a Coolify defect unless a current-template creation or update path is shown
to produce that combination automatically.
~~~

[coolify-pr-6272]: https://github.com/coollabsio/coolify/pull/6272
[coolify-template-dependency]: https://github.com/coollabsio/coolify/blob/b8866b87e8e855e041c21330352ca615521afed3/templates/compose/penpot.yaml#L13-L17
[coolify-template-healthcheck]: https://github.com/coollabsio/coolify/blob/b8866b87e8e855e041c21330352ca615521afed3/templates/compose/penpot.yaml#L58-L62
[coolify-template-tags]: https://github.com/coollabsio/coolify/blob/b8866b87e8e855e041c21330352ca615521afed3/templates/compose/penpot.yaml#L10-L65
[penpot-backend-command]: https://github.com/penpot/penpot/blob/1d2c37e52c733f74017d90b0fd1ae2d074a5c33d/docker/images/Dockerfile.backend#L102-L112
[penpot-backend-dockerfile]: https://github.com/penpot/penpot/blob/1d2c37e52c733f74017d90b0fd1ae2d074a5c33d/docker/images/Dockerfile.backend#L50-L100
[penpot-docker-guide]: https://help.penpot.app/technical-guide/getting-started/docker/
[penpot-issue-6900]: https://github.com/penpot/penpot/issues/6900
[penpot-legacy-storage]: https://github.com/penpot/penpot/blob/1d2c37e52c733f74017d90b0fd1ae2d074a5c33d/backend/src/app/storage.clj#L28-L36
[penpot-ready-route]: https://github.com/penpot/penpot/blob/1d2c37e52c733f74017d90b0fd1ae2d074a5c33d/backend/src/app/http/debug.clj#L971
