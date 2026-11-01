//! First-run screen copy,
//!  ported from `FirstRunState.kt`.
//!
//! The three first-run screens explain an open source that cannot be read or has
//! no audio.
//!  Every string and action order lives in `first_run_copy`,
//!  so the pane
//! and the tests read the same source.
//!  No Slint or I/O is used.

/// What:
///      `pub const ALLOW_ACCESS_LABEL` names the filled action that asks for music access.
/// Why:
///      The copy and the pane both refer to this label,
///  so one constant keeps them in step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ALLOW_ACCESS_LABEL = "Allow access";
/// ```
pub const ALLOW_ACCESS_LABEL: &str = "Allow access";

/// What:
///      `pub const OPEN_FOLDER_LABEL` names the action that chooses another folder.
/// Why:
///      Two states offer this action,
///  and the pane maps the label to one callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OPEN_FOLDER_LABEL = "Open a folder";
/// ```
pub const OPEN_FOLDER_LABEL: &str = "Open a folder";

/// What:
///      `pub const SETTINGS_LABEL` names the in-app Settings action.
/// Why:
///      Settings is in-app Settings,
///  not Android's permission page,
///  so its label stays separate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SETTINGS_LABEL = "Settings";
/// ```
pub const SETTINGS_LABEL: &str = "Settings";

/// What:
///      `pub const ANALYSIS_EXPLANATION` holds the true-peak sentence shown under the declined state's actions.
/// Why:
///      Analysis runs automatically,
///  so the explanation states that it runs without offering a choice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ANALYSIS_EXPLANATION = "True peak is measured automatically for every audio file. ...";
/// ```
pub const ANALYSIS_EXPLANATION: &str = concat!(
    "True peak is measured automatically for every audio file. ",
    "Analysis uses CPU and battery; playback remains available while it runs.",
);

/// What:
///      `pub enum FirstRunState` declares the first-run screens.
/// Why:
///      Each screen explains an open source that cannot be read or has no audio,
///  and the set is closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FirstRunState = "DECLINED" | "SYSTEM_NO_AUDIO" | "FOLDER_NO_AUDIO";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstRunState {
    /// The device music library is open,
    ///  but access to music on this device was not granted.
    Declined,
    /// The device music library was read completely and holds no audio.
    SystemNoAudio,
    /// The chosen folder was read completely and holds no supported audio file.
    FolderNoAudio,
    /// No folder is open, so the desktop asks the user to choose one.
    NoFolderOpen,
    /// The opened folder is still being read, so its tracks are not known yet.
    Reading,
}

/// Methods of `FirstRunState`.
impl FirstRunState {
    /// What:
    ///      `pub const ALL` lists the five screens in declaration order.
    /// Why:
    ///      Tests walk every state,
    ///  and this list keeps that walk exhaustive.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ALL: readonly FirstRunState[] = ["DECLINED", "SYSTEM_NO_AUDIO", "FOLDER_NO_AUDIO"];
    /// ```
    pub const ALL: [FirstRunState; 5] = [
        FirstRunState::Declined,
        FirstRunState::SystemNoAudio,
        FirstRunState::FolderNoAudio,
        FirstRunState::NoFolderOpen,
        FirstRunState::Reading,
    ];
}

/// What:
///      `pub struct FirstRunCopy` is the immutable copy for one first-run state.
/// Why:
///      The pane draws exactly these strings and actions,
///  and tests compare them directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FirstRunCopy = { title: string; body: string; primaryAction: string | null;
///   secondaryActions: string[]; explainsAnalysis: boolean };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstRunCopy {
    /// Headline that names the source the state is about.
    pub title: String,
    /// Explanation of the open source and what the actions do.
    pub body: String,
    /// Label of the filled action,
    ///  or `None` when the state draws no filled action.
    pub primary_action: Option<&'static str>,
    /// Labels of the outlined actions,
    ///  in the order they are drawn.
    pub secondary_actions: Vec<&'static str>,
    /// True when the true-peak explanation belongs below the actions.
    pub explains_analysis: bool,
}

/// What:
///      `pub enum FirstRunError` is the failure of `first_run_copy`.
/// Why:
///      A headline without a folder name would be wrong,
///  so the folder state refuses to build one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FirstRunError extends Error { /* "The folder no-audio state needs a non-blank folder name." */ }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstRunError {
    /// The folder no-audio state received no folder name,
    ///  or one that is blank.
    BlankFolderName,
}

/// Display text for `FirstRunError`.
impl std::fmt::Display for FirstRunError {
    /// What:
    ///      `fmt` writes the message for a missing or blank folder name.
    /// Why:
    ///      Callers and the tests compare this exact text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return "The folder no-audio state needs a non-blank folder name."; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::BlankFolderName => {
                formatter.write_str("The folder no-audio state needs a non-blank folder name.")
            }
        };
    }
}

/// Marks `FirstRunError` as a standard error.
impl std::error::Error for FirstRunError {}

