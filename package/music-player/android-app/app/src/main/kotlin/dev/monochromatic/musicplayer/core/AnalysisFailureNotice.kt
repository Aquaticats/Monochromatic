// What the analysis-failure notification says, whether it can be posted, and what happens when it cannot (D123).

// What:     `package ...core` places the wording and the decisions beside the other platform-independent logic.
// Why:      Everything here is a pure function of its inputs, so JVM tests reach every branch without a device.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `data class AnalysisFailureNotice(title, text)` is what the notification shows.
// Why:      The notification has a title and a body, while the old in-app message was one text of two lines; the
//           first line is the title and the rest is the body, with the same words.
//
// In TS you'd write (pseudocode):
// ```ts
// type AnalysisFailureNotice = { title: string; text: string };
// ```
/** The two texts of the analysis-failure notification. */
data class AnalysisFailureNotice(
    /** First line: which file, or how many files, could not be analyzed. */
    val title: String,
    /** Second line: why, and what the app does next. */
    val text: String,
)

// What:     `fun analysisFailureNotice(fileNames: List<String>): AnalysisFailureNotice` words the failure.
// Why:      D113 and D123: a file that cannot be decoded is reported once, in plain words, with no retry button; the
//           app tries it again when the library next changes or at the next start, and the text says so. One file is
//           named; several collapse to a count with the first one named.
//
// In TS you'd write (pseudocode):
// ```ts
// function analysisFailureNotice(names: string[]): AnalysisFailureNotice { /* one name, or a count and the first */ }
// ```
/**
 * Returns the notification texts for one or more files whose true peak could not be measured.
 *
 * @param fileNames bare file names of the failed files, at least one
 * @return the title and the text of the notification
 */
fun analysisFailureNotice(fileNames: List<String>): AnalysisFailureNotice {
    require(fileNames.isNotEmpty()) { "an analysis failure names at least one file" }
    /** The first file named in the notice. */
    val first: String = shortName(fileNames.first())
    /** What happens next, the same for every count. */
    val next = "The app tries again when your library changes."
    if (fileNames.size == 1) {
        return AnalysisFailureNotice(
            title = "Could not analyze $first.",
            text = "Its file could not be decoded. $next",
        )
    }
    return AnalysisFailureNotice(
        title = "Could not analyze ${fileNames.size} files, including $first.",
        text = "They could not be decoded. $next",
    )
}

// What:     `enum class NotificationPostVerdict` says whether a notification can be posted, or the first reason not.
// Why:      The log line for a refused post names the reason, and a test can pin each reason separately.
//
// In TS you'd write (pseudocode):
// ```ts
// type NotificationPostVerdict = "post" | "permissionDenied" | "notificationsDisabled" | "channelDisabled";
// ```
/** Whether a notification can be posted now, and if not, which condition stops it. */
enum class NotificationPostVerdict {
    /** Every condition holds, so the post can be made. */
    POST,

    /** Android 13 or later, and the user has not granted the notification permission. */
    PERMISSION_DENIED,

    /** The user turned off all notifications of the app. */
    NOTIFICATIONS_DISABLED,

    /** The user turned off the channel the notification would be posted on. */
    CHANNEL_DISABLED,
}

// What:     `fun notificationPostVerdict(...)` decides whether the analysis-failure notification can be posted.
// Why:      D123 has no fallback: when any condition fails the message goes nowhere but the log. The permission is
//           only a condition from Android 13, where it became a runtime prompt (`NOTIFICATION_PERMISSION_MIN_SDK`);
//           the app-wide switch and the channel switch are separate inputs, because the user can turn off either one
//           alone.
//
// In TS you'd write (pseudocode):
// ```ts
// function notificationPostVerdict(a): Verdict {
//   if (a.sdkInt >= 33 && !a.permissionGranted) return "permissionDenied";
//   if (!a.notificationsEnabled) return "notificationsDisabled";
//   return a.channelDisabled ? "channelDisabled" : "post";
// }
// ```
/**
 * Decides whether a notification can be posted.
 *
 * @param sdkInt Android API level of the device
 * @param permissionGranted whether the notification permission is held (ignored below Android 13)
 * @param notificationsEnabled whether the user left notifications of the app switched on
 * @param channelDisabled whether the user switched off the channel the notification is posted on
 * @return [NotificationPostVerdict.POST], or the first condition that stops the post
 */
