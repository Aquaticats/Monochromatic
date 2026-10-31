/**
 Tests for reporting a refusal instead of crashing out of a CLI.

 THE UNEXPECTED-FAULT CASES ARE THE ONES THAT CONSTRAIN THE DESIGN, and they
 replaced a forwarding case that pinned the opposite contract. That case
 checked a foreign class came straight back out, on the reasoning that
 catching every `Error` destroys the stack of a genuine programming fault.

 Re-throwing was shown not to be neutral: it hands the decision to whatever
 prints next, and Node's reporter renders a cause chain, which is how a YAML
 refusal published a page's front matter. So everything is caught now, and the
 cases in this file hold both halves at once: the message must NOT be repeated, and
 the frames MUST still be there.

 THREE EXIT CODES NOW, AND EACH IS DEFINED AGAINST THE OTHER TWO. One change
 closed the message of every class that had not declared itself quote-free,
 and a later rule decided which of our own may speak. So what a reader gets depends
 on the class thrown: a stated refusal says its sentence and stops at 6, a
 marked class that is not one says its sentence AND keeps its frames at 5, and
 everything else is named without being quoted. The cases in this file hold one of
 each, so a later change that collapses the three into one report fails here
 rather than in an operator's terminal.

 BOTH SWAPS ARE DISPOSABLE. `process.exitCode` is process-wide, so a case that
 set it and walked away would decide the whole suite's exit code, and a suite
 reporting 680 passes while exiting 4 is worse than a failing test.

 THE SUITE RUNS AT `concurrency: 1` FOR THE SAME REASON, and it was written
 without that first. `describe` runs children concurrently by default, so the
 three cases raced on `console.error` and on `process.exitCode`: one case saw
 zero captured lines because a sibling's disposal had already put the real
 reporter back, and another read `undefined` where it had just written zero.
 Both swaps are process-wide, and there is exactly one process here: the
 runner spawns `node` once per test FILE, so nothing outside this file is
 touched, and nothing inside it may overlap.

 TWO KINDS OF CASE LIVE IN OTHER FILES, because the names they need belong to
 packages above the one that owns `reportingRefusals`, and a test file may
 import only from its own package and the packages that package depends on.
 The bench report's refusal is held in `bench-report-refusal.unit.test.ts`,
 and the coverage census's baseline reader and source map refusal are held in
 `coverage-census-refusals.unit.test.ts`. What the three files share (the two
 swaps, the arguments and environment, and the name and body of the case an
 operator's own mistake is reported by) is in
 `cli-refusal-cases.test-fixture.ts`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SheetBindingError, } from '@monochromatic-dev/translation-repair-lane-contest';
import { RunJsonUnreadableError, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  GradedSheetExistsError,
  StatedRefusalError,
  UnsafeSeedError,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  EmptyPoolError,
  MixedGenerationError,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  createSeatTally,
  RUN_SEATS,
} from '@monochromatic-dev/translation-repair-stage';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import { BedrockCreditOverrideError, } from '@monochromatic-dev/translation-repair-provider';
import {
  HardCapOverrideError,
  LedgerShapeError,
  registerRunEndReport,
  reportingRefusals,
  RunConfigError,
  RunsDirectoryBusyError,
  SpendCeilingOverrideError,
  framesOf,
  isUnexpectedFault,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  BARE_ARGV,
  caseName,
  collectingErrors,
  holdingExitCode,
  NO_KEYS,
  REFUSED_AS_STATED,
  reportsMistake,
} from './cli-refusal-cases.test-fixture.ts';
import { withFreshRunSeats, } from '@monochromatic-dev/translation-repair-stage/ts/corpus-run/fresh-run-seats.test-fixture.ts';

//region CLI refusal tests

/**
 Exit code a CLI leaves behind when a run file would not read.
 */
const COULD_NOT_READ = 4;

/**
 Lines a reporting run is expected to print.
 */
const REPORTED_LINES = 2;

/**
 Lines an unexpected fault is expected to print: name, explanation, frames.
 */
