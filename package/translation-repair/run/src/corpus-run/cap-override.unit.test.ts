/**
 Tests for the per-entry ceiling an invocation runs under.

 The ceiling is what stops one entry running away with a whole pass, so the
 case that matters most is the one where the override is WRONG. Falling back
 to the default there would leave an operator believing a run is bounded the
 way they asked for when it is bounded some other way, and nothing downstream
 could tell them otherwise: an artifact records no ceiling.

 The empty-string case is the other one worth having, and this package already
 carries the scar. `resolveRunsDir` had the same shape, and an
 exported-but-empty variable is an ordinary shell accident rather than an
 intention.

 The override text is handed in, since the entry file reads the environment
 and hands its text on (ledger B398). One case points the process's own
 variable elsewhere to show the read never consults it; it sets and restores
 the variable in a suite run one case at a time, so the value leaks into no
 other case.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  capOutlastsOneCall,
  capTooTightNote,
  HARD_CAP_VAR,
  HardCapOverrideError,
  resolveHardCapMinutes,
} from '../../dist/final/node/index.mjs';

/**
 Built-in ceiling these cases fall back to, standing in for the shipped one.
 */
const FALLBACK = 420;

/**
 Removes the hard-cap variable from the environment.

 @example
 ```ts
 clearHardCap();
 ```
 */
function clearHardCap(): void {
  Reflect.deleteProperty(
    process.env,
    HARD_CAP_VAR,
  );
}

/**
 Points the process's own hard-cap variable at a value for one case,
 restoring it after.

 @param says - value the process's variable holds while the case runs

 @returns Disposable that puts the variable back as it stood, its absence
 included

 @example
 ```ts
 using exported = hardCapSaying({ says: '15', },);
 ```
 */
