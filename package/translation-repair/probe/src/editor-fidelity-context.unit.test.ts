/**
 Tests for the context the editor fidelity probe puts on its sheet: the sections either side of the passage, the
 declared names, and the whole original, prepared from the pages a reviewed reference was cut from by the same
 preparation and the same window readers the pipeline uses. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  editorSheetContextOf,
  FidelityReferenceError,
} from '../dist/final/node/index.mjs';
import {
  HEADED_ARCHIVE_AFTER,
  HEADED_ARCHIVE_BEFORE,
  HEADED_ARCHIVE_HEADING,
  HEADED_SOURCE_AFTER,
  HEADED_SOURCE_BEFORE,
  HEADED_SOURCE_HEADING,
  headedReference,
  inventedReference,
} from './editor-fidelity.test-fixture.ts';

await describe({
  name: editorSheetContextOf.name,
  children: [
    it({
      name: 'TAKES THE SLICE BEFORE AND THE SLICE AFTER THE PASSAGE as the neighbouring original and the neighbouring '
        + 'translation, joined by a blank line, as the pipeline\'s window readers give them: for a passage below '
        + 'its heading the slice before is the heading, as it is for the first paragraph of a section in production',
      fn: async () => {
        const context = editorSheetContextOf({ reference: headedReference(), },);
        expect(context.neighbouringSourceText,).toBe(`${HEADED_SOURCE_HEADING}\n\n${HEADED_SOURCE_AFTER}`,);
        expect(context.neighbouringIncumbentText,).toBe(`${HEADED_ARCHIVE_HEADING}\n\n${HEADED_ARCHIVE_AFTER}`,);
      },
    },),

    it({
      name: 'TAKES THE SECTION BEFORE AND THE SECTION AFTER when the passage range covers its heading too, so the '
        + 'slices over the passage count as one position and the neighbours lie outside it',
      fn: async () => {
        /**
         The headed reference with its passage range moved up to the heading.
         */
        const reference = headedReference();
        const headingAt = reference.pages.archiveFile.indexOf(HEADED_ARCHIVE_HEADING,);
        const context = editorSheetContextOf({
          reference: {
            ...reference,
            spec: {
              ...reference.spec,
              archive: {
                ...reference.spec.archive,
                startOffset: headingAt,
              },
            },
          },
        },);
        expect(context.neighbouringSourceText,).toBe(`${HEADED_SOURCE_BEFORE}\n\n${HEADED_SOURCE_AFTER}`,);
        expect(context.neighbouringIncumbentText,).toBe(`${HEADED_ARCHIVE_BEFORE}\n\n${HEADED_ARCHIVE_AFTER}`,);
      },
    },),

    it({
      name: 'HANDS THE WHOLE ORIGINAL PAGE as the document the preservation gate reads markup against, not the passage',
      fn: async () => {
        const reference = headedReference();
        expect(editorSheetContextOf({ reference, },).documentSourceText,).toBe(reference.pages.sourceFile,);
        expect(reference.pages.sourceFile.length,).toBeGreaterThan(reference.sourceText.length,);
      },
    },),

    it({
      name: 'CARRIES THE IDENTITY CONTEXT the preparation reads off the pair, which names who the original is about: '
        + 'the pronoun line of the invented page',
      fn: async () => {
        const { identityContext, } = editorSheetContextOf({ reference: headedReference(), },);
        expect(identityContext,).toContain('pronoun: ORIGINAL refers to this person as',);
      },
    },),

    it({
      name: 'HOLDS NO NEIGHBOUR when the passage is on a page of one section, as a lone slice has none, and says so '
        + 'with empty texts rather than failing',
      fn: async () => {
        const context = editorSheetContextOf({ reference: inventedReference(), },);
        expect(context.neighbouringSourceText,).toBe('',);
        expect(context.neighbouringIncumbentText,).toBe('',);
      },
    },),

    it({
      name: 'REFUSES A PASSAGE NO SECTION OF THE PAGE HOLDS, whose neighbours could only be guessed: an empty window '
        + 'would read as a slice with no neighbours',
      fn: async () => {
        const reference = headedReference();
        expect(function asksOutsidePage(): void {
          editorSheetContextOf({
            reference: {
              ...reference,
              spec: {
                ...reference.spec,
                archive: {
                  ...reference.spec.archive,
                  startOffset: reference.pages.archiveFile.length + 10,
                  endOffset: reference.pages.archiveFile.length + 20,
                },
              },
            },
          },);
        },).toThrow(FidelityReferenceError,);
      },
    },),
  ],
},);
