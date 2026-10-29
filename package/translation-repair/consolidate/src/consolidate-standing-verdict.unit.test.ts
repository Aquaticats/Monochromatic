/**
 Tests for the two verdicts on a standing text and the log line each
 refusal writes.

 WHAT THESE PIN: an invalid standing and an unendorsed one are refused
 for different reasons and the log says which, with the deterministic
 findings on the first. One warning covered both on the 2026-09-04
 luxuanwen3 pass, and the cause (a link destination the archive had
 rewritten) had to be read out of the slice records.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  INELIGIBLE_STANDING_REPLACED_FINDING,
  readStandingVerdict,
} from '../dist/final/node/index.mjs';

/**
 Original slice, linking one destination.
 */
const SOURCE = '她的头像由[画师](https://twitter.com/cat)绘制。';

/**
 Page rendering that kept the destination.
 */
const PAGE_KEPT = 'Her avatar was drawn by [the artist](https://twitter.com/cat).';

/**
 Rendering that carries neither destination, invalid under the
 either-rendering rule of 2026-09-04 as under the rule before it.
 */
const LINK_DROPPED = 'Her avatar was drawn by the artist.';

/**
 Logger whose warnings are kept for the assertions, the rest forwarded.

 @returns Logger and the warnings it received

 @example
 ```ts
 const { l, warnings, } = capturing();
 ```
 */
function capturing(): {
  readonly l: Logger;
  readonly warnings: string[];
} {
  /**
   Warnings in the order written.
   */
  const warnings: string[] = [];
  /**
   Forwarding logger for every level but warn.
   */
  const l: Logger = {
    ...tagged({ tag: 'consolidate-standing-verdict-test', },),
    warn: function warn(message: string,): void {
      warnings.push(message,);
    },
  };
  return {
    l,
    warnings,
  };
}

