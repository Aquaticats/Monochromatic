/**
 Heading-boundary windows preserve one following body without crossing another section.

 @module
 */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  neighbouringIncumbent,
  neighbouringSource,
  sliceNeighbourContexts,
} from '../dist/final/node/index.mjs';

/** Nonconsecutive stamps keep array positions distinct from result keys. */
const STAMP_STEP = 10;

/**
 Builds independently parsed source slices and distinguishable archive counterparts.

 @param texts - physical slice contents in document order
 @returns Paired fixtures whose stamps cannot be mistaken for positions
 @example
 ```ts
 const slices = pairs(['Current.', '## Friends', 'A friend returned.']);
 ```
 */
function pairs(texts: readonly string[]): readonly ChunkPair[] {
  return texts.map(function pair(text, index): ChunkPair {
    /** Actual parser nodes identify headings without guessing from string prefixes. */
    const source = parseDocument({ text, });
    /** Distinct archive text reveals whether both sides selected the same positions. */
    const archive = `archive ${String(index,)}`;
    return {
      source: { sliceIndex: index, startOffset: 0, endOffset: text.length, text, nodes: source.nodes, },
      target: {
        sliceIndex: (index + 1) * STAMP_STEP,
        startOffset: 0,
        endOffset: archive.length,
        text: archive,
        nodes: [],
      },
    };
  },);
}

await describe({
  name: 'heading-boundary fidelity context',
  children: [
    it({
      name: 'INCLUDES the dated body after the immediate heading on both sides of the pair',
      fn: async () => {
        const slices = pairs(['Current.', '## Friends', 'In April 2022 her friend came out to her.', 'Later unrelated prose.'],);
        const source = neighbouringSource({ slices, slicePosition: 0, },);
        const archive = neighbouringIncumbent({ slices, slicePosition: 0, },);
        expect(source,).toBe('## Friends\n\nIn April 2022 her friend came out to her.',);
        expect(archive,).toBe('archive 1\n\narchive 2',);
        expect(source,).not.toContain('Later unrelated',);
        expect(sliceNeighbourContexts({ slices, },).get(STAMP_STEP,),).toEqual({ sourceText: source, incumbentText: archive, },);
      },
    },),
    it({
      name: 'DOES NOT reach backward through the current section heading',
      fn: async () => {
        const slices = pairs(['Previous section body.', '## Current section', 'Current body.'],);
        expect(neighbouringSource({ slices, slicePosition: 2, },),).toBe('## Current section',);
        expect(neighbouringIncumbent({ slices, slicePosition: 2, },),).toBe('archive 1',);
      },
    },),
    it({
      name: 'COUNTS media as the following body and does not skip it to reach distant prose',
      fn: async () => {
        const media = '<PhotoScroll photos={["picture.webp"]} />';
        const slices = pairs(['Current.', '## Pictures', media, 'Distant prose.'],);
        expect(neighbouringSource({ slices, slicePosition: 0, },),).toBe(`## Pictures\n\n${media}`,);
        expect(neighbouringIncumbent({ slices, slicePosition: 0, },),).toBe('archive 1\n\narchive 2',);
      },
    },),
    ...['## Another section', '## Another section\n\nIts body.', '',].map(function barrier(text) {
      return it({
        name: `STOPS after the first heading when the next slice is ${JSON.stringify(text,)}`,
        fn: async () => {
          const slices = pairs(['Current.', '## Boundary', text, 'Distant prose.'],);
          expect(neighbouringSource({ slices, slicePosition: 0, },),).toBe('## Boundary',);
          expect(neighbouringIncumbent({ slices, slicePosition: 0, },),).toBe('archive 1',);
        },
      },);
    },),
    it({
      name: 'ACCEPTS nonempty unknown-node content as one body without further searching',
      fn: async () => {
        const slices = pairs(['Current.', '## Boundary', 'Unknown but present body.', 'Distant prose.'],)
          .map(function removeNodes(slice, index): ChunkPair {
            return index === 2 ? { ...slice, source: { ...slice.source, nodes: [], }, } : slice;
          },);
        expect(neighbouringSource({ slices, slicePosition: 0, },),).toBe('## Boundary\n\nUnknown but present body.',);
        expect(neighbouringIncumbent({ slices, slicePosition: 0, },),).toBe('archive 1\n\narchive 2',);
      },
    },),
    it({
      name: 'KEEPS a metadata barrier outside the body window',
      fn: async () => {
        const slices = pairs(['Current.', '## Boundary', 'name: Cat', 'Distant prose.'],)
          .map(function metadata(slice, index): ChunkPair {
            return index === 2 ? { ...slice, syntax: 'front-matter', } : slice;
          },);
        expect(neighbouringSource({ slices, slicePosition: 0, },),).toBe('## Boundary',);
        expect(neighbouringIncumbent({ slices, slicePosition: 0, },),).toBe('archive 1',);
        expect(neighbouringSource({ slices, slicePosition: 2, },),).toBe('',);
        expect(neighbouringIncumbent({ slices, slicePosition: 2, },),).toBe('',);
      },
    },),
    it({
      name: 'DOES NOT infer a heading from unknown-node text or extend through immediate metadata',
      fn: async () => {
        const base = pairs(['Current.', '## Unproven heading', 'Following body.'],);
        const unknown = base.map(function unstructured(slice, index): ChunkPair {
          return index === 1 ? { ...slice, source: { ...slice.source, nodes: [], }, } : slice;
        },);
        expect(neighbouringSource({ slices: unknown, slicePosition: 0, },),).toBe('## Unproven heading',);
        const metadata = base.map(function mark(slice, index): ChunkPair {
          return index === 1 ? { ...slice, syntax: 'front-matter', } : slice;
        },);
        expect(neighbouringSource({ slices: metadata, slicePosition: 0, },),).toBe('',);
        expect(neighbouringIncumbent({ slices: metadata, slicePosition: 0, },),).toBe('',);
      },
    },),
    it({
      name: 'PRESERVES ordinary neighbors and a final heading without a following body',
      fn: async () => {
        const slices = pairs(['Before.', 'Current.', 'After.', 'Farther.'],);
        expect(neighbouringSource({ slices, slicePosition: 1, },),).toBe('Before.\n\nAfter.',);
        expect(neighbouringIncumbent({ slices, slicePosition: 1, },),).toBe('archive 0\n\narchive 2',);
        const lastHeading = pairs(['Current.', '## End'],);
        expect(neighbouringSource({ slices: lastHeading, slicePosition: 0, },),).toBe('## End',);
      },
    },),
  ],
},);
