// What:     `package dev.monochromatic.musicplayer.core` places the first-run decision beside the first-run copy.
// Why:      The decision chooses among the same states that the copy describes, so both live in one package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `sealed interface AllowAccessAction` names what the Allow access button performs.
// Why:      The button asks Android while Android still shows its prompt, and opens the app's page after that.
//
// In TS you'd write (pseudocode):
// ```ts
// type AllowAccessAction = { kind: "requestPermission" } | { kind: "openAppSettings" };
// ```
/** What the Allow access button performs in the declined first-run state. */
sealed interface AllowAccessAction {
    // What:     `data object RequestPermission` asks Android for music access with its permission prompt.
    // Why:      Android shows the prompt until the user has refused it for good, so the button can ask until then.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "requestPermission" }
    // ```
    /** Asks Android for access to music on this device with the system permission prompt. */
    data object RequestPermission : AllowAccessAction

    // What:     `data object OpenAppSettings` opens this app's page in Android's settings.
    // Why:      Once Android stops showing its prompt, that page is the only place where access can still be granted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // { kind: "openAppSettings" }
    // ```
    /** Opens this app's page in Android's settings, where access can still be granted. */
    data object OpenAppSettings : AllowAccessAction
}

// What:     `data class FirstRunInputs(...)` gathers the facts that the first-run decision reads.
// Why:      One immutable value keeps the decision a pure function with no Android types, so it is testable on the JVM.
//
// In TS you'd write (pseudocode):
// ```ts
// type FirstRunInputs = { permissionGranted: boolean; heldFolderName: string | null; loading: boolean;
//   trackCount: number; canPromptForPermission: boolean };
// ```
/** Facts about the open source and the music permission that the first-run decision reads. */
data class FirstRunInputs(
    // What:     `val permissionGranted: Boolean` says whether Android grants access to music on this device.
    // Why:      Without the permission and without a held folder, no source can be read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // permissionGranted: boolean;
    // ```
    /** True when Android grants access to music on this device. */
    val permissionGranted: Boolean,
    // What:     `val heldFolderName: String?` is the display name of the held folder, or null when none is held.
    // Why:      A held folder takes priority over the device library, so its presence decides which source is open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // heldFolderName: string | null;
    // ```
    /** Display name of the folder whose grant is held, or null when no folder grant is held. */
    val heldFolderName: String?,
    // What:     `val loading: Boolean` says whether the library read is still running.
    // Why:      A track count of zero is a finished result only after the read completes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // loading: boolean;
    // ```
    /** True while the open library is still being read. */
    val loading: Boolean,
    // What:     `val trackCount: Int` is the number of tracks loaded from the open library.
    // Why:      The no-audio states need a count of zero after a complete read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // trackCount: number;
    // ```
    /** Number of tracks loaded from the open library. */
    val trackCount: Int,
    // What:     `val canPromptForPermission: Boolean` says whether Android still shows its permission prompt.
    // Why:      The Allow access button asks Android only while Android still shows the prompt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // canPromptForPermission: boolean;
    // ```
    /** True while Android still shows its permission prompt for music access. */
    val canPromptForPermission: Boolean,
    /** True from the moment the app opens the permission prompt until the user answers it. */
    val promptShowing: Boolean = false,
)

// What:     `data class FirstRunDecision(...)` is the first-run screen to draw and the Allow access action.
// Why:      A null state means the player shows, and the action is read only when the declined state is drawn.
//
// In TS you'd write (pseudocode):
// ```ts
// type FirstRunDecision = { state: "DECLINED" | "SYSTEM_NO_AUDIO" | "FOLDER_NO_AUDIO" | null;
//   allowAccess: AllowAccessAction };
// ```
/** The first-run screen to draw, or null when the player shows, with the action of the Allow access button. */
data class FirstRunDecision(
    // What:     `val state: FirstRunState?` is the screen to draw, or null when the player shows.
    // Why:      Null keeps the player on screen whenever a source has tracks or is still being read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state: FirstRunState | null;
    // ```
    /** First-run screen to draw, or null when the player shows. */
    val state: FirstRunState?,
    // What:     `val allowAccess: AllowAccessAction` is what the Allow access button performs.
    // Why:      The caller runs this action when the user presses the button in the declined state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // allowAccess: AllowAccessAction;
    // ```
    /** Action that the Allow access button performs for this decision. */
    val allowAccess: AllowAccessAction,
    /** True while the permission prompt is open and unanswered, when the screen stays blank instead of an error. */
    val awaitingAnswer: Boolean = false,
)

