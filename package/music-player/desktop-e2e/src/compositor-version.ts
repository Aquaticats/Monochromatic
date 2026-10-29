/**
 What the checks need from the nested compositor, and how to tell from its `--version` line whether it has that.
 
 The key names F1 to F24 and Menu, the chords, the timed hold with the client's key repeat, Unicode typing, exact sizes
 and focus return after a dialog first appeared in compositor 0.4.0. An older compositor answers `unknown key` for them,
 so the run refuses to start with a message that names the fix instead of failing a dozen checks one by one. Any newer
 version is accepted: nothing here compares a version for equality.
 
 @example
 ```ts
 import { compositorVersionProblem, } from './compositor-version.ts';
 
 compositorVersionProblem({ output: 'monochromatic-nested-wayland-session 0.3.0', path: '/c', },); // a message
 ```
 
 @packageDocumentation
 */

/**
 The first compositor version that has every control the runner uses: `key` with F-keys and Menu, `type` of Unicode,
 exact `resize`, `windows` and `focus`. A compositor of this version or newer is accepted.
 */
export const FIRST_VERSION_WITH_RUNNER_CONTROLS = '0.4.0';

/**
 How many numbers a version has: major, minor and patch.
 */
const VERSION_PARTS = 3;

/**
 Tells whether a text is one or more decimal digits and nothing else.
 
 @param text - one dot-separated part of a version
 
 @returns whether every character of `text` is a digit and `text` is not empty
 
 @example
 ```ts
 isDigits('12',); // true
 ```
 */
function isDigits(text: string,): boolean {
  if (text === '')
    return false;
  for (const character of text.split('',))
    if ((character < '0') || (character > '9'))
      return false;
  return true;
}

/**
 Reads the version number from the text `--version` prints, which ends with the number. A pre-release or build suffix
 (`0.5.0-dev`, `0.5.0+local`) is ignored, so a local build of a newer compositor is not refused for its suffix.
 
 @param output - the compositor's `--version` output, such as `monochromatic-nested-wayland-session 0.4.0`
 
 @returns the `[major, minor, patch]` numbers, or an empty list when the last word is not a version
 
 @example
 ```ts
 parseCompositorVersion('monochromatic-nested-wayland-session 0.4.0',); // [0, 4, 0]
 ```
 */
export function parseCompositorVersion(output: string,): readonly number[] {
  /**
   The words of the output.
   */
  const words = output.trim()
    .split(' ',);

  /**
   The last word of the output.
   */
  const word = words.at(-1,) ?? '';

  /**
   The word without a pre-release or build suffix.
   */
  const core = word.split('-',)[0]
    ?.split('+',)[0]
    ?? '';

  /**
   The dot-separated parts of that word.
   */
  const parts = core.split('.',);
  if (parts.length !== VERSION_PARTS)
    return [];
  if (!parts.every(isDigits,))
    return [];
  return parts.map(Number,);
}

/**
 Tells whether a version is the minimum or newer.
 
 @param version - the version found, as `[major, minor, patch]`
 
 @param minimum - the oldest acceptable version
 
 @returns whether `version` is at least `minimum`
 
 @example
 ```ts
 isAtLeast({ version: [0, 4, 1,], minimum: [0, 4, 0,], },); // true
 ```
 */
export function isAtLeast(
  {
    version,
    minimum,
  }: {
    readonly minimum: readonly number[];
    readonly version: readonly number[];
  },
): boolean {
  for (const [index, wanted,] of minimum.entries()) {
    /**
     The part of the version at the same position.
     */
    const found = version[index] ?? 0;
    if (found !== wanted)
      return found > wanted;
  }
  return true;
}

/**
 Explains why a compositor cannot run the checks.
 
 @param output - the compositor's `--version` output
 
 @param path - the executable that was asked
 
 @returns a message that names the problem and the ways out, or an empty string when the compositor has the controls
 
 @example
 ```ts
 compositorVersionProblem({ output: 'x 0.4.0', path: '/c', },); // ''
 ```
 */
export function compositorVersionProblem(
  {
    output,
    path,
  }: {
    readonly output: string;
    readonly path: string;
  },
): string {
  /**
   The version the compositor reported.
   */
  const version = parseCompositorVersion(output,);

  /**
   What the checks need, in words, and the ways to get it.
   */
  const need = `the checks need compositor ${FIRST_VERSION_WITH_RUNNER_CONTROLS} or newer (key names F1 to F24 and `
    + 'Menu, Unicode typing, exact sizes, windows and focus). Install the current one with '
    + '"mise upgrade cargo:monochromatic-nested-wayland-session", or set E2E_COMPOSITOR to a build of '
    + 'package/nested-wayland-session/cli';
  if (version.length === 0)
    return `the compositor ${path} printed no version ("${output.trim()}"); ${need}`;
  if (isAtLeast({
    version,
    minimum: parseCompositorVersion(FIRST_VERSION_WITH_RUNNER_CONTROLS,),
  },))
    return '';
  return `the compositor ${path} is ${version.join('.',)}; ${need}`;
}
