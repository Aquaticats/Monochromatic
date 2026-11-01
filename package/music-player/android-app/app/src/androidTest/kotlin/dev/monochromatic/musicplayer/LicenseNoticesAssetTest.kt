// Instrumented check of the license notice asset: the installed APK carries assets/THIRD-PARTY-LICENSES.txt in every
// variant, the Licenses page reads it from the app's own assets, and the splitter cuts what it holds into blocks.

package dev.monochromatic.musicplayer

import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import dev.monochromatic.musicplayer.core.LICENSE_BLOCK_MAX_LINES
import dev.monochromatic.musicplayer.core.LicenseBlock
import dev.monochromatic.musicplayer.core.splitLicenseNotices
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertTrue
import org.junit.Test

/** Verifies the packed license notices on an emulator or device. */
class LicenseNoticesAssetTest {
    /** Confirms the app's asset opens and holds either the generated notices or the note of a development build. */
    @Test
    fun theAppCarriesTheNoticeAsset() {
        /** Context of the installed app, whose assets the Licenses page reads. */
        val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
        /** The asset's text. */
        val text: String = runBlocking { readLicenseNoticesAsset(context) }
        assertTrue(
            "the asset holds the notices or the development note, but starts: ${text.take(80)}",
            text.contains("Android libraries (") || text.contains("carries no third-party license notices"),
        )
    }

    /** Confirms the asset splits into bounded blocks, and that a generated file lists the media3 session library. */
    @Test
    fun theAssetSplitsIntoBoundedBlocks() {
        /** Context of the installed app. */
        val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
        /** The asset's text. */
        val text: String = runBlocking { readLicenseNoticesAsset(context) }
        /** The list items the Licenses page would draw. */
        val blocks: List<LicenseBlock> = splitLicenseNotices(text)
        assertTrue("the asset has at least one block", blocks.isNotEmpty())
        assertTrue("no block is longer than the line limit", blocks.all { it.text.lines().size <= LICENSE_BLOCK_MAX_LINES })
        if (text.contains("Android libraries (")) {
            assertTrue("a generated file names media3", blocks.any { it.text.contains("androidx.media3:media3-session") })
        }
    }
}
