/**
 Stand-in `cargo` for the build step's tests, so a control proves what the step asks Cargo and in what order without
 building a crate or touching a build directory of this machine.

 The stand-in is a program under a fixture directory, named by `--cargo`, that answers `metadata` with the workspace the
 fixture was made for, records every call with whether the program the step names was there at that moment, and either
 writes that program or writes nothing, as the fixture asks.

 In TS you'd write (pseudocode):
 ```ts
 // await using standIn = await createCargoStandIn({ ... }); buildFresh({ cargoProgram: standIn.program, ... })
 ```
 */
import {
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { isMissingPathError } from './native-gate-paths.ts';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 Scratch parent of every fixture directory, resolved so a path Cargo names back is the same text as the fixture's own;
 never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  await realpath(join(
    homedir(),
    'temp',
    'agent',
  ),),
);

/**
 Environment variable that names the fixture directory the stand-in writes its calls into.
 */
export const standInDirectoryVariable = 'BUILD_STAND_IN_DIR';

/**
 Environment variable that names the target directory the stand-in's `metadata` reports.
 */
export const standInTargetDirectoryVariable = 'BUILD_STAND_IN_TARGET_DIRECTORY';

/**
 Environment variable that names the build directory the stand-in's `metadata` reports; none is reported when it is
 unset.
 */
export const standInBuildDirectoryVariable = 'BUILD_STAND_IN_BUILD_DIRECTORY';

/**
 Environment variable that names the workspace root the stand-in's `metadata` reports.
 */
export const standInWorkspaceRootVariable = 'BUILD_STAND_IN_WORKSPACE_ROOT';

/**
 Environment variable that names the package the stand-in's `metadata` reports as the workspace's only member.
 */
export const standInMemberVariable = 'BUILD_STAND_IN_MEMBER';

/**
 Environment variable that names the manifest the stand-in's `metadata` reports for that member.
 */
export const standInManifestVariable = 'BUILD_STAND_IN_MANIFEST';

/**
 Environment variable that names the program the stand-in writes on `build`, which a fixture sets and a case about no
 program leaves unset.
 */
export const standInArtifactVariable = 'BUILD_STAND_IN_ARTIFACT';

/**
 Environment variable that, when it is `1`, makes the stand-in's `clean` fail with the words a broken build directory
 prints.
 */
export const standInCleanFailsVariable = 'BUILD_STAND_IN_CLEAN_FAILS';

/**
 Environment variable that, when it is set, makes the stand-in wait that many milliseconds before it answers, so a
 control proves the step's own bound without waiting the real one.
 */
export const standInDelayVariable = 'BUILD_STAND_IN_DELAY_MILLISECONDS';

/**
 What one recorded call of the stand-in was.
 */
export type CargoCall = {
  /**
   The arguments, in the order the step passed them.
   */
  readonly args: readonly string[];
  /**
   Whether the program the step names was at its path when the stand-in was started.
   */
  readonly artifactThere: boolean;
};

/**
 The stand-in program: records the call, answers `metadata` for the fixture's workspace, and writes the fixture's
 program on `build` when the fixture names one.
 */
