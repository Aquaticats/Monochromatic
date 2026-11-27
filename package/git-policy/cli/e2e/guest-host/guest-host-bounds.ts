/**
 The bounds every guest-driving script shares: how long one host command and one
 guest command may take, how often the host asks a detached guest script whether
 it has finished, and how large the swap file a guest's build gets.

 These are the values the three drivers carried separately, kept here so a change
 to one is a change to all of them. What each bounds is stated beside it; the
 guest's own `timeout` fires `guestTimeoutMarginSeconds` before `mvm` would kill
 the command, so a guest script that runs away leaves its own log rather than a
 kill nobody can read.

 @example
 ```ts
 const seconds = guestCommandTimeoutSeconds({ timeoutMilliseconds: 900_000 });
 ```
 */

import { millisecondsPerSecond, } from '../../bin/native-release-host-control-requirements.ts';

/**
 Milliseconds one host command may take: the `mvm` client calls and the host's
 `tar` around a guest run.
 */
export const hostCommandTimeoutMilliseconds = 120_000;

/**
 Seconds the guest's own `timeout` fires before `mvm`'s kill, so a script that
 runs away is stopped inside the guest, where its log is.
 */
export const guestTimeoutMarginSeconds = 60;

/**
 How often the host asks a guest whether a detached script has finished, in
 milliseconds: a build takes minutes, so a closer poll only costs the guest
 agent's own answers.
 */
export const pollIntervalMilliseconds = 15_000;

/**
 The swap file a guest adds, in gibibytes, so one `rustc` at five gigabytes does
 not exhaust the guest's memory.
 */
export const swapGibibytes = '6G';

/**
 The mode a host-side log file is made with: readable and writable by the account
 that runs the driver, and by nobody else, because a guest log holds the paths
 and the answers of a run.
 */
export const hostLogFileMode = 0o600;

/**
 How many host-command bounds one guest-recipe step gets: the recipe builds
 OpenZFS, the Rust toolchain and the crate inside the guest, so its own step is
 many times one `mvm` call.
 */
const recipeBoundsPerHostCommand = 60;

/**
 Milliseconds one guest-recipe step may take.

 @returns the bound, which is a host command's own bound sixty times over
 */
export const recipeTimeoutMilliseconds: number = hostCommandTimeoutMilliseconds
  * recipeBoundsPerHostCommand;

/**
 Seconds a guest command is given, after the margin the guest's own `timeout`
 leaves before `mvm`'s kill.

 @param timeoutMilliseconds - milliseconds the host waits for the command

 @returns the whole seconds the guest's `timeout` is given

 @example
 ```ts
 guestCommandTimeoutSeconds({ timeoutMilliseconds: 900_000 });
 ```
 */
export function guestCommandTimeoutSeconds({
  timeoutMilliseconds,
}: {
  readonly timeoutMilliseconds: number;
},): number {
  return Math.floor(timeoutMilliseconds / millisecondsPerSecond)
    - guestTimeoutMarginSeconds;
}
