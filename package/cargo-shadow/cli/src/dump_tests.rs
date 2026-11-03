//! The unit tests of the CSV reader and the dump fill,
//! ported from `dump.unit.test.ts`,
//! `csv` cases and the tar cases against
//! an archive built in memory.

use std::collections::BTreeSet;
use std::io::{Cursor, Read, Write};

use flate2::Compression;
use flate2::write::GzEncoder;

use crate::csv::read_csv;
use crate::dump::{dump_timestamp, fill_histories_from_dump, needs_api};
use crate::policy::VersionRecord;
use crate::test_scratch::scratch;

fn rows_of(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut reader: Cursor<Vec<u8>> = Cursor::new(text.as_bytes().to_vec());
    read_csv(&mut reader, &mut |row: &[String]| {
        rows.push(row.to_vec());
        return true;
    })
    .expect("reads");
    return rows;
}

#[test]
fn plain_rows_split_on_commas_and_line_feeds_and_a_carriage_return_is_dropped() {
    assert_eq!(rows_of("a,b\r\nc,d\n"), vec![vec!["a", "b"], vec!["c", "d"]]);
}

#[test]
fn quoted_fields_keep_commas_line_breaks_and_doubled_quotes() {
    assert_eq!(rows_of("id,text\n1,\"a,b\nc \"\"q\"\"\"\n"), vec![vec!["id", "text"], vec!["1", "a,b\nc \"q\""]]);
}

#[test]
fn a_last_row_without_a_final_line_feed_and_empty_fields_are_read() {
    assert_eq!(rows_of("a,,c\nx,y,"), vec![vec!["a", "", "c"], vec!["x", "y", ""]]);
}

#[test]
fn the_callback_can_stop_the_read() {
    let mut seen: usize = 0;
    let mut reader: Cursor<Vec<u8>> = Cursor::new(b"a\nb\nc\n".to_vec());

    read_csv(&mut reader, &mut |_row: &[String]| {
        seen += 1;
        return seen < 2;
    })
    .expect("reads");

    assert_eq!(seen, 2);
}

#[test]
fn a_utc_dump_timestamp_becomes_rfc_3339_and_anything_else_is_refused() {
    assert_eq!(dump_timestamp("2026-10-29 12:00:00.123456+00").expect("utc"), "2026-10-29T12:00:00.123456Z");
    assert!(dump_timestamp("2026-10-29 12:00:00+02").is_err());
}

fn record(created_at: &str, publisher_id: &str) -> VersionRecord {
    return VersionRecord {
        num: "1.0.0".to_string(),
        created_at: created_at.to_string(),
        publisher_id: publisher_id.to_string(),
        trustpub_repository: String::new(),
        repository: String::new(),
    };
}

#[test]
fn only_a_publisher_less_release_since_trusted_publishing_began_needs_the_api() {
    assert!(!needs_api(&[record("2025-06-30T23:59:59Z", "")]));
    assert!(!needs_api(&[record("2026-01-01T00:00:00Z", "7")]));
    assert!(needs_api(&[record("2025-07-01T00:00:00Z", "")]));
}

/// A gzip tar with the given (path,
/// content) entries.
fn archive(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (path, content) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, path, content.as_bytes()).expect("append");
    }
    let bytes: Vec<u8> = builder.into_inner().expect("finish the tar");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes).expect("compress");
    return encoder.finish().expect("finish the gzip");
}

fn wanted(names: &[&str]) -> BTreeSet<String> {
    return names.iter().map(|name| { return name.to_string(); }).collect();
}

const CRATES: &str = "id,name,other\n1,wanted,x\n2,unwanted,y\n3,trusted,z\n";
const VERSIONS: &str = "crate_id,num,created_at,published_by,repository\n1,1.0.0,2024-01-01 00:00:00.5+00,7,\"https://example.invalid/a,b\"\n1,1.0.1,2024-02-01 00:00:00+00,7,\n2,9.9.9,2024-01-01 00:00:00+00,1,\n3,1.0.0,2026-01-01 00:00:00+00,,\n";

