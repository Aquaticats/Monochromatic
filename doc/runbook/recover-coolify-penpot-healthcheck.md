# Recover Coolify Penpot after the backend healthcheck loses Node.js

## What this proves

This procedure tests and repairs one specific Coolify Penpot failure:

```text
Frontend          Exited
Penpot Backend    Running (unhealthy)
Penpot Exporter   Running (healthy)
Postgres          Running (healthy)
Redis             Running (healthy)
```

The candidate cause is documented in
[`doc/troubleshooting/coolify-penpot-latest-node-healthcheck.md`](../troubleshooting/coolify-penpot-latest-node-healthcheck.md).
On 2026-09-02,
the registry's `penpotapp/backend:latest` value is Penpot `2.17.2`.
That image contains `curl` but not `node`,
while the affected Compose healthcheck executes `node`.

The registry value does not prove which image an existing Coolify container runs.
This runbook checks the live version and readiness endpoint before changing Compose.

Direct mutation was not possible because this repository has no Coolify endpoint or authorization for the deployment.
The public endpoint `https://penpot.c.aquati.cat` was probed and returned HTTP `502`.
The replacement healthcheck was verified against a disposable Penpot `2.17.2` stack with PostgreSQL and Redis:
`curl` returned `OK`,
Docker reported `healthy`,
and the original Node.js command failed with status `127` against the same ready backend.

If Penpot starts but browser requests time out at external port `8080`,
use
[`doc/runbook/correct-coolify-penpot-public-uri.md`](correct-coolify-penpot-public-uri.md)
as a separate configuration repair.
Do not repeat this healthcheck recovery for that follow-up symptom.

## Setup

Status:
TODO

Prerequisites:

- Browser with network access to the Coolify dashboard that owns `https://penpot.c.aquati.cat`.
- Coolify account with permission to view logs,
  open a service terminal,
  manage backups,
  update Compose,
  and deploy the Penpot service.
- Existing service containing the named volumes `penpot-assets`,
  `penpot-postgresql-data`,
  and `penpot-redis-data`.
- No concurrent Penpot deployment or backup in progress.

Obtain the deployment-specific Coolify dashboard URL from the server operator or deployment records.
Do not use `https://penpot.c.aquati.cat` as the dashboard URL;
that is the Penpot application URL.

1.  Open the Coolify dashboard URL in the browser.
    The Coolify sign-in page or dashboard appears.
2.  Sign in with the authorized Coolify account.
    The Coolify dashboard appears without an authorization error.
3.  Open **Projects**.
    The project list appears.
4.  Open the project and environment containing the Penpot service for `penpot.c.aquati.cat`.
    The environment's resource list appears.
5.  Open the Penpot service.
    The service page shows frontend,
    backend,
    exporter,
    PostgreSQL,
    and Redis resources.

## Steps

Status:
TODO

### Confirm this diagnosis

1.  Open **Runtime Logs**.
    The service log viewer appears.
2.  Select **Penpot Backend**.
    Backend startup logs appear.
3.  Find the line containing `hint="welcome to penpot"`.
    Record its exact `version="..."` value.
4.  If no `welcome to penpot` line exists,
    preserve the complete log from the first `exec` line through the final error and stop this runbook.
    The backend has an application startup failure rather than only a healthcheck failure.
5.  Open **Terminal**.
    The service terminal page appears.
6.  Select the **Penpot Backend** container.
    The terminal prompt opens inside the running backend container.
7.  Run:

    ```bash
    command -v node || true
    ```

    For the diagnosed `2.17.2` mismatch,
    the command prints no path.
8.  Run:

    ```bash
    command -v curl || true
    ```

    For backend `2.17.2`,
    the command prints `/usr/bin/curl`.
9.  Run:

    ```bash
    curl --fail --silent --show-error http://127.0.0.1:6060/readyz
    ```

    The diagnosed healthcheck-only failure prints `OK` and returns to the prompt without an error.
10. If `node` has a path,
    `curl` has no path,
    or the readiness request does not print `OK`,
    preserve the backend logs and stop this runbook.
    Those results disprove or fail to confirm this diagnosis.

### Back up persistent data

