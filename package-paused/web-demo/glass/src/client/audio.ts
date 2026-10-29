/**
 * Procedural WebAudio for the glass corridor: crack snaps, shatter
 * crashes, shard ticks, and a soft throw whoosh. No samples; everything
 * synthesizes from noise buffers and decaying sine partials, so the demo
 * stays a single self-contained file.
 *
 * The context unlocks lazily on the first pointer gesture, as autoplay
 * policy requires.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the audio module.
 */
const l = tagged({
  tag: 'audio',
  l: parentLogger,
},);

/**
 * Master volume and rate limits.
 */
export const AUDIO_TUNING = {
  /**
   * Master gain applied to every voice.
   */
  masterGain: 0.5,
  /**
   * Shortest interval between shard ticks, seconds.
   */
  tickInterval: 0.06,
  /**
   * Exponential envelope floor gains ramp down to.
   */
  envelopeFloor: 1e-4,
  /**
   * Longest random offset into the shared noise buffer, seconds.
   */
  noiseSeekSpread: 0.4,
  /**
   * Crack partial frequency base and random spread, Hz.
   */
  crackPartial: {
    base: 2_600,
    spread: 900,
  },
  /**
   * Shatter ring-out voices and their frequency and duration ranges.
   */
  shatterVoices: {
    count: 5,
    baseHz: 2_800,
    spreadHz: 4_600,
    baseSeconds: 0.22,
    spreadSeconds: 0.28,
  },
  /**
   * Shard tick frequency base and random spread, Hz.
   */
  tick: {
    baseHz: 6_200,
    spreadHz: 2_000,
  },
} as const;

/**
 * Audio system handle. Every play call is safe before unlock: voices
 * simply do not sound until the context exists.
 */
export type AudioSystem = {
  /**
   * Creates or resumes the context; call from a user gesture.
   */
  readonly unlock: () => void;
  /**
   * Soft throw whoosh.
   */
  readonly playThrow: () => void;
  /**
   * Sharp snap for the crack stage.
   */
  readonly playCrack: () => void;
  /**
   * Full crash for the collapse.
   */
  readonly playShatter: () => void;
  /**
   * Tiny tick for shards bouncing on the floor, rate limited.
   */
  readonly playTick: () => void;
};

/**
 * Creates the audio system.
 *
 * @returns audio system handle
 *
 * @example
 * ```ts
 * const audio = createAudio();
 * canvas.addEventListener('pointerdown', () => { audio.unlock(); },);
 * ```
 */
