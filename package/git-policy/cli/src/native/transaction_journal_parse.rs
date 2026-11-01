//! What:
//!  Strict parsers of every schema-version-2 journal record.
//! Why:
//!  Recovery reads records either wrapper wrote;
//!  every missing or mistyped field fails
//!      closed with the record and field named,
//!  exactly as the incumbent's
//!      `commit-transaction-journal-parse.ts` and `commit-transaction-journal-reader.ts` do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const preparing = parsePreparingRecord(bytes);
//! ```

/// The digest of an index snapshot.
use super::content_digest::ContentDigest;
/// The event form of a pathname and the base64 of its bytes.
use super::event_path::{EventPath, base64_decode_standard};
/// Record field readers and the fatal decoder.
use super::json_record::{decode_fatal, number_equals, parse_object, safe_integer, string_field};
/// The fail-closed recovery failure.
use super::recovery_error::RecoveryError;
/// The record types and filenames.
use super::transaction_journal::{
    AddedPath, Base, Conclusion, FileIdentity, IndexLockRecord, JOURNAL_SCHEMA_VERSION,
    LandingOperation, LandingRecord, LockIdentity, PREPARED_FILENAME, PREPARING_FILENAME,
    PreparedRecord, PreparingRecord, REF_UPDATED_FILENAME, RefFormat, SymbolicHead,
    TransactionMode,
};
/// What:
///  `Map` is a JSON object's table,
///  `Value` any JSON value.
/// Why:
///   Records are checked field by field from the general form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Value = unknown;
/// ```
use serde_json::{Map, Value};

/// What:
///  A field reader over one untrusted record object,
///  naming the record in failures.
/// Why:
///   The incumbent's `createRecordReader`:
///  each accessor fails closed with
///       "Transaction record <name> has a malformed <key> field."
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordReader = { string(key): string; ... };
/// ```
pub struct RecordReader<'a> {
    /// Record name used in diagnostics,
    ///  `#`-joined for nested objects.
    name: String,
    /// The record object.
    map: &'a Map<String, Value>,
}

/// Typed accessors,
///  each failing closed with the field named.
impl<'a> RecordReader<'a> {
    /// What:
    ///  A reader over a parsed object.
    /// Why:
    ///   Nested objects get their own reader with a `#`-joined name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// createRecordReader({ name, value })
    /// ```
    pub fn new(name: &str, map: &'a Map<String, Value>) -> Self {
        return Self {
            name: String::from(name),
            map,
        };
    }

    /// What:
    ///  The malformed-field failure.
    /// Why:
    ///   Every accessor reports the same sentence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// malformed(key)
    /// ```
    pub fn malformed(&self, key: &str) -> RecoveryError {
        return RecoveryError(format!(
            "Transaction record {} has a malformed {key} field.",
            self.name
        ));
    }

    /// What:
    ///  A required text field;
    ///  any text,
    ///  empty included,
    ///  as `typeof === 'string'`.
    /// Why:
    ///   The incumbent's `readString`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.string(key)
    /// ```
    pub fn string(&self, key: &str) -> Result<String, RecoveryError> {
        match string_field(self.map, key) {
            Some(text) => return Ok(String::from(text)),
            None => return Err(self.malformed(key)),
        }
    }

    /// What:
    ///  An optional text field:
    ///  absent is `None`,
    ///  present must be text.
    /// Why:
    ///   The incumbent's `optionalString` (`key in value`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.optionalString(key)
    /// ```
    pub fn optional_string(&self, key: &str) -> Result<Option<String>, RecoveryError> {
        if !self.map.contains_key(key) {
            return Ok(None);
        }
        return Ok(Some(self.string(key)?));
    }

    /// What:
    ///  A required boolean field.
    /// Why:
    ///   The incumbent's `boolean`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.boolean(key)
    /// ```
    pub fn boolean(&self, key: &str) -> Result<bool, RecoveryError> {
        match self.map.get(key) {
            Some(Value::Bool(flag)) => return Ok(*flag),
            Some(_) | None => return Err(self.malformed(key)),
        }
    }

