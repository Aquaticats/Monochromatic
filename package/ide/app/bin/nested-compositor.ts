// The nested compositor every IDE check hosts the application in.
// What: the name to spawn (resolved through PATH, where `mise run toolchain:install` installs the
// `cargo:monochromatic-nested-wayland-session` tool), a check that the resolved build is current, and how a check
// ends the session it started.
// Why: a build inside each worktree's target directory goes stale. A 0.1.1 copy kept running in several worktrees
// after 0.1.2 made the window launch minimized, and every check run through it opened a visible window on the user's
// desktop. The synced tool is one copy that `mise run toolchain:install` keeps current. Ending a session is the other
// half of starting one: a compositor that ignores its quit request must not hang a check, and a compositor that is
// already going away closes its socket instead of replying, so every wait on it states a bound.
import { spawnSync } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';

/**
 Milliseconds a compositor gets to end after its quit request before a check stops it by force.
 */
export const SESSION_END_PATIENCE_MS = 10000;

/**
 Name of the executable every check spawns, resolved through PATH.
 */
export const compositor = 'monochromatic-nested-wayland-session';

/**
 The oldest version these checks have run against that launches the window minimized (0.4.0).
 */
export const oldestCompositor: readonly [number, number, number] = [0, 4, 0];

/**
 The first `major.minor.patch` run of ASCII digits in `text` as three numbers, or null when there is none.

 What: a cursor scan with no regular expression. A candidate starts at the first digit of a digit run, the run must be
 followed by `.`, a second run, `.`, and a third run; a failed candidate resumes at the end of its first run, because a
 start inside the same run sees the same tail and fails the same way.

 Why: the unanchored leftmost match of `(\d+)\.(\d+)\.(\d+)` is what a `--version` line needs, and the scan answers it in
 one linear pass. `\d` is ASCII only there, so the scan accepts only the characters 0 to 9 and not other Unicode digits.

 @param text - the output of `--version`
 @returns the three numbers, or null when the text holds no such run
 */
export function readVersion(text: string): [number, number, number] | null {
  const isDigit = (index: number): boolean => index < text.length && text.charCodeAt(index) >= 48 && text.charCodeAt(index) <= 57;
  const runEnd = (from: number): number => {
    let end = from;
    while (isDigit(end)) end += 1;
    return end;
  };
  let start = 0;
  while (start < text.length) {
    if (!isDigit(start)) {
      start += 1;
      continue;
    }
    const majorEnd = runEnd(start);
    const minorStart = majorEnd + 1;
    const minorEnd = runEnd(minorStart);
    const patchStart = minorEnd + 1;
    const patchEnd = runEnd(patchStart);
    if (text[majorEnd] === '.' && minorEnd > minorStart && text[minorEnd] === '.' && patchEnd > patchStart) {
      return [Number(text.slice(start, majorEnd)), Number(text.slice(minorStart, minorEnd)), Number(text.slice(patchStart, patchEnd))];
    }
    start = majorEnd;
  }
  return null;
}

/**
 Checks that a compositor of the current version is on PATH.

 @throws {Error} when no compositor is on PATH or the one found is older than `oldestCompositor`
 */
export function demandCurrentCompositor(): void {
  const result = spawnSync(compositor, ['--version'], { encoding: 'utf8' });
  if (result.error) {
    throw new Error('The nested compositor is not on PATH (' + result.error.message + '). Run `mise run toolchain:install`'
      + ' so the cargo:monochromatic-nested-wayland-session tool is installed, and start the check through `mise run`.');
  }
  const version = readVersion(result.stdout);
  if (version === null) throw new Error('Could not read a version from `' + compositor + ' --version` (status ' + result.status + '): '
      + result.stdout + result.stderr);
  const older = oldestCompositor.reduce<boolean | null>((verdict, part, index) => verdict ?? (version[index] === part ? null : (version[index] ?? 0) < part), null);
  if (older === true) {
    throw new Error('The nested compositor on PATH is ' + version.join('.') + ', older than ' + oldestCompositor.join('.')
      + '. Older builds open a visible window on the host desktop. Run `mise run toolchain:install` to provision the locked tool version.');
  }
}

/**
 What: nothing, after `ms` milliseconds.

 Why: a wait raced against a request or an exit needs a value of the same type to settle with.

 @param ms - milliseconds to wait
 @returns a promise no value resolves
 */
export async function silentFor(ms: number): Promise<undefined> {
  await delay(ms);
  return undefined;
}

/**
 What: whether the session ended within `ms` of this call.

 Why: a compositor that ignores its quit request must not hang a check, and every check states the same question with
      its own patience.

 @param exited - the session's own exit promise
 @param ms - how long the session gets to end
 @returns true when the session ended first
 */
export async function endedWithin(exited: Promise<unknown>, ms: number): Promise<boolean> {
  return await Promise.race([endedNow(exited), stillRunningAfter(ms)]);
}

/**
 What: true once `exited` settles.

 @param exited - the session's own exit promise
 @returns a promise resolving true
 */
async function endedNow(exited: Promise<unknown>): Promise<boolean> {
  await exited;
  return true;
}

/**
 What: false after `ms` milliseconds.

 @param ms - milliseconds to wait
 @returns a promise resolving false
 */
async function stillRunningAfter(ms: number): Promise<boolean> {
  await delay(ms);
  return false;
}

/**
 What: the reply to one control request, or `undefined` when the request fails.

 Why: a compositor that is going away closes its socket instead of replying, and that failure must not reject the
      race that ends a session.

 @param ask - sends one control line
 @returns the reply, or nothing when the request failed
 */
export async function quietReply(ask: () => Promise<string>): Promise<string | undefined> {
  try {
    return await ask();
  } catch (error) {
    console.error('the compositor did not reply: ' + (error instanceof Error ? error.message : String(error)));
    return undefined;
  }
}

/**
 What: end a session: ask it to quit, then stop it by signal when it does not end.

 Why: three checks end a compositor the same way, and one spelling keeps the order, the signals, and the patience in
      one place.

 @param ask - sends the quit request
 @param exited - the session's own exit promise
 @param kill - sends a signal to the session's own process
 @param patienceMs - how long the session gets to end after the quit request
 */
export async function endSession({ ask, exited, kill, patienceMs = SESSION_END_PATIENCE_MS }: {
  ask: () => Promise<string>;
  exited: Promise<unknown>;
  kill: (signal: NodeJS.Signals) => void;
  patienceMs?: number;
}): Promise<void> {
  try {
    await ask();
  } catch (error) {
    // The session cannot be asked to quit; a signal is the only way to end it.
    console.error('the session did not accept its quit request: ' + (error instanceof Error ? error.message : String(error)));
    kill('SIGTERM');
  }
  if (!(await endedWithin(exited, patienceMs))) kill('SIGKILL');
}
