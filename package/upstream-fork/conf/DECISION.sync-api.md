# Decision: synchronous file I/O in conf-fork

Records why `package/upstream-fork/conf` keeps a synchronous read/write
 surface and how the repository's Node sync API ban is scoped to it.
 Written for a reviewer deciding whether the lint suppressions in
 `src/file-io.ts` and `src/encryption.ts` are justified.

## Context

Upstream [`conf`](https://github.com/sindresorhus/conf) 15.1.0 exposes a
 wholly synchronous API:
 `get`,
 `has`,
 `set`,
 `delete`,
 `clear`,
 and the
 `store` accessor all read and write the config file before returning
 (`source/index.ts`,
 `_readStore`,
 `_write`,
 `set store`).
 A migrating caller therefore expects `config.set(...)` to be on disk when
 it returns;
 making the fork asynchronous would change that contract and break the
 fork's purpose as a drop-in behavioral mirror.

The repository bans Node synchronous APIs through
 `no-restricted-syntax/no-sync`
 (declared at `package/config/oxlint/src/rule/restriction.ts`,
 implemented at `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.ts`
 with provenance resolution in `no-sync.provenance.ts`,
 matching `*Sync` members of `node:` builtin receivers).

## Why configuration cannot express this

The rule's `meta` carries no schema and no allow-list options
 (`no-sync.ts` exposes only `messages` and `meta`);
 its provenance matcher has no per-call exemption mechanism.
 The repository-level escapes are a file-scoped `off` override in
 `package/config/oxlint/src/overrides.ts` or inline `oxlint-disable` comments.
 An override would silence the rule for whole files unconditionally,
 with no call-site justification and no review hook,
 and `AGENTS.md` LN7 forbids loosening rules without prior approval.
 Inline disables keep every suppressed call visible and reasoned,
 the same shape `package/ssg/aquati.cat/src/build/compress-lib.ts` uses for
 its structurally sync zstd loop.

## Decision

Keep the upstream synchronous contract.
 Every `*Sync` call lives in `src/file-io.ts`,
 wrapped in one `oxlint-disable`/`oxlint-enable` block that names this
 decision document,
 plus one PBKDF2 derivation call in `src/encryption.ts` (key derivation must
 finish inside the synchronous write).
 No other module may call Node sync APIs;
 they use the file-io wrappers.
 Test files are exempt through the repository's `testOverride` and exercise
 the wrappers directly.

## Rejected alternatives

- An asynchronous fork.
   It would not be behaviorally compatible with upstream `conf`, which is
   the whole point of the fork.
- A file-scoped oxlint override for `src/file-io.ts`.
   Unconditional suppression loses per-call justification and loosens a
   rule without approval (`AGENTS.md` LN7).
- Per-call disables scattered across the store.
   Same suppression count as the chosen design but without one reviewable
   choke point.
- Async `fs/promises` bridged through `deasync`-style blocking.
   Adds a second runtime path and a worse trust surface than the direct
   sync calls it would hide.
