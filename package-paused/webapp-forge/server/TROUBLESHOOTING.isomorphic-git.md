# TROUBLESHOOTING: isomorphic-git server-side smart-HTTP

This file documents the capability gap between what `isomorphic-git@^1.27` ships
and what a server needs to terminate `git clone` and `git push` over the smart-HTTP
protocol. It is the result of the Phase 2 spike flagged in the plan at
`~/.claude/plans/let-s-put-our-effort-valiant-hopper.md` under "Open design choices".

## Context

The webapp-forge server speaks git over HTTP only (no SSH). The protocol surface is:

- `GET  /{owner}/{repo}.git/info/refs?service=git-upload-pack`
- `GET  /{owner}/{repo}.git/info/refs?service=git-receive-pack`
- `POST /{owner}/{repo}.git/git-upload-pack` (clone, fetch)
- `POST /{owner}/{repo}.git/git-receive-pack` (push)

isomorphic-git is built primarily for clients (it speaks to GitHub/etc. as a client
of someone else's git server). The plan calls for "a thin TypeScript layer over
isomorphic-git's pack and ref primitives" but explicitly flags the risk that the
exposed primitives may be insufficient.

## Method

Cloned `isomorphic-git/isomorphic-git` (depth=1) to inspect:

- `src/internal-apis.js` (everything the package exports beyond the public client API)
- `src/wire/*` (pkt-line / smart-HTTP framing)
- `src/commands/{pack,uploadPack}.js` (server-relevant command implementations)
- `src/managers/GitRefManager.js` (ref read/write)
- `src/models/{GitPktLine,GitSideBand,GitPackIndex}.js` (low-level codecs)
- `src/api/indexPack.js` (public API for ingesting a received packfile)

## Capability matrix

| Operation                                                          | Status
| Path                                                                |
| ------------------------------------------------------------------ |
--------------------------------------------------------------------------------- |
------------------------------------------------------------------- |
| Read git object (`readObject`)                                     | available
| `internal-apis` -> `_readObject`                                    |
| Write git object (`writeObject`)                                   | available
| `internal-apis` -> `_writeObject`                                   |
| Resolve ref to OID                                                 | available
| `internal-apis` -> `GitRefManager.resolve`                          |
| List refs                                                          | available
| `internal-apis` -> `GitRefManager.listRefs`                         |
| Build packfile from OID set                                        | available
| `internal-apis` -> `_pack` (`commands/pack.js`)                     |
| Index/apply received packfile                                      | available via public API
| `indexPack({fs, dir, gitdir, filepath})` writes `.idx` next to pack |
| Write `info/refs` advertisement                                    | available
| `internal-apis` -> `writeRefsAdResponse`                            |
| Parse `git-upload-pack` request (server reads wants/haves)         | available
| `internal-apis` -> `parseUploadPackRequest`                         |
| Write `git-upload-pack` response (NAK + sideband pack)             | **missing on server side**
(only client-side parser)                              | must vendor
|
| Parse `git-receive-pack` request (server reads ref updates + pack) | **missing**
| must vendor                                                         |
| Write `git-receive-pack` response (unpack + ref-update report)     | **missing** (only client-side parser)
| must vendor                                                         |
| Sideband multiplex (pack/progress/error onto channels 1/2/3)       | **missing:
`GitSideBand.mux` is commented out** at `src/models/GitSideBand.js:82` | must vendor
|

Source citation for the missing pieces:

- `src/models/GitSideBand.js:82-148`: the `static mux(...)` method is fully commented
  out. `demux` (client-side) is implemented; `mux` (server-side) is not. The commented
  block uses Node `PassThrough` streams; we will rewrite it as an async iterator over
  `Uint8Array` chunks for runtime portability.
- `src/wire/`: every file is named `parse*Response` (client) or `write*Request` (client),
  except `parseUploadPackRequest` (server) and `writeRefsAdResponse` (server). There
  is no `parseReceivePackRequest` and no `writeReceivePackResponse`.

## Decision

Vendor the missing wire pieces in `src/git/pack-protocol.ts`. Estimated total LOC,
based on counting analogous client-side files in isomorphic-git:

- sideband mux: ~50 LOC (the commented reference is ~70 LOC of Node streams; an
  async-iterator rewrite is shorter)
- `parseReceivePackRequest`: ~40 LOC (matches `parseUploadPackRequest` shape)
- `writeReceivePackResponse`: ~40 LOC
- `writeUploadPackResponse`: ~30 LOC (NAK + sideband-wrapped pack delivery)

Total: roughly 160 LOC of new wire code. This stays within the plan's "thin layer"
characterisation. The protocol shapes are RFC-documented (`Documentation/gitprotocol-pack.txt`
in git itself) so we are vendoring well-specified format code, not novel logic.

## Filesystem strategy

isomorphic-git's pack/ref operations all take an `fs` argument. The expected interface
is documented at `src/models/FileSystem.js:24-35`:

```
['readFile', 'writeFile', 'mkdir', 'rmdir', 'unlink', 'stat', 'lstat',
 'readdir', 'readlink', 'symlink']
```

**Decision pending** until task #16 actually starts. Two viable options:

- (a) Real local scratch directory per repo via `node:fs/promises`, e.g.
  `${WEBAPP_FORGE_GITDIR_ROOT}/${owner}/${repo}.git/`. Simpler; no virtual fs; the
  directory **is** the source of truth (backed up like any other persistent volume).
- (b) libSQL-backed virtual fs implementing the 9 methods. Aligns with
  the rest-of-system data plane but expands the vendored surface
  (rmRecursive, lstat, symlink edge cases).

Phase 2's verification target (`git clone` / `git push` round-trip) is unambiguously
testable under (a). The choice between (a) and (b) only matters for multi-machine
deployment, which is Phase 3+ scope.

## Verification plan (pre-merge)

1. `git clone http://localhost:3000/x/y.git /tmp/clone` against a freshly seeded repo, assert exit 0 and matching tree.
2. `git push http://localhost:3000/x/y.git HEAD:refs/heads/feat-abc` from a local clone,
   assert exit 0 and assert the `push` event hits the event log.
3. Round-trip a binary blob (e.g. a 5 MB random file) to confirm sideband-64k chunking handles the 65,519 byte payload
   limit correctly.
4. Push 100 small refs in one POST to confirm the ref-update parser handles batched updates.

## Draft GitHub issue (to file upstream against isomorphic-git)

Title: server-side smart-HTTP support is incomplete (sideband mux commented out, no receive-pack request parser)

Labels: enhancement, server-side

Description:

> isomorphic-git ships several server-relevant primitives via `src/internal-apis.js`
> (e.g. `_pack`, `parseUploadPackRequest`, `writeRefsAdResponse`, `GitRefManager`),
> but a few key pieces are missing for anyone writing a server in pure JS:
>
> - `src/models/GitSideBand.js`: `mux` is commented out (lines 82-148). Only `demux`
>   (client-side parsing) is implemented. Servers need to multiplex packfile bytes,
>   progress messages, and fatal errors onto sideband channels 1/2/3.
> - `src/wire/`: there is no `parseReceivePackRequest`, so a server cannot parse a
>   client's `git-receive-pack` POST (the leading ref-update lines and trailing pack
>   stream).
> - `src/wire/`: there is no `writeReceivePackResponse`, so a server cannot emit the
>   "unpack ok" / "ng <ref> <reason>" status report after applying a pushed pack.
> - `src/wire/`: there is no `writeUploadPackResponse` (NAK + sideband-wrapped pack
>   delivery).
>
> Each gap forces server implementations to vendor wire-format code, duplicating
> the well-tested client-side functions in this repo. Reproduction: clone the repo
> and grep `wire/`: only `parseUploadPackRequest` and `writeRefsAdResponse` are
> server-flavoured.
>
> Suggested fix: extract the commented `GitSideBand.mux` into a runtime-portable
> async-iterator helper, add the four missing wire functions (mirroring the existing
> client-side parsers/writers), and export them from `internal-apis.js`. Happy to
> open a PR.
