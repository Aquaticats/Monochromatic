import { resolve } from 'node:path';

import { buildCommitMessage } from './commit-message.ts';
import {
  defaultMemoryMax,
  defaultUnitSuffix,
  manifestFileName,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import {
  repeatedValues,
  requiredValue,
  rootOf,
  splitArguments,
  valueOr,
} from './option-parsing.ts';
import {
  parseCommitScope,
  parseCrateName,
  parseRepositoryPath,
  parseSystemdValue,
  parseTaskName,
  parseUnitName,
} from './validate.ts';
import { assertReleaseVersion } from './version.ts';

/**
 Everything a release needs, validated.
 */
export type ReleaseOptions = {
  /**
   Absolute path of the repository root; every other path is relative to it.
   */
  readonly root: string;
  /**
   Name of the crate being released, as in its manifest and on crates.io.
   */
  readonly crateName: string;
  /**
   Repository-relative directory of the crate, holding its `Cargo.toml` and `Cargo.lock`.
   */
  readonly crateDirectory: string;
  /**
   Package scope of the commit subject.
   */
  readonly scope: string;
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
   Repository-relative manifests that pin the crate exactly; each is moved with it and committed with its lockfile.
   */
  readonly pinFiles: readonly string[];
  /**
   Binaries the offline check builds; empty to check the whole crate.
   */
  readonly checkBins: readonly string[];
  /**
   Mise tasks that refresh lockfiles after the edit, in order.
   */
  readonly lockTasks: readonly string[];
  /**
   Systemd `MemoryMax` of the publish dry run.
   */
  readonly memoryMax: string;
  /**
   Name of the transient unit of the publish dry run.
   */
  readonly unitName: string;
};

/**
 Option definitions of the release command in the `node:util` `parseArgs` format.
 */
const releaseOptionDefinitions = {
  'root': { type: 'string' },
  'crate-name': { type: 'string' },
  'crate-dir': { type: 'string' },
  'scope': { type: 'string' },
  'pin-file': {
    type: 'string',
    multiple: true
  },
  'check-bin': {
    type: 'string',
    multiple: true
  },
  'lock-task': {
    type: 'string',
    multiple: true
  },
  'memory-max': { type: 'string' },
  'unit-name': { type: 'string' },
} as const;

/**
 Check that a pin file names a manifest, because its lockfile is found beside it.

 @param value - repository-relative path already checked as a path

 @returns the value unchanged

 @throws CrateReleaseError when the path does not end in the manifest file name

 @example
 ```ts
 assertManifestPath('package/a.fuzz/Cargo.toml');
 ```
 */
function assertManifestPath(value: string): string {
  if ((value !== manifestFileName) && (!value.endsWith(`/${manifestFileName}`)))
    throw new CrateReleaseError(
      `--pin-file must name a ${manifestFileName} file, because its lockfile is committed beside it, got ${JSON.stringify(value)}.`,
    );
  return value;
}

/**
 Read and validate the command line of the release command.

 @param argv - arguments after the script path: options, then `<from> <to> <note>`

 @param root - repository root used when `--root` is absent, or a function that finds it only when needed

 @returns the validated options

 @throws CrateReleaseError naming the argument and the valid form when anything is missing, malformed or unsafe

 @example
 ```ts
 parseReleaseArguments({ argv: ['--crate-name', 'c', '--crate-dir', 'p', '--scope', 's', '0.1.0', '0.1.1', 'n'], root: '/r' });
 ```
 */
export function parseReleaseArguments({
  argv,
  root,
}: {
  readonly argv: readonly string[];
  readonly root: string | (() => string);
}): ReleaseOptions {
  /**
   Options and positionals, split but not yet validated.
   */
  const raw = splitArguments({
    argv,
    options: releaseOptionDefinitions
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
   Crate name, checked.
   */
  const crateName = parseCrateName({
    option: '--crate-name',
    value: requiredValue({
      raw,
      name: 'crate-name'
    })
  });
  /**
   Commit scope, checked.
   */
  const scope = parseCommitScope({
    option: '--scope',
    value: requiredValue({
      raw,
      name: 'scope'
    })
  });
  // Building the subject here refuses a bad note before any file changes.
  buildCommitMessage({
    scope,
    version: to,
    note
  });
  return {
    root: resolve(rootOf({
      raw,
      root
    })),
    crateName,
    crateDirectory: parseRepositoryPath({
      option: '--crate-dir',
      value: requiredValue({
        raw,
        name: 'crate-dir'
      })
    }),
    scope,
    from,
    to,
    note,
    pinFiles: repeatedValues({
      raw,
      name: 'pin-file'
    })
      .map(function parsePinFile(value) {
        return assertManifestPath(parseRepositoryPath({
          option: '--pin-file',
          value
        }));
      }),
    checkBins: repeatedValues({
      raw,
      name: 'check-bin'
    })
      .map(function parseCheckBin(value) {
        return parseCrateName({
          option: '--check-bin',
          value
        });
      }),
    lockTasks: repeatedValues({
      raw,
      name: 'lock-task'
    })
      .map(function parseLockTask(value) {
        return parseTaskName({
          option: '--lock-task',
          value
        });
      }),
    memoryMax: parseSystemdValue({
      option: '--memory-max',
      value: valueOr({
        raw,
        name: 'memory-max',
        fallback: defaultMemoryMax
      })
    }),
    unitName: parseUnitName({
      option: '--unit-name',
      value: valueOr({
        raw,
        name: 'unit-name',
        fallback: `${crateName}${defaultUnitSuffix}`
      }),
    }),
  };
}
