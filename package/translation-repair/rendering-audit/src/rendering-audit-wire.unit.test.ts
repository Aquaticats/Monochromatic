/**
 Tests for what shape the rendering audit accepts off the network.

 SHAPE IS THE ONLY QUESTION HERE. A reply carrying words this version does not
 know is a voice that ANSWERED, and refusing it at the wire would file it as a
 lost voice instead, which is how a vocabulary problem disappears into the
 degradation rate. Whether the words are known, and whether the quotes prove
 anything, belongs to the screen.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  FINDING_FIELDS,
  isRenderingAuditReportWire,
  RENDERING_AUDIT_CATEGORIES,
  RENDERING_AUDIT_RESPONSE_FORMAT,
  RENDERING_AUDIT_VERDICTS,
} from '../dist/final/node/index.mjs';

/**
 Well-shaped reply, which the shape cases break one field at a time.
 */
const SOUND_REPLY = {
  verdict: 'defects-found',
  findings: [
    {
      category: 'altered-polarity',
      sourceLocator: '她们不吃罐头',
      sourceFocus: '不吃',
      candidateLocator: 'They eat canned food',
      candidateFocus: 'eat',
      reason: 'the original denies what the candidate asserts',
    },
  ],
};

await describe({
  name: isRenderingAuditReportWire.name,
  children: [
    it({
      name: 'ACCEPTS a reply carrying a verdict and a list of fully-formed findings',
      fn: async () => {
        expect(isRenderingAuditReportWire(SOUND_REPLY,),).toBe(true,);
      },
    },),
    it({
      name: 'ACCEPTS a reply that claims nothing, since an auditor finding no defect is an answer',
      fn: async () => {
        expect(
          isRenderingAuditReportWire({
            verdict: 'no-defect-found',
            findings: [],
          },),
        ).toBe(true,);
      },
    },),
    it({
      name:
        'ACCEPTS words this version does not know, because a reply that parsed and named an unknown '
          + 'category is a voice that answered: refusing it here would count it as a lost voice and hide a '
          + 'vocabulary problem inside the degradation rate',
      fn: async () => {
        expect(
          isRenderingAuditReportWire({
            verdict: 'catastrophic',
            findings: [
              {
                ...SOUND_REPLY.findings[0],
                category: 'altered-whiskers',
              },
            ],
          },),
        ).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES anything that is not a record, including the array and the bare string a model may answer with',
      fn: async () => {
        expect(isRenderingAuditReportWire(undefined,),).toBe(false,);
        expect(isRenderingAuditReportWire(null,),).toBe(false,);
        expect(isRenderingAuditReportWire('no-defect-found',),).toBe(false,);
        expect(isRenderingAuditReportWire([SOUND_REPLY,],),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES a reply with no verdict, or one that is not a string',
      fn: async () => {
        expect(isRenderingAuditReportWire({ findings: [], },),).toBe(false,);
        expect(
          isRenderingAuditReportWire({
            verdict: 2,
            findings: [],
          },),
        ).toBe(false,);
      },
    },),
    it({
      name:
        'REFUSES a reply whose findings are missing or not a list, including the single object a model '
          + 'sends when it found exactly one thing',
      fn: async () => {
        expect(isRenderingAuditReportWire({ verdict: 'no-defect-found', },),).toBe(false,);
        expect(
          isRenderingAuditReportWire({
            verdict: 'defects-found',
            findings: SOUND_REPLY.findings[0],
          },),
        ).toBe(false,);
      },
    },),
    it({
      name:
        'REFUSES a finding missing ANY of its six fields, so a claim never reaches the screen with a '
          + 'quote field the screen would read as a deliberately empty one',
      fn: async () => {
        for (const field of FINDING_FIELDS) {
          /**
           Sound finding with exactly one field taken out.
           */
          const partial = Object.fromEntries(
            Object.entries(SOUND_REPLY.findings[0] ?? {},)
              .filter(function keepOthers([key,],): boolean {
                return key !== field;
              },),
          );

          expect(
            isRenderingAuditReportWire({
              verdict: 'defects-found',
              findings: [partial,],
            },),
          ).toBe(false,);
        }
      },
    },),
    it({
      name: 'REFUSES a finding whose quote is a number rather than text, and one that is not a record at all',
      fn: async () => {
        expect(
          isRenderingAuditReportWire({
            verdict: 'defects-found',
            findings: [
              {
                ...SOUND_REPLY.findings[0],
                sourceFocus: 3,
              },
            ],
          },),
        ).toBe(false,);
        expect(
          isRenderingAuditReportWire({
            verdict: 'defects-found',
            findings: ['altered-polarity',],
          },),
        ).toBe(false,);
      },
    },),
    it({
      name:
        'NAMES every field it checks for in the response format too, since a schema and a guard that '
          + 'drift apart produce replies the provider considers valid and this reader then discards as '
          + 'malformed, which reads as model failure rather than as our own',
      fn: async () => {
        // OVER THE SERIALIZED FORM, because the response format is typed as an
        // opaque JSON schema and reaching into it would need an assertion this
        // codebase does not allow. Drift is what the case is for, and a field
        // absent from the schema is absent from its text.
        const asked = JSON.stringify(RENDERING_AUDIT_RESPONSE_FORMAT,);
        for (const field of FINDING_FIELDS)
          expect(asked.includes(`"${field}"`,),).toBe(true,);
      },
    },),
    it({
      name:
        'DECLARES and REQUIRES the keys of the reply and of each finding in alphabetical order, since Charm Hyper '
          + 'returns a tool call\'s keys alphabetically and the scan of `source-scan` cannot read a schema built from a list',
      fn: async () => {
        // OVER THE SERIALIZED FORM, which keeps the order the keys were declared in.
        const asked = JSON.stringify(RENDERING_AUDIT_RESPONSE_FORMAT,);
        const sorted = [
          'candidateFocus',
          'candidateLocator',
          'category',
          'reason',
          'sourceFocus',
          'sourceLocator',
        ];
        expect([...FINDING_FIELDS,],).toEqual(sorted,);
        expect(asked.includes('"required":["findings","verdict"]',),).toBe(true,);
        expect(asked.includes(`"required":${JSON.stringify(sorted,)}`,),).toBe(true,);
        // THE CATEGORY AND THE VERDICT LIST THE VOCABULARIES THE SCREEN READS AS AN enum (P1 of the second review of the
        // Claude route); the other fields are plain text.
        expect(asked.includes(
          `"properties":{${sorted.map(function declared(field,): string {
            return (field === 'category')
              ? `"${field}":{"type":"string","enum":${JSON.stringify(RENDERING_AUDIT_CATEGORIES,)}}`
              : `"${field}":{"type":"string"}`;
          },).join(',',)}}`,
        ),).toBe(true,);
        expect(asked.indexOf('"properties":{"findings":',),).toBeGreaterThan(-1,);
        expect(asked.indexOf(`"verdict":{"type":"string","enum":${JSON.stringify(RENDERING_AUDIT_VERDICTS,)}}`,),)
          .toBeGreaterThan(asked.indexOf('"findings":{"type":"array"',),);
      },
    },),
  ],
},);
