# PNPM 12.3.4 offline frozen-lock verification refuses missing metadata mirrors

## Symptom

During the translation-repair merge,
PNPM `12.3.4` runs `install --lockfile-only --offline --frozen-lockfile`
through the repository's `prepare:pnpm:install` task.
Lockfile regeneration has succeeded,
but a fresh verifier fails with `ERR_PNPM_META_FETCH_FAIL` wrapping:

```text
# main-merge-lock-generation-H8Jd5i/task.out
ERR_PNPM_NO_OFFLINE_META: Failed to resolve showify in package mirror
.../v11/metadata/registry.npmjs.org/showify.jsonl
```

Providing only that abbreviated record moves the missing input to `@anthropic-ai/sdk`.
Providing the abbreviated package inventory reaches a separate missing input:

```text
# main-merge-lock-generation-d4wXdS/task.out
ERR_PNPM_NO_OFFLINE_META: Failed to resolve showify in package mirror
.../v11/metadata-full/registry.npmjs.org/showify.jsonl
```

The existing host mirror is `metadata-full-filtered`.
A package store or successful regeneration does not establish that the fresh verifier's mirror paths exist.

## Source trace

The inspected release is tag `v12.3.4`,
commit `666c35e95c17f1a36414adc28bc25a73a8f1f67f`,
in the read-only `pnpm-12.3.4-merge-20260914` checkout of `pnpm/pnpm`.

`pnpm/crates/resolving-npm-resolver/src/create_npm_resolution_verifier.rs:1056`
requests the abbreviated mirror when its shared in-memory cache has no usable projection:

```rust
// create_npm_resolution_verifier.rs
full_metadata: false,
filter_metadata: false,
offline: self.offline,
```

Its full trust-metadata fallback at `create_npm_resolution_verifier.rs:1239`
requests a different namespace:

```rust
// create_npm_resolution_verifier.rs
full_metadata: true,
filter_metadata: false,
offline: self.offline,
```

`pnpm/crates/resolving-npm-resolver/src/fetch_full_metadata_cached.rs:82`
selects the directory from those flags.
Its offline branch at line 118 does not fetch a missing body:

```rust
// fetch_full_metadata_cached.rs
if opts.offline {
    if let Some(meta) = load_meta_async(mirror_path.as_deref()).await {
        return Ok(meta);
    }
    return Err(FetchMetadataError::NoOfflineMeta {
        pkg_name: pkg_name.to_string(),
        pkg_mirror: mirror_path.unwrap_or_else(PathBuf::new),
    });
}
```

The shared in-memory trust lookup at `create_npm_resolution_verifier.rs:1215`
accepts both full and filtered-full entries:

```rust
// create_npm_resolution_verifier.rs
cache
    .get(&format!("{key}:full"))
    .or_else(|| cache.get(&format!("{key}:full:filtered")))
```

Its comment states that `clear_meta` retains `time`,
per-version `_npmUser` and `dist`,
the fields consumed by the trust check.
This is evidence about the trust projection,
not documentation endorsing an on-disk directory alias.
The custom mount mapping is verified separately by real offline invocation.

## Verification and retained workaround

The native CLI help confirms that `--lockfile-only` does not download packages or write `node_modules`,
`--ignore-scripts` disables lifecycle scripts,
and `--network-concurrency` bounds concurrent network requests.
The retained help files are `merge-pnpm-install-help-20260914.txt`
and `merge-pnpm-cache-help-20260914.txt` in private agent scratch.

Working catalog:

- A private package acquires abbreviated metadata with `--lockfile-only --ignore-scripts --no-runtime`.
- The acquisition container has 2 GiB RAM,
  2 GiB swap,
  two CPUs and 512 PIDs,
  with network concurrency set to two.
  Neither the repository nor the real home is mounted.
- The verifier mounts the abbreviated mirror read-only
  and exposes the original filtered-full bytes at its full-metadata lookup path.
- `main-merge-lock-generation-JLN4yl` then verifies all 773 lockfile entries,
  exits zero and leaves `pnpm-lock.yaml` byte-identical.
  It has no network and no inherited `lockfile-verified.jsonl` verdict.
- `main-merge-metadata-proof-oVKGP8` retains 1492 exact-byte metadata files,
  totaling `215155142` bytes.
  `main-merge-lock-generation-IWD4cP` repeats the successful no-network check from that owned snapshot.
  The metadata hashes and PNPM executable hash match before and after verification.

The retained invocation is reproducible through:

```sh
# Private merge verification harness; it invokes the repository's Mise task.
node /var/home/user/temp/agent/run-main-merge-frozen-lock-check-r4-20260914.mts
```

The harness creates fresh private output directories.
It does not disable trust or release-age policy,
change the repository lockfile,
install dependencies into the worktree,
or modify PNPM's source or installed executable.

Tradeoffs:
the directory mapping depends on the inspected release's metadata projection and loader.
Re-audit it for another release.
Cached metadata is not current-registry evidence,
and a read-only container mount alone is not an immutable host snapshot.
The owned snapshot and rerun establish the retained byte boundary,
not authentication or a current-registry claim.
The local harness workaround has not been reviewed.

## What does not work

- Providing only the package store does not provide the required metadata mirrors.
- An abbreviated mirror does not supply the full trust projection.
- The existing filtered-full mirror is not automatically selected by this cold verifier path.
- `pnpm view showify --json` succeeds in the private probe but leaves no mirror file there.
- Disabling supply-chain checks would answer a different question and is not used.

## Upstream filing decision

1.  Upstream fault is not established:
    the custom offline profile lacks the inputs requested by the inspected implementation.
2.  The measured remedy is cache preparation and mount configuration,
    not a required upstream code change.
3.  Offline verification is supported;
    [issue 11801][issue] and [merged PR 13671][pull] explicitly require a cache-miss diagnostic.
4.  No upstream contribution is proposed.
    The existing merged PR discloses AI assistance and includes maintainer approval;
    that is not permission to misrepresent a local cache omission as a new defect.
5.  The existing issue is closed by the cited PR.
    This observation does not show a recurrence of its network-access bug.
6.  No upstream patch is prototyped because the first constraint is not established.
    The consumer-side configuration is exercised instead.

The repository's `.out-of-scope/` inventory has no PNPM-specific exemption.
Nothing new is filed:
the existing issue already explains offline cache misses,
and no additive upstream defect is established here.

[issue]: https://github.com/pnpm/pnpm/issues/11801
[pull]: https://github.com/pnpm/pnpm/pull/13671
