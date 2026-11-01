import { resolve } from 'node:path';

import {
  defaultAssetTemplates,
  defaultAttempts,
  defaultCurlMaxTimeSeconds,
  defaultPollIntervalSeconds,
  defaultRunListLimit,
  defaultTagTemplate,
  defaultUserAgent,
  defaultWorkflow,
  versionPlaceholder,
} from './constants.ts';
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
  parseCrateName,
  parseRepository,
  parseSha,
  parseTemplate,
  parseUserAgent,
  parseWholeNumber,
  parseWorkflowName,
} from './validate.ts';
import { assertReleaseVersion } from './version.ts';

/**
 Everything the publish check needs, validated.
 */
export type WaitOptions = {
  /**
   Absolute path of the repository root, where `gh` runs so it finds the repository.
   */
  readonly root: string;
  /**
   Name of the crate, as on crates.io.
   */
  readonly crateName: string;
  /**
   Lower-case commit hash or prefix whose workflow run is awaited.
   */
  readonly sha: string;
  /**
   Version expected on crates.io and in the release.
   */
  readonly version: string;
  /**
   Workflow file that publishes.
   */
  readonly workflow: string;
  /**
   `owner/name` passed to `gh`; absent to let `gh` read the repository from the working directory.
   */
  readonly repo?: string;
  /**
   Looks for the run before giving up.
   */
  readonly attempts: number;
  /**
   Seconds between two looks.
   */
  readonly pollIntervalSeconds: number;
  /**
   Recent runs listed on each look.
   */
  readonly runLimit: number;
  /**
   Template of the release tag.
   */
  readonly tagTemplate: string;
  /**
   Templates of the asset names the release must hold.
   */
  readonly assetTemplates: readonly string[];
  /**
   `User-Agent` sent to crates.io.
   */
  readonly userAgent: string;
  /**
   Seconds before `curl` gives up.
   */
  readonly maxTimeSeconds: number;
};

/**
 Option definitions of the wait command in the `node:util` `parseArgs` format.
 */
const waitOptionDefinitions = {
  'root': { type: 'string' },
  'crate-name': { type: 'string' },
  'workflow': { type: 'string' },
  'repo': { type: 'string' },
  'attempts': { type: 'string' },
  'poll-interval': { type: 'string' },
  'run-limit': { type: 'string' },
  'tag-template': { type: 'string' },
  'asset-template': {
    type: 'string',
    multiple: true
  },
  'user-agent': { type: 'string' },
  'max-time': { type: 'string' },
} as const;

/**
 Read a whole-number option, or its default when absent.

 @param raw - split command line

 @param name - long option name without the hyphens

 @param fallback - default, checked like a given value

 @returns the number

 @example
 ```ts
 numberOption({ raw, name: 'attempts', fallback: 45 });
 ```
 */
function numberOption({
  raw,
  name,
  fallback,
}: {
  readonly raw: RawArguments;
  readonly name: string;
  readonly fallback: number;
}): number {
  return parseWholeNumber({
    option: `--${name}`,
    value: valueOr({
      raw,
      name,
      fallback: String(fallback)
    })
  });
}

/**
 Read and validate the command line of the wait command.

 @param argv - arguments after the script path: options, then `<sha> <version>`

 @param root - repository root used when `--root` is absent, or a function that finds it only when needed

 @returns the validated options

 @throws CrateReleaseError naming the argument and the valid form when anything is missing, malformed or unsafe

 @example
 ```ts
 parseWaitArguments({ argv: ['--crate-name', 'c', 'abcdef0', '1.0.0'], root: '/r' });
 ```
 */
export function parseWaitArguments({
  argv,
  root,
}: {
  readonly argv: readonly string[];
  readonly root: string | (() => string);
}): WaitOptions {
  /**
   Options and positionals, split but not yet validated.
   */
  const raw = splitArguments({
    argv,
    options: waitOptionDefinitions
  });
  /**
   The two positional arguments and anything beyond them.
   */
  const [sha, version, ...extra] = raw.positionals;
  if ((sha === undefined) || (version === undefined)
    || (extra.length > 0))
    throw new CrateReleaseError(
      `Expected exactly two arguments <sha> <version> after the options, got ${String(raw.positionals
        .length)}.`,
    );
  /**
   Repository, when one was named.
   */
  const repo = optionValue({
    raw,
    name: 'repo'
  });
  /**
   Asset templates as given; the defaults apply when none is given.
   */
  const assetTemplates = repeatedValues({
    raw,
    name: 'asset-template'
  });
  return {
    root: resolve(rootOf({
      raw,
      root
    })),
    crateName: parseCrateName({
      option: '--crate-name',
      value: requiredValue({
        raw,
        name: 'crate-name'
      })
    }),
    sha: parseSha({
      option: '<sha>',
      value: sha
    }),
    version: assertReleaseVersion({
      option: '<version>',
      value: version
    }),
    workflow: parseWorkflowName({
      option: '--workflow',
      value: valueOr({
        raw,
        name: 'workflow',
        fallback: defaultWorkflow
      })
    }),
    ...(repo.found ? {
      repo: parseRepository({
        option: '--repo',
        value: repo.value
      })
    } : {}),
    attempts: numberOption({
      raw,
      name: 'attempts',
      fallback: defaultAttempts
    }),
    pollIntervalSeconds: numberOption({
      raw,
      name: 'poll-interval',
      fallback: defaultPollIntervalSeconds
    }),
    runLimit: numberOption({
      raw,
      name: 'run-limit',
      fallback: defaultRunListLimit
    }),
    tagTemplate: parseTemplate({
      option: '--tag-template',
      value: valueOr({
        raw,
        name: 'tag-template',
        fallback: defaultTagTemplate
      }),
      required: [versionPlaceholder],
    }),
    assetTemplates: (assetTemplates.length === 0 ? defaultAssetTemplates : assetTemplates)
      .map(function parseAssetTemplate(value) {
        return parseTemplate({
          option: '--asset-template',
          value,
          required: [versionPlaceholder]
        });
      }),
    userAgent: parseUserAgent({
      option: '--user-agent',
      value: valueOr({
        raw,
        name: 'user-agent',
        fallback: defaultUserAgent
      })
    }),
    maxTimeSeconds: numberOption({
      raw,
      name: 'max-time',
      fallback: defaultCurlMaxTimeSeconds
    }),
  };
}
