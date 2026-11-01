# Inspect a Coolify LibreChat vector database reported twice as exited

## What this proves

This read-only procedure distinguishes:

- real `vectordb` containers from duplicate Coolify component records;
- a stopped PostgreSQL process from a stale Coolify status;
- an old Coolify build susceptible to the empty-query false-exit bug from a current build;
- the installed LibreChat image's actual health endpoint from its configured health check;
- the Coolify resource record and named volume attached to the live container.

Direct automation was unavailable when this runbook was written.
No authenticated Coolify browser session,
Coolify API endpoint,
or API token was available to the agent.
The service-level Coolify terminal also reported that no containers were available,
so it could not inspect the destination Docker daemon.

## Setup

Status:
TODO

Use a browser with owner access to the Coolify instance.
For self-hosted Coolify,
have SSH access to the Coolify control-plane host.
Have separate SSH access to the destination server that runs the LibreChat containers.
These can be the same server,
but do not assume that they are.
Have permission to run Docker commands on each applicable host.

The commands do not print container environment variables or credentials.
Review log output before sharing it because application logs can contain local identifiers.

Sign in to Coolify,
open **Projects**,
select the project and environment containing LibreChat,
select the LibreChat Service,
and open its **Configuration** page.
The required starting state is the LibreChat **Configuration** page with both **Vectordb** cards visible.
Record the displayed Coolify project,
environment,
and Service names.
Copy the Service UUID from the URL segment immediately after `/service/`.

In the command examples,
replace uppercase placeholders such as `SERVICE_UUID`,
`COMPOSE_PROJECT`,
and `VECTORDB_CONTAINER_ID` with observed values.
Do not type the placeholder words literally.

## Steps

Status:
TODO

1.  Read the version displayed beside **Coolify** in the top bar.
    The expected outcome is a value such as `v4.3.14`.
    Record this rendered version rather than inferring it from a mutable container image reference.

2.  Confirm that the open page is the LibreChat **Configuration** page.
    The expected outcome is a Compose resource list containing both **Vectordb** cards.

3.  Open the first **Vectordb** resource card in a new browser tab.
    The expected outcome is a settings page for that displayed card.

4.  Record the final UUID in the first card's URL and the sentence beneath its **Vectordb** heading.
    The expected sentence contains either
    `Identity, image, and public access for this compose application.`
    or
    `Identity, image, and public access for this compose database.`

5.  Return to **Configuration** and open the second **Vectordb** resource card in another browser tab.
    The expected outcome is a settings page for the other displayed card.

6.  Record the final UUID in the second card's URL and the sentence beneath its **Vectordb** heading.
    The expected sentence contains either exact value from step 4.

7.  Open a terminal on the workstation and connect with `ssh` to the destination server that runs LibreChat.
    The expected outcome is a shell prompt on that destination server.

8.  Confirm the active Docker context:

    ```bash
    docker context show
    ```

    The expected outcome identifies the intended destination context.
    Stop if it identifies another context.

9.  Confirm the Docker daemon identity:

    ```bash
    docker info --format 'Name={{.Name}} ServerVersion={{.ServerVersion}} RootDir={{.DockerRootDir}}'
    ```

    The expected outcome identifies the intended destination Docker daemon.
    Stop if it identifies another host.

10. Run an unfiltered container inventory as a positive control:

    ```bash
    docker ps --all --no-trunc \
      --format 'table {{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}'
    ```

    The expected outcome includes known destination containers.
    If it prints only the header,
    resolve the wrong-host,
    wrong-context,
    or Docker-daemon problem before trusting any filtered result.

11. Run an unfiltered volume inventory as a positive control:

    ```bash
    docker volume ls --format 'table {{.Name}}\t{{.Driver}}'
    ```

    The expected outcome includes named volumes on this destination daemon.

