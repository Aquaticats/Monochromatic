// The controller installs libraries that were built off the main thread, and answers the questions Search needs of the
// library as it is now: where a URI is, and which track sits at an index. Installing a built library must leave the
// player exactly where installing the plain list leaves it.

// What:     `package dev.monochromatic.musicplayer` places the cases beside the controller.
// Why:      They construct a `PlayerController` with the fake engine, as `PlayerControllerTest` does.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the path.
// ```
package dev.monochromatic.musicplayer

import dev.monochromatic.musicplayer.core.Session
import dev.monochromatic.musicplayer.core.buildLibrary
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the built installs and the lookups. */
class PlayerControllerBuiltTest {
    /** Builds a small library whose display paths share no prefix. */
    private fun library(): List<Track> = listOf(
        Track("u-a", "Alpha/one.flac"),
        Track("u-b", "Alpha/two.flac"),
        Track("u-c", "Beta/three.flac"),
        Track("u-d", "Beta/Disc 2/four.flac"),
    )

    /** Reads the page labels and rows of a controller as one comparable value. */
    private fun shown(controller: PlayerController): List<Any?> = listOf(
        controller.uiState.pageLabels, controller.uiState.pageItems, controller.uiState.currentIndex,
        controller.uiState.queueSize, controller.uiState.loading,
    )

    /** A built library installs the same state as the plain list, for reconcile, open and finish. */
    @Test
    fun builtInstallsEqualPlainInstalls() {
        val plain = PlayerController(FakeAudioEngine())
        val built = PlayerController(FakeAudioEngine())
        plain.reconcileLibrary(library())
        built.reconcileBuilt(buildLibrary(library()), feedSearch = false)
        assertEquals(shown(plain), shown(built))
        val openPlain = PlayerController(FakeAudioEngine())
        val openBuilt = PlayerController(FakeAudioEngine())
        openPlain.openLibrary(library())
        openBuilt.openBuilt(buildLibrary(library()), feedSearch = false)
        assertEquals(shown(openPlain), shown(openBuilt))
        val session = Session(selected = "u-c", positionSecs = 3.0)
        val finishPlain = PlayerController(FakeAudioEngine())
        val finishBuilt = PlayerController(FakeAudioEngine())
        assertEquals(finishPlain.finishLoad(library(), session), finishBuilt.finishBuilt(buildLibrary(library()), session, false))
        assertEquals(shown(finishPlain), shown(finishBuilt))
        assertEquals("u-c", finishBuilt.uriAt(finishBuilt.uiState.currentIndex!!))
    }

    /** The loaded track is found again by the hint the build carries, and by a scan when the hint is stale. */
    @Test
    fun theLoadedTrackSurvivesAReconcileWithOrWithoutAHint() {
        val engine = FakeAudioEngine()
        val controller = PlayerController(engine)
        controller.openLibrary(library())
        controller.playIndex(2)
        assertEquals("u-c", controller.loadedUriSnapshot())
        val grown = library() + Track("u-e", "Gamma/five.flac")
        controller.reconcileBuilt(buildLibrary(grown, controller.loadedUriSnapshot()), feedSearch = false)
        assertEquals("u-c", controller.uriAt(controller.uiState.currentIndex!!))
        controller.reconcileBuilt(buildLibrary(grown + Track("u-f", "Gamma/six.flac"), "stale"), feedSearch = false)
        assertEquals("u-c", controller.uriAt(controller.uiState.currentIndex!!))
    }

    /** A rescan that found the same tracks changes nothing, and a rescan that found others installs them. */
    @Test
    fun aRescanOfTheSameTracksInstallsNothing() = runBlocking {
        val controller = PlayerController(FakeAudioEngine())
        controller.openLibrary(library())
        val before = controller.urisSnapshot()
        controller.reconcileFromScan(library())
        assertTrue("the same list object stays installed", before === controller.urisSnapshot())
        controller.reconcileFromScan(library().drop(1))
        assertEquals(3, controller.uiState.queueSize)
        assertFalse(before === controller.urisSnapshot())
    }

    /** URI lookups and track reads reflect the installed library, with display paths as the pages show them. */
    @Test
    fun lookupsReflectTheInstalledLibrary() {
        val controller = PlayerController(FakeAudioEngine())
        controller.openLibrary(listOf(Track("u1", "Music/Rock/a.flac"), Track("u2", "Music/Pop/b.flac")))
        assertEquals(1, controller.indexOfUri("u2"))
        assertEquals(-1, controller.indexOfUri("missing"))
        assertEquals(Track("u1", "Rock/a.flac"), controller.trackAt(0))
        assertNull(controller.trackAt(5))
        assertEquals(listOf("Rock/a.flac", "Pop/b.flac"), controller.libraryTracks().map { it.displayPath })
    }
}
