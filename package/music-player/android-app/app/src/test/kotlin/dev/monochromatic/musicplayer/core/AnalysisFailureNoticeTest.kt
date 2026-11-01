package dev.monochromatic.musicplayer.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** D123: the wording of the analysis-failure notification, the decision to post it, and the no-fallback delivery. */
class AnalysisFailureNoticeTest {
    /** Calls the decision with named arguments, every input defaulting to the state in which posting works. */
    private fun verdict(
        sdkInt: Int = 36,
        permissionGranted: Boolean = true,
        notificationsEnabled: Boolean = true,
        channelDisabled: Boolean = false,
    ): NotificationPostVerdict = notificationPostVerdict(
        sdkInt = sdkInt,
        permissionGranted = permissionGranted,
        notificationsEnabled = notificationsEnabled,
        channelDisabled = channelDisabled,
    )

    /** What a [deliverAnalysisFailureNotice] call did, recorded by the lambdas of one case. */
    private class Delivery {
        /** Notices handed to the poster, in order. */
        val posted = ArrayList<AnalysisFailureNotice>()

        /** Warning messages, in order. */
        val warnings = ArrayList<String>()

        /** Failures that came with the warnings, in order. */
        val causes = ArrayList<Throwable?>()
    }

    /** Runs one delivery with the given verdict and poster and returns what it returned and recorded. */
    private fun deliver(
        names: List<String>,
        verdict: () -> NotificationPostVerdict = { NotificationPostVerdict.POST },
        post: (AnalysisFailureNotice, Delivery) -> Unit = { notice, delivery -> delivery.posted.add(notice) },
    ): Pair<Boolean, Delivery> {
        val delivery = Delivery()
        val result = deliverAnalysisFailureNotice(
            fileNames = names,
            verdict = verdict,
            post = { notice -> post(notice, delivery) },
            warn = { message, cause ->
                delivery.warnings.add(message)
                delivery.causes.add(cause)
            },
        )
        return result to delivery
    }

    @Test
    fun oneFileIsNamedInTheTitleAndTheRetrySentenceIsTheText() {
        val notice = analysisFailureNotice(listOf("broken.mp3"))
        assertEquals("Could not analyze broken.mp3.", notice.title)
        assertEquals("Its file could not be decoded. The app tries again when your library changes.", notice.text)
    }

    @Test
    fun severalFilesCollapseToACountWithTheFirstFileNamed() {
        val notice = analysisFailureNotice(listOf("a.mp3", "b.mp3", "c.mp3"))
        assertEquals("Could not analyze 3 files, including a.mp3.", notice.title)
        assertEquals("They could not be decoded. The app tries again when your library changes.", notice.text)
    }

    @Test
    fun twoFilesAlreadyUseTheCountWording() {
        val notice = analysisFailureNotice(listOf("a.mp3", "b.mp3"))
        assertEquals("Could not analyze 2 files, including a.mp3.", notice.title)
    }

    @Test
    fun theWordingIsTheOldInAppMessageWordForWord() {
        val notice = analysisFailureNotice(listOf("broken.mp3"))
        assertEquals(
            "Could not analyze broken.mp3.\n" +
                "Its file could not be decoded. The app tries again when your library changes.",
            notice.title + "\n" + notice.text,
        )
    }

    @Test
    fun aLongFileNameIsShortenedWithAnEllipsis() {
        val name = "x".repeat(60) + ".mp3"
        val notice = analysisFailureNotice(listOf(name))
        assertEquals("Could not analyze " + "x".repeat(47) + "….", notice.title)
        assertTrue(analysisFailureNotice(listOf(name, "b.mp3")).title.contains("x".repeat(47) + "…"))
    }

    @Test
    fun noFileIsRefused() {
        val refused = runCatching { analysisFailureNotice(emptyList()) }.exceptionOrNull()
        assertTrue(refused is IllegalArgumentException)
    }

    @Test
    fun everyConditionHoldingMeansPost() {
        assertEquals(NotificationPostVerdict.POST, verdict())
    }

    @Test
    fun aDeniedPermissionStopsThePostFromAndroid13() {
        assertEquals(NotificationPostVerdict.PERMISSION_DENIED, verdict(sdkInt = 33, permissionGranted = false))
        assertEquals(NotificationPostVerdict.PERMISSION_DENIED, verdict(sdkInt = 36, permissionGranted = false))
    }