const FAULT_LINES = 3;

/**
 Index of the frames line among those three.
 */
const FRAMES_LINE = 2;

/**
 Exit code a CLI leaves behind when it broke for a reason nobody planned for.
 */
const UNEXPECTED_FAULT = 5;

/**
 Message the fault fixture carries, which must never reach a reader.

 Phrased as something a real error could say about content it was handed,
 because that is the shape this guard exists for.
 */
const FAULT_MESSAGE = 'a tabby walked across Pouncewick';

/**
 Byte offset the fixture refusal names.
 */
const FIXTURE_BYTE = 27;

/**
 Lines a stated refusal is expected to print: the sentence, and nothing after.
 */
const STATED_LINES = 1;

/**
 Message the stated fixture carries, which MUST reach a reader.

 Shaped as a usage line because that is what the marker exists for: the words
 an operator needs most are the ones saying what to type next.
 */
const STATED_MESSAGE = 'name at least one basket: sunbeam-report <path> [<path> ...]';

/**
 File the ledger fixture names, which its message may repeat.
 */
const LEDGER_FILE = 'ledger/000007.json';

/**
 Field the ledger fixture reports missing, which its message may repeat.
 */
const LEDGER_FIELD = 'ballots';

/**
 Builds the refusal these cases are reported about.

 @returns Refusal naming a file, a class and an offset

 @example
 ```ts
 throw fixtureRefusal();
 ```
 */
function fixtureRefusal(): RunJsonUnreadableError {
  return new RunJsonUnreadableError({
    file: 'run.json',
    failure: 'SyntaxError',
    at: FIXTURE_BYTE,
  },);
}

/**
 Variable a run configuration refusal names.
 */
const CONFIG_VARIABLE = 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY';

/**
 Message that refusal carries: the variable name and a fix, which is all
 that class may ever say.
 */
const CONFIG_MESSAGE = `${CONFIG_VARIABLE} is not set; run under mise so sops injects it`;

/**
 What a directory of settled entries holds across two built pipelines.
 */
const TWO_PIPELINES = {
  groups: [
    {
      digest: 'sha256:aaaaaaaaaaaaaaaa',
      entryIds: ['Mittens',],
    },
    {
      digest: 'sha256:bbbbbbbbbbbbbbbb',
      entryIds: ['Pepper',],
    },
  ],
  total: 2,
  tipByEntry: new Map<string, string>(),
  malformedIds: [],
  untaggedIds: [],
  legacyIds: [],
};

/**
 Refusals an operator's own mistake raises: a setting, a seed, a path or a
 pool they chose.
 */
const OPERATOR_MISTAKES: readonly Error[] = [
  new SpendCeilingOverrideError({ value: 'plenty', },),
  new BedrockCreditOverrideError({ value: 'plenty', },),
  new UnsafeSeedError({ seed: '../escape', },),
  new GradedSheetExistsError({ path: '/runs/grading-sheet-seed.md', },),
  new EmptyPoolError({
    census: TWO_PIPELINES,
    requiredCommit: 'cafe1234cafe1234',
  },),
  new MixedGenerationError({ census: TWO_PIPELINES, },),
  new HardCapOverrideError({ value: 'soon', },),
  new RunsDirectoryBusyError({
    runsDir: '/runs/sunbeam',
    holder: {
      pid: 4_242,
      startedAt: '2026-10-05T10:00:00.000Z',
      token: 'sunbeam-pass',
      identity: { kind: 'unrecorded', },
    },
    judgedBy: 'identity',
  },),
  new SheetBindingError({
    kind: 'seed',
    sheetLabel: 'repair sheet',
    sheetPath: '/runs/grading-sheet-sunbeam.md',
    manifestPath: '/runs/sample-manifest-moonbeam.json',
  },),
  new CorpusReadError({
    detail: 'people/whiskers/page.md at feedfac',
    cause: new Error('fatal: not a git repository',),
    commit: 'unopened',
  },),
];