12. Inspect Coolify-managed Compose Service containers with non-secret identity labels:

    ```bash
    docker inspect $(docker ps --all --quiet \
      --filter 'label=coolify.managed=true' \
      --filter 'label=coolify.type=service') \
      --format 'ID={{.Id}} Name={{.Name}} Status={{.State.Status}} ComposeProject={{index .Config.Labels "com.docker.compose.project"}} ComposeService={{index .Config.Labels "com.docker.compose.service"}} CoolifyServiceId={{index .Config.Labels "coolify.serviceId"}} CoolifyProject={{index .Config.Labels "coolify.projectName"}} CoolifyEnvironment={{index .Config.Labels "coolify.environmentName"}} CoolifyResource={{index .Config.Labels "coolify.resourceName"}}'
    ```

    The expected outcome lets the reader match the recorded Coolify project,
    environment,
    and Service names to a Compose project without relying on the Service UUID.
    Accept only one matching `CoolifyServiceId` and `ComposeProject` pair.
    If no pair or multiple pairs match,
    do not select one by name.
    On self-hosted Coolify,
    complete steps 26 through 28 to obtain `TARGET_SERVICE_ID`,
    then rerun this step with
    `--filter 'label=coolify.serviceId=TARGET_SERVICE_ID'` in place of both filters.
    Coolify Cloud requires an unambiguous label match or a supported API result.

13. Inspect all Compose containers whose service key is `vectordb`:

    ```bash
    docker inspect $(docker ps --all --quiet \
      --filter 'label=com.docker.compose.service=vectordb') \
      --format 'ID={{.Id}} Name={{.Name}} Image={{.Config.Image}} Status={{.State.Status}} ComposeProject={{index .Config.Labels "com.docker.compose.project"}} CoolifyServiceId={{index .Config.Labels "coolify.serviceId"}} CoolifyProject={{index .Config.Labels "coolify.projectName"}} CoolifyEnvironment={{index .Config.Labels "coolify.environmentName"}} CoolifyResource={{index .Config.Labels "coolify.resourceName"}}'
    ```

    The expected outcome is one row per real `vectordb` container across all Compose projects.
    Select a row only when its Compose project and `CoolifyServiceId` match the unique pair from step 12.

14. If the preceding command prints no matching `vectordb` row,
    search container names as a compatibility fallback:

    ```bash
    docker ps --all --no-trunc \
      --filter 'name=vectordb' \
      --format 'table {{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}'
    ```

    The expected outcome is either a concrete `vectordb` container row or only the header.

15. List detached Compose volume candidates for the `vectordb-data` key:

    ```bash
    docker volume ls \
      --filter 'label=com.docker.compose.volume=vectordb-data' \
      --format 'table {{.Name}}\t{{.Driver}}'
    ```

    The expected outcome includes the PostgreSQL volume even when its container is absent.
    A header-only result triggers the name fallback in step 17.

16. For each candidate from step 15,
    run this command once:

    ```bash
    docker volume inspect VECTORDB_VOLUME_NAME \
      --format 'Name={{.Name}} Project={{index .Labels "com.docker.compose.project"}} ComposeVolume={{index .Labels "com.docker.compose.volume"}} Mountpoint={{.Mountpoint}}'
    ```

    Each run should identify its Compose project.
    Match it to `COMPOSE_PROJECT` from step 12 when that project still has containers.

17. If step 15 found no candidate,
    search volume names as a compatibility fallback:

    ```bash
    docker volume ls \
      --filter 'name=vectordb' \
      --format 'table {{.Name}}\t{{.Driver}}'
    ```

    The expected outcome is a volume candidate or only the header.
    Run step 16 for every candidate found by this fallback.
    Do not choose between multiple candidates by recency or name similarity alone.

18. For each `vectordb` container ID found,
    run this command once:

    ```bash
    docker inspect VECTORDB_CONTAINER_ID \
      --format 'State={{json .State}} Image={{.Config.Image}} Healthcheck={{json .Config.Healthcheck}}{{println}}ComposeProject={{index .Config.Labels "com.docker.compose.project"}} ComposeService={{index .Config.Labels "com.docker.compose.service"}}{{println}}CoolifyServiceId={{index .Config.Labels "coolify.serviceId"}} CoolifySubType={{index .Config.Labels "coolify.service.subType"}} CoolifySubId={{index .Config.Labels "coolify.service.subId"}}{{println}}{{range .Mounts}}Mount={{.Name}}:{{.Destination}}{{println}}{{end}}'
    ```

    Each run should contain `State=`,
    `Image=`,
    `ComposeService=vectordb`,
    Coolify resource labels,
    and a mount ending in `/var/lib/postgresql/data`.
    Empty Coolify labels are evidence to record,
    not values to guess.

