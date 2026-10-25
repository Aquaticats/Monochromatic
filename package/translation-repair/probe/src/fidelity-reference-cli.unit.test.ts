/**
 Tests for the compiled `judge-fidelity-probe` command's zero-call preflight:
 which requests it refuses before any corpus or provider access, with which
 exit status and stderr, and what it prints when it accepts one. A request the
 reviewed references refuse is the operator's to mend, so it is a stated
 refusal at exit six with its one line, as the command's other refusals of
 what was typed are.

 Each run is the built command in an owned empty directory with no provider
 key set, `--cap 0`, so no case can read a corpus or spend. The entries the
 cases filter by are read from the checked-in manifest, never named here.
 Fixtures are cat-themed invention.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { REVIEWED_FIDELITY_REFERENCES, } from '../dist/final/node/index.mjs';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './corpus-run/built-entry.test-fixture.ts';
import {
  refusalMessage,
  withoutStamp,
} from './fidelity-reference.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_UNMEASURED, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Preflight runs
// The built command, run for its zero-call preflight only.

/**
 Exit status the command gives a refusal it states in its own words.
 */
const REFUSED_AS_STATED = 6;

/**
 What a preflight run left behind.
 */
type PreflightRun = {
  /**
   Exit status, zero for an accepted request.
   */
  readonly code: number;

  /**
   Standard output, one entry per line.
   */
  readonly stdout: readonly string[];

  /**
   Standard error, one entry per line.
   */
  readonly stderr: readonly string[];
};

/**
 Splits command output into lines, each log line's timestamp dropped so a
 case compares the line whole.

 @param text - output read

 @returns Its non-empty lines

 @example
 ```ts
 const lines = linesOf({ text: '[info] [2026-01-01T00:00:00.000Z] [probe] purr\n', },); // ['[info] [probe] purr']
 ```
 */
function linesOf({ text, }: { readonly text: string; },): readonly string[] {
  return text.split('\n',)
    .filter((line,) => line !== '',)
    .map((line,) => withoutStamp({ line, },),);
}

/**
 Runs only the zero-call preflight in an owned empty directory.

 @param extra - arguments after `--cap 0` and the one approved candidate

 @returns Exit status and both output streams

 @example
 ```ts
 const run = await preflight(['--context',],);
 ```
 */
async function preflight(extra: readonly string[],): Promise<PreflightRun> {
  await using owned = await scratchDir({ prefix: 'reviewed-fidelity-cli-', },);
  /**
   The directory every path setting points at, so nothing outside it is read or written.
   */
  const directory = owned.path;
  /**
   What the built command did, whatever its exit code; the fixture removes
   every provider key from its environment, and the blanks here keep the
   command's own settings empty rather than absent.
   */
  const run = await runBuiltCommand({
    command: 'judge-fidelity-probe',
    packageEntry: PROBE_ENTRY,
    args: [
      '--cap',
      '0',
      '--candidates',
      SEAT_HYPER_OPENROUTER_UNMEASURED,
      '--candidates-alone',
      ...extra,
    ],
    cwd: directory,
    env: {
      TRANSLATION_REPAIR_RUNS_DIR: directory,
      TRANSLATION_REPAIR_CORPUS_CLONE_DIR: directory,
      TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'fixture-only',
      TRANSLATION_REPAIR_CHARM_HYPER_API_KEY: '',
      TRANSLATION_REPAIR_OPENROUTER_API_KEY: '',
      TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: '',
      TRANSLATION_REPAIR_NEURALWATT_API_KEY: '',
    },
  },);
  return {
    code: run.code,
    stdout: linesOf({ text: run.stdout, },),
    stderr: linesOf({ text: run.stderr, },),
  };
}

/**
 The stderr line of a refused request: the command's name and the refusal's whole message.

 @param referenceId - identifier the refusal names

 @returns The line

 @example
 ```ts
 expect(run.stderr,).toEqual([refusedLine({ referenceId: 'unreviewed context', },),],);
 ```
 */
function refusedLine({ referenceId, }: { readonly referenceId: string; },): string {
  return `judge-fidelity-probe: ${refusalMessage({
    referenceId,
    operation: 'request',
  },)}`;
}

/**
 Runs the built command with the arguments given, in an owned empty directory that is also the corpus clone.

 @param args - every argument after the script path

 @returns Exit status and both output streams, one entry per line

 @example
 ```ts
 const run = await runAsBuilt({ args: ['--cap', 'kitten',], },);
 ```
 */
async function runAsBuilt({ args, }: { readonly args: readonly string[]; },): Promise<PreflightRun> {
  await using owned = await scratchDir({ prefix: 'reviewed-fidelity-as-built-', },);
  /**
   What the built command did, whatever its exit code; the fixture removes
   every provider key from its environment, so this child carries none.
   */
  const run = await runBuiltCommand({
    command: 'judge-fidelity-probe',
    packageEntry: PROBE_ENTRY,
    args,
    cwd: owned.path,
    env: {
      TRANSLATION_REPAIR_RUNS_DIR: owned.path,
      TRANSLATION_REPAIR_CORPUS_CLONE_DIR: owned.path,
    },
  },);
  return {
    code: run.code,
    stdout: linesOf({ text: run.stdout, },),
    stderr: linesOf({ text: run.stderr, },),
  };
}