#[test]
fn the_wanted_histories_are_written_and_a_possibly_trusted_one_is_left_to_the_api() {
    let cache = scratch();
    let bytes: Vec<u8> = archive(&[("2026-10-29/data/crates.csv", CRATES), ("2026-10-29/data/versions.csv", VERSIONS), ("2026-10-29/data/later.csv", "x\n")]);

    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted", "trusted"]), cache.path()).expect("fills");

    assert_eq!(result.found, 2);
    assert_eq!(result.written, 1);
    assert_eq!(result.left_to_api, vec!["trusted".to_string()]);
    let mut text: String = String::new();
    std::fs::File::open(cache.path().join("wanted.json")).expect("cache file").read_to_string(&mut text).expect("read");
    assert!(text.contains("\"num\":\"1.0.1\"") && text.contains("2024-01-01T00:00:00.5Z") && text.contains("https://example.invalid/a,b"));
    assert!(!cache.path().join("unwanted.json").exists());
    assert!(!cache.path().join("trusted.json").exists());
}

#[test]
fn an_existing_cache_file_is_kept() {
    let cache = scratch();
    std::fs::write(cache.path().join("wanted.json"), "[]\n").expect("existing");
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", VERSIONS)]);

    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path()).expect("fills");

    assert_eq!((result.kept, result.written), (1, 0));
    assert_eq!(std::fs::read_to_string(cache.path().join("wanted.json")).expect("read"), "[]\n");
}

#[test]
fn a_dump_with_versions_before_crates_a_missing_column_or_no_versions_is_refused_naming_why() {
    let cache = scratch();
    let reversed = archive(&[("d/data/versions.csv", VERSIONS), ("d/data/crates.csv", CRATES)]);
    let renamed = archive(&[("d/data/crates.csv", "id,title\n1,x\n"), ("d/data/versions.csv", VERSIONS)]);
    let truncated = archive(&[("d/data/crates.csv", CRATES)]);

    let first = fill_histories_from_dump(Box::new(Cursor::new(reversed)), &wanted(&["wanted"]), cache.path()).expect_err("refused");
    let second = fill_histories_from_dump(Box::new(Cursor::new(renamed)), &wanted(&["wanted"]), cache.path()).expect_err("refused");
    let third = fill_histories_from_dump(Box::new(Cursor::new(truncated)), &wanted(&["wanted"]), cache.path()).expect_err("refused");

    assert!(first.message.contains("versions.csv before crates.csv"));
    assert!(second.message.contains("\"name\" column"));
    assert!(third.message.contains("ended before versions.csv"));
}

#[test]
fn the_registry_reads_the_dump_only_above_the_break_even_and_the_api_serves_a_crate_the_dump_left() {
    use crate::http::{DumpSource, RegistryError, TextFetcher};
    use crate::registry::{CratesIo, Registry};
    use std::cell::Cell;
    use std::rc::Rc;

    struct OneArchive {
        bytes: Vec<u8>,
        opened: Rc<Cell<u32>>,
    }
    impl DumpSource for OneArchive {
        fn open(&self, _url: &str) -> Result<Box<dyn Read>, RegistryError> {
            self.opened.set(self.opened.get() + 1);
            return Ok(Box::new(Cursor::new(self.bytes.clone())));
        }
    }
    struct NoNetwork;
    impl TextFetcher for NoNetwork {
        fn get(&self, url: &str) -> Result<String, RegistryError> {
            return Err(RegistryError { message: format!("no network: {url}"), status: None, retry_after_seconds: 0 });
        }
    }
    let root = scratch();
    let opened: Rc<Cell<u32>> = Rc::new(Cell::new(0));
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", VERSIONS)]);
    let registry: CratesIo = CratesIo::new(root.path(), Box::new(NoNetwork), std::time::Duration::ZERO, std::time::Duration::ZERO)
        .with_dump(Box::new(OneArchive { bytes, opened: Rc::clone(&opened) }), 2);

    registry.prefetch(&wanted(&["wanted"])).expect("below the break-even: nothing is read");
    assert_eq!(opened.get(), 0);
    registry.prefetch(&wanted(&["wanted", "trusted"])).expect("at the break-even the dump is read");

    assert_eq!(opened.get(), 1);
    let history = registry.history("wanted", &["1.0.1".to_string()]).expect("served from the cache the dump filled, no network");
    assert_eq!(history.len(), 2);
    assert!(registry.history("trusted", &["1.0.0".to_string()]).is_err(), "a crate left to the API needs the network");
}

