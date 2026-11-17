//! Tests URL encoding and routing with disposable browser profile data.

/// Imports the path and file functions used to create disposable fixtures.
use std::{ffi::OsStr, fs};

/// Imports the GIO methods used to check file URI decoding.
use gio::prelude::FileExt;

/// Imports the app entry points exercised by these tests.
use crate::{profiles, target};

/// What: Checks that a web URL keeps its query and fragment byte for byte.
/// Why: The picker must forward the supplied address unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(parse(url).uri, url);
/// ```
#[test]
fn web_url_keeps_escaping_and_fragment() -> anyhow::Result<()> {
    let address = "https://example.test/path?q=%23%25%26&name=work#tab";
    let parsed = target::parse(OsStr::new(address))?;
    assert_eq!(parsed.uri, address);
    assert_eq!(parsed.display, address);
    return Ok(());
}

/// What: Checks both desktop file URI syntax and raw paths containing reserved characters.
/// Why: A raw `#` would otherwise become a fragment and a literal percent may look like an escape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(fileURLToPath(parse(filename).uri), filename);
/// ```
#[test]
fn file_path_round_trips_through_desktop_uri() -> anyhow::Result<()> {
    let fixture = tempfile::tempdir()?;
    let path = fixture.path().join("a # b% 雪.htm");
    fs::write(&path, "<title>Fixture</title>")?;
    let parsed = target::parse(path.as_os_str())?;
    let recovered = gio::File::for_uri(&parsed.uri).path();
    assert_eq!(recovered.as_deref(), Some(path.as_path()));
    let from_uri = target::parse(OsStr::new(&parsed.uri))?;
    assert_eq!(from_uri.uri, parsed.uri);
    assert!(target::parse(OsStr::new("ftp://example.test/path")).is_err());
    assert!(target::parse(fixture.path().as_os_str()).is_err());
    return Ok(());
}

/// What: Presents one SQLite group next to one legacy profile belonging to a different channel.
/// Why: The chooser must avoid duplicate aliases and route each name only to its own installation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(profilesForFirefox(catalog, release), [work, personal]);
/// assert.deepEqual(profilesForFirefox(catalog, esr), [legacy]);
/// ```
#[test]
fn firefox_sqlite_and_legacy_profiles_keep_channel_identity() -> anyhow::Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    for directory in ["release", "esr", "work", "personal", "legacy", "Profile Groups"] {
        fs::create_dir(root.join(directory))?;
    }
    let release = root.join("release");
    let esr = root.join("esr");
    let executable = "firefox";
    fs::write(release.join(executable), "")?;
    fs::write(esr.join(executable), "")?;
    for (name, platform) in [("work", &release), ("legacy", &esr)] {
        fs::write(
            root.join(name).join("compatibility.ini"),
            format!("[Compatibility]\nLastPlatformDir={}\n", platform.display()),
        )?;
    }
    fs::write(
        root.join("profiles.ini"),
        "[Profile0]\nName=Group anchor\nIsRelative=1\nPath=work\nStoreID=fixture\n\
         [Profile1]\nName=Legacy\nIsRelative=1\nPath=legacy\n",
    )?;
    let database = rusqlite::Connection::open(root.join("Profile Groups/fixture.sqlite"))?;
    database.execute_batch(
        "CREATE TABLE Profiles (name TEXT, path TEXT);\
         INSERT INTO Profiles VALUES ('Work', 'work'), ('Personal', 'personal'), ('Stale', 'missing');",
    )?;

    let found = profiles::firefox_profiles_at(root);
    let release_profiles = profiles::profiles_for_firefox(&found, &release.join(executable), false);
    let esr_profiles = profiles::profiles_for_firefox(&found, &esr.join(executable), false);
    let release_names: Vec<&str> = release_profiles.iter().map(|profile| profile.name.as_str()).collect();
    assert_eq!(release_names, ["Personal", "Work"]);
    let esr_names: Vec<&str> = esr_profiles.iter().map(|profile| profile.name.as_str()).collect();
    assert_eq!(esr_names, ["Legacy"]);
    assert_eq!(esr_profiles[0].arguments, [OsStr::new("--profile"), root.join("legacy").as_os_str(), OsStr::new("--new-tab")]);
    return Ok(());
}

/// What: Creates one normal profile, one omitted profile, and an absent profile in Helium metadata.
/// Why: The launch command needs the directory key while the picker shows the human-readable name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(heliumProfiles(root), [{ name: "Work", args: [rootArg, profileArg] }]);
/// ```
#[test]
fn helium_uses_the_directory_key_and_ignores_stale_rows() -> anyhow::Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    for directory in ["Profile 7", "Omitted"] {
        fs::create_dir(root.join(directory))?;
    }
    fs::write(root.join("Local State"), r#"{"profile":{"info_cache":{
        "Profile 7":{"name":"Work"},
        "Omitted":{"name":"Hidden","is_omitted":true},
        "Deleted":{"name":"Gone"},
        "../..":{"name":"Outside"}
    }}}"#)?;
    let found = profiles::helium_profiles(root);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "Work");
    assert_eq!(found[0].arguments[1], "--profile-directory=Profile 7");
    return Ok(());
}