    /// What:
    ///  A required safe integer of at least 1.
    /// Why:
    ///   The incumbent's `positiveInteger`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.positiveInteger(key)
    /// ```
    pub fn positive_integer(&self, key: &str) -> Result<i64, RecoveryError> {
        match self.map.get(key).and_then(safe_integer) {
            Some(whole) if whole >= 1 => return Ok(whole),
            Some(_) | None => return Err(self.malformed(key)),
        }
    }

    /// What:
    ///  A required array of text.
    /// Why:
    ///   The incumbent's `strings`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.strings(key)
    /// ```
    pub fn strings(&self, key: &str) -> Result<Vec<String>, RecoveryError> {
        let Some(Value::Array(items)) = self.map.get(key) else {
            return Err(self.malformed(key));
        };
        let mut texts: Vec<String> = Vec::with_capacity(items.len());
        for item in items {
            let Value::String(text) = item else {
                return Err(self.malformed(key));
            };
            texts.push(text.clone());
        }
        return Ok(texts);
    }

    /// What:
    ///  A required text field that names a path,
    ///  with the exact bytes from `bytes_key` when the record has them.
    /// Why:
    ///   A name that is not UTF-8 is stored as its readable text and, beside it, the base64 of its bytes;
    ///  a record written by the incumbent has only the text, which is then the exact name.
    ///  Bytes whose readable text differs from the text field are a damaged record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.path(key, bytesKey)
    /// ```
    pub fn path(&self, key: &str, bytes_key: &str) -> Result<Vec<u8>, RecoveryError> {
        let text: String = self.string(key)?;
        if !self.map.contains_key(bytes_key) {
            return Ok(text.into_bytes());
        }
        let encoded: String = self.string(bytes_key)?;
        return self.exact_path(text.as_str(), encoded.as_str(), bytes_key);
    }

    /// What:
    ///  A required array of text that names paths,
    ///  with the exact bytes of every path from `bytes_key` when the record has them.
    /// Why:
    ///   See `path`;
    ///  the two arrays line up entry for entry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.paths(key, bytesKey)
    /// ```
    pub fn paths(&self, key: &str, bytes_key: &str) -> Result<Vec<Vec<u8>>, RecoveryError> {
        let texts: Vec<String> = self.strings(key)?;
        if !self.map.contains_key(bytes_key) {
            return Ok(texts.into_iter().map(String::into_bytes).collect());
        }
        let encoded: Vec<String> = self.strings(bytes_key)?;
        if encoded.len() != texts.len() {
            return Err(self.malformed(bytes_key));
        }
        let mut paths: Vec<Vec<u8>> = Vec::with_capacity(texts.len());
        for (text, entry) in texts.iter().zip(encoded.iter()) {
            paths.push(self.exact_path(text.as_str(), entry.as_str(), bytes_key)?);
        }
        return Ok(paths);
    }

    /// What:
    ///  The bytes a base64 entry names,
    ///  required to read as the text beside it.
    /// Why:
    ///   Both readers of a path accept exactly the pairs the writer prints.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// exactPath(text, encoded)
    /// ```
    fn exact_path(&self, text: &str, encoded: &str, bytes_key: &str) -> Result<Vec<u8>, RecoveryError> {
        let Some(bytes) = base64_decode_standard(encoded) else {
            return Err(self.malformed(bytes_key));
        };
        if EventPath::from_git_bytes(bytes.as_slice()).text() != text {
            return Err(self.malformed(bytes_key));
        }
        return Ok(bytes);
    }

