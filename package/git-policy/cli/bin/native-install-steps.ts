/**
 What an install of one wrapper release runs, and the texts it reads back afterwards.

 These are the parts of `native:install` that decide nothing about a command line: the plan of steps, the words that carry
 a step's summary, the release text an installed wrapper prints, the newest line of the wrapper's commit timing record,
 and the text `journalctl --since` reads. Keeping them here lets the command file stay about its own command line.
 */
import { readFile } from 'node:fs/promises';

import {
  taskRunnerProgram,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/constants.ts';

import { NativeGateError } from './native-gate-error.ts';
import {
  gitDeadlineMilliseconds,
  gitProgram,
  promptlessGitSettings,
} from './native-gate-paths.ts';
import { runGateCommandOrThrow } from './native-gate-run.ts';

/**
 The tool the repository's toolchain lock records the wrapper as, which is the entry an install moves.
 */
export const wrapperToolName = 'cargo:monochromatic-git-policy-cli';

/**
 The lockfile an install writes, relative to the checkout.
 */
export const lockFileName = 'toolchain.lock.json';

/**
 The timing record the wrapper appends one JSON line per commit to, beside the common Git directory.
 */
export const commitTimingFileName = 'monochromatic-git-policy-cli-commit-timing.jsonl';

/**
 The journal identifiers one sample commit is read back from: this release's own program name, and the name the service
 carried in earlier releases, so a sample commit that an older service answered is still found.
 */
export const journalIdentifiers: readonly string[] = [
  'monochromatic-git-policy-cli',
  'cli-git-service',
];

/**
 How many lines of a step's output the install prints, which is the summary the scratch script printed rather than the
 whole of a download log.
 */
export const summaryLineLimit = 40;

/**
 One step of an install, named by the program and its arguments so a refusal says what it was doing.
 */
export type InstallStep = {
  /**
   The program to run.
   */
  readonly program: string;
  /**
   Its arguments, each kept whole.
   */
  readonly args: readonly string[];
  /**
   The checkout it runs in.
   */
  readonly repository: string;
  /**
   The words its summary lines carry.
   */
  readonly markers: readonly string[];
};

/**
 The text an installed wrapper's `git --version` line carries for one release.

 The wrapper prints `monochromatic-git-policy-cli <version>, shadowing git at <path>` after Git's own first line
 (`README.md`, "Version and help"), so the release asked for is named by this text and by nothing else.

 @param version - the release an install moves to

 @returns the text the installed wrapper has to print

 @example
 ```ts
 wrapperVersionLine({ version: '0.6.1-alpha.10' });
 ```
 */
export function wrapperVersionLine({ version }: { readonly version: string; }): string {
  return `monochromatic-git-policy-cli ${version},`;
}

/**
 The words every root mode is reached through, spelled after the wrapper's own word.

 The release holds one executable, and its ZFS route's root modes are modes of it
 (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`); the words are choice 2's
 (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`), and the module that decides a
 start reads the same two constants, so an install asks the build with the spelling the dispatch accepts.

 @example
 ```ts
 rootModeWords.join(' '); // 'monochromatic-git-policy-cli zfs-root'
 ```
 */
export const rootModeWords = [
  'monochromatic-git-policy-cli',
  'zfs-root',
] as const;

/**
 The first release that answered the root modes' words.

 A build of a release before it cannot answer `<the words> --version` at all, so an install of one checks what that
 release can answer and skips the words check: the wrapper is installed, the lock is moved and committed under that
 release's name, and the line that release prints is printed. From this release on the words check runs, and a build
 that answers nothing has lost the dispatch, or the wrapper on `PATH` is another release
 (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`, the release that carried the change,
 and `doc/decision/cli-git-alpha-46-release-pass-four-answers.md`, "Rolling back with `native:install`").
 The release list beside it says which releases those are, and `bin/native-install.unit.test.ts` fails when the two
 disagree.

 @example
 ```ts
 rootModesFirstRelease; // '0.6.1-alpha.46'
 ```
 */
export const rootModesFirstRelease = '0.6.1-alpha.46';

/**
 The releases whose builds answer the root modes' words, oldest first: the release list the floor above is kept beside.

 A release that answered the words is added here when it is released, and {@link rootModesFirstRelease} is the oldest
 entry, which is what makes one fact of two statements checkable instead of remembered. The floor, not this list,
 decides the skip: a release no list can name yet is still older than the floor or newer than it, and the install
 answers it the same way.

 @example
 ```ts
 rootModesReleases[0]; // '0.6.1-alpha.46', which is rootModesFirstRelease
 ```
 */
export const rootModesReleases: readonly string[] = ['0.6.1-alpha.46'];

/**
 The text the installed build's `--version` prints for one release.

 The root modes print `<the words> <version>` (`worktree_clone_mount_handler::handler_install_paths::version_line`), so
 the release asked for is named by this text and by nothing else.

 @param version - the release an install moves to

 @returns the text the installed build has to print

 @example
 ```ts
 rootModesVersionLine({ version: '0.6.1-alpha.10' }); // 'monochromatic-git-policy-cli zfs-root 0.6.1-alpha.10'
 ```
 */
export function rootModesVersionLine({ version }: { readonly version: string; }): string {
  return `${rootModeWords.join(' ')} ${version}`;
}

/**
 The number of numeric components a release's core has, before any prerelease.
 */
const releaseCoreLength = 3;

/**
 One release split into the parts semantic version precedence is decided on.
 */
type ParsedRelease = {
  /**
   Major, minor and patch, in that order.
   */
  readonly core: readonly [number, number, number];
  /**
   The dot-separated identifiers after the first hyphen, empty for a release.
   */
  readonly prerelease: readonly string[];
};

/**
 Splits one release of this wrapper into its core and its prerelease identifiers.

 @param version - the release text, such as `0.6.1-alpha.46`

 @returns the parts the comparison reads

 @throws NativeGateError when the text is not a release of this shape

 @example
 ```ts
 parseRelease({ version: '0.6.1-alpha.46+build' }); // { core: [0, 6, 1], prerelease: ['alpha', '46'] }
 ```
 */
function parseRelease({ version }: { readonly version: string; }): ParsedRelease {
  /**
   The text before any build metadata, which never takes part in precedence.
   */
  const withoutBuild = version.split('+',)[0] ?? '';
  /**
   Where the prerelease begins, or the end of the text when the release has none.
   */
  const hyphenAt = withoutBuild.indexOf('-',);
  /**
   The core's text, which is everything before the prerelease.
   */
  const coreText = hyphenAt === (-1) ? withoutBuild : withoutBuild.slice(
    0,
    hyphenAt,
  );
  /**
   The core's parts, which each have to be a plain count.
   */
  const parts = coreText.split('.',);
  if (
    (parts.length !== releaseCoreLength)
      || (!parts.every(function isCount(part) {
        return /^\d+$/.test(part,);
      },))
  )
    throw new NativeGateError(notARelease({ version, }));
  const [major, minor, patch] = parts.map(Number,);
  if ((major === undefined) || (minor === undefined) || (patch === undefined))
    throw new NativeGateError(notARelease({ version, }));
  /**
   The prerelease identifiers, which each have to be a word or a count.
   */
  const prerelease = hyphenAt === (-1) ? [] : withoutBuild.slice(hyphenAt + 1,)
    .split('.',);
  if (prerelease.some(function isIdentifier(identifier) {
    return identifier === '';
  },))
    throw new NativeGateError(notARelease({ version, }));
  return {
    core: [major, minor, patch],
    prerelease,
  };
}

/**
 The refusal of a version this install cannot order.

 @param version - the text the command line gave

 @returns the sentence, which names what was typed and the shape a release has

 @example
 ```ts
 notARelease({ version: 'latest' }); // '"latest" is not a release this install can order: give the release as major.minor.patch'
 ```
 */
function notARelease({ version }: { readonly version: string; }): string {
  return `${JSON.stringify(version,)} is not a release this install can order: give the release as major.minor.patch, such as 0.6.1 or 0.6.1-alpha.46, so an install can tell one release from another.`;
}

/**
 Compares two prerelease identifiers by the order semantic versions give them.

 @param left - an identifier of one release

 @param right - the identifier of the other release at the same place

 @returns negative, zero or positive like `Array#sort`'s comparator

 @example
 ```ts
 compareIdentifiers({ left: '9', right: '10' }); // negative
 ```
 */
function compareIdentifiers({
  left,
  right,
}: {
  readonly left: string;
  readonly right: string;
},): number {
  /**
   Whether each identifier is all digits, which is what makes it compare as a number.
   */
  const leftIsCount = /^\d+$/.test(left,);
  const rightIsCount = /^\d+$/.test(right,);
  if (leftIsCount && rightIsCount)
    return Number(left,) - Number(right,);
  if (leftIsCount)
    return -1;
  if (rightIsCount)
    return 1;
  if (left === right)
    return 0;
  return left < right ? -1 : 1;
}

/**
 Compares two releases of this wrapper by the precedence semantic versions have, which is the order a rollback moves in.

 A numeric component is compared as a number, and the prerelease identifiers only when the cores are equal: a release
 without a prerelease is above one with it, a numeric identifier is below an alphanumeric one and above a smaller number,
 and the release with more identifiers is above the one whose identifiers it starts with.

 @param left - the release under test

 @param right - the release it is compared with

 @returns negative when `left` comes before `right`, zero when the two are the same release, positive otherwise

 @throws NativeGateError when either text is not a release this install can order

 @example
 ```ts
 compareReleases({ left: '0.6.1-alpha.9', right: '0.6.1-alpha.10' }); // negative
 ```
 */
export function compareReleases({
  left,
  right,
}: {
  readonly left: string;
  readonly right: string;
},): number {
  /**
   The two releases, parsed once each.
   */
  const one = parseRelease({ version: left, });
  const other = parseRelease({ version: right, });
  for (let index = 0; index < releaseCoreLength; index += 1) {
    /**
     The difference of the two numeric components, which decides the answer when it is nonzero.
     */
    const difference = (one.core[index] ?? 0) - (other.core[index] ?? 0);
    if (difference !== 0)
      return difference;
  }
  // A release without a prerelease is above every prerelease of the same core.
  if (one.prerelease.length === 0)
    return other.prerelease.length === 0 ? 0 : 1;
  if (other.prerelease.length === 0)
    return -1;
  /**
   How many identifiers the two share, which is as far as they are compared pairwise.
   */
  const shared = Math.min(
    one.prerelease.length,
    other.prerelease.length,
  );
  for (let index = 0; index < shared; index += 1) {
    /**
     The difference of the two identifiers at this place, which decides the answer when it is nonzero.
     */
    const difference = compareIdentifiers({
      left: one.prerelease[index] ?? '',
      right: other.prerelease[index] ?? '',
    },);
    if (difference !== 0)
      return difference;
  }
  // The identifiers compared equal as far as both go, so the one with more identifiers is above.
  return one.prerelease.length - other.prerelease.length;
}

/**
 Whether the build of one release can be asked the root modes' words at all.

 The question is one comparison against {@link rootModesFirstRelease}: every release from it answers the words and no
 release before it can, so an install of one of those keeps the checks that release can answer and skips this one.
 A version this repository's releases never take is refused here, before the first step runs, rather than ordered by a
 guess.

 @param version - the release an install moves to

 @returns whether the install asks that build the words

 @throws NativeGateError when the version is not a release this install can order

 @example
 ```ts
 releaseAnswersTheRootModes({ version: '0.6.1-alpha.45' }); // false
 ```
 */
export function releaseAnswersTheRootModes({ version }: { readonly version: string; }): boolean {
  return compareReleases({
    left: version,
    right: rootModesFirstRelease,
  },) >= 0;
}

/**
 The lines of one step's output that carry its summary: the lines the repository's own tool prints when it moved or
 wrote something, or when it installed a tool.

 @param output - everything the step printed, on both streams

 @param markers - the words one of those lines holds

 @returns those lines, at most {@link summaryLineLimit} of them

 @example
 ```ts
 summaryLinesOf({ output: 'moved cargo:a 1.0.0 -> 1.1.0\n', markers: ['moved', 'wrote'] });
 ```
 */
export function summaryLinesOf({
  output,
  markers,
}: {
  readonly output: string;
  readonly markers: readonly string[];
}): readonly string[] {
  return output.split('\n')
    .filter(function carries(line) {
      return markers.some(function holdsWord(word) {
        return line.includes(word);
      });
    })
    .slice(
    0,
    summaryLineLimit
    );
}

/**
 The last line of one commit timing record.

 @param text - the record's text, which may be empty

 @returns the record's newest line and how many lines it holds, both empty and zero when it holds nothing

 @example
 ```ts
 newestTimingLine({ text: '{"step":"commit"}\n' });
 ```
 */
export function newestTimingLine({ text }: { readonly text: string; }): {
  readonly line: string;
  readonly count: number;
} {
  /**
   The record's lines, without the empty text a trailing newline leaves behind.
   */
  const lines = text.split('\n')
    .filter(function hasText(line) {
      return line.trim() !== '';
    });
  return {
    line: lines.at(-1)
      ?? '',
    count: lines.length,
  };
}

/**
 The line an install prints about the wrapper's own timing record of the sample commit.

 @param timing - the record's newest line and how many lines it holds

 @returns the report, which says so when the wrapper left no record at all

 @example
 ```ts
 timingReportLine({ timing: { line: '{"step":"commit"}', count: 1 } });
 ```
 */
export function timingReportLine({ timing }: { readonly timing: {
  readonly line: string;
  readonly count: number
}; }): string {
  return timing.count === 0
    ? `== ${commitTimingFileName} holds no line yet, so the commit was answered without a record`
    : `== newest timing line (${String(timing.count)} lines in all)\n${timing.line}`;
}

/**
 One moment as the text `journalctl --since` reads, in UTC with a second precision.

 @param moment - the moment to write

 @returns the stamp, such as `2026-11-11 18:37:02 UTC`

 @example
 ```ts
 journalSince({ moment: new Date('2026-11-11T18:37:02Z') });
 ```
 */
export function journalSince({ moment }: { readonly moment: Date; }): string {
  // `toISOString` writes `2026-11-11T18:37:02.123Z`, and `journalctl` reads the same moment with a space and no fractions.
  return `${moment.toISOString()
    .slice(
      0,
      '2026-11-11T18:37:02'.length
    )
    .replace(
      'T',
      ' '
    )} UTC`;
}

/**
 The text one step is named by in a refusal and in the printed log.

 @param step - the step

 @returns the program and its arguments, as a person would have typed them

 @example
 ```ts
 stepNameOf({ step });
 ```
 */
export function stepNameOf({ step }: { readonly step: InstallStep; }): string {
  return `${step.program} ${step.args
    .join(' ')}`;
}

/**
 The plan an install follows, in the order it runs.

 @param repository - the checkout the install writes in

 @param version - the release to move to

 @returns the steps, each with the words its summary lines carry

 @example
 ```ts
 installSteps({ repository: '/repo', version: '0.6.1-alpha.10' });
 ```
 */
export function installSteps({
  repository,
  version,
}: {
  readonly repository: string;
  readonly version: string;
}): readonly [
  InstallStep,
  InstallStep,
  InstallStep
] {
  return [
    {
      program: taskRunnerProgram,
      args: [
        'run',
        'toolchain:update',
        '--',
        wrapperToolName,
      ],
      repository,
      markers: [
        'moved',
        'wrote',
      ],
    },
    {
      program: taskRunnerProgram,
      args: [
        'run',
        'toolchain:install',
      ],
      repository,
      markers: ['installed',],
    },
    {
      program: gitProgram,
      args: [
        'commit',
        `--message=chore(toolchain): ${wrapperToolName.slice('cargo:'.length)} ${version}`,
        '--',
        lockFileName,
      ],
      repository,
      markers: [],
    },
  ];
}

/**
 Reads one file of the checkout, answering empty text when it is not there.

 @param path - the file to read

 @returns its text, or empty text when it is missing

 @throws NativeGateError when the file is there and cannot be read

 @example
 ```ts
 await readOrEmpty({ path: '/repo/toolchain.lock.json' });
 ```
 */
export async function readOrEmpty({ path }: { readonly path: string; }): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8'
    );
  }
  catch (error) {
    // A timing record the wrapper has not written yet is an answer; anything else is a file this run could not read.
    if (((typeof error) === 'object') && (error !== null)
      && (Reflect.get(
        error,
        'code',
      ) === 'ENOENT'))
      return '';
    throw new NativeGateError(
      `the file ${path} is there and could not be read: ${String(error)}`,
      { cause: error, }
    );
  }
}

/**
 The common Git directory of one checkout, which holds the timing record and is the directory a worktree shares with the
 checkout it belongs to.

 @param repository - the checkout to ask

 @returns the absolute path of the common Git directory

 @throws NativeGateError when git cannot name it

 @example
 ```ts
 await gitCommonDirectory({ repository: '/repo' });
 ```
 */
export async function gitCommonDirectory({ repository }: { readonly repository: string; }): Promise<string> {
  return (await runGateCommandOrThrow({
    program: gitProgram,
    args: [
      '-C',
      repository,
      'rev-parse',
      '--path-format=absolute',
      '--git-common-dir',
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: {
      ...process.env,
      ...promptlessGitSettings,
    },
  })).stdout
    .trim();
}
