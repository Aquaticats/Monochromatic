# Override a Coolify v4 shared Penpot public URI in the frontend

## What this proves

This procedure repairs a running Penpot service whose browser log contains:

```text
public-uri="https://penpot.c.aquati.cat:8080/"
```

The resulting browser request fails with:

```text
:8080/api/main/methods/get-enabled-flags:1 Failed to load resource: net::ERR_CONNECTION_TIMED_OUT
```

The diagnosed call chain is documented in
[`doc/troubleshooting/coolify-penpot-public-uri-internal-port.md`][public-uri-diagnosis].
The Source Compose can omit frontend `PENPOT_PUBLIC_URI` while Coolify v4 still injects a service-wide value through
`env_file: .env`.
The narrow repair adds an explicit frontend value,
which takes precedence over that shared file.

The public application and configuration endpoint were reachable for read-only probes,
but this repository has no Coolify endpoint or authorization for deployment mutation.
The live Compose edit must therefore be performed in the authorized Coolify dashboard.

## Setup

Status:
TODO

Prerequisites:

- Browser with network access to the Coolify dashboard that owns `https://penpot.c.aquati.cat`.
- Coolify account allowed to view source and deployable Compose,
  update the service,
  and recreate its containers.
- Running or degraded Penpot service with frontend,
  backend,
  exporter,
  PostgreSQL,
  and Redis components.
- No concurrent Penpot deployment or backup.

1.  Open the deployment's Coolify dashboard in a browser.
    The Coolify sign-in page or dashboard appears.
2.  Sign in with the authorized Coolify account.
    The dashboard appears without an authorization error.
3.  Open **Projects**.
    The project list appears.
4.  Open the project and environment containing `penpot.c.aquati.cat`.
    The environment's resource list appears.
5.  Open the Penpot service.
    Component cards for frontend,
    backend,
    exporter,
    PostgreSQL,
    and Redis appear.
6.  Record the service's current status.
    The status is available for choosing one recreation action later.

## Steps

Status:
TODO

1.  Open **Configuration** and then **General**.
    The service's general configuration appears.
2.  Click **Edit Compose File**.
    The **Source Compose** editor opens.
3.  Copy the complete source Compose text to a local file outside Coolify.
    The file preserves the exact pre-repair configuration for rollback.
4.  Confirm the saved and editable definitions still contain these volume names:

    ```text
    penpot-assets
    penpot-postgresql-data
    penpot-redis-data
    ```

    All persistent data sources remain represented before the edit.
5.  Find `services.frontend.environment` in **Source Compose**.
    The list contains `SERVICE_URL_FRONTEND_8080` and `PENPOT_FLAGS`.
6.  Keep this frontend routing declaration unchanged:

    ```yaml
    - SERVICE_URL_FRONTEND_8080
    ```

    Coolify retains port `8080` as the frontend container's internal target.
7.  Add this entry to the same frontend environment list:

    ```yaml
    - 'PENPOT_PUBLIC_URI=https://penpot.c.aquati.cat'
    ```

    Frontend now explicitly overrides Coolify's shared service environment.
8.  Leave the existing backend and exporter public URI entries unchanged:

    ```yaml
    - 'PENPOT_PUBLIC_URI=https://penpot.c.aquati.cat'
    ```

    Backend and exporter retain their already-correct values.
9.  Leave the custom Caddy configuration and frontend port mapping unchanged:

    ```yaml
    - '127.0.0.1:1009:8080'
    ```

    This repair changes only frontend application configuration.
10. Confirm the editable Compose text still contains these mounts exactly:

    ```yaml
    - 'penpot-assets:/opt/data/assets'
    - 'penpot-postgresql-data:/var/lib/postgresql/data'
    - 'penpot-redis-data:/data'
    ```

    No persistent mount was renamed or removed.
11. Click **Save changes**.
    Coolify confirms the Compose update and reparses the service.
12. Open **Deployable Compose**.
    The generated definition appears.
13. Find the frontend `environment:` block.
    It contains explicit `PENPOT_PUBLIC_URI=https://penpot.c.aquati.cat` without `:8080`.
14. If the explicit frontend value is absent or includes `:8080`,
    copy the complete **Deployable Compose** text to a local file and stop this runbook.
    The file preserves evidence of a separate Coolify parsing problem.
