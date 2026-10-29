/**
 Tests that each boundary reading a parse as a refusal reads as the refusal only an engine stack exhaustion raised by the
 parse call itself, and lets an exhaustion raised by any other code of the same try propagate. Each boundary is run twice
 through its public entry: once with the parser failing the way a stack too short to build the refusal makes it fail
 (the exhaustion leaves the grammar's function raw), and once with the parse returning a tree that raises the exhaustion
 when the boundary's own code reads it.

 The entries are read from the built package; the ones no caller of the package needs are exported from its index marked
 `@internal`, for tests of the built artifact like this one.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { tenseCandidateGroups, } from '@monochromatic-dev/translation-repair-translate-stage';
import { splitTargetOnlyRun, } from '@monochromatic-dev/translation-repair-translate-lane';
import {
  isLineStructured,
  strictRefusalOffset,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { introducedStructuralRegressions, } from '@monochromatic-dev/translation-repair-assembly';
import { inspectParagraph, } from '@monochromatic-dev/translation-repair-repair';
import {
  FootnoteRewriteError,
  fragmentReadingOf,
  MarkdownParseError,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  blocksOrWholeText,
  proseBodyTree,
  readPageSkeleton,
  readSliceSkeleton,
} from '@monochromatic-dev/translation-repair-glossary';
import { blockBreaksIn, } from '@monochromatic-dev/translation-repair-consistency';
import { archiveContributorNameForms, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  addedFormulaFindings,
  foldSoftBreaks,
  sourceBreakDisplay,
  unwrappedLinkFindings,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  activeFootnoteMarkers,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  ENGINE_EXHAUSTION_MESSAGE,
  withExhaustionFromMethod,
  withExhaustionFromParse,
  withExhaustionFromWalk,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/parse-exhaustion.test-fixture.ts';
import { grammarAcceptsDocument, } from '@monochromatic-dev/translation-repair-document-structure/ts/tag-name-spellings.test-fixture.ts';

/**
 The one kind a page the plain grammar refuses is reported as, by the assembly check.
 */
const UNREADABLE_PAGE = 'unreadable-page';

/**
 A text the strict grammar refuses by its nesting bound before the parser reads it, so only the plain grammar parses it.
 */
const STRICT_REFUSES_BY_BOUND = '{'.repeat(257,);

/**
 Whether a thrown value is the engine's stack exhaustion, unconverted.

 @param value - what a call threw

 @returns Whether it is a `RangeError` with the engine's message

 @example
 ```ts
 const raw = isRawExhaustion({ value: new RangeError('Maximum call stack size exceeded',), }); // true
 ```
 */
function isRawExhaustion({ value, }: { readonly value: unknown; }): boolean {
  return (value instanceof RangeError) && (!(value instanceof MarkdownParseError)) && (value.message === ENGINE_EXHAUSTION_MESSAGE);
}

/**
 Whether a receiver is a non-empty list of which every member has a property, which names the one list a built-in method is
 asked to raise on, since the build keeps no name of a callback to name it by.

 @param receiver - what a built-in method is called on

 @param property - property every member of the list the test names has

 @returns Whether the receiver is such a list

 @example
 ```ts
 const named = holdsOnly({ receiver: [{ count: 1, },], property: 'count', },); // true
 ```
 */
function holdsOnly(
  {
    receiver,
    property,
  }: {
    readonly receiver: unknown;
    readonly property: string;
  },
): boolean {
  if ((!Array.isArray(receiver,)) || (receiver.length === 0))
    return false;
  return receiver.every(function hasProperty(member,): boolean {
    return ((typeof member) === 'object') && (member !== null) && (property in member);
  },);
}

/**
 What `inspectParagraph` answers for a paragraph with a definition appended while the parser fails with no room to
 build its refusal, from one parse on.

 @param afterParses - how many parses go to the real parser first, so 0 fails the paragraph alone and 1 fails the one
 with the definitions appended

 @returns Inspection

 @example
 ```ts
 const inspection = inspectedWhileParserFails({ afterParses: 1, },);
 ```
 */
