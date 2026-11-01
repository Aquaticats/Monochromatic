// What:     `package dev.monochromatic.musicplayer.core` places this test beside the first-run decision it checks.
// Why:      Same-package tests call `firstRunDecisionFor` and `firstRunCanPrompt` without imports.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     JUnit assertions register each decision check.
// Why:      Every screen and action is compared with an explicit expected value.
//
// In TS you'd write (pseudocode):
// ```ts
// import { expect, test } from "test";
// ```
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

// What:     `class FirstRunDecisionTest` groups the decision checks for the first-run screens and the Allow access
//           action.
// Why:      The decision decides what the user sees on first launch, so each source and permission case is checked
//           here.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("firstRunDecisionFor", () => { ... });
// ```
/** Verifies which first-run screen and Allow access action each combination of source and permission produces. */
class FirstRunDecisionTest {
    // What:     `private val withoutSource: FirstRunInputs` is the base case with no permission and no folder.
    // Why:      Each test changes one fact from this base, so the expected screen follows from that one change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const withoutSource = { permissionGranted: false, heldFolderName: null, loading: false, trackCount: 0, ... };
    // ```
    /** Inputs with no permission, no held folder, a finished read, no track, and a prompt still available. */
    private val withoutSource: FirstRunInputs = FirstRunInputs(
        permissionGranted = false,
        heldFolderName = null,
        loading = false,
        trackCount = 0,
        canPromptForPermission = true,
    )

    // What:     `private val withDeviceAccess: FirstRunInputs` is the base case with the permission granted.
    // Why:      The device library is readable only with the permission, so this base covers the system-library cases.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const withDeviceAccess = { ...withoutSource, permissionGranted: true };
    // ```
    @Test
    fun noVerdictAndABlankScreenWhileThePromptIsUnanswered() {
        val decision = firstRunDecisionFor(withoutSource.copy(promptShowing = true))
        assertNull(decision.state)
        assertTrue(decision.awaitingAnswer)
    }

    @Test
    fun theDeclinedScreenShowsOnceThePromptIsAnswered() {
        val decision = firstRunDecisionFor(withoutSource.copy(promptShowing = false))
        assertEquals(FirstRunState.DECLINED, decision.state)
        assertFalse(decision.awaitingAnswer)
    }

    @Test
    fun aGrantedPermissionIsNeverAwaitingEvenWhileThePromptFlagIsStillSet() {
        val decision = firstRunDecisionFor(withDeviceAccess.copy(promptShowing = true))
        assertFalse(decision.awaitingAnswer)
        assertEquals(FirstRunState.SYSTEM_NO_AUDIO, decision.state)
    }

    /** Inputs with the music permission granted, no held folder, a finished read, and no track. */
    private val withDeviceAccess: FirstRunInputs = withoutSource.copy(permissionGranted = true)

    // What:     `private val withFolder: FirstRunInputs` is the base case with a held folder named Albums.
    // Why:      A held folder takes priority over the device library, so this base covers the folder cases.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const withFolder = { ...withoutSource, heldFolderName: "Albums" };
    // ```
    /** Inputs with a held folder named Albums, no permission, a finished read, and no track. */
    private val withFolder: FirstRunInputs = withoutSource.copy(heldFolderName = FOLDER_NAME_FOR_TESTS)

    // What:     `declinedWhenNoSourceAndReadFinished` checks that no source gives the declined pane.
    // Why:      With neither the permission nor a held folder, the device library cannot be read, so the declined pane
    //           shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("declined", () => expect(decide(withoutSource).state).toBe("DECLINED"));
    // ```
    /** Confirms that no permission and no folder gives the declined first-run screen. */
    @Test
    fun declinedWhenNoSourceAndReadFinished() {
        assertEquals(FirstRunState.DECLINED, firstRunDecisionFor(withoutSource).state)
    }

    // What:     `declinedWhileLoadingWithoutSource` checks that a loading flag does not hide the declined pane.
    // Why:      With no source the controller never starts a read, so its loading flag stays true; the pane must still
    //           show.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("declined while loading", () => expect(decide({ ...withoutSource, loading: true }).state).toBe("DECLINED"));
    // ```
    /** Confirms that the declined screen shows even while the loading flag is still set. */
    @Test
    fun declinedWhileLoadingWithoutSource() {
        assertEquals(
            FirstRunState.DECLINED,
            firstRunDecisionFor(withoutSource.copy(loading = true)).state,
        )
    }

    // What:     `declinedWithTracksStillReported` checks that a track count does not change the declined pane.
    // Why:      Without a source, the count is not a result of a read, so the declined pane still shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("declined with tracks", () => expect(decide({ ...withoutSource, trackCount: 3 }).state).toBe("DECLINED"));
    // ```
    /** Confirms that a non-zero track count without a source still gives the declined screen. */
    @Test
    fun declinedWithTracksStillReported() {
        assertEquals(
            FirstRunState.DECLINED,
            firstRunDecisionFor(withoutSource.copy(trackCount = 3)).state,
        )
    }

