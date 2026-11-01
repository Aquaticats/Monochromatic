// Android persistence boundary for the pure Session model. New writes contain one
// playback-mode key. Former shuffle and repeat keys are read only for migration.

// What:     `package ...musicplayer` places this Android-specific object beside the
//           service that calls it.
// Why:      SharedPreferences must stay outside the platform-independent core package.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

// What:     Android Context resolves app-private services and storage.
// Why:      Both public operations open this app's SharedPreferences file.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Context } from "android/content";
// ```
import android.content.Context

// What:     SharedPreferences is Android's primitive key/value store.
// Why:      The session is a small flat record that needs no database.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { SharedPreferences } from "android/content";
// ```
import android.content.SharedPreferences

// What:     AndroidX `edit` opens an editor, runs one receiver lambda, and applies it.
// Why:      Migration and save each update related keys in one batch.
//
// In TS you'd write (pseudocode):
// ```ts
// import { edit } from "androidx/core/content";
// ```
import androidx.core.content.edit

// What:     PlaybackMode is the new four-state model and owns wire decoding helpers.
// Why:      Persistence converts old fields into this value before returning a Session.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PlaybackMode } from "./core/PlaybackMode";
// ```
import dev.monochromatic.musicplayer.core.PlaybackMode

// What:     Session is the immutable pure value written and restored here.
// Why:      Storage code should not expose raw keys to the controller.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Session } from "./core/Session";
// ```
import dev.monochromatic.musicplayer.core.Session

// What:     `import android.util.Log` writes the warning when a stored template is replaced by its default.
// Why:      A corrupt or refused stored text must leave a trace, so the fallback is visible in the log.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `TemplateKind` and the template engine types validate stored template text.
// Why:      A stored template is used only when it parses and applies, so the store needs the engine and its samples.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TemplateKind, evaluateTemplate, parseTemplate } from "./core/TrackTemplate";
// ```
import dev.monochromatic.musicplayer.core.PlayingTrackFields
import dev.monochromatic.musicplayer.core.TEMPLATE_SAMPLE_TRACKS
import dev.monochromatic.musicplayer.core.TemplateKind
import dev.monochromatic.musicplayer.core.TemplateResult
import dev.monochromatic.musicplayer.core.TemplateTrack
import dev.monochromatic.musicplayer.core.TrackFields
import dev.monochromatic.musicplayer.core.evaluateTemplate
import dev.monochromatic.musicplayer.core.parseTemplate
import java.util.concurrent.ConcurrentHashMap

// What:     `object` creates one stateless singleton, unlike an instantiable `class`.
// Why:      Session persistence needs no per-instance state.
//
// In TS you'd write (pseudocode):
// ```ts
// export const SessionStore = { load() {}, save() {} };
// ```
/** Loads, migrates, and saves the Android playback session. */
object SessionStore {
    /** App-private preferences file name. */
    private const val PREFS_NAME: String = "session"

    /** Selected-track content URI key. */
    private const val KEY_SELECTED: String = "selected"

    /** Raw Double-bit resume-position key. */
    private const val KEY_POSITION: String = "position_bits"

    /** Float output-gain key. */
    private const val KEY_VOLUME: String = "volume"

    /** Current single playback-mode key. */
    private const val KEY_PLAYBACK_MODE: String = "playback_mode"

    /** Former shuffle key, retained only as migration input. */
    private const val LEGACY_KEY_SHUFFLE: String = "shuffle"

    /** Former repeat-track key, retained only as migration input. */
    private const val LEGACY_KEY_REPEAT: String = "repeat_track"

    // What:     `private const val KEY_AUDIO_ACCESS_REQUESTED: String = "audio_access_requested"` names the request
    //           flag.
    // Why:      The first-run decision needs to know whether this app already asked Android for music access, because
    //           Android stops showing its prompt after repeated refusals and the flag tells the two cases apart.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const KEY_AUDIO_ACCESS_REQUESTED = "audio_access_requested";
    // ```
    /** Persisted flag that this app has already asked Android for music access. */
    private const val KEY_AUDIO_ACCESS_REQUESTED: String = "audio_access_requested"

    // What:     `private const val KEY_NOTIFICATION_ACCESS_REQUESTED: String = "notification_access_requested"` names
    //           the flag that records the one notification permission ask.
    // Why:      D107 asks for the notification permission once, so a denial must still be remembered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const KEY_NOTIFICATION_ACCESS_REQUESTED = "notification_access_requested";
    // ```
    /** Preference key of the flag that records that the app asked for the notification permission. */
    private const val KEY_NOTIFICATION_ACCESS_REQUESTED: String = "notification_access_requested"

