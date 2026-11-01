// What:     `package dev.monochromatic.musicplayer.core` places the first-run copy beside the
//           other platform-independent core logic.
// Why:      The states and their copy can be tested on the host JVM without a Compose runtime.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `const val ALLOW_ACCESS_LABEL: String` names the filled action that asks for music access.
// Why:      The copy and the pane both refer to this label, so one constant keeps them in step.
//
// In TS you'd write (pseudocode):
// ```ts
// const ALLOW_ACCESS_LABEL = "Allow access";
// ```
/** Label of the action that asks Android for access to music on this device. */
const val ALLOW_ACCESS_LABEL: String = "Allow access"

// What:     `const val OPEN_FOLDER_LABEL: String` names the action that chooses another folder.
// Why:      Two states offer this action, and the pane maps the label to one callback.
//
// In TS you'd write (pseudocode):
// ```ts
// const OPEN_FOLDER_LABEL = "Open a folder";
// ```
/** Label of the action that opens the folder picker. */
const val OPEN_FOLDER_LABEL: String = "Open a folder"

// What:     `const val SETTINGS_LABEL: String` names the in-app Settings action.
// Why:      Settings is in-app Settings, not Android's permission page, so its label stays separate.
//
// In TS you'd write (pseudocode):
// ```ts
// const SETTINGS_LABEL = "Settings";
// ```
/** Label of the action that opens the in-app Settings pane. */
const val SETTINGS_LABEL: String = "Settings"

// What:     `const val ANALYSIS_EXPLANATION: String` holds the true-peak sentence shown under the
//           declined state's actions.
// Why:      D84 makes analysis automatic, so the explanation states that it runs without offering a choice.
//
// In TS you'd write (pseudocode):
// ```ts
// const ANALYSIS_EXPLANATION = "True peak is measured automatically for every audio file. ...";
// ```
/** Explains that true-peak analysis runs automatically and that playback stays available. */
const val ANALYSIS_EXPLANATION: String =
    "True peak is measured automatically for every audio file. " +
        "Analysis uses CPU and battery; playback remains available while it runs."

// What:     `enum class FirstRunState` declares the three first-run screens.
// Why:      Each screen explains an open source that cannot be read or has no audio, and the set is closed.
//
// In TS you'd write (pseudocode):
// ```ts
// type FirstRunState = "DECLINED" | "SYSTEM_NO_AUDIO" | "FOLDER_NO_AUDIO";
// ```
/** Names the first-run screens shown when the open library cannot be read or holds no audio. */
enum class FirstRunState {
    // What:     `DECLINED` is the first entry of the enum.
    // Why:      Access refused to the device library, which stays open as the library.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // DECLINED = "DECLINED",
    // ```
    /** The device music library is open, but Android has not granted access to music on this device. */
    DECLINED,

    // What:     `SYSTEM_NO_AUDIO` is the second entry of the enum.
    // Why:      The device library was read completely and holds no audio.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // SYSTEM_NO_AUDIO = "SYSTEM_NO_AUDIO",
    // ```
    /** The device music library was read completely and holds no audio. */
    SYSTEM_NO_AUDIO,

    // What:     `FOLDER_NO_AUDIO` is the third entry of the enum.
    // Why:      The chosen folder was read completely and holds no supported audio.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // FOLDER_NO_AUDIO = "FOLDER_NO_AUDIO",
    // ```
    /** The chosen folder was read completely and holds no supported audio file. */
    FOLDER_NO_AUDIO,
}

// What:     `data class FirstRunCopy(...)` is the immutable copy for one first-run state.
// Why:      The pane draws exactly these strings and actions, and tests compare them directly.
//
// In TS you'd write (pseudocode):
// ```ts
// type FirstRunCopy = { title: string; body: string; primaryAction: string | null;
//   secondaryActions: string[]; explainsAnalysis: boolean };
// ```
/** Display copy and action labels for one first-run state. */
data class FirstRunCopy(
    // What:     `val title: String` is the headline of the state.
    // Why:      The headline names the source the state is about.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // title: string;
    // ```
    /** Headline that names the source the state is about. */
    val title: String,
    // What:     `val body: String` is the explanation under the headline.
    // Why:      The body says what is open and what the actions do, without claiming more than was checked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // body: string;
    // ```
    /** Explanation of the open source and what the actions do. */
    val body: String,
    // What:     `val primaryAction: String?` is the label of the filled button, or null when none.
    // Why:      The filled button is the one action the state recommends; null draws no filled button.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // primaryAction: string | null;
    // ```
    /** Label of the filled action, or null when the state draws no filled action. */
    val primaryAction: String?,
    // What:     `val secondaryActions: List<String>` lists the outlined button labels in display order.
    // Why:      Declined access draws Open a folder before Settings, and no-audio states draw Settings alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // secondaryActions: readonly string[];
    // ```
    /** Labels of the outlined actions, in the order they are drawn. */
    val secondaryActions: List<String>,
    // What:     `val explainsAnalysis: Boolean` says whether the true-peak explanation is drawn.
    // Why:      Only the declined state carries the explanation, as D101 approves.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // explainsAnalysis: boolean;
    // ```
    /** True when the true-peak explanation belongs below the actions. */
    val explainsAnalysis: Boolean,
)