/// What:
///      `pub fn first_run_copy` returns the copy for one first-run state.
/// Why:
///      One pure function owns every string,
///  so the pane and the tests read the same source.
///      `folder_name` is required only by the folder no-audio state,
///  and is ignored by the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstRunCopy(state: FirstRunState, folderName: string | null): FirstRunCopy { ... }
/// ```
///
/// # Errors
///
/// Returns `FirstRunError::BlankFolderName` when `state` is the folder no-audio state and
/// `folder_name` is absent or blank.
pub fn first_run_copy(
    state: FirstRunState,
    folder_name: Option<&str>,
) -> Result<FirstRunCopy, FirstRunError> {
    return match state {
        FirstRunState::Declined => Ok(FirstRunCopy {
            title: "The device music library can't be read".to_owned(),
            body: concat!(
                "It is open as your library, but access to music on this device was not granted. ",
                "Allow access, or open a folder instead.",
            )
            .to_owned(),
            primary_action: Some(ALLOW_ACCESS_LABEL),
            secondary_actions: vec![OPEN_FOLDER_LABEL, SETTINGS_LABEL],
            explains_analysis: true,
        }),
        FirstRunState::SystemNoAudio => Ok(FirstRunCopy {
            title: "No audio found in the device music library".to_owned(),
            body: concat!(
                "The device music library has no audio files. ",
                "You can open a different folder instead.",
            )
            .to_owned(),
            primary_action: Some(OPEN_FOLDER_LABEL),
            secondary_actions: vec![SETTINGS_LABEL],
            explains_analysis: false,
        }),
        FirstRunState::FolderNoAudio => folder_no_audio_copy(folder_name),
        FirstRunState::NoFolderOpen => Ok(no_folder_open_copy()),
        FirstRunState::Reading => Ok(reading_copy()),
    };
}

/// What:
///      `fn reading_copy() -> FirstRunCopy`.
///  The copy for a folder that is still being read: says the library is being looked through and that nothing is wrong.
/// Why:
///       While the folder walk runs the window used to say "No folder is open" (the polish review finding EM1). The words
///  follow Search's "Still looking through your library" (D69) and promise nothing the list does not yet do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readingCopy = (): FirstRunCopy => ({ title: "Reading your library", secondaryActions: ["Settings"] });
/// ```
fn reading_copy() -> FirstRunCopy {
    return FirstRunCopy {
        title: "Reading your library".to_owned(),
        body: concat!(
            "Looking through your music for audio files. ",
            "A large library takes a little while, and the list appears here when it is done.",
        )
        .to_owned(),
        primary_action: None,
        secondary_actions: vec![SETTINGS_LABEL],
        explains_analysis: false,
    };
}

/// What:
///      `folder_no_audio_copy` builds the folder no-audio copy,
///  naming the chosen folder in the headline.
/// Why:
///      The headline must name the folder that was checked,
///  so a usable name is required here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// case "FOLDER_NO_AUDIO": return { title: `No audio found in ${folderName}`, ... };
/// ```
/// What:
///      `fn no_folder_open_copy() -> FirstRunCopy`.
///  The desktop copy for a start with no folder open: says so and offers `Open a folder`.
/// Why:
///       The desktop has no device library to open, so the only way to start is to choose a folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noFolderOpenCopy = (): FirstRunCopy => ({ title: "No folder is open", primaryAction: "Open a folder", secondaryActions: ["Settings"] });
/// ```
fn no_folder_open_copy() -> FirstRunCopy {
    return FirstRunCopy {
        title: "No folder is open".to_owned(),
        body: concat!(
            "Open a folder to start. The player lists the audio files in it and in its subfolders.",
        )
        .to_owned(),
        primary_action: Some(OPEN_FOLDER_LABEL),
        secondary_actions: vec![SETTINGS_LABEL],
        explains_analysis: false,
    };
}

fn folder_no_audio_copy(folder_name: Option<&str>) -> Result<FirstRunCopy, FirstRunError> {
    let name = non_blank_folder_name(folder_name)?;
    return Ok(FirstRunCopy {
        // The name sits on its own line, so the headline wraps between "found in" and the name, never inside a name that
        // contains a hyphen (the polish review finding EM4).
        title: format!("No audio found in\n{name}"),
        body: concat!(
            "This folder and its subfolders have no audio files the player can play. ",
            "Open a different folder to try again.",
        )
        .to_owned(),
        primary_action: Some(OPEN_FOLDER_LABEL),
        secondary_actions: vec![SETTINGS_LABEL],
        explains_analysis: false,
    });
}

/// What:
///      `non_blank_folder_name` returns the folder name when it is present and not blank.
/// Why:
///      A missing or blank name would print an empty headline,
///  so the function refuses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nonBlankFolderName(folderName: string | null): string {
///   if (folderName === null || folderName.trim() === "") throw new Error("Folder name is required");
///   return folderName;
/// }
/// ```
fn non_blank_folder_name(folder_name: Option<&str>) -> Result<&str, FirstRunError> {
    return match folder_name {
        Some(name) if !name.trim().is_empty() => Ok(name),
        _ => Err(FirstRunError::BlankFolderName),
    };
}

/// Unit tests in `first_run_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "first_run_tests.rs"]
mod tests;