export function createAudio(): AudioSystem {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: createAudio.name,
    l,
  },);
  /**
   * Mutable audio state: context and noise buffer exist only after the
   * first unlock gesture, and the tick limiter timestamp advances as
   * ticks play.
   */
  const state: {
    /**
     * Lazily created context; absent until the first gesture.
     */
    context?: AudioContext;
    /**
     * Shared one-second white-noise buffer, built once per context.
     */
    noise?: AudioBuffer;
    /**
     * Context time before which shard ticks are dropped.
     */
    nextTickAt: number;
  } = { nextTickAt: 0, };
  /**
   * Plays a filtered noise burst.
   *
   * @param duration - burst length in seconds
   *
   * @param frequency - bandpass center in Hz
   *
   * @param quality - bandpass Q; higher rings longer
   *
   * @param gain - peak gain before the master
   */
  function noiseBurst(
    {
      duration,
      frequency,
      quality,
      gain,
    }: Readonly<{
      duration: number;
      frequency: number;
      quality: number;
      gain: number;
    }>,
  ): void {
    /**
     * Locals for the guard so narrowing survives the closures below.
     */
    const {
      context,
      noise,
    } = state;
    if ((context === undefined) || (noise === undefined))
      return;
    /**
     * Noise source for this voice.
     */
    const source = context.createBufferSource();
    source.buffer = noise;
    /**
     * Bandpass shaping the noise into glassy hiss.
     */
    const filter = context.createBiquadFilter();
    filter.type = 'bandpass';
    filter.frequency
      .value = frequency;
    filter.Q
      .value = quality;
    /**
     * Exponential decay envelope.
     */
    const envelope = context.createGain();
    envelope.gain
      .setValueAtTime(
        gain * AUDIO_TUNING.masterGain,
        context.currentTime,
      );
    envelope.gain
      .exponentialRampToValueAtTime(
        AUDIO_TUNING.envelopeFloor,
        context.currentTime + duration,
      );
    source.connect(filter,)
      .connect(envelope,)
      .connect(context.destination,);
    source.start(
      context.currentTime,
      Math.random() * AUDIO_TUNING.noiseSeekSpread,
    );
    source.stop(context.currentTime + duration,);
  }
  /**
   * Plays one decaying sine partial.
   *
   * @param frequency - partial frequency in Hz
   *
   * @param duration - decay length in seconds
   *
   * @param gain - peak gain before the master
   */
  function partial(
    {
      frequency,
      duration,
      gain,
    }: Readonly<{
      frequency: number;
      duration: number;
      gain: number;
    }>,
  ): void {
    /**
     * Local for the guard so narrowing holds through the voice setup.
     */
    const { context, } = state;
    if (context === undefined)
      return;
    /**
     * Sine oscillator for this partial.
     */
    const oscillator = context.createOscillator();
    oscillator.type = 'sine';
    oscillator.frequency
      .value = frequency;
    /**
     * Exponential decay envelope.
     */
    const envelope = context.createGain();
    envelope.gain
      .setValueAtTime(
        gain * AUDIO_TUNING.masterGain,
        context.currentTime,
      );
    envelope.gain
      .exponentialRampToValueAtTime(
        AUDIO_TUNING.envelopeFloor,
        context.currentTime + duration,
      );
    oscillator.connect(envelope,)
      .connect(context.destination,);
    oscillator.start();
    oscillator.stop(context.currentTime + duration,);
  }
  return {
    unlock: function unlock(): void {
      if (state.context !== undefined) {
        if (state.context
          .state
          === 'suspended')
          void state.context
            .resume();
        return;
      }
      /**
       * Fresh context created by this first gesture.
       */
      const context = new AudioContext();
      /**
       * Noise buffer shared by every burst voice.
       */
      const noise = context.createBuffer(
        1,
        context.sampleRate,
        context.sampleRate,
      );
      /**
       * Channel filled with uniform white noise.
       */
      const channel = noise.getChannelData(0,);
      for (let sample = 0; sample < channel.length; sample++)
        channel[sample] = (Math.random() * 2) - 1;
      state.context = context;
      state.noise = noise;
      innerL.info('audio context unlocked',);
    },
    playThrow: function playThrow(): void {
      noiseBurst({
        duration: 0.14,
        frequency: 900,
        quality: 1.2,
        gain: 0.12,
      },);
    },
    playCrack: function playCrack(): void {
      noiseBurst({
        duration: 0.09,
        frequency: 3_800,
        quality: 2.5,
        gain: 0.55,
      },);
      partial({
        frequency: AUDIO_TUNING.crackPartial
          .base
          + (Math.random()
            * AUDIO_TUNING.crackPartial
            .spread),
        duration: 0.1,
        gain: 0.18,
      },);
    },
    playShatter: function playShatter(): void {
      noiseBurst({
        duration: 0.55,
        frequency: 5_200,
        quality: 0.9,
        gain: 0.7,
      },);
      noiseBurst({
        duration: 0.35,
        frequency: 2_300,
        quality: 1.4,
        gain: 0.45,
      },);
      partial({
        frequency: 130,
        duration: 0.18,
        gain: 0.3,
      },);
      for (let voice = 0; voice
        < AUDIO_TUNING.shatterVoices
        .count; voice++)
        partial({
          frequency: AUDIO_TUNING.shatterVoices
            .baseHz
            + (Math.random()
              * AUDIO_TUNING.shatterVoices
              .spreadHz),
          duration: AUDIO_TUNING.shatterVoices
            .baseSeconds
            + (Math.random()
              * AUDIO_TUNING.shatterVoices
              .spreadSeconds),
          gain: 0.08,
        },);
    },
    playTick: function playTick(): void {
      /**
       * Local for the guard and the limiter comparison.
       */
      const { context, } = state;
      if (context === undefined)
        return;
      if (context.currentTime < state.nextTickAt)
        return;
      state.nextTickAt = context.currentTime + AUDIO_TUNING.tickInterval;
      noiseBurst({
        duration: 0.05,
        frequency: AUDIO_TUNING.tick
          .baseHz
          + (Math.random()
            * AUDIO_TUNING.tick
            .spreadHz),
        quality: 4,
        gain: 0.12,
      },);
    },
  };
}