await describe({
  name: reportingRefusals.name,
  children: [
    it({
      name: 'REPORTS a refusal this package wrote, naming the command and leaving a read code',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw fixtureRefusal();
          },
        },);

        expect(process.exitCode,).toBe(COULD_NOT_READ,);
        expect(printed.lines.length,).toBe(REPORTED_LINES,);
        expect(printed.lines[0],)
          .toBe('score-verify: could not read run.json as JSON (SyntaxError at byte 27)',);
      },
    },),
    it({
      name: 'NAMES a class this package did not write, without repeating what it said',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new RangeError(FAULT_MESSAGE,);
          },
        },);

        expect(process.exitCode,).toBe(UNEXPECTED_FAULT,);
        expect(printed.lines.length,).toBe(FAULT_LINES,);
        expect(printed.lines[0],).toBe('score-verify: refused by RangeError',);
      },
    },),
    it({
      name: 'REFUSES to print the message anywhere, frames line included',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new RangeError(FAULT_MESSAGE,);
          },
        },);

        /**
         Everything the reporter said, as one body to search.
         */
        const said = printed.lines.join('\n',);

        expect(said.includes(FAULT_MESSAGE,),).toBe(false,);
      },
    },),
    it({
      name: 'KEEPS the frames, so an unexpected fault stays locatable',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new RangeError(FAULT_MESSAGE,);
          },
        },);

        /**
         Frames as the reporter rendered them.
         */
        const frames = printed.lines[FRAMES_LINE] ?? '';

        expect(frames.includes('at ',),).toBe(true,);
      },
    },),
    it({
      name: 'CUTS THE FRAMES OFF WHERE THE STACK\'S HEADER ENDS, so a message holding a line break and then `at ` '
        + 'prints no line of the message',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new RangeError('the cat sat\n    at the whiskers of the diary page (diary.ts:3:4)\nand then slept',);
          },
        },);

        /**
         Everything the reporter said, as one body to search.
         */
        const said = printed.lines.join('\n',);

        expect(said.includes('diary',),).toBe(false,);
        expect((printed.lines[FRAMES_LINE] ?? '').includes('at ',),).toBe(true,);
      },
    },),
    it({
      name: 'NAMES NO FRAMES where the stack\'s header no longer holds the message it was built with, rather than '
        + 'guess where the frames begin',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            /**
             Fault whose message was rewritten after its stack was recorded.
             */
            const fault = new RangeError('the cat sat',);
            /**
             Stack read once, which is when the runtime writes its header: the header holds the message as it
             stood then.
             */
            const recorded = fault.stack;
            expect(recorded?.startsWith('RangeError: the cat sat\n',),).toBe(true,);
            fault.message = 'the cat sat\n    at the whiskers of the diary page (diary.ts:3:4)';
            throw fault;
          },
        },);

        expect(printed.lines[FRAMES_LINE],).toBe(
          '  (no frames: the stack\'s header does not hold the error\'s message, so where its frames begin is unknown)',
        );
      },
    },),
    it({
      name: 'NAMES NO FRAMES for a thrown value that is no Error, as a body throwing a bare text leaves, and reports '
        + 'it as a fault at its code',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            /**
             What a library rejecting with a bare text rather than an Error
             throws.
             */
            const bareText: unknown = FAULT_MESSAGE;
            throw bareText;
          },
        },);

        expect({
          code: process.exitCode,
          lines: printed.lines,
        },).toEqual({
          code: UNEXPECTED_FAULT,
          lines: [
            'score-verify: refused by a thrown value that is not an Error',
            '  This is a fault in the command rather than in the run. The stack frames printed with it name the '
              + 'built files it stopped in.',
            '  (no frames: the thrown value is not an Error)',
          ],
        },);
      },
    },),
    it({
      name: 'NAMES NO FRAMES for an Error that recorded no stack, rather than printing a missing stack as frames',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            /**
             Fault whose stack was taken off before it was thrown, as a
             library that strips its errors' stacks leaves them.
             */
            const fault = new RangeError(FAULT_MESSAGE,);
            Reflect.deleteProperty(
              fault,
              'stack',
            );
            throw fault;
          },
        },);

        expect({
          code: process.exitCode,
          lines: printed.lines,
        },).toEqual({
          code: UNEXPECTED_FAULT,
          lines: [
            'score-verify: refused by RangeError',
            '  This is a fault in the command rather than in the run. The stack frames printed with it name the '
              + 'built files it stopped in.',
            '  (no frames: this error recorded no stack)',
          ],
        },);
      },
    },),
    it({
      name: 'REPEATS a refusal stated in our own words, at its own code and with no frames',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new StatedRefusalError({ says: STATED_MESSAGE, },);
          },
        },);

        expect(process.exitCode,).toBe(REFUSED_AS_STATED,);
        expect(printed.lines.length,).toBe(STATED_LINES,);
        expect(printed.lines[0],).toBe(`score-verify: ${STATED_MESSAGE}`,);
      },
    },),
    ...OPERATOR_MISTAKES.map(function operatorMistakeCase(refusal,) {
      return it({
        name: caseName({ refusal, },),
        fn: () => reportsMistake({ refusal, },),
      },);
    },),
    it({
      name: 'KEEPS both halves for a marked class on the fault path, sentence and frames',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'ledger-report',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new LedgerShapeError({
              from: LEDGER_FILE,
              field: LEDGER_FIELD,
            },);
          },
        },);

        /**
         Frames as the reporter rendered them.
         */
        const frames = printed.lines[FRAMES_LINE] ?? '';

        expect(process.exitCode,).toBe(UNEXPECTED_FAULT,);
        expect(printed.lines.length,).toBe(FAULT_LINES,);
        expect(printed.lines[0],)
          .toBe(`ledger-report: ledger file ${LEDGER_FILE} has no usable ${LEDGER_FIELD}`,);
        expect(frames.includes('at ',),).toBe(true,);
      },
    },),
    it({
      name: 'PRINTS the seat report when the run ends clean, naming the seat '
        + 'that never answered, so a roster half that failed every call cannot '
        + 'pass as a comparison',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        /**
         Tally of a run in which one seat answered and one threw every time.
         */
        const seats = createSeatTally();
        seats.record({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, outcome: 'usable', },);
        seats.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
        seats.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);

        process.exitCode = 0;
        await reportingRefusals({
          what: 'editor-calibrate',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            // A run that finishes, which is what a half-dark calibration hid behind.
          },
          seats,
        },);

        /**
         The one line a reader who is not grepping must see.
         */
        const dark = printed.lines.find(function isDarkLine(line,): boolean {
          return line.startsWith('SEATS DARK: ',);
        },) ?? '';

        expect(process.exitCode,).toBe(0,);
        expect(
          printed.lines.includes('SEAT hf:openai/gpt-oss-120b asked=1 usable=1 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0',),
        ).toBe(true,);
        expect(
          printed.lines.includes('SEAT gemma-4-26b-a4b-it asked=2 usable=0 unusable=0 threw=2 silent=0 abandoned=0 unreachable=0',),
        ).toBe(true,);
        expect(dark.startsWith(
          'SEATS DARK: 1 of 2 seats asked produced nothing usable this run: '
            + 'gemma-4-26b-a4b-it (asked 2, unusable 0, threw 2).',
        ),).toBe(true,);
        expect(dark.includes('gpt-oss',),).toBe(false,);
      },
    },),
    it({
      name: 'PRINTS the seat report under a stated refusal too, so the report '
        + 'is on every exit path and not only the clean one',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        /**
         Tally with one seat that answered, unusably, every time.
         */
        const seats = createSeatTally();
        seats.record({ modelId: SEAT_HYPER_VISION, outcome: 'unusable', },);

        await reportingRefusals({
          what: 'editor-calibrate',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new StatedRefusalError({ says: STATED_MESSAGE, },);
          },
          seats,
        },);

        expect(process.exitCode,).toBe(REFUSED_AS_STATED,);
        expect(printed.lines[0],).toBe(`editor-calibrate: ${STATED_MESSAGE}`,);
        expect(printed.lines[1],).toBe('SEAT minimax-m3 asked=1 usable=0 unusable=1 threw=0 silent=0 abandoned=0 unreachable=0',);
        expect((printed.lines[2] ?? '').startsWith('SEATS DARK: 1 of 1 seat asked',),).toBe(true,);
      },
    },),
    it({
      name: 'DEFAULTS to the run-wide tally the client factory counts into, so '
        + 'no CLI has to thread it',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);
        using _fresh = withFreshRunSeats();

        RUN_SEATS.record({ modelId: SEAT_HYPER_VISION, outcome: 'usable', },);

        process.exitCode = 0;
        await reportingRefusals({
          what: 'corpus-pass',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            // A run that finishes.
          },
        },);

        expect(process.exitCode,).toBe(0,);
        expect(printed.lines.length,).toBe(1,);
        expect(printed.lines[0],).toBe('SEAT minimax-m3 asked=1 usable=1 unusable=0 threw=0 silent=0 abandoned=0 unreachable=0',);
      },
    },),
    it({
      name: 'REPORTS a run configuration refusal in its own words at exit 6, '
        + 'since that message names a variable and a fix and never content',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        await reportingRefusals({
          what: 'editor-calibrate',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            throw new RunConfigError({ variable: CONFIG_VARIABLE, },);
          },
          seats: createSeatTally(),
        },);

        expect(process.exitCode,).toBe(REFUSED_AS_STATED,);
        expect(printed.lines.length,).toBe(STATED_LINES,);
        expect(printed.lines[0],).toBe(`editor-calibrate: ${CONFIG_MESSAGE}`,);
      },
    },),
    it({
      name: 'LEAVES a clean run alone when no seat was asked, touching neither '
        + 'the exit code nor stderr',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        process.exitCode = 0;
        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async () => {
            // A body that simply finishes, which is every ordinary run.
          },
          seats: createSeatTally(),
        },);

        expect(process.exitCode,).toBe(0,);
        expect(printed.lines.length,).toBe(0,);
      },
    },),
    it({
      name: 'REFUSES a command line the command does not read before its body runs, at the stated code, where '
        + 'an argument once ran as if nothing had been typed (ledger B75)',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        /**
         Whether the body ran at all.
         */
        let ran = false;
        await reportingRefusals({
          what: 'score-verify',
          argv: [
            ...BARE_ARGV,
            '--only',
            'Tabby_01',
          ],
          env: NO_KEYS,
          run: async () => {
            ran = true;
          },
        },);

        expect(ran,).toBe(false,);
        expect(process.exitCode,).toBe(REFUSED_AS_STATED,);
        expect(printed.lines,).toEqual([
          'score-verify: --only is not a flag this command reads; this command takes no argument but its '
            + 'flags, and was given "Tabby_01". Usage: score-verify',
        ],);
      },
    },),
    it({
      name: 'HANDS the body the command line it read, the equals form and a switch included',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        /**
         What the body saw of the line.
         */
        const seen: unknown[] = [];
        await reportingRefusals({
          what: 'corpus-pass',
          argv: [
            ...BARE_ARGV,
            '--only=Tabby_01',
            '--plan',
          ],
          env: NO_KEYS,
          run: async ({ line, },) => {
            seen.push(
              line.flag('only',),
              line.switched('plan',),
            );
          },
          seats: createSeatTally(),
        },);

        expect(seen,).toEqual([
          {
            kind: 'written',
            flag: '--only',
            value: 'Tabby_01',
          },
          true,
        ],);
        expect(printed.lines,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6 when the process holds a key and names no command, before the command '
        + 'line is read and before the body runs',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        /**
         Whether the body ran at all.
         */
        let ran = false;
        await reportingRefusals({
          what: 'score-verify',
          argv: [
            ...BARE_ARGV,
            '--only',
            'Tabby_01',
          ],
          env: { WHISKER_API_KEY: 'purr', },
          run: async () => {
            ran = true;
          },
        },);

        expect(ran,).toBe(false,);
        expect(process.exitCode,).toBe(REFUSED_AS_STATED,);
        expect(printed.lines,).toEqual([
          'score-verify: will not run in a process that holds provider keys '
            + '(1 variable whose name ends in _API_KEY has a value) unless that process names this command as the '
            + 'one it means to start. '
            + 'The command\'s task does that: mise run //package/translation-repair/probe:score-verify '
            + '(the task builds the package first). '
            + 'To start the built file without building, as when a pass is running and nothing may rebuild, '
            + 'set TRANSLATION_REPAIR_STARTED_BY=score-verify for that one start. '
            + 'A process that holds no provider key needs neither.',
        ],);
      },
    },),
    it({
      name: 'RUNS the body when the process holds a key and names the command, as its own task does',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        /**
         Whether the body ran at all.
         */
        let ran = false;
        await reportingRefusals({
          what: 'score-verify',
          argv: BARE_ARGV,
          env: { WHISKER_API_KEY: 'purr', TRANSLATION_REPAIR_STARTED_BY: 'score-verify', },
          run: async () => {
            ran = true;
          },
          seats: createSeatTally(),
        },);

        expect(ran,).toBe(true,);
        expect(printed.lines,).toEqual([],);
      },
    },),
    it({
      name: 'HANDS THE BODY THE LINES IT WILL PRINT AT ITS END, built once, so the body can record the seat report and the '
        + 'end reports before its log closes and the command still prints each line once, as before (ledger B1111)',
      fn: async () => {
        using held = holdingExitCode();
        using printed = collectingErrors({ lines: [], },);

        /**
         Tally of a run in which one seat threw.
         */
        const seats = createSeatTally();
        seats.record({ modelId: SEAT_HYPER_TEXT_BEDROCK, outcome: 'threw', },);
        /**
         How often the end report was built.
         */
        const built = { count: 0, };
        registerRunEndReport({
          key: 'plan-energy-for-the-case',
          report: async () => {
            built.count += 1;
            return 'NEURALWATT plan energy: a made-up figure';
          },
        },);
        /**
         Lines the body was handed, asked for twice.
         */
        const handed: (readonly string[])[] = [];

        process.exitCode = 0;
        await reportingRefusals({
          what: 'editor-calibrate',
          argv: BARE_ARGV,
          env: NO_KEYS,
          run: async ({ closing, }) => {
            handed.push(
              await closing.lines(),
              await closing.lines(),
            );
          },
          seats,
        },);

        expect(process.exitCode,).toBe(0,);
        expect(built.count,).toBe(1,);
        expect(handed[0],).toBe(handed[1],);
        expect(handed[0]?.[0],).toBe('NEURALWATT plan energy: a made-up figure',);
        expect(printed.lines,).toEqual(handed[0],);
      },
    },),
    it({
      name: 'GIVES THE STACK FRAMES OF AN UNEXPECTED FAULT and nothing for a refusal the commands state in their own words, '
        + 'or for a run file that would not read: the frames, without the message or the cause (ledger B1111)',
      fn: async () => {
        /**
         Frames of a plain error.
         */
        const frames = framesOf({ error: new Error(FAULT_MESSAGE,), },);

        expect(frames.includes('    at ',),).toBe(true,);
        expect(frames.includes(FAULT_MESSAGE,),).toBe(false,);
        expect(isUnexpectedFault({ error: new Error(FAULT_MESSAGE,), },),).toBe(true,);
        expect(isUnexpectedFault({ error: 'a bare text', },),).toBe(true,);
        expect(isUnexpectedFault({ error: new StatedRefusalError({ says: STATED_MESSAGE, },), },),).toBe(false,);
        expect(isUnexpectedFault({ error: fixtureRefusal(), },),).toBe(false,);
      },
    },),
  ],
  concurrency: 1,
},);

//endregion CLI refusal tests
