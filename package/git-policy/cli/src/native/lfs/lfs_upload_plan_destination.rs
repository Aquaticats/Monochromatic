//! What:
//!  Where an upload goes:
//! what `git lfs push` takes as its destination,
//! and the server the upload is connected to.
//! Why:
//!  A destination the caller typed may be a path rather than a remote name,
//! and `git-lfs` accepts only a remote name or a URL;
//! the server an upload contacts is not the push's destination,
//! but the endpoint `git-lfs` itself picks from the configuration,
//! in git-lfs's own order of keys and after the URL alias tables,
//! so a stop that held a connection names the server that did not answer.
//! The plan and its arguments both read these,
//! and they live here so that each file passes the package's length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const destination = lfsDestination(remote, directory, configured); const server = lfsServerOf(remote, all);
//! ```

/// The host a URL names,
/// with any userinfo left out.
use crate::lfs_output_mask::host_of;
/// The one test of a colon,
/// which the destination of an upload reads too.
use crate::lfs_upload_refspecs::is_colon_byte;
/// One setting of the configuration the plan reads.
use crate::lfs_upload_settings::setting;
/// Owned OS text for arguments.
use std::ffi::{
    OsStr,
    OsString,
};
/// Locations.
use std::path::Path;
/// `file://` and percent-encoding for a path,
/// where `git-lfs` accepts only a remote name or an URL.
fn file_url(absolute: &Path) -> OsString {
    let mut url: Vec<u8> = b"file://".to_vec();
    for byte in absolute.as_os_str().as_encoded_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(byte) {
            url.push(*byte);
        } else {
            url.extend_from_slice(format!("%{byte:02X}").as_bytes());
        }
    }
    return <OsString as std::os::unix::ffi::OsStringExt>::from_vec(url);
}

/// What:
///  Turn a destination typed in place of a remote name into what `git lfs push` accepts.
/// A remote name and an URL stay as
///  they are;
/// a path (it starts with `/`,
/// `./` or `../`,
/// or names an existing file or directory relative to `directory`
///  and is not a configured remote) becomes a percent-encoded `file://` URL.
/// Why:
///   Git pushes to a path directly,
/// and the old hook flow handed `git-lfs` the URL Git had made of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isPath(remote) ? pathToFileURL(resolve(directory, remote)) : remote
/// ```
pub(crate) fn lfs_destination(
    remote: &OsStr,
    directory: &Path,
    configured: bool,
) -> OsString {
    let bytes: &[u8] = remote.as_encoded_bytes();
    let mut has_scheme: bool = false;
    for window in bytes.windows(3) {
        if window == b"://" {
            has_scheme = true;
            break;
        }
    }
    let scp_like: bool = match bytes.iter().position(is_colon_byte) {
        Some(colon) => !bytes[..colon].contains(&b'/'),
        None => false,
    };
    let explicit_path: bool =
        bytes.starts_with(b"/") || bytes.starts_with(b"./") || bytes.starts_with(b"../");
    if configured || has_scheme || (scp_like && !explicit_path) {
        return remote.to_owned();
    }
    let candidate = directory.join(Path::new(remote));
    if !explicit_path && !candidate.exists() {
        return remote.to_owned();
    }
    return match std::path::absolute(&candidate) {
        Ok(absolute) => file_url(absolute.as_path()),
        Err(_) => remote.to_owned(),
    };
}

/// What:
///  The server one upload is connected to,
///  as its host and port with any userinfo left out,
///  in the order git-lfs 3.8.1 itself picks that server for an upload.
/// Why:
///   A stop of an upload says which server held the connection,
///  and the server of an upload is not the push's destination:
///  git-lfs's own order,
/// read at `lfsapi/endpoint_finder.go:93-140` (`getEndpoint`,
/// `RemoteEndpoint`)
///  and `:190-201` (`GitRemoteURL`),
/// is
///  `lfs.pushurl`,
///  then `lfs.url`,
///  then the named remote's `lfspushurl`,
/// `lfsurl`,
/// `pushurl` and `url`,
///  then the same four of `origin` when the named remote answered nothing.
///  Measured against git-lfs 3.8.1:
/// with `lfs.pushurl` and `lfs.url` both set the batch request went to `lfs.pushurl`,
///  with `lfs.url` and `remote.origin.lfsurl` both set it went to `lfs.url`,
///  and with only `remote.origin.lfsurl` set git-lfs sent nothing there.
///  A destination that is a URL is not an endpoint of its own:
///  `git lfs push` takes a remote name and not a URL (`commands/command_push.go`),
///  so the configuration answers for the upload and the URL only names where Git pushes.
///  A key holding no URL (a remote name that is no remote,
///  a path) answers nothing,
///  so the stop falls back to the words of the program that saw the server.
///  Every endpoint is rewritten through the URL alias tables before its host is read,
///  as git-lfs rewrites it before parsing (`NewEndpoint` calls it,
/// `lfsapi/endpoint_finder.go:234-235`;
///  `ReplaceUrlAlias`,
/// `:345-357`):
///  `url.<base>.pushInsteadOf` first for an upload,
///  `url.<base>.insteadOf` after it,
///  so the stop names the host the upload contacted rather than the alias it was configured with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsServerOf(remote: string, all: Settings): string | undefined
/// ```
pub fn lfs_server_of(
    remote: &OsStr,
    all: &[(String, String)],
) -> Option<String> {
    let text: String = remote.to_string_lossy().into_owned();
    let push_aliases: Vec<(&str, &str)> = url_aliases(all, true);
    let fetch_aliases: Vec<(&str, &str)> = url_aliases(all, false);
    let endpoint_host = |value: &str| -> Option<String> {
        let rewritten: Option<String> =
            replace_url_alias(value, push_aliases.as_slice()).or_else(|| {
                return replace_url_alias(value, fetch_aliases.as_slice());
            });
        return host_of(rewritten.as_deref().unwrap_or(value));
    };
    // The two global keys answer first, whatever the remote's own keys say (git-lfs `getEndpoint`, lines 99 and 104).
    for key in ["lfs.pushurl", "lfs.url"] {
        if let Some(value) = setting(all, key)
            && let Some(host) = endpoint_host(value)
        {
            return Some(host);
        }
    }
    // A named remote other than `origin` answers with its own keys, and `origin` answers behind it when they are silent.
    let mut names: Vec<&str> = Vec::new();
    if !text.is_empty() && text != "origin" {
        names.push(text.as_str());
    }
    names.push("origin");
    for name in names {
        for key in [
            format!("remote.{name}.lfspushurl"),
            format!("remote.{name}.lfsurl"),
        ] {
            if let Some(value) = setting(all, key.as_str())
                && let Some(host) = endpoint_host(value)
            {
                return Some(host);
            }
        }
        // `git-lfs` reads one value per key and the last one written wins (`config/git_fetcher.go:121-128`).
        for value in [
            setting(all, &format!("remote.{name}.pushurl")),
            setting(all, &format!("remote.{name}.url")),
        ] {
            if let Some(host) = value.and_then(|endpoint| {
                return endpoint_host(endpoint);
            }) {
                return Some(host);
            }
        }
    }
    return None;
}

