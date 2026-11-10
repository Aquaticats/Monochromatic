# Meilisearch 1.53.1: a Coolify LibreChat restart rejects a 1.12.3 database

A Coolify LibreChat Service can stop after a container recreation when its
Meilisearch image uses the mutable `latest` tag and its persistent volume was
created by Meilisearch 1.12.3.

## Symptom

The Meilisearch component repeatedly restarts with:

```text
Error: Your database version (1.12.3) is incompatible with your current engine version (1.53.1).
To migrate data between Meilisearch versions, please follow our guide on
https://www.meilisearch.com/docs/learn/update_and_migration/updating.
Alternatively, you can set the `--upgrade-db` flag (or the `MEILI_UPGRADE_DB`
environment variable) to upgrade the database on startup.
```

LibreChat remains stopped because its Compose definition requires Meilisearch
to pass its health check before LibreChat starts.

The following messages are separate from this failure:

- MongoDB's `Connection not authenticating` entries are informational records
  from the unauthenticated `mongosh` health check in this Compose definition.
- The RAG API `SyntaxWarning: invalid escape sequence` does not stop Uvicorn;
  its log reaches `Application startup complete`.
- PostgreSQL's transaction warnings do not show a fatal database failure;
  the supplied log reaches `database system is ready to accept connections`.
  A currently exited vector database still requires its own exit-code and
  readiness check.

## Root cause

The persistent `meilisearch-data` volume retained a 1.12.3 database while a
container recreation resolved `getmeili/meilisearch:latest` to engine 1.53.1.
A plain Docker restart does not pull an image,
 but a Coolify redeploy or image
pull can recreate a container from a newer image behind a mutable tag.

Meilisearch reads the database version and rejects any mismatch unless the
upgrade option is enabled.
 In Meilisearch tag `v1.53.1`,
 commit
`577f7af28942b71782eab1e59f44ad8296ce0a92`,
`crates/meilisearch/src/lib.rs:476-489` contains the gate:

```rust
if db_major != bin_major || db_minor != bin_minor || db_patch != bin_patch {
    if opt.upgrade_db {
        update_version_file_for_dumpless_upgrade(
            opt,
            index_scheduler_opt,
            (db_major, db_minor, db_patch),
            (bin_major, bin_minor, bin_patch),
        )?;
    } else {
        return Err(VersionFileError::VersionMismatch {
```

The emitted diagnostic and its upgrade instruction come from
`crates/meilisearch-types/src/versioning.rs:81-87`:

```rust
#[error(
    "Your database version ({major}.{minor}.{patch}) is incompatible with your current engine version ({}).\n\
    To migrate data between Meilisearch versions, please follow our guide on https://www.meilisearch.com/docs/learn/update_and_migration/updating.\n\
    Alternatively, you can set the `--upgrade-db` flag (or the `MEILI_UPGRADE_DB` environment variable) to upgrade the database on startup.",
```

The environment variable is a Boolean command-line option.
`crates/meilisearch/src/option.rs:444-453` binds it directly:

```rust
/// When set,
/// Meilisearch will upgrade its database on startup if it was created by a
/// previous version.
#[clap(long, env = MEILI_UPGRADE_DB, default_value_t)]
#[serde(default)]
pub upgrade_db: bool,
```

A direct upgrade from this database version is supported.
 The lower bound in
`crates/index-scheduler/src/upgrade/mod.rs:64-70` rejects versions before
1.12.0,
 not 1.12.3:

```rust
if initial_version < (1, 12, 0) {
    bail!(
        "Database version {initial_major}.{initial_minor}.{initial_patch} is too old to be upgraded via `--upgrade-db`. Please generate a dump using the v{initial_major}.{initial_minor}.{initial_patch} and import it in the v{target_major}.{target_minor}.{target_patch}",
    );
}
```

The same module runs each required migration and then registers an
`UpgradeDatabase` task (`crates/index-scheduler/src/upgrade/mod.rs:73-114`).
Meilisearch's test suite also carries a 1.12.0 fixture through the current
upgrade path at
`crates/meilisearch/tests/upgrade/v1_12/v1_12_0.rs:15-29`.

## Verification

Verified on 2026-09-03 with the published container images:

- `getmeili/meilisearch:v1.12.3`,
   image configuration
  `9547aac4aabca4f08693680112e794ec8bad21b7c9020b87f206c86f421dbb13`
- `getmeili/meilisearch:v1.53.1`,
   image configuration
  `357e14dd8105e8ce58cbd8dee32a3cbf56409110dea7abf92ca01d725e1e83f9`
- Meilisearch source tag `v1.53.1`,
   commit
  `577f7af28942b71782eab1e59f44ad8296ce0a92`

The disposable harness created a 1.12.3 index containing one document,
stopped that engine,
 and mounted the same directory into 1.53.1:

```bash
mkdir --parents /tmp/meili-upgrade-test

podman run --rm \
  --memory=1g \
  --cpus=2 \
  --publish 127.0.0.1:17700:7700 \
  --volume /tmp/meili-upgrade-test:/meili_data:Z \
  --env MEILI_ENV=production \
  --env MEILI_MASTER_KEY=agent-verification-key-1234567890 \
  getmeili/meilisearch:v1.12.3

podman run --rm \
  --memory=1g \
  --cpus=2 \
  --volume /tmp/meili-upgrade-test:/meili_data:Z \
  --env MEILI_ENV=production \
  --env MEILI_MASTER_KEY=agent-verification-key-1234567890 \
  getmeili/meilisearch:v1.53.1
```

