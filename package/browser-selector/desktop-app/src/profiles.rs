//! Reads the profile metadata used by the supported native browser launchers.

/// Imports the file, path, and native argument types used by browser profile stores.
use std::{collections::BTreeMap, env, ffi::OsString, fs, path::{Component, Path, PathBuf}};

/// Imports GLib's INI reader for Firefox's profile and compatibility files.
use gio::glib::{self, KeyFile, KeyFileFlags};

/// Imports SQLite's connection and read-only flag for Firefox Selectable Profiles.
use rusqlite::{Connection, OpenFlags};

/// What: Describes a named profile's browser arguments before adding the target.
/// Why: The picker binds a visible name to exact arguments without a shell command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Profile = { name: string; arguments: NativeString[] };
/// ```
pub(crate) struct Profile {
    /// Profile name from the browser's metadata.
    pub(crate) name: String,
    /// Existing directory verified again when launching the profile.
    pub(crate) directory: PathBuf,
    /// Individual native arguments that select the profile.
    pub(crate) arguments: Vec<OsString>,
}

/// What: Holds a Firefox profile plus the browser installation last observed using it.
/// Why: Native Firefox variants can share a profile catalog while running separate instances.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FirefoxProfile = { name: string; path: Path; platform?: Path };
/// ```
pub(crate) struct FirefoxProfile {
    /// Name shown in the picker.
    name: String,
    /// Canonical directory passed to `--profile`.
    path: PathBuf,
    /// Canonical application directory recorded by Firefox.
    platform: Option<PathBuf>,
}

/// What: Identifies the installation that last opened a profile, if recorded.
/// Why: Reading `compatibility.ini` avoids confusing ESR, Developer Edition, and release Firefox.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastPlatform(profile: Path): Path | undefined;
/// ```
fn last_platform(profile: &Path) -> Option<PathBuf> {
    // Load the existing compatibility file. A new profile may have no such file.
    let metadata = KeyFile::new();
    metadata.load_from_file(profile.join("compatibility.ini"), KeyFileFlags::NONE).ok()?;
    let path = metadata.string("Compatibility", "LastPlatformDir").ok()?;
    let recorded_path = Path::new(path.as_str());
    let canonical = fs::canonicalize(recorded_path).unwrap_or_else(|_| return recorded_path.to_path_buf());
    return Some(canonical);
}

/// What: Reads profile names and paths from a Firefox Selectable Profiles database.
/// Why: The toolkit profile entry only represents one member of that group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupProfiles(path: Path): [name: string, path: string][];
/// ```
fn group_profiles(path: &Path) -> rusqlite::Result<Vec<(String, String)>> {
    // Open in read-only mode so a missing store stays missing.
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = connection.prepare("SELECT name, path FROM Profiles")?;
    // The closure reads both columns as owned strings before advancing to the next row.
    let rows = statement.query_map([], |row| return Ok((row.get(0)?, row.get(1)?)))?;
    return rows.collect();
}

/// What: Resolves an existing profile directory from a path relative to Firefox's data root.
/// Why: Canonical paths deduplicate the toolkit and SQLite entries for a group member.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function profileDirectory(root: Path, storedPath: string): Path | undefined;
/// ```
fn profile_directory(root: &Path, stored_path: &str) -> Option<PathBuf> {
    if stored_path.is_empty() {
        return None;
    }
    // Joining an absolute path preserves it. Firefox also supports profiles outside the data root.
    let path = fs::canonicalize(root.join(stored_path)).ok()?;
    if path.is_dir() {
        return Some(path);
    }
    return None;
}

/// What: Tests whether a stored name identifies a single native path component.
/// Why: Group identifiers and Chromium directory keys are names within a known directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function singleComponent(name: string): boolean;
/// ```
fn single_component(name: &str) -> bool {
    let path = Path::new(name);
    let mut parts = path.components();
    // The matched component must equal the original to reject alternate slash spellings.
    return matches!(parts.next(), Some(Component::Normal(part)) if part == path.as_os_str())
        && parts.next().is_none();
}