function inspectedWhileParserFails({ afterParses, }: { readonly afterParses: number; },): unknown {
  return withExhaustionFromParse({
    run: function inspect(): unknown {
      return inspectParagraph({ text: 'A cat[^1] naps.', definitions: '[^1]: A note.', },);
    },
    selection: { afterParses, },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'boundaries that read an exhaustion of the parse as the grammar\'s refusal',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'blockBreaksIn READS AN EXHAUSTION OF THE PARSE AS THE REFUSAL and lets one raised by reading the tree propagate',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function count(): number {
                return blockBreaksIn({ text: 'a\n\nb', },);
              },
            },),).toBe(1,);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function count(): number {
                    return blockBreaksIn({ text: 'a\n\nb', },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'isLineStructured READS AN EXHAUSTION OF THE PARSE AS THE REFUSAL and lets one raised by reading the tree propagate',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function ask(): boolean {
                return isLineStructured({ text: 'a\n\nb', },);
              },
            },),).toBe(false,);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function ask(): boolean {
                    return isLineStructured({ text: 'a\n\nb', },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'blocksOrWholeText READS AN EXHAUSTION OF THE PARSE AS THE REFUSAL and lets one raised by reading the tree propagate',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function split(): readonly string[] {
                return blocksOrWholeText({ text: 'a\n\nb', },);
              },
            },),).toEqual(['a\n\nb',],);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function split(): readonly string[] {
                    return blocksOrWholeText({ text: 'a\n\nb', },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'addedFormulaFindings READS AN EXHAUSTION OF THE PARSE AS THE REFUSAL and lets one raised by reading the tree '
            + 'propagate',
          fn: async () => {
            /**
             The findings for a candidate that forms formulas.
             */
            const findings = (): readonly string[] => addedFormulaFindings({
              sourceText: '猫',
              candidateText: 'The cat costs $5 and $10.',
            },);
            expect(withExhaustionFromParse({ run: findings, },),).toEqual([],);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({ run: findings, },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'readSliceSkeleton READS AN EXHAUSTION OF THE PARSE AS THE REFUSAL and lets one raised by reading the tree propagate',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function read(): string {
                return readSliceSkeleton({ text: 'A cat naps.', },).kind;
              },
            },),).toBe('unparseable',);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function read(): string {
                    return readSliceSkeleton({ text: 'A cat naps.', },).kind;
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'readPageSkeleton READS AN EXHAUSTION OF THE PLAIN PARSE AS THE REFUSAL and lets one raised by reading the plain '
            + 'tree propagate, for a page only the plain grammar parses',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function read(): string {
                return readPageSkeleton({ text: STRICT_REFUSES_BY_BOUND, },).read.kind;
              },
            },),).toBe('unparseable',);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function read(): string {
                    return readPageSkeleton({ text: STRICT_REFUSES_BY_BOUND, },).read.kind;
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'unwrappedLinkFindings READS AN EXHAUSTION OF THE PLAIN PARSE AS NO ATOMS and lets one raised by reading the plain '
            + 'tree propagate',
          fn: async () => {
            /**
             The findings for a rendering that only the plain grammar parses.
             */
            const findings = (): readonly string[] => unwrappedLinkFindings({
              sourceText: '[the nap](https://example.invalid/nap)',
              candidateText: STRICT_REFUSES_BY_BOUND,
            },);
            expect(withExhaustionFromParse({ run: findings, },),).toEqual([],);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({ run: findings, },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'foldSoftBreaks READS AN EXHAUSTION OF THE PARSE AS THE REFUSAL and lets one raised by the code that picks the '
            + 'paragraphs propagate',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function fold(): string {
                return foldSoftBreaks({ text: 'The cat slept,\nand woke.', },);
              },
            },),).toBe('The cat slept,\nand woke.',);
            expect(isRawExhaustion({
              value: caught(function pick(): void {
                withExhaustionFromMethod({
                  owner: Array.prototype as unknown as Record<string, unknown>,
                  method: 'filter',
                  applies: function isThePick(receiver,): boolean {
                    return holdsOnly({ receiver, property: 'zone', },);
                  },
                  run: function fold(): string {
                    return foldSoftBreaks({ text: 'The cat slept,\nand woke.', },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'introducedStructuralRegressions READS AN EXHAUSTION OF THE ASSEMBLY\'S PARSE AS AN UNREADABLE PAGE and lets one '
            + 'raised by the code that compares the counts propagate',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function compare(): readonly string[] {
                return introducedStructuralRegressions({
                  incumbentText: 'A cat naps.',
                  assembledText: 'ASSEMBLED cat naps.',
                },);
              },
              selection: { appliesTo: function isAssembled(input,): boolean {
                return String(input,).includes('ASSEMBLED',);
              }, },
            },),).toEqual([UNREADABLE_PAGE,],);
            expect(isRawExhaustion({
              value: caught(function compare(): void {
                withExhaustionFromMethod({
                  owner: Array.prototype as unknown as Record<string, unknown>,
                  method: 'filter',
                  applies: function isTheComparison(receiver,): boolean {
                    return holdsOnly({ receiver, property: 'count', },);
                  },
                  run: function compareKinds(): readonly string[] {
                    return introducedStructuralRegressions({
                      incumbentText: 'A cat naps.',
                      assembledText: 'A cat naps.',
                    },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'sourceBreakDisplay READS AN EXHAUSTION OF THE PARSE AS THE REFUSAL and lets one raised by the code that writes the '
            + 'display propagate',
          fn: async () => {
            /**
             A slice with an explicit break in it: a backslash ending a line.
             */
            const sourceText = 'The cat naps\\\nand wakes.';
            /**
             The display of that slice.
             */
            const display = (): string => sourceBreakDisplay({
              sourceText,
              archiveText: '',
              incumbentKind: 'absent',
            },);
            expect(withExhaustionFromParse({ run: display, },),).toBe(sourceText,);
            expect(isRawExhaustion({
              value: caught(function write(): void {
                withExhaustionFromMethod({
                  owner: Array.prototype as unknown as Record<string, unknown>,
                  method: 'join',
                  applies: function isTheDisplay(receiver,): boolean {
                    return Array.isArray(receiver,) && receiver.includes('<br/>',);
                  },
                  run: display,
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'sourceBreakDisplay LETS AN EXHAUSTION RAISED BY THE WALK OF THE PARSED TREE FOR ITS BREAK SPANS PROPAGATE, the '
            + 'walk being no part of the parse, which the slice body parse reads as the refusal itself',
          fn: async () => {
            /**
             A slice with an explicit break in it: a backslash ending a line.
             */
            const sourceText = 'The cat naps\\\nand wakes.';
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromMethod({
                  owner: Array.prototype as unknown as Record<string, unknown>,
                  method: 'toReversed',
                  applies: function isTheWalk(receiver,): boolean {
                    return holdsOnly({ receiver, property: 'type', },);
                  },
                  run: function display(): string {
                    return sourceBreakDisplay({
                      sourceText,
                      archiveText: '',
                      incumbentKind: 'absent',
                    },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'archiveContributorNameForms READS AN EXHAUSTION OF THE PARSE AS NO REFERENCE LINKS and lets one raised by walking '
            + 'the tree propagate',
          fn: async () => {
            /**
             An archive page whose attribution line holds a reference link.
             */
            const text = 'Contributors for this entry: Mika, [Neko][n]\n\n[n]: https://example.test\n';
            expect(withExhaustionFromParse({
              run: function read(): readonly string[] {
                return archiveContributorNameForms({ text, },);
              },
            },),).toEqual(['[Neko][n]', 'Mika',],);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function read(): readonly string[] {
                    return archiveContributorNameForms({ text, },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'fragmentReadingOf READS AN EXHAUSTION OF EITHER PARSE AS THE REFUSAL and lets one raised by reading either tree '
            + 'propagate',
          fn: async () => {
            /**
             A fragment with a footnote reference and its definition.
             */
            const text = 'A cat[^1] naps.\n\n[^1]: A note.\n';
            expect([
              withExhaustionFromParse({
                run: function read(): boolean {
                  return fragmentReadingOf({ text, },).definitions.settled;
                },
              },),
              withExhaustionFromParse({
                run: function read(): boolean {
                  return fragmentReadingOf({ text, },).definitions.settled;
                },
                selection: { afterParses: 1, },
              },),
            ],).toEqual([false, true,],);
            expect([
              isRawExhaustion({
                value: caught(function walkPlain(): void {
                  withExhaustionFromWalk({
                    run: function read(): unknown {
                      return fragmentReadingOf({ text, },);
                    },
                  },);
                },),
              },),
              isRawExhaustion({
                value: caught(function walkStrict(): void {
                  withExhaustionFromWalk({
                    run: function read(): unknown {
                      return fragmentReadingOf({ text, },);
                    },
                    selection: { afterParses: 1, },
                  },);
                },),
              },),
            ],).toEqual([true, true,],);
          },
        },),
        it({
          name: 'activeFootnoteMarkers READS AN EXHAUSTION OF THE PARSE AS A SYNTAX REFUSAL and lets one raised by walking the tree '
            + 'propagate',
          fn: async () => {
            /**
             A page with one footnote.
             */
            const text = 'Real[^1].\n\n[^1]: Note.';
            expect(withExhaustionFromParse({
              run: function read(): string {
                return caught(function mark(): void {
                  activeFootnoteMarkers({ text, },);
                },) instanceof FootnoteRewriteError ? 'refused' : 'other';
              },
            },),).toBe('refused',);
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function mark(): unknown {
                    return activeFootnoteMarkers({ text, },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'proseBodyTree, splitTargetOnlyRun AND tenseCandidateGroups READ AN EXHAUSTION OF THEIR PARSE AS THE REFUSAL, '
            + 'whose try holds no code beyond the parse call that a test can make raise',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function read(): readonly [number, number, number, number,] {
                return [
                  proseBodyTree({ text: 'A cat.', },).root.children.length,
                  splitTargetOnlyRun({ sourceText: 'A cat.', incumbentText: 'A cat.', },).protectedText.length,
                  tenseCandidateGroups({ lifeEnded: true, texts: [{ label: 'a', text: 'Mittens naps.', },], },).length,
                  strictRefusalOffset({ text: 'A cat.', },).refused ? 1 : 0,
                ];
              },
            },),).toEqual([0, 0, 0, 1,],);
          },
        },),
        it({
          name: 'splitTargetOnlyRun LETS AN EXHAUSTION RAISED AFTER ITS PARSE RETURNED PROPAGATE, the code that reads the '
            + 'blocks off the parse being no part of the parse',
          fn: async () => {
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function split(): unknown {
                    return splitTargetOnlyRun({ sourceText: 'A cat.', incumbentText: 'A cat.', },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'tenseCandidateGroups LETS AN EXHAUSTION RAISED AFTER ITS PARSE RETURNED PROPAGATE, the code that reads the '
            + 'blocks off the parse being no part of the parse',
          fn: async () => {
            expect(isRawExhaustion({
              value: caught(function walk(): void {
                withExhaustionFromWalk({
                  run: function group(): unknown {
                    return tenseCandidateGroups({ lifeEnded: true, texts: [{ label: 'a', text: 'Mittens naps.', },], },);
                  },
                },);
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'inspectParagraph READS AN EXHAUSTION OF EITHER PARSE AS AN UNPARSEABLE PARAGRAPH, the paragraph alone and the '
            + 'paragraph with its definitions appended',
          fn: async () => {
            /**
             The answer for a paragraph the strict grammar refused.
             */
            const unparseable = {
              kind: 'rejected',
              reason: 'unparseable',
            };
            expect([
              inspectedWhileParserFails({ afterParses: 0, },),
              inspectedWhileParserFails({ afterParses: 1, },),
            ],).toEqual([unparseable, unparseable,],);
          },
        },),
        it({
          name: 'grammarAcceptsDocument OF THE TAG NAME FIXTURE READS AN EXHAUSTION OF THE PARSE AS A DOCUMENT THE GRAMMAR DOES NOT '
            + 'ACCEPT',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function ask(): boolean {
                return grammarAcceptsDocument({ document: '<details>\n\nA cat naps.\n\n</details>\n', },);
              },
            },),).toBe(false,);
          },
        },),
        it({
          name: 'parseDocument RAISES THE PLAIN GRAMMAR\'S REFUSAL FOR AN EXHAUSTION OF BOTH PARSES, where plain markdown is the '
            + 'fallback of the strict grammar',
          fn: async () => {
            expect(withExhaustionFromParse({
              run: function read(): unknown {
                return caught(function parse(): void {
                  parseDocument({ text: 'A cat.', },);
                },);
              },
            },),).toBeInstanceOf(MarkdownParseError,);
          },
        },),
      ],
    },),
  ],
},);
