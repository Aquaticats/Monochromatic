/**
 Keyboard end-to-end runner for the desktop music player.
 
 The runner hosts the release binary in a nested Wayland compositor, presses real keys through the compositor's
 control socket, and reads the app's state from its `ui_state` log lines. The command is `music-player-desktop-e2e`;
 this entry exposes the pure helpers that the unit tests cover.
 
 @example
 ```ts
 import { loadRunConfig, } from '@monochromatic-dev/music-player-desktop-e2e/ts';
 ```
 
 @packageDocumentation
 */

export { stripAnsi, } from './ansi.ts';
export {
  loadRunConfig,
  type RunConfig,
} from './config.ts';
export {
  chordCommands,
  keyCommand,
} from './control.ts';
export { runE2e, } from './run.ts';
