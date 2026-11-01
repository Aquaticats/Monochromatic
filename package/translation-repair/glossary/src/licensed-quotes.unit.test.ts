/**
 Tests for which quoted defect text an envelope is licensed to lose.

 WHY THIS FILE EXISTS. The preservation gate lets an edit delete exactly what
 its issues quoted as the defect; everything else in the replaced span has to
 survive. Three decisions inside that collection were measured on 2026-08-25
 to decide nothing any case asserts, and each fails in its own direction.

 A SOURCE-SIDE QUOTE licensed nothing, because it is Chinese prose that never
 appears in the English being edited, but it does widen the licence list an
 editor is handed. AN EMPTY QUOTE licenses the empty string, which every span
 contains, so the gate would wave through any deletion at all. AND A REPEATED
 QUOTE, filed by two issues about the same wording, would be carried twice and
 counted twice by anything reading the list as evidence.

 FIXTURES ARE CAST rather than fully built, following
 `translate-lane-wordings.unit.test.ts`: this function reads four fields off
 two large model types, and a faithful fixture would bury the case in
 ballots, tallies and offsets that decide nothing here.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { buildLicensedQuotes, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Builds one adjudicated issue quoting the given spans.

 @param issueId - identity envelopes name it by

 @param spans - side and wording of each quote it filed

 @returns Issue carrying exactly those quotes

 @example
 ```ts
 const issue = issueOf({ issueId: 'adjudicated/nap', spans: [], },);
 ```
 */
function issueOf(
  {
    issueId,
    spans,
  }: {
    readonly issueId: string;
    readonly spans: readonly {
      readonly side: string;
      readonly quotedText: string;
    }[];
  },
): AdjudicatedIssue {
  return {
    issueId,
    claims: [{ claim: { spans, }, },],
  } as unknown as AdjudicatedIssue;
}

/**
 Builds one envelope naming the issues it answers.

 @param issueIds - issues whose quotes it may lose

 @returns Envelope shaped as the editor stage passes one

 @example
 ```ts
 const envelope = envelopeOf({ issueIds: ['adjudicated/nap',], },);
 ```
 */
function envelopeOf(
  { issueIds, }: { readonly issueIds: readonly string[]; },
): EditableEnvelope {
  return {
    envelopeId: 'envelope/sill',
    issueIds,
  } as unknown as EditableEnvelope;
}

/**
 Reads the one envelope`s licence list out of the returned map.

 @param issues - issues the chunk carries

 @param issueIds - issues the envelope answers

 @returns Quotes licensed for that envelope

 @example
 ```ts
 const quotes = licensedFor({ issues, issueIds: ['adjudicated/nap',], },);
 ```
 */
function licensedFor(
  {
    issues,
    issueIds,
  }: {
    readonly issues: readonly AdjudicatedIssue[];
    readonly issueIds: readonly string[];
  },
): readonly string[] {
  return buildLicensedQuotes({
    envelopes: [envelopeOf({ issueIds, },),],
    issues,
  },)
    .get('envelope/sill',) ?? [];
}

//endregion Fixtures

await describe({
  name: buildLicensedQuotes.name,
  children: [
    it({
      name: 'KEEPS target-side quotes only, since a quote of the Chinese original never appears in the '
        + 'English being edited and would only widen the list of wordings an editor may delete',
      fn: async () => {
        expect(licensedFor({
          issues: [issueOf({
            issueId: 'adjudicated/nap',
            spans: [
              {
                side: 'source',
                quotedText: '金枪鱼',
              },
              {
                side: 'target',
                quotedText: 'the tuna',
              },
            ],
          },),],
          issueIds: ['adjudicated/nap',],
        },),).toEqual(['the tuna',],);
      },
    },),
    it({
      name: 'REFUSES an empty quote, which every span contains, so a claim that quoted nothing cannot '
        + 'license the deletion of anything at all',
      fn: async () => {
        expect(licensedFor({
          issues: [issueOf({
            issueId: 'adjudicated/nap',
            spans: [
              {
                side: 'target',
                quotedText: '',
              },
              {
                side: 'target',
                quotedText: 'the tuna',
              },
            ],
          },),],
          issueIds: ['adjudicated/nap',],
        },),).toEqual(['the tuna',],);
      },
    },),
    it({
      name: 'COLLAPSES one wording two issues both quoted into a single licence, so anything reading '
        + 'the list as evidence counts the defect once rather than once per filer',
      fn: async () => {
        expect(licensedFor({
          issues: [
            issueOf({
              issueId: 'adjudicated/nap',
              spans: [{
                side: 'target',
                quotedText: 'the tuna',
              },],
            },),
            issueOf({
              issueId: 'adjudicated/sill',
              spans: [{
                side: 'target',
                quotedText: 'the tuna',
              },],
            },),
          ],
          issueIds: [
            'adjudicated/nap',
            'adjudicated/sill',
          ],
        },),).toEqual(['the tuna',],);
      },
    },),
    it({
      name: 'LICENSES NOTHING for an issue the list does not hold, so the gate asks the edit to keep every '
        + 'wording the envelope replaces, and still licenses the quote of an issue it does hold',
      fn: async () => {
        expect(licensedFor({
          issues: [issueOf({
            issueId: 'adjudicated/nap',
            spans: [{
              side: 'target',
              quotedText: 'the tuna',
            },],
          },),],
          issueIds: [
            'adjudicated/missing',
            'adjudicated/nap',
          ],
        },),).toEqual(['the tuna',],);
        expect(licensedFor({
          issues: [],
          issueIds: ['adjudicated/missing',],
        },),).toEqual([],);
      },
    },),
  ],
},);
