// What:     Unit tests for `first_run.rs`, pulled in by
//           `#[cfg(test)] #[path = "first_run_tests.rs"] mod tests;` at the bottom of `first_run.rs`.
//           Mirrors every case of the Kotlin `FirstRunStateTest` with the same inputs and expected text.
// Why:      Keep the tests beside the code. The copy is approved text, so each string is compared exactly.

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need `first_run_copy`, the state enum, the label constants, and `FirstRunError`.
use super::*;

// What:     `fn copy_of(state, folder_name) -> FirstRunCopy` returns the copy for a state the test expects to succeed.
// Why:      Every case except the rejection tests uses a valid state, so one helper keeps the tests short.
//
// In TS you'd write (pseudocode):
// ```ts
// const copyOf = (state: FirstRunState, folderName: string | null) => firstRunCopy(state, folderName);
// ```
fn copy_of(state: FirstRunState, folder_name: Option<&str>) -> FirstRunCopy {
    return first_run_copy(state, folder_name).expect("copy for a state with a non-blank folder name");
}

// What:     `#[test] fn declined_title_is_exact()` checks the declined headline word for word.
// Why:      The approved headline must not drift.
#[test]
fn declined_title_is_exact() {
    assert_eq!(
        copy_of(FirstRunState::Declined, None).title,
        "The device music library can't be read"
    );
}

// What:     `#[test] fn declined_body_is_exact()` checks the declined body word for word.
// Why:      The body names only the assessed source and the two ways forward.
#[test]
fn declined_body_is_exact() {
    assert_eq!(
        copy_of(FirstRunState::Declined, None).body,
        "It is open as your library, but access to music on this device was not granted. \
         Allow access, or open a folder instead."
    );
}

// What:     `#[test] fn declined_offers_allow_access_first()` checks the filled action and the outlined order.
// Why:      The filled button is Allow access, followed by Open a folder and Settings outlined.
#[test]
fn declined_offers_allow_access_first() {
    let copy = copy_of(FirstRunState::Declined, None);
    assert_eq!(copy.primary_action, Some("Allow access"));
    assert_eq!(copy.secondary_actions, ["Open a folder", "Settings"]);
}

// What:     `#[test] fn declined_explains_analysis()` checks that the declined state carries the explanation.
// Why:      The true-peak explanation sits under the declined buttons.
#[test]
fn declined_explains_analysis() {
    assert!(copy_of(FirstRunState::Declined, None).explains_analysis);
}

// What:     `#[test] fn system_no_audio_title_is_exact()` checks the device library headline word for word.
// Why:      The headline for a device library with no audio is approved text.
#[test]
fn system_no_audio_title_is_exact() {
    assert_eq!(
        copy_of(FirstRunState::SystemNoAudio, None).title,
        "No audio found in the device music library"
    );
}

// What:     `#[test] fn system_no_audio_body_is_exact()` checks the device library body word for word.
// Why:      The body says the library has no audio and names a folder alternative, in plain words (EM4).
#[test]
fn system_no_audio_body_is_exact() {
    assert_eq!(
        copy_of(FirstRunState::SystemNoAudio, None).body,
        "The device music library has no audio files. You can open a different folder instead."
    );
}

// What:     `#[test] fn system_no_audio_offers_folder_then_settings()` checks the filled and outlined actions.
// Why:      The filled button already opens a folder, so no second folder button is drawn.
#[test]
fn system_no_audio_offers_folder_then_settings() {
    let copy = copy_of(FirstRunState::SystemNoAudio, None);
    assert_eq!(copy.primary_action, Some("Open a folder"));
    assert_eq!(copy.secondary_actions, ["Settings"]);
}

// What:     `#[test] fn folder_no_audio_inserts_folder_name()` checks the chosen folder name in the headline.
// Why:      The headline must name the folder that was checked.
#[test]
fn folder_no_audio_inserts_folder_name() {
    assert_eq!(
        copy_of(FirstRunState::FolderNoAudio, Some("Cult of Luna")).title,
        "No audio found in\nCult of Luna"
    );
}