fun notificationPostVerdict(
    sdkInt: Int,
    permissionGranted: Boolean,
    notificationsEnabled: Boolean,
    channelDisabled: Boolean,
): NotificationPostVerdict = when {
    sdkInt >= NOTIFICATION_PERMISSION_MIN_SDK && !permissionGranted -> NotificationPostVerdict.PERMISSION_DENIED
    !notificationsEnabled -> NotificationPostVerdict.NOTIFICATIONS_DISABLED
    channelDisabled -> NotificationPostVerdict.CHANNEL_DISABLED
    else -> NotificationPostVerdict.POST
}

// What:     `fun unplayableFilesNotice(fileNames, timedOut)` words the report of files the player passed (D122, D123).
// Why:      A track the player moved to by itself and could not open is reported as a system notification, never as
//           the in-app message of a tap. The words are those of the tap-time message (`trackLoadFailureMessage`), split
//           into a title and a text; several files collapse to a count that names the first, and a timeout says the
//           music library was slow, because no file was shown to be broken.
//
// In TS you'd write (pseudocode):
// ```ts
// function unplayableFilesNotice(names: string[], timedOut: boolean): AnalysisFailureNotice { /* count or name */ }
// ```
/**
 * Returns the notification texts for one or more files the player passed because it could not open them.
 *
 * @param fileNames bare file names in the order met, at least one
 * @param timedOut whether the music library did not answer in time, which replaces the reason in the text
 * @return the title and the text of the notification
 */
fun unplayableFilesNotice(fileNames: List<String>, timedOut: Boolean): AnalysisFailureNotice {
    require(fileNames.isNotEmpty()) { "an unplayable-files report names at least one file" }
    /** The first file named in the notice. */
    val first: String = shortName(fileNames.first())
    return AnalysisFailureNotice(
        title = if (fileNames.size == 1) {
            "Could not play $first."
        } else {
            "Could not play ${fileNames.size} files, including $first."
        },
        text = when {
            timedOut -> "The music library did not answer in time. Try again in a moment."
            fileNames.size == 1 -> "Its file could not be opened."
            else -> "They could not be opened."
        },
    )
}

// What:     `fun deliverAnalysisFailureNotice(...)` posts the notice when it can be posted and otherwise only logs.
// Why:      D123: the notification is the only place the failure is shown. When it cannot be posted, the sweep gets
//           one warning in the log and nothing in the app; a failure while deciding or posting is a refused post too,
//           and it never reaches the sweep that called it.
//
// In TS you'd write (pseudocode):
// ```ts
// function deliver(names, verdict, post, warn): boolean {
//   if (names.length === 0) return false;
//   try { v = verdict(); } catch (e) { warn("...", e); return false; }
//   if (v !== "post") { warn(`not posted: ${v}`, null); return false; }
//   try { post(analysisFailureNotice(names)); return true; } catch (e) { warn("...", e); return false; }
// }
// ```
/**
 * Posts the analysis-failure notice for [fileNames] when the verdict allows it.
 *
 * The warnings carry the reason and the number of files, never a file name.
 *
 * @param fileNames bare names of the files that failed for the first time; an empty list posts nothing
 * @param verdict reads the platform state and decides whether the post can be made
 * @param post posts the notice as one system notification
 * @param warn writes one warning line, with the failure that caused it when there is one
 * @param noticeFor words the notice from the file names; the analysis failure wording unless the caller passes its own
 * @param subject what the warnings call the notification, so a log line says which report was not posted
 * @return true only when the notice was handed to [post] and [post] returned
 */
fun deliverAnalysisFailureNotice(
    fileNames: List<String>,
    verdict: () -> NotificationPostVerdict,
    post: (AnalysisFailureNotice) -> Unit,
    warn: (message: String, cause: Throwable?) -> Unit,
    noticeFor: (List<String>) -> AnalysisFailureNotice = ::analysisFailureNotice,
    subject: String = "analysis failure",
): Boolean {
    if (fileNames.isEmpty()) {
        return false
    }
    /** The decision, or null when reading the platform state failed. */
    val decided: NotificationPostVerdict? = try {
        verdict()
    } catch (expectedFailure: RuntimeException) {
        warn("$subject notification not posted: the notification state could not be read", expectedFailure)
        null
    }
    if (decided == null) {
        return false
    }
    if (decided != NotificationPostVerdict.POST) {
        warn("$subject notification not posted for ${fileNames.size} files: ${decided.name}", null)
        return false
    }
    try {
        post(noticeFor(fileNames))
    } catch (expectedFailure: RuntimeException) {
        warn("$subject notification for ${fileNames.size} files could not be posted", expectedFailure)
        return false
    }
    return true
}
