# Hard links across mounts of the same filesystem

## Owned build failure

Auto-mode's immutable release optimization compares file metadata and complete content hashes,
then replaces a verified new file with a hard link to the previous release.
A private bounded build fails with Node's filesystem diagnostic:

```text
EXDEV: cross-device link not permitted, link
```

Both files have equal device identifiers.
The previous release is visible through a different bind mount from the new output.
This is an owned build assumption exposed by supported filesystem behavior.
No Node or Linux defect is established.

## Deciding contract and reproduction

The current [Linux `link(2)` manual](https://man7.org/linux/man-pages/man2/link.2.html)
describes `EXDEV` for paths on different mounted filesystems and explicitly states:

> Linux permits a filesystem to be mounted at multiple points,
> but `link()` does not work across different mounts,
> even if the same filesystem is mounted on both.

The probe runs on kernel `7.2.9-ogc3.1.fc44.x86_64` and Node `26.12.0`,
inside Podman with 2 GiB memory,
two CPUs and no network.
Separate writable bind mounts contain disposable old and new files on the same host filesystem.
The real `link` positive control returns `EXDEV` while `stat().dev` agrees.
Frozen `532c20e618` then fails the maintained helper regression on exactly that error.

`package/pi-plugin/auto-mode/native/build-reuse.ts` now catches `EXDEV` from the link operation,
retains the already verified new file and reports no shared bytes for that entry.
Its comparison and rename errors,
and other link errors,
continue to propagate.
The existing metadata device check remains a useful exclusion,
but cannot prove link availability.

## Consumer verification and tradeoff

`package/pi-plugin/auto-mode/native/build-reuse.platform.test.ts` invokes the actual `native/build.ts` command
in a disposable installation.
Its first build verifies CLI loading and publication despite the refused cross-mount links,
reports zero reused files,
retains complete old and new license bytes,
and leaves no temporary shared links.
The next build compares releases on one mount and shares the license inode.
Complete child stdout and stderr are retained beside the installation.
The actual-command test passes at the frozen checkpoint containing `57c1ae4c03`.

The task is `mise run //package/pi-plugin/auto-mode:test:native:build-reuse:mount`,
with the disposable previous-file and output-mount paths as arguments.
The private driver and logs are retained under
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018`.
The fallback trades deduplication for correctness on unsupported mount boundaries.
It does not copy over the prior release or modify a running consumer's files.

## Upstream filing decision

No upstream issue is filed.
The measured syscall result matches the documented Linux contract,
and Node preserves its `EXDEV` code.
The defect belongs to the repository's optional optimization and is repaired at that boundary.
The user-visible build failure,
same-mount success control and consumer-level verification are retained here.
