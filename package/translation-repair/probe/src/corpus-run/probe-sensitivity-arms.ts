import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type PriorIssueDisclosure,
  type ProbedEditKind,
  PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
  type RepairRegion,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  BASELINE_TEXT,
  CLEAN_REGION,
  COMMA_ISSUE,
  CONTRADICTING_REGION,
  LABEL_BASELINE_TEXT,
  LICENSED_DELETION_REGION,
  LICENSING_ISSUE,
  MISLABELLED_DELETION_REGION,
  MISLABELLING_ISSUE,
  OMITTING_REGION,
  PRIOR_ISSUE,
  REFINED_BASELINE_TEXT,
  REFINED_CLEAN_REGION,
  REFINED_CONTRADICTING_REGION,
  REFINED_OMITTING_REGION,
  UNLABELLED_DELETION_REGION,
} from './probe-sensitivity-input.ts';

//region Sensitivity arms
// EVERY CALL THE SENSITIVITY INSTRUMENT MAKES, AS DATA, so a test can read what
// each arm sends before a run spends anything on it.
//
// The instrument's `prior=shown` arm was once found sending the same prompt as
// its `prior=absent` arm. It passed the prior issue and no disclosure, relying
// on the probe's default, and that default had flipped to `withheld` when
// production stopped rendering the list; the two arms then differed only in
// what the deterministic screen dismissed, while the closing note told the
// reader the difference was the production prompt. An arm that names what it
// sends, and a test that holds the name to the value, is the cure.
//
// THREE LISTS, because two effects were being conflated. `none` sends no
// prior issue at all. `withheld` sends the prior issue to the screen and not
// to the prompt, which is what production does. `rendered` writes the prior
// issue into the sheet under "PRE-EXISTING DEFECTS THIS EDIT TARGETED", which
// is the prompt production abandoned because it silenced the stage. The gap
// between `none` and `withheld` is the screen; the gap between `withheld` and
// `rendered` is the prompt.

/**
 The three lists, in the order each region is run.

 THE TYPE IS DERIVED FROM THIS LIST, so a list cannot be added to the type
 without being run, and `DISCLOSURE_OF_LIST` cannot leave it without a
 disclosure.
 */
const PRIOR_ISSUE_LISTS = [
  'none',
  'withheld',
  'rendered',
] as const;

/**
 What the prober was told about the prior issue, printed as `list=`.

 @example
 ```ts
 const list: PriorIssueList = 'withheld';
 ```
 */
export type PriorIssueList = typeof PRIOR_ISSUE_LISTS[number];

/**
 Which accepted issue the arm carries, printed as `issue=`.

 @example
 ```ts
 const issue: IssueLabel = 'false-addition';
 ```
 */
export type IssueLabel = 'none' | 'prior' | 'unrelated' | 'false-addition' | 'true-addition';

/**
 One probe call of the sensitivity instrument.

 @example
 ```ts
 const first = SENSITIVITY_ARMS[0];
 ```
 */
export type SensitivityArm = {
  /**
   Region under test.
   */
  readonly region: RepairRegion;

  /**
   What a working probe should conclude, for the printed line only.
   */
  readonly expectation: string;

  /**
   Which list the prober was given.
   */
  readonly list: PriorIssueList;

  /**
   Which accepted issue the list carries.
   */
  readonly issue: IssueLabel;

  /**
   Accepted issues sent as pre-existing; empty under `list: 'none'`.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   Disclosure sent to the probe, held to `list` by the unit test.
   */
  readonly disclosure: PriorIssueDisclosure;

  /**
   Framing under test.
   */
  readonly editKind: ProbedEditKind;

  /**
   Translation the region was cut from.
   */
  readonly baselineText: string;
};

/**
 List production sends, read off the same constant the pass uses.

 @example
 ```ts
 console.log(`production sends list=${PRODUCTION_LIST}`,);
 ```
 */
export const PRODUCTION_LIST: PriorIssueList = PRODUCTION_PRIOR_ISSUE_DISCLOSURE;

/**
 A region and what a working probe should say about it.
 */
type ProbedRegion = {
  readonly region: RepairRegion;
  readonly expectation: string;
};

/**
 Disclosure each list value sends; `none` has nothing to disclose and sends
 production's, so the prompt is production's prompt with an empty list.

 A RECORD OVER EVERY LIST, so a list added to `PriorIssueList` cannot send
 production's disclosure by default: the compiler refuses it here until the
 disclosure it sends is named.
 */
const DISCLOSURE_OF_LIST: Readonly<Record<PriorIssueList, PriorIssueDisclosure>> = {
  none: PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
  withheld: 'withheld',
  rendered: 'rendered',
};

/**
 Disclosure a list value sends.

 @param list - list the arm is labelled with

 @returns Disclosure the probe receives

 @example
 ```ts
 const disclosure = disclosureFor('rendered',);
 ```
 */
