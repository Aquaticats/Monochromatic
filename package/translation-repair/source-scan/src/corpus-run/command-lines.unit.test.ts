/**
 Tests the declaration of what every runner reads from its command line
 (ledger B75): that it declares every runner the build makes and nothing
 else, that each declaration can be read as written, that each runner's
 mise task description offers exactly the flags its runner reads, and that
 the living docs invoke each runner only with flags it reads.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isAsciiLowerLetter, } from '@monochromatic-dev/translation-repair-markup';
import {
  COMMAND_LINES,
  COMMAND_OWNERS,
} from '@monochromatic-dev/translation-repair-run';
import {
  readLivingRepositoryDocs,
  readPackageDocPaths,
  REPOSITORY_ROOT,
} from '../living-docs.test-fixture.ts';
import {
  readPackageSource,
  readTaskFiles,
  runnerEntriesOf,
} from '../family-source.test-fixture.ts';

/**
 The package directory that owns each runner, by runner name, which names the task a doc starts it by.
 */
const RUNNER_OWNERS: ReadonlyMap<string, string> = new Map(runnerEntriesOf({ files: await readPackageSource(), },)
  .map(function ownerOf({ name, directory, },): readonly [string, string,] {
    return [name, directory,];
  },),);

/**
 Start of a task's run line naming a built runner.
 */
const RUN_PREFIX = 'run = "node dist/final/node/';

/**
 End of that run line.
 */
const RUN_SUFFIX = '.mjs"';

/**
 Start of a task's description line.
 */
const DESCRIPTION_PREFIX = 'description = ';

/**
 Start of a table header line in the task file.
 */
const HEADER_PREFIX = '[';

/**
 What starts a long flag.
 */
const LONG_PREFIX = '--';

/**
 What a long flag's word may carry between its letters.
 */
const WORD_DASH = '-';

/**
 Whether a character can continue a long flag's word once its first letter
 is written: a lowercase ASCII letter or a dash.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for `a` to `z` and `-` only

 @example
 ```ts
 continuesFlagWord({ character: '-', },); // true
 ```
 */
function continuesFlagWord({ character, }: { readonly character: string; },): boolean {
  return (character === WORD_DASH) || isAsciiLowerLetter({ character, },);
}

/**
 Whether a declared flag name is a word a long flag can carry: a lowercase
 ASCII letter, then lowercase letters and dashes.

 @param name - flag name as a declaration writes it, without its dashes

 @returns True when `--` and the name spell a flag a description can name

 @example
 ```ts
 isFlagWord({ name: 'require-providers', },); // true
 isFlagWord({ name: 'Only', },); // false
 ```
 */
function isFlagWord({ name, }: { readonly name: string; },): boolean {
  return isAsciiLowerLetter({ character: name.charAt(0,), },)
    && Array.from(name.slice(1,),).every(function continues(character,): boolean {
      return continuesFlagWord({ character, },);
    },);
}

/**
 Whether a long flag starts at a position: two dashes, then a lowercase
 ASCII letter. A bare `--` followed by a space starts none.

 @param text - text a flag may start in
 @param at - UTF-16 offset of the first dash

 @returns True when the dashes and a first letter stand at the offset

 @example
 ```ts
 startsFlag({ text: 'pass -- --only', at: 5, },); // false
 startsFlag({ text: 'pass -- --only', at: 8, },); // true
 ```
 */
function startsFlag({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number;
},): boolean {
  return text.startsWith(
    LONG_PREFIX,
    at,
  )
    && isAsciiLowerLetter({ character: text.charAt(at + LONG_PREFIX.length,), },);
}

/**
 Every long flag a text spells, in order: two dashes, then a lowercase word
 that may carry inner dashes. A bare `--`, which descriptions and docs write
 before the arguments mise passes on, has no word and is not one.

 One pass from the start, so the time is linear in the text.

 @param text - task description, or what a doc types after a runner

 @returns Flags, dashes included, as often as the text spells them

 @example
 ```ts
 flagsSpelled({ text: 'probe (pass -- --only a,b)', },); // ['--only']
 ```
 */
