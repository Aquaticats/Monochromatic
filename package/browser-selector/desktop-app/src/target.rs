//! Resolves the command-line input as either a web URL or a local file.

/// Imports native command-line strings, which can hold paths that UTF-8 cannot represent.
use std::ffi::OsStr;

/// Imports GIO's file methods to turn native paths into escaped file URLs.
use gio::prelude::FileExt;

/// What: Owns the URI and the name shown in the picker.
/// Why: Browsers need a URI and the picker can show a decoded local path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { uri: string; display: string };
/// ```
pub(crate) struct Target {
    /// URI sent to the application selected by the user.
    pub(crate) uri: String,
    /// URL as supplied, or a readable local path.
    pub(crate) display: String,
}

/// What: Parses an argument borrowed from the command line.
/// Why: Browser dispatch needs a validated web URL or an existing native file URI.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse(input: NativeString): Target;
/// ```
pub(crate) fn parse(input: &OsStr) -> anyhow::Result<Target> {
    // What: Test the UTF-8 view of the argument when one exists.
    // Why: URL schemes are UTF-8, while native paths can contain arbitrary bytes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = tryDecodeUtf8(input);
    // if (text !== undefined && isWebUrl(text)) { /* URL handling */ }
    // ```
    if let Some(text) = input.to_str()
        && let Some((scheme, _)) = text.split_once("://")
        && (scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"))
    {
        // `?` returns the parsing error immediately if GIO rejects the URL.
        let parsed = gio::glib::Uri::parse(text, gio::glib::UriFlags::ENCODED)?;
        let has_host = parsed.host().is_some_and(|host| return !host.is_empty());
        anyhow::ensure!(has_host, "The URL needs a hostname.");

        // Return the success value with owned copies, preserving the URL as supplied.
        return Ok(Target { uri: text.to_owned(), display: text.to_owned() });
    }

    // GIO handles both desktop file URIs and paths supplied directly to the CLI.
    let file = gio::File::for_commandline_arg(input);
    // A missing path means GIO recognised a non-local URI. Stop before launching it.
    let path = file.path().ok_or_else(|| anyhow::anyhow!("Open an HTTP or HTTPS link, or a local file."))?;
    anyhow::ensure!(path.is_file(), "The file {} could not be opened.", path.display());

    // Return the escaped URI and preserve the familiar path for the interface.
    return Ok(Target { uri: file.uri().to_string(), display: path.display().to_string() });
}
