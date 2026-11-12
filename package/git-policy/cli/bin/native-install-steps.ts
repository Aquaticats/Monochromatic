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
