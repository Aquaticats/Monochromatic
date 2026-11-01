/**
 Command-line argument parsing and invocation validation.
 
 @module
 */

import { parseArgs, } from 'node:util';

import {
  type CliInputArgument,
  parsePositionalInput,
} from './cli-input.ts';
import { CliInvocationError, } from './cli-invocation-error.ts';
import type { ApplyAuthority, } from './plan-model.ts';

/**
 Help-only command that bypasses mode requirement.
 */
export type HelpCliArguments = {
  readonly kind: 'help';
};

/**
 Validated adapter run command.
 */
export type RunCliArguments = {
  readonly kind: 'run';
  readonly mode: 'interactive' | 'non-interactive';
  readonly input: CliInputArgument;
  readonly repositoryUrl?: string;
  readonly applyAuthority?: ApplyAuthority;
};

/**
 Complete validated command-line union.
 */
export type CliArguments = HelpCliArguments | RunCliArguments;

/**
 Raw parse result from Node argument parser.
 */
type RawArguments = ReturnType<typeof parseArgs>;

/**
 Parses tokens through strict Node argument grammar.
 
 @param arguments_ - CLI tokens excluding executable and script paths.
 
 @returns Raw values and positionals.
 
 @throws {@link CliInvocationError} when Node parser rejects syntax.
 */
function parseRaw(arguments_: readonly string[],): RawArguments {
  try {
    return parseArgs({
      args: [...arguments_,],
      strict: true,
      allowPositionals: true,
      options: {
        interactive: {
          type: 'boolean',
          short: 'i',
        },
        'non-interactive': {
          type: 'boolean',
        },
        apply: {
          type: 'boolean',
        },
        'non-security-only': {
          type: 'boolean',
        },
        all: {
          type: 'boolean',
        },
        repo: {
          type: 'string',
        },
        help: {
          type: 'boolean',
        },
      },
    },);
  }
  catch (error: unknown) {
    throw new CliInvocationError(`invalid command arguments: ${String(error,)}`,);
  }
}

/**
 Validates non-interactive apply authority flags.
 
 @param apply - Whether mutation boundary is requested.
 
 @param nonSecurityOnly - Whether security findings must be withheld.
 
 @param all - Whether all security findings are asserted safe to disclose.
 
 @returns Optional authority property for preview or applied run.
 
 @throws {@link CliInvocationError} for contradictory authority.
 */
function applyAuthorityMetadata({
  apply,
  nonSecurityOnly,
  all,
}: {
  readonly apply: boolean;
  readonly nonSecurityOnly: boolean;
  readonly all: boolean;
},): Pick<RunCliArguments, 'applyAuthority'> {
  if (nonSecurityOnly && all) {
    throw new CliInvocationError('`--non-security-only` and `--all` are mutually exclusive',);
  }
  if ((!apply) && (nonSecurityOnly || all)) {
    throw new CliInvocationError('security authority flags require `--apply`',);
  }
  if (!apply) {
    return {};
  }
  if (nonSecurityOnly) {
    return { applyAuthority: 'non-security-only', };
  }
  if (all) {
    return { applyAuthority: 'all', };
  }
  return { applyAuthority: 'default', };
}

/**
 Parses and validates complete command invocation.
 
 @param arguments - CLI tokens excluding executable and script paths.
 
 @returns Help or runnable explicit-mode command.
 
 @throws {@link CliInvocationError} for invocation misuse.
 
 @example
 ```ts
 parseCliArguments({ arguments: ['--non-interactive', 'review.json'] });
 ```
 */
export function parseCliArguments({
  arguments: arguments_,
}: {
  readonly arguments: readonly string[];
},): CliArguments {
  /**
   Strict raw option and positional parse.
   */
  const raw = parseRaw(arguments_,);
  if (raw.values
    .help
    === true) {
    return { kind: 'help', };
  }
  /**
   Explicit interactive mode selection.
   */
  const interactive = raw.values
    .interactive
    === true;
  /**
   Explicit non-interactive mode selection.
   */
  const nonInteractive = raw.values['non-interactive'] === true;
  if (interactive === nonInteractive) {
    throw new CliInvocationError('exactly one of `--interactive` or `--non-interactive` is required',);
  }
  /**
   Optional canonical repository URL string for later validation.
   */
  const repositoryMetadata = (typeof raw.values
    .repo) === 'string'
    ? { repositoryUrl: raw.values
      .repo, }
    : {};
  /**
   Mutation flag state.
   */
  const apply = raw.values
    .apply
    === true;
  /**
   Non-security-only authority state.
   */
  const nonSecurityOnly = raw.values['non-security-only'] === true;
  /**
   All-findings disclosure authority state.
   */
  const all = raw.values
    .all
    === true;
  if (interactive) {
    if (apply || nonSecurityOnly
      || all) {
      throw new CliInvocationError('interactive mode cannot use apply authority flags',);
    }
    return {
      kind: 'run',
      mode: 'interactive',
      input: parsePositionalInput({
        positionals: raw.positionals,
        interactive: true,
      }),
      ...repositoryMetadata,
    };
  }
  return {
    kind: 'run',
    mode: 'non-interactive',
    input: parsePositionalInput({
      positionals: raw.positionals,
      interactive: false,
    }),
    ...repositoryMetadata,
    ...applyAuthorityMetadata({
      apply,
      nonSecurityOnly,
      all,
    }),
  };
}