// What:     `private fun nonBlankFolderName(folderName: String?): String` returns a usable folder name.
// Why:      A missing or blank name would print an empty headline, so the function throws instead.
//
// In TS you'd write (pseudocode):
// ```ts
// function nonBlankFolderName(folderName: string | null): string {
//   if (folderName === null || folderName.trim() === "") throw new Error("Folder name is required");
//   return folderName;
// }
// ```
/**
 * Returns the folder name when it is present and not blank.
 *
 * @param folderName name supplied by the caller
 * @return the same name, unchanged
 * @throws IllegalArgumentException when the name is null or blank
 */
private fun nonBlankFolderName(folderName: String?): String {
    if (folderName == null || folderName.isBlank()) {
        throw IllegalArgumentException("The folder no-audio state needs a non-blank folder name.")
    }
    return folderName
}

// What:     `fun firstRunCopy(state: FirstRunState, folderName: String?): FirstRunCopy` returns the copy.
// Why:      One pure function owns every string, so the pane and the tests read the same source.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunCopy(state: FirstRunState, folderName: string | null): FirstRunCopy { ... }
// ```
/**
 * Returns the copy for one first-run state.
 *
 * @param state first-run screen to describe
 * @param folderName name of the chosen folder, required for the folder no-audio state
 * @return headline, body, action labels, and the analysis flag for the state
 * @throws IllegalArgumentException when the folder no-audio state has no non-blank folder name
 */
fun firstRunCopy(state: FirstRunState, folderName: String?): FirstRunCopy = when (state) {
    // What:     The declined branch returns the approved copy with Allow access as the filled action.
    // Why:      D101 approves this order, and D100 keeps the library open even when unreadable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // case "DECLINED": return { title, body, primaryAction: "Allow access", ... };
    // ```
    FirstRunState.DECLINED -> FirstRunCopy(
        title = "The device music library can't be read",
        body = "It is open as your library, but access to music on this device was not granted. " +
            "Allow access, or open a folder instead.",
        primaryAction = ALLOW_ACCESS_LABEL,
        secondaryActions = listOf(OPEN_FOLDER_LABEL, SETTINGS_LABEL),
        explainsAnalysis = true,
    )
    // What:     The system no-audio branch returns the device library copy with Open a folder filled.
    // Why:      The filled action already opens a folder, so no second folder button is drawn.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // case "SYSTEM_NO_AUDIO": return { title, body, primaryAction: "Open a folder", ... };
    // ```
    FirstRunState.SYSTEM_NO_AUDIO -> FirstRunCopy(
        title = "No audio found in the device music library",
        body = "The device music library was checked completely. " +
            "You can open a folder that is not listed there.",
        primaryAction = OPEN_FOLDER_LABEL,
        secondaryActions = listOf(SETTINGS_LABEL),
        explainsAnalysis = false,
    )
    // What:     The folder no-audio branch names the chosen folder in the headline.
    // Why:      The headline must name the folder that was checked, so the folder name is required here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // case "FOLDER_NO_AUDIO": return { title: `No audio found in ${folderName}`, ... };
    // ```
    FirstRunState.FOLDER_NO_AUDIO -> {
        /** Non-blank folder name inserted into the headline. */
        val name: String = nonBlankFolderName(folderName = folderName)
        FirstRunCopy(
            title = "No audio found in $name",
            body = "The chosen folder was checked completely and contains no supported audio files. " +
                "Opening a different folder changes the music source.",
            primaryAction = OPEN_FOLDER_LABEL,
            secondaryActions = listOf(SETTINGS_LABEL),
            explainsAnalysis = false,
        )
    }
}
