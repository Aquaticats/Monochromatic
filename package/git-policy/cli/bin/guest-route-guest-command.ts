/**
 The command one driver's script runs inside the guest.

 `mvm exec NAME -- <command>` runs the command through the guest's own shell, so a script that overran would wait for
 `mvm`'s kill and lose the transport with the log. Every driver therefore wraps its script in the guest's `timeout`,
 which fires `GUEST_TIMEOUT_MARGIN_SECONDS` before the bound `mvm` is given, so an overrun is a line in the guest's log
 instead of a lost run. The same command refuses a machine whose hostname is not the guest's, so a run that reached
 another machine performs nothing.
 */
import { shellQuote, } from './guest-route-shell-quote.ts';

/**
 Milliseconds in one second, so a bound a driver states in milliseconds is the seconds the guest's `timeout` reads.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Seconds the guest's own `timeout` fires before `mvm`'s kill, which writes the overrun into the guest's log.
 */
export const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

/**
 Write the command `mvm exec` runs for one guest script.

 @param name - the guest the command must find itself in

 @param script - the shell script to run inside the guest

 @param timeoutMilliseconds - how long the guest may take, which the guest's own `timeout` is sized from

 @returns the command, its script quoted whole for the guest's `sh -ec`

 @example
 ```ts
 const command = guestCommandLine({
   name: 'forced-act',
   script: 'set -eu\nzpool list -H -o name',
   timeoutMilliseconds: 900_000,
 });
 ```
 */
export function guestCommandLine({
  name,
  script,
  timeoutMilliseconds,
}: {
  readonly name: string;
  readonly script: string;
  readonly timeoutMilliseconds: number;
},): string {
  /**
   Seconds the guest's `timeout` allows, which fires before `mvm`'s own kill.
   */
  const guestSeconds = Math.floor(timeoutMilliseconds / MILLISECONDS_PER_SECOND) - GUEST_TIMEOUT_MARGIN_SECONDS;
  return `test "$(hostname)" = ${name} && timeout ${guestSeconds} sh -ec ${shellQuote(script,)}`;
}