function hardCapSaying({ says, }: { readonly says: string; },): Disposable {
  /**
   Value before the case, restored on dispose.
   */
  const before = process.env[HARD_CAP_VAR];

  process.env[HARD_CAP_VAR] = says;

  return {
    [Symbol.dispose]: function restore(): void {
      if (before === undefined)
        clearHardCap();
      else
        process.env[HARD_CAP_VAR] = before;
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: resolveHardCapMinutes.name,
      // One case at a time: a case points the process's own variable
      // elsewhere while the read runs.
      concurrency: 1,
      children: [
        it({
          name: 'SPELLS the variable the way the documentation does. Earlier in '
            + 'this work a sibling key was written as TRANSLATION_REPAIR_HYPER_API_KEY '
            + 'when the real name carried CHARM, and nothing failed until a live '
            + 'probe did: an operator who exports a name nothing reads gets the '
            + 'default and no complaint',
          fn: async () => {
            expect(HARD_CAP_VAR,).toBe('TRANSLATION_REPAIR_HARD_CAP_MINUTES',);
          },
        },),

        it({
          name: 'USES the built-in ceiling when nothing overrides it, which is '
            + 'every ordinary run',
          fn: async () => {
            expect(resolveHardCapMinutes({
              fallback: FALLBACK,
              raw: '',
            },),).toBe(FALLBACK,);
          },
        },),

        it({
          name: 'HONORS a positive override, which is what lets the re-attempt '
            + 'queue be exercised against an entry that fits in one run',
          fn: async () => {
            expect(resolveHardCapMinutes({
              fallback: FALLBACK,
              raw: '15',
            },),).toBe(15,);
          },
        },),

        it({
          name: 'IGNORES an empty override and falls back, since an '
            + 'exported-but-empty variable is a shell accident rather than an '
            + 'intention',
          fn: async () => {
            expect(resolveHardCapMinutes({
              fallback: FALLBACK,
              raw: '   ',
            },),).toBe(FALLBACK,);
          },
        },),

        it({
          name: 'REFUSES a value that is not a number rather than falling back, '
            + 'because a typo silently becoming the default leaves an operator '
            + 'believing the run is bounded the way they asked for',
          fn: async () => {
            expect(
              caught(function readTypo(): number {
                return resolveHardCapMinutes({
                  fallback: FALLBACK,
                  raw: 'soon',
                },);
              },),
            ).toBeInstanceOf(HardCapOverrideError,);
          },
        },),

        it({
          name: 'REFUSES a trailing-unit value such as `30m`, which `parseFloat` '
            + 'would have read as 30 and accepted: the number is right and the '
            + 'operator\'s belief about what they set is not',
          fn: async () => {
            expect(
              caught(function readUnit(): number {
                return resolveHardCapMinutes({
                  fallback: FALLBACK,
                  raw: '30m',
                },);
              },),
            ).toBeInstanceOf(HardCapOverrideError,);
          },
        },),

        it({
          name: 'REFUSES zero and negatives, which would cut every entry before it '
            + 'bought a single slice and leave the queue dropping all of them',
          fn: async () => {
            expect(
              caught(function readZero(): number {
                return resolveHardCapMinutes({
                  fallback: FALLBACK,
                  raw: '0',
                },);
              },),
            ).toBeInstanceOf(HardCapOverrideError,);
            expect(
              caught(function readNegative(): number {
                return resolveHardCapMinutes({
                  fallback: FALLBACK,
                  raw: '-5',
                },);
              },),
            ).toBeInstanceOf(HardCapOverrideError,);
          },
        },),

        it({
          name: 'REFUSES a value not written as a plain decimal, which `Number` read as minutes nobody typed: a '
            + 'hexadecimal, an exponent, a sign, a space either side and a point missing digits on one side, '
            + 'while a plain decimal such as 7.5 still reads (ledger B73)',
          fn: async () => {
            /**
             Spellings `Number` reads as minutes that no operator writes as minutes.
             */
            const spellings = [
              '0x10',
              '1e1',
              '+15',
              ' 15',
              '15 ',
              '15.',
              '.5',
            ];
            expect(spellings.map(function refusedOf(raw,): boolean {
              return caught(function readSpelling(): number {
                return resolveHardCapMinutes({
                  fallback: FALLBACK,
                  raw,
                },);
              },) instanceof HardCapOverrideError;
            },),).toEqual(spellings.map(function refused(): boolean {
              return true;
            },),);
            expect(resolveHardCapMinutes({
              fallback: FALLBACK,
              raw: '7.5',
            },),).toBe(7.5,);
          },
        },),

        it({
          name: 'REFUSES A POSITIVE FIGURE THAT ROUNDS TO ZERO WHOLE MILLISECONDS, naming the figure and the '
            + 'smallest figure accepted, and replaces no figure an operator wrote',
          fn: async () => {
            /**
             Error raised for a figure of 0.006 milliseconds.
             */
            const refusal = caught(function readTooFine(): number {
              return resolveHardCapMinutes({
                fallback: FALLBACK,
                raw: '0.0000001',
              },);
            },);
            expect(refusal,).toBeInstanceOf(HardCapOverrideError,);
            expect(String(refusal,),).toContain('received "0.0000001"',);
            expect(String(refusal,),).toContain('0.0000083334 minutes',);
          },
        },),

        it({
          name: 'READS HALF A MILLISECOND AND MORE as the figure written, and refuses a finer one, so the '
            + 'smallest figure the refusal names is itself accepted',
          fn: async () => {
            expect(resolveHardCapMinutes({
              fallback: FALLBACK,
              raw: '0.0000083334',
            },),).toBe(0.0000083334,);
            expect(resolveHardCapMinutes({
              fallback: FALLBACK,
              raw: '0.0000167',
            },),).toBe(0.0000167,);
            expect(caught(function readBelowHalf(): number {
              return resolveHardCapMinutes({
                fallback: FALLBACK,
                raw: '0.0000083',
              },);
            },),).toBeInstanceOf(HardCapOverrideError,);
          },
        },),

        it({
          name: 'READS ONLY THE TEXT IT IS HANDED, never the process\'s own variable, whose unreadable value '
            + 'would refuse the read and whose number would replace the fallback',
          fn: async () => {
            {
              using refused = hardCapSaying({ says: 'soon', },);
              expect(resolveHardCapMinutes({
                fallback: FALLBACK,
                raw: '15',
              },),).toBe(15,);
            }
            {
              using exported = hardCapSaying({ says: '15', },);
              expect(resolveHardCapMinutes({
                fallback: FALLBACK,
                raw: '',
              },),).toBe(FALLBACK,);
            }
          },
        },),
      ],
    },),

    describe({
      name: capOutlastsOneCall.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a ceiling longer than one exchange, which is the only shape that can buy a slice',
          fn: async () => {
            expect(capOutlastsOneCall({
              capMs: 520_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            },),).toBe(true,);
          },
        },),
        it({
          name:
            'REFUSES a ceiling equal to one exchange, because the attempt ends at the same instant the '
              + 'exchange is allowed to and nothing has returned yet',
          fn: async () => {
            expect(capOutlastsOneCall({
              capMs: 360_000,
              perCallMs: 360_000,
              claudeCallMs: 360_000,
            },),).toBe(false,);
          },
        },),
        it({
          name:
            'REFUSES the ceiling a live run actually used, five minutes against a six minute exchange, '
              + 'which cached nothing across two attempts and reported STALLED',
          fn: async () => {
            expect(capOutlastsOneCall({
              capMs: 300_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            },),).toBe(false,);
          },
        },),
        it({
          name:
            'REFUSES A CEILING BETWEEN THE TWO DEADLINES (F2 of the re-check of the second fix round): it outlasts the '
              + 'default exchange and cuts a Claude exchange that its own deadline allows',
          fn: async () => {
            expect(capOutlastsOneCall({
              capMs: 400_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            },),).toBe(false,);
            expect(capOutlastsOneCall({
              capMs: 500_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: capTooTightNote.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES BOTH NUMBERS, so a reader can tell which one to move without reading the source',
          fn: async () => {
            /**
             Note built over the ceiling and exchange deadline of the live run.
             */
            const note = capTooTightNote({
              capMs: 300_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            },);

            expect(note.includes('300000',),).toBe(true,);
            expect(note.includes('360000',),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES THE CLAUDE DEADLINE BESIDE THE DEFAULT ONE (F2 of the re-check of the second fix round)',
          fn: async () => {
            /**
             Note built over a ceiling between the two deadlines.
             */
            const note = capTooTightNote({
              capMs: 400_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            },);

            expect(note.includes('400000',),).toBe(true,);
            expect(note.includes('360000',),).toBe(true,);
            expect(note.includes('460000',),).toBe(true,);
          },
        },),
        it({
          name:
            'SAYS WHAT FOLLOWS rather than only that something is wrong: no slice caches, so the queue '
              + 'reads no progress and drops the entry',
          fn: async () => {
            /**
             Same note, read for its consequence clause.
             */
            const note = capTooTightNote({
              capMs: 300_000,
              perCallMs: 360_000,
              claudeCallMs: 460_000,
            },);

            expect(note.includes('no slice caches',),).toBe(true,);
            expect(note.includes('stalled',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