// What:     `#[test] fn folder_no_audio_uses_the_supplied_name()` checks that a different name replaces the first.
// Why:      The headline is built from the argument, so it must follow whatever folder is passed.
#[test]
fn folder_no_audio_uses_the_supplied_name() {
    assert_eq!(
        copy_of(FirstRunState::FolderNoAudio, Some("Albums")).title,
        "No audio found in\nAlbums"
    );
}

// What:     `#[test] fn folder_no_audio_body_is_exact()` checks the folder body word for word.
// Why:      The body says in plain words that the folder holds nothing to play and what to do (EM4).
#[test]
fn folder_no_audio_body_is_exact() {
    assert_eq!(
        copy_of(FirstRunState::FolderNoAudio, Some("Albums")).body,
        "This folder and its subfolders have no audio files the player can play. \
         Open a different folder to try again."
    );
}

// What:     `#[test] fn folder_no_audio_offers_folder_then_settings()` checks the folder state's actions.
// Why:      The folder state mirrors the device state: Open a folder filled, Settings outlined.
#[test]
fn folder_no_audio_offers_folder_then_settings() {
    let copy = copy_of(FirstRunState::FolderNoAudio, Some("Albums"));
    assert_eq!(copy.primary_action, Some("Open a folder"));
    assert_eq!(copy.secondary_actions, ["Settings"]);
}

// What:     `#[test] fn folder_no_audio_rejects_missing_name()` checks that an absent name is refused.
// Why:      A headline without a folder name would be wrong, so the state refuses to build it.
#[test]
fn folder_no_audio_rejects_missing_name() {
    let error = first_run_copy(FirstRunState::FolderNoAudio, None)
        .expect_err("a missing folder name must be rejected");
    assert_eq!(
        error.to_string(),
        "The folder no-audio state needs a non-blank folder name."
    );
}

// What:     `#[test] fn folder_no_audio_rejects_blank_name()` checks that a whitespace-only name is refused.
// Why:      A blank name would produce a headline that ends with nothing after "in".
#[test]
fn folder_no_audio_rejects_blank_name() {
    let error = first_run_copy(FirstRunState::FolderNoAudio, Some("   "))
        .expect_err("a blank folder name must be rejected");
    assert_eq!(error, FirstRunError::BlankFolderName);
}

// What:     `#[test] fn folder_name_is_ignored_by_other_states()` checks the device states ignore a folder name.
// Why:      A device headline must never name a folder, even when one is passed in.
#[test]
fn folder_name_is_ignored_by_other_states() {
    assert_eq!(
        copy_of(FirstRunState::SystemNoAudio, Some("Albums")).title,
        "No audio found in the device music library"
    );
}

// What:     `#[test] fn no_state_offers_analysis_choice()` checks that no action label asks about analysis.
// Why:      Analysis is automatic, so no first-run action may ask about it.
#[test]
fn no_state_offers_analysis_choice() {
    for state in FirstRunState::ALL {
        let copy = copy_of(state, Some("Albums"));
        let labels = copy.primary_action.into_iter().chain(copy.secondary_actions.iter().copied());
        for label in labels {
            assert!(
                !label.to_lowercase().contains("analys"),
                "{state:?} offered an analysis action: {label}"
            );
        }
    }
}

// What:     `#[test] fn only_declined_explains_analysis()` checks that only the declined state explains analysis.
// Why:      The explanation is drawn once, under the declined buttons, and no other state draws it.
#[test]
fn only_declined_explains_analysis() {
    let explaining: Vec<FirstRunState> = FirstRunState::ALL
        .into_iter()
        .filter(|state| return copy_of(*state, Some("Albums")).explains_analysis)
        .collect();
    assert_eq!(explaining, [FirstRunState::Declined]);
}

