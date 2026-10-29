# TODO: Investigate Edit tool false-positive staleness rejection

## Problem

The built-in Claude Code `Edit` tool rejected a write to `manager-defs.ts` with:

> File has been modified since read,
>  either by the user or by a linter.
>  Read it again before attempting to write it.

The file was **not** modified.
 Re-reading confirmed identical content.

## Reproduction timeline

1. `Read` on `package/file-enforcer/core/src/package/manager-defs.ts` (harness records staleness marker)
2. `Write` creates a **sibling** file `manager-defs.unit.test.ts` in the same directory
3. `dprint fmt` runs:
    scans workspace,
    reformats the test file (1 file changed),
    `stat()`s other `.ts` files
4. `bun test` runs:
    imports `manager-defs.ts` at runtime (read-only)
5. `mise run lint` runs `task-tsgo --build`,
    writes `tsconfig.tsbuildinfo` in the dist tree
6. `Edit` on `manager-defs.ts`:
    **rejected as stale**
7. `Read` on `manager-defs.ts`:
    content is byte-identical to step 1

No PostToolUse hooks are configured that modify `.ts` files
(confirmed by checking `.claude/settings.json` and user-level settings).

## Hypothesis

The Edit tool's staleness check uses filesystem metadata (mtime or inode change) rather than content hash.
Between the initial Read and the Edit,
 one of the intervening tools (dprint scanning,
 bun importing,
 tsgo building)
may have updated the file's `atime`,
 `ctime`,
 or caused the filesystem to report a different `mtime`
without changing content.
 On copy-on-write or overlay filesystems (e.g. ostree-based Fedora),
metadata updates can be more aggressive than on ext4.

## Desired outcome

Replace the built-in `Edit` tool with an MCP-based edit server
that uses **content hashing** (e.g. SHA-256 of the file at read time vs. at edit time)
instead of mtime/inode for staleness detection.
 This eliminates false positives from:

- Formatters scanning but not changing a file
- Build tools writing artifacts in sibling directories
- Runtime imports (`bun`,
   `node`) touching `atime`
- Filesystem metadata quirks on immutable distros

## Investigation steps for the next session

1. Check if Claude Code's Edit tool source is accessible (it ships with the CLI:
    look in the npm package or GitHub repo)
2. Identify the exact staleness mechanism (mtime,
    inode,
    content hash,
    ETag,
    or combination)
3. Prototype an MCP server exposing `read_file` and `edit_file` tools with content-hash-based staleness
4. Test whether Claude Code allows MCP tools to shadow built-in tool names,
    or if the MCP tools need distinct names
5. If shadowing is not possible,
    explore the `--disable-tool` CLI flag to suppress the built-in Edit and alias the MCP replacement

## Related context

- Workspace:
   `/var/home/user/Monochromatic`
- OS:
   Fedora Atomic (ostree-based,
   `/home` is a symlink to `/var/home`)
- File affected:
   `package/file-enforcer/core/src/package/manager-defs.ts`
- Claude Code version:
   check `claude --version` at investigation time