Coolify's storage backups are file-level archives.
Use the database-aware backup for PostgreSQL.
Use a storage backup for `penpot-assets` with container stopping enabled,
which prevents asset writes during that archive.
These independently valid backups are not a coordinated cross-store recovery point:
the backend can write between the database dump and asset archive.
Treat them as protection from operator mistakes during this same-version configuration repair,
not as proof that an arbitrary version rollback can restore an exact instant.
A coordinated rollback backup must keep all write-producing Penpot containers stopped across both captures.
A local backup protects against this Compose change but not loss of the deployment server;
configure S3 separately when off-server protection is required.

11. Open **Backups**.
    The **Backups** page lists database and storage backup schedules.
12. If a PostgreSQL database schedule exists,
    click its **Back up now** button.
    Its status changes to **In progress**.
13. If no PostgreSQL database schedule exists,
    click **Add backup** and then **Database backup**.
    The **New database backup** dialog opens.
14. In **Database**,
    select the Penpot PostgreSQL service.
    The selected database appears in the field.
15. Enter `daily` in **Frequency**.
    The frequency field contains `daily`.
16. Click **Add schedule**.
    The PostgreSQL schedule appears on **Backups**.
17. Click that schedule's **Back up now** button.
    Its status changes to **In progress**.
18. Open the PostgreSQL schedule's **Executions** view after the run finishes.
    The newest execution shows **Success** rather than **Failed**.
19. If a `penpot-assets` storage schedule exists,
    open it and select **General**.
    The **Backup schedule** settings appear.
20. If no `penpot-assets` storage schedule exists,
    return to **Backups**,
    click **Add backup**,
    and then click **Storage backup**.
    The **New storage backup** dialog opens.
21. Select `penpot-assets` in **Backup target**.
    The field identifies the frontend or backend asset volume.
22. Enter `daily` in **Frequency**.
    The frequency field contains `daily`.
23. Click **Create schedule**.
    The `penpot-assets` schedule opens.
24. In **Archive behavior**,
    select **Stop containers during archive**.
    Coolify saves the safer archive behavior.
25. Click **Back up now**.
    The asset backup starts and its status changes to **In progress**.
26. Open **Executions** after the asset backup finishes.
    The newest execution shows **Success** and **Local Storage** or **S3 Storage** under backup availability.
27. If either backup shows **Failed**,
    read and preserve its execution message and stop this runbook.
    Do not redeploy without a successful PostgreSQL backup and asset backup.

### Repair and pin the healthcheck

28. Open **Settings** and then **General** if the configuration sidebar is present.
    The **Service details** section appears.
29. Click **Edit Compose file**.
    The **Docker Compose** editor opens.
30. Copy the complete current Compose text to a local text file outside Coolify.
    The saved file contains the pre-recovery image tags,
    environment variables,
    healthchecks,
    top-level volume declarations,
    volume mounts,
    and service names.
31. Confirm the saved and editable Compose text still names all three volume sources:

    ```text
    penpot-assets
    penpot-postgresql-data
    penpot-redis-data
    ```

    No volume source has been renamed or removed.
32. If the recorded backend version is `2.17.2`,
    change the frontend image line to:

    ```yaml
    image: 'penpotapp/frontend:2.17.2'
    ```

    The frontend is pinned to the already-running backend release.
33. If the recorded backend version is `2.17.2`,
    change the backend image line to:

    ```yaml
    image: 'penpotapp/backend:2.17.2'
    ```

    The backend is pinned without changing its running release.
34. If the recorded backend version is `2.17.2`,
    change the exporter image line to:

    ```yaml
    image: 'penpotapp/exporter:2.17.2'
    ```

    Every Penpot application component now uses release `2.17.2`.
35. If the recorded backend version differs from `2.17.2`,
    stop this runbook before changing any image line.
    Do not turn healthcheck recovery into an unplanned Penpot upgrade or downgrade.
36. Replace the complete `penpot-backend.healthcheck` block with:

    ```yaml
    healthcheck:
      test:
        - CMD
        - curl
        - --fail
        - --silent
        - --show-error
        - 'http://127.0.0.1:6060/readyz'
      interval: 10s
      timeout: 30s
      retries: 15
    ```

    The healthcheck now uses `/usr/bin/curl`,
    which the live-container probe confirmed.
37. Leave these existing storage entries unchanged during recovery:

    ```yaml
    - PENPOT_ASSETS_STORAGE_BACKEND=assets-fs
    - PENPOT_STORAGE_ASSETS_FS_DIRECTORY=/opt/data/assets
    ```

    Recovery remains limited to version pinning and the healthcheck.
