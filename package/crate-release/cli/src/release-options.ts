import { resolve } from 'node:path';

import { buildCommitMessage } from './commit-message.ts';
import {
  defaultCpuQuota,
  defaultMemoryMax,
  defaultReleaseHost,
  defaultTasksMax,
  defaultUnitSuffix,
  releaseHosts,
} from './constants.ts';
import {
  type CrateEntry,
  type CrateRegistry,
  findCrate,
} from './crate-registry.ts';
import { CrateReleaseError } from './error.ts';
import {
  optionValue,
  type RawArguments,
  repeatedValues,
  requiredValue,
  rootOf,
  splitArguments,
  valueOr,
} from './option-parsing.ts';
import {
  parseCommitScope,
  parseCrateName,
  parseManifestPath,
  parseRepositoryPath,
  parseSystemdValue,
  parseUnitName,
} from './validate.ts';
import { assertReleaseVersion } from './version.ts';

/**
 Where a release runs; see `releaseHosts`.
 */
export type ReleaseHost = (typeof releaseHosts)[number];

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
   Binaries the check builds; empty to check the whole crate.
   */
  readonly checkBins: readonly string[];
  /**
   Repository-relative directories whose lockfile records the changed version of a workspace member after the edit, in
   order.
   */
  readonly lockDirectories: readonly string[];
  /**
   Where the release runs, which decides how cargo is called and whether a manifest that already holds the target is
   accepted.
   */
  readonly host: ReleaseHost;
  /**
   Systemd `MemoryMax` of the publish dry run on a local host.
   */
  readonly memoryMax: string;
  /**
   Systemd `CPUQuota` of the publish dry run on a local host.
   */
  readonly cpuQuota: string;
  /**
   Systemd `TasksMax` of the publish dry run on a local host.
   */
  readonly tasksMax: string;
  /**
   Name of the transient unit of the publish dry run on a local host.
   */
  readonly unitName: string;
};

/**
 Option definitions of the release command in the `node:util` `parseArgs` format.
 */
const releaseOptionDefinitions = {
  'root': { type: 'string' },
  'crate': { type: 'string' },
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
  'lock-dir': {
    type: 'string',
    multiple: true
  },
  'host': { type: 'string' },
  'memory-max': { type: 'string' },
  'cpu-quota': { type: 'string' },
  'tasks-max': { type: 'string' },
  'unit-name': { type: 'string' },
} as const;

/**
 Options that spell out a crate, which `--crate` replaces by reading the registry file.
 */
const crateSpellingOptions: readonly string[] = [
  'crate-name',
  'crate-dir',
  'scope',
  'pin-file',
  'check-bin',
  'lock-dir',
];

/**
 The crate's arguments, either spelled out on the command line or read from the registry by `--crate`.
 */
type CrateArguments = CrateEntry & { readonly crateName: string };

/**
 Read the arguments of the crate from the registry, refusing a command line that also spells them out.

 @param raw - split command line

 @param crate - the `--crate` value

 @param registry - the registry, read only when `--crate` is given

 @returns the crate's name and arguments

 @throws CrateReleaseError when a spelled-out option is given beside `--crate`, or the crate is not in the registry

 @example
 ```ts
 registryArguments({ raw, crate: 'forbidden-strings', registry });
 ```
 */
function registryArguments({
  raw,
  crate,
  registry,
}: {
  readonly raw: RawArguments;
  readonly crate: string;
  readonly registry: () => CrateRegistry;
}): CrateArguments {
  /**
   Spelled-out options that were given beside `--crate`.
   */
  const conflicting = crateSpellingOptions.filter(function isGiven(name) {
    return raw.singles
      .has(name)
      || (repeatedValues({
        raw,
        name
      })
        .length
        > 0);
  });
  if (conflicting.length > 0)
    throw new CrateReleaseError(
      `--crate reads the crate's arguments from the registry, so ${
        conflicting.map(function dashed(name) {
          return `--${name}`;
        })
          .join(', ')
      } must not be given with it; drop them, or drop --crate to spell the crate out.`,
    );
  return {
    crateName: parseCrateName({
      option: '--crate',
      value: crate
    }),
    ...findCrate({
      registry: registry(),
      name: crate
    }),
  };
}