    /// What:
    ///  A required nested object,
    ///  as its own reader.
    /// Why:
    ///   The incumbent's `object`;
    ///  arrays and `null` are not objects here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.object(key)
    /// ```
    pub fn object(&self, key: &str) -> Result<RecordReader<'a>, RecoveryError> {
        match self.map.get(key) {
            Some(Value::Object(nested)) => {
                return Ok(RecordReader {
                    name: format!("{}#{key}", self.name),
                    map: nested,
                });
            }
            Some(_) | None => return Err(self.malformed(key)),
        }
    }

    /// What:
    ///  A required base.
    /// Why:
    ///   The incumbent's `base`:
    ///  `unborn`,
    ///  or `commit` with an `oid`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.base(key)
    /// ```
    pub fn base(&self, key: &str) -> Result<Base, RecoveryError> {
        let nested: RecordReader<'a> = self.object(key)?;
        let kind: String = nested.string("kind")?;
        if kind == "unborn" {
            return Ok(Base::Unborn);
        }
        if kind != "commit" {
            return Err(self.malformed(key));
        }
        return Ok(Base::Commit(nested.string("oid")?));
    }

    /// What:
    ///  A required file identity.
    /// Why:
    ///   The incumbent's `identity`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.identity(key)
    /// ```
    pub fn identity(&self, key: &str) -> Result<FileIdentity, RecoveryError> {
        let nested: RecordReader<'a> = self.object(key)?;
        return Ok(FileIdentity {
            device: nested.string("device")?,
            inode: nested.string("inode")?,
        });
    }

    /// What:
    ///  An optional content digest of an index snapshot:
    ///  absent is `None`,
    ///  present must be `{ "length": <whole number>, "fnv1a64": <16 lowercase hexadecimal digits> }`.
    /// Why:
    ///   Records written before the digests existed have none and keep their old trust in the artifact identity;
    ///  a digest that is present but misspelled is a damaged record and never read as absent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.optionalDigest(key)
    /// ```
    pub fn optional_digest(&self, key: &str) -> Result<Option<ContentDigest>, RecoveryError> {
        if !self.map.contains_key(key) {
            return Ok(None);
        }
        let nested: RecordReader<'a> = self.object(key)?;
        let length: u64 = match nested.map.get("length").and_then(safe_integer) {
            Some(whole) if whole >= 0 => whole as u64,
            Some(_) | None => return Err(self.malformed(key)),
        };
        let hash_text: String = nested.string("fnv1a64")?;
        match ContentDigest::from_record(length, hash_text.as_str()) {
            Some(digest) => return Ok(Some(digest)),
            None => return Err(self.malformed(key)),
        }
    }

    /// What:
    ///  A required lock identity.
    /// Why:
    ///   The incumbent's `lock`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.lock(key)
    /// ```
    pub fn lock(&self, key: &str) -> Result<LockIdentity, RecoveryError> {
        return Ok(LockIdentity {
            file: self.identity(key)?,
            fs_id: self.object(key)?.string("fsId")?,
        });
    }

    /// What:
    ///  A required list of added-path records.
    /// Why:
    ///   The incumbent's `addedPaths`:
    ///  each item an object with a regular or executable mode.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read.addedPaths(key)
    /// ```
    pub fn added_paths(&self, key: &str) -> Result<Vec<AddedPath>, RecoveryError> {
        let Some(Value::Array(items)) = self.map.get(key) else {
            return Err(self.malformed(key));
        };
        let mut records: Vec<AddedPath> = Vec::with_capacity(items.len());
        for item in items {
            // `typeof item !== 'object' || item === null` admits arrays; their fields are missing.
            let Value::Object(fields) = item else {
                return Err(self.malformed(key));
            };
            let nested: RecordReader<'_> = RecordReader {
                name: format!("{}#{key}", self.name),
                map: fields,
            };
            let git_mode: String = nested.string("gitMode")?;
            if git_mode != "100644" && git_mode != "100755" {
                return Err(self.malformed(key));
            }
            records.push(AddedPath {
                path: nested.path("path", "pathBytes")?,
                git_mode,
                original_oid: nested.string("originalOid")?,
                intended_oid: nested.string("intendedOid")?,
            });
        }
        return Ok(records);
    }
}

