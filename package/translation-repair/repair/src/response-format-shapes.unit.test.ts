/**
 Shape assertions for every `*_RESPONSE_FORMAT` structured-output constant.

 Each of these constrains what shape of reply a provider can even produce,
 ahead of the paired `is*Wire` guard that checks the parsed JSON a second
 time. That ordering is exactly why a schema drifting from its guard is
 dangerous rather than merely redundant: a required field dropped here, or
 an enum narrowed here, changes what the provider is asked to send, and the
 guard downstream only ever sees replies shaped the way THIS constant
 currently asks for. Nothing exercises these constants themselves anywhere
 else, unlike the guards, which every paired `*-wire.unit.test.ts` file
 already covers with scripted examples.

 One block per constant, each pinning its schema name, every field the wire
 depends on being required, and the enum values where a schema constrains
 one. No invented prose is needed: every assertion reads the shipped
 constant directly, so there is no corpus content and no fixture to invent.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  TRANSLATE_REPAIR_RESPONSE_FORMAT,
  TRANSLATE_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { COVERAGE_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-archive';
import {
  ADJUDICATION_RESPONSE_FORMAT,
  CANDIDATE_SELECT_RESPONSE_FORMAT,
  CRITIC_RESPONSE_FORMAT,
  INTRODUCED_DEFECT_RESPONSE_FORMAT,
  RESOLUTION_RESPONSE_FORMAT,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  EDITOR_RESPONSE_FORMAT,
  REFINE_RESPONSE_FORMAT,
  RESTORATION_JUDGE_RESPONSE_FORMAT,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: 'ADJUDICATION_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the panel_ballot schema and requires claim, reason and vote on every verdict, '
            + 'reason declared before vote (owner, 2026-09-27, "Reason before vote"), and group and '
            + 'sameDefect on every conflict group, so the panel cannot answer without naming both '
            + 'halves of either kind of entry',
          fn: async () => {
            expect(ADJUDICATION_RESPONSE_FORMAT.json_schema.name,).toBe('panel_ballot',);

            /**
             Schema body as JSON, so a required-field or enum change shows
             up as a substring that stopped matching rather than as a deep
             object diff.
             */
            const schema = JSON.stringify(ADJUDICATION_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["verdicts"]',);
            expect(schema,).toContain('"required":["claim","reason","vote"]',);
            expect(schema,).toContain('"required":["group","sameDefect"]',);
            // DECLARED FIRST, as the sheet's reply shape shows it: a decoder
            // held to the schema may write fields in declared order, and
            // nothing here should invite the vote ahead of its reason.
            expect(schema.indexOf('"reason":{',),).toBeGreaterThan(0,);
            expect(schema.indexOf('"reason":{',),).toBeLessThan(schema.indexOf('"vote":{',),);
          },
        },),
      ],
    },),
    describe({
      name: 'CANDIDATE_SELECT_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the candidate_ballot schema and requires assessment and best on every ballot, so a '
            + 'ballot cannot name a candidate without a reason attached, the reason declared before the '
            + 'candidate number (owner, 2026-10-21, "Reason first everywhere")',
          fn: async () => {
            expect(CANDIDATE_SELECT_RESPONSE_FORMAT.json_schema.name,).toBe('candidate_ballot',);
            const schema = JSON.stringify(CANDIDATE_SELECT_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["assessment","best"]',);
            expect(schema.indexOf('"assessment":{',),).toBeLessThan(schema.indexOf('"best":{',),);
          },
        },),
      ],
    },),
    describe({
      name: 'COVERAGE_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the coverage_report schema, requires assessment, copiedSpan and coverage, the reason '
            + 'declared before the verdict (owner, 2026-10-21, "Reason first everywhere"), and limits '
            + 'coverage to full, partial or none, so a wider or narrower degree could not reach the '
            + 'guard even mislabeled',
          fn: async () => {
            expect(COVERAGE_RESPONSE_FORMAT.json_schema.name,).toBe('coverage_report',);
            const schema = JSON.stringify(COVERAGE_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["assessment","copiedSpan","coverage"]',);
            expect(schema.indexOf('"assessment":{',),).toBeLessThan(schema.indexOf('"coverage":{',),);
            expect(schema,).toContain('"enum":["full","partial","none"]',);
          },
        },),
      ],
    },),
    describe({
      name: 'CRITIC_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the critic_report schema and requires category, severity and summary on every '
            + 'issue, so a filed issue cannot arrive without the three fields every downstream stage '
            + 'reads',
          fn: async () => {
            expect(CRITIC_RESPONSE_FORMAT.json_schema.name,).toBe('critic_report',);
            const schema = JSON.stringify(CRITIC_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["issues"]',);
            expect(schema,).toContain('"required":["category","severity","summary"]',);
          },
        },),
      ],
    },),
    describe({
      name: 'EDITOR_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the editor_report schema and requires region and newText on every edit, so a '
            + 'proposed edit cannot name a region without the replacement text the apply gate needs',
          fn: async () => {
            expect(EDITOR_RESPONSE_FORMAT.json_schema.name,).toBe('editor_report',);
            const schema = JSON.stringify(EDITOR_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["edits"]',);
            expect(schema,).toContain('"required":["newText","region"]',);
          },
        },),
      ],
    },),
    describe({
      name: 'INTRODUCED_DEFECT_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the introduced_defect_report schema and requires EVERY field on every check by '
            + 'design: category, evidence, omittedText, region, severity, verdict and verdictReason are all '
            + 'required rather than optional, since an optional field is where per-model structured '
            + 'output diverges most and a lost voice here costs a whole region\'s telemetry',
          fn: async () => {
            expect(INTRODUCED_DEFECT_RESPONSE_FORMAT.json_schema.name,).toBe('introduced_defect_report',);
            const schema = JSON.stringify(INTRODUCED_DEFECT_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["checks"]',);
            expect(schema,).toContain(
              '"required":["category","evidence","omittedText","region","severity","verdict","verdictReason"]',
            );
          },
        },),
      ],
    },),
    describe({
      name: 'REFINE_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the refine_report schema and requires paragraph and newText on every rewrite, '
            + 'so a rewrite cannot bind to a paragraph without carrying the replacement it stands for',
          fn: async () => {
            expect(REFINE_RESPONSE_FORMAT.json_schema.name,).toBe('refine_report',);
            const schema = JSON.stringify(REFINE_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["rewrites"]',);
            expect(schema,).toContain('"required":["newText","paragraph"]',);
          },
        },),
      ],
    },),
    describe({
      name: 'RESOLUTION_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the resolution_report schema and requires issue and verdict on every check, so '
            + 'a checker cannot answer without saying which issue the verdict is about',
          fn: async () => {
            expect(RESOLUTION_RESPONSE_FORMAT.json_schema.name,).toBe('resolution_report',);
            const schema = JSON.stringify(RESOLUTION_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["checks"]',);
            expect(schema,).toContain('"required":["issue","verdict"]',);
          },
        },),
      ],
    },),
    describe({
      name: 'RESTORATION_JUDGE_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the restoration_judgment schema and requires reference and verdict on every '
            + 'judgment, so a judgment cannot arrive without naming which reference it judges',
          fn: async () => {
            expect(RESTORATION_JUDGE_RESPONSE_FORMAT.json_schema.name,).toBe('restoration_judgment',);
            const schema = JSON.stringify(RESTORATION_JUDGE_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["judgments"]',);
            expect(schema,).toContain('"required":["reference","verdict"]',);
          },
        },),
      ],
    },),
    describe({
      name: 'TRANSLATE_REPAIR_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the translation_repair_report schema, requires resolution, translation and '
            + 'translatorNote, and limits resolution to revised, unable or as-intended, so a reply cannot '
            + 'claim a repair happened while withholding the text or the reasoning behind it',
          fn: async () => {
            expect(TRANSLATE_REPAIR_RESPONSE_FORMAT.json_schema.name,).toBe('translation_repair_report',);
            const schema = JSON.stringify(TRANSLATE_REPAIR_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["resolution","translation","translatorNote"]',);
            expect(schema,).toContain('"enum":["revised","unable","as-intended"]',);
          },
        },),
      ],
    },),
    describe({
      name: 'TRANSLATE_RESPONSE_FORMAT',
      children: [
        it({
          name: 'names the translation_report schema and requires translation, so a translator cannot '
            + 'answer with an empty reply and still satisfy the schema',
          fn: async () => {
            expect(TRANSLATE_RESPONSE_FORMAT.json_schema.name,).toBe('translation_report',);
            const schema = JSON.stringify(TRANSLATE_RESPONSE_FORMAT.json_schema.schema,);
            expect(schema,).toContain('"required":["translation"]',);
          },
        },),
      ],
    },),
  ],
},);
