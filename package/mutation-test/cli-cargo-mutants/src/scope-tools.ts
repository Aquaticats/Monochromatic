import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { evidenceJson } from './campaign-files.ts';
import {
  usesOnly,
  decimalDigits
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';
import { CargoMutantsToolingError } from './error.ts';
import { exclusionArguments } from './excluded-mutants.ts';
import { mapInSequence } from './in-sequence.ts';
import { runCaptured } from './run-captured.ts';

/**
 One scope of a product: a name and the cargo-mutants options that select its mutants.
 */
export type ScopeListing = readonly [
  string,
  readonly string[]
];

/**
 Result of comparing the union of the scope listings with the unscoped listing.
 */
export type CoverageSummary = {
  readonly unscoped: number;
  readonly unscopedDistinct: number;
  readonly union: number;
  readonly perScope: Readonly<Record<string, number>>;
  readonly outside: readonly string[];
  readonly unknown: readonly string[];
};

/**
 Separator of the two numbers in a `--shard k/n` value.
 */
const shardSeparator = '/';

/**
 List one scope's mutant names on the host, exactly as a campaign of that scope would select them.

 @param select - cargo-mutants options that select the scope's mutants, such as `--file` pairs
 
 @param additionalExclusions - patterns the product excludes on top of the repository-wide kinds
 
 @returns the mutant names, one per line of cargo-mutants' listing
 
 @throws CargoMutantsToolingError when cargo-mutants exits nonzero
 
 @throws Error when cargo cannot be started
 
 @example
 ```ts
 await listMutants({ select: ['--file', 'src/a.rs'], additionalExclusions: [] });
 ```
 */
export async function listMutants({
  select,
  additionalExclusions,
}: {
  readonly select: readonly string[];
  readonly additionalExclusions: readonly string[];
}): Promise<string[]> {
  /**
   What the listing left behind.
   */
  const result = await runCaptured({
    program: 'cargo',
    args: [
      'mutants',
      '--list',
      '--no-config',
      ...exclusionArguments(additionalExclusions),
      ...select,
    ],
    capture: true,
  });
  if (result.status !== 0)
    throw new CargoMutantsToolingError(`cargo mutants --list failed (status ${result.status}): ${result.stderr}`);
  return result.stdout
    .split('\n')
    .filter(function isName(line) {
      return line.length > 0;
    });
}

/**
 Compare the union of the scope listings with the unscoped listing, name by name.

 A mutant outside every scope is never tried by any campaign, so any name outside, any name only in a scope, or any
 repeated name in the unscoped listing means the scopes do not hold exactly the product's mutants.

 @param all - the unscoped listing
 
 @param listings - each scope's name and listing
 
 @returns the counts and the names that differ
 
 @example
 ```ts
 coverageSummary({ all: ['a'], listings: [['--one', ['a']]] });
 ```
 */
export function coverageSummary({
  all,
  listings,
}: {
  readonly all: readonly string[];
  readonly listings: readonly (readonly [
    string,
    readonly string[]
  ])[];
}): CoverageSummary {
  /**
   Every name some scope lists.
   */
  const union = new Set(listings.flatMap(function names([, listed]) {
    return listed;
  }));
  /**
   Distinct names of the unscoped listing.
   */
  const unscopedSet = new Set(all);
  return {
    unscoped: all.length,
    unscopedDistinct: unscopedSet.size,
    union: union.size,
    perScope: Object.fromEntries(listings.map(function count([option, listed]) {
      return [
        option,
        listed.length
      ];
    })),
    outside: all.filter(function isOutside(name) {
      return !union.has(name);
    }),
    unknown: [...union].filter(function isUnknown(name) {
      return !unscopedSet.has(name);
    }),
  };
}

/**
 Prove that the scopes together hold every mutant of the unscoped listing, and keep every listing as evidence.

 @param scopes - each scope's option name and the cargo-mutants options that select it
 
 @param unscopedSelect - the options of the unscoped listing, usually none
 
 @param additionalExclusions - patterns the product excludes on top of the repository-wide kinds
 
 @param evidenceRoot - directory that keeps the evidence of every run
 
 @throws CargoMutantsToolingError when the scopes do not hold exactly the unscoped mutants
 
 @example
 ```ts
 await checkScopeCoverage({ scopes: [], unscopedSelect: [], additionalExclusions: [], evidenceRoot: '/e' });
 ```
 */
export async function checkScopeCoverage({
  scopes,
  unscopedSelect,
  additionalExclusions,
  evidenceRoot,
}: {
  readonly scopes: readonly ScopeListing[];
  readonly unscopedSelect: readonly string[];
  readonly additionalExclusions: readonly string[];
  readonly evidenceRoot: string;
}): Promise<void> {
  await mkdir(
    evidenceRoot,
    { recursive: true },
  );
  /**
   Evidence directory of this check.
   */
  const evidence = await mkdtemp(join(
    evidenceRoot,
    'coverage-',
  ));
  /**
   The unscoped listing.
   */
  const all = await listMutants({
    select: unscopedSelect,
    additionalExclusions,
  });
  await writeFile(
    join(
      evidence,
      'unscoped.txt',
    ),
    `${all.join('\n')}\n`,
  );
  /**
   Each scope's name and listing, in scope order.
   */
  const listings = await mapInSequence({
    items: scopes,
    run: async function listScope([option, select]) {
      /**
       Names this scope lists.
       */
      const names = await listMutants({
        select,
        additionalExclusions,
      });
      await writeFile(
        join(
          evidence,
          `${option.slice(2)}.txt`,
        ),
        `${names.join('\n')}\n`,
      );
      return [
        option,
        names
      ] as const;
    },
  });
  /**
   Counts and differences.
   */
  const summary = coverageSummary({
    all,
    listings,
  });
  await writeFile(
    join(
      evidence,
      'summary.json',
    ),
    evidenceJson(summary),
  );
  console.log(JSON.stringify(
    summary,
    null,
    2,
  ));
  console.log(`Coverage evidence: ${evidence}`);
  if ((summary.outside
    .length
    > 0) || (summary.unknown
      .length
      > 0)
    || (summary.unscopedDistinct !== summary.unscoped))
    throw new CargoMutantsToolingError(`The scopes do not hold exactly the unscoped mutants: ${summary.outside
      .length} outside every scope, ${summary.unknown
        .length} not in the unscoped listing; inspect ${evidence}.`);
}

/**
 Take a repeated option and its value out of a command line.

 @param options - command-line arguments
 
 @param flag - the option, such as `--shard`
 
 @returns the value after the first occurrence, if any, and the arguments without that option and value
 
 @example
 ```ts
 takeOption({ options: ['--core', '--re', 'x'], flag: '--re' });
 ```
 */
function takeOption({
  options,
  flag,
}: {
  readonly options: readonly string[];
  readonly flag: string;
}): {
  readonly value?: string;
  readonly present: boolean;
  readonly rest: string[];
} {
  /**
   Position of the option.
   */
  const at = options.indexOf(flag);
  if (at === (-1))
    return {
      present: false,
      rest: [...options],
    };
  /**
   Value after the option.
   */
  const value = options[at + 1];
  return {
    ...(value === undefined ? {} : { value }),
    present: true,
    rest: options.filter(function outside(
      _,
      index
    ) {
      return (index !== at) && (index !== (at
        + 1));
    }),
  };
}

/**
 Take `--shard <k>/<n>` out of a command line.

 @param options - command-line arguments
 
 @returns the shard text when the option is present, and the other arguments
 
 @throws CargoMutantsToolingError when the value is not `k/n` with k from 0 to n - 1
 
 @example
 ```ts
 takeShard(['--core', '--shard', '1/3']);
 ```
 */
export function takeShard(options: readonly string[]): {
  readonly shard?: string;
  readonly rest: string[];
} {
  /**
   The option, its value, and the arguments around it.
   */
  const taken = takeOption({
    options,
    flag: '--shard',
  });
  if (!taken.present)
    return { rest: taken.rest };
  /**
   The numbers on either side of the separator.
   */
  const parts = (taken.value ?? '').split(shardSeparator);
  if ((parts.length !== 2) || (!parts.every(function isNumber(part) {
    return usesOnly({
      text: part,
      allowed: decimalDigits,
    });
  }))
    || (Number(parts[0]) >= Number(parts[1])))
    throw new CargoMutantsToolingError(`--shard needs k/n with k from 0 to n - 1, not ${taken.value}.`);
  return {
    shard: taken.value ?? '',
    rest: taken.rest,
  };
}

/**
 Take `--re <regex>` out of a command line.

 @param options - command-line arguments
 
 @returns the mutant-name regex when the option is present, and the other arguments
 
 @throws CargoMutantsToolingError when the option has no regex or an empty one
 
 @example
 ```ts
 takeMutantRegex(['--core', '--re', 'Foo::bar']);
 ```
 */
export function takeMutantRegex(options: readonly string[]): {
  readonly examine?: string;
  readonly rest: string[];
} {
  /**
   The option, its value, and the arguments around it.
   */
  const taken = takeOption({
    options,
    flag: '--re',
  });
  if (!taken.present)
    return { rest: taken.rest };
  if ((taken.value === undefined) || (taken.value
    .length
    === 0))
    throw new CargoMutantsToolingError('--re needs a mutant-name regex.');
  return {
    examine: taken.value,
    rest: taken.rest,
  };
}
