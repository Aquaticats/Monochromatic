package dev.monochromatic.musicplayer

/**
 * Test double of [AudioEngine] that, unlike [FakeAudioEngine], reports what the real engine reports: a change of the
 * playing state reaches the controller's callback, and a load of a URI listed in [unreadable] ends in a load failure
 * with playback stopped. It lets a controller test see the playing flag the deck and the rows are drawn from.
 */
class ReportingAudioEngine : AudioEngine {
    /** URIs whose load fails, as a file the provider refuses to open does. */
    val unreadable: MutableSet<String> = mutableSetOf()

    /** URI of the last load, whether it succeeded or not. */
    var loadedUri: String? = null

    /** Every load call in order, as pairs of the URI and the play flag it asked for. */
    val loads: MutableList<Pair<String, Boolean>> = mutableListOf()

    /** Number of times the controller asked the engine to pause. */
    var pauseCount: Int = 0

    /** Whether the engine is playing now, which the controller learns only through its callback. */
    var playing: Boolean = false
        private set

    /** Handler told when [playing] changes, or null before the controller registers one. */
    private var onPlayingChanged: ((Boolean) -> Unit)? = null

    /** Handler told when a load fails, or null before the controller registers one. */
    private var onLoadFailed: ((String, LoadFailure) -> Unit)? = null

    /** Sets [playing] and tells the controller when the value changed. */
    private fun setPlaying(value: Boolean) {
        if (playing == value) {
            return
        }
        playing = value
        onPlayingChanged?.invoke(value)
    }

    /** Records the load; a listed URI fails and stops playback, any other plays when asked to. */
    override fun load(uri: String, play: Boolean) {
        loadedUri = uri
        loads += uri to play
        if (uri in unreadable) {
            setPlaying(false)
            onLoadFailed?.invoke(uri, LoadFailure.UNREADABLE)
            return
        }
        setPlaying(play)
    }

    /** Starts playback of what is loaded. */
    override fun play() {
        setPlaying(true)
    }

    /** Stops playback and counts the call. */
    override fun pause() {
        pauseCount += 1
        setPlaying(false)
    }

    /** Ignores seeks, which no row test reads. */
    override fun seekTo(positionSec: Double) {
    }

    /** Ignores the volume, which no row test reads. */
    override fun setVolume(volume: Float) {
    }

    /** Reports the start of the track. */
    override fun positionSec(): Double = 0.0

    /** Reports an unknown length. */
    override fun durationSec(): Double = 0.0

    /** Reports the playing state. */
    override fun playWhenReady(): Boolean = playing

    /** Keeps the handler, which [setPlaying] calls. */
    override fun setOnPlayingChanged(callback: (Boolean) -> Unit) {
        onPlayingChanged = callback
    }

    /** Ignores the handler, because no case ends a track. */
    override fun setOnTrackEnded(callback: () -> Unit) {
    }

    /** Keeps the handler, which [load] calls for an unreadable URI. */
    override fun setOnLoadFailed(callback: (uri: String, reason: LoadFailure) -> Unit) {
        onLoadFailed = callback
    }

    /** Has nothing to free. */
    override fun release() {
    }
}