/// What:
///  Parse a record's bytes into its object after checking schema version and state.
///       Returns the parsed object;
///  the caller builds a reader over it.
/// Why:
///   The incumbent's `openRecord`:
///  fatal UTF-8,
///  a JSON object,
///  version 2,
///  the expected
///       state;
///  anything else fails with the record named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openRecord({ bytes, name, state }): RecordReader;
/// ```
pub fn open_record(
    bytes: &[u8],
    name: &str,
    state: &str,
) -> Result<Map<String, Value>, RecoveryError> {
    let failure: RecoveryError = RecoveryError(format!(
        "Transaction record {name} is not a schema-version-2 {state} record."
    ));
    let Some(text) = decode_fatal(bytes) else {
        return Err(failure);
    };
    let Some(map) = parse_object(text) else {
        return Err(failure);
    };
    if !number_equals(&map, "schemaVersion", JOURNAL_SCHEMA_VERSION)
        || string_field(&map, "state") != Some(state)
    {
        return Err(failure);
    }
    return Ok(map);
}

/// What:
///  Parse `preparing.json`.
/// Why:
///   The incumbent's `parsePreparingRecord`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePreparingRecord(bytes: Uint8Array): PreparingRecord;
/// ```
pub fn parse_preparing(bytes: &[u8]) -> Result<PreparingRecord, RecoveryError> {
    let map: Map<String, Value> = open_record(bytes, PREPARING_FILENAME, "preparing")?;
    let read: RecordReader<'_> = RecordReader::new(PREPARING_FILENAME, &map);
    let mode_text: String = read.string("mode")?;
    let conclusion_text: String = read.string("conclusion")?;
    let format_text: String = read.string("refFormat")?;
    let (Some(mode), Some(conclusion), Some(ref_format)) = (
        parse_mode(mode_text.as_str()),
        parse_conclusion(conclusion_text.as_str()),
        parse_ref_format(format_text.as_str()),
    ) else {
        return Err(RecoveryError(String::from(
            "Transaction record preparing.json has a malformed mode, conclusion, or refFormat field.",
        )));
    };
    let transaction_id: String = read.string("transactionId")?;
    let base: Base = read.base("base")?;
    let head: RecordReader<'_> = read.object("symbolicHead")?;
    let head_kind: String = head.string("kind")?;
    let symbolic_head: SymbolicHead = if head_kind == "detached" {
        SymbolicHead::Detached
    } else if head_kind == "branch" {
        SymbolicHead::Branch(head.string("ref")?)
    } else {
        return Err(RecoveryError(String::from(
            "Transaction record preparing.json has a malformed symbolicHead field.",
        )));
    };
    return Ok(PreparingRecord {
        transaction_id,
        mode,
        base,
        symbolic_head,
        target_ref: read.string("targetRef")?,
        conclusion,
        repository_root: read.string("repositoryRoot")?,
        git_dir: read.string("gitDir")?,
        common_dir: read.string("commonDir")?,
        real_index_path: read.string("realIndexPath")?,
        object_directory: read.string("objectDirectory")?,
        ref_format,
        empty_tree_oid: read.string("emptyTreeOid")?,
        shadow_path: read.string("shadowPath")?,
        selected_pathspecs: read.paths("selectedPathspecs", "selectedPathspecBytes")?,
        invoked_at: read.string("invokedAt")?,
    });
}

/// What:
///  The mode a wire spelling names.
/// Why:
///   Unknown spellings are malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mode === 'explicit-path' || mode === 'index'
/// ```
fn parse_mode(text: &str) -> Option<TransactionMode> {
    match text {
        "explicit-path" => return Some(TransactionMode::ExplicitPath),
        "index" => return Some(TransactionMode::Index),
        _ => return None,
    }
}

/// What:
///  The conclusion kind a wire spelling names.
/// Why:
///   Unknown spellings are malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isConclusionKind(text)
/// ```
fn parse_conclusion(text: &str) -> Option<Conclusion> {
    match text {
        "none" => return Some(Conclusion::None),
        "amend" => return Some(Conclusion::Amend),
        "merge" => return Some(Conclusion::Merge),
        "cherry-pick" => return Some(Conclusion::CherryPick),
        "revert" => return Some(Conclusion::Revert),
        _ => return None,
    }
}

/// What:
///  The ref format a wire spelling names.
/// Why:
///   Unknown spellings are malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refFormat === 'files' || refFormat === 'reftable'
/// ```
fn parse_ref_format(text: &str) -> Option<RefFormat> {
    match text {
        "files" => return Some(RefFormat::Files),
        "reftable" => return Some(RefFormat::Reftable),
        _ => return None,
    }
}

