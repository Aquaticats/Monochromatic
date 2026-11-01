# Git 2.55 archive filtering invalidates raw-blob benchmark corpus attribution

## Symptom

The frozen source tree contained 13,860 blob entries totaling 366,655,696 bytes.
The executed Git tar archive was 602,992,640 bytes.
The initial research statement that Git archive preserved raw object bytes,
including LFS pointer text,
was wrong.
The benchmark actually consumed an archive with configured LFS materializations.

This was an evidence-attribution failure,
not a benchmark runtime failure.
The regex benchmark completed both unchanged repetitions;
the later audit reconstructed and verified the exact executed archive.

## Root cause

Git `v2.55.0`,
commit `e9019fcafe0040228b8631c30f97ae1adb61bcdc`,
reads regular blobs in `archive.c:101`
and explicitly calls working-tree conversion at `archive.c:107`:

```c
/* archive.c:101, Git v2.55.0, abridged. */
buffer = odb_read_object(the_repository->objects, oid, type, sizep);
if (buffer && S_ISREG(mode)) {
    struct strbuf buf = STRBUF_INIT;
    size_t size = 0;

    strbuf_attach(&buf, buffer, *sizep, *sizep + 1);
    convert_to_working_tree(args->repo->index, path, buf.buf, buf.len, &buf, &meta);
    if (commit)
        format_subst(commit, buf.buf, buf.len, &buf, args->pretty_ctx);
    buffer = strbuf_detach(&buf, &size);
    *sizep = size;
}
```

The working-tree conversion runs independently of the `format_subst` conditional.
Calling archive with a frozen commit therefore does not itself establish raw-blob byte identity.
The frozen root `.gitattributes` declares LFS filters for image assets.
The audit found 1,464 converted members.
Each converted member's source blob was an LFS version-one pointer,
and its archive bytes matched that pointer's SHA-256 object ID and size.
The other members matched their original Git blob IDs.

## Verification

The original benchmark was `hzfbgz4xq7`,
research commit `111910f0352b0e65e8f3f0befa61282915b90e2c`.
Application source was `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
The same-image audit was `8ksdxnqq25`,
research commit `7861d87ac6a93d33b0636b4aeea34e5b2929cad6`.
Its actual Git executable reported `git version 2.55.0`.
Both used prepared image digest
`sha256:1c2b469a9d7fae8bc364e6392f771c28565647a04c40847f345bc140dfce4e88`.

The regenerated archive had the exact original executed SHA-256:
`47698c6be07012ab111e7586c1fd23d8b1f610ca5d3f570e53147da9a30ad368`.
The audit examined all 13,860 regular-file or symlink members,
found no missing members,
and verified every conversion against its frozen LFS pointer.
Full member inventories and retained logs are in the compressed raw receipts linked from
[`depot-rust-coverage-2026-10-18.evidence.json`](../audit/depot-rust-coverage-2026-10-18.evidence.json).

The research branch owns this runnable task:

```sh
# Research branch package/monorepo-task/core-exec/mise.toml, not the main checkout.
mise run //package/monorepo-task/core-exec:research:rust:corpus-audit
```

Its positive catalog comprises direct Git-blob matches and content-addressed LFS materializations.
Its failing catalog contains 1,464 direct raw-blob equality comparisons:
those archive members differ from their frozen pointer blobs.
Each is admitted only after the independent LFS size and SHA-256 check.
The audit also guards unknown conversions,
missing members and archive-digest mismatch;
those additional conditions were not fault-injected in this run.

## Verified workarounds

The consumer-side correction records the archive digest before use,
then verifies its full member inventory against the frozen tree:

The source-pinned task `research:rust:corpus-audit` is the runnable implementation.
The archive-writing correction in the executed benchmark harness is:

```js
// package/monorepo-task/core-exec/bin/rust-benchmarks.mjs, imports and context omitted.
run('archive-baked-frozen-public-source', 'git',
  ['archive', '--format=tar', '--output', archive, frozen]);
const archiveHasher = createHash('sha256');
for await (const chunk of createReadStream(archive)) archiveHasher.update(chunk);
const archiveBytes = statSync(archive).size;
const archiveSha256 = archiveHasher.digest('hex');
```

The actual executable implementation uses Python's standard tar reader,
Git blob SHA-1 reconstruction and SHA-256 LFS object validation.
It retains all member hashes.
This proves exact input identity for the executed export,
including LFS media;
it does not claim a pointer-only corpus or an export independent of filter configuration.
No corpus bytes,
source files or seeds were changed.
The benchmark was not rerun for this metadata audit.

The archive is written through Git's native `--output` option and hashed as a stream.
The selected-file positive control passed before the full remote invocation.
This avoids the earlier planned 256 MiB synchronous stdout buffer,
which was smaller than the measured raw tree before tar overhead or conversion.
The tradeoff is explicit owned disk space for the archive and extracted baked corpus.

## What does not work

A commit hash alone does not identify post-filter archive bytes.
Counting raw Git blob sizes does not predict converted tar size.
Matching one selected file does not verify the complete corpus.
Removing LFS or changing filter settings would create a different corpus;
that was not used to retroactively validate these benchmark timings.
The initial raw-byte statement is retained as a corrected historical claim.

## Upstream filing decision

The `.out-of-scope/` inventory contains no Git-archive exemption.
Read-only search `repo:git-lfs/git-lfs "archive smudge"` returned no matches.
That narrow result does not establish an absence of related discussions.
Git's `.github/CONTRIBUTING.md` directs reports and contributions to its mailing list,
rather than GitHub issues.

1.  No upstream fault is established.
    The source explicitly performs working-tree conversion;
    the incorrect raw-byte claim belonged to this consumer report.
2.  The consumer can verify and label the export correctly,
    as the completed audit demonstrates.
3.  Git supports commit archives and configured conversion;
    no promise of pointer-only output is asserted here.
4.  Git's contribution entry point was read.
    A filing-specific policy audit was not continued because no upstream fix is proposed.
5.  No maintainer position is inferred from the empty Git LFS tracker search.
6.  The executed fix is the consumer identity audit and corrected attribution.
    No upstream source prototype is warranted by this incident.

Nothing to add upstream.
No issue,
comment or mailing-list message was sent.