/// What: Reads a Firefox profile catalog and overlays its Selectable Profiles names and paths.
/// Why: The picker needs current profiles from either storage format without duplicate paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firefoxProfilesAt(root: Path): FirefoxProfile[];
/// ```
pub(crate) fn firefox_profiles_at(root: &Path) -> Vec<FirefoxProfile> {
    let index = KeyFile::new();
    // A missing legacy catalog can coexist with readable modern profile databases.
    let _ = index.load_from_file(root.join("profiles.ini"), KeyFileFlags::NONE);
    // A path-keyed map retains just one label for each real profile directory.
    let mut profiles = BTreeMap::new();
    let mut groups = BTreeMap::new();
    let group_root = root.join("Profile Groups");
    for section in index.groups().iter() {
        let group = section.as_str();
        if !group.starts_with("Profile") {
            continue;
        }
        let Ok(stored_path) = index.string(group, "Path") else { continue; };
        let anchor = profile_directory(root, stored_path.as_str());
        if let Ok(store) = index.string(group, "StoreID")
            && single_component(store.as_str())
        {
            // The anchor's last platform also gives an installation for a new member with no compatibility file yet.
            let parent_platform = anchor.as_deref().and_then(last_platform);
            let database = group_root.join(format!("{store}.sqlite"));
            groups.insert(database, parent_platform);
        }
        if let Some(path) = anchor
            && let Ok(name) = index.string(group, "Name")
        {
            let platform = last_platform(&path);
            profiles.insert(path.clone(), FirefoxProfile { name: name.to_string(), platform, path });
        }
    }
    // Firefox can leave selectable group metadata with no StoreID in the legacy catalog.
    if let Ok(entries) = fs::read_dir(&group_root) {
        for directory_entry in entries.flatten() {
            let path = directory_entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("sqlite")) && path.is_file() {
                groups.entry(path).or_insert(None);
            }
        }
    }

    for (database, anchor_platform) in groups {
        let Ok(rows) = group_profiles(&database) else { continue; };
        let mut members = Vec::new();
        let mut platforms = std::collections::BTreeSet::new();
        for (name, stored_path) in rows {
            if let Some(path) = profile_directory(root, &stored_path) {
                let platform = last_platform(&path);
                if let Some(found) = &platform {
                    platforms.insert(found.clone());
                }
                members.push(FirefoxProfile { name, platform, path });
            }
        }
        // With no anchor, use a shared installation only if the evidence agrees across every known member.
        let fallback = anchor_platform.or_else(|| {
            if platforms.len() == 1 {
                return platforms.pop_first();
            }
            return None;
        });
        for mut member in members {
            if member.platform.is_none() {
                member.platform = fallback.clone();
            }
            profiles.insert(member.path.clone(), member);
        }
    }
    return profiles.into_values().collect();
}

/// What: Reads the native Firefox catalogs from the existing application-data locations.
/// Why: Older Firefox catalogs can stay under the home directory after newer versions start using XDG.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firefoxProfiles(): FirefoxProfile[];
/// ```
pub(crate) fn firefox_profiles() -> Vec<FirefoxProfile> {
    let legacy = glib::home_dir().join(".mozilla/firefox");
    let current = glib::user_config_dir().join("mozilla/firefox");
    let mut profiles = firefox_profiles_at(&legacy);
    for profile in firefox_profiles_at(&current) {
        if !profiles.iter().any(|stored| return stored.path == profile.path) {
            profiles.push(profile);
        }
    }
    return profiles;
}

/// What: Builds Firefox profile arguments for one installed browser executable.
/// Why: Firefox remotes by both profile path and channel, and channels can have incompatible versions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function profilesForFirefox(profiles: FirefoxProfile[], executable: Path, onlyInstallation: boolean): Profile[];
/// ```
pub(crate) fn profiles_for_firefox(
    profiles: &[FirefoxProfile],
    executable: &Path,
    include_unassigned: bool,
) -> Vec<Profile> {
    let canonical = fs::canonicalize(executable).ok();
    let installation = canonical.as_deref().and_then(Path::parent);
    let mut selected = Vec::new();
    for profile in profiles {
        if let Some(platform) = profile.platform.as_deref() {
            if installation != Some(platform) {
                continue;
            }
        } else if !include_unassigned {
            continue;
        }

        // `--new-tab` receives the URI appended by the common launcher.
        let arguments = vec!["--profile".into(), profile.path.clone().into_os_string(), "--new-tab".into()];
        selected.push(Profile { name: profile.name.clone(), directory: profile.path.clone(), arguments });
    }
    return selected;
}

/// What: Finds Helium's native data root using Chromium's environment overrides.
/// Why: The launch command must identify the same user-data root as the profile inventory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function heliumRoot(): Path;
/// ```
pub(crate) fn helium_root() -> PathBuf {
    if let Some(directory) = env::var_os("CHROME_USER_DATA_DIR")
        && !directory.is_empty()
    {
        return PathBuf::from(directory);
    }
    if let Some(directory) = env::var_os("CHROME_CONFIG_HOME")
        && !directory.is_empty()
    {
        return PathBuf::from(directory).join("net.imput.helium");
    }
    return glib::user_config_dir().join("net.imput.helium");
}

/// What: Reads display names and directory keys from Helium's `Local State` file.
/// Why: Passing a name in place of the directory key can create a new Chromium profile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function heliumProfiles(root: Path): Profile[];
/// ```
pub(crate) fn helium_profiles(root: &Path) -> Vec<Profile> {
    let Ok(canonical_root) = fs::canonicalize(root) else { return Vec::new(); };
    let Ok(content) = fs::read(canonical_root.join("Local State")) else { return Vec::new(); };
    let Ok(state) = serde_json::from_slice::<serde_json::Value>(&content) else { return Vec::new(); };
    let Some(entries) = state.pointer("/profile/info_cache").and_then(serde_json::Value::as_object) else {
        return Vec::new();
    };

    let mut profiles = Vec::new();
    // Supply the exact root explicitly so Helium cannot resolve the key under a different root.
    let mut data_argument = OsString::from("--user-data-dir=");
    data_argument.push(&canonical_root);
    for (key, details) in entries {
        if !single_component(key)
            || details["is_omitted"].as_bool() == Some(true)
            || matches!(key.as_str(), "Guest Profile" | "System Profile")
        {
            continue;
        }
        let directory = canonical_root.join(key);
        if !directory.is_dir() {
            continue;
        }
        let name = details["name"].as_str().filter(|text| return !text.is_empty()).unwrap_or(key);
        let arguments = vec![data_argument.clone(), format!("--profile-directory={key}").into()];
        profiles.push(Profile { name: name.to_owned(), directory, arguments });
    }
    return profiles;
}
