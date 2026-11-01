// The nested compositor every IDE check hosts the application in.
// What: the name to spawn (resolved through PATH, where `mise run toolchain:install` installs the
// `cargo:monochromatic-nested-wayland-session` tool) and a check that the resolved build is current.
// Why: a build inside each worktree's target directory goes stale. A 0.1.1 copy kept running in several worktrees
// after 0.1.2 made the window launch minimized, and every check run through it opened a visible window on the user's
// desktop. The synced tool is one copy that `mise run toolchain:install` keeps current.
import { spawnSync } from 'node:child_process';

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
