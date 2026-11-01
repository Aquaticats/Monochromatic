/**
 Tests for the sensitivity instrument's arms table.

 The instrument's `prior=shown` arm was once found sending the same prompt as
 its `prior=absent` arm, because it relied on a default that had flipped. The
 cases in this file hold every arm's printed label to the disclosure it sends, so a
 run's lines cannot describe a prompt effect that is a screen effect again,
 and they pin the run plan: every accuracy region under all three lists,
 every labelling region under both lists that carry an issue, and the
 production arm read off the constant the pass sends.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { PRODUCTION_PRIOR_ISSUE_DISCLOSURE, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  PRODUCTION_LIST,
  SENSITIVITY_ARMS,
  type SensitivityArm,
} from '../../dist/final/node/index.mjs';

/**
 Lists each region of a group was run under, sorted for comparison.

 @param arms - arms of one region

 @returns Sorted list labels

 @example
 ```ts
 const lists = listsOf(arms,);
 ```
 */
function listsOf(arms: readonly SensitivityArm[],): readonly string[] {
  return arms
    .map(function toList(arm,): string {
      return arm.list;
    },)
    .toSorted();
}

/**
 Groups arms by the region they probe.

 @param arms - arms to group

 @returns Arms per envelope id

 @example
 ```ts
 const byRegion = groupByRegion(SENSITIVITY_ARMS,);
 ```
 */
function groupByRegion(arms: readonly SensitivityArm[],): ReadonlyMap<string, readonly SensitivityArm[]> {
  return arms.reduce(function into(
    groups: Map<string, readonly SensitivityArm[]>,
    arm,
  ): Map<string, readonly SensitivityArm[]> {
    groups.set(
      arm.region.envelopeId,
      [
        ...(groups.get(arm.region.envelopeId,) ?? []),
        arm,
      ],
    );
    return groups;
  }, new Map<string, readonly SensitivityArm[]>(),);
}

await describe({
  name: 'SENSITIVITY_ARMS',
  children: [
    it({
      name: 'SENDS the disclosure its list names, so the printed label is what the prober got',
      fn: async () => {
        for (const arm of SENSITIVITY_ARMS) {
          if (arm.list === 'rendered')
            expect(arm.disclosure,).toBe('rendered',);
          if (arm.list === 'withheld')
            expect(arm.disclosure,).toBe('withheld',);
        }
        expect(SENSITIVITY_ARMS.some(function isRendered(arm,): boolean {
          return arm.list === 'rendered';
        },),).toBe(true,);
        expect(SENSITIVITY_ARMS.some(function isWithheld(arm,): boolean {
          return arm.list === 'withheld';
        },),).toBe(true,);
      },
    },),

    it({
      name: 'CARRIES an issue under every list but none, and no issue under list=none',
      fn: async () => {
        for (const arm of SENSITIVITY_ARMS) {
          expect(arm.issues.length === 0,).toBe(arm.list === 'none',);
          expect(arm.issue === 'none',).toBe(arm.list === 'none',);
        }
      },
    },),

    it({
      name: 'RUNS every accuracy region under all three lists, so none-against-withheld isolates the '
        + 'screen and withheld-against-rendered isolates the prompt',
      fn: async () => {
        /**
         Accuracy arms carrying the prior issue or nothing.
         */
        const accuracy = SENSITIVITY_ARMS.filter(function isAccuracy(arm,): boolean {
          return (arm.editKind === 'accuracy-repair') && ((arm.issue === 'prior') || (arm.issue === 'none'));
        },);

        /**
         Arms per region.
         */
        const byRegion = groupByRegion(accuracy,);

        expect(byRegion.size,).toBe(3,);
        for (const arms of byRegion.values())
          expect(listsOf(arms,),).toStrictEqual(['none', 'rendered', 'withheld',],);
      },
    },),

    it({
      name: 'RUNS every labelling region under both lists that carry an issue',
      fn: async () => {
        /**
         Arms that vary what the list says.
         */
        const labelling = SENSITIVITY_ARMS.filter(function isLabelling(arm,): boolean {
          return (arm.issue === 'unrelated') || (arm.issue === 'false-addition') || (arm.issue === 'true-addition');
        },);

        /**
         Arms per region.
         */
        const byRegion = groupByRegion(labelling,);

        expect(byRegion.size,).toBe(3,);
        for (const arms of byRegion.values())
          expect(listsOf(arms,),).toStrictEqual(['rendered', 'withheld',],);
      },
    },),

    it({
      name: 'NAMES the production list from the constant the pass sends, and runs the refinement arms under it',
      fn: async () => {
        /**
         Arms under the naturalness framing.
         */
        const refinement = SENSITIVITY_ARMS.filter(function isRefinement(arm,): boolean {
          return arm.editKind === 'naturalness-refinement';
        },);

        expect(PRODUCTION_LIST,).toBe(PRODUCTION_PRIOR_ISSUE_DISCLOSURE,);
        expect(refinement.length,).toBe(3,);
        for (const arm of refinement) {
          expect(arm.list,).toBe(PRODUCTION_LIST,);
          expect(arm.disclosure,).toBe(PRODUCTION_PRIOR_ISSUE_DISCLOSURE,);
        }
      },
    },),
  ],
},);
