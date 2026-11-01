# Recursive directory enumeration follows directory symlinks

## Measured behavior and deciding source

On Node `26.12.0`,
`fs.promises.readdir(path, {recursive: true})` returns children reached through directory symlinks.
Adding `withFileTypes: true` still returns those children,
while correctly identifying the link itself as a symbolic link.
Inspecting the returned entries does not prevent the enumeration from entering that link first.

A disposable probe contains an owned child directory and a sibling directory outside the scan root.
The scan root links to that sibling.
The positive control sees the owned child's file;
both recursive variants also return the external file through the link.
The probe runs without network under Podman,
with 2 GiB memory,
two CPUs and a 256 MiB V8 heap limit.
The receipt is
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/node-readdir-probe-receipt.json`.

Node source revision `26e72d85a611e222e3f06025de8a3c496923bacd`
identifies itself as release `26.12.0` in `src/node_version.h`.
`RecursiveReadDir::Scan` in `src/node_file.cc:2564` calls `uv_fs_stat` for a symbolic link,
records whether its target is a directory and queues that relative directory for scanning.
The matching `fsPromises.readdir` section in `doc/api/fs.md`
describes recursive enumeration and the returned entry type,
without a guarantee of ignoring directory symlinks.
The deciding source and docs are retained privately beside the probe receipt.

## Owned build defect and consumer regression

`package/pi-plugin/auto-mode/native/build-reuse.ts` claimed its recursive enumeration did not follow dependency links.
The real build contradicts that claim.
The committed `native/build-reuse.platform.test.ts` adds an opaque directory symlink to a copied contract asset.
Its disposable target has mode `000`,
and a direct `readdir` control first proves `EACCES`.
The actual `native/build.ts --owned` command then fails in `reuseNativeBuildFiles`
with `EACCES: permission denied, scandir` through that external symlink.
The retained stderr names the generated release path and `native/build-reuse.ts:136`.

The replacement walks nonrecursive directory entries,
extends its queue only for actual directories and collects only regular files.
It leaves observed symbolic links untouched.
The real build passes that case,
retains the opaque symlink and still shares eligible root and nested files by inode on the same mount.
Its independent cross-mount control still proves `EXDEV` and preserves intact files during publication.
Fresh package types and the actual three-build regression pass after the sequential-queue restructuring.
Scoped lint checks both maintained files with zero warnings and errors across 486 rules.
Restoring recursive enumeration in a separate private build fails the third actual command with `EACCES`.
Skipping real child directories fails the nested-file inode assertion after the second actual command.
Restoring the original implementation passes all three commands,
retaining six independent raw stdout/stderr files.
The mutation receipt is `reuse-traversal-built-mutants.json` beside the probe receipt.

## Diagnostic orchestration failure

The supervisor's first mutation-log collector recursively enumerates an output mount
containing a `node_modules` link to the shared dependency graph.
The actual child build has already terminated with its expected `EXDEV` failure.
Its unbounded host-side collector then receives `SIGKILL`.
In that window,
the kernel reports a memory-cgroup OOM kill of a Node process with `anon-rss:3895500kB`.
The original helper does not record its PID,
so that kernel PID is not independently matched to the helper.
The unsafe traversal itself is independently reproduced by the bounded probe.

Recovery inspects the completed child receipt and its raw output before running another build.
The private mutated source is restored from its frozen original.
The collector now visits only known fixture-package and per-build log directories,
using nonrecursive entries and rejecting symbolic links before descent.
The completed first mutation is recovered without executing it again.
The remaining mutation and restored consumer both complete,
retaining independent stdout and stderr for both builds.

Bounding a subprocess does not bound its parent's traversal or log collection.
The user keeps root `RXI` unchanged and places this concrete lesson in package documentation.
This host-side incident is separate from the earlier MiMo process's V8 heap-limit failure;
their causes are not established as the same.

## Upstream filing decision

No Node issue is filed from this evidence.
The observed behavior matches the deciding implementation,
and the owned build asserted an enumeration boundary the API does not guarantee.
The repair belongs to the build's traversal and the diagnostic collector.
No Node or Linux defect is attributed.
