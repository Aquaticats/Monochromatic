/**
 Shell resolution owned by this package.

 @module
 */

import {
  access,
  constants,
} from 'node:fs/promises';
import {
  delimiter,
  isAbsolute,
  join,
} from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  ABSOLUTE_BASH_PATH,
  BASH_COMMAND_NAME,
  SH_COMMAND_NAME,
  SHELL_COMMAND_FLAG,
} from './constants.ts';
import { bashPokeLogger, } from './logger.ts';

//region Types

/**
 Shell command and the flag that carries a command string.
 */
type ShellInvocation = {
  /**
   Executable path or bare name resolved through `PATH`.
   */
  readonly command: string;

  /**
   Flags preceding the command text, so the text is never read as a filename.
   */
  readonly args: readonly string[];
};

/**
 Replacement executability probe, so tests resolve shells without touching disk.
 */
type ExistsProbe = (path: string) => Promise<boolean>;

/**
 One `PATH` candidate paired with its probe result.
 */
type CandidateProbe = {
  /**
   Absolute path the probe examined.
   */
  readonly candidate: string;

  /**
   Whether that path is executable by this process.
   */
  readonly executable: boolean;
};

//endregion Types

//region Probing

/**
 Reports whether a path exists and is executable by this process.
 
 @param path - filesystem path to probe
 
 @returns whether the path can be executed
 
 @example
 ```ts
 await executableExists('/bin/bash');
 ```
 */
async function executableExists(path: string, ): Promise<boolean> {
  /**
   Function-scoped logger naming this boundary in every record.
   */
  const l = tagged({
    tag: executableExists.name,
    l: bashPokeLogger,
  }, );
  try {
    await access(
      path,
      constants.X_OK,
    );
    return true;
  }
  catch (error: unknown) {
    // An absent or non-executable candidate is the expected outcome of probing,
    // not a fault, so resolution continues with the next candidate. The value is
    // logged rather than discarded so a permission surprise stays visible.
    l.debug(`shell candidate ${path} is not executable: ${caughtValueText(error, )}`, );
    return false;
  }
}

/**
 Lists every `PATH` directory holding an executable with the given name.
 
 @param name - executable name to search for
 
 @param pathValue - `PATH` content, split on the platform delimiter
 
 @param exists - executability probe
 
 @returns executable candidate paths in `PATH` order
 
 @example
 ```ts
 await executablesOnPath({ name: 'bash', pathValue: '/usr/bin', exists: executableExists, },);
 ```
 */
async function executablesOnPath(
  {
    name,
    pathValue,
    exists,
  }: {
    readonly name: string;
    readonly pathValue: string;
    readonly exists: ExistsProbe;
  },
): Promise<readonly string[]> {
  /**
   Absolute candidates, one per non-empty `PATH` entry.
   */
  const candidates: string[] = pathValue
    .split(delimiter, )
    .filter(function isUsableDirectory(entry: string, ): boolean {
      return entry.length > 0;
    }, )
    .map(function toCandidate(entry: string, ): string {
      return join(
        entry,
        name,
      );
    }, );

  /**
   Probe results, gathered concurrently because each is an independent syscall.
   */
  const probes: readonly CandidateProbe[] = await Promise.all(
    candidates.map(async function probeCandidate(candidate: string, ): Promise<CandidateProbe> {
      return {
        candidate,
        executable: await exists(candidate, ),
      };
    }, ),
  );

  return probes
    .filter(function isExecutable(probe: CandidateProbe, ): boolean {
      return probe.executable;
    }, )
    .map(function toPath(probe: CandidateProbe, ): string {
      return probe.candidate;
    }, );
}

//endregion Probing

//region Resolution

/**
 Resolves the shell used to run a background command.
 
 Resolution mirrors Pi's own documented order, `/bin/bash` then `bash` on
 `PATH` then `sh`, because the Pi setting that would override it is unreachable
 from an extension and diverging would change command semantics for the same
 keystroke.
 
 @param exists - executability probe
 
 @param pathValue - `PATH` content
 
 @returns shell command plus the flag pair that carries command text
 
 @example
 ```ts
 await resolveShell({ pathValue: process.env.PATH ?? '', },);
 ```
 */
async function resolveShell(
  {
    exists = executableExists,
    pathValue,
  }: {
    readonly exists?: ExistsProbe;
    readonly pathValue: string;
  },
): Promise<ShellInvocation> {
  /**
   Function-scoped logger recording which candidate won.
   */
  const l = tagged({
    tag: resolveShell.name,
    l: bashPokeLogger,
  }, );

  /**
   Flags carrying command text, identical for every candidate.
   */
  const args: readonly string[] = [SHELL_COMMAND_FLAG, ];

  if (isAbsolute(ABSOLUTE_BASH_PATH, ) && (await exists(ABSOLUTE_BASH_PATH, ))) {
    l.debug(`resolved shell ${ABSOLUTE_BASH_PATH}`, );
    return {
      command: ABSOLUTE_BASH_PATH,
      args,
    };
  }

  /**
   Bash candidates on `PATH`, which covers systems without `/bin/bash`.
   */
  const bashCandidates = await executablesOnPath({
    name: BASH_COMMAND_NAME,
    pathValue,
    exists,
  }, );
  /**
   First bash on `PATH`, absent when none is executable.
   */
  const [bashPath] = bashCandidates;
  if (bashPath !== undefined) {
    l.debug(`resolved shell ${bashPath}`, );
    return {
      command: bashPath,
      args,
    };
  }

  /**
   POSIX shell candidates, the last resolution step Pi documents.
   */
  const shCandidates = await executablesOnPath({
    name: SH_COMMAND_NAME,
    pathValue,
    exists,
  }, );
  /**
   First POSIX shell on `PATH`, absent when none is executable.
   */
  const [shPath] = shCandidates;
  if (shPath !== undefined) {
    l.debug(`resolved shell ${shPath}`, );
    return {
      command: shPath,
      args,
    };
  }

  // Nothing is executable on this machine, so the bare name is returned and the
  // spawn failure reaches the user as a poke instead of an extension crash.
  l.warn(`no executable shell found; falling back to bare ${SH_COMMAND_NAME}`, );
  return {
    command: SH_COMMAND_NAME,
    args,
  };
}

//endregion Resolution

export {
  executableExists,
  executablesOnPath,
  resolveShell,
};

export type {
  CandidateProbe,
  ExistsProbe,
  ShellInvocation,
};