    @Test
    fun theMissingPermissionIsIgnoredBelowAndroid13() {
        assertEquals(NotificationPostVerdict.POST, verdict(sdkInt = 32, permissionGranted = false))
        assertEquals(NotificationPostVerdict.POST, verdict(sdkInt = 26, permissionGranted = false))
    }

    @Test
    fun notificationsTurnedOffStopThePostOnEveryLevel() {
        assertEquals(NotificationPostVerdict.NOTIFICATIONS_DISABLED, verdict(notificationsEnabled = false))
        val old = verdict(sdkInt = 28, notificationsEnabled = false)
        assertEquals(NotificationPostVerdict.NOTIFICATIONS_DISABLED, old)
    }

    @Test
    fun aChannelTurnedOffStopsThePost() {
        assertEquals(NotificationPostVerdict.CHANNEL_DISABLED, verdict(channelDisabled = true))
        assertEquals(NotificationPostVerdict.CHANNEL_DISABLED, verdict(sdkInt = 26, channelDisabled = true))
    }

    @Test
    fun theFirstFailingConditionIsTheReason() {
        assertEquals(
            NotificationPostVerdict.PERMISSION_DENIED,
            verdict(permissionGranted = false, notificationsEnabled = false, channelDisabled = true),
        )
        assertEquals(
            NotificationPostVerdict.NOTIFICATIONS_DISABLED,
            verdict(notificationsEnabled = false, channelDisabled = true),
        )
    }

    @Test
    fun thePermissionThresholdIsTheSameConstantTheAskPolicyUses() {
        val first = NOTIFICATION_PERMISSION_MIN_SDK
        assertEquals(NotificationPostVerdict.PERMISSION_DENIED, verdict(sdkInt = first, permissionGranted = false))
        assertEquals(NotificationPostVerdict.POST, verdict(sdkInt = first - 1, permissionGranted = false))
    }

    @Test
    fun severalFilesOfOneReportMakeOneNotification() {
        val (result, delivery) = deliver(listOf("a.mp3", "b.mp3", "c.mp3"))
        assertTrue(result)
        assertEquals(1, delivery.posted.size)
        assertEquals("Could not analyze 3 files, including a.mp3.", delivery.posted.single().title)
        assertTrue(delivery.warnings.isEmpty())
    }

    @Test
    fun anEmptyReportPostsAndLogsNothing() {
        var asked = false
        val (result, delivery) = deliver(emptyList(), verdict = {
            asked = true
            NotificationPostVerdict.POST
        })
        assertFalse(result)
        assertFalse(asked)
        assertTrue(delivery.posted.isEmpty())
        assertTrue(delivery.warnings.isEmpty())
    }

    @Test
    fun everyRefusedVerdictPostsNothingAndLogsOneWarningWithTheReasonAndNoFileName() {
        for (refusal in NotificationPostVerdict.entries.filter { verdict -> verdict != NotificationPostVerdict.POST }) {
            val (result, delivery) = deliver(listOf("secret-name.mp3", "other.mp3"), verdict = { refusal })
            assertFalse(result)
            assertTrue(delivery.posted.isEmpty())
            assertEquals(1, delivery.warnings.size)
            assertTrue(delivery.warnings.single().contains(refusal.name))
            assertTrue(delivery.warnings.single().contains("2 files"))
            assertFalse(delivery.warnings.single().contains("secret-name"))
            assertEquals(listOf<Throwable?>(null), delivery.causes)
        }
    }

    @Test
    fun aPostThatFailsIsLoggedAndNeverThrownToTheSweep() {
        val failure = IllegalStateException("system service died")
        val (result, delivery) = deliver(listOf("secret-name.mp3"), post = { _, _ -> throw failure })
        assertFalse(result)
        assertEquals(1, delivery.warnings.size)
        assertFalse(delivery.warnings.single().contains("secret-name"))
        assertEquals(listOf<Throwable?>(failure), delivery.causes)
    }

