// Instrumented cases of a load whose file opens but does not decode, run against the real native engine. The
// files are invented: a tone written by the test and a block of bytes that is not audio.
//
// What:     `package dev.monochromatic.musicplayer` places the cases beside `RustEngine`.
// Why:      The cases construct the engine and read its callbacks, which the package exposes.
//
// In TS you'd write (pseudocode):
// ```ts
// // The file path is the module.
// ```
package dev.monochromatic.musicplayer

import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicReference

/** Sample rate in hertz of the invented tone. */
private const val TONE_RATE: Int = 44_100

/** Length in seconds of the invented tone. */
private const val TONE_SECONDS: Int = 2

/** Longest wait in milliseconds for an answer from the engine worker. */
private const val ANSWER_WAIT_MS: Long = 10_000L

/** Interval in milliseconds between checks for an answer. */
private const val ANSWER_STEP_MS: Long = 50L

/** Time in milliseconds a good load is given to report a failure it should not have. */
private const val QUIET_WAIT_MS: Long = 2_000L

/**
 * A file that opens but does not decode must end in `onLoadFailed`, and the deck must not keep the previous track's
 * length. Before the engine reported such failures the worker dropped the load and the app stayed silent.
 */
class RustEngineLoadFailureTest {
    /** Context of the app under test. */
    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext

    /** Writes an invented mono 16-bit tone as a WAV file in the app cache and returns its path. */
    private fun writeTone(name: String): String {
        /** Count of sample frames in the tone. */
        val frames: Int = TONE_RATE * TONE_SECONDS
        /** Bytes of the PCM payload. */
        val dataBytes: Int = frames * 2
        /** Little-endian header followed by the samples. */
        val bytes: ByteBuffer = ByteBuffer.allocate(44 + dataBytes).order(ByteOrder.LITTLE_ENDIAN)
        bytes.put("RIFF".toByteArray()).putInt(36 + dataBytes).put("WAVEfmt ".toByteArray()).putInt(16)
        bytes.putShort(1).putShort(1).putInt(TONE_RATE).putInt(TONE_RATE * 2).putShort(2).putShort(16)
        bytes.put("data".toByteArray()).putInt(dataBytes)
        for (frame in 0 until frames) {
            /** Sample value of a quiet 440 hertz sine at this frame. */
            val sample: Double = Math.sin(2.0 * Math.PI * 440.0 * frame / TONE_RATE) * 0.3 * Short.MAX_VALUE
            bytes.putShort(sample.toInt().toShort())
        }
        /** Cache file that holds the tone. */
        val file = File(context.cacheDir, name)
        file.writeBytes(bytes.array())
        return file.absolutePath
    }

    /** Writes a file of bytes that no decoder accepts, under an audio name, and returns its path. */
    private fun writeNotAudio(name: String): String {
        /** Cache file that holds the invented bytes. */
        val file = File(context.cacheDir, name)
        file.writeBytes(ByteArray(NOT_AUDIO_BYTES) { index -> (index * 31 % 251).toByte() })
        return file.absolutePath
    }

    /** Runs [block] on the main thread and returns its value. */
    private fun <T> onMain(block: () -> T): T {
        /** Holds the value the main thread produced. */
        val holder = AtomicReference<T>()
        InstrumentationRegistry.getInstrumentation().runOnMainSync { holder.set(block()) }
        return holder.get()
    }

    /** Waits up to [ANSWER_WAIT_MS] for [condition] to hold, checking on the test thread. */
    private fun awaitTrue(condition: () -> Boolean): Boolean {
        /** Clock reading in milliseconds at which the wait gives up. */
        val deadline: Long = System.currentTimeMillis() + ANSWER_WAIT_MS
        while (System.currentTimeMillis() < deadline) {
            if (condition()) {
                return true
            }
            Thread.sleep(ANSWER_STEP_MS)
        }
        return condition()
    }

    /** A file that opens but does not decode reports an unreadable load, and a good load reports nothing. */
    @Test
    fun aFileThatOpensButDoesNotDecodeReportsAnUnreadableLoadAndClearsTheLength() {
        /** Path of the good tone. */
        val tone: String = writeTone("invented-tone.wav")
        /** Path of the bytes that are not audio. */
        val notAudio: String = writeNotAudio("invented-not-audio.flac")
        /** Failures the engine reported, as the URI and the reason. */
        val failures: MutableList<Pair<String, LoadFailure>> = CopyOnWriteArrayList()
        /** Engine under test, built on the main thread as the app does. */
        val engine: RustEngine = onMain {
            RustEngine(context).also { built ->
                built.setVolume(0.0f)
                built.setOnLoadFailed { uri, reason -> failures += uri to reason }
            }
        }
        try {
            onMain { engine.load(tone, false) }
            assertTrue("the good tone reports its length", awaitTrue { onMain { engine.durationSec() } > 1.0 })
            Thread.sleep(QUIET_WAIT_MS)
            assertEquals("a good load reports no failure", emptyList<Pair<String, LoadFailure>>(), failures.toList())
            onMain { engine.load(notAudio, true) }
            assertTrue("the decode failure reaches onLoadFailed", awaitTrue { failures.isNotEmpty() })
            assertEquals(listOf(notAudio to LoadFailure.UNREADABLE), failures.toList())
            assertEquals("the deck keeps no length from the previous track", 0.0, onMain { engine.durationSec() }, 0.0)
        } finally {
            onMain { engine.release() }
        }
    }

    companion object {
        /** Size in bytes of the invented file that is not audio. */
        private const val NOT_AUDIO_BYTES: Int = 4096
    }
}
