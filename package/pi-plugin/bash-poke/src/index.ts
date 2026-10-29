/**
 Pi extension that runs `!` commands in the background and pokes the model when they finish.

 Native `!` blocks the editor until a command exits and refuses a second command
 while one runs. This extension takes execution over through Pi's `user_bash`
 event, hands the editor back immediately with a placeholder recorded in context,
 and sends the model a poke carrying the real output when the process exits. The
 motivating case is `! sleep 7200` as a delayed continue after a provider usage
 limit resets, which needs the poke and does not need the extension to know
 anything about limits.

 @module
 */

import { homedir, } from 'node:os';

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { loadSettings, } from './config.ts';
import { bashPokeLogger, } from './logger.ts';
import { registerBashPoke, } from './register.ts';
import { resolveShell, } from './shell-resolution.ts';

//region Extension entry

/**
 Loads settings, resolves the shell, and registers bash-poke.
 
 Both happen before registration because every handler closes over them, and Pi
 awaits an async factory before emitting `session_start`. A settings file that
 exists but cannot be honored fails the load loudly rather than degrading
 silently, since a knob that does nothing is worse than an error.
 
 @param pi - Pi extension API receiving the registrations
 
 @throws BashPokeConfigError when the settings file is unreadable, invalid, or fails validation
 
 @example
 ```ts
 pi --no-extensions -e package/pi-plugin/bash-poke/src/index.ts
 ```
 */
export default async function bashPoke(
  pi: ForeignBorrowed<ExtensionAPI>,
): Promise<void> {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: bashPoke.name,
    l: bashPokeLogger,
  }, );

  /**
   Settings for this extension instance, resolved from the global config file.
   */
  const settings = await loadSettings({ home: homedir(), }, );

  /**
   Shell every job in this instance runs through.
   */
  const shell = await resolveShell({ pathValue: process.env
    .PATH
    ?? '', }, );
  l.debug('registering bash-poke extension', );
  registerBashPoke({
    pi,
    settings,
    shell,
  }, );
}

//endregion Extension entry

//region Public API
// Exported so unit tests can drive every code path through the built artifact
// rather than through sibling source imports.

export * as constants from './constants.ts';

export { BashPokeConfigError, } from './config-error.ts';
export { configPathForHome, } from './config-paths.ts';
export {
  DEFAULT_SETTINGS,
  parseSettings,
  readBooleanSetting,
  readNumberSetting,
  readStringSetting,
  SETTING_KEYS,
  SETTING_KEY_SET,
} from './config-schema.ts';
export type {
  SettingKey,
  SettingsRecord,
} from './config-schema.ts';
export type { BashPokeSettings, } from './config-types.ts';
export {
  loadSettings,
  parseSettingsJson,
  readSettingsFile,
} from './config.ts';
export type {
  SettingsFileRead,
  SettingsJsonParse,
} from './config.ts';
export {
  createEscapeListener,
  isLoneEscape,
  shouldCancelOnEscape,
} from './escape-intercept.ts';
export type { EscapeGate, } from './escape-intercept.ts';
export { createJobRegistry, } from './job-registry.ts';
export type { JobRegistry, } from './job-registry.ts';
export {
  readClosePayload,
  startJob,
} from './job-runner.ts';
export type {
  CloseReading,
  JobOutcome,
  KillTimers,
  RunningJob,
  StartJobInput,
} from './job-runner.ts';
export { bashPokeLogger, } from './logger.ts';
export {
  cardLabel,
  contentText,
  pokeCardText,
  registerPokeRenderer,
} from './message-renderer.ts';
export type {
  ContentPart,
  PokeMessage,
} from './message-renderer.ts';
export {
  elisionMarker,
  middleOutFromParts,
} from './middle-out.ts';
export {
  createOutputRecorder,
  trimRollingBuffer,
} from './output-recorder.ts';
export type {
  OutputRecorder,
  OutputSnapshot,
  RecorderState,
} from './output-recorder.ts';
export {
  FORMAT_CHARACTER_RANGES,
  isControlCode,
  isFormatCharacter,
  isKeptCode,
  isKeptWhitespace,
  isSurrogateCode,
  sanitizeOutput,
} from './output-sanitize.ts';
export type { FormatCharacterRange, } from './output-sanitize.ts';
export { deliverPoke, } from './poke-delivery.ts';
export type { PokeDelivery, } from './poke-delivery.ts';
export {
  buildPokeContent,
  fenceFor,
  jobOutputText,
  jobStatusText,
  longestBacktickRun,
  spoolNote,
} from './poke-text.ts';
export type { PokeDetails, } from './poke-text.ts';
export {
  createProgressView,
  renderProgressLines,
} from './progress-widget.ts';
export type {
  ProgressSurface,
  ProgressView,
  ThrottleState,
} from './progress-widget.ts';
export {
  pendingBashResult,
  registerBashPoke,
} from './register.ts';
export type {
  PendingBashResult,
  Registration,
} from './register.ts';
export { createSessionBinding, } from './session-binding.ts';
export type { SessionBinding, } from './session-binding.ts';
export {
  executableExists,
  executablesOnPath,
  resolveShell,
} from './shell-resolution.ts';
export type {
  CandidateProbe,
  ExistsProbe,
  ShellInvocation,
} from './shell-resolution.ts';
export {
  discardingSpoolWriter,
  openSpoolWriter,
  spoolDirectory,
} from './spool.ts';
export type { SpoolWriter, } from './spool.ts';
export {
  isErrorWithCode,
  isMissingFileCode,
} from './system-error.ts';
export type { ErrorWithCode, } from './system-error.ts';

//endregion Public API
