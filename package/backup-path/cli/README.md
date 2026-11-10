# @monochromatic-dev/backup-path-cli

Copies a file or directory into `./bak/<timestamp>/`,
 preserving file
timestamps.
 Useful before destructive edits to a path you do not want
to lose.

## What it does

1. Takes one positional argument:
    the path to back up.
2. Logs `Backing up <path>` to stdout.
3. Computes a timestamp from `new Date().toISOString()` with colons stripped
   (e.g. `2026-05-14T061711.030Z`).
4. Creates `./bak/<timestamp>/` relative to the current working directory.
5. Recursively copies the source into `./bak/<timestamp>/<basename>`,
   preserving file mtimes and atimes and rejecting a copy whose
   destination already exists.

## Usage

Run directly against the source:

```sh
node package/backup-path/cli/src/index.ts ./some/file-or-dir
```

Or against the built output once the package is installed as a
workspace dependency:

```sh
node node_modules/@monochromatic-dev/backup-path-cli/dist/final/node/index.js ./some/file-or-dir
```

There is no `mise run` task wrapper;
 invoke `node` directly.

## Arguments

- positional `<path>`:
   file or directory to back up.
   Required.
- `--help`:
   print usage and exit.

No environment variables are read.

## Quirks

- `./bak/` is **not** auto-created.
   The internal `mkdir('bak/<timestamp>')`
  call runs without `recursive: true`,
   so a missing `./bak/` parent
  fails with `ENOENT: no such file or directory, mkdir 'bak/...'`.
  Run `mkdir bak` once in the working directory on first use.
- An existing destination is rejected,
  not replaced.
  The copy runs `fs.cp` with `force: false` and `errorOnExist: true`,
  which raises `ERR_FS_CP_EEXIST` and leaves the earlier copy as it was.
  Both options are needed:
  `force` defaults to `true`,
  and on its own it leaves `errorOnExist` inert,
  so a collision would replace the earlier copy in silence.
- `./bak/<timestamp>/` is created with a non-recursive `mkdir`,
  so an invocation whose timestamped folder already exists fails there
  first,
  with `EEXIST` from `mkdir`.
  The millisecond-resolution timestamp makes that happen only for a
  second invocation inside the same millisecond.
- The copy uses Node's `fs.cp` with `preserveTimestamps: true`.
   File
  permissions and ownership follow the platform default of that API
  and are not explicitly preserved.
