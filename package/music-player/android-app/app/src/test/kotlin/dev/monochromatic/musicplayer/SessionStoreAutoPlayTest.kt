package dev.monochromatic.musicplayer

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import dev.monochromatic.musicplayer.core.PlaybackMode
import dev.monochromatic.musicplayer.core.Session
import dev.monochromatic.musicplayer.core.TemplateKind
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/**
 * Pins how the Auto-play on select setting is stored (D129, D133): on by default, on when an older preference file has
 * no such key, kept across a round trip, and never touched by the session save or the older keys.
 */
@RunWith(RobolectricTestRunner::class)
class SessionStoreAutoPlayTest {
    /** Application context whose preference file the cases use. */
    private val context: Context = ApplicationProvider.getApplicationContext()

    /** Starts every case from an empty preference file. */
    @Before
    fun emptyThePreferenceFile() {
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).edit().clear().commit()
    }

    /** Raw boolean under the setting's key, or null when the key is absent. */
    private fun rawValue(): Boolean? {
        /** Preference file as the store opens it. */
        val preferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return if (preferences.contains(KEY)) preferences.getBoolean(KEY, false) else null
    }

    /** A first run, with no file at all, reads on. */
    @Test
    fun theSettingIsOnByDefault() {
        assertTrue(SessionStore.loadAutoPlayOnSelect(context))
        assertEquals("reading never writes the key", null, rawValue())
    }

    /** A file written before the setting existed reads on and still loads its own values. */
    @Test
    fun anOlderFileWithoutTheKeyReadsOnAndStillLoads() {
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).edit()
            .putString("selected", "content://invented/alpha/3")
            .putLong("position_bits", 12.5.toRawBits())
            .putFloat("volume", 0.5f)
            .putString("playback_mode", "shuffle_all")
            .putString("template_TRACK_ROWS", "\$mi(title)\$")
            .commit()
        assertTrue(SessionStore.loadAutoPlayOnSelect(context))
        /** The session restored from the older file. */
        val session: Session = SessionStore.load(context)
        assertEquals("content://invented/alpha/3", session.selected)
        assertEquals(12.5, session.positionSecs, 0.0)
        assertEquals(0.5f, session.volume, 0.0f)
        assertEquals(PlaybackMode.SHUFFLE_ALL, session.playbackMode)
        assertEquals(null, rawValue())
    }

    /** A saved off survives a read, a saved on replaces it, and the raw file holds the value. */
    @Test
    fun theSettingRoundTrips() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = false)
        assertFalse(SessionStore.loadAutoPlayOnSelect(context))
        assertEquals(false, rawValue())
        SessionStore.saveAutoPlayOnSelect(context, enabled = true)
        assertTrue(SessionStore.loadAutoPlayOnSelect(context))
        assertEquals(true, rawValue())
    }

    /** Saving the playback session, which the controller does often, leaves the setting as it was. */
    @Test
    fun savingTheSessionKeepsTheSetting() {
        SessionStore.saveAutoPlayOnSelect(context, enabled = false)
        SessionStore.save(context, Session(selected = "content://invented/alpha/1", positionSecs = 3.0))
        assertFalse(SessionStore.loadAutoPlayOnSelect(context))
        assertEquals(false, rawValue())
        assertEquals("content://invented/alpha/1", SessionStore.load(context).selected)
    }

    /** Saving the setting leaves the saved templates and the session values alone. */
    @Test
    fun savingTheSettingKeepsTheTemplatesAndTheSession() {
        SessionStore.saveTemplate(context, TemplateKind.TRACK_ROWS, "\$mi(title)\$")
        SessionStore.save(context, Session(selected = "content://invented/alpha/2", volume = 0.25f))
        SessionStore.saveAutoPlayOnSelect(context, enabled = false)
        assertEquals("\$mi(title)\$", SessionStore.loadTemplate(context, TemplateKind.TRACK_ROWS))
        /** The session after the setting was saved. */
        val session: Session = SessionStore.load(context)
        assertEquals("content://invented/alpha/2", session.selected)
        assertEquals(0.25f, session.volume, 0.0f)
    }

    /** Names of the preference file and key, spelled out so a rename in the store fails here. */
    private companion object {
        /** Name of the preference file of the session. */
        const val PREFS_NAME: String = "session"

        /** Key of the Auto-play on select setting. */
        const val KEY: String = "auto_play_on_select"
    }
}
