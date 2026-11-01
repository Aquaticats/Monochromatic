// The Android side of the true-peak analysis notification (decision D107): channel, builder and actions.

// What:     `package dev.monochromatic.musicplayer` places the builder beside the sweep service that posts it.
// Why:      Everything with a decision in it lives in `core`; this file only maps content onto Android types.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import dev.monochromatic.musicplayer.core.ANALYSIS_NOTIFICATION_TITLE
import dev.monochromatic.musicplayer.core.AnalysisNotificationContent

/** Id of the notification channel that carries only the analysis notification. */
internal const val ANALYSIS_CHANNEL_ID: String = "analysis"

/** Channel name that the system settings show to the user. */
private const val ANALYSIS_CHANNEL_NAME: String = "True-peak analysis"

/** Id of the channel that earlier builds created, which is deleted so settings does not list a stale one. */
private const val LEGACY_CHANNEL_ID: String = "peak-sweep"

/** Fixed id of the single analysis notification. */
internal const val ANALYSIS_NOTIFICATION_ID: Int = 4201

/** Largest value of the progress bar, which the content's whole percent fills. */
private const val PROGRESS_MAX: Int = 100

/** Label of the button that pauses the analysis. */
private const val PAUSE_LABEL: String = "Pause"

/** Label of the button that resumes a paused analysis. */
private const val RESUME_LABEL: String = "Resume"

// What:     `fun deleteLegacyAnalysisChannel(context)` removes the channel that builds before D107 created.
// Why:      A person who upgrades after the first analysis finished never starts another sweep, so the sweep
//           service never runs its channel setup and the old "Indexing audio levels" entry would stay in the
//           system settings for good. The activity calls this on every launch; deleting a missing channel is a
//           no-op.
//
// In TS you'd write (pseudocode):
// ```ts
// function deleteLegacyAnalysisChannel(context) { manager.deleteChannel("peak-sweep"); }
// ```
/**
 * Deletes the `peak-sweep` notification channel of earlier builds, so the system settings list only the current
 * analysis channel. Safe to call when the channel does not exist.
 *
 * @param context any context of this app
 */
internal fun deleteLegacyAnalysisChannel(context: Context) {
    context.getSystemService(NotificationManager::class.java).deleteNotificationChannel(LEGACY_CHANNEL_ID)
}

// What:     `fun ensureAnalysisChannel(context)` creates the silent low-importance channel and drops the old one.
// Why:      D107 wants a silent, low-priority notification on its own channel. Importance low already means no
//           sound; the explicit sound and vibration settings keep it silent whatever the system default is.
//
// In TS you'd write (pseudocode):
// ```ts
// function ensureAnalysisChannel(context) {
//   manager.deleteChannel("peak-sweep");
//   manager.createChannel({ id: "analysis", ... });
// }
// ```
/**
 * Creates the `analysis` notification channel when it is missing and deletes the channel of earlier builds.
 *
 * @param context any context of this app
 */
internal fun ensureAnalysisChannel(context: Context) {
    deleteLegacyAnalysisChannel(context)
    /** The system notification service that owns the channels. */
    val manager: NotificationManager = context.getSystemService(NotificationManager::class.java)
    if (manager.getNotificationChannel(ANALYSIS_CHANNEL_ID) != null) {
        return
    }
    /** The silent channel, with no sound, no vibration and no launcher badge. */
    val channel = NotificationChannel(ANALYSIS_CHANNEL_ID, ANALYSIS_CHANNEL_NAME, NotificationManager.IMPORTANCE_LOW)
    channel.setSound(null, null)
    channel.enableVibration(false)
    channel.setShowBadge(false)
    manager.createNotificationChannel(channel)
}

// What:     `fun buildAnalysisNotification(context, content)` maps the content onto a notification.
// Why:      The title is fixed, the text is the count, and the bar is determinate once the total is known. The
//           notification is ongoing and alerts once at most (never, on the silent channel); the one action flips
//           between Pause and Resume with the content's pause flag.
//
// In TS you'd write (pseudocode):
// ```ts
// function buildAnalysisNotification(context, content): Notification { /* title, text, progress, action */ }
// ```
/**
 * Builds the analysis notification for [content].
 *
 * @param context any context of this app
 * @param content what the notification shows
 * @return the notification, ready to post or to hand to `startForeground`
 */
internal fun buildAnalysisNotification(context: Context, content: AnalysisNotificationContent): Notification {
    /** Builder on the silent channel with the fixed parts of the notification. */
    val builder: NotificationCompat.Builder = NotificationCompat.Builder(context, ANALYSIS_CHANNEL_ID)
        .setContentTitle(ANALYSIS_NOTIFICATION_TITLE)
        .setSmallIcon(R.drawable.ic_stat_analysis)
        .setCategory(NotificationCompat.CATEGORY_PROGRESS)
        .setPriority(NotificationCompat.PRIORITY_LOW)
        .setSilent(true)
        .setOnlyAlertOnce(true)
        .setOngoing(true)
        .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
        .setProgress(PROGRESS_MAX, content.percent, content.indeterminate)
        .addAction(pauseAction(context = context, paused = content.paused))
    if (content.text.isNotEmpty()) {
        builder.setContentText(content.text)
    }
    openAppIntent(context)?.let { openApp -> builder.setContentIntent(openApp) }
    return builder.build()
}

// What:     `private fun pauseAction(context, paused)` builds the one action button.
// Why:      The button is the D26 control: Pause while the analysis runs, Resume while it is paused. It sends an
//           explicit, immutable broadcast to this app's own receiver.
//
// In TS you'd write (pseudocode):
// ```ts
// const pauseAction = (paused) => ({
//   label: paused ? "Resume" : "Pause",
//   onPress: broadcast(paused ? RESUME : PAUSE),
// });
// ```
/** Returns the Pause button, or the Resume button when the analysis is paused. */
private fun pauseAction(context: Context, paused: Boolean): NotificationCompat.Action {
    /** Broadcast action that the pressed button sends. */
    val action: String = if (paused) ACTION_ANALYSIS_RESUME else ACTION_ANALYSIS_PAUSE
    /** Explicit intent for this app's own receiver, so no other app can receive or send it. */
    val intent: Intent = Intent(context, AnalysisActionReceiver::class.java).setAction(action)
    /** Immutable pending broadcast, because the receiver needs nothing filled in later. */
    val pending: PendingIntent = PendingIntent.getBroadcast(
        context,
        0,
        intent,
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )
    /** Framework media icon, because the app ships no drawable resources of its own. */
    val icon: Int = if (paused) android.R.drawable.ic_media_play else android.R.drawable.ic_media_pause
    return NotificationCompat.Action.Builder(icon, if (paused) RESUME_LABEL else PAUSE_LABEL, pending).build()
}

// What:     `internal fun openAppIntent(context)` builds the tap target of a notification.
// Why:      Tapping the notification brings the player back, the way tapping any media notification does; the
//           analysis-failure notification (D123) uses the same target.
//
// In TS you'd write (pseudocode):
// ```ts
// const openAppIntent = (context) => launchIntentOf(context.packageName);
// ```
/** Returns a pending intent that opens the app, or null when the app has no launcher activity. */
internal fun openAppIntent(context: Context): PendingIntent? {
    /** Launcher intent of this app, which brings the existing task forward. */
    val launch: Intent = context.packageManager.getLaunchIntentForPackage(context.packageName) ?: return null
    return PendingIntent.getActivity(
        context,
        0,
        launch,
        PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
    )
}