fn read_all(text: &str) -> std::io::Result<Vec<Vec<String>>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut reader: Cursor<Vec<u8>> = Cursor::new(text.as_bytes().to_vec());
    read_csv(&mut reader, &mut |row: &[String]| {
        rows.push(row.to_vec());
        return true;
    })?;
    return Ok(rows);
}

/// Review finding (malformed EOF): a stream that ends inside a quoted field is cut off, not finished.
#[test]
fn a_stream_that_ends_inside_a_quoted_field_is_an_error() {
    assert!(read_all("a,b\n1,\"never closed").is_err());
}

/// Review finding (malformed quoting): text after a closing quote, and a quote inside a bare field, are not CSV.
#[test]
fn text_after_a_closing_quote_and_a_quote_inside_a_bare_field_are_errors() {
    assert!(read_all("a,b\n\"x\"y,2\n").is_err());
    assert!(read_all("a,b\nx\"y,2\n").is_err());
}

/// Review finding (bounded memory): one field larger than the bound is refused instead of grown.
#[test]
fn a_field_over_the_bound_is_refused() {
    let text: String = format!("a\n\"{}\"\n", "x".repeat(crate::csv::MAX_FIELD_BYTES + 1));

    assert!(read_all(&text).is_err());
}

/// A row with more fields than the bound is refused.
#[test]
fn a_row_over_the_field_bound_is_refused() {
    let text: String = format!("a\n{}\n", ",".repeat(crate::csv::MAX_ROW_FIELDS + 1));

    assert!(read_all(&text).is_err());
}

/// Review finding (padding of short rows): a row narrower than the header is a damaged table, and nothing may be cached
/// from it.
#[test]
fn a_short_row_refuses_the_table_and_nothing_is_cached() {
    let cache = scratch();
    let short_versions: &str = "crate_id,num,created_at,published_by,repository\n1,1.0.0,2024-01-01 00:00:00+00,7\n";
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", short_versions)]);

    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path());

    assert!(result.is_err());
    assert!(!cache.path().join("wanted.json").exists());
}

/// Review finding: a versions table cut off inside a quoted field caches nothing.
#[test]
fn a_versions_table_cut_off_inside_a_quote_caches_nothing() {
    let cache = scratch();
    let cut: &str = "crate_id,num,created_at,published_by,repository\n1,1.0.0,2024-01-01 00:00:00+00,7,\"https://example.invalid/unfinished";
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", cut)]);

    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path());

    assert!(result.is_err());
    assert!(!cache.path().join("wanted.json").exists());
}

/// Review finding (retained history): one crate's history is bounded.
#[test]
fn more_releases_of_one_crate_than_the_bound_are_refused() {
    let cache = scratch();
    let mut versions: String = "crate_id,num,created_at,published_by,repository\n".to_string();
    for number in 0..=crate::dump::MAX_VERSIONS_PER_CRATE {
        versions.push_str(&format!("1,1.0.{number},2024-01-01 00:00:00+00,7,\n"));
    }
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", &versions)]);

    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path());

    assert!(result.is_err());
    assert!(!cache.path().join("wanted.json").exists());
}
