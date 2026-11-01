# webapp-forge-server

## Status: development paused

Active development is paused pending repo-wide work.
`mise run //packages/webapp-forge/server:lint` currently reports 13 errors
from `no-restricted-syntax/no-regex` across
`src/server/routes/{me,read}.ts`,
`src/storage/adapter-s3.ts`,
`src/storage/write-buffer.unit.test.ts`,
`src/worker/render-phase2.ts`,
and `src/worker/render.ts`.
The refactor is deferred;
resume by completing the no-regex sweep documented in `HANDOVER.no-regex.md`.

GitHub-alternative forge with **rebuild-on-write fragment cache**.

Issues, pull requests, repository pages, file trees, blobs, diffs, and filter lists
are stored as content-addressed HTML fragments in object storage.
A worker maps every event in the log to a fixed dependency graph of fragment keys,
re-renders the affected fragments, and writes them back atomically.

See `/home/user/.claude/plans/let-s-put-our-effort-valiant-hopper.md` for the full design.

## Run

```sh
mise run //packages/webapp-forge/server:dev
```

The server listens on `:3000` by default. Override with `--port=N` or `PORT=N`.
The libSQL database lives at `./data/forge.db`. Override with `--db=PATH` or `DB_PATH=PATH`.

## Phase 1 scope

Phase 1 is the smallest end-to-end loop:

- Data layer: users, repos, issues, comments, labels, issue_labels, events, fragment_index, sequences
- Storage adapter: in-memory only
- Fragments: `IssueDetail`, `FilterList`
- Dependency graph: `comment.created`, `issue.labeled`
- Dispatcher: synchronous in-request rebuild for Phase 1 (asynchronous worker pool is Phase 2+)
- Routes: create-issue, create-comment, fetch-fragment

Phases 2 to 4 (git protocol, auth, S3 adapter, additional fragments,
cross-cutting renames, Pagefind, swap shim) are additive on top of Phase 1.

## Phase 1 boot-time cache warming

Because the storage adapter is process-local in-memory, a fresh server
boot cannot see fragments rendered in a previous process (most notably,
the `forge:seed` CLI runs in its own process). The server therefore
drains every event in the libSQL log into the in-memory storage adapter
on startup before accepting requests.

Phase 2 introduces a persistent S3-compatible adapter; once that lands,
the fragment cache survives restarts and the warm-on-boot pass becomes
a fast no-op (the sequence guard discards every event whose fragments
already match storage).
