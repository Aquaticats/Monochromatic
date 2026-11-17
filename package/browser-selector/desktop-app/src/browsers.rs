//! Discovers registered browsers and sends the target to the selected launcher.

/// Imports containers, native paths, and the direct process launcher.
use std::{collections::BTreeMap, path::PathBuf, process::Command};

/// Imports the extension methods GIO adds to registered applications.
use gio::prelude::AppInfoExt;

/// Imports profile readers for the native browser installations that advertise profile choices.
use crate::profiles;

/// Desktop file ID used to exclude the selector from its own list of choices.
const SELECTOR_ID: &str = "monochromatic.browser-selector.desktop";

/// What: The selector can dispatch through a desktop entry or a browser with profile arguments.
/// Why: Plain choices preserve native launcher behavior while named choices select an exact profile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Launch = { app: AppInfo } | { program: Path; arguments: NativeString[] };
/// ```
enum Launch {
    /// Run the desktop entry through GIO using its ordinary behavior.
    Desktop(gio::AppInfo),
    /// Pass the profile and target as individual arguments to a supported native executable.
    Profile { program: PathBuf, profile: profiles::Profile },
}

/// What: The row label and its launch operation stay coupled as one record.
/// Why: The UI can choose an index without reconstructing destinations from labels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Choice = { label: string; launch: Launch };
/// ```
pub(crate) struct Choice {
    /// Browser name, with profile name for a named choice.
    pub(crate) label: String,
    /// Operation this choice will perform.
    launch: Launch,
}

impl Choice {
    /// What: Opens one URI in the choice's application or profile.
    /// Why: Launching a specific browser bypasses the selector's installed default association.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// choice.open(uri);
    /// ```
    pub(crate) fn open(&self, uri: &str) -> anyhow::Result<()> {
        // Each branch attaches the target after routing to a specific installed browser.
        match &self.launch {
            Launch::Desktop(app) => {
                app.launch_uris(&[uri], gio::AppLaunchContext::NONE)?;
            }
            Launch::Profile { program, profile } => {
                anyhow::ensure!(profile.directory.is_dir(), "The profile directory is no longer available.");
                Command::new(program).args(&profile.arguments).arg(uri).spawn()?;
            }
        }
        return Ok(());
    }
}

/// What: Recognises the registered Firefox channels with automatic profile support.
/// Why: Other registered browsers remain available through their own desktop entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isFirefox(id: string): boolean;
/// ```
fn is_firefox(id: &str) -> bool {
    return matches!(id, "firefox.desktop" | "firefox-esr.desktop" | "firefox-developer-edition.desktop");
}

/// What: Resolves the executable when the desktop command consists of a program and one URL field.
/// Why: A profile launch may change the browser arguments but must preserve the launcher's semantics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function simpleProgram(app: AppInfo): Path | undefined;
/// ```
fn simple_program(app: &gio::AppInfo) -> Option<PathBuf> {
    let command_line = app.commandline()?;
    let arguments = gio::glib::shell_parse_argv(command_line).ok()?;
    if arguments.len() == 2 && (arguments[1] == "%u" || arguments[1] == "%U") {
        return gio::glib::find_program_in_path(app.executable()?);
    }
    // Apps with package-specific wrapper arguments remain available through GIO.
    return None;
}

/// What: Reads web scheme handlers and adds profile rows for the supported native launchers.
/// Why: HTML associations can include editors, while web scheme handlers identify browser choices.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function discover(): Choice[];
/// ```
pub(crate) fn discover() -> Vec<Choice> {
    let mut browsers = BTreeMap::new();
    for scheme in ["x-scheme-handler/http", "x-scheme-handler/https"] {
        for app in gio::AppInfo::all_for_type(scheme) {
            let Some(id) = app.id() else { continue; };
            if id == SELECTOR_ID || !app.should_show() || !app.supports_uris() {
                continue;
            }
            browsers.insert(id.to_string(), app);
        }
    }

    // Remember which Firefox installs have straightforward profile-capable launchers.
    let mut firefox = BTreeMap::new();
    for (id, app) in &browsers {
        if is_firefox(id) && let Some(program) = simple_program(app) {
            firefox.insert(id.clone(), program);
        }
    }
    let firefox_profiles = if firefox.is_empty() { Vec::new() } else { profiles::firefox_profiles() };

    let mut choices = Vec::new();
    for (id, app) in browsers {
        let name = app.display_name().to_string();
        if let Some(program) = firefox.get(&id) {
            for profile in profiles::profiles_for_firefox(&firefox_profiles, program, firefox.len() == 1) {
                choices.push(Choice {
                    label: format!("{name} · {}", profile.name),
                    launch: Launch::Profile { program: program.clone(), profile },
                });
            }
        } else if id == "helium.desktop" && let Some(program) = simple_program(&app) {
            for profile in profiles::helium_profiles(&profiles::helium_root()) {
                choices.push(Choice {
                    label: format!("{name} · {}", profile.name),
                    launch: Launch::Profile { program: program.clone(), profile },
                });
            }
        }
        choices.push(Choice { label: name, launch: Launch::Desktop(app) });
    }
    choices.sort_by_cached_key(|choice| return choice.label.to_lowercase());
    return choices;
}
