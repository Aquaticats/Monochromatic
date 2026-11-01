/**
 Tests for what a roster's coverage replies add up to.

 The asymmetry these pin: coverage can be proven, by pointing at the English
 carrying the passage, and absence cannot be, since nothing exhibits text that
 is not there. A claim nobody can anchor is therefore neither proof of coverage
 nor a vote for absence.

 Fixtures mirror corpus structure only. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage';
import {
  type CoverageReportWire,
  isCoverageReportWire,
  judgeCoverage,
} from '../dist/final/node/index.mjs';

/**
 Translation every quote is checked against.
 */
const TARGET_TEXT = `The cat sleeps on the windowsill each morning and naps on its cushion at noon.

She watches the birds outside.
`;

/**
 That translation as an anchor target.
 */
const TARGET = {
  text: TARGET_TEXT,
  nodes: parseDocument({ text: TARGET_TEXT, },).nodes,
};

/**
 Builds one heard voice carrying a coverage reply.

 @param modelId - roster member the reply came from

 @param coverage - degree it claims

 @param quote - English it points at

 @returns Voice shaped as the roster returns them

 @example
 ```ts
 const voice = voiceOf({ modelId: 'hf:cat/Cat-A' as RosterModelId, coverage: 'none', quote: '', },);
 ```
 */