/**
 The two lines an accepted preflight prints.

 @param selected - reviewed references the request selects

 @returns The judges line and the preflight line

 @example
 ```ts
 expect(run.stdout,).toEqual(acceptedLines({ selected: 3, },),);
 ```
 */
function acceptedLines({ selected, }: { readonly selected: number; },): readonly string[] {
  return [
    `[info] [judge-fidelity-probe] judges: ${SEAT_HYPER_OPENROUTER_UNMEASURED}`,
    `[info] [judge-fidelity-probe] preflight only: ${
      String(selected,)
    } reviewed reference ${(selected === 1) ? 'specification' : 'specifications'} selected; no corpus or model calls`,
  ];
}

/**
 The checked-in entry that has no reviewed alteration, which a request for the default families cannot select alone.

 @returns Its entry identifier and the number of references the manifest holds for it

 @throws {@link Error} when every reviewed entry has an alteration, since the case would then pin nothing

 @example
 ```ts
 const { entryId, } = entryWithoutAlteration();
 ```
 */
function entryWithoutAlteration(): { readonly entryId: string; readonly references: number; } {
  /**
   First reference with no alteration variant.
   */
  const found = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES.find(
    (spec,) => !spec.damages.some((damage,) => damage.kind === 'alteration',),
  ),);
  return {
    entryId: found.entryId,
    references: REVIEWED_FIDELITY_REFERENCES.filter((spec,) => spec.entryId === found.entryId,).length,
  };
}

//endregion Preflight runs

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'judge-fidelity-probe preflight',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES AS STATED an entry with no reviewed reference, naming it on its one stderr line, with exit '
            + 'status six and nothing on stdout',
          fn: async () => {
            const run = await preflight([
              '--only',
              'not-reviewed-fixture',
            ],);
            expect(run,).toEqual({
              code: REFUSED_AS_STATED,
              stdout: [],
              stderr: [refusedLine({ referenceId: 'not-reviewed-fixture', },),],
            },);
          },
        },),
        it({
          name: 'REFUSES AS STATED unreviewed context, naming it on its one stderr line with exit status six, rather '
            + 'than changing what a gold comparison means',
          fn: async () => {
            const run = await preflight(['--context',],);
            expect(run,).toEqual({
              code: REFUSED_AS_STATED,
              stdout: [],
              stderr: [refusedLine({ referenceId: 'unreviewed context', },),],
            },);
          },
        },),
        it({
          name: 'REFUSES AS STATED the default families for an entry with no reviewed alteration, naming the missing '
            + 'family on its one stderr line with exit status six, rather than silently dropping it',
          fn: async () => {
            const { entryId, } = entryWithoutAlteration();
            const run = await preflight([
              '--only',
              entryId,
            ],);
            expect(run,).toEqual({
              code: REFUSED_AS_STATED,
              stdout: [],
              stderr: [refusedLine({ referenceId: 'damage selection (alteration)', },),],
            },);
          },
        },),
        it({
          name: 'ACCEPTS an explicitly narrower family for that entry, printing the judge and its reviewed '
            + 'references selected, with no spend line',
          fn: async () => {
            const { entryId, references, } = entryWithoutAlteration();
            const run = await preflight([
              '--only',
              entryId,
              '--damage',
              'deletion',
            ],);
            expect(run.code,).toBe(0,);
            expect(run.stderr,).toEqual([],);
            expect(run.stdout,).toEqual(acceptedLines({ selected: references, },),);
          },
        },),
        it({
          name: 'ACCEPTS the whole checked-in manifest for the default families, which some reference reviews each of, '
            + 'printing the judge and every reference selected, with no spend line',
          fn: async () => {
            const run = await preflight([],);
            expect(run.code,).toBe(0,);
            expect(run.stderr,).toEqual([],);
            expect(run.stdout,).toEqual(acceptedLines({ selected: REVIEWED_FIDELITY_REFERENCES.length, },),);
          },
        },),
      ],
    },),
    describe({
      name: 'judge-fidelity-probe as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A CAP THAT IS NOT A WHOLE NUMBER as a stated refusal with exit status six, naming what was '
            + 'typed, and prints nothing on stdout',
          fn: async () => {
            const run = await runAsBuilt({
              args: [
                '--cap',
                'kitten',
              ],
            },);
            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toEqual([],);
            expect(run.stderr,).toEqual([
              'judge-fidelity-probe: --cap needs a whole number written in digits, at most 9007199254740991, '
                + 'and "kitten" is not one',
            ],);
          },
        },),
        it({
          name: 'STOPS AT A CLONE DIRECTORY THAT IS NO REPOSITORY after naming its judge, with the reference it '
            + 'could not read and no row written, since git cannot open it',
          fn: async () => {
            const run = await runAsBuilt({
              args: [
                '--cap',
                '1',
                '--candidates',
                SEAT_HYPER_OPENROUTER_UNMEASURED,
                '--candidates-alone',
              ],
            },);
            /**
             First reviewed reference, the one the read starts with.
             */
            const first = nonNullishOrThrow(REVIEWED_FIDELITY_REFERENCES[0],);
            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toEqual([`[info] [judge-fidelity-probe] judges: ${SEAT_HYPER_OPENROUTER_UNMEASURED}`,],);
            expect(run.stderr,).toEqual([
              `judge-fidelity-probe: corpus read failed for ${first.corpusSha}:people/${first.entryId}/page.md `
                + '(unreadable-clone); git could not open the clone: check that the directory exists, is the top of '
                + 'a git repository, and is one git may read.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