    // What:     `private const val KEY_TEMPLATE_PREFIX: String = "template_"` starts each template's preference key.
    // Why:      Each template kind keeps its own text, so its key is this prefix followed by the kind's name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const KEY_TEMPLATE_PREFIX = "template_";
    // ```
    /** Prefix of the preference key that holds one template's stored text. */
    private const val KEY_TEMPLATE_PREFIX: String = "template_"

    // What:     `private const val KEY_AUTO_PLAY_ON_SELECT: String = "auto_play_on_select"` names the setting key.
    // Why:      D129 keeps Auto-play on select beside the templates in this preference file; a file without the key
    //           reads as on (the default), so an old file loads unchanged and nothing needs migrating.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const KEY_AUTO_PLAY_ON_SELECT = "auto_play_on_select";
    // ```
    /** Preference key of the Auto-play on select setting. */
    private const val KEY_AUTO_PLAY_ON_SELECT: String = "auto_play_on_select"

    // What:     `private val TEMPLATE_VALIDATION_TRACKS` are the stand-in tracks plus one track with no values.
    // Why:      A stored template must apply to a track whose duration and peak are unknown, which is every row
    //           before analysis, so the blank track is checked alongside the samples.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const TEMPLATE_VALIDATION_TRACKS = [...TEMPLATE_SAMPLE_TRACKS, blankTrack];
    // ```
    /** Stand-in tracks and one track with no values, which every stored template must apply to. */
    private val TEMPLATE_VALIDATION_TRACKS: List<PlayingTrackFields> = TEMPLATE_SAMPLE_TRACKS + listOf(
        PlayingTrackFields(
            track = TrackFields(
                title = "",
                file = "",
                ext = "",
                folder = "",
                path = "",
                len = null,
                peak = null,
            ),
            placeInFolder = null,
            tracksInFolder = null,
        ),
    )

    // What:     `fun load(context: Context): Session` reads primitives and returns
    //           one typed value. Nullable String reads distinguish missing keys.
    // Why:      A valid new value wins; otherwise old fields migrate once and are
    //           removed so only one playback choice remains persisted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function load(context: Context): Session { /* decode or migrate */ }
    // ```
    /** Restores a session and atomically upgrades former playback settings. */
    fun load(context: Context): Session {
        /** Preferences handle reused for all reads and any migration write. */
        val preferences: SharedPreferences = prefs(context)
        /** Current-format text, or null when this store still uses the old schema. */
        val storedMode: String? = preferences.getString(KEY_PLAYBACK_MODE, null)
        /** Exactly one decoded playback behavior. */
        val playbackMode: PlaybackMode = if (storedMode == null) {
            PlaybackMode.fromLegacy(
                preferences.getString(LEGACY_KEY_SHUFFLE, null),
                preferences.getBoolean(LEGACY_KEY_REPEAT, false),
            )
        } else {
            PlaybackMode.fromStoredName(storedMode)
        }
        if (
            storedMode == null ||
            preferences.contains(LEGACY_KEY_SHUFFLE) ||
            preferences.contains(LEGACY_KEY_REPEAT)
        ) {
            preferences.edit {
                if (storedMode == null) {
                    putString(KEY_PLAYBACK_MODE, playbackMode.storedName)
                }
                remove(LEGACY_KEY_SHUFFLE)
                remove(LEGACY_KEY_REPEAT)
            }
        }
        return Session(
            selected = preferences.getString(KEY_SELECTED, null),
            positionSecs = Double.fromBits(preferences.getLong(KEY_POSITION, 0L)),
            volume = preferences.getFloat(KEY_VOLUME, 1.0f),
            playbackMode = playbackMode,
        )
    }

    // What:     `fun save(context, session)` batches primitive writes through an
    //           editor receiver. `toRawBits` preserves the 64-bit Double exactly.
    // Why:      New saves contain one playback mode and proactively erase stale old keys.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function save(context: Context, session: Session): void { /* batch writes */ }
    // ```
    /** Persists current state using only the four-state playback schema. */
    fun save(context: Context, session: Session) {
        prefs(context).edit {
            putString(KEY_SELECTED, session.selected)
            putLong(KEY_POSITION, session.positionSecs.toRawBits())
            putFloat(KEY_VOLUME, session.volume)
            putString(KEY_PLAYBACK_MODE, session.playbackMode.storedName)
            remove(LEGACY_KEY_SHUFFLE)
            remove(LEGACY_KEY_REPEAT)
        }
    }