19. Inspect every container in the identified LibreChat Compose project:

    ```bash
    docker inspect $(docker ps --all --quiet \
      --filter 'label=com.docker.compose.project=COMPOSE_PROJECT') \
      --format 'ID={{.Id}} Name={{.Name}} Image={{.Config.Image}} Status={{.State.Status}} ComposeService={{index .Config.Labels "com.docker.compose.service"}} CoolifyServiceId={{index .Config.Labels "coolify.serviceId"}}'
    ```

    The expected outcome includes each actual LibreChat stack container that currently exists.
    `docker inspect` reports that it needs an argument when only a detached volume identifies the project.

20. For each MongoDB,
    Meilisearch,
    vector database,
    and LibreChat container ID from step 19,
    run this command once:

    ```bash
    docker inspect CONTAINER_ID \
      --format 'Name={{.Name}}{{println}}{{range .Mounts}}Mount={{.Name}}:{{.Destination}}{{println}}{{end}}'
    ```

    Each run should record the current named-volume identity and destination without printing environment variables.

21. For each `vectordb` container ID,
    read its latest PostgreSQL output:

    ```bash
    docker logs --timestamps --tail 200 VECTORDB_CONTAINER_ID
    ```

    Each run should return up to 200 timestamped log lines or a concrete Docker error.

22. Discover the installed LibreChat container in the identified project:

    ```bash
    docker ps --all --no-trunc \
      --filter 'label=com.docker.compose.project=COMPOSE_PROJECT' \
      --filter 'label=com.docker.compose.service=librechat' \
      --format 'table {{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}'
    ```

    The expected outcome is the installed LibreChat container row or only the header.

23. If a LibreChat container exists,
    inspect its configured health check:

    ```bash
    docker inspect LIBRECHAT_CONTAINER_ID \
      --format 'State={{json .State}} Healthcheck={{json .Config.Healthcheck}}'
    ```

    The expected outcome records both the configured command and Docker's latest health-check evidence.

24. If the LibreChat container is running,
    probe its current `/health` endpoint:

    ```bash
    docker exec LIBRECHAT_CONTAINER_ID \
      wget --no-verbose --tries=1 --spider http://127.0.0.1:3080/health
    ```

    The expected healthy output includes
    `remote file exists`.
    Record an exact `404 Not Found`,
    connection error,
    or missing-command error instead of substituting another tool.

25. If the LibreChat container is running,
    probe its configured legacy `/api/health` endpoint:

    ```bash
    docker exec LIBRECHAT_CONTAINER_ID \
      wget --no-verbose --tries=1 --spider http://127.0.0.1:3080/api/health
    ```

    The expected obsolete-route result is `404 Not Found`.
    Different output proves that the installed artifact differs from the reviewed current source.

26. On self-hosted Coolify,
    connect with `ssh` to the Coolify control-plane host.
    The expected outcome is a shell prompt on the host running the Coolify application and `coolify-db` containers.
    Skip steps 27 through 29 for Coolify Cloud.

27. Identify the Coolify database container without assuming it shares the destination host:

    ```bash
    docker ps --all --no-trunc \
      --filter 'name=coolify-db' \
      --format 'table {{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}'
    ```

    The expected outcome contains the PostgreSQL container that stores Coolify's control-plane metadata.

