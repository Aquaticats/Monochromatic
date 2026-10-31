//! What:
//!  Test-only helpers:
//! the committed data revision under `fixtures/data`,
//! in-memory npm tarballs,
//!  and a fake registry that serves them over loopback HTTP.
//! Why:
//!  Several test modules need the same real revision and the same download plumbing;
//!  one helper module keeps them honest about which files they read and keeps tests off the network.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const fixtureFiles = (firefoxEsr: string): DataFiles => ({ /* the four committed files */ });
//! ```

/// What:
///  Import `DataFiles` from `crate::data::DataFiles`.
/// Why:
///  Selectors and the loader pass the decoded data around as one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DataFiles } from 'crate/data';
/// ```
use crate::data::DataFiles;

/// Raw text of the committed caniuse-lite `agents.js`.
pub const AGENTS_JS: &str = include_str!("../fixtures/data/agents.js.txt");
/// Raw text of the committed caniuse-lite `browsers.js`.
pub const BROWSERS_JS: &str = include_str!("../fixtures/data/browsers.js.txt");
/// Raw text of the committed caniuse-lite `browserVersions.js`.
pub const BROWSER_VERSIONS_JS: &str = include_str!("../fixtures/data/browserVersions.js.txt");
/// Raw text of the committed node-releases `envs.json`.
pub const ENVS_JSON: &str = include_str!("../fixtures/data/envs.json.txt");
/// The ESR value the conformance fixtures were generated against.
pub const FIXTURE_ESR: &str = "153.6.0esr";

/// What:
///  The committed revision as `DataFiles` with a chosen ESR value.
/// Why:
///  Tests vary the ESR value without touching the four data files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtureFiles(firefoxEsr: string): DataFiles;
/// ```
pub fn fixture_files(firefox_esr: &str) -> DataFiles<'_> {
    return DataFiles {
        agents_js: AGENTS_JS,
        browsers_js: BROWSERS_JS,
        browser_versions_js: BROWSER_VERSIONS_JS,
        envs_json: ENVS_JSON,
        firefox_esr,
    };
}

/// What:
///  One empty scratch directory beside the build output (never under `/tmp`),
/// removed when the guard drops.
/// Why:
///  Cache and lock tests write files and must not touch real state or the system temporary directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dir = await mkdtemp(join(buildDir, 'test-scratch', 'x-'));
/// ```
pub fn scratch() -> tempfile::TempDir {
    let build_dir: Option<std::path::PathBuf> = std::env::current_exe()
        .ok()
        .and_then(|executable| return executable.ancestors().nth(3).map(std::path::Path::to_path_buf));
    let root: std::path::PathBuf = build_dir
        .unwrap_or_else(|| return std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"))
        .join("test-scratch");
    std::fs::create_dir_all(&root).expect("make the scratch root");
    return tempfile::tempdir_in(root).expect("make a scratch directory");
}

/// Gzip a finished tar byte vector.
fn gzip(tar_bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    std::io::Write::write_all(&mut encoder, tar_bytes).expect("gzip the tar bytes");
    return encoder.finish().expect("finish gzip");
}

/// What:
///  Build a gzip-compressed tar archive in memory from `(path, contents)` pairs,
/// all regular files.
/// Why:
///  Tests need real npm-shaped tarballs without network access or committed binary blobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildTgz(entries: [string, Uint8Array][]): Uint8Array;
/// ```
pub fn build_tgz(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (path, contents) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        builder.append_data(&mut header, path, *contents).expect("append entry");
    }
    return gzip(&builder.into_inner().expect("finish tar"));
}

/// What:
///  Build a gzip-compressed tar whose single entry is a symbolic link at `path`.
/// Why:
///  A wanted path that is a link must be refused instead of read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildSymlinkTgz(path: string): Uint8Array;
/// ```
pub fn build_symlink_tgz(path: &str) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(0);
    header.set_entry_type(tar::EntryType::Symlink);
    builder.append_link(&mut header, path, "/etc/passwd").expect("append link");
    return gzip(&builder.into_inner().expect("finish tar"));
}

/// What:
///  The tarball of a fake `caniuse-lite` package holding the committed data files.
/// Why:
///  Cache,
/// update and API tests download this from the fake registry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function caniuseTgz(): Uint8Array;
/// ```
pub fn caniuse_tgz() -> Vec<u8> {
    return build_tgz(&[
        ("package/package.json", b"{}"),
        ("package/data/agents.js", AGENTS_JS.as_bytes()),
        ("package/data/browsers.js", BROWSERS_JS.as_bytes()),
        ("package/data/browserVersions.js", BROWSER_VERSIONS_JS.as_bytes()),
    ]);
}

/// What:
///  The tarball of a fake `node-releases` package holding the committed release list.
/// Why:
///  Same as `caniuse_tgz`,
/// for the second package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nodeReleasesTgz(): Uint8Array;
/// ```
pub fn node_releases_tgz() -> Vec<u8> {
    return build_tgz(&[("package/data/processed/envs.json", ENVS_JSON.as_bytes())]);
}

/// What:
///  A lock whose integrities match the in-memory fixture tarballs.
/// Why:
///  Tests resolve against the fake registry with the same verification the real lock gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtureLock(): DataLock;
/// ```
pub fn fixture_lock() -> crate::lock::DataLock {
    return crate::lock::DataLock {
        caniuse_lite: crate::lock::PinnedPackage {
            name: "caniuse-lite".to_string(),
            version: "1.0.30001820".to_string(),
            integrity: crate::integrity::integrity_of(&caniuse_tgz()),
        },
        node_releases: crate::lock::PinnedPackage {
            name: "node-releases".to_string(),
            version: "2.0.61".to_string(),
            integrity: crate::integrity::integrity_of(&node_releases_tgz()),
        },
        firefox_esr: FIXTURE_ESR.to_string(),
    };
}

/// What:
///  A fake registry serving the two fixture tarballs at npm's tarball paths.
/// Why:
///  Cache-miss tests fetch from it,
/// and its request log shows whether a fetch happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtureRegistry(): FakeRegistry;
/// ```
pub fn fixture_registry() -> crate::fake_registry::FakeRegistry {
    return crate::fake_registry::FakeRegistry::start(vec![
        ("/caniuse-lite/-/caniuse-lite-1.0.30001820.tgz", 200, caniuse_tgz()),
        ("/node-releases/-/node-releases-2.0.61.tgz", 200, node_releases_tgz()),
    ]);
}