// What:     `fun firstRunCanPrompt(requestedBefore: Boolean, shouldShowRationale: Boolean): Boolean` says whether
//           Android still shows its permission prompt.
// Why:      Before any request Android shows the prompt; after one request, Android shows it again only while the
//           rationale rule holds, and stops for good after the user picks "don't ask again".
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunCanPrompt(requestedBefore: boolean, shouldShowRationale: boolean): boolean {
//   return !requestedBefore || shouldShowRationale;
// }
// ```
/**
 * Returns whether Android still shows its permission prompt for music access.
 *
 * @param requestedBefore whether this app has already asked for the permission
 * @param shouldShowRationale the Android rationale rule for the permission after the last request
 * @return true while the prompt can still appear
 */
fun firstRunCanPrompt(requestedBefore: Boolean, shouldShowRationale: Boolean): Boolean =
    !requestedBefore || shouldShowRationale

// What:     `private fun firstRunStateFor(inputs: FirstRunInputs): FirstRunState?` picks the screen to draw.
// Why:      A held folder decides the source first, then the device library needs the permission, and an unread
//           or non-empty source shows the player.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunStateFor(inputs: FirstRunInputs): FirstRunState | null { /* folder, permission, count */ }
// ```
/**
 * Returns the first-run screen for the open source, or null when the player shows.
 *
 * @param inputs facts about the held folder, the permission, and the library read
 * @return the screen to draw, or null when the library has tracks or is still being read
 */
private fun firstRunStateFor(inputs: FirstRunInputs): FirstRunState? {
    /** True when the open library finished reading and holds no track. */
    val completeAndEmpty: Boolean = !inputs.loading && inputs.trackCount == 0
    if (inputs.heldFolderName != null) {
        return if (completeAndEmpty) FirstRunState.FOLDER_NO_AUDIO else null
    }
    if (!inputs.permissionGranted) {
        return FirstRunState.DECLINED
    }
    return if (completeAndEmpty) FirstRunState.SYSTEM_NO_AUDIO else null
}

// What:     `fun firstRunDecisionFor(inputs: FirstRunInputs): FirstRunDecision` decides the first-run screen and
//           the Allow access action.
// Why:      One pure function owns the rule, so the host only draws what it returns and the tests check every case.
//
// In TS you'd write (pseudocode):
// ```ts
// function firstRunDecisionFor(inputs: FirstRunInputs): FirstRunDecision {
//   return { state: firstRunStateFor(inputs), allowAccess: inputs.canPromptForPermission ? "request" : "settings" };
// }
// ```
/**
 * Decides which first-run screen to draw and which action the Allow access button performs.
 *
 * @param inputs facts about the held folder, the permission, the library read, and the prompt
 * @return the screen to draw (or null for the player) and the Allow access action
 */
fun firstRunDecisionFor(inputs: FirstRunInputs): FirstRunDecision {
    /** Action of the Allow access button, which asks Android while the prompt can still appear. */
    val allowAccess: AllowAccessAction = if (inputs.canPromptForPermission) {
        AllowAccessAction.RequestPermission
    } else {
        AllowAccessAction.OpenAppSettings
    }
    /** Whether the user has not answered the permission prompt yet, so no verdict about the library exists. */
    val awaiting: Boolean = inputs.promptShowing && !inputs.permissionGranted && inputs.heldFolderName == null
    return FirstRunDecision(
        state = if (awaiting) null else firstRunStateFor(inputs),
        allowAccess = allowAccess,
        awaitingAnswer = awaiting,
    )
}