// What:     `#[test] fn every_state_offers_settings()` checks that each state keeps a path to in-app Settings.
// Why:      No first-run state may lack a way forward, so Settings stays in every state.
#[test]
fn every_state_offers_settings() {
    for state in FirstRunState::ALL {
        let copy = copy_of(state, Some("Albums"));
        assert!(
            copy.secondary_actions.contains(&SETTINGS_LABEL),
            "{state:?} has no Settings action"
        );
    }
}

// What:     `#[test] fn analysis_explanation_is_exact()` checks the true-peak sentence word for word.
// Why:      The approved explanation must not be paraphrased.
#[test]
fn analysis_explanation_is_exact() {
    assert_eq!(
        ANALYSIS_EXPLANATION,
        "True peak is measured automatically for every audio file. Analysis uses CPU and battery; playback remains available while it runs."
    );
}

// What:     `#[test] fn no_folder_open_title_is_exact()` checks the desktop no-folder headline word for word.
// Why:      The start with no folder open is shown only on desktop, so its copy is pinned here.
#[test]
fn no_folder_open_title_is_exact() {
    assert_eq!(copy_of(FirstRunState::NoFolderOpen, None).title, "No folder is open");
}

// What:     `#[test] fn no_folder_open_offers_open_folder_then_settings()` checks the filled and outlined actions.
// Why:      Choosing a folder is the only way to start, so it is the filled action.
#[test]
fn no_folder_open_offers_open_folder_then_settings() {
    let copy = copy_of(FirstRunState::NoFolderOpen, None);
    assert_eq!(copy.primary_action, Some(OPEN_FOLDER_LABEL));
    assert_eq!(copy.secondary_actions, vec![SETTINGS_LABEL]);
    assert!(!copy.explains_analysis);
}

// What:     `#[test] fn no_folder_open_ignores_a_folder_name()` checks that the no-folder copy takes no name.
// Why:      No folder exists in this state, so a supplied name must not leak into the copy.
#[test]
fn no_folder_open_ignores_a_folder_name() {
    assert_eq!(copy_of(FirstRunState::NoFolderOpen, Some("Albums")).title, "No folder is open");
}

// What:     `#[test] fn the_folder_name_sits_on_its_own_line_so_a_hyphenated_name_is_never_split_by_the_headline()`.
// Why:      EM4: `No audio found in lib-noaudio` wrapped inside the name at its hyphen.
#[test]
fn the_folder_name_sits_on_its_own_line_so_a_hyphenated_name_is_never_split_by_the_headline() {
    let title = copy_of(FirstRunState::FolderNoAudio, Some("lib-noaudio")).title;
    assert_eq!(title.lines().collect::<Vec<_>>(), ["No audio found in", "lib-noaudio"]);
}

// What:     `#[test] fn reading_says_the_library_is_being_read_and_names_no_folder()`.
// Why:      EM1: while the walk runs the window said "No folder is open"; the reading copy follows D69's wording.
#[test]
fn reading_says_the_library_is_being_read_and_names_no_folder() {
    let copy = copy_of(FirstRunState::Reading, None);
    assert_eq!(copy.title, "Reading your library");
    assert_eq!(copy.primary_action, None, "nothing is wrong, so there is no action to take");
    assert_eq!(copy.secondary_actions, [SETTINGS_LABEL]);
    assert!(!copy.explains_analysis);
    assert_eq!(copy_of(FirstRunState::Reading, Some("Albums")), copy, "the copy takes no folder name");
}

// What:     `#[test] fn no_copy_uses_the_jargon_the_polish_review_found()`.
// Why:      EM4: "was checked completely" and "changes the music source" read like internals; "dir" is jargon.
#[test]
fn no_copy_uses_the_jargon_the_polish_review_found() {
    for state in FirstRunState::ALL {
        let copy = copy_of(state, Some("Albums"));
        let text = format!("{} {}", copy.title, copy.body).to_lowercase();
        for banned in ["checked completely", "music source", " dir "] {
            assert!(!text.contains(banned), "{state:?} still says {banned:?}");
        }
    }
}
