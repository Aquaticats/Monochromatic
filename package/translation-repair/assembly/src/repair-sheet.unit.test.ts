/**
 Tests for the repair grading sheet, including the property that matters most
 for comparing rounds: the DETECTION sheet still shows no repair text.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertRepairMeasurable,
  countUnrecordedRepairs,
  formatGradingSheet,
  type GradableRepair,
  type GradingCandidate,
  UnmeasurableRepairError,
} from '@monochromatic-dev/translation-repair-house-rule';
import { formatRepairSheet, } from '../dist/final/node/index.mjs';

/**
 Replacement text no detection sheet may ever contain.
 */
const REPLACEMENT = 'The cat is asleep on the windowsill.';

/**
 Text the replacement stands in for.
 */
const REPLACED = 'The cat is doing the sleeping on the windowsill.';

/**
 Builds one sampled candidate, optionally carrying repair provenance.

 @param repair - provenance, omitted to model a pre-recording artifact

 @returns Candidate both sheets render

 @example
 ```ts
 const candidate = catCandidate({},);
 ```
 */
function catCandidate(
  { repair, }: { readonly repair?: GradableRepair; },
): GradingCandidate {
  return {
    entryId: 'Kitten',
    band: 'medium' as const,
    issueId: 'adjudicated/nap',
    category: 'accuracy/awkward',
    severity: 'minor',
    summary: 'The nap sentence reads as a literal gloss.',
    sourceQuotes: ['猫猫在窗台上睡觉。',],
    targetQuotes: [REPLACED,],
    sourceAnchor: 'quoted' as const,
    ...(repair === undefined ? {} : { repair, }),
  };
}

/**
 Builds repair provenance with one region.

 @param disposition - what became of the repair

 @param issueIds - issues the region serves

 @param refined - whether the naturalness lane rewrote the slice

 @param rewriteReachedReader - whether the returned document carries that
 rewrite, which is what decides whether a final wording was recorded at all

 @returns Provenance the repair sheet renders

 @example
 ```ts
 const repair = catRepair({ disposition: 'shipped', },);
 ```
 */
function catRepair(
  {
    disposition,
    issueIds = ['adjudicated/nap',],
    refined = false,
    rewriteReachedReader = true,
  }: {
    readonly disposition: string;
    readonly issueIds?: readonly string[];
    readonly refined?: boolean;
    readonly rewriteReachedReader?: boolean;
  },
): GradableRepair {
  return {
    disposition,
    regions: [
      {
        issueIds,
        before: REPLACED,
        editorAfter: REPLACEMENT,
      },
    ],
    refined,
    ...((refined && rewriteReachedReader)
      ? { finalSliceText: 'The cat sleeps on the windowsill all afternoon.', }
      : {}),
  };
}

/**
 Shortest run of backticks Markdown accepts as a fence.
 */
const FENCE_MIN = 3;

/**
 Removes every fenced block from a sheet, leaving only text a Markdown reader
 would interpret as sheet structure.

 Tracks the opening fence and drops lines until a fence at least as long
 closes it, which is how a Markdown reader resolves the same question.

 @param sheet - rendered sheet

 @returns Sheet text outside every fenced block

 @example
 ```ts
 const structure = stripFences({ sheet, },);
 ```
 */
function stripFences({ sheet, }: { readonly sheet: string; },): string {
  /**
   Kept lines and the fence currently open, if any.
   */
  const state: {
    readonly kept: string[];
    open: string;
  } = {
    kept: [],
    open: '',
  };
  for (const line of sheet.split('\n',)) {
    /**
     Leading backtick run length of this line; a linear scan rather than a
     pattern, since the rule is "how many backticks start this line".
     */
    let run = 0;
    while ((run < line.length) && (line.charAt(run,) === '`'))
      run += 1;

    if (state.open === '') {
      if (run >= FENCE_MIN) {
        state.open = '`'.repeat(run,);
        continue;
      }
      state.kept
        .push(line,);
      continue;
    }
    if (run >= state.open
      .length)
      state.open = '';
  }
  return state.kept
    .join('\n',);
}