function voiceOf(
  {
    modelId,
    coverage,
    quote,
  }: {
    readonly modelId: RosterModelId;
    readonly coverage: CoverageReportWire['coverage'];
    readonly quote: string;
  },
): HeardVoice<CoverageReportWire> {
  return {
    modelId,
    value: {
      coverage,
      copiedSpan: quote,
      assessment: 'fixture',
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: judgeCoverage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports CARRIED when a majority point at English the document really holds, which is the '
            + 'shape of a passage the translation merged into a neighbouring sentence rather than omitted',
          fn: async () => {
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'full',
                  quote: 'naps on its cushion at noon',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-B' as RosterModelId,
                  coverage: 'partial',
                  quote: 'The cat sleeps on the windowsill each morning',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-C' as RosterModelId,
                  coverage: 'full',
                  quote: 'She watches the birds outside.',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-D' as RosterModelId,
                  coverage: 'full',
                  quote: 'The cat sleeps on the windowsill each morning and naps on its cushion at noon.',
                },),
              ],
              document: TARGET,
              asked: 4,
              quorumMet: true,
            },);
            expect(verdict.kind,).toBe('carried',);
            expect(verdict.anchoredFull,).toBe(3,);
            expect(verdict.anchoredPartial,).toBe(1,);
            expect(verdict.absent,).toBe(0,);
            // THE THREE FULL VOTES ARE THE EVIDENCE; the partial voter's quote is not.
            expect(verdict.evidence
              .length,).toBe(3,);
          },
        },),
        it({
          name: 'reports ABSENT when a majority find nothing, which is the only evidence absence can ever '
            + 'have: no reader can exhibit text that is not there',
          fn: async () => {
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'none',
                  quote: '',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-B' as RosterModelId,
                  coverage: 'none',
                  quote: '',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-C' as RosterModelId,
                  coverage: 'full',
                  quote: 'She watches the birds outside.',
                },),
              ],
              document: TARGET,
              asked: 6,
              quorumMet: true,
            },);
            // NOT absent: two of six asked is not a majority of the roster, however
            // large a share of the voices heard it is.
            expect(verdict.kind,).toBe('split',);
            expect(verdict.absent,).toBe(2,);
            expect(verdict.anchoredFull,).toBe(1,);
          },
        },),
        it({
          name: 'DROPS a claim quoting English the document does not hold, and does not count it for '
            + 'absence either: a bad quote is a voice that answered unusably, and reading it as agreement '
            + 'with "nothing carries this" would turn an invented quote into a reason to insert text',
          fn: async () => {
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'full',
                  quote: 'The kitten chases butterflies in the yard.',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-B' as RosterModelId,
                  coverage: 'full',
                  quote: 'She purrs by the fire.',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-C' as RosterModelId,
                  coverage: 'none',
                  quote: '',
                },),
              ],
              document: TARGET,
              asked: 6,
              quorumMet: true,
            },);
            expect(verdict.unanchored,).toBe(2,);
            expect(verdict.anchoredFull,).toBe(0,);
            expect(verdict.unanchoredQuotes
              .length,).toBe(2,);
            expect(verdict.absent,).toBe(1,);
            // NOT absent, though the only usable voice said so: one voice of three
            // is not a majority, and two unusable answers do not make it one.
            expect(verdict.kind,).toBe('split',);
          },
        },),
        it({
          name: 'reports SPLIT when neither side reaches a majority of the roster, so a passage the '
            + 'roster disagrees about is never inserted on a plurality',
          fn: async () => {
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'full',
                  quote: 'She watches the birds outside.',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-B' as RosterModelId,
                  coverage: 'none',
                  quote: '',
                },),
              ],
              document: TARGET,
              asked: 6,
              quorumMet: true,
            },);
            expect(verdict.kind,).toBe('split',);
          },
        },),
        it({
          name: 'anchors a quote that differs only in curly versus straight punctuation, since a model '
            + 'copying English out of a prompt normalises quotation marks and an archive uses both',
          fn: async () => {
            /** Translation carrying a curly apostrophe. */
            const curlyText = 'The cat’s cushion is warm at noon.\n';
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'full',
                  quote: "The cat's cushion is warm at noon.",
                },),
              ],
              document: {
                text: curlyText,
                nodes: parseDocument({ text: curlyText, },).nodes,
              },
              asked: 1,
              quorumMet: true,
            },);
            expect(verdict.anchoredFull,).toBe(1,);
            expect(verdict.kind,).toBe('carried',);
          },
        },),

        it({
          name: 'anchors a quote copied across the place a note between translators was cut from, since the '
            + 'coverage sheet shows the translation without its notes and a quote of it is found there and '
            + 'nowhere in the stored text; the evidence it records is still text the document holds',
          fn: async () => {
            /** Translation whose first line ends with a note the archive\'s translators left each other. */
            const noteText = 'The cat naps on its cushion.{/* 这句不太对劲 */}\nShe wakes at noon.\n';
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'full',
                  quote: 'The cat naps on its cushion.\nShe wakes at noon.',
                },),
              ],
              document: {
                text: noteText,
                nodes: parseDocument({ text: noteText, },).nodes,
              },
              asked: 1,
              quorumMet: true,
              original: '猫在垫子上打盹，中午醒来。',
            },);
            expect(verdict.anchoredFull,).toBe(1,);
            expect(verdict.kind,).toBe('carried',);
            // THE EVIDENCE IS THE DOCUMENT'S OWN SLICE from the first anchor to the last, so it holds the note
            // between them: a reader of the stored text (`anchorRegion`) must find it there word for word.
            expect(verdict.evidence
              .at(0,),).toBe(noteText.trimEnd(),);
          },
        },),

        it({
          name: 'records the DOCUMENT\'S text as evidence rather than the quote it was sent, since a quote '
            + 'anchored across a soft wrap is by definition text the document does not hold literally, and '
            + 'evidence a reader cannot find is not evidence',
          fn: async () => {
            /** Translation whose sentence is wrapped mid-way, as the archive stores prose. */
            const wrappedText = 'The cat naps on its\ncushion at noon.\n';
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'full',
                  // Space-joined, which is how a model copies a wrapped sentence.
                  quote: 'naps on its cushion at noon',
                },),
              ],
              document: {
                text: wrappedText,
                nodes: parseDocument({ text: wrappedText, },).nodes,
              },
              asked: 1,
              quorumMet: true,
            },);
            expect(verdict.anchoredFull,).toBe(1,);
            expect(verdict.evidence
              .at(0,),).toBe('naps on its\ncushion at noon',);
            expect(wrappedText.includes(verdict.evidence
              .at(0,) ?? '',),).toBe(true,);
          },
        },),
        it({
          name: 'records only the FULL votes\' regions as evidence: a partial voter quotes what it found instead, '
            + 'which may sit anywhere on the page, and a lane rewriting that sentence must not stop the entry '
            + '(one entry\'s sixth pass of 2026-09-09)',
          fn: async () => {
            /** Translation carrying the credits and an unrelated sentence. */
            const pageText = 'She had chased the red dot many times before.\n\nContributors: Tuantuan, Doudou.\n';
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'full',
                  quote: 'Contributors: Tuantuan, Doudou.',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-B' as RosterModelId,
                  coverage: 'full',
                  quote: 'Contributors: Tuantuan, Doudou.',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-C' as RosterModelId,
                  coverage: 'partial',
                  quote: 'She had chased the red dot many times before.',
                },),
              ],
              document: {
                text: pageText,
                nodes: parseDocument({ text: pageText, },).nodes,
              },
              asked: 3,
              quorumMet: true,
            },);
            expect(verdict.kind,).toBe('carried',);
            expect(verdict.anchoredPartial,).toBe(1,);
            expect(verdict.evidence,).toStrictEqual([
              'Contributors: Tuantuan, Doudou.',
              'Contributors: Tuantuan, Doudou.',
            ],);
          },
        },),
        it({
          name: 'REFUSES to decide when quorum was not met, however the answers fell: a roster that mostly '
            + 'went silent has not examined the translation, and a verdict from what is left reads as an '
            + 'examination that happened',
          fn: async () => {
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'none',
                  quote: '',
                },),
              ],
              document: TARGET,
              asked: 6,
              quorumMet: false,
            },);
            expect(verdict.kind,).toBe('inconclusive',);
            expect(verdict.heard,).toBe(1,);
            expect(verdict.asked,).toBe(6,);
          },
        },),
        it({
          name: 'keeps PARTLY-CARRIED apart from carried, since a section with one rendered fact and the '
            + 'rest missing must not be inserted whole and must not be recorded as translated either',
          fn: async () => {
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'partial',
                  quote: 'She watches the birds outside.',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-B' as RosterModelId,
                  coverage: 'partial',
                  quote: 'naps on its cushion at noon',
                },),
                voiceOf({
                  modelId: 'hf:cat/Cat-C' as RosterModelId,
                  coverage: 'none',
                  quote: '',
                },),
              ],
              document: TARGET,
              asked: 3,
              quorumMet: true,
            },);
            expect(verdict.kind,).toBe('partly-carried',);
            expect(verdict.anchoredPartial,).toBe(2,);
            expect(verdict.anchoredFull,).toBe(0,);
          },
        },),
        it({
          name: 'counts a majority of the ROSTER rather than of the voices heard, so silence cannot lower '
            + 'the bar: one voice reporting nothing, with five models lost, used to decide absence',
          fn: async () => {
            const verdict = judgeCoverage({
              voices: [
                voiceOf({
                  modelId: 'hf:cat/Cat-A' as RosterModelId,
                  coverage: 'none',
                  quote: '',
                },),
              ],
              document: TARGET,
              asked: 6,
              quorumMet: true,
            },);
            expect(verdict.kind,).toBe('split',);
            expect(verdict.absent,).toBe(1,);
          },
        },),
      ],
    },),
    describe({
      name: isCoverageReportWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a claim of coverage with no quote, rather than counting it: the quote is the only '
            + 'part of this reply anything can check, so a reply without one carries no evidence and the '
            + 'roster should ask that model again rather than record it as answered',
          fn: async () => {
            expect(isCoverageReportWire({
              coverage: 'full',
              copiedSpan: '',
              assessment: 'it is in there somewhere',
            },),).toBe(false,);
            expect(isCoverageReportWire({
              coverage: 'partial',
              copiedSpan: '   ',
              assessment: 'whitespace is not evidence',
            },),).toBe(false,);
            // Nor is a quote no reader sees, which trim() keeps (ledger B40).
            expect(isCoverageReportWire({
              coverage: 'full',
              copiedSpan: '\u{3164}',
              assessment: 'a filler is not evidence',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES a report of no coverage that still quotes something, since the two fields '
            + 'contradict each other and neither can be trusted over the other',
          fn: async () => {
            expect(isCoverageReportWire({
              coverage: 'none',
              copiedSpan: 'She watches the birds outside.',
              assessment: 'contradicts itself',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'accepts the two shapes that agree with themselves, and refuses a degree outside the three '
            + 'the sheet asks for',
          fn: async () => {
            expect(isCoverageReportWire({
              coverage: 'none',
              copiedSpan: '',
              assessment: 'nothing renders it',
            },),).toBe(true,);
            // A quote showing nothing quotes nothing, so it agrees with no coverage
            // (ledger B40).
            expect(isCoverageReportWire({
              coverage: 'none',
              copiedSpan: '\u{200B}',
              assessment: 'nothing renders it',
            },),).toBe(true,);
            expect(isCoverageReportWire({
              coverage: 'full',
              copiedSpan: 'She watches the birds outside.',
              assessment: 'rendered here',
            },),).toBe(true,);
            expect(isCoverageReportWire({
              coverage: 'mostly',
              copiedSpan: 'She watches the birds outside.',
              assessment: 'not one of the three',
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