    // What:     `fun warmUp(context: Context)` reads the session file in full.
    // Why:      The first-run decision and the restore read it on the main thread; reading it first on a worker thread
    //           leaves it in memory, so those reads do not touch the disk (StrictMode flagged 529 ms there).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function warmUp(): void { storage.readAll(); }
    // ```
    /** Reads every saved session value, off the main thread, so a later read on the main thread finds it in memory. */
    fun warmUp(context: Context) {
        Log.i(LOG_TAG, "warm-up read ${prefs(context).all.size} saved values")
        audioAccessRequestedCache = prefs(context).getBoolean(KEY_AUDIO_ACCESS_REQUESTED, false)
        // The player host reads both templates on its first frame; checking a saved template parses and evaluates it
        // (StrictMode flagged 435 ms on the main thread), so the check runs here once and the host finds the answer.
        TemplateKind.entries.forEach { kind -> loadTemplate(context, kind) }
    }

    /** The template text of each kind as last read or written here, so the main thread never checks one again. */
    private val templateCache: ConcurrentHashMap<TemplateKind, String> = ConcurrentHashMap()

    /** The music-access request flag as last read or written here; the first-run decision asks on every frame. */
    @Volatile
    private var audioAccessRequestedCache: Boolean? = null

    // What:     `fun loadAudioAccessRequested(context: Context): Boolean` reads the request flag.
    // Why:      The first-run decision reads it on every composition, and a missing flag means no request was made yet.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function loadAudioAccessRequested(context: Context): boolean { return storage.getBoolean(KEY, false); }
    // ```
    /** Returns whether this app has already asked Android for music access, or false when nothing was saved. */
    fun loadAudioAccessRequested(context: Context): Boolean {
        /** The flag as it was last read or written in this process, or null before the first of either. */
        val known: Boolean? = audioAccessRequestedCache
        if (known != null) {
            return known
        }
        /** The flag read from the preference file. */
        val stored: Boolean = prefs(context).getBoolean(KEY_AUDIO_ACCESS_REQUESTED, false)
        audioAccessRequestedCache = stored
        return stored
    }

    // What:     `fun saveAudioAccessRequested(context: Context, requested: Boolean)` writes the request flag.
    // Why:      The flag is written before the permission prompt opens, so a process death during the prompt still
    //           leaves the request on record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function saveAudioAccessRequested(context: Context, requested: boolean): void { storage.setBoolean(KEY,
    //           requested); }
    // ```
    /** Records whether this app has asked Android for music access. */
    fun saveAudioAccessRequested(context: Context, requested: Boolean) {
        audioAccessRequestedCache = requested
        prefs(context).edit {
            putBoolean(KEY_AUDIO_ACCESS_REQUESTED, requested)
        }
    }

    // What:     `fun loadAutoPlayOnSelect(context: Context): Boolean` reads the Auto-play on select setting.
    // Why:      The host reads it when it first composes, and a missing key means on (D133). `warmUp` has already
    //           loaded the preference file, so the read is from memory and needs no cache of its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function loadAutoPlayOnSelect(context: Context): boolean { return storage.getBoolean(KEY, true); }
    // ```
    /** Returns whether selecting a track also starts it, or true when nothing was saved. */
    fun loadAutoPlayOnSelect(context: Context): Boolean =
        prefs(context).getBoolean(KEY_AUTO_PLAY_ON_SELECT, true)

    // What:     `fun saveAutoPlayOnSelect(context: Context, enabled: Boolean)` writes the setting.
    // Why:      The Settings switch calls this on every toggle, so the choice survives a restart; it sits outside
    //           `Session`, so the controller's session saves cannot overwrite it with an older value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function saveAutoPlayOnSelect(context: Context, enabled: boolean): void { storage.setBoolean(KEY, enabled); }
    // ```
    /** Persists the Auto-play on select setting under its own preference key. */
    fun saveAutoPlayOnSelect(context: Context, enabled: Boolean) {
        prefs(context).edit {
            putBoolean(KEY_AUTO_PLAY_ON_SELECT, enabled)
        }
    }

    // What:     `fun loadNotificationAccessRequested(context: Context): Boolean` reads the notification ask flag.
    // Why:      The analysis asks for the notification permission once, so a missing flag means not asked yet.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function loadNotificationAccessRequested(context: Context): boolean { return storage.getBoolean(KEY, false); }
    // ```
    /** Returns whether this app has already shown the notification permission prompt, or false when unsaved. */
    fun loadNotificationAccessRequested(context: Context): Boolean =
        prefs(context).getBoolean(KEY_NOTIFICATION_ACCESS_REQUESTED, false)

    // What:     `fun saveNotificationAccessRequested(context: Context, requested: Boolean)` writes the ask flag.
    // Why:      The flag is written before the prompt opens, so a denial or a process death never asks again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function saveNotificationAccessRequested(context: Context, requested: boolean): void {
    //   storage.setBoolean(KEY, requested);
    // }
    // ```
    /** Records whether this app has shown the notification permission prompt. */
    fun saveNotificationAccessRequested(context: Context, requested: Boolean) {
        prefs(context).edit {
            putBoolean(KEY_NOTIFICATION_ACCESS_REQUESTED, requested)
        }
    }