38. Confirm the editable Compose text still contains these three mounts exactly:

    ```yaml
    - 'penpot-assets:/opt/data/assets'
    - 'penpot-postgresql-data:/var/lib/postgresql/data'
    - 'penpot-redis-data:/data'
    ```

    All persistent mounts remain attached to their original container paths.
39. Click **Save changes**.
    Coolify confirms the Compose update and the save control stops showing a loading state.
40. If the service status is **Degraded**,
    click **Actions** and then **Force Restart**.
    The **Service Startup** deployment log opens and Coolify recreates the service containers.
41. If the service status is **Stopped** instead,
    click **Actions** and then **Force Deploy**.
    The **Service Startup** deployment log opens and Coolify recreates the service containers.
42. Leave the deployment log open until it finishes.
    The final log reports success rather than
    `dependency failed to start: container penpot-backend ... is unhealthy`.
43. Open `https://penpot.c.aquati.cat` in a new browser tab.
    The Penpot login page or authenticated dashboard appears instead of an HTTP `502` page.

Only one deployment branch in steps 40 and 41 applies.
Do not run both.
Current Coolify source implements both paths with Compose `--force-recreate`,
which is required because a plain Docker container restart would retain the old healthcheck.

## What to check

Status:
TODO

Confirm all of the following before closing the incident:

- **Penpot Backend** shows `Running (healthy)`.
- **Frontend** shows `Running (healthy)` or `Running`.
- **Penpot Exporter** shows `Running (healthy)`.
- **Postgres** shows `Running (healthy)`.
- **Redis** shows `Running (healthy)`.
- Backend logs contain `hint="welcome to penpot"` and `version="2.17.2"`.
- Backend logs contain `hint="starting http server"` and `port=6060`.
- Deployment logs do not contain `node: command not found`.
- Deployment logs do not contain
  `dependency failed to start: container penpot-backend`.
- `https://penpot.c.aquati.cat` no longer returns HTTP `502`.
- Existing teams and files are visible after login.
- Opening one existing file loads its canvas and assets.
- Exporting one disposable test shape completes and downloads an output file.

If the backend remains unhealthy,
open **Runtime Logs**,
select **Penpot Backend**,
and preserve the complete startup log from the first `exec` line through the final error.
A database migration,
secret,
or storage error is a separate incident from the repaired healthcheck.
Do not delete or recreate volumes while diagnosing it.

After recovery,
replace the deprecated storage variable names in a separate backed-up change:

```yaml
- PENPOT_OBJECTS_STORAGE_BACKEND=fs
- PENPOT_OBJECTS_STORAGE_FS_DIRECTORY=/opt/data/assets
```

That cleanup is not required to recover this incident.

## Restore

Status:
TODO

The pre-recovery configuration is broken when paired with backend `2.17.2`.
Restore it only to investigate a distinct regression,
and expect the original outage to return.

1.  Open the Penpot service's **Settings**.
    The settings page appears.
2.  Open **General** if the configuration sidebar is present.
    The **Service details** section appears.
3.  Click **Edit Compose file**.
    The **Docker Compose** editor opens.
4.  Replace the editor contents with the complete pre-recovery Compose text saved during setup.
    The editor again shows the original tags,
    healthcheck,
    environment entries,
    and volume definitions.
5.  Confirm the Compose text still contains `penpot-assets`,
    `penpot-postgresql-data`,
    and `penpot-redis-data`.
    All three named volume declarations remain present.
6.  Click **Save changes**.
    Coolify confirms the Compose update.
7.  Click **Actions** and use **Force Restart** for a degraded service or **Force Deploy** for a stopped service.
    The deployment log opens and applies the restored configuration through container recreation.

Never use **Force Cleanup Containers**,
delete the Penpot service,
run Compose with `--volumes`,
or remove any of the three named volumes as part of this recovery.

Do not downgrade Penpot to `2.11.1` against the current database.
If a version rollback becomes necessary,
use a pre-upgrade recovery set captured while Penpot writes were quiesced,
restore its PostgreSQL and asset data,
then deploy the Penpot version matching that set.
Do not treat the two independently created recovery backups in this runbook as a coordinated rollback set.
Coolify does not restore storage archives from its storage-backup page,
so test and document that restore path separately before relying on it for rollback.
