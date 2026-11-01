// The volume rule for a short interruption that allows ducking, such as a notification sound (D134).

// What:     `package ...core` places this pure rule beside the other platform-independent rules.
// Why:      The rule takes a float and a boolean, so host JVM tests exercise it without Android framework classes.
//
// In TS you'd write (pseudocode):
// ```ts
// // Module identity comes from the file path.
// ```
package dev.monochromatic.musicplayer.core

// What:     `const val DUCKED_VOLUME_FACTOR: Float = 0.2f` is the share of the volume that stays while ducked.
// Why:      It is the level Android's own ducking ramps to (-14 dB), so a notification sound sounds the same here as
//           it does over other apps. The framework cannot duck this app's AAudio output itself, so the engine does it.
//
// In TS you'd write (pseudocode):
// ```ts
// const DUCKED_VOLUME_FACTOR = 0.2;
// ```
/** Share of the user volume that remains while another sound ducks the player. */
const val DUCKED_VOLUME_FACTOR: Float = 0.2f

// What:     `fun volumeWhileDucked(userVolume: Float, ducked: Boolean): Float` gives the volume the engine plays at.
// Why:      A can-duck focus loss lowers the playing volume instead of pausing, and the regained focus puts the user
//           volume back, so the engine keeps the user volume and the duck flag apart and combines them here.
//
// In TS you'd write (pseudocode):
// ```ts
// const volumeWhileDucked = (userVolume: number, ducked: boolean): number =>
//   ducked ? userVolume * DUCKED_VOLUME_FACTOR : userVolume;
// ```
/** Returns the volume to play at: the [userVolume] lowered by [DUCKED_VOLUME_FACTOR] while [ducked]. */
fun volumeWhileDucked(userVolume: Float, ducked: Boolean): Float =
    if (ducked) userVolume * DUCKED_VOLUME_FACTOR else userVolume