28. Read the application and database records for this Service:

    ```bash
    docker exec COOLIFY_DB_CONTAINER_ID \
      psql --username=coolify --dbname=coolify --csv \
      --command="SELECT s.id AS service_id, 'application' AS kind, sa.id, sa.uuid, sa.name, sa.image, sa.status FROM service_applications AS sa JOIN services AS s ON s.id = sa.service_id WHERE s.uuid = 'SERVICE_UUID' UNION ALL SELECT s.id AS service_id, 'database' AS kind, sd.id, sd.uuid, sd.name, sd.image, sd.status FROM service_databases AS sd JOIN services AS s ON s.id = sd.service_id WHERE s.uuid = 'SERVICE_UUID' ORDER BY name, kind, id;"
    ```

    The expected outcome is one row per Coolify card.
    Record the single repeated `service_id` as `TARGET_SERVICE_ID`.
    Match card URL UUIDs from steps 4 and 6 to the `uuid` column,
    then match the live container's `CoolifySubType` and `CoolifySubId` from step 18 to `kind` and `id`.

29. Read persistent-storage metadata for the PostgreSQL mount:

    ```bash
    docker exec COOLIFY_DB_CONTAINER_ID \
      psql --username=coolify --dbname=coolify --csv \
      --command="SELECT id, name, mount_path, resource_type, resource_id FROM local_persistent_volumes WHERE mount_path = '/var/lib/postgresql/data' ORDER BY resource_type, resource_id, id;"
    ```

    The expected outcome includes metadata for the volume reported by steps 16 or 18.
    Use `resource_type` and `resource_id` to determine which Coolify record owns that metadata.

30. Return to each **Vectordb** browser tab and leave it open without pressing **Delete**,
    **Convert to Application**,
    or **Convert to Database**.
    The expected outcome is unchanged resource configuration and preserved storage.

## What to check

Status:
TODO

Record and compare these exact values:

- Rendered Coolify version from step 1.
  A version older than `v4.0.0-beta.466` is susceptible to the upstream empty-query false-exit bug.
  Version alone does not prove that this incident was caused by that bug.
- Unfiltered destination inventories from steps 10 and 11.
  They validate that later empty filtered results are meaningful.
- Coolify identity labels and Compose project from step 12.
  These prevent selecting another LibreChat stack that also has a `vectordb` service.
- Number of real matching `vectordb` rows from steps 13 and 14.
  Two cards with one Docker row prove that the cards do not represent two current containers.
- Detached volume identity and Compose project from steps 15 through 17 when no container exists.
  Do not mutate an ambiguous candidate.
- `State.Status`,
  `State.ExitCode`,
  `State.OOMKilled`,
  `State.Error`,
  `State.StartedAt`,
  and `State.FinishedAt` from step 18.
- Exact final PostgreSQL line from step 21.
  Healthy startup includes `database system is ready to accept connections`.
- `CoolifySubType` and `CoolifySubId` from step 18,
  matched to `kind` and `id` from step 28.
  This match identifies the record represented by the live container.
- Card UUIDs and descriptions from steps 4 and 6,
  matched to the control-plane rows from step 28.
  Do not choose a card to delete from its description alone.
- Named-volume identities from steps 16,
  18,
  and 20,
  matched to storage metadata from step 29.
  Preserve all MongoDB,
  Meilisearch,
  and PostgreSQL volume identities across later recovery.
- LibreChat health configuration and both endpoint probes from steps 23 through 25.
  These test the installed artifact rather than inferring from current upstream source.

A false-exit attribution requires a live or logged Coolify `Exited` transition
while destination Docker still reports the same container as running.
Static version inspection proves susceptibility only.

Report only that the positive controls in steps 10 and 11 were nonempty.
Share the unique target rows from steps 12,
13,
18,
19,
22,
and 23 through 25.
Share only the last PostgreSQL error or ready line from step 21 unless more context is requested.
For self-hosted Coolify,
share the target Service rows from step 28 and matching storage rows from step 29.
Redact unrelated container and volume names,
private project and environment names,
domains,
IP addresses,
credentials,
tokens,
and secret-bearing health-check arguments.

## Restore

Status:
TODO

No Docker or Coolify state was changed.
Close the two **Vectordb** browser tabs without saving.
Run `exit` once for each SSH session:

```bash
exit
```

The expected outcome is the workstation shell prompt and the same containers,
Coolify records,
and named volumes that existed before inspection.