/// What:
///  The alias table git-lfs applies to an endpoint of one operation,
///  as `(alias, replacement)` pairs.
/// Why:
///  `url.<base>.pushInsteadOf` and `url.<base>.insteadOf` rewrite a URL before git-lfs parses it
///  (`lfsapi/endpoint_finder.go:234-235`),
///  and an upload reads the push table first and the fetch table after it (`ReplaceUrlAlias`,
/// `:345-357`).
///  A key's base is the replacement and its value is the alias;
///  a later value of one alias wins,
/// as it does in git-lfs's own table (`storeAlias`,
/// `:399-407`).
///  An empty alias is left out because git-lfs never applies one:
///  it keeps the empty value and rewrites only when the longest matching alias is longer than nothing
///  (`replaceUrlAlias`,
/// `:361-378`),
///  so the endpoint it contacts is the unrewritten URL;
///  Git does rewrite it (`git remote get-url --push` shows the replacement),
/// and this table follows git-lfs because
///  git-lfs is the program that makes the request.
///  An empty base is kept,
/// because it is a replacement of nothing and git-lfs applies it:
///  the endpoint is the URL with the alias taken off its front,
///  and the host read from it is the one inside whatever follows the alias
///  (measured with git-lfs 3.8.1:
/// `lfs.url=https://alias.invalid/https://127.0.0.1:39506/lfs` under an empty base
///  `https://alias.invalid/` gave `Endpoint=https://127.0.0.1:39506/lfs`),
///  so leaving it out would name the alias's host,
/// which git-lfs never contacts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function urlAliases(all: Settings, push: boolean): Array<[alias: string, replacement: string]>
/// ```
fn url_aliases<'a>(
    all: &'a [(String, String)],
    push: bool,
) -> Vec<(&'a str, &'a str)> {
    let suffix: &str = if push { ".pushinsteadof" } else { ".insteadof" };
    let mut aliases: Vec<(&'a str, &'a str)> = Vec::new();
    // The last entry of a key wins, so the entries are read backwards and one alias is kept once.
    for (key, value) in all.iter().rev() {
        let Some(base) = key.strip_prefix("url.").and_then(|rest| {
            return rest.strip_suffix(suffix);
        }) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        if aliases.iter().any(|(alias, _replacement)| {
            return *alias == value.as_str();
        }) {
            continue;
        }
        aliases.push((value.as_str(), base));
    }
    return aliases;
}

/// What:
///  `rawurl` with the longest matching alias's replacement in place of its prefix,
///  or nothing when no alias matches.
/// Why:
///  git-lfs chooses the longest alias that prefixes the URL (`replaceUrlAlias`,
/// `:361-378`),
///  so a narrower rule can rewrite a subpath of a broader one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceUrlAlias(rawurl: string, aliases: Array<[alias: string, replacement: string]>): string | undefined
/// ```
fn replace_url_alias(
    rawurl: &str,
    aliases: &[(&str, &str)],
) -> Option<String> {
    let mut longest: Option<(&str, &str)> = None;
    for (alias, replacement) in aliases {
        if !rawurl.starts_with(alias) {
            continue;
        }
        if longest.is_none_or(|(found, _replacement)| {
            return alias.len() > found.len();
        }) {
            longest = Some((alias, replacement));
        }
    }
    return longest.map(|(alias, replacement)| {
        return format!("{replacement}{}", &rawurl[alias.len()..]);
    });
}

/// The remotes the configuration defines (a `url` or a `pushurl`).
pub(crate) fn configured_remotes(all: &[(String, String)]) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for (key, _) in all {
        let Some(rest) = key.strip_prefix("remote.") else {
            continue;
        };
        let Some((name, variable)) = rest.rsplit_once('.') else {
            continue;
        };
        if variable == "url" || variable == "pushurl" {
            names.push(name);
        }
    }
    return names;
}

/// Whether any `url.<base>.pushInsteadOf` rewrite is configured,
/// which can send the push elsewhere than the fetch.
pub(crate) fn has_push_rewrite(all: &[(String, String)]) -> bool {
    for (key, _) in all {
        if key.starts_with("url.") && key.ends_with(".pushinsteadof") {
            return true;
        }
    }
    return false;
}
