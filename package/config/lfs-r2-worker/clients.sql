-- Per-machine upload credentials for the monochromatic-lfs-index D1 database.
--
-- One row per machine allowed to store objects. `id` is the HTTP Basic user name,
-- `label` is how a person recognises the machine, `scope` says what the row may do
-- (`write` is the only scope the Worker accepts, because downloads are anonymous),
-- and `hash` is the lowercase hex sha256 of the token, never the token itself.
-- `created`, `expires` and `revoked` are epoch milliseconds; NULL in `expires`
-- means the row never expires, and NULL in `revoked` means it still stands.
--
-- Writing the first row retires the shared LFS_WRITE_TOKEN secret, because the
-- Worker accepts that secret only while this table holds no row. A revoked row
-- still counts, so cancelling every machine never reopens the shared secret.
--
-- Rows are written and revoked by hand with `wrangler d1 execute`; the procedure
-- is doc/runbook/lfs-r2-worker.md. Deleting rows is not part of it: revoke instead.
CREATE TABLE IF NOT EXISTS clients (
  id TEXT PRIMARY KEY NOT NULL,
  label TEXT NOT NULL,
  scope TEXT NOT NULL DEFAULT 'write',
  hash TEXT NOT NULL CHECK (length(hash) = 64),
  created INTEGER NOT NULL,
  expires INTEGER,
  revoked INTEGER
) WITHOUT ROWID;