await describe({
  name: formatRepairSheet.name,
  children: [
    it({
      name: 'the DETECTION sheet renders only its six known lines per item, so '
        + 'no repair field can reach it and change what its number measures',
      fn: async () => {
        // A visible correction makes an alleged defect look more real. If it
        // leaked onto the detection sheet, round three's precision would be
        // measured by a different instrument than round two's, and the change
        // of instrument would read as a change in the pipeline.
        //
        // Asserted structurally rather than by naming the strings that must be
        // absent: a check for "does not contain the replacement text" passes
        // happily on a leak through some FUTURE field, which is exactly the
        // leak nothing else would catch.
        const sheet = formatGradingSheet({
          sample: [catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),],
          seed: 'cat-seed',
          bar: 0.9,
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);

        /** Line prefixes the detection sheet is allowed to render per item. */
        const allowed = [
          '### ',
          '- entry: ',
          '- category: ',
          '- claim: ',
          '- zh source: ',
          '- en target: ',
        ];

        /** Item block, everything after the header's horizontal rule. */
        const body = sheet.split('\n---\n',)
          .at(-1,)
          ?? '';

        /** Rendered lines that no allowed prefix accounts for. */
        const unexpected = body.split('\n',)
          .filter(function isRendered(line,) {
            return line !== '';
          },)
          .filter(function isUnaccounted(line,) {
            return !allowed.some(function starts(prefix,) {
              return line.startsWith(prefix,);
            },);
          },);
        expect(unexpected,).toEqual([],);
        expect(sheet.includes(REPLACEMENT,),).toBe(false,);
      },
    },),

    it({
      name: 'orders the grader to finish detection first and gives a shipped '
        + 'repair its own grade box',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('GRADE THE DETECTION SHEET FIRST',),).toBe(true,);
        expect(sheet.includes(REPLACEMENT,),).toBe(true,);
        expect(sheet.includes('repair grade: [ ]',),).toBe(true,);
      },
    },),

    it({
      name: 'never shows the checker verdict, which would anchor the human '
        + 'toward agreement on the very population they are auditing',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        // Asserted against the model rather than against wording: GradableRepair
        // carries no checker field at all, so no rewording of this sheet can
        // start disclosing one.
        expect(
          Object.keys(catRepair({ disposition: 'shipped', },),),
        )
          .toEqual([
            'disposition',
            'regions',
            'refined',
          ],);
        expect(sheet.includes('resolved',),).toBe(false,);
      },
    },),

    it({
      name: 'fences replaced text so a replacement carrying markdown cannot '
        + 'invent a heading or a grade box on the sheet',
      fn: async () => {
        // The replacement is corpus-derived model output crossing into markdown
        // grammar. Interpolated raw, a line like the fixture's `editorAfter` puts a
        // grade box
        // on the sheet that nobody wrote, and a grader would fill it in.
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: {
                disposition: 'shipped',
                regions: [
                  {
                    issueIds: ['adjudicated/nap',],
                    before: 'before text',
                    editorAfter:
                      '### 99. injected\n- repair grade: [ ]\n``` not a fence',
                  },
                ],
                refined: false,
              },
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);

        // Counted OUTSIDE fenced blocks, because that is the real property:
        // fencing does not delete the injected characters, it stops them being
        // read as sheet. A raw substring count would fail on text that renders
        // harmlessly as code.
        /** Sheet with every fenced block removed. */
        const outsideFences = stripFences({ sheet, },);

        /** Grade boxes surviving outside fences, one per gradable item. */
        const boxes = outsideFences.split('- repair grade: [ ]',)
          .length
          - 1;
        expect(boxes,).toBe(1,);

        /** Item headings surviving outside fences. */
        const headings = outsideFences.split('\n### ',)
          .length
          - 1;
        expect(headings,).toBe(1,);

        // The injected backtick run must not be able to close its own block,
        // which means the chosen fence has to be longer than it.
        expect(sheet.includes('````',),).toBe(true,);
      },
    },),

    it({
      name: 'says a deletion was a deletion instead of rendering an empty '
        + 'replacement that reads as a rendering fault',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: {
                disposition: 'shipped',
                regions: [
                  {
                    issueIds: ['adjudicated/nap',],
                    before: 'a fabricated sentence',
                    editorAfter: '',
                  },
                ],
                refined: false,
              },
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('DELETED',),).toBe(true,);
      },
    },),

    it({
      name: 'says a replacement that shows a reader nothing was a deletion, rendering the sheet exactly as for '
        + 'the empty replacement: spaces, a zero-width space, a Hangul filler, or both among spaces',
      fn: async () => {
        /**
         Repair sheet over one region whose replacement is as given.

         @param editorAfter - what the pipeline wrote in place of the region

         @returns Rendered sheet

         @example
         ```ts
         const sheet = sheetAfter('\u{3164}',);
         ```
         */
        function sheetAfter(editorAfter: string,): string {
          return formatRepairSheet({
            sample: [
              catCandidate({
                repair: {
                  disposition: 'shipped',
                  regions: [
                    {
                      issueIds: ['adjudicated/nap',],
                      before: 'a fabricated sentence',
                      editorAfter,
                    },
                  ],
                  refined: false,
                },
              },),
            ],
            seed: 'cat-seed',
            corpusSha: 'sha/1',
            drawDigest: 'digest-of-this-draw',
          },);
        }
        /** The sheet for a replacement the pipeline wrote as nothing at all. */
        const deleted = sheetAfter('',);
        expect([
          '   ',
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ].map(sheetAfter,),).toEqual([
          deleted,
          deleted,
          deleted,
          deleted,
        ],);
      },
    },),

    it({
      name: 'shows the returned slice for an item whose targeted repair lost '
        + 'but whose slice the naturalness lane rewrote anyway, since the text '
        + 'the reader got is not the original either',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: catRepair({
                disposition: 'not-selected',
                refined: true,
              },),
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('The cat sleeps on the windowsill all afternoon.',))
          .toBe(true,);
        expect(sheet.includes('rewrote this slice anyway',),).toBe(true,);
        expect(sheet.includes('- repair grade: [ ]',),).toBe(false,);
      },
    },),

    it({
      name: 'says the rewrite was TAKEN BACK, and shows no returned slice, for '
        + 'a refined item whose wording the document never carried. Rendering '
        + 'the fence anyway would put an empty block under "the slice as '
        + 'actually returned", which reads as a slice that came back blank',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: catRepair({
                disposition: 'withdrawn',
                refined: true,
                rewriteReachedReader: false,
              },),
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('taken back before the document was returned',))
          .toBe(true,);
        expect(sheet.includes('the slice as actually returned',),).toBe(false,);
        expect(sheet.includes('rewrote this slice anyway',),).toBe(false,);
        expect(sheet.includes('- repair grade: [ ]',),).toBe(false,);
      },
    },),

    it({
      name: 'carries the zh original onto this sheet, since that is what "does '
        + 'it fix it" is answered against',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('猫猫在窗台上睡觉。',),).toBe(true,);
      },
    },),

    it({
      name: 'withholds the grade box where no repair reached the reader, and '
        + 'says which of those happened instead of leaving a gap',
      fn: async () => {
        for (const disposition of [
          'not-selected',
          'withdrawn',
          'no-region',
        ]) {
          /** Sheet for one non-shipping disposition. */
          const sheet = formatRepairSheet({
            sample: [catCandidate({ repair: catRepair({ disposition, },), },),],
            seed: 'cat-seed',
            corpusSha: 'sha/1',
            drawDigest: 'digest-of-this-draw',
          },);
          expect(sheet.includes('repair grade: [ ]',),).toBe(false,);
          expect(sheet.includes(disposition,),).toBe(true,);
          expect(sheet.includes('counts against coverage',),).toBe(true,);
        }
      },
    },),

    it({
      name: 'withholds the refinement caveat from an ungradable item, since '
        + '"grade the final wording" beside "not graded" is a contradiction '
        + 'the grader would have to resolve alone',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: catRepair({
                disposition: 'not-selected',
                refined: true,
              },),
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('grade the FINAL wording',),).toBe(false,);
        expect(sheet.includes('counts against coverage',),).toBe(true,);
      },
    },),

    it({
      name: 'refuses a gate sample carrying ANY pre-recording item, since the '
        + 'sheet would otherwise render and the round would report a repair '
        + 'number over whatever fraction happened to be recorded',
      fn: async () => {
        /** Mixed sample: one recorded repair and one from an older run. */
        const mixed = [
          catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),
          catCandidate({},),
        ];
        expect(countUnrecordedRepairs({ sample: mixed, },),).toBe(1,);

        /** Failure raised by the unmeasurable item. */
        let caught: unknown;
        try {
          assertRepairMeasurable({ sample: mixed, },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(UnmeasurableRepairError,);
        expect((caught as Error).message,).toContain('1 of 2',);
      },
    },),

    it({
      name: 'admits a gate sample where every item states what was written',
      fn: async () => {
        /** Sample whose every item carries recorded provenance. */
        const measurable = [
          catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),
          catCandidate({ repair: catRepair({ disposition: 'no-region', },), },),
        ];
        expect(countUnrecordedRepairs({ sample: measurable, },),).toBe(0,);
        // A `no-region` item is a real measurement, not a missing one: it says
        // no targeted repair exists, which belongs in the coverage denominator.
        assertRepairMeasurable({ sample: measurable, },);
      },
    },),

    it({
      name: 'marks a pre-recording item as ungradable rather than as a repair '
        + 'that never happened',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [catCandidate({},),],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('NOT GRADABLE',),).toBe(true,);
        expect(sheet.includes('repair grade: [ ]',),).toBe(false,);
      },
    },),

    it({
      name: 'shows the final wording and says to grade it when the naturalness '
        + 'lane rewrote the paragraph after the repair was written',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: catRepair({
                disposition: 'shipped',
                refined: true,
              },),
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('The cat sleeps on the windowsill all afternoon.',))
          .toBe(true,);
        expect(sheet.includes('grade the RETURNED wording',),).toBe(true,);
      },
    },),

    it({
      name: 'discloses the other accepted issues a shared edit was written '
        + 'for, so one replacement is not read as this issue\'s own repair',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: catRepair({
                disposition: 'shipped',
                issueIds: [
                  'adjudicated/nap',
                  'adjudicated/chase',
                ],
              },),
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('SHARED',),).toBe(true,);
        // Issue ids are 64-character hashes a grader cannot look up, so the
        // sheet names sheet POSITIONS instead and says plainly when a sibling
        // was not drawn.
        expect(sheet.includes('adjudicated/chase',),).toBe(false,);
        expect(sheet.includes('none of which were drawn into this sample',),)
          .toBe(true,);
      },
    },),

    it({
      name: 'names the sheet positions a shared edit repeats under, since that '
        + 'is the fact a grader can act on when they meet the same before and '
        + 'after text again',
      fn: async () => {
        /** Two drawn issues served by one shared region. */
        const shared = catRepair({
          disposition: 'shipped',
          issueIds: [
            'adjudicated/nap',
            'adjudicated/chase',
          ],
        },);
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({ repair: shared, },),
            {
              ...catCandidate({ repair: shared, },),
              issueId: 'adjudicated/chase',
            },
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('appear here as item(s) 2',),).toBe(true,);
        expect(sheet.includes('appear here as item(s) 1',),).toBe(true,);
        expect(sheet.includes('adjudicated/chase',),).toBe(false,);
      },
    },),

    it({
      name: 'counts the drawn siblings of a shared edit when some were not drawn, and lists their '
        + 'positions in sheet order whatever order the edit names them in',
      fn: async () => {
        /** One region serving four issues, three of them drawn. */
        const shared = catRepair({
          disposition: 'shipped',
          issueIds: [
            'adjudicated/nap',
            'adjudicated/chase',
            'adjudicated/pounce',
            'adjudicated/groom',
          ],
        },);
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({ repair: shared, },),
            {
              ...catCandidate({ repair: shared, },),
              issueId: 'adjudicated/pounce',
            },
            {
              ...catCandidate({ repair: shared, },),
              issueId: 'adjudicated/chase',
            },
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        // Item 1's siblings are named chase (item 3), pounce (item 2) and
        // groom (not drawn).
        expect(sheet.includes('3 other accepted issue(s), of which 2 appear here as item(s) 2, 3.',),)
          .toBe(true,);
      },
    },),

    it({
      name: 'says a region written into an insertion point replaced nothing, instead of fencing an '
        + 'empty before',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({
              repair: {
                disposition: 'shipped',
                regions: [
                  {
                    issueIds: ['adjudicated/nap',],
                    before: '',
                    editorAfter: 'The cat also naps in the afternoon sun.',
                  },
                ],
                refined: false,
              },
            },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('- before: (nothing; this is an insertion point',),).toBe(true,);
        expect(sheet.includes('- before, the text that was replaced:',),).toBe(false,);
      },
    },),

    it({
      name: 'carries a disposition the sheet does not know to the grader with a note to report it, '
        + 'and no grade box, rather than dropping the item from the population',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [catCandidate({ repair: catRepair({ disposition: 'held-by-a-later-pass', },), },),],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes(
          '- outcome: held-by-a-later-pass (unrecognized disposition; report this rather than grading it)',
        ),).toBe(true,);
        expect(sheet.includes('- repair grade:',),).toBe(false,);
      },
    },),

    it({
      name: 'carries a disposition spelled as a name every object inherits as unrecognized too, where a '
        + 'plain-object table of notes printed the inherited function or object as the note (ledger B77)',
      fn: async () => {
        for (const disposition of ['constructor', '__proto__', 'toString',]) {
          /**
           Sheet for one item whose stored disposition is that name.
           */
          const sheet = formatRepairSheet({
            sample: [catCandidate({ repair: catRepair({ disposition, },), },),],
            seed: 'cat-seed',
            corpusSha: 'sha/1',
            drawDigest: 'digest-of-this-draw',
          },);
          expect(sheet.includes(
            `- outcome: ${disposition} (unrecognized disposition; report this rather than grading it)`,
          ),).toBe(true,);
        }
      },
    },),

    it({
      name: 'says a side quoted nothing, rather than printing an empty line the grader would read '
        + 'as a lost quote',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            {
              ...catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),
              sourceQuotes: [],
            },
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('- zh original says: (nothing quoted on this side)',),).toBe(true,);
      },
    },),

    it({
      name: 'numbers items from one so they line up with the detection sheet',
      fn: async () => {
        const sheet = formatRepairSheet({
          sample: [
            catCandidate({ repair: catRepair({ disposition: 'shipped', },), },),
            catCandidate({ repair: catRepair({ disposition: 'no-region', },), },),
          ],
          seed: 'cat-seed',
          corpusSha: 'sha/1',
          drawDigest: 'digest-of-this-draw',
        },);
        expect(sheet.includes('### 1. Kitten',),).toBe(true,);
        expect(sheet.includes('### 2. Kitten',),).toBe(true,);
      },
    },),
  ],
},);
