# Node recursive mkdir can report missing paths on a read-only filesystem

## Observed behavior

Node `26.12.0` on the Linux verification host reports different errors for the same disposable read-only mount:

- `writeFile` creating a missing file rejects with `EROFS`.
- Non-recursive `mkdir` creating a missing directory rejects with `EROFS`.
- `mkdir` with `{ recursive: true }` creating a missing directory rejects with `ENOENT`.

The independent file and non-recursive directory controls establish that the mount is read-only.
The recursive error alone does not establish that a parent directory is missing.
Logger `0.4.2` exposed this behavior while trying to create a default project log directory.
Do not diagnose a writable installation from the absence of an `EROFS` diagnostic.

## Deciding source

The installed Node release's `src/node_file.cc`,
function `MKDirpAsync`,
handles `UV_EACCES`,
`UV_ENOSPC`,
`UV_ENOTDIR` and `UV_EPERM` directly.
`UV_EROFS` takes its default branch,
which checks the path using `uv_fs_stat`.
That callback passes its own result to the continuation.
For a directory that does not exist,
the stat result is `ENOENT`,
replacing the original mkdir error.
This explains the observed diagnostic without treating the kernel or mount as defective.

Source was fetched from the [Node v26.12.0 release],
with the deciding branch at lines 2043 to 2100.
The retained local source is `node-v26.12.0-node-file.cc` in the verification directory.

## Application behavior and verification

The logger attempts its default project file and falls back on a failed probe,
preserving the actual caught diagnostic in the notice.
It does not limit fallback to an `EACCES` or `EROFS` allow-list.
Explicit argument or environment destinations remain strict and report failure without being redirected.

The real mounted consumer uses a private home,
private state directory,
read-only project and two-CPU,
2 GiB container.
It proves both error controls,
default-logger persistence and strict explicit-destination failure through built artifacts.
The retained receipt is `logger-default-fallback-readonly-mkdir-controls.log` under
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018`.
The package contract and regression evidence live in
[`package/logger/core/doc/file-destinations.md`](../../package/logger/core/doc/file-destinations.md).

[Node v26.12.0 release]: https://github.com/nodejs/node/blob/v26.12.0/src/node_file.cc
