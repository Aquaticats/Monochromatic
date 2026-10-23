/**
 Tests for the lines a corpus pass prints: the restriction, the pairs it could
 not read, the START line, every limit a launch changed, the required
 providers, the plan and the closing line. Each is read whole.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { CorpusPinSetting, } from '@monochromatic-dev/translation-repair-run';
import {
  passDoneLine,
  passIncompleteLine,
  passLaunchLines,
  passOnlyLines,
  passPlanLine,
  passProvidersOffLines,
  passRequiredLines,
  passStartLine,
} from '../../dist/final/node/index.mjs';

/**
 Corpus pin neither half of which a launch overrode.
 */
const BUILT_IN_PIN: CorpusPinSetting = {
  pin: {
    cloneDir: '/clones/cats',
    commitSha: 'a'.repeat(40,),
  },
  cloneDirSource: 'fallback',
  commitSource: 'fallback',
};

/**
 Limits no launch changed: the built-in ceiling, a room for one exchange,
 the built-in allowance, no window note and the built-in pin.
 */
const UNCHANGED = {
  hardCapMs: 25_200_000,
  builtInCapMinutes: 420,
  perCallMs: 360_000,
  claudeCallMs: 460_000,
  spendCeilingUsd: 20,
  graceNote: '',
  writerNote: '',
  pinSetting: BUILT_IN_PIN,
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: passOnlyLines.name,
      children: [
        it({
          name: 'SAYS NOTHING for a pass restricted to no entry',
          fn: async () => {
            expect(passOnlyLines({ onlyIds: new Set<string>(), },),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES ONE ENTRY the pass was restricted to, saying the operator chose it in place of every '
            + 'pending pair and that it runs in the pass\'s own order if still pending, in the singular',
          fn: async () => {
            expect(passOnlyLines({ onlyIds: new Set(['tabby',],), },),).toEqual([
              'ONLY tabby (chosen by hand in place of every pending pair at the pin; if still pending it runs in '
                + 'the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a hand-picked '
                + 'entry never enters a pool later draws treat as natural accumulation)',
            ],);
          },
        },),
        it({
          name: 'NAMES SEVERAL ENTRIES in code point order, whatever order they were written in, so two runs of '
            + 'one selection log alike, saying those of them still pending run in the pass\'s own order',
          fn: async () => {
            expect(passOnlyLines({ onlyIds: new Set([
              '😺',
              'mittens',
              'Tabby',
            ],), },),).toEqual([
              'ONLY Tabby,mittens,😺 (chosen by hand in place of every pending pair at the pin; those still pending '
                + 'run in the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a hand-picked '
                + 'entry never enters a pool later draws treat as natural accumulation)',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: passIncompleteLine.name,
      children: [
        it({
          name: 'NAMES AN ENTRY whose English page is absent, with what the read said',
          fn: async () => {
            expect(passIncompleteLine({ gap: {
              id: 'mittens',
              side: 'target',
              detail: 'missing-object',
            }, },),).toBe('INCOMPLETE mittens: target page absent at the pin (missing-object)',);
          },
        },),
        it({
          name: 'NAMES AN ENTRY whose original page is absent',
          fn: async () => {
            expect(passIncompleteLine({ gap: {
              id: 'biscuit',
              side: 'source',
              detail: 'missing-object',
            }, },),).toBe('INCOMPLETE biscuit: source page absent at the pin (missing-object)',);
          },
        },),
      ],
    },),

    describe({
      name: passStartLine.name,
      children: [
        it({
          name: 'REPORTS the tip, the pipeline and its files, the pending and the finished counts and both ceilings',
          fn: async () => {
            expect(passStartLine({ facts: {
              tip: 'deadbeef',
              pipelineDigest: 'sha256-tree-v1:cafe',
              fileCount: 189,
              pending: 1,
              done: 0,
              softBudgetMs: 259_200_000,
              hardCapMs: 25_200_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            }, },),).toBe(
              'START tip=deadbeef pipeline=sha256-tree-v1:cafe files=189 pending=1 done=0 soft=259200000ms '
                + 'hard=25200000ms call=360000ms claude-call=460000ms',
            );
          },
        },),
        it({
          name: 'REPORTS several pending and finished entries',
          fn: async () => {
            expect(passStartLine({ facts: {
              tip: 'deadbeef',
              pipelineDigest: 'sha256-tree-v1:cafe',
              fileCount: 1,
              pending: 12,
              done: 3,
              softBudgetMs: 60_000,
              hardCapMs: 90_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            }, },),).toBe(
              'START tip=deadbeef pipeline=sha256-tree-v1:cafe files=1 pending=12 done=3 soft=60000ms hard=90000ms '
                + 'call=360000ms claude-call=460000ms',
            );
          },
        },),
      ],
    },),

    describe({
      name: passLaunchLines.name,
      children: [
        it({
          name: 'SAYS NOTHING when no limit was changed and one exchange fits in the ceiling',
          fn: async () => {
            expect(passLaunchLines(UNCHANGED,),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES A CEILING OF SEVERAL MINUTES in the plural, and a fractional one too',
          fn: async () => {
            expect(passLaunchLines({
              ...UNCHANGED,
              hardCapMs: 600_000,
            },),).toEqual([
              'CAP OVERRIDDEN by TRANSLATION_REPAIR_HARD_CAP_MINUTES: entries run under 10 minutes rather than '
                + 'the built-in 420',
            ],);
            expect(passLaunchLines({
              ...UNCHANGED,
              hardCapMs: 510_000,
            },),).toEqual([
              'CAP OVERRIDDEN by TRANSLATION_REPAIR_HARD_CAP_MINUTES: entries run under 8.5 minutes rather than '
                + 'the built-in 420',
            ],);
          },
        },),
        it({
          name: 'NAMES A CEILING OF ONE MINUTE in the singular, and warns that it is no longer than one exchange',
          fn: async () => {
            expect(passLaunchLines({
              ...UNCHANGED,
              hardCapMs: 60_000,
            },),).toEqual([
              'CAP OVERRIDDEN by TRANSLATION_REPAIR_HARD_CAP_MINUTES: entries run under 1 minute rather than '
                + 'the built-in 420',
              'CAP TOO TIGHT: an attempt runs 60000ms, which is not longer than the 460000ms one model exchange is '
                + 'allowed (360000ms by default, 460000ms for a Claude exchange). Attempts are cut before an exchange '
                + 'can return, so no slice caches, every attempt '
                + 'reports no progress, and the queue drops the entry as stalled after its second try. Raise the '
                + 'ceiling above one exchange to buy anything, or keep it here to exercise the stall path '
                + 'deliberately.',
            ],);
          },
        },),
        it({
          name: 'WARNS of a ceiling equal to one exchange, which does not outlast it, without naming it overridden '
            + 'when it is the built-in',
          fn: async () => {
            expect(passLaunchLines({
              ...UNCHANGED,
              perCallMs: 25_200_000,
              claudeCallMs: 25_200_000,
            },),).toEqual([
              'CAP TOO TIGHT: an attempt runs 25200000ms, which is not longer than the 25200000ms one model '
                + 'exchange is allowed (25200000ms by default, 25200000ms for a Claude exchange). Attempts are cut '
                + 'before an exchange can return, so no slice caches, every '
                + 'attempt reports no progress, and the queue drops the entry as stalled after its second try. '
                + 'Raise the ceiling above one exchange to buy anything, or keep it here to exercise the stall '
                + 'path deliberately.',
            ],);
          },
        },),
        it({
          name: 'WARNS of a ceiling between the default exchange and a Claude exchange (F2 of the re-check of the '
            + 'second fix round), naming both deadlines',
          fn: async () => {
            /**
             Lines for a ceiling that outlasts the default exchange and not a Claude one.
             */
            const lines = passLaunchLines({
              ...UNCHANGED,
              hardCapMs: 400_000,
              claudeCallMs: 460_000,
            },);

            expect(lines.length,).toBe(2,);
            expect(lines[1]?.startsWith('CAP TOO TIGHT: an attempt runs 400000ms',),).toBe(true,);
            expect(lines[1]?.includes('460000ms',),).toBe(true,);
            expect(lines[1]?.includes('360000ms',),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES AN OVERRIDDEN SPEND ALLOWANCE',
          fn: async () => {
            expect(passLaunchLines({
              ...UNCHANGED,
              spendCeilingUsd: 5,
            },),).toEqual([
              'SPEND CEILING OVERRIDDEN by TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD: new entries stop once this '
                + 'run has spent 5 USD on any one of openrouter, deepseek, xiaomi rather than the built-in 20',
            ],);
          },
        },),
        it({
          name: 'PRINTS BOTH WINDOW NOTES as the entry file handed them, and a corpus pin a launch overrode',
          fn: async () => {
            expect(passLaunchLines({
              ...UNCHANGED,
              graceNote: 'STRAGGLER GRACE OVERRIDDEN: a window',
              writerNote: 'WRITER GRACE OVERRIDDEN: another window',
              pinSetting: {
                pin: {
                  cloneDir: '/clones/cats',
                  commitSha: 'b'.repeat(40,),
                },
                cloneDirSource: 'TRANSLATION_REPAIR_CORPUS_CLONE_DIR',
                commitSource: 'fallback',
              },
            },),).toEqual([
              'STRAGGLER GRACE OVERRIDDEN: a window',
              'WRITER GRACE OVERRIDDEN: another window',
              `CORPUS PIN OVERRIDDEN: clone /clones/cats from TRANSLATION_REPAIR_CORPUS_CLONE_DIR, commit ${
                'b'.repeat(40,)
              } from fallback`,
            ],);
          },
        },),
        it({
          name: 'ORDERS the lines cap, spend, windows, pin, then the warning',
          fn: async () => {
            expect(passLaunchLines({
              hardCapMs: 120_000,
              builtInCapMinutes: 420,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
              spendCeilingUsd: 1,
              graceNote: 'grace',
              writerNote: 'writer',
              pinSetting: {
                pin: {
                  cloneDir: '/clones/cats',
                  commitSha: 'c'.repeat(40,),
                },
                cloneDirSource: 'fallback',
                commitSource: 'TRANSLATION_REPAIR_CORPUS_COMMIT',
              },
            },).map(function firstWord(line,): string {
              return line.split(' ',)[0] ?? '';
            },),).toEqual([
              'CAP',
              'SPEND',
              'grace',
              'writer',
              'CORPUS',
              'CAP',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: passRequiredLines.name,
      children: [
        it({
          name: 'SAYS NOTHING when no provider was required',
          fn: async () => {
            expect(passRequiredLines({ providers: [], },),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES ONE REQUIRED PROVIDER as wet',
          fn: async () => {
            expect(passRequiredLines({ providers: ['hyper',], },),).toEqual(['REQUIRED-PROVIDERS hyper status=wet',],);
          },
        },),
        it({
          name: 'NAMES SEVERAL REQUIRED PROVIDERS in the order they were required',
          fn: async () => {
            expect(passRequiredLines({ providers: [
              'openrouter',
              'hyper',
            ], },),).toEqual(['REQUIRED-PROVIDERS openrouter,hyper status=wet',],);
          },
        },),
      ],
    },),

    describe({
      name: passProvidersOffLines.name,
      children: [
        it({
          name: 'SAYS NOTHING when no provider is off',
          fn: async () => {
            expect(passProvidersOffLines({ off: [], },),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES EACH PROVIDER THAT IS OFF AND WHY, one line each in the order given, so the launch says Bedrock '
            + 'is not asked instead of the run finding out by a refusal',
          fn: async () => {
            expect(passProvidersOffLines({ off: [
              {
                provider: 'bedrock',
                reason: 'the cat tore up the key',
              },
              {
                provider: 'hyper',
                reason: 'the dog ate the balance',
              },
            ], },),).toEqual([
              'PROVIDER OFF bedrock: the cat tore up the key',
              'PROVIDER OFF hyper: the dog ate the balance',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: passPlanLine.name,
      children: [
        it({
          name: 'NAMES NO ENTRY for a plan with nothing pending',
          fn: async () => {
            expect(passPlanLine({
              tip: 'deadbeef',
              pipelineDigest: 'sha256-tree-v1:cafe',
              pendingIds: [],
              previewCount: 5,
            },),).toBe('PLAN ok tip=deadbeef pipeline=sha256-tree-v1:cafe client=constructed pending=0 first=',);
          },
        },),
        it({
          name: 'NAMES THE ONE ENTRY of a plan with one pending',
          fn: async () => {
            expect(passPlanLine({
              tip: 'deadbeef',
              pipelineDigest: 'sha256-tree-v1:cafe',
              pendingIds: ['tabby',],
              previewCount: 5,
            },),).toBe('PLAN ok tip=deadbeef pipeline=sha256-tree-v1:cafe client=constructed pending=1 first=tabby',);
          },
        },),
        it({
          name: 'PREVIEWS ONLY THE FIRST ENTRIES of a longer plan, and counts them all',
          fn: async () => {
            expect(passPlanLine({
              tip: 'deadbeef',
              pipelineDigest: 'sha256-tree-v1:cafe',
              pendingIds: [
                'a',
                'b',
                'c',
              ],
              previewCount: 2,
            },),).toBe('PLAN ok tip=deadbeef pipeline=sha256-tree-v1:cafe client=constructed pending=3 first=a,b',);
          },
        },),
      ],
    },),

    describe({
      name: passDoneLine.name,
      children: [
        it({
          name: 'REPORTS what the pass finished of what it set out to run, the artifacts among the pairs its walk '
            + 'over every entry found, against those pairs less the declined ones, the declined pairs and the '
            + 'artifacts of entries the walk found no complete pair for apart, and the time taken',
          fn: async () => {
            expect(passDoneLine({
              processed: 1,
              pending: 2,
              pairs: {
                walked: 'every-entry',
                ids: new Set([
                  'biscuit',
                  'mochi',
                  'tabby',
                  'tama',
                ],),
              },
              settledIds: new Set([
                'biscuit',
                'ghost',
                'tabby',
              ],),
              declinedIds: new Set(['mochi',],),
              elapsedMs: 5,
            },),).toBe('DONE processed=1 of pending=2; artifacts=2/3 declined=1 unpairedArtifacts=1 elapsed=5ms',);
          },
        },),
        it({
          name: 'COUNTS A PAIR CARRYING BOTH AN ARTIFACT AND A DECLINE RECORD once, among the artifacts',
          fn: async () => {
            expect(passDoneLine({
              processed: 0,
              pending: 0,
              pairs: {
                walked: 'every-entry',
                ids: new Set(['tabby',],),
              },
              settledIds: new Set(['tabby',],),
              declinedIds: new Set(['tabby',],),
              elapsedMs: 5,
            },),).toBe('DONE processed=0 of pending=0; artifacts=1/1 declined=0 unpairedArtifacts=0 elapsed=5ms',);
          },
        },),
        it({
          name: 'SAYS THE PAIRS WERE COUNTED OVER THE NAMED ENTRIES ALONE when --only kept the walk to them, and '
            + 'counts no artifact apart for want of a pair, since the walk read no other entry',
          fn: async () => {
            expect(passDoneLine({
              processed: 1,
              pending: 1,
              pairs: {
                walked: 'named-entries',
                ids: new Set(['tabby',],),
              },
              settledIds: new Set([
                'biscuit',
                'tabby',
              ],),
              declinedIds: new Set<string>(),
              elapsedMs: 5,
            },),).toBe(
              'DONE processed=1 of pending=1; artifacts=1/1 declined=0 (pairs counted over the entries --only '
                + 'named, not every pair at the pin) elapsed=5ms',
            );
          },
        },),
      ],
    },),
  ],
},);
