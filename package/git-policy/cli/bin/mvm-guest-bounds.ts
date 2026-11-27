/**
 The bounds and the file modes every guest driver shares: how long one host command may take, how often the host asks a
 guest whether a detached script has finished, how long the guest's own `timeout` fires before `mvm`'s kill, how large
 the swap file a guest's build gets, how long one step of the guest recipe may take, and the permissions a run's folders
 and logs are made with.

 Each value was carried by three sets of guest helpers before one set answered it
 (`bin/guest-route-permissions.ts`, `bin/guest-route-commands.ts` and `e2e/guest-host/guest-host-bounds.ts`), and a
 change to one is a change to every driver now. What each bounds is stated beside it.
 */

import { millisecondsPerSecond, } from './native-release-host-control-requirements.ts';

/**
 Milliseconds one host command may take: the `mvm` client calls and the host's `tar` around a guest run.
 */
export const HOST_COMMAND_TIMEOUT_MS = 120_000;

/**
 Seconds the guest's own `timeout` fires before `mvm`'s kill, so a script that runs away is stopped inside the guest,
 where its log is, instead of losing the transport with the command.
 */
export const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

/**
 How often the host asks a guest whether a detached script has finished, in milliseconds: a build takes minutes, so a
 closer poll only costs the guest agent's own answers.
 */
export const POLL_INTERVAL_MS = 15_000;

/**
 The swap file a guest adds, in gibibytes, so one `rustc` of this crate's tests at 6.7 to 6.9 GiB does not exhaust the
 guest's 8 GiB (`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`, "The lock driver's build at one job, and its
 arm").
 */
export const SWAP_GIBIBYTES = '6G';

/**
 How many host-command bounds one guest-recipe step gets: the recipe builds the guest's packages, OpenZFS and the Rust
 toolchain inside the step it is asked for, so its own step is many times one `mvm` call.
 */
const RECIPE_BOUNDS_PER_HOST_COMMAND = 60;

/**
 Milliseconds one guest-recipe step may take.
 */
export const RECIPE_STEP_TIMEOUT_MS: number = HOST_COMMAND_TIMEOUT_MS * RECIPE_BOUNDS_PER_HOST_COMMAND;

/**
 Milliseconds in one second, so a bound stated in milliseconds becomes the seconds a shell's `timeout` takes.
 */
export const MILLISECONDS_PER_SECOND: number = millisecondsPerSecond;

/**
 The permissions a folder a driver makes has: the account that runs the driver, and nobody else.
 */
export const FOLDER_MODE = 0o700;

/**
 The permissions a log a driver writes has: the account that runs the driver, and nobody else. A log of a guest run
 holds the guest's own output, the pool it built and the names of the fixtures it destroyed, and this machine's `/home`
 is shared, so nothing another account may read is written into one.
 */
export const LOG_FILE_MODE = 0o600;
