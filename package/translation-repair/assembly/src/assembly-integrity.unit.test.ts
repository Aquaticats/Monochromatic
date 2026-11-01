/**
 Tests for the assembly-time footnote guard.

 What this covers that no per-slice check can: a footnote is a relation
 BETWEEN slices, so a candidate that drops, renames or invents a marker
 validates perfectly inside its own slice and breaks the document.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import { footnoteIdentifiers, } from '@monochromatic-dev/translation-repair-document-structure';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  AssemblyContractError,
  guardFootnoteAssembly,
  introducedFootnoteFindings,
  introducedStructuralRegressions,
} from '../dist/final/node/index.mjs';

/**
 Original document: three sections, so the reference, the unrelated prose and
 the definition each land in their OWN slice. A footnote that fits inside one
 slice is not the case this guard exists for.
 */
const SOURCE_TEXT = `## 猫

猫猫在窗台上打盹〔1〕。

## 鸟

窗台上有一只鸟。

## 注

〔1〕：那是它最喜欢的位置。
`;

/**
 Translation as it stands, with the footnote pair intact and split the same
 way.
 */
const TARGET_TEXT = `## The cat

The cat is doing the sleeping on the windowsill[^1].

## The bird

On the windowsill there is being a bird.

## Notes

[^1]: That is its favourite spot.
`;

/**
 Prepares the fixture pair and returns its slices.

 @param targetText - translation to prepare against the fixture original

 @returns Slices in document order

 @example
 ```ts
 const slices = fixtureSlices({ targetText: TARGET_TEXT, },);
 ```
 */
function fixtureSlices({ targetText, }: { readonly targetText: string; },) {
  return prepareDocumentPair({
    sourceText: SOURCE_TEXT,
    targetText,
  },).slices;
}

/**
 Chunk index of the slice whose incumbent text contains a needle.

 @param slices - prepared slices

 @param needle - text that slice carries

 @returns Chunk index of the first slice carrying it

 @example
 ```ts
 const index = sliceCarrying({ slices, needle: '[^1]:', },);
 ```
 */
function sliceCarrying(
  {
    slices,
    needle,
  }: {
    readonly slices: readonly {
      readonly target: {
        readonly sliceIndex: number;
        readonly text: string;
      };
    }[];
    readonly needle: string;
  },
): number {
  /**
   First slice whose incumbent carries the needle.
   */
  const found = slices.find(function carries(slice,): boolean {
    return slice.target
      .text
      .includes(needle,);
  },);
  if (found === undefined)
    throw new Error(`no slice carries ${needle}`,);
  return found.target
    .sliceIndex;
}

/**
 Chunk indices in ascending order, so a comparison says which slices were
 withdrawn rather than which round withdrew them.

 @param indices - chunk indices in withdrawal order

 @returns Same indices, ascending

 @example
 ```ts
 expect(byIndex({ indices: guarded.revertedChunkIndices, },),).toEqual([0, 1,],);
 ```
 */
