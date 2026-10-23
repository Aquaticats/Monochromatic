/**
 Holds the three contest formats to the vocabularies their guards read (P1 of the second review of the Claude route).
 A live Claude call under the lane contest's schema returned `"verdict": "Consistent"`, valid under a plain string and
 refused by `isLaneContestWire`, which reads `verdict` against the candidate names plus the refusal and
 `verdictOnArchive` against the archive verdicts. The consolidation gate and the polish gate build the same format over
 their own names. Each case compares the format the stage sends with the constant its guard reads, imported from the
 module that owns the guard.

 The findings lists stay plain strings in all three: a finding may annotate the candidate it names, and the reader takes
 the name from the start of the finding.

 Nothing here reaches a provider: the schemas are read, never sent.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  CONSOLIDATE_GATE_NAMES,
  CONSOLIDATE_GATE_RESPONSE_FORMAT,
  CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT,
  isConsolidateGateWire,
  isConsolidationPolishGateWire,
  POLISH_NAMES,
} from '@monochromatic-dev/translation-repair-consolidate';
import {
  ARCHIVE_VERDICTS,
  CANDIDATE_NAMES,
  isLaneContestWire,
  LANE_CONTEST_RESPONSE_FORMAT,
  readLaneContestBallot,
} from '@monochromatic-dev/translation-repair-lane-contest';

/**
 Label no vocabulary holds, which every guard must refuse.
 */
const NOT_A_LABEL = 'not-a-label-of-any-vocabulary';

/**
 Reads the labels a schema lists for one member.

 @param schema - `properties` of a contest format, as the format holds them

 @param member - member whose `enum` is read

 @returns Listed labels

 @throws {@link TypeError} where the member lists none, so a check over the labels cannot pass for want of any

 @example
 ```ts
 const labels = listedLabels({
   schema: LANE_CONTEST_RESPONSE_FORMAT.json_schema.schema.properties,
   member: 'verdict',
 },);
 ```
 */
function listedLabels(
  {
    schema,
    member,
  }: {
    readonly schema: unknown;
    readonly member: string;
  },
): readonly string[] {
  /**
   The member's node.
   */
  const node = isJsonRecord(schema,) ? schema[member] : undefined;
  if ((!isJsonRecord(node,)) || (!Array.isArray(node.enum,)) || (node.enum.length === 0))
    throw new TypeError(`The member ${member} lists no enum`,);
  return node.enum.filter(function isText(label: unknown,): label is string {
    return (typeof label) === 'string';
  },);
}

await describe({
  name: 'the contest formats',
  concurrency: 1,
  children: [
    it({
      name: 'LISTS THE LANE CONTEST\'S VERDICT AS AN enum OF CANDIDATE_NAMES AND ITS ARCHIVE VERDICT AS ONE OF '
        + 'ARCHIVE_VERDICTS, the constants isLaneContestWire reads, and keeps the findings plain',
      fn: async () => {
        expect(LANE_CONTEST_RESPONSE_FORMAT.json_schema.schema.properties,).toEqual({
          assessment: { type: 'string', },
          dropped: { type: 'array', items: { type: 'string', }, },
          unsupported: { type: 'array', items: { type: 'string', }, },
          verdict: { type: 'string', enum: [...CANDIDATE_NAMES,], },
          verdictOnArchive: { type: 'string', enum: [...ARCHIVE_VERDICTS,], },
        },);
        /**
         The listings the model reads, fed to the guard and the reader the stage runs (F10 of the re-check of the second
         fix round).
         */
        const { properties, } = LANE_CONTEST_RESPONSE_FORMAT.json_schema.schema;
        /**
         Wire of one reply, with the verdicts the case sets.
         */
        const wireOf = (verdict: string, verdictOnArchive: string,) => ({
          assessment: 'the cat naps',
          dropped: [],
          unsupported: [],
          verdict,
          verdictOnArchive,
        });
        expect(listedLabels({ schema: properties, member: 'verdict', },).every(function taken(name,): boolean {
          return isLaneContestWire(wireOf(name, ARCHIVE_VERDICTS[0] ?? '',),);
        },),).toBe(true,);
        /**
         Reply whose verdict is no label.
         */
        const unlabelled = wireOf(NOT_A_LABEL, ARCHIVE_VERDICTS[0] ?? '',);
        expect(isLaneContestWire(unlabelled,),).toBe(false,);
        expect(listedLabels({ schema: properties, member: 'verdictOnArchive', },).every(function read(name,): boolean {
          return readLaneContestBallot({ wire: wireOf(CANDIDATE_NAMES[0] ?? '', name,) as never, },).archive === name;
        },),).toBe(true,);
        /**
         Ballot read from a reply whose archive verdict is no label.
         */
        const unreadArchive = readLaneContestBallot({
          wire: wireOf(CANDIDATE_NAMES[0] ?? '', NOT_A_LABEL,) as never,
        },);
        expect(unreadArchive.archive,).toBeUndefined();
      },
    },),

    it({
      name: 'LISTS THE CONSOLIDATION GATE\'S VERDICT AS AN enum OF ITS GUARD\'S NAMES and asks no archive verdict, since its '
        + 'standing choice is the archive',
      fn: async () => {
        expect(CONSOLIDATE_GATE_RESPONSE_FORMAT.json_schema.schema.properties,).toEqual({
          assessment: { type: 'string', },
          dropped: { type: 'array', items: { type: 'string', }, },
          unsupported: { type: 'array', items: { type: 'string', }, },
          verdict: { type: 'string', enum: [...CONSOLIDATE_GATE_NAMES,], },
        },);
        // The listing the model reads, fed to the guard the stage runs (F10 of the re-check of the second fix round).
        expect(listedLabels({
          schema: CONSOLIDATE_GATE_RESPONSE_FORMAT.json_schema.schema.properties,
          member: 'verdict',
        },).every(function taken(name,): boolean {
          return isConsolidateGateWire({ assessment: 'the cat naps', verdict: name, },);
        },),).toBe(true,);
        expect(isConsolidateGateWire({ assessment: 'the cat naps', verdict: NOT_A_LABEL, },),).toBe(false,);
      },
    },),

    it({
      name: 'LISTS THE POLISH GATE\'S VERDICT AS AN enum OF POLISH_NAMES and asks no archive verdict',
      fn: async () => {
        expect(CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT.json_schema.schema.properties,).toEqual({
          assessment: { type: 'string', },
          dropped: { type: 'array', items: { type: 'string', }, },
          unsupported: { type: 'array', items: { type: 'string', }, },
          verdict: { type: 'string', enum: [...POLISH_NAMES,], },
        },);
        // The listing the model reads, fed to the guard the stage runs (F10 of the re-check of the second fix round).
        expect(listedLabels({
          schema: CONSOLIDATION_POLISH_GATE_RESPONSE_FORMAT.json_schema.schema.properties,
          member: 'verdict',
        },).every(function taken(name,): boolean {
          return isConsolidationPolishGateWire({ assessment: 'the cat naps', verdict: name, },);
        },),).toBe(true,);
        expect(isConsolidationPolishGateWire({ assessment: 'the cat naps', verdict: NOT_A_LABEL, },),).toBe(false,);
      },
    },),
  ],
},);