15. If the recorded service status was **Stopped**,
    click **Actions** and then **Force Deploy**.
    The **Service Startup** deployment log opens and Coolify recreates the containers.
16. If the recorded service status was not **Stopped**,
    click **Actions** and then **Force Restart**.
    The **Service Startup** deployment log opens and Coolify recreates the containers.
17. Leave the deployment log open until it finishes.
    The final deployment line reports success rather than an unhealthy dependency.
18. Open `https://penpot.c.aquati.cat/js/config.js?public-uri-check=20260903` in a new browser tab.
    The cache-busting URL returns
    `var penpotPublicURI = "https://penpot.c.aquati.cat";` without `:8080`.
19. Open a new private browser window with **Ctrl+Shift+N** in Chromium-based browsers.
    A private window appears with a separate browser cache.
20. Open `https://penpot.c.aquati.cat` in the private window.
    The Penpot login page appears through normal HTTPS without a timeout.
21. Open browser developer tools with **F12**.
    Developer tools appear.
22. Select **Console**.
    Penpot's startup messages appear.
23. Find the startup message containing `public-uri=`.
    It contains `public-uri="https://penpot.c.aquati.cat/"` without `:8080`.
24. Select **Network**.
    The network request list appears.
25. Reload the page with **Ctrl+R**.
    New Penpot requests populate the list.
26. Filter for `get-enabled-flags`.
    Its request URL starts with
    `https://penpot.c.aquati.cat/api/main/methods/get-enabled-flags` and has no `:8080`.

Only one recreation branch in steps 15 and 16 applies.
Do not run both.
A Compose save does not change an existing container's environment,
so container recreation is required for the frontend entrypoint to regenerate `js/config.js`.

## What to check

Status:
TODO

Confirm all of the following before closing the incident:

- `https://penpot.c.aquati.cat/js/config.js?public-uri-check=20260903` contains
  `var penpotPublicURI = "https://penpot.c.aquati.cat";`.
- Browser logs contain `public-uri="https://penpot.c.aquati.cat/"` without `:8080`.
- Browser requests use `https://penpot.c.aquati.cat/api/` without `:8080`.
- Browser logs do not contain `net::ERR_CONNECTION_TIMED_OUT` for `get-enabled-flags`.
- Caddy continues serving `https://penpot.c.aquati.cat` on standard HTTPS.
- Backend,
  frontend,
  exporter,
  PostgreSQL,
  and Redis retain their pre-repair healthy or running states.
- Existing teams and files remain visible after login.
- Opening an existing file loads its canvas and assets.
- Exporting one disposable test shape completes and downloads an output file.
- The three original volume names and mount destinations remain unchanged in **Deployable Compose**.

The verified unauthenticated browser boundary produced:

```text
public-uri="https://penpot.c.aquati.cat/"
GET https://penpot.c.aquati.cat/api/main/methods/get-enabled-flags 401
```

The immediate `401` is the expected unauthenticated boundary response.
It replaces the previous connection timeout and proves that the browser now uses standard HTTPS.
Authenticated project,
asset,
and export checks remain required.

## Restore

Status:
TODO

Configuration rollback is sufficient for this environment-only change.
Do not restore PostgreSQL or asset backups merely because the public URI deployment fails.

1.  Open the Penpot service's **Configuration** and then **General**.
    The service's general configuration appears.
2.  Click **Edit Compose File**.
    The **Source Compose** editor opens.
3.  Replace the editor contents with the complete pre-repair Compose text saved during setup.
    The explicit frontend public URI entry disappears.
4.  Confirm the restored text still names `penpot-assets`,
    `penpot-postgresql-data`,
    and `penpot-redis-data`.
    All persistent data sources remain represented.
5.  Click **Save changes**.
    Coolify confirms the restored Compose definition.
6.  If the service is **Stopped**,
    click **Actions** and then **Force Deploy**.
    Coolify recreates the containers from the restored definition.
7.  If the service is not **Stopped**,
    click **Actions** and then **Force Restart**.
    Coolify recreates the containers from the restored definition.
8.  Leave the deployment log open until it finishes.
    The final deployment line reports success or reproduces the inherited public URI symptom.

Never delete or rename the service,
run Compose with `--volumes`,
use **Force Cleanup Containers**,
or remove any Penpot volume during this repair or rollback.

[public-uri-diagnosis]: ../troubleshooting/coolify-penpot-public-uri-internal-port.md
