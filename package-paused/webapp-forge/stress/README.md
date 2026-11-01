# webapp-forge-stress

Stress harness for the webapp-forge stack.

Each scenario seeds synthetic data, drives an event burst, and asserts
the rebuild-on-write **invalidation invariants** plus p50/p99 latency
and throughput targets.

## Scenarios

| Scenario         | Setup                | Driver                                   | Invariants                       
 |
| ---------------- | -------------------- | ---------------------------------------- |
 ----------------------------------------------------------------------------------------- |
| `hot-repo`       | one repo, 10K issues | mitata-paced bursts of `comment.created` | rebuild p99 < 5s;
 zero stale fragments after drain                                        |
| `bursty-comment` | one issue, 60s burst | sustained 167 evts/sec
 on the same issue | detail-fragment rebuild count <= 60 (1/sec debounce); final fragment matches ground truth |
| `wide-service`   | broad service graph  | fanout across many repos and services    | list
 and service fragments invalidate without stale reads                                 |
| `force-push`     | repo with branches   | force-push event sequence                | branch
 and commit fragments converge after rewrite                                        |

Phase 3 adds `cross-cutting-rename` and `filter-fanout`.

## Run

```sh
mise run //packages/webapp-forge/stress:stress -- --scenario=hot-repo
mise run //packages/webapp-forge/stress:stress -- --scenario=wide-service
mise run //packages/webapp-forge/stress:stress -- --scenario=force-push
mise run //packages/webapp-forge/stress:stress -- --scenario=all
mise run //packages/webapp-forge/stress:stress:hot-repo
mise run //packages/webapp-forge/stress:stress:bursty-comment
```

A failing invariant exits non-zero so CI gates on it.

## Garage S3 backend (optional)

By default the harness runs against the in-memory storage adapter
(fastest; sufficient for invariant checks). Production-realistic
latency numbers require an S3-compatible endpoint; we use Garage in
podman.

```sh
mise run prepare:garage
podman exec -it monochromatic-garage-running cat /secrets/garage-credentials.env
# export the printed AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY,
# GARAGE_ENDPOINT, GARAGE_BUCKET, AWS_REGION
mise run forge:stress -- --scenario=hot-repo --storage=s3
```

The S3 adapter (`packages/webapp-forge/server/src/storage/adapter-s3.ts`) speaks
path-style requests against any S3-compatible endpoint, so `GARAGE_ENDPOINT`
can also point at R2 in production.

Stop the container:

```sh
podman stop monochromatic-garage-running
```
