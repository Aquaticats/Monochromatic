import { access, } from 'node:fs/promises';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';


//region Git command
// WHICH GIT this package spawns, decided once and shared.
//
// Two modules resolved it differently. The census preferred the real binary and
// fell back to whatever the PATH resolves, while the run configuration named the
// real binary outright, so a machine without it could answer ancestry and still
// fail at startup reading HEAD. That is a portability difference between two
// answers to one question, which is the kind of split that only shows up on
// someone else's machine.

/**
 Logger for the probe's one line; the command takes no caller-supplied one.
 */
const gitLog = contextRoot({ tag: 'git-command', },);

/**
 Real git binary, preferred over the PATH entry.

 `git` on this repository's PATH resolves to `node_modules/.bin/git`, a shim
 carrying staging guards. Those guards are irrelevant to read-only calls, but
 resolving through a shim makes ancestry depend on a wrapper that exists for
 an unrelated reason, so the real binary is asked for by name.
 */
const SYSTEM_GIT = '/usr/bin/git';

/**
 One in-flight or settled probe for the git command, keyed by the path probed.

 A Map rather than a module-root `let`, which the lint rule forbids and which
 this does not need: the entry is written once. Holding a PROMISE rather than a
 value keeps resolution on first use rather than on import, so loading this
 module never touches the filesystem, and concurrent callers share one probe
 instead of racing several.
 */
const gitProbe = new Map<string, Promise<string>>();

/**
 Finds a git to spawn, preferring the real binary over the PATH entry.

 @returns Command name or absolute path

 @example
 ```ts
 const git = await detectGit();
 ```
 */
async function detectGit(): Promise<string> {
  try {
    await access(SYSTEM_GIT,);
    return SYSTEM_GIT;
  }
  catch (error) {
    // Absent is ordinary anywhere that is not this machine. Logged rather than
    // swallowed, because falling back to PATH means git is whatever the shell
    // resolves, including a shim, and that is worth seeing in a report which
    // turns on what git answered.
    gitLog.warn(
      `${SYSTEM_GIT} not present (${String(error,)}); using git from PATH`,
    );
    return 'git';
  }
}

/**
 Git command to spawn, resolved once per process.

 Not itself async: it hands back the memoised promise, so concurrent callers
 share one probe rather than racing several.

 @returns Promise of the command to spawn

 @example
 ```ts
 const git = await resolveGit();
 ```
 */
export function resolveGit(): Promise<string> {
  /**
   Probe already started for this path, when one has been.
   */
  const started = gitProbe.get(SYSTEM_GIT,);
  if (started !== undefined)
    return started;

  /**
   Probe this call starts, stored before it settles so a second caller joins
   it rather than spawning its own.
   */
  const probe = detectGit();
  gitProbe.set(
    SYSTEM_GIT,
    probe,
  );
  return probe;
}

//endregion Git command