await describe({
  name: readStandingVerdict.name,
  children: [
    it({
      name: 'WITHHOLDS AN INVALID STANDING and names the finding behind it, where the archive never '
        + 'carried the passage and so offers nothing to stand in',
      fn: async () => {
        const { l, warnings, } = capturing();
        /**
         Verdict beside the refusal the gate sheet will carry.
         */
        const { standingRefusal, ...verdict } = readStandingVerdict({
          sourceText: SOURCE,
          standingText: LINK_DROPPED,
          incumbentText: '',
          lineStructured: false,
          choice: 'translate',
          contestVerdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
          sliceIndex: 1,
          l,
        },);
        expect(verdict,).toEqual({
          standingValid: false,
          standingMayShip: false,
          settlementText: LINK_DROPPED,
          findings: [],
          incumbentStandsIn: false,
        },);
        expect(warnings.length,).toBe(1,);
        expect(warnings[0],).toContain(standingRefusal ?? 'refusal missing',);
        expect(warnings[0],).toContain('slice 1: consolidation standing text fails the deterministic publication rule',);
        expect(warnings[0],).toContain('The ORIGINAL carries link-url https://twitter.com/cat and your translation does not',);
        expect(warnings[0]?.includes('the incumbent fails it too',),).toBe(false,);
      },
    },),

    it({
      name: 'STANDS THE INCUMBENT IN for an invalid standing where the incumbent passes the gate, '
        + 'without endorsement and with the replacement recorded: the owner\'s addendum of 2026-09-09 '
        + 'on the sixth Mio, whose slice 3 archive was valid and never offered',
      fn: async () => {
        const { l, warnings, } = capturing();
        expect(readStandingVerdict({
          sourceText: SOURCE,
          standingText: LINK_DROPPED,
          incumbentText: PAGE_KEPT,
          lineStructured: false,
          choice: 'translate',
          contestVerdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
          sliceIndex: 3,
          l,
        },),).toEqual({
          standingValid: true,
          standingMayShip: false,
          settlementText: PAGE_KEPT,
          findings: [INELIGIBLE_STANDING_REPLACED_FINDING,],
          incumbentStandsIn: true,
        },);
        expect(warnings.length,).toBe(1,);
        expect(warnings[0],).toContain('slice 3: consolidation standing text fails the deterministic publication rule',);
        expect(warnings[0],).toContain('the incumbent passes it and stands in',);
      },
    },),

    it({
      name: 'WITHHOLDS BOTH where the incumbent fails the gate too, naming both refusals, so neither '
        + 'text is offered to the slate and no stand-in is claimed',
      fn: async () => {
        const { l, warnings, } = capturing();
        /**
         Verdict beside the refusal the gate sheet will carry.
         */
        const { standingRefusal, ...verdict } = readStandingVerdict({
          sourceText: SOURCE,
          standingText: LINK_DROPPED,
          incumbentText: 'The artist drew it.',
          lineStructured: false,
          choice: 'translate',
          contestVerdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
          sliceIndex: 4,
          l,
        },);
        expect(verdict,).toEqual({
          standingValid: false,
          standingMayShip: false,
          settlementText: LINK_DROPPED,
          findings: [],
          incumbentStandsIn: false,
        },);
        expect(warnings.length,).toBe(1,);
        expect(warnings[0],).toContain(standingRefusal ?? 'refusal missing',);
        expect(warnings[0],).toContain('withheld from the slate',);
        expect(warnings[0],).toContain('the incumbent fails it too',);
      },
    },),

    it({
      name: 'READS NO INCUMBENT where the standing IS the incumbent, the luxuanwen3 shape, so one '
        + 'refused text is not offered as its own replacement',
      fn: async () => {
        const { l, warnings, } = capturing();
        /**
         Verdict beside the refusal the gate sheet will carry.
         */
        const { standingRefusal, ...verdict } = readStandingVerdict({
          sourceText: SOURCE,
          standingText: LINK_DROPPED,
          incumbentText: LINK_DROPPED,
          lineStructured: false,
          choice: 'neither',
          contestVerdict: {
            kind: 'settled-neither',
            archive: 'declined',
          },
          sliceIndex: 5,
          l,
        },);
        expect(verdict,).toEqual({
          standingValid: false,
          standingMayShip: false,
          settlementText: LINK_DROPPED,
          findings: [],
          incumbentStandsIn: false,
        },);
        expect(warnings.length,).toBe(1,);
        expect(warnings[0],).toContain(standingRefusal ?? 'refusal missing',);
        expect(warnings[0],).toContain('withheld from the slate',);
        expect(warnings.some(function namesIncumbent(warning,): boolean {
          return warning.includes('the incumbent fails it too',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'NAMES A LACK OF ENDORSEMENT on a valid standing the contest declined',
      fn: async () => {
        const { l, warnings, } = capturing();
        expect(readStandingVerdict({
          sourceText: SOURCE,
          standingText: PAGE_KEPT,
          incumbentText: PAGE_KEPT,
          lineStructured: false,
          choice: 'neither',
          contestVerdict: {
            kind: 'settled-neither',
            archive: 'declined',
          },
          sliceIndex: 2,
          l,
        },),).toEqual({
          standingValid: true,
          standingMayShip: false,
          settlementText: PAGE_KEPT,
          findings: [],
          incumbentStandsIn: false,
        },);
        expect(warnings,).toEqual([
          'slice 2: consolidation standing text lacks contest endorsement and remains retryable',
        ],);
      },
    },),

    it({
      name: 'READS A FRONT-MATTER STANDING AND INCUMBENT UNDER THE FRONT-MATTER RULE: a standing that renames a '
        + 'YAML field is refused and the valid incumbent stands in, where the Markdown rule the same call makes '
        + 'without the syntax role keeps the standing',
      fn: async () => {
        /** The slice's metadata, whose name the alias repeats. */
        const sourceText = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';
        /** The contest's winner, which renamed a field. */
        const renamed = '---\ntitle: Maomao Cat\ninfo:\n  alias: Maomao Cat\n---\n';
        /** The archive's metadata, valid under either rule. */
        const incumbent = '---\nname: Maomao Cat\ninfo:\n  alias: Maomao Cat\n---\n';
        /** The slice as both calls read it, apart from the syntax role. */
        const slice = {
          sourceText,
          standingText: renamed,
          incumbentText: incumbent,
          lineStructured: false,
          choice: 'translate',
          contestVerdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
          sliceIndex: 6,
        } as const;
        /** Verdict under the Markdown rule. */
        const markdown = readStandingVerdict({
          ...slice,
          l: capturing().l,
        },);
        /** Verdict under the front-matter rule. */
        const frontMatter = readStandingVerdict({
          ...slice,
          syntax: 'front-matter',
          l: capturing().l,
        },);
        expect([
          [markdown.incumbentStandsIn, markdown.settlementText,],
          [frontMatter.incumbentStandsIn, frontMatter.settlementText,],
        ],).toEqual([
          [false, renamed,],
          [true, incumbent,],
        ],);
      },
    },),

    it({
      name: 'WRITES NOTHING for a valid standing with a won contest behind it',
      fn: async () => {
        const { l, warnings, } = capturing();
        expect(readStandingVerdict({
          sourceText: SOURCE,
          standingText: PAGE_KEPT,
          incumbentText: PAGE_KEPT,
          lineStructured: false,
          choice: 'translate',
          contestVerdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
          sliceIndex: 3,
          l,
        },),).toEqual({
          standingValid: true,
          standingMayShip: true,
          settlementText: PAGE_KEPT,
          findings: [],
          incumbentStandsIn: false,
        },);
        expect(warnings,).toEqual([],);
      },
    },),
  ],
},);