    // What:     `fun loadTemplate(context: Context, kind: TemplateKind): String` reads one template's stored text.
    // Why:      The rows, the deck line, and Settings all read a template, and each gets only a text that applies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function loadTemplate(context: Context, kind: TemplateKind): string { /* stored text, or the default */ }
    // ```
    /** Returns the stored text of one template, or its default when nothing valid is stored. */
    fun loadTemplate(context: Context, kind: TemplateKind): String {
        /** Text this kind answered before, which a read or a save in this process already settled. */
        val cached: String? = templateCache[kind]
        if (cached != null) {
            return cached
        }
        /** Text saved under this template's key, or null when nothing is saved. */
        val stored: String? = prefs(context).getString(templateKey(kind), null)
        /** Text the screens use: the stored text when it applies, otherwise the kind's default. */
        val text: String = templateOrDefault(stored = stored, kind = kind)
        if (stored != null && text != stored) {
            Log.w(LOG_TAG, "stored template did not apply, default used kind=${kind.name}")
        }
        templateCache[kind] = text
        return text
    }

    // What:     `fun saveTemplate(context: Context, kind: TemplateKind, text: String)` writes one template's text.
    // Why:      The editor calls this only for a draft that applies, so the store holds no refused template.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function saveTemplate(context: Context, kind: TemplateKind, text: string): void { /* one key per kind */ }
    // ```
    /** Persists the text of one template under its own preference key. */
    fun saveTemplate(context: Context, kind: TemplateKind, text: String) {
        templateCache[kind] = text
        prefs(context).edit {
            putString(templateKey(kind), text)
        }
    }

    // What:     `internal fun templateOrDefault(stored: String?, kind: TemplateKind): String` picks the text to use.
    // Why:      A missing text or one that does not apply falls back to the default, so no screen receives a refusal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function templateOrDefault(stored: string | null, kind: TemplateKind): string {
    //   return stored !== null && templateApplies(stored, kind) ? stored : kind.defaultTemplate;
    // }
    // ```
    /** Returns the stored text when it parses and applies to every validation track, or the default otherwise. */
    internal fun templateOrDefault(stored: String?, kind: TemplateKind): String {
        if (stored == null) {
            return kind.defaultTemplate
        }
        if (!templateApplies(text = stored, kind = kind)) {
            return kind.defaultTemplate
        }
        return stored
    }

    // What:     `internal fun templateApplies(text: String, kind: TemplateKind): Boolean` checks one template text.
    // Why:      The check is the editor's own rule, a text that parses and shows on every track, so a saved text
    //           can never be refused later on a row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function templateApplies(text: string, kind: TemplateKind): boolean { /* parse, then every track shows */ }
    // ```
    /** Whether the text parses and shows on every validation track for the given template kind. */
    internal fun templateApplies(text: String, kind: TemplateKind): Boolean {
        if (parseTemplate(text).errors.isNotEmpty()) {
            return false
        }
        return TEMPLATE_VALIDATION_TRACKS.all { track ->
            evaluateTemplate(text, templateTrackFor(kind, track), kind.fields) is TemplateResult.Shown
        }
    }

    // What:     `private fun templateTrackFor(kind, track): TemplateTrack` picks the field source a template reads.
    // Why:      Track rows read the track alone, and the playing track also reads its place in the folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const templateTrackFor = (kind, track) => (kind === "trackRows" ? track.track : track);
    // ```
    /** The field source that one template kind reads for a validation track. */
    private fun templateTrackFor(kind: TemplateKind, track: PlayingTrackFields): TemplateTrack = when (kind) {
        TemplateKind.TRACK_ROWS -> track.track
        TemplateKind.PLAYING_TRACK -> track
    }

    // What:     `private fun templateKey(kind: TemplateKind): String` names the preference key of one template.
    // Why:      Every template reads and writes through the same key rule, so the two directions cannot drift.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const templateKey = (kind: TemplateKind): string => `template_${kind.name}`;
    // ```
    /** Returns the preference key that holds one template's text. */
    private fun templateKey(kind: TemplateKind): String = KEY_TEMPLATE_PREFIX + kind.name

    // What:     `getSharedPreferences` opens one app-private named primitive store.
    // Why:      All operations must resolve the same file with private visibility.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function prefs(context: Context): SharedPreferences { /* open private store */ }
    // ```
    /** Opens the app-private session preference file. */
    private fun prefs(context: Context): SharedPreferences =
        context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
}