/// What:
///  Parse `prepared.json`.
/// Why:
///   The incumbent's `parsePreparedRecord`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePreparedRecord(bytes: Uint8Array): PreparedRecord;
/// ```
pub fn parse_prepared(bytes: &[u8]) -> Result<PreparedRecord, RecoveryError> {
    let map: Map<String, Value> = open_record(bytes, PREPARED_FILENAME, "prepared")?;
    let read: RecordReader<'_> = RecordReader::new(PREPARED_FILENAME, &map);
    return Ok(PreparedRecord {
        shadow_path: read.string("shadowPath")?,
        prepared_oid: read.string("preparedOid")?,
        signed: read.boolean("signed")?,
        intended_tree_oid: read.string("intendedTreeOid")?,
        committed_paths: read.paths("committedPaths", "committedPathBytes")?,
        added_paths: read.added_paths("addedPaths")?,
        selected_worktree_paths: read.added_paths("selectedWorktreePaths")?,
    });
}

/// What:
///  Parse `index-lock-<n>.json`.
/// Why:
///   The incumbent's `parseIndexLockRecord`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseIndexLockRecord(bytes: Uint8Array): IndexLockRecord;
/// ```
pub fn parse_index_lock(bytes: &[u8]) -> Result<IndexLockRecord, RecoveryError> {
    let name: &str = "index-lock record";
    let map: Map<String, Value> = open_record(bytes, name, "index-locked")?;
    let read: RecordReader<'_> = RecordReader::new(name, &map);
    return Ok(IndexLockRecord {
        attempt: read.positive_integer("attempt")?,
        lock: read.lock("lock")?,
    });
}

/// What:
///  Parse `landing-<n>.json`.
/// Why:
///   The incumbent's `parseLandingRecord`:
///  a commit landing must name its new OID.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLandingRecord(bytes: Uint8Array): LandingRecord;
/// ```
pub fn parse_landing(bytes: &[u8]) -> Result<LandingRecord, RecoveryError> {
    let name: &str = "landing record";
    let map: Map<String, Value> = open_record(bytes, name, "landing")?;
    let read: RecordReader<'_> = RecordReader::new(name, &map);
    let operation_text: String = read.string("operation")?;
    let new_oid: Option<String> = read.optional_string("newOid")?;
    let pack_name: Option<String> = read.optional_string("packName")?;
    let operation: LandingOperation = match operation_text.as_str() {
        "commit" if new_oid.is_some() => LandingOperation::Commit,
        "normalize-only" => LandingOperation::NormalizeOnly,
        _ => {
            return Err(RecoveryError(String::from(
                "Transaction record landing record has a malformed operation or newOid field.",
            )));
        }
    };
    return Ok(LandingRecord {
        attempt: read.positive_integer("attempt")?,
        operation,
        expected_old: read.base("expectedOld")?,
        new_oid,
        landed_tree_oid: read.string("landedTreeOid")?,
        pre_landing_index: read.identity("preLandingIndex")?,
        post_index: read.identity("postIndex")?,
        lock: read.lock("lock")?,
        pack_name,
        added_paths: read.added_paths("addedPaths")?,
        selected_worktree_paths: read.added_paths("selectedWorktreePaths")?,
        pre_landing_digest: read.optional_digest("preLandingDigest")?,
        post_digest: read.optional_digest("postDigest")?,
    });
}

/// What:
///  Parse `ref-updated.json` into the landed commit.
/// Why:
///   The incumbent's `parseRefUpdatedRecord`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRefUpdatedRecord(bytes: Uint8Array): RefUpdatedRecord;
/// ```
pub fn parse_ref_updated(bytes: &[u8]) -> Result<String, RecoveryError> {
    let map: Map<String, Value> = open_record(bytes, REF_UPDATED_FILENAME, "ref-updated")?;
    return RecordReader::new(REF_UPDATED_FILENAME, &map).string("landedOid");
}

/// Incumbent-fixture and malformed-record controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_journal_parse_tests.rs"]
mod tests;