function byIndex(
  { indices, }: { readonly indices: readonly number[]; },
): readonly number[] {
  return indices.toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'assembly-integrity',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'counts every footnote identifier a text mentions, in EITHER role: '
            + 'attribution asks which slice changed its mention of an identifier, '
            + 'and a definition line mentions its own label',
          fn: async () => {
            expect([
              ...footnoteIdentifiers({ text: 'A nap[^1] and a bird[^2].', },)
                .entries(),
            ],).toEqual([
              ['reference gfm 1', 1,],
              ['reference gfm 2', 1,],
            ],);
            // The role is what a bare identifier cannot say: turning a definition
            // into prose that refers to it leaves the identifier counted once
            // either way.
            expect(footnoteIdentifiers({ text: '[^1]: That is its spot.', },)
              .get('definition gfm 1',),).toBe(1,);
            expect(footnoteIdentifiers({ text: '猫猫打盹〔1〕。', },)
              .get('reference fullwidth-bracket 1',),).toBe(1,);
            expect(footnoteIdentifiers({ text: '〔1〕：那是它的位置。', },)
              .get('definition fullwidth-bracket 1',),).toBe(1,);
          },
        },),

        it({
          name: 'CANONICALIZES a set of replacements that reassemble to the archive '
            + 'text. Moving a paragraph across a slice join changes both slices and '
            + 'no byte of the document, so every later reader would be told the '
            + 'document carries changes it does not carry. Withdrawing them is the '
            + 'true document-level claim, and each lane still holds what it decided',
          fn: async () => {
            // Paragraphs small enough that subdivision groups several into one
            // slice, which is what puts a movable join between two of them. One
            // paragraph per slice leaves only the blank line between them, and
            // nothing can move across that without changing a byte.
            const targetText = `${
          Array.from(
            { length: 30, },
            function toParagraph(
              _unused,
              index,
            ): string {
              return `Whisker note ${String(index,)}: the cat naps on the sill.`;
            },
          )
            .join('\n\n',)
        }\n`;

            /**
             Original the same shape, since preparation pairs them by structure.
             */
            const sourceText = `${
          Array.from(
            { length: 30, },
            function toParagraph(
              _unused,
              index,
            ): string {
              return `第${String(index,)}条：猫猫在窗台上打盹。`;
            },
          )
            .join('\n\n',)
        }\n`;

            /**
             Slices of a document long enough to subdivide.
             */
            const { slices, } = prepareDocumentPair({
              sourceText,
              targetText,
            },);

            const [first, second,] = slices;
            if ((first === undefined) || (second === undefined))
              throw new Error('fixture produced fewer than two slices',);

            // Positive control on the FIXTURE, not on the guard: the join is only
            // movable when the two slices are separated by exactly a blank line and
            // the later one holds more than one paragraph. Both are properties of
            // the subdivision, and a change to its budget would quietly turn this
            // case into a test of nothing.
            expect(targetText.slice(
              first.target
                .endOffset,
              second.target
                .startOffset,
            ),).toBe('\n\n',);

            /**
             Paragraphs of the later slice, whose first one moves back.
             */
            const [moved, ...kept] = second.target
              .text
              .split('\n\n',);
            expect(kept.length,).toBeGreaterThan(0,);

            /**
             Both slices rewritten, each differing from its own incumbent, with
             one paragraph reassigned from the later to the earlier.
             */
            const replacements = [
              {
                sliceIndex: first.target
                  .sliceIndex,
                replacementText: `${
              first.target
                .text
            }\n\n${String(moved,)}`,
              },
              {
                sliceIndex: second.target
                  .sliceIndex,
                replacementText: kept.join('\n\n',),
              },
            ];
            for (const replacement of replacements) {
              /**
               Slice this replacement rewrites.
               */
              const slice = slices.find(function names(candidate,): boolean {
                return candidate.target
                  .sliceIndex === replacement.sliceIndex;
              },);
              expect(replacement.replacementText,).not
                .toBe(slice?.target.text,);
            }

            const guarded = guardFootnoteAssembly({
              targetText,
              slices,
              replacements,
            },);
            // The document the replacements would have produced IS the archive, so
            // there is nothing to ship and nothing to blame anyone for.
            expect(guarded.assembledText,).toBe(targetText,);
            expect(guarded.replacements,).toEqual([],);
            expect(byIndex({ indices: guarded.revertedChunkIndices, },),).toEqual([
              first.target
                .sliceIndex,
              second.target
                .sliceIndex,
            ],);
            expect(guarded.findings
              .some(function isCanonicalized(finding,): boolean {
                return finding.startsWith('assembly-net-zero-canonicalized',);
              },),).toBe(true,);
          },
        },),

        it({
          name: 'leaves an EMPTY replacement list alone rather than canonicalizing '
            + 'it, since a document nobody changed is the ordinary unchanged run '
            + 'and must not acquire a finding saying something was withdrawn',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [],
            },);
            expect(guarded.assembledText,).toBe(TARGET_TEXT,);
            expect(guarded.revertedChunkIndices,).toEqual([],);
            expect(guarded.findings,).toEqual([],);
          },
        },),

        it({
          name: 'keeps every replacement when the footnote graph survives, which '
            + 'is the ordinary case and the one a guard must not tax',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Replacement that renders the same footnote reference.
             */
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                {
                  sliceIndex: sliceCarrying({
                    slices,
                    needle: 'doing the sleeping',
                  },),
                  replacementText: 'The cat naps on the windowsill[^1].',
                },
              ],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([],);
            expect(guarded.assembledText,).toContain('naps on the windowsill[^1]',);
            expect(guarded.findings,).toEqual([],);
            expect(guarded.trimmed,).toEqual([],);
          },
        },),

        it({
          name: 'REVERTS the slice that dropped a reference, leaving its definition '
            + 'resolved. Nothing inside that slice was wrong: the sentence reads '
            + 'well, and the marker it lost belongs to a line in another slice',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Index of the slice carrying the reference.
             */
            const referring = sliceCarrying({
              slices,
              needle: 'doing the sleeping',
            },);

            /**
             Assembly where the reference was translated away.
             */
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                {
                  sliceIndex: referring,
                  replacementText: 'The cat naps on the windowsill.',
                },
              ],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([referring,],);
            expect(guarded.assembledText,).toBe(TARGET_TEXT,);
            expect(guarded.findings
              .some(function namesRevert(finding,): boolean {
                return finding.startsWith('assembly-footnote-reverted',);
              },),).toBe(true,);
          },
        },),

        it({
          name: 'TRIMS an orphan definition out of a definitions-only insertion '
            + 'and keeps the sibling the page needs, instead of withdrawing the '
            + 'whole insertion (the nineteenth hakureico pass of 2026-09-09 shipped '
            + 'a reference with no note that way)',
          fn: async () => {
            /**
             Archive whose body refers to a note it never defines.
             */
            const referring = 'The cat naps on the windowsill[^1].\n';

            /**
             One content slice plus the anchor where the notes belong.
             */
            const slices: readonly ChunkPair[] = [
              {
                source: {
                  kind: 'content',
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 12,
                  text: '猫猫在窗台上打盹〔1〕。',
                },
                target: {
                  kind: 'content',
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: referring.length,
                  text: referring,
                },
              },
              {
                source: {
                  kind: 'content',
                  sliceIndex: 1,
                  nodes: [],
                  startOffset: 14,
                  endOffset: 40,
                  text: '〔1〕：那是它最喜欢的位置。\n\n〔2〕：一只麻雀。',
                },
                target: makeInsertionChunk({
                  sliceIndex: 1,
                  offset: referring.length,
                },),
              },
            ];
            const guarded = guardFootnoteAssembly({
              targetText: referring,
              slices,
              replacements: [{
                sliceIndex: 1,
                replacementText: '[^1]: That is its favourite spot.\n\n[^2]: A sparrow.\n',
              },],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([],);
            expect(guarded.assembledText,).toContain('[^1]: That is its favourite spot.',);
            expect(guarded.assembledText,).not.toContain('[^2]',);
            expect(guarded.replacements,).toEqual([{
              sliceIndex: 1,
              replacementText: '[^1]: That is its favourite spot.\n',
            },],);
            expect(guarded.findings,).toEqual([
              'assembly-footnote-trimmed orphan-definition gfm 2 (slice 1)',
            ],);
            expect(guarded.trimmed,).toEqual([{
              sliceIndex: 1,
              replacementText: '[^1]: That is its favourite spot.\n',
            },],);
          },
        },),

        it({
          name: 'TRIMS the orphan out of two notes one line apart, the shape the bench writes (the twenty-first '
            + 'hakureico pass of 2026-09-09 withdrew both notes and the reference over it)',
          fn: async () => {
            /**
             Archive whose body refers to a note it never defines.
             */
            const referring = 'The cat naps on the windowsill[^1].\n';

            /**
             One content slice plus the anchor where the notes belong.
             */
            const slices: readonly ChunkPair[] = [
              {
                source: {
                  kind: 'content',
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 12,
                  text: '猫猫在窗台上打盹〔1〕。',
                },
                target: {
                  kind: 'content',
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: referring.length,
                  text: referring,
                },
              },
              {
                source: {
                  kind: 'content',
                  sliceIndex: 1,
                  nodes: [],
                  startOffset: 14,
                  endOffset: 40,
                  text: '〔1〕：那是它最喜欢的位置。\n〔2〕：一只麻雀。',
                },
                target: makeInsertionChunk({
                  sliceIndex: 1,
                  offset: referring.length,
                },),
              },
            ];
            const guarded = guardFootnoteAssembly({
              targetText: referring,
              slices,
              replacements: [{
                sliceIndex: 1,
                replacementText: '[^1]: That is its favourite spot.\n[^2]: A sparrow.',
              },],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([],);
            expect(guarded.trimmed,).toEqual([{
              sliceIndex: 1,
              replacementText: '[^1]: That is its favourite spot.',
            },],);
            expect(guarded.assembledText,).not.toContain('[^2]',);
          },
        },),

        it({
          name: 'WITHDRAWS a prose replacement that carries an orphan definition '
            + 'among its paragraphs, since cutting a block out of judged prose '
            + 'would ship a text nobody judged',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Index of the slice carrying the reference.
             */
            const referring = sliceCarrying({
              slices,
              needle: 'doing the sleeping',
            },);
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [{
                sliceIndex: referring,
                replacementText: '## The cat\n\nThe cat naps on the windowsill[^1].\n\n[^9]: A note nothing points at.',
              },],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([referring,],);
            expect(guarded.assembledText,).toBe(TARGET_TEXT,);
            expect(guarded.findings,).toEqual([
              'assembly-footnote-reverted orphan-definition gfm 9 (round 1)',
            ],);
          },
        },),

        it({
          name: 'WITHDRAWS ONLY THE CARRIER WHOSE ORIGINAL DOES NOT DEFINE THE NOTE when a definition is '
            + 'doubled, keeping the rendering of the slice that owns it (class forty-nine: shi_Yumiaoya3 '
            + 'withdrew the opening section and the definition alike, and the page lost both)',
          fn: async () => {
            /**
             Archive that never rendered the note, so both carriers are changes.
             */
            const skeleton = TARGET_TEXT.replace("[^1]: That is its favourite spot.\n", '',);

            /**
             Slices of the pair over that archive.
             */
            const slices = fixtureSlices({ targetText: skeleton, },);

            /**
             Index of the slice carrying the reference.
             */
            const referring = sliceCarrying({
              slices,
              needle: 'doing the sleeping',
            },);

            /**
             Index of the slice carrying the definition.
             */
            const defining = sliceCarrying({
              slices,
              needle: '## Notes',
            },);

            /**
             Assembly where the referring slice wrote the definition in as well
             and the defining slice rendered its own.
             */
            const guarded = guardFootnoteAssembly({
              targetText: skeleton,
              slices,
              replacements: [
                {
                  sliceIndex: referring,
                  replacementText: 'The cat naps on the windowsill[^1].\n\n[^1]: That is its favourite spot.',
                },
                {
                  sliceIndex: defining,
                  replacementText: '## Notes\n\n[^1]: That is its favourite spot indeed.',
                },
              ],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([referring,],);
            expect(guarded.assembledText.includes('favourite spot indeed',),).toBe(true,);
          },
        },),

        it({
          name: 'WITHDRAWS EVERY CARRIER OF A DOUBLED DEFINITION when every carrier\'s original defines the note, '
            + 'since ownership then names no one to keep',
          fn: async () => {
            /**
             Original that defines the note in the opening section as well as in
             the notes, each on a line of its own.
             */
            const doubledSource = SOURCE_TEXT.replace('打盹〔1〕。\n', '打盹〔1〕。\n〔1〕：那是它最喜欢的位置。\n',);

            /**
             Archive that never rendered the note, so both carriers are changes.
             */
            const skeleton = TARGET_TEXT.replace("[^1]: That is its favourite spot.\n", '',);

            /**
             Slices of the pair over that original and archive.
             */
            const { slices, } = prepareDocumentPair({
              sourceText: doubledSource,
              targetText: skeleton,
            },);

            /**
             Index of the slice carrying the reference.
             */
            const referring = sliceCarrying({
              slices,
              needle: 'doing the sleeping',
            },);

            /**
             Index of the slice carrying the definition.
             */
            const defining = sliceCarrying({
              slices,
              needle: '## Notes',
            },);
            const guarded = guardFootnoteAssembly({
              targetText: skeleton,
              slices,
              replacements: [
                {
                  sliceIndex: referring,
                  replacementText: 'The cat naps on the windowsill[^1].\n\n[^1]: That is its favourite spot.',
                },
                {
                  sliceIndex: defining,
                  replacementText: '## Notes\n\n[^1]: That is its favourite spot indeed.',
                },
              ],
            },);
            expect(doubledSource,).not.toBe(SOURCE_TEXT,);
            expect(byIndex({ indices: guarded.revertedChunkIndices, },),).toEqual(byIndex({ indices: [referring, defining,], },),);
            expect(guarded.assembledText,).toBe(skeleton,);
          },
        },),

        it({
          name: 'ITERATES TO A FIXPOINT, because one revert can orphan an '
            + 'identifier a DIFFERENT slice introduced alongside it. One pass '
            + 'reverts the slice that dropped [^1] and ships a [^2] definition '
            + 'whose only reference went with it',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Slice that referred to the footnote, and now renumbers it.
             */
            const referring = sliceCarrying({
              slices,
              needle: 'doing the sleeping',
            },);

            /**
             Slice with no footnote of its own, which gains the definition.
             */
            const bird = sliceCarrying({
              slices,
              needle: 'there is being a bird',
            },);

            /**
             Assembly where one slice renumbered its reference and another
             supplied the matching definition.
             */
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                {
                  sliceIndex: referring,
                  replacementText: 'The cat naps on the windowsill[^2].',
                },
                {
                  sliceIndex: bird,
                  replacementText: 'A bird sits on the windowsill.\n\n'
                    + '[^2]: That is its favourite spot.',
                },
              ],
            },);
            // The first pass sees [^1] orphaned and reverts the renumbering. That
            // revert is what orphans [^2], which only a second pass can see.
            expect(byIndex({ indices: guarded.revertedChunkIndices, },),).toEqual(
              byIndex({ indices: [referring, bird,], },),
            );
            expect(guarded.assembledText,).toBe(TARGET_TEXT,);
          },
        },),

        it({
          name: 'lets a slice INTRODUCE a footnote pair, since a translation may '
            + 'restore a footnote the archive never rendered, and both halves '
            + 'landing together leave the graph whole',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Slice gaining a reference, and the one gaining its definition.
             */
            const bird = sliceCarrying({
              slices,
              needle: 'there is being a bird',
            },);

            /**
             Assembly introducing a complete new pair.
             */
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                {
                  sliceIndex: bird,
                  replacementText: 'A bird sits on the windowsill[^2].\n\n'
                    + '[^2]: A sparrow, most mornings.',
                },
              ],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([],);
            expect(guarded.assembledText,).toContain('[^2]: A sparrow',);
          },
        },),

        it({
          name: 'WITHDRAWS EVERY REPLACEMENT when the defect belongs to no slice. '
            + 'A stray comment opener masks markers document-wide, so the slice '
            + 'that wrote it never changed its own mention of anything; choosing a '
            + 'slice to blame would be a guess, and shipping a document the lane '
            + 'knowingly broke is worse than shipping the archive',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Assembly whose prose opens a comment it never closes, hiding the
             definition that follows it in another slice.
             */
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                {
                  sliceIndex: sliceCarrying({
                    slices,
                    needle: 'there is being a bird',
                  },),
                  replacementText: 'A bird sits on the windowsill. <!-- a note',
                },
              ],
            },);
            expect(guarded.assembledText,).toBe(TARGET_TEXT,);
            expect(guarded.replacements,).toEqual([],);
            expect(guarded.findings
              .some(function namesWithdrawal(finding,): boolean {
                return finding.startsWith('assembly-withdrew-every-replacement',);
              },),).toBe(true,);
          },
        },),

        it({
          name: 'BLAMES THE RIGHT SLICE for a footnote whose label is a WORD, not '
            + 'a number. Markdown folds `[^Note]` and `[^note]` together and mdast '
            + 'reports the folded spelling, while a scan of slice text sees what '
            + 'was written; before those two agreed, this document lost the '
            + 'innocent repair in another slice as well',
          fn: async () => {
            /**
             Translation whose footnote is labelled by a word, spelled one way at
             the reference and another at the definition. The parser reads them as
             one footnote, which is exactly what makes the two spellings a hazard
             rather than a broken document.
             */
            const wordLabelled = TARGET_TEXT.replace(
              '[^1].',
              '[^Note].',
            )
              .replace(
              '[^1]:',
              '[^note]:',
              );

            /**
             Slices of the fixture pair, prepared against the word-labelled
             translation.
             */
            const slices = fixtureSlices({ targetText: wordLabelled, },);

            /**
             Assembly where one replacement drops the reference and another,
             touching no footnote at all, tidies an unrelated slice.
             */
            const guarded = guardFootnoteAssembly({
              targetText: wordLabelled,
              slices,
              replacements: [
                {
                  sliceIndex: sliceCarrying({
                    slices,
                    needle: 'doing the sleeping',
                  },),
                  replacementText: 'The cat naps on the windowsill.',
                },
                {
                  sliceIndex: sliceCarrying({
                    slices,
                    needle: 'there is being a bird',
                  },),
                  replacementText: 'A bird sits on the windowsill.',
                },
              ],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([
              sliceCarrying({
                slices,
                needle: 'doing the sleeping',
              },),
            ],);
            // The point of attributing at all: an edit that broke nothing ships.
            expect(guarded.assembledText,).toContain('A bird sits on the windowsill.',);
            expect(guarded.findings
              .some(function namesBlanketWithdrawal(finding,): boolean {
                return finding.startsWith('assembly-withdrew-every-replacement',);
              },),).toBe(false,);
          },
        },),

        it({
          name: 'reads an unterminated comment and an MDX downgrade as structural '
            + 'regressions, which name no footnote identifier and so can be found '
            + 'no other way',
          fn: async () => {
            expect(introducedStructuralRegressions({
              incumbentText: 'A settled paragraph.\n',
              assembledText: 'A settled paragraph. <!-- never closed\n',
            },),).toContain('unterminated-html-comment',);
            expect(introducedStructuralRegressions({
              incumbentText: 'A settled paragraph.\n',
              assembledText: 'A settled paragraph.\n',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'does NOT blame the lane for a defect the archive already had: an '
            + 'incumbent whose definition nothing references keeps that state, and '
            + 'no replacement is reverted for it',
          fn: async () => {
            /**
             Translation whose footnote definition is already orphaned.
             */
            const orphaned = `## The cat

The cat is doing the sleeping on the windowsill.

## The bird

On the windowsill there is being a bird.

## Notes

[^1]: That is its favourite spot.
`;

            /**
             Slices of that pair.
             */
            const slices = fixtureSlices({ targetText: orphaned, },);

            /**
             Assembly that changes prose and leaves the orphan alone.
             */
            const guarded = guardFootnoteAssembly({
              targetText: orphaned,
              slices,
              replacements: [
                {
                  sliceIndex: sliceCarrying({
                    slices,
                    needle: 'doing the sleeping',
                  },),
                  replacementText: 'The cat naps on the windowsill.',
                },
              ],
            },);
            expect(guarded.revertedChunkIndices,).toEqual([],);
            expect(guarded.assembledText,).toContain('The cat naps on the windowsill.',);
          },
        },),

        it({
          name: 'REFUSES a replacement that repeats its own slice verbatim, rather '
            + 'than reading it as a net-zero assembly. Both reassemble to the '
            + 'archive text, and only one of them is a run where somebody chose '
            + 'something: canonicalizing a no-op would hand back an empty surviving '
            + 'set no reader can tell from an honest one. The lanes check this '
            + 'before calling, so what this covers is every caller that does not',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Slice whose incumbent the replacement will repeat.
             */
            const repeated = slices.find(function isBirdSlice(slice,): boolean {
              return slice.target
                .text
                .includes('there is being a bird',);
            },);
            if (repeated === undefined)
              throw new Error('fixture lost its bird slice',);

            /**
             Failure the guard raised.
             */
            let caught: unknown;
            try {
              guardFootnoteAssembly({
                targetText: TARGET_TEXT,
                slices,
                replacements: [{
                  sliceIndex: repeated.target
                    .sliceIndex,
                  replacementText: repeated.target
                    .text,
                },],
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(AssemblyContractError,);
            expect(String(caught,),).toContain('carries the archive wording',);
          },
        },),

        it({
          name: 'RECORDS a structural regression the same round it is SEEN, even '
            + 'when a footnote took the blame for the withdrawal. Withdrawing that '
            + 'slice answers both defects at once here, so no later round sees the '
            + 'regression, and before this the document reported only the footnote '
            + 'while the parse damage left no trace. The finding says OBSERVED '
            + 'rather than reverted, because what this round withdraws is what an '
            + 'identifier named, and whether that answers the regression too is '
            + 'unknown until the next round',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);

            /**
             Assembly whose one replacement drops the footnote reference AND opens
             an MDX expression it never closes: the footnote names an identifier
             this slice stopped mentioning, so attribution succeeds and the
             blanket-withdrawal branch never runs.
             */
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [{
                sliceIndex: sliceCarrying({
                  slices,
                  needle: 'doing the sleeping',
                },),
                replacementText: 'The cat naps on the windowsill {\'unclosed',
              },],
            },);
            expect(guarded.assembledText,).toBe(TARGET_TEXT,);
            expect(guarded.findings
              .some(function namesFootnote(finding,): boolean {
                return finding.startsWith('assembly-footnote-reverted',);
              },),).toBe(true,);
            expect(guarded.findings
              .some(function namesStructure(finding,): boolean {
                return finding.startsWith('assembly-structure-observed mdx-downgraded',);
              },),).toBe(true,);
            // Attributed, so the blanket withdrawal never ran: the distinction
            // matters because that branch already reported structural damage.
            expect(guarded.findings
              .some(function namesBlanketWithdrawal(finding,): boolean {
                return finding.startsWith('assembly-withdrew-every-replacement',);
              },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: 'two structural breaks in two slices (class fifty-eight, XingZ607)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WITHDRAWS THE BROKEN SLICES ONE ROUND AT A TIME BY THE PARSER\'S POSITION and keeps the rest, '
            + 'instead of withdrawing every replacement because no single withdrawal repairs a page with two breaks '
            + '(XingZ607: two lone closing tags, 88 slices withdrawn)',
          fn: async () => {
            /**
             Slices of the fixture pair.
             */
            const slices = fixtureSlices({ targetText: TARGET_TEXT, },);
            /**
             Slice carrying the cat sentence, broken by an unclosed expression.
             */
            const cat = sliceCarrying({ slices, needle: 'doing the sleeping', },);
            /**
             Slice carrying the bird sentence, broken the same way.
             */
            const bird = sliceCarrying({ slices, needle: 'there is being a bird', },);
            /**
             Slice carrying the note, reworded and whole.
             */
            const note = sliceCarrying({ slices, needle: '[^1]:', },);
            const guarded = guardFootnoteAssembly({
              targetText: TARGET_TEXT,
              slices,
              replacements: [
                {
                  sliceIndex: cat,
                  replacementText: 'The cat naps on the windowsill[^1]. {\'unclosed',
                },
                {
                  sliceIndex: bird,
                  replacementText: 'A bird sits on the windowsill. {\'unclosed',
                },
                {
                  sliceIndex: note,
                  replacementText: '[^1]: That is the spot it likes best.',
                },
              ],
            },);
            expect(byIndex({ indices: guarded.revertedChunkIndices, },),).toEqual(byIndex({ indices: [cat, bird,], },),);
            expect(guarded.replacements
              .map(function toIndex(replacement,): number {
                return replacement.sliceIndex;
              },),).toEqual([note,],);
            expect(guarded.assembledText,).toContain('the spot it likes best',);
            expect(guarded.findings
              .some(function namesAdvance(finding,): boolean {
                return finding.startsWith(`assembly-structure-advancing-withdrawal slice ${String(cat,)}`,);
              },),).toBe(true,);
            expect(guarded.findings
              .some(function namesBlanketWithdrawal(finding,): boolean {
                return finding.startsWith('assembly-withdrew-every-replacement',);
              },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: introducedFootnoteFindings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPENDS ONE INHERITED DEFECT ON ONE ASSEMBLED DEFECT, so a second duplicate definition of '
            + 'an identifier the archive already duplicated is still reported, which is what counting '
            + 'rather than set-differencing was for',
          fn: async () => {
            /**
             Archive already defining one identifier twice, which is one defect.
             */
            const incumbentText = 'Mittens naps.[^1]\n\n[^1]: Since spring.\n\n[^1]: Since the spring.';

            /**
             Assembly defining it a third time, which is two defects of that key.
             */
            const assembledText =
              'Mittens naps.[^1]\n\n[^1]: Since spring.\n\n[^1]: Since the spring.\n\n[^1]: Every spring.';

            /**
             Defects the assembly added beyond what the archive carried.
             */
            const introduced = introducedFootnoteFindings({
              incumbentText,
              assembledText,
            },);

            expect(introduced.length,).toBe(1,);
            expect(introduced[0]?.kind,).toBe('duplicate-definition',);
            expect(introduced[0]?.identifier,).toBe('1',);
          },
        },),
        it({
          name: 'REPORTS NOTHING when the assembly carries exactly the defects the archive already had, '
            + 'which is the control that makes the case above a count rather than an alarm on any defect',
          fn: async () => {
            /**
             Same duplicated identifier on both sides, carried through unchanged.
             */
            const carried = 'Mittens naps.[^1]\n\n[^1]: Since spring.\n\n[^1]: Since the spring.';

            expect(introducedFootnoteFindings({
              incumbentText: carried,
              assembledText: carried,
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