/**
 Read the arguments of the crate as spelled out on the command line.

 @param raw - split command line

 @returns the crate's name and arguments

 @throws CrateReleaseError naming the argument and the valid form when a required option is missing or unsafe

 @example
 ```ts
 spelledArguments(raw);
 ```
 */
function spelledArguments(raw: RawArguments): CrateArguments {
  return {
    crateName: parseCrateName({
      option: '--crate-name',
      value: requiredValue({
        raw,
        name: 'crate-name'
      })
    }),
    crateDirectory: parseRepositoryPath({
      option: '--crate-dir',
      value: requiredValue({
        raw,
        name: 'crate-dir'
      })
    }),
    scope: parseCommitScope({
      option: '--scope',
      value: requiredValue({
        raw,
        name: 'scope'
      })
    }),
    pinFiles: repeatedValues({
      raw,
      name: 'pin-file'
    })
      .map(function parsePinFile(value) {
        return parseManifestPath({
          option: '--pin-file',
          value
        });
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
    lockDirectories: repeatedValues({
      raw,
      name: 'lock-dir'
    })
      .map(function parseLockDirectory(value) {
        return parseRepositoryPath({
          option: '--lock-dir',
          value
        });
      }),
  };
}

/**
 Read the release host.

 @param raw - split command line

 @returns the host named by `--host`, or the default

 @throws CrateReleaseError listing the hosts when the value is none of them

 @example
 ```ts
 parseReleaseHost(raw);
 ```
 */
function parseReleaseHost(raw: RawArguments): ReleaseHost {
  /**
   The value as given, or the default.
   */
  const value = valueOr({
    raw,
    name: 'host',
    fallback: defaultReleaseHost
  });
  /**
   The host the value names, when it names one.
   */
  const host = releaseHosts.find(function isNamed(candidate) {
    return candidate === value;
  });
  if (host === undefined)
    throw new CrateReleaseError(`--host must be one of ${releaseHosts.join(', ')}, got ${JSON.stringify(value)}.`);
  return host;
}

/**
 Read and validate the command line of the release command.

 @param argv - arguments after the script path: options, then `<from> <to> <note>`

 @param root - repository root used when `--root` is absent, or a function that finds it only when needed

 @param registry - the registry of releasable crates, read only when `--crate` is given

 @returns the validated options

 @throws CrateReleaseError naming the argument and the valid form when anything is missing, malformed or unsafe

 @example
 ```ts
 parseReleaseArguments({ argv: ['--crate', 'forbidden-strings', '0.1.0', '0.1.1', 'n'], root: '/r', registry });
 ```
 */
export function parseReleaseArguments({
  argv,
  root,
  registry,
}: {
  readonly argv: readonly string[];
  readonly root: string | (() => string);
  readonly registry: () => CrateRegistry;
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
   The `--crate` value, when given.
   */
  const crate = optionValue({
    raw,
    name: 'crate'
  });
  /**
   The crate's name and arguments, from the registry or spelled out.
   */
  const crateArguments = crate.found
    ? registryArguments({
      raw,
      crate: crate.value,
      registry
    })
    : spelledArguments(raw);
  // Building the subject here refuses a bad note before any file changes.
  buildCommitMessage({
    scope: crateArguments.scope,
    version: to,
    note
  });
  return {
    root: resolve(rootOf({
      raw,
      root
    })),
    ...crateArguments,
    from,
    to,
    note,
    host: parseReleaseHost(raw),
    memoryMax: parseSystemdValue({
      option: '--memory-max',
      value: valueOr({
        raw,
        name: 'memory-max',
        fallback: defaultMemoryMax
      })
    }),
    cpuQuota: parseSystemdValue({
      option: '--cpu-quota',
      value: valueOr({
        raw,
        name: 'cpu-quota',
        fallback: defaultCpuQuota
      })
    }),
    tasksMax: parseSystemdValue({
      option: '--tasks-max',
      value: valueOr({
        raw,
        name: 'tasks-max',
        fallback: defaultTasksMax
      })
    }),
    unitName: parseUnitName({
      option: '--unit-name',
      value: valueOr({
        raw,
        name: 'unit-name',
        fallback: `${crateArguments.crateName}${defaultUnitSuffix}`
      }),
    }),
  };
}