    // What:     `allowAccessRequestsWhilePromptAvailable` checks that Allow access asks Android while the prompt can
    //           show.
    // Why:      Before any refusal, the button must open the system prompt, which is the only way to grant access.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("request", () => expect(decide(withoutSource).allowAccess).toBe("requestPermission"));
    // ```
    /** Confirms that the Allow access action requests the permission while the prompt is still available. */
    @Test
    fun allowAccessRequestsWhilePromptAvailable() {
        assertEquals(AllowAccessAction.RequestPermission, firstRunDecisionFor(withoutSource).allowAccess)
    }

    // What:     `allowAccessOpensAppSettingsWhenPromptGone` checks that Allow access opens the app's settings once
    //           Android stops prompting.
    // Why:      After repeated refusals Android shows no prompt, so the app's settings page is the only way left to
    //           grant access.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("settings", () => expect(decide({ ...withoutSource, canPromptForPermission: false
    //           }).allowAccess).toBe("openAppSettings"));
    // ```
    /** Confirms that the Allow access action opens the app's settings once Android no longer shows its prompt. */
    @Test
    fun allowAccessOpensAppSettingsWhenPromptGone() {
        assertEquals(
            AllowAccessAction.OpenAppSettings,
            firstRunDecisionFor(withoutSource.copy(canPromptForPermission = false)).allowAccess,
        )
    }

    // What:     `systemNoAudioWhenPermissionGrantedAndReadFinishedEmpty` checks the device library with no audio.
    // Why:      A finished read of the device library that holds no track is the system no-audio state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("system empty", () => expect(decide(withDeviceAccess).state).toBe("SYSTEM_NO_AUDIO"));
    // ```
    /** Confirms that a finished, empty device library gives the system no-audio screen. */
    @Test
    fun systemNoAudioWhenPermissionGrantedAndReadFinishedEmpty() {
        assertEquals(FirstRunState.SYSTEM_NO_AUDIO, firstRunDecisionFor(withDeviceAccess).state)
    }

    // What:     `playerWhenDeviceLibraryHasTracks` checks that loaded tracks show the player.
    // Why:      With tracks in the device library, the first-run pane must not cover the library.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("player", () => expect(decide({ ...withDeviceAccess, trackCount: 4 }).state).toBeNull());
    // ```
    /** Confirms that a device library with tracks gives no first-run screen. */
    @Test
    fun playerWhenDeviceLibraryHasTracks() {
        assertNull(firstRunDecisionFor(withDeviceAccess.copy(trackCount = 4)).state)
    }

    // What:     `playerWhileDeviceLibraryLoads` checks that an unfinished read does not show a no-audio screen.
    // Why:      A zero count during a read is not a result, so the player shows until the read completes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("loading", () => expect(decide({ ...withDeviceAccess, loading: true }).state).toBeNull());
    // ```
    /** Confirms that a device library still being read gives no first-run screen. */
    @Test
    fun playerWhileDeviceLibraryLoads() {
        assertNull(firstRunDecisionFor(withDeviceAccess.copy(loading = true)).state)
    }

    // What:     `folderNoAudioWhenHeldFolderReadFinishedEmptyWithoutPermission` checks a held folder with no audio.
    // Why:      A held folder is readable without the permission, so an empty finished read gives the folder no-audio
    //           state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder empty", () => expect(decide(withFolder).state).toBe("FOLDER_NO_AUDIO"));
    // ```
    /** Confirms that a finished, empty held folder gives the folder no-audio screen without the permission. */
    @Test
    fun folderNoAudioWhenHeldFolderReadFinishedEmptyWithoutPermission() {
        assertEquals(FirstRunState.FOLDER_NO_AUDIO, firstRunDecisionFor(withFolder).state)
    }

    // What:     `folderNoAudioTakesPriorityOverGrantedDeviceLibrary` checks that a held folder decides the source
    //           first.
    // Why:      A held folder is the source that the library reads, even when the device library is also permitted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder priority", () => expect(decide({ ...withFolder, permissionGranted: true
    //           }).state).toBe("FOLDER_NO_AUDIO"));
    // ```
    /** Confirms that a held folder with no audio shows the folder screen even when the device library is granted. */
    @Test
    fun folderNoAudioTakesPriorityOverGrantedDeviceLibrary() {
        assertEquals(
            FirstRunState.FOLDER_NO_AUDIO,
            firstRunDecisionFor(withFolder.copy(permissionGranted = true)).state,
        )
    }