    @Test
    fun aVerdictThatFailsIsLoggedPostsNothingAndNeverThrownToTheSweep() {
        val failure = SecurityException("no access")
        val (result, delivery) = deliver(listOf("a.mp3"), verdict = { throw failure })
        assertFalse(result)
        assertTrue(delivery.posted.isEmpty())
        assertEquals(1, delivery.warnings.size)
        assertEquals(listOf<Throwable?>(failure), delivery.causes)
    }

    @Test
    fun aFileThatFailedBeforeIsNotReportedAgainAcrossTwoSweeps() {
        val sweeps = 2
        var notifications = 0
        var ledger = SweepLedger.EMPTY
        repeat(sweeps) {
            val record = ledger.record(done = emptyList(), failures = listOf("content://invented/1" to 7L))
            ledger = record.ledger
            val names = record.newFailures.map { uri -> trackFileNameOf(uri) }
            val (_, delivery) = deliver(names)
            notifications += delivery.posted.size
        }
        assertEquals(1, notifications)
    }

    /** D122 and D123: one file passed by the player is named in the title, with the tap-time reason as the text. */
    @Test
    fun anUnplayableFileIsNamedWithTheWordsOfTheTapMessage() {
        val notice = unplayableFilesNotice(listOf("broken.mp3"), timedOut = false)
        assertEquals("Could not play broken.mp3.", notice.title)
        assertEquals("Its file could not be opened.", notice.text)
    }

    /** Several files passed in one walk make one notice that counts them and names the first. */
    @Test
    fun severalUnplayableFilesCollapseToACountWithTheFirstNamed() {
        val notice = unplayableFilesNotice(listOf("a.mp3", "b.mp3", "c.mp3"), timedOut = false)
        assertEquals("Could not play 3 files, including a.mp3.", notice.title)
        assertEquals("They could not be opened.", notice.text)
    }

    /** A walk that ended on a timeout says the music library was slow, for one file and for several. */
    @Test
    fun aTimeoutSaysTheLibraryDidNotAnswer() {
        val slow = "The music library did not answer in time. Try again in a moment."
        assertEquals(slow, unplayableFilesNotice(listOf("a.mp3"), timedOut = true).text)
        assertEquals(slow, unplayableFilesNotice(listOf("a.mp3", "b.mp3"), timedOut = true).text)
    }

    /** A long file name is shortened as in the tap-time message. */
    @Test
    fun aLongUnplayableFileNameIsShortened() {
        val name = "x".repeat(60) + ".mp3"
        assertEquals(
            "Could not play " + "x".repeat(47) + "….",
            unplayableFilesNotice(listOf(name), timedOut = false).title,
        )
    }

    /** A report with no file is a caller error. */
    @Test
    fun noUnplayableFileIsRefused() {
        val refused = runCatching { unplayableFilesNotice(emptyList(), timedOut = false) }.exceptionOrNull()
        assertTrue(refused is IllegalArgumentException)
    }

    /** The delivery posts the notice it is given and names its own subject in the warning, with no file name. */
    @Test
    fun aDeliveryPostsTheGivenNoticeAndWarnsInItsOwnWords() {
        /** Notices handed to the poster. */
        val posted = ArrayList<AnalysisFailureNotice>()
        /** Warning lines. */
        val warnings = ArrayList<String>()
        val ok = deliverAnalysisFailureNotice(
            fileNames = listOf("a.mp3"),
            verdict = { NotificationPostVerdict.POST },
            post = { notice -> posted.add(notice) },
            warn = { message, _ -> warnings.add(message) },
            noticeFor = { names -> unplayableFilesNotice(names, timedOut = false) },
            subject = "unplayable files",
        )
        assertTrue(ok)
        assertEquals("Could not play a.mp3.", posted.single().title)
        deliverAnalysisFailureNotice(
            fileNames = listOf("secret-name.mp3"),
            verdict = { NotificationPostVerdict.CHANNEL_DISABLED },
            post = { notice -> posted.add(notice) },
            warn = { message, _ -> warnings.add(message) },
            noticeFor = { names -> unplayableFilesNotice(names, timedOut = false) },
            subject = "unplayable files",
        )
        assertEquals(1, posted.size)
        assertEquals("unplayable files notification not posted for 1 files: CHANNEL_DISABLED", warnings.single())
    }
}