function disclosureFor(list: PriorIssueList,): PriorIssueDisclosure {
  return DISCLOSURE_OF_LIST[list];
}

/**
 Accuracy regions under all three lists.

 @returns Nine arms, three per region

 @example
 ```ts
 const arms = accuracyArms();
 ```
 */
function accuracyArms(): readonly SensitivityArm[] {
  /**
   Regions and what a working probe should say about each.
   */
  const regions: readonly ProbedRegion[] = [
    {
      region: CLEAN_REGION,
      expectation: 'no-damage',
    },
    {
      region: OMITTING_REGION,
      expectation: 'damage-omission',
    },
    {
      region: CONTRADICTING_REGION,
      expectation: 'damage-meaning-inverted',
    },
  ];
  return regions.flatMap(function underEachList(
    {
      region,
      expectation,
    },
  ): readonly SensitivityArm[] {
    return PRIOR_ISSUE_LISTS.map(function toArm(list,): SensitivityArm {
      return {
        region,
        expectation,
        list,
        issue: (list === 'none') ? 'none' : 'prior',
        issues: (list === 'none') ? [] : [PRIOR_ISSUE,],
        disclosure: disclosureFor(list,),
        editKind: 'accuracy-repair',
        baselineText: BASELINE_TEXT,
      };
    },);
  },);
}

/**
 The naturalness framing, one arm per region under production's list. It is
 a different prompt asking the same question, and a working accuracy probe
 proves nothing about it; its control is the clean region, since the lane
 exists to rephrase and a prober that reads rephrasing as damage would flag
 every refinement the pipeline makes.

 @returns Three arms

 @example
 ```ts
 const arms = refinementArms();
 ```
 */
function refinementArms(): readonly SensitivityArm[] {
  /**
   Regions and what a working probe should say about each.
   */
  const regions: readonly ProbedRegion[] = [
    {
      region: REFINED_CLEAN_REGION,
      expectation: 'no-damage',
    },
    {
      region: REFINED_OMITTING_REGION,
      expectation: 'damage-omission',
    },
    {
      region: REFINED_CONTRADICTING_REGION,
      expectation: 'damage-meaning-inverted',
    },
  ];
  return regions.map(function toArm(
    {
      region,
      expectation,
    },
  ): SensitivityArm {
    return {
      region,
      expectation,
      list: PRODUCTION_LIST,
      issue: 'prior',
      issues: [PRIOR_ISSUE,],
      disclosure: PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
      editKind: 'naturalness-refinement',
      baselineText: REFINED_BASELINE_TEXT,
    };
  },);
}

/**
 The labelling regions, which vary what the list SAYS rather than whether
 there is one, under both lists that carry it. Rendered, the prober reads the
 label; withheld, only the screen does. The first two regions delete the SAME
 source-supported clause and differ only in what the list says about it; the
 third deletes content the original genuinely lacks, so silence there is
 correct.

 @returns Six arms, two per region

 @example
 ```ts
 const arms = labellingArms();
 ```
 */
function labellingArms(): readonly SensitivityArm[] {
  /**
   Regions with the issue each carries and what a working probe should say.
   */
  const regions: readonly {
    readonly region: RepairRegion;
    readonly expectation: string;
    readonly issue: IssueLabel;
    readonly issues: readonly AdjudicatedIssue[];
  }[] = [
    {
      region: UNLABELLED_DELETION_REGION,
      expectation: 'damage-omission',
      issue: 'unrelated',
      issues: [COMMA_ISSUE,],
    },
    {
      region: MISLABELLED_DELETION_REGION,
      expectation: 'damage-omission',
      issue: 'false-addition',
      issues: [MISLABELLING_ISSUE,],
    },
    {
      region: LICENSED_DELETION_REGION,
      expectation: 'no-damage',
      issue: 'true-addition',
      issues: [LICENSING_ISSUE,],
    },
  ];
  /**
   Lists that carry an issue at all: every list but `none`, which carries
   none by its own definition.
   */
  const carrying: readonly PriorIssueList[] = [
    'withheld',
    'rendered',
  ];
  return regions.flatMap(function underEachList(labelled,): readonly SensitivityArm[] {
    return carrying.map(function toArm(list,): SensitivityArm {
      return {
        region: labelled.region,
        expectation: labelled.expectation,
        list,
        issue: labelled.issue,
        issues: labelled.issues,
        disclosure: disclosureFor(list,),
        editKind: 'accuracy-repair',
        baselineText: LABEL_BASELINE_TEXT,
      };
    },);
  },);
}

/**
 Every arm the instrument runs, in run order.

 @example
 ```ts
 for (const arm of SENSITIVITY_ARMS) await probeOne({ arm, },);
 ```
 */
export const SENSITIVITY_ARMS: readonly SensitivityArm[] = [
  ...accuracyArms(),
  ...refinementArms(),
  ...labellingArms(),
];

//endregion Sensitivity arms