    // What:     `playerWhenHeldFolderHasTracksWithoutPermission` checks that a readable folder shows the player.
    // Why:      A held folder with tracks is a working source, so the permission is not needed to show the player.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder player", () => expect(decide({ ...withFolder, trackCount: 2 }).state).toBeNull());
    // ```
    /** Confirms that a held folder with tracks gives no first-run screen even without the permission. */
    @Test
    fun playerWhenHeldFolderHasTracksWithoutPermission() {
        assertNull(firstRunDecisionFor(withFolder.copy(trackCount = 2)).state)
    }

    // What:     `playerWhileHeldFolderLoadsWithPermission` checks that a folder still being read shows the player.
    // Why:      The folder read is not finished, so no no-audio result exists yet and the loading notice shows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("folder loading", () => expect(decide({ ...withFolder, permissionGranted: true, loading: true
    //           }).state).toBeNull());
    // ```
    /** Confirms that a held folder still being read gives no first-run screen. */
    @Test
    fun playerWhileHeldFolderLoadsWithPermission() {
        assertNull(firstRunDecisionFor(withFolder.copy(permissionGranted = true, loading = true)).state)
    }

    // What:     `allowAccessIsRequestForHeldFolderWhilePromptAvailable` checks the action when a folder holds the
    //           source.
    // Why:      The action depends only on the prompt rule, so a held folder does not change what Allow access does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("action with folder", () => expect(decide(withFolder).allowAccess).toBe("requestPermission"));
    // ```
    /**
     * Confirms that Allow access still requests the permission when a folder holds the source and the prompt is
     * available.
     */
    @Test
    fun allowAccessIsRequestForHeldFolderWhilePromptAvailable() {
        assertEquals(AllowAccessAction.RequestPermission, firstRunDecisionFor(withFolder).allowAccess)
    }

    // What:     `systemNoAudioKeepsSettingsActionWhenPromptGone` checks the device-library screen with the prompt gone.
    // Why:      The no-audio screen has no Allow access button, but the decision still reports the action, which must
    //           follow the prompt rule rather than the screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("empty with prompt gone", () => expect(decide({ ...withDeviceAccess, canPromptForPermission: false
    //           }).allowAccess).toBe("openAppSettings"));
    // ```
    /** Confirms that the device-library screen reports the settings action once the prompt is gone. */
    @Test
    fun systemNoAudioKeepsSettingsActionWhenPromptGone() {
        val decision: FirstRunDecision = firstRunDecisionFor(withDeviceAccess.copy(canPromptForPermission = false))
        assertEquals(FirstRunState.SYSTEM_NO_AUDIO, decision.state)
        assertEquals(AllowAccessAction.OpenAppSettings, decision.allowAccess)
    }

    // What:     `promptAvailableBeforeAnyRequest` checks the prompt rule for a permission never requested.
    // Why:      A first launch has no refusal yet, so Android still shows the prompt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("fresh", () => expect(firstRunCanPrompt(false, false)).toBe(true));
    // ```
    /** Confirms that the prompt is available before this app has made any request. */
    @Test
    fun promptAvailableBeforeAnyRequest() {
        assertTrue(firstRunCanPrompt(requestedBefore = false, shouldShowRationale = false))
    }

    // What:     `promptAvailableWhileRationaleHolds` checks the prompt rule after a first refusal.
    // Why:      After one refusal Android still shows the rationale, which means the prompt can show again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("rationale", () => expect(firstRunCanPrompt(true, true)).toBe(true));
    // ```
    /** Confirms that the prompt is still available after one refusal, while Android shows the rationale. */
    @Test
    fun promptAvailableWhileRationaleHolds() {
        assertTrue(firstRunCanPrompt(requestedBefore = true, shouldShowRationale = true))
    }

    // What:     `promptGoneAfterRequestWithoutRationale` checks the prompt rule after a permanent refusal.
    // Why:      Once a request was made and Android no longer shows the rationale, the prompt has stopped for good.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // test("gone", () => expect(firstRunCanPrompt(true, false)).toBe(false));
    // ```
    /** Confirms that the prompt is gone after a request when Android no longer shows the rationale. */
    @Test
    fun promptGoneAfterRequestWithoutRationale() {
        assertFalse(firstRunCanPrompt(requestedBefore = true, shouldShowRationale = false))
    }

    // What:     `private companion object` holds the folder name the tests use.
    // Why:      A named constant keeps the folder fixture readable, and it is not a magic value in each test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const FOLDER_NAME_FOR_TESTS = "Albums";
    // ```
    /** Folder name used by the folder cases. */
    private companion object {
        // What:     `const val FOLDER_NAME_FOR_TESTS: String = "Albums"` names the folder fixture.
        // Why:      The folder cases share one name, so a change in the fixture is made once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const FOLDER_NAME_FOR_TESTS = "Albums";
        // ```
        /** Folder name used by the folder cases. */
        const val FOLDER_NAME_FOR_TESTS: String = "Albums"
    }
}
