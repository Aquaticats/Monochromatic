// What:     `package dev.monochromatic.musicplayer` places the first-run access helpers beside the activity that uses
//           them.
// Why:      The helpers read the activity, the saved request flag, and the document id, and the activity calls them.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.content.Intent` builds the intent that opens the app's settings page.
// Why:      The declined state's Allow access button opens that page once Android stops showing its prompt.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Intent } from "android/content";
// ```
import android.content.Intent

// What:     `import android.net.Uri` names the package and the folder tree.
// Why:      The settings intent addresses the package by URI, and the folder tree arrives as a URI.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Uri } from "android/net";
// ```
import android.net.Uri

// What:     `import android.provider.DocumentsContract` reads the document id of a folder tree.
// Why:      The folder's display name is the last segment of that id, so the copy can name the folder.
//
// In TS you'd write (pseudocode):
// ```ts
// import { DocumentsContract } from "android/provider";
// ```
import android.provider.DocumentsContract

// What:     `import android.provider.Settings` names the settings action for the app's page.
// Why:      The action constant is the standard way to reach an app's page in Android's settings.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Settings } from "android/provider";
// ```
import android.provider.Settings

// What:     `import androidx.compose.runtime.Composable` marks the pane content as a function that emits UI.
// Why:      The pane content is a composable lambda, so the player host can draw it inside its own layout.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { FC } from "react";
// ```
import androidx.compose.runtime.Composable

// What:     `import androidx.activity.ComponentActivity` is the activity type that asks for the permission.
// Why:      Only an activity can report the permission rationale rule, so the helper takes the activity itself.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { ComponentActivity } from "androidx/activity";
// ```
import androidx.activity.ComponentActivity

// What:     `import dev.monochromatic.musicplayer.core.firstRunCanPrompt` and `folderNameFromDocumentId` are the pure
//           rules.
// Why:      The Android reads stay here, and the rules that decide what they mean stay in core and are tested there.
//
// In TS you'd write (pseudocode):
// ```ts
// import { firstRunCanPrompt, folderNameFromDocumentId } from "./core/FirstRunDecision";
// ```
import dev.monochromatic.musicplayer.core.FirstRunState
import dev.monochromatic.musicplayer.core.firstRunCanPrompt
import dev.monochromatic.musicplayer.core.folderNameFromDocumentId

// What:     `internal fun heldFolderNameOf(treeUri: Uri): String` names the folder that a tree URI grants.
// Why:      The first-run copy names the folder, and the name comes from the tree's document id.
//
// In TS you'd write (pseudocode):
// ```ts
// function heldFolderNameOf(treeUri: URL): string { return folderNameFromDocumentId(treeDocumentId(treeUri)); }
// ```
/**
 * Returns the display name of the folder that a persisted tree URI grants.
 *
 * @param treeUri tree URI of the folder that the user chose
 * @return the folder name, never blank
 */
internal fun heldFolderNameOf(treeUri: Uri): String =
    folderNameFromDocumentId(DocumentsContract.getTreeDocumentId(treeUri))

// What:     `internal fun canPromptAudioAccess(activity: ComponentActivity): Boolean` reads whether Android still
//           shows the permission prompt.
// Why:      The saved request flag and the activity's rationale rule are the two facts the pure rule needs.
//
// In TS you'd write (pseudocode):
// ```ts
// function canPromptAudioAccess(activity: Activity): boolean { return firstRunCanPrompt(saved, rationale(activity)); }
// ```
/**
 * Returns whether Android still shows its permission prompt for music access.
 *
 * @param activity activity that asks for the permission and reports the rationale rule
 * @return true while the prompt can still appear, false once Android stops showing it
 */
internal fun canPromptAudioAccess(activity: ComponentActivity): Boolean = firstRunCanPrompt(
    requestedBefore = SessionStore.loadAudioAccessRequested(activity),
    shouldShowRationale = activity.shouldShowRequestPermissionRationale(audioPermission()),
)

// What:     `internal fun appDetailsSettingsIntent(packageName: String): Intent` builds the intent for the app's page.
// Why:      Once Android stops showing its prompt, this page is where the user can still grant music access.
//
// In TS you'd write (pseudocode):
// ```ts
// function appDetailsSettingsIntent(packageName: string): Intent { /* settings page for the package */ }
// ```
/**
 * Returns the intent that opens this app's page in Android's settings.
 *
 * @param packageName package name of this app, which addresses its settings page
 * @return the settings intent for the package
 */
internal fun appDetailsSettingsIntent(packageName: String): Intent =
    Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", packageName, null))

// What:     `internal fun firstRunPaneContent(...)` returns the composable that draws the first-run pane for one state.
// Why:      The player host draws the pane inside its own layout, and the returned lambda carries the pane's actions,
//           with Settings supplied by the host because only the host owns the Settings route.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunPaneContent(state, folderName, onAllowAccess, onOpenFolder): (onSettings) => UIElement {
//   return (onSettings) => FirstRunPane({ state, folderName, actions: { onAllowAccess, onOpenFolder, onSettings } });
// }
// ```
/**
 * Returns the composable that draws the first-run pane for one state, wiring its actions to the given callbacks.
 *
 * @param state first-run screen to draw
 * @param folderName display name of the held folder, required for the folder no-audio state
 * @param onAllowAccess runs the action that the Allow access button performs
 * @param onOpenFolder opens the folder picker
 * @return the pane content, which takes the callback that opens the Settings route
 */
internal fun firstRunPaneContent(
    state: FirstRunState,
    folderName: String?,
    onAllowAccess: () -> Unit,
    onOpenFolder: () -> Unit,
): @Composable (onSettings: () -> Unit) -> Unit = { onSettings ->
    firstRunPane(
        state = state,
        folderName = folderName,
        actions = FirstRunActions(
            onAllowAccess = onAllowAccess,
            onOpenFolder = onOpenFolder,
            onSettings = onSettings,
        ),
    )
}
