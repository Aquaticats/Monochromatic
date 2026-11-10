import { resolve } from 'node:path';

import { buildCommitMessage } from './commit-message.ts';
import {
  type CrateRegistry,
  type DispatchGateCommand,
  findCrate,
} from './crate-registry.ts';
import { CrateReleaseError } from './error.ts';
import {
  optionValue,
  requiredValue,
  rootOf,
  splitArguments,
  switchGiven,
} from './option-parsing.ts';
import type { ProgramCall } from './run-program.ts';
import {
  parseCrateName,
  parseRepository,
} from './validate.ts';
import { assertReleaseVersion } from './version.ts';

/**
 Everything the release dispatch needs, validated.
 */
export type DispatchOptions = {
  /**
   Absolute path of the repository root, where `gh` runs so it finds the repository.
   */
  readonly root: string;
  /**
   Crate to release, as listed in the registry.
   */
  readonly crate: string;
  /**
   Version the crate holds now.
   */
  readonly from: string;
  /**
   Version to release.
   */
  readonly to: string;
  /**
   One-line summary that ends the commit subject.
   */
  readonly note: string;
  /**
   True to run everything except the push of the bump and the start of the publish.
   */
  readonly dryRun: boolean;
  /**
   `owner/name` passed to `gh`; absent to let `gh` read the repository from the working directory.
   */
  readonly repo?: string;
  /**
   The crate's own release gate, when its entry in the registry names one: the command a real release waits for, run
   from the crate's directory before the workflow is started at all.
   */
  readonly dispatchGate?: ProgramCall;
};

/**
 Option definitions of the dispatch command in the `node:util` `parseArgs` format.
 */
const dispatchOptionDefinitions = {
  'root': { type: 'string' },
  'crate': { type: 'string' },
  'repo': { type: 'string' },
  'dry-run': { type: 'boolean' },
} as const;

/**
 Resolve the crate's dispatch gate into the program call the dispatch runs.

 The gate runs from the crate's own directory, so the paths it names are the crate's, and never the repository root.

 @param gate - the command the crate's registry entry names

 @param root - absolute repository root

 @param crateDirectory - repository-relative directory of the crate

 @returns the program call, ready to run

 @example
 ```ts
 gateCall({ gate: { program: 'node', args: ['bin/gate.ts'] }, root: '/repo', crateDirectory: 'package/a/cli' });
 ```
 */
function gateCall({
  gate,
  root,
  crateDirectory,
}: {
  readonly gate: DispatchGateCommand;
  readonly root: string;
  readonly crateDirectory: string;
}): ProgramCall {
  return {
    program: gate.program,
    args: gate.args,
    cwd: resolve(
      root,
      crateDirectory
    ),
  };
}

/**
 Read and validate the command line of the dispatch command.

 Everything the workflow would refuse is refused here first, so a typo costs no workflow run: the crate must be in the
 registry, the versions must be release versions that differ, and the note must fit the commit subject of the crate's
 scope.

 @param argv - arguments after the script path: options, then `<from> <to> <note>`

 @param root - repository root used when `--root` is absent, or a function that finds it only when needed

 @param registry - the registry of releasable crates

 @returns the validated options

 @throws CrateReleaseError naming the argument and the valid form when anything is missing, malformed or unsafe

 @example
 ```ts
 parseDispatchArguments({ argv: ['--crate', 'c', '0.1.0', '0.1.1', 'n'], root: '/r', registry });
 ```
 */
export function parseDispatchArguments({
  argv,
  root,
  registry,
}: {
  readonly argv: readonly string[];
  readonly root: string | (() => string);
  readonly registry: () => CrateRegistry;
}): DispatchOptions {
  /**
   Options and positionals, split but not yet validated.
   */
  const raw = splitArguments({
    argv,
    options: dispatchOptionDefinitions
  });
  /**
   The three positional arguments and anything beyond them.
   */
  const [from, to, note, ...extra] = raw.positionals;
  if ((from === undefined) || (to === undefined)
    || (note === undefined)
    || (extra.length > 0))
    throw new CrateReleaseError(
      `Expected exactly three arguments <from> <to> <note> after the options, got ${String(raw.positionals
        .length)}; quote a note that has spaces.`,
    );
  assertReleaseVersion({
    option: '<from>',
    value: from
  });
  assertReleaseVersion({
    option: '<to>',
    value: to
  });
  if (from === to)
    throw new CrateReleaseError(`<to> must differ from <from>; both are ${JSON.stringify(from)}.`);
  /**
   The crate, checked as a name and found in the registry.
   */
  const crate = parseCrateName({
    option: '--crate',
    value: requiredValue({
      raw,
      name: 'crate'
    })
  });
  /**
   The crate's registry entry, which names the scope of the release commit, the manifests that pin the crate, and the
   command a real release waits for.
   */
  const entry = findCrate({
    registry: registry(),
    name: crate
  });
  // Building the subject here refuses a bad note before a workflow run is spent on it.
  buildCommitMessage({
    scope: entry.scope,
    version: to,
    note
  });
  /**
   Repository, when one was named.
   */
  const repo = optionValue({
    raw,
    name: 'repo'
  });
  /**
   Absolute repository root, from which the crate's own directory is named.
   */
  const rootPath = resolve(rootOf({
    raw,
    root
  }));
  return {
    root: rootPath,
    crate,
    from,
    to,
    note,
    dryRun: switchGiven({
      raw,
      name: 'dry-run'
    }),
    ...(repo.found ? {
      repo: parseRepository({
        option: '--repo',
        value: repo.value
      })
    } : {}),
    ...(entry.dispatchGate === undefined ? {} : {
      dispatchGate: gateCall({
        gate: entry.dispatchGate,
        root: rootPath,
        crateDirectory: entry.crateDirectory
      })
    }),
  };
}