### Configurations that work

- Engine 1.12.3 with database 1.12.3 starts and serves the indexed document.
- Engine 1.53.1 with database 1.12.3 and `MEILI_UPGRADE_DB=true` starts,
  reports a succeeded `upgradeDatabase` task from `v1.12.3` to `v1.53.1`,
  and serves the same document.
- Engine 1.53.1 starts again without the upgrade variable after the migration
  and still serves the document.
- Engine 1.35.1 with a fresh volume matches the version pinned by LibreChat's
  current `deploy-compose.yml`;
   LibreChat can rebuild its search indexes from
  MongoDB using its documented reset synchronization command.

### Configurations that fail

- Engine 1.53.1 with database 1.12.3 and no upgrade option exits with code 1
  and the quoted version-mismatch diagnostic.
- Engine 1.12.3 cannot open a volume after it has been upgraded to 1.53.1.
  Meilisearch rejects database downgrades,
   so rollback then requires restoring
  the pre-upgrade volume backup.

The disposable one-document migration verifies the specific format transition
and option spelling.
 It does not substitute for restoring and testing a backup
of a production volume or for exercising LibreChat's search interface.

## Verified workarounds

### Restore availability with 1.12.3

Pin the Meilisearch image to `getmeili/meilisearch:v1.12.3` and retain the
original volume.
 This is the least invasive recovery before any migration has
changed the volume.

Tradeoff:
 1.12.3 is not the version in LibreChat's current deployment Compose,
and Meilisearch only maintains its latest engine release.

### Align LibreChat with 1.35.1 and rebuild search

Pin `getmeili/meilisearch:v1.35.1` to a new named volume,
 retain the old volume,
and run LibreChat's documented `reset-meili-sync` command.
 This follows
LibreChat's current `deploy-compose.yml` pairing and treats Meilisearch as the
rebuildable conversation-search index while MongoDB remains authoritative.

Tradeoff:
 reindexing consumes storage,
 processor time,
 and OpenSearch downtime
in proportion to the stored conversation and message data.
 Any custom data
stored directly in Meilisearch would not be recreated by LibreChat and must be
migrated instead.

### Upgrade the existing volume to 1.53.1

After a cold,
 restore-tested backup,
 pin
`getmeili/meilisearch:v1.53.1`,
 set `MEILI_UPGRADE_DB=true`,
 and keep LibreChat
stopped until `GET /tasks?types=upgradeDatabase` reports `status: succeeded`.
Remove the variable after the first successful migration.

Tradeoff:
 Meilisearch documents that this upgrade is not atomic,
 downgrade is
not supported after it starts,
 and 1.53.1 is newer than LibreChat's published
1.35.1 Compose pairing.
 The disposable verification covered one index and one
document,
 not a production LibreChat dataset.

## What does not work

- Repeatedly restarting the same `latest` image cannot change the database
  compatibility check.
- Increasing the health-check retries only delays the dependency failure.
- Deleting or reinitializing MongoDB is unrelated and risks losing LibreChat's
  authoritative users,
   conversations,
   and messages.
- Deleting the vector database does not repair Meilisearch and risks losing RAG
  embeddings.
- Pinning 1.12.3 after an in-place 1.53.1 migration does not roll the database
  back.
   Restore the matching pre-upgrade backup instead.
- Treating a successful archive job as a verified backup is insufficient.
  Restore it into a disposable volume and start the matching Meilisearch image.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers this incident.
 Searches of open and closed
Meilisearch issues found issue
[meilisearch/meilisearch#5534](https://github.com/meilisearch/meilisearch/issues/5534),
where a mutable container image produced the same mismatch and enabling the
then-current upgrade variable resolved it.
 LibreChat discussion
[danny-avila/LibreChat#9812](https://github.com/danny-avila/LibreChat/discussions/9812)
points operators to LibreChat's reset synchronization procedure.

1.  **Is it really upstream's fault?**
     No. Meilisearch intentionally prevents a
    different engine version from opening a versioned database without an
    explicit migration request.
2.  **Can upstream fix it?**
     The supported upgrade path already exists through
    `MEILI_UPGRADE_DB` or a dump.
3.  **Are they supporting this use case?**
     Yes.
     Meilisearch documents both
    upgrade methods,
     and LibreChat documents rebuilding search from MongoDB.
4.  **Would the repo welcome our contribution?**
     Meilisearch has contribution
    guidance and issue templates,
     with no discovered prohibition on an external
    report.
     There is no defect to report here.
5.  **Will they likely fix it?**
     Not applicable.
     Removing the explicit migration
    gate would conflict with the documented database compatibility policy.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     No
    source fix is warranted.
     The verified fix is deployment configuration at
    the consumer boundary.

The behavior is documented and the existing issue already records the same
resolution.
 There is nothing additive to file or comment upstream.

## References

- [Meilisearch updating guide](https://www.meilisearch.com/docs/resources/migration/updating)
- [Meilisearch configuration reference](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference)
- [LibreChat Meilisearch reset synchronization](https://www.librechat.ai/docs/configuration/meilisearch#reset-synchronization)
- [LibreChat current deployment Compose](https://github.com/danny-avila/LibreChat/blob/main/deploy-compose.yml)
- [Coolify Docker Compose Service configuration](https://next.coolify.io/docs/services/configuration/docker-compose)
- [Coolify storage mount backups](https://next.coolify.io/docs/core/persistent-storage/storage-mounts/backups)