const standInCargoProgram = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const args = process.argv.slice(2);
const directory = process.env.${standInDirectoryVariable};
const artifact = process.env.${standInArtifactVariable};
fs.appendFileSync(
  path.join(directory, 'cargo-calls.jsonl'),
  JSON.stringify({ args, artifactThere: artifact ? fs.existsSync(artifact) : false }) + '\\n',
);
const delay = Number(process.env.${standInDelayVariable} ?? '0');
if (Number.isFinite(delay) && delay > 0) {
  const slept = Date.now();
  while (Date.now() - slept < delay) { /* wait as the fixture asked */ }
}
if (args[0] === 'metadata') {
  const build = process.env.${standInBuildDirectoryVariable};
  process.stdout.write(JSON.stringify({
    workspace_root: process.env.${standInWorkspaceRootVariable},
    target_directory: process.env.${standInTargetDirectoryVariable},
    ...(build === undefined ? {} : { build_directory: build }),
    packages: [{ name: process.env.${standInMemberVariable}, manifest_path: process.env.${standInManifestVariable} }],
  }));
  process.exit(0);
}
if (args[0] === 'clean' && process.env.${standInCleanFailsVariable} === '1') {
  process.stderr.write('error: failed to remove the stand-in\\'s build state\\n');
  process.exit(101);
}
if (args[0] === 'build' && artifact) {
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  fs.writeFileSync(artifact, '#!/bin/sh\\necho stand-in\\n', { mode: 0o700 });
}
process.exit(0);
`;

/**
 Fixture holding the stand-in program and the environment that describes the workspace it answers for.
 */
export type CargoStandIn = AsyncDisposable & {
  /**
   The fixture's own directory.
   */
  readonly directory: string;
  /**
   The stand-in program, which a control passes to the build step as `--cargo`.
   */
  readonly program: string;
  /**
   The environment the step runs with: the process's own, so the stand-in's own shebang finds its interpreter, and every
   fact the fixture described.
   */
  readonly environment: NodeJS.ProcessEnv;
};

/**
 What a control describes for one fixture.
 */
export type CargoStandInFacts = {
  /**
   The target directory the stand-in's `metadata` reports.
   */
  readonly targetDirectory: string;
  /**
   The build directory it reports, which a control leaves unset when its case is about a Cargo that names none.
   */
  readonly buildDirectory?: string;
  /**
   The workspace root it reports.
   */
  readonly workspaceRoot: string;
  /**
   The member name it reports.
   */
  readonly member: string;
  /**
   The manifest path it reports for that member.
   */
  readonly manifestPath: string;
  /**
   The program it writes on `build`; a case about no program leaves it unset.
   */
  readonly artifact?: string;
  /**
   Whether its `clean` fails with Cargo's own words about a build directory it cannot empty.
   */
  readonly cleanFails?: boolean;
  /**
   Milliseconds it waits before it answers, which a control sets to prove the step's bound.
   */
  readonly delayMilliseconds?: number;
};

/**
 Creates the stand-in program of one fixture.

 @param facts - the workspace the stand-in answers for, and what its `build` writes

 @returns the fixture; declare it with `await using`

 @example
 ```ts
 await using standIn = await createCargoStandIn({ targetDirectory: '/w/target', workspaceRoot: '/w', member: 'probe', manifestPath: '/w/Cargo.toml' });
 ```
 */
export async function createCargoStandIn(facts: CargoStandInFacts): Promise<CargoStandIn> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   Fresh directory for this fixture.
   */
  const directory = await mkdtemp(join(
    scratchParent,
    'git-policy-build-fresh-',
  ));
  /**
   The stand-in program itself.
   */
  const program = join(
    directory,
    'cargo',
  );
  await writeFile(
    program,
    standInCargoProgram,
    { mode: ownerOnlyMode },
  );
  return {
    directory,
    program,
    environment: {
      ...process.env,
      [standInDirectoryVariable]: directory,
      [standInTargetDirectoryVariable]: facts.targetDirectory,
      [standInWorkspaceRootVariable]: facts.workspaceRoot,
      [standInMemberVariable]: facts.member,
      [standInManifestVariable]: facts.manifestPath,
      ...(facts.buildDirectory === undefined ? {} : { [standInBuildDirectoryVariable]: facts.buildDirectory }),
      ...(facts.artifact === undefined ? {} : { [standInArtifactVariable]: facts.artifact }),
      ...(facts.cleanFails === true ? { [standInCleanFailsVariable]: '1' } : {}),
      ...(facts.delayMilliseconds === undefined ? {} : { [standInDelayVariable]: String(facts.delayMilliseconds,) }),
    },
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Reads one recorded line as a call, as a list of one call or none.

 @param line - the line the stand-in wrote

 @returns the call, or an empty list when the line is not one

 @example
 ```ts
 const calls = recorded.split('\\n').flatMap(callOrNothing);
 ```
 */
function callOrNothing(line: string): readonly CargoCall[] {
  /**
   The parsed line, which the stand-in wrote as one object.
   */
  const parsed: unknown = JSON.parse(line,);
  if ((typeof parsed) !== 'object') return [];
  if (parsed === null) return [];
  /**
   The arguments it recorded.
   */
  const args: unknown = Reflect.get(
    parsed,
    'args',
  );
  /**
   Whether the program was there when it was started.
   */
  const artifactThere: unknown = Reflect.get(
    parsed,
    'artifactThere',
  );
  if (!Array.isArray(args)) return [];
  if ((typeof artifactThere) !== 'boolean') return [];
  return [{
    args: args.map(function asWord(word: unknown): string {
      return (typeof word) === 'string' ? word : '';
    },),
    artifactThere,
  }];
}

/**
 Reads what the stand-in was asked, one record per call.

 @param directory - the fixture's directory

 @returns the calls, in the order the step made them

 @throws Error when the record is there and cannot be read, which is no fixture this test made

 @example
 ```ts
 const calls = await recordedCargoCalls({ directory: standIn.directory });
 ```
 */
export async function recordedCargoCalls({ directory }: { readonly directory: string; }): Promise<readonly CargoCall[]> {
  /**
   The record itself, as the stand-in wrote it.
   */
  const recorded = await textOrNothing({ path: join(
    directory,
    'cargo-calls.jsonl',
  ) },);
  return recorded
    .split('\n',)
    .filter(function isALine(line: string): boolean {
      return line !== '';
    },)
    .flatMap(callOrNothing);
}

/**
 Reads a file, answering with no lines when it is not there.

 @param path - the file to read

 @returns its text, or empty text when the file is absent

 @throws Error when the file is there and cannot be read

 @example
 ```ts
 const recorded = await textOrNothing({ path: '/w/cargo-calls.jsonl' });
 ```
 */
async function textOrNothing({ path }: { readonly path: string; }): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (caught) {
    // Absent means the stand-in was never started; a file that cannot be read is nobody's "no calls".
    if (!isMissingPathError({ error: caught })) throw caught;
    return '';
  }
}
