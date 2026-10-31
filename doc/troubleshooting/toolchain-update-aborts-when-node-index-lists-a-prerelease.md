# `toolchain:update` aborts when the nodejs.org index lists a pre-release first

## Symptom

Observed 2026-10-29.
`mise run toolchain:update` fails on the `node` tool and writes nothing,
which blocks every other tool update in the same run:

```text
node failed: SHASUMS256.txt has no row for node-v27.0.0-alpha.0.0.0-linux-arm64-musl.tar.gz
```

## Cause

`https://nodejs.org/dist/index.json` lists releases newest first and includes pre-releases.
On 2026-10-29 the first entry was `v27.0.0-alpha.0.0.0`,
followed by the stable `v26.13.0` (LTS Lithium), `v26.12.1`, and so on.
A pre-release has a `SHASUMS256.txt` without rows for the recorded artifacts,
so the row lookup in `resolveNode` fails.
The resolver took `index[0]` as the newest release without checking that it was stable.

## Fix

`resolveNode` in `package/file-enforcer/core/src/toolchain/update.resolvers.node.ts`
now picks the first index entry whose `version` is exactly `v<major>.<minor>.<patch>`,
so entries with a pre-release suffix are skipped.
An index with no such entry still fails with `nodejs.org reports no release`.
The unit tests in `package/file-enforcer/core/src/toolchain/update.unit.test.ts` replay an index with a leading
pre-release, a pre-release-only index, an empty index, and a stable-first index.

## Workaround before the fix

None short of editing the recorded node rows by hand.