function flagsSpelled({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Flags found so far.
   */
  const flags: string[] = [];

  /**
   Where the scan stands.
   */
  let at = 0;
  while (at < text.length) {
    if (!startsFlag({
      text,
      at,
    },)) {
      at += 1;
      continue;
    }

    /**
     Just past the flag's last character.
     */
    let end = at + LONG_PREFIX.length + 1;
    while (continuesFlagWord({ character: text.charAt(end,), },))
      end += 1;
    flags.push(text.slice(
      at,
      end,
    ),);
    at = end;
  }
  return flags;
}

/**
 Each runner's task description in the task file, by runner name.

 READ LINE BY LINE, NOT AS TOML, since the package has no TOML parser and
 every task here writes its description and run on lines of their own; a
 runner whose task this cannot find is missing from the answer, which the
 case holds against the runner list, so a change of layout fails loudly.

 @param text - family's task files, joined

 @returns Description of the task that runs each runner, by runner name

 @example
 ```ts
 const descriptions = taskDescriptions({ text: (await readTaskFiles()).map(({ text, },) => text).join('\n',), },);
 ```
 */
function taskDescriptions({ text, }: { readonly text: string; },): ReadonlyMap<string, string> {
  /**
   Descriptions found, by runner name.
   */
  const found = new Map<string, string>();

  /**
   Description of the task being read, empty until its line is reached.
   */
  let description = '';
  for (const line of text.split('\n',)) {
    if (line.startsWith(HEADER_PREFIX,))
      description = '';
    else if (line.startsWith(DESCRIPTION_PREFIX,)) {
      /**
       The description as TOML writes it, a quoted string JSON reads alike.
       */
      const parsed: unknown = JSON.parse(line.slice(DESCRIPTION_PREFIX.length,),);
      if ((typeof parsed) !== 'string')
        throw new Error(`a task description in mise.toml is not a quoted string: ${line}`,);
      description = parsed;
    } else if (line.startsWith(RUN_PREFIX,) && line.endsWith(RUN_SUFFIX,)) {
      found.set(
        line.slice(
          RUN_PREFIX.length,
          -RUN_SUFFIX.length,
        ),
        description,
      );
    }
  }
  return found;
}

/**
 How a doc spells an invocation of a runner, up to what is typed after it:
 the package task, the same task named from the repository root (under the
 directory the family was split out of, which docs written before 2026-10-18 still name,
 and under the family package that owns the runner), and the built file.

 @param runner - runner name

 @returns Spellings, each ending where the typed arguments start

 @example
 ```ts
 invocationsOf({ runner: 'corpus-pass', },)[0]; // 'mise run corpus-pass '
 ```
 */
function invocationsOf({ runner, }: { readonly runner: string; },): readonly string[] {
  /**
   Package directory that owns the runner, absent for a name no entry lists.
   */
  const owner = RUNNER_OWNERS.get(runner,);
  return [
    `mise run ${runner} `,
    `mise run //package/module/translation-repair:${runner} `,
    ...((owner === undefined) ? [] : [`mise run //package/translation-repair/${owner}:${runner} `,]),
    `node dist/final/node/${runner}.mjs`,
  ];
}

/**
 Every flag a doc's runner invocations write that the runner does not read.

 Each line is read for every runner's spellings; what follows one, up to the
 next backtick or the line's end, is what the doc types after the runner. An
 invocation broken across lines is read up to its first line's end.

 @param path - doc path, which names each finding

 @param text - doc text

 @returns Each flag as `path:line runner --flag`, and how many invocations
 were read, so an empty list is read against a scan that saw some

 @example
 ```ts
 const { undeclared, invocations, } = undeclaredFlagsIn({ path: 'doc/cat.md', text, },);
 ```
 */
function undeclaredFlagsIn(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): {
  readonly undeclared: readonly string[];
  readonly invocations: number;
} {
  /**
   Each invocation found: where, which runner, and what was typed after it.
   */
  const found = text.split('\n',).flatMap(function invocationsOnLine(line, index,): readonly {
    readonly where: string;
    readonly runner: string;
    readonly typed: string;
  }[] {
    return Object.keys(COMMAND_LINES,).flatMap(function spellingsOf(runner,): readonly {
      readonly where: string;
      readonly runner: string;
      readonly typed: string;
    }[] {
      return invocationsOf({ runner, },).flatMap(function occurrencesOf(spelling,): readonly {
        readonly where: string;
        readonly runner: string;
        readonly typed: string;
      }[] {
        return line
          .split(spelling,)
          .slice(1,)
          .map(function typedAfter(after,): {
            readonly where: string;
            readonly runner: string;
            readonly typed: string;
          } {
            /**
             Where the code span closes, absent when it runs to the line's end.
             */
            const closes = after.indexOf('`',);
            return {
              where: `${path}:${String(index + 1,)}`,
              runner,
              typed: (closes === (-1)) ? after : after.slice(
                0,
                closes,
              ),
            };
          },);
      },);
    },);
  },);
  return {
    undeclared: found.flatMap(function undeclaredOf({
      where,
      runner,
      typed,
    },): readonly string[] {
      /**
       What the runner reads.
       */
      const spec = COMMAND_LINES[runner as keyof typeof COMMAND_LINES];

      /**
       Its flags, dashes included.
       */
      const declared = new Set(
        [
          ...Object.keys(spec.valued,),
          ...Object.keys(spec.repeatable,),
          ...Object.keys(spec.switches,),
        ].map(function dashed(name,): string {
          return `${LONG_PREFIX}${name}`;
        },),
      );
      return flagsSpelled({ text: typed, },)
        .filter(function isUndeclared(flag,): boolean {
          return !declared.has(flag,);
        },)
        .map(function shown(flag,): string {
          return `${where} ${runner} ${flag}`;
        },);
    },),
    invocations: found.length,
  };
}

/**
 The docs the invocation case reads, from the repository root: the living
 repository-level docs and the family's own docs and README and those of
 every family package. The audit ledger is left out, since it records what was typed when it was typed.

 @returns Paths from the repository root

 @example
 ```ts
 const paths = await invokingDocs();
 ```
 */
async function invokingDocs(): Promise<readonly string[]> {
  /**
   Living repository-level docs.
   */
  const {
    decisionRecords,
    handover,
    currentPlanning,
    operations,
  } = await readLivingRepositoryDocs();
  return [
    ...decisionRecords,
    ...handover,
    ...currentPlanning,
    ...operations,
    ...(await readPackageDocPaths())
      .filter(function isLivingMarkdown(path,): boolean {
        // The ledger and its parts keep the command lines of their time as written.
        return !(path.split('/',).at(-1,) ?? '').startsWith('audit-ledger',);
      },),
  ];
}

/**
 A runner with its owner, as one line, so two lists of them compare by order of text.

 @param pair - runner's name, then the directory name of the package that owns it

 @returns Two joined by a space

 @example
 ```ts
 lineOf(['corpus-pass', 'corpus-pass',],); // 'corpus-pass corpus-pass'
 ```
 */
function lineOf([command, owner,]: readonly [string, string,],): string {
  return `${command} ${owner}`;
}

await describe({
  name: 'COMMAND_LINES',
  children: [
    it({
      name: 'DECLARES every runner the build makes and nothing else, so no runner reads a line undeclared',
      fn: async () => {
        expect(Object.keys(COMMAND_LINES,).toSorted(),).toEqual([...RUNNER_OWNERS.keys(),].toSorted(),);
      },
    },),
    it({
      name: 'OWNS each runner in the package whose build entries list it, so the task a refusal names is the task '
        + 'that starts the runner',
      fn: async () => {
        expect(Object.entries(COMMAND_OWNERS,).map(lineOf,).toSorted(),).toEqual([...RUNNER_OWNERS,].map(lineOf,)
          .toSorted(),);
      },
    },),
    it({
      name: 'DECLARES each flag once, under a name a long flag can carry, and positions that can be written',
      fn: async () => {
        /**
         What is wrong with each declaration, empty for none.
         */
        const problems = Object.entries(COMMAND_LINES,).flatMap(function problemsOf([command, spec,],): string[] {
          /**
           Every flag the declaration names, in all three groups.
           */
          const names = [
            ...Object.keys(spec.valued,),
            ...Object.keys(spec.repeatable,),
            ...Object.keys(spec.switches,),
          ];
          /**
           Names of the positions, how many must be written, and whether the last repeats.
           */
          const {
            names: positions,
            least,
            rest,
          } = spec.positionals;
          return [
            ...((new Set(names,).size === names.length) ? [] : [`${command} names a flag twice`,]),
            ...names
              .filter(function isMalformed(name,): boolean {
                return !isFlagWord({ name, },);
              },)
              .map(function malformed(name,): string {
                return `${command} names flag ${name}`;
              },),
            ...((least <= positions.length) ? [] : [`${command} requires more positions than it names`,]),
            ...((rest && (positions.length === 0)) ? [`${command} repeats a position it does not name`,] : []),
          ];
        },);
        expect(problems,).toEqual([],);
      },
    },),
    it({
      name: 'OFFERS in each runner\'s task description exactly the flags the runner reads, so a description '
        + 'cannot offer a flag the runner refuses or leave out one it reads',
      fn: async () => {
        /**
         Each runner's task description.
         */
        const descriptions = taskDescriptions({ text: (await readTaskFiles()).map(function textOf({ text, },): string {
          return text;
        },).join('\n',), },);
        expect([...descriptions.keys(),].toSorted(),).toEqual(Object.keys(COMMAND_LINES,).toSorted(),);

        /**
         The flags each description spells, by runner.
         */
        const offered = Object.fromEntries(Object.keys(COMMAND_LINES,).map(function offeredBy(command,): [
          string,
          readonly string[],
        ] {
          return [
            command,
            [...new Set(flagsSpelled({ text: descriptions.get(command,) ?? '', },),),].toSorted(),
          ];
        },),);

        /**
         The flags each runner reads, by runner.
         */
        const read = Object.fromEntries(Object.entries(COMMAND_LINES,).map(function readBy([command, spec,],): [
          string,
          readonly string[],
        ] {
          return [
            command,
            [
              ...Object.keys(spec.valued,),
              ...Object.keys(spec.repeatable,),
              ...Object.keys(spec.switches,),
            ]
              .map(function dashed(name,): string {
                return `--${name}`;
              },)
              .toSorted(),
          ];
        },),);

        expect(offered,).toEqual(read,);
      },
    },),
    it({
      name: 'FINDS a flag a doc types after a runner that the runner does not read, in each spelling of an '
        + 'invocation, the equals form included, and leaves declared flags and a runner named bare',
      fn: async () => {
        expect(undeclaredFlagsIn({
          path: 'doc/cat.md',
          text: [
            'Run `mise run corpus-pass -- --olny Tabby_01 --plan` first.',
            'Or `mise run //package/module/translation-repair:coverage-probe -- --cap 2 --cpa=3`,',
            'or `mise run //package/translation-repair/coverage-census:coverage-probe -- --cap 2 --cpa=4`,',
            'then `node dist/final/node/verify-published.mjs --nap`, and `mise run corpus-pass` bare.',
          ].join('\n',),
        },),).toEqual({
          undeclared: [
            'doc/cat.md:1 corpus-pass --olny',
            'doc/cat.md:2 coverage-probe --cpa',
            'doc/cat.md:3 coverage-probe --cpa',
            'doc/cat.md:4 verify-published --nap',
          ],
          invocations: 4,
        },);
      },
    },),
    it({
      name: 'ENDS an invocation at a shell control operator or redirection outside quotes, so the flags of the '
        + 'command after it are not the runner\'s, ledger B1122',
      fn: async () => {
        /**
         What follows a runner and ends its flags: a pipe, an or, an and, a separator, a redirection of output
         or input, and a redirection of the error stream into the output.
         */
        const operators = ['|', '||', '&&', ';', '>', '2>&1', '<',] as const;

        // A flag the runner reads, then an operator, then a flag it does not: nothing is found.
        expect(undeclaredFlagsIn({
          path: 'doc/cat.md',
          text: operators.map(function afterRunner(operator,): string {
            return `    mise run corpus-pass -- --plan ${operator} rg --count 'Z] START '`;
          },).join('\n',),
        },),).toEqual({
          undeclared: [],
          invocations: operators.length,
        },);

        // A line continued into a pipe: the next line is not read as the runner's.
        expect(undeclaredFlagsIn({
          path: 'doc/cat.md',
          text: [
            '    mise run corpus-pass -- --plan \\',
            '      | rg --fixed-strings \'Z] START \'',
          ].join('\n',),
        },),).toEqual({
          undeclared: [],
          invocations: 1,
        },);

        // A flag the runner does not read before the operator is still found, and none after it.
        expect(undeclaredFlagsIn({
          path: 'doc/cat.md',
          text: operators.map(function beforeOperator(operator,): string {
            return `    mise run corpus-pass -- --olny Tabby_01 ${operator} rg --count 'Z] START '`;
          },).join('\n',),
        },),).toEqual({
          undeclared: operators.map(function found(_operator, index,): string {
            return `doc/cat.md:${String(index + 1,)} corpus-pass --olny`;
          },),
          invocations: operators.length,
        },);
      },
    },),
    it({
      name: 'READS PAST an operator inside quotes or escaped, which is an argument and not the end of the '
        + 'invocation, ledger B1122',
      fn: async () => {
        expect(undeclaredFlagsIn({
          path: 'doc/cat.md',
          text: [
            'mise run corpus-pass -- "a | b" --olny Tabby_01',
            'mise run corpus-pass -- \'a ; b > c\' --olny Tabby_01',
            'mise run corpus-pass -- "a \\" | b" --olny Tabby_01',
            'mise run corpus-pass -- a\\|b --olny Tabby_01',
            'mise run corpus-pass -- "a | b" | rg --count c',
          ].join('\n',),
        },),).toEqual({
          undeclared: [
            'doc/cat.md:1 corpus-pass --olny',
            'doc/cat.md:2 corpus-pass --olny',
            'doc/cat.md:3 corpus-pass --olny',
            'doc/cat.md:4 corpus-pass --olny',
          ],
          invocations: 5,
        },);
      },
    },),
    it({
      name: 'INVOKES each runner in the living docs only with flags it reads, so a doc cannot hand a reader a '
        + 'command the runner refuses',
      fn: async () => {
        /**
         Each doc's findings and invocation count.
         */
        const scanned = await Promise.all((await invokingDocs()).map(async function scanDoc(path,): Promise<{
          readonly undeclared: readonly string[];
          readonly invocations: number;
        }> {
          return undeclaredFlagsIn({
            path,
            text: await readFile(
              join(
                REPOSITORY_ROOT,
                path,
              ),
              'utf8',
            ),
          },);
        },),);

        // The docs invoke runners, so an empty list is not a scan that read
        // nothing.
        expect(scanned.some(function invokes({ invocations, },): boolean {
          return invocations > 0;
        },),).toBe(true,);
        expect(scanned.flatMap(function undeclaredOf({ undeclared, },): readonly string[] {
          return undeclared;
        },),).toEqual([],);
      },
    },),
  ],
},);
