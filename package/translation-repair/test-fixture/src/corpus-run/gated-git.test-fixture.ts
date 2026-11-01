//region Gated git
// A STAND-IN GIT BINARY WHOSE CHILDREN END IN AN ORDER A CASE CHOOSES.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. A corpus read names a git binary through
// the pin (`CorpusPin.gitPath`), so a case that wants two reads to fail in a
// set order points the pin here. Every read of the stand-in fails; the one
// whose path ends with `releasedBy` fails at once and leaves a marker as it
// exits, and the one whose path ends with `held` waits for that marker and a
// further pause before it fails, so the held read ends after the other
// whichever way the machine is loaded. A read of any other path fails at once.

import {
  chmod,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Pause, in milliseconds, the held read keeps after it sees the marker, so the
 other read's exit has reached the parent before the held read ends.
 */
const HELD_PAUSE_MS = 150;

/**
 Permission bits that make the stand-in runnable.
 */
const EXECUTABLE_MODE = 0o755;

/**
 How often, in milliseconds, the held read looks for the marker.
 */
const MARKER_POLL_MS = 10;

/**
 Writes the stand-in git binary into a directory.

 @param dir - scratch directory the script and its marker are written into

 @param held - path suffix of the read that ends last

 @param releasedBy - path suffix of the read that ends first

 @param stderr - text every refused read prints, which decides the failure kind
 the corpus reader gives it

 @returns Absolute path to hand a pin as `gitPath`

 @example
 ```ts
 const gitPath = await writeGatedGit({ dir, held: 'page.md', releasedBy: 'page.en.md', stderr: 'fatal: no', },);
 ```
 */
export async function writeGatedGit(
  {
    dir,
    held,
    releasedBy,
    stderr,
  }: {
    readonly dir: string;
    readonly held: string;
    readonly releasedBy: string;
    readonly stderr: string;
  },
): Promise<string> {
  /**
   Where the stand-in is written.
   */
  const scriptPath = join(
    dir,
    'gated-git.mjs',
  );

  /**
   What the first-ending read leaves for the held read.
   */
  const markerPath = join(
    dir,
    'released.marker',
  );

  /**
   Settings the script reads, as data so no value is pasted into code.
   */
  const settings = JSON.stringify({
    held,
    releasedBy,
    stderr,
    markerPath,
    pauseMs: HELD_PAUSE_MS,
    pollMs: MARKER_POLL_MS,
  },);
  await writeFile(
    scriptPath,
    [
      `#!${process.execPath}`,
      'import { existsSync, writeFileSync, } from \'node:fs\';',
      `const settings = ${settings};`,
      'const spec = process.argv.at(-1);',
      'function refuse() {',
      '  process.stderr.write(settings.stderr + String.fromCharCode(10));',
      '  process.exit(128);',
      '}',
      'if (spec.endsWith(settings.releasedBy)) {',
      '  process.on(\'exit\', function leaveMarker() { writeFileSync(settings.markerPath, \'\'); });',
      '  refuse();',
      '}',
      'else if (spec.endsWith(settings.held)) {',
      '  const poll = setInterval(function look() {',
      '    if (existsSync(settings.markerPath)) {',
      '      clearInterval(poll);',
      '      setTimeout(refuse, settings.pauseMs);',
      '    }',
      '  }, settings.pollMs);',
      '}',
      'else refuse();',
      '',
    ].join('\n',),
    'utf8',
  );
  await chmod(
    scriptPath,
    EXECUTABLE_MODE,
  );
  return scriptPath;
}

//endregion Gated git
