-- Presence index of the monochromatic-lfs-index D1 database: one row per oid stored in the R2 bucket.
-- A row means the object exists; a missing row proves nothing, so the Worker confirms misses against R2.
CREATE TABLE IF NOT EXISTS present (oid TEXT PRIMARY KEY) WITHOUT ROWID;
