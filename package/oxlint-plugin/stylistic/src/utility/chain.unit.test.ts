import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ChainSegment,
  renderCanonical,
  selectBreaks,
} from '../../dist/final/node/index.mjs';

/** Attached segment helper for building fixture segment streams. */
const attached: ChainSegment = { isBreak: false, };

/**
 Parameters for {@link breakAt}.
 */
type BreakAtParams = {
  /**
   Byte offset where the break's continuation line begins.
   */
  readonly breakOffset: number;
  /**
   Byte offset of the node owning the leading token; anchors the line's indent.
   */
  readonly anchorOffset: number;
};

/**
 Builds a break segment carrying both offsets for fixture segment streams.
 
 The anchor offset travels through selection so tests can prove a break keeps
 the indent anchor it was built with.
 
 @returns break segment carrying the given offsets
 */
function breakAt({
  breakOffset,
  anchorOffset,
}: BreakAtParams,): ChainSegment {
  return {
    isBreak: true,
    breakOffset,
    anchorOffset,
  };
}

await describe({
  name: '',
  children: [
    //region selectBreaks

    describe({
      name: selectBreaks.name,
      children: [
        it({
          name: 'returns no breaks when nothing breaks',
          fn: async () => {
            expect(
              selectBreaks([
                attached,
                attached,
                attached,
              ],),
            ).toEqual([],);
          },
        },),
        it({
          name: 'keeps a break point on the head line when only one segment precedes it',
          fn: async () => {
            // `a + b`: [leaf, operator break]; the break sits at index 1 and attaches.
            expect(
              selectBreaks([
                attached,
                breakAt({
                  breakOffset: 2,
                  anchorOffset: 0,
                },),
              ],),
            ).toEqual([],);
          },
        },),
        it({
          name: 'breaks every break point at segment index two or greater',
          fn: async () => {
            // `obj.b.c.d`: [leaf, .b@4, .c@6, .d@8]; `.b` attaches, `.c` and `.d` break.
            expect(
              selectBreaks([
                attached,
                breakAt({
                  breakOffset: 4,
                  anchorOffset: 1,
                },),
                breakAt({
                  breakOffset: 6,
                  anchorOffset: 2,
                },),
                breakAt({
                  breakOffset: 8,
                  anchorOffset: 3,
                },),
              ],),
            ).toEqual([
              {
                breakOffset: 6,
                anchorOffset: 2,
              },
              {
                breakOffset: 8,
                anchorOffset: 3,
              },
            ],);
          },
        },),
        it({
          name: 'counts attached segments toward the head when locating the first break',
          fn: async () => {
            // `foo().bar()[0]`: [leaf, call, .bar@5, call, computed]; `.bar` is index 2 and breaks.
            expect(
              selectBreaks([
                attached,
                attached,
                breakAt({
                  breakOffset: 5,
                  anchorOffset: 0,
                },),
                attached,
                attached,
              ],),
            ).toEqual([
              {
                breakOffset: 5,
                anchorOffset: 0,
              },
            ],);
          },
        },),
      ],
    },),

    //endregion selectBreaks

    //region renderCanonical

    describe({
      name: renderCanonical.name,
      children: [
        it({
          name: 'returns the verbatim region when there are no breaks',
          fn: async () => {
            expect(
              renderCanonical({
                sourceText: 'obj.method()',
                regionStart: 0,
                regionEnd: 12,
                breaks: [],
                headIndent: '  ',
              },),
            ).toBe('obj.method()',);
          },
        },),
        it({
          name: 'lays each break two columns past its anchor line',
          fn: async () => {
            // `obj.b.c.d`: dots at 5 (`.c`) and 7 (`.d`), both anchored at `obj`.
            expect(
              renderCanonical({
                sourceText: 'obj.b.c.d',
                regionStart: 0,
                regionEnd: 9,
                breaks: [
                  {
                    breakOffset: 5,
                    anchorOffset: 0,
                  },
                  {
                    breakOffset: 7,
                    anchorOffset: 0,
                  },
                ],
                headIndent: '',
              },),
            ).toBe('obj.b\n  .c\n  .d',);
          },
        },),
        it({
          name: 'trims trailing whitespace so no line ends in spaces',
          fn: async () => {
            // `a  +  b` with a break at the operator (index 2) leaves no trailing space on the head.
            expect(
              renderCanonical({
                sourceText: 'a  +  b',
                regionStart: 0,
                regionEnd: 7,
                breaks: [
                  {
                    breakOffset: 3,
                    anchorOffset: 0,
                  },
                ],
                headIndent: '',
              },),
            ).toBe('a\n  +  b',);
          },
        },),
        it({
          name: 'nests a break past an anchor riding an earlier continuation line',
          fn: async () => {
            // `aa.b().c() + dd.e().f()`: breaks at `.c` (6), `+` (11), `.f` (19); the
            // `.f` anchor `dd` (13) rides the `+ dd.e()` line, so `.f` nests one
            // level deeper than that line.
            expect(
              renderCanonical({
                sourceText: 'aa.b().c() + dd.e().f()',
                regionStart: 0,
                regionEnd: 23,
                breaks: [
                  {
                    breakOffset: 6,
                    anchorOffset: 0,
                  },
                  {
                    breakOffset: 11,
                    anchorOffset: 0,
                  },
                  {
                    breakOffset: 19,
                    anchorOffset: 13,
                  },
                ],
                headIndent: '',
              },),
            ).toBe('aa.b()\n  .c()\n  + dd.e()\n    .f()',);
            // Re-rendering the output at its own break offsets reproduces it.
            expect(
              renderCanonical({
                sourceText: 'aa.b()\n  .c()\n  + dd.e()\n    .f()',
                regionStart: 0,
                regionEnd: 33,
                breaks: [
                  {
                    breakOffset: 9,
                    anchorOffset: 0,
                  },
                  {
                    breakOffset: 16,
                    anchorOffset: 0,
                  },
                  {
                    breakOffset: 29,
                    anchorOffset: 18,
                  },
                ],
                headIndent: '',
              },),
            ).toBe('aa.b()\n  .c()\n  + dd.e()\n    .f()',);
          },
        },),
        it({
          name: 'anchors on a verbatim interior line use that line\'s source indent',
          fn: async () => {
            // `a\n  + /* c\n      x */ dd\n    .e`: the `+` slice spans two source
            // lines because of the comment, and the `.e` anchor `dd` (22) sits on
            // the interior line indented six columns, so `.e` nests to eight.
            expect(
              renderCanonical({
                sourceText: 'a\n  + /* c\n      x */ dd\n    .e',
                regionStart: 0,
                regionEnd: 31,
                breaks: [
                  {
                    breakOffset: 4,
                    anchorOffset: 0,
                  },
                  {
                    breakOffset: 29,
                    anchorOffset: 22,
                  },
                ],
                headIndent: '',
              },),
            ).toBe('a\n  + /* c\n      x */ dd\n        .e',);
          },
        },),
        it({
          name: 'is idempotent: re-rendering already-canonical source reproduces it',
          fn: async () => {
            // Already laid out: dots at 8 (`.c`) and 13 (`.d`); trailing indent trims away.
            const canonical = 'obj.b\n  .c\n  .d';
            expect(
              renderCanonical({
                sourceText: canonical,
                regionStart: 0,
                regionEnd: canonical.length,
                breaks: [
                  {
                    breakOffset: 8,
                    anchorOffset: 0,
                  },
                  {
                    breakOffset: 13,
                    anchorOffset: 0,
                  },
                ],
                headIndent: '',
              },),
            ).toBe(canonical,);
          },
        },),
      ],
    },),

    //endregion renderCanonical
  ],
},);
