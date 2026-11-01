import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RepairRegion, } from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Probe sensitivity inputs
// Every region and issue the sensitivity check probes, kept apart from the
// runner so each can grow without the other hitting the file cap.
//
// These are cat-themed invention. NO corpus text, licensed or otherwise, takes
// part in this check.

/**
 Original the accuracy regions are judged against.

 It DOES say the cat wakes when the sun moves, and says nothing about salmon.
 Both facts are load-bearing for the labelling arm, which turns on whether a
 prober checks a claim against this text or believes the label it was handed.
 */
export const SOURCE_TEXT = `猫猫在窗台上睡觉，太阳移动时她会醒来。
猫猫追蝴蝶，她很喜欢蝴蝶。`;

/**
 Translation before any replacement.
 */
export const BASELINE_TEXT = `The cat is doing the sleeping on the windowsill, and she wakes when the sun moves.
The cat is doing the chasing of butterflies, which she loves.`;

/**
 Region whose replacement fixes the stated defect and introduces nothing.

 The control. A probe that flags this is over-eager, which is the failure this
 prompt was built to avoid, and finding it here would be as informative as
 finding the opposite.
 */
export const CLEAN_REGION: RepairRegion = {
  envelopeId: 'envelope/clean',
  issueIds: ['adjudicated/tense',],
  before: 'The cat is doing the sleeping on the windowsill.',
  editorAfter: 'The cat sleeps on the windowsill.',
};

/**
 Region whose replacement fixes the tense and DROPS the second clause.

 Omission is the damage class the screen was widened for, and the one a
 forward-only quote requirement could never anchor.
 */
export const OMITTING_REGION: RepairRegion = {
  envelopeId: 'envelope/omitting',
  issueIds: ['adjudicated/tense',],
  before: 'The cat is doing the sleeping on the windowsill, and she wakes when the sun moves.',
  editorAfter: 'The cat sleeps on the windowsill.',
};

/**
 Region whose replacement fixes the tense and inverts the meaning.

 Blatant on purpose: the source says the cat likes butterflies and the
 replacement says she hates them. A reviewer shown both texts cannot miss it
 without failing at the task entirely.
 */
export const CONTRADICTING_REGION: RepairRegion = {
  envelopeId: 'envelope/contradicting',
  issueIds: ['adjudicated/tense',],
  before: 'The cat is doing the chasing of butterflies, which she loves.',
  editorAfter: 'The cat chases butterflies, which she hates.',
};

/**
 Repaired text the NATURALNESS regions are rewrites of.

 Already correct and already grammatical, which is the state the lane actually
 receives. A fixture that started from broken text would let a prober credit
 the rewrite for fixing something, and the question here is only what the
 rewrite BROKE.
 */
export const REFINED_BASELINE_TEXT =
  `The cat sleeps on the windowsill, and she wakes when the sun moves.
The cat chases butterflies, which she loves.`;

/**
 Rewrite that only smooths the wording.

 The control for the naturalness framing, and the one that matters most: this
 lane exists to rephrase, so a prober that reports rephrasing as damage would
 flag every refinement the pipeline ever makes.
 */
export const REFINED_CLEAN_REGION: RepairRegion = {
  envelopeId: 'refinement/clean',
  issueIds: ['adjudicated/tense',],
  before: 'The cat sleeps on the windowsill, and she wakes when the sun moves.',
  editorAfter: 'The cat sleeps on the windowsill and wakes when the sun moves.',
};

/**
 Rewrite that smooths the wording and DROPS the waking clause.
 */
export const REFINED_OMITTING_REGION: RepairRegion = {
  envelopeId: 'refinement/omitting',
  issueIds: ['adjudicated/tense',],
  before: 'The cat sleeps on the windowsill, and she wakes when the sun moves.',
  editorAfter: 'The cat sleeps on the windowsill.',
};

/**
 Rewrite that smooths the wording and inverts the meaning.
 */
export const REFINED_CONTRADICTING_REGION: RepairRegion = {
  envelopeId: 'refinement/contradicting',
  issueIds: ['adjudicated/tense',],
  before: 'The cat chases butterflies, which she loves.',
  editorAfter: 'The cat chases butterflies, which she hates.',
};

/**
 Accepted issue every original region was cut for, rendered into the sheet
 exactly as production renders it.

 Its summary names the progressive gloss, which IS present in each region's
 before text and IS fixed by each replacement. That is the point: a prober
 tempted to report the region's known defect has one sitting in front of it,
 labelled as not a finding.
 */
export const PRIOR_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/tense',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/tense',
      claim: {
        category: 'style/awkward-phrasing',
        severity: 'major',
        summary: 'Progressive gloss "is doing the" reads as machine output.',
        spans: [],
      },
    },
  ],
  tallies: {},
};

/**
 Translation the LABELLING arm's regions were cut from.

 Carries both of that arm's before texts, so each region sits in a baseline
 that actually contains it, the way production regions do.
 */
export const LABEL_BASELINE_TEXT =
  `The cat sleeps on the windowsill, and she wakes when the sun moves.
The cat chases butterflies, which she loves, and she dreams of salmon.`;

/**
 Deletion of source-supported content, labelled with an unrelated defect.

 The arm's own control: the edit drops a clause the original states outright,
 and nothing in the issue list points at that clause, so a prober has no
 licence to excuse the deletion. Damage here is the answer a working probe
 gives, and silence here would mean the probe cannot see omission at all,
 independently of any labelling effect.
 */
export const UNLABELLED_DELETION_REGION: RepairRegion = {
  envelopeId: 'deletion/unlabelled',
  issueIds: ['adjudicated/comma',],
  before: 'The cat sleeps on the windowsill, and she wakes when the sun moves.',
  editorAfter: 'The cat sleeps on the windowsill.',
};

/**
 The SAME deletion, labelled as removing an unsupported addition.

 The one variable in the arm. Byte-identical before and after text to
 {@link UNLABELLED_DELETION_REGION}; only the issue named in `issueIds`
 differs, and that issue is FALSE. A prober is shown the original, which
 states the waking clause, so the label is refutable from what it was given.
 */
export const MISLABELLED_DELETION_REGION: RepairRegion = {
  envelopeId: 'deletion/mislabelled',
  issueIds: ['adjudicated/waking-addition',],
  before: 'The cat sleeps on the windowsill, and she wakes when the sun moves.',
  editorAfter: 'The cat sleeps on the windowsill.',
};

/**
 Deletion of content the original really does not support, truthfully
 labelled.

 The negative control, and the reason the arm can distinguish a probe that
 reads the source from one that flags every deletion. Silence here is the
 CORRECT answer, and a probe that claims damage here would be unusable as a
 gate no matter how well it scores on the other two.
 */
export const LICENSED_DELETION_REGION: RepairRegion = {
  envelopeId: 'deletion/licensed',
  issueIds: ['adjudicated/salmon-addition',],
  before: 'The cat chases butterflies, which she loves, and she dreams of salmon.',
  editorAfter: 'The cat chases butterflies, which she loves.',
};

/**
 Unrelated defect naming nothing the edit deleted.
 */
export const COMMA_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/comma',
  status: 'accepted',
  severity: 'minor',
  claims: [
    {
      claimId: 'claim/comma',
      claim: {
        category: 'style/awkward-phrasing',
        summary: 'Comma before "and" splits the sentence more heavily than the original does.',
        severity: 'minor',
        spans: [],
      },
    },
  ],
  tallies: {},
};

/**
 FALSE addition claim: the original does state that the cat wakes.

 This is the shape of the precision failure measured on the corpus, where a
 critic calls content unsupported because the licensing evidence sits outside
 the window it judged. What it does to the probe is the question: an editor
 obeying this claim deletes real content, and a prober told the claim was a
 pre-existing defect is being invited to read that deletion as the repair.
 */
export const MISLABELLING_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/waking-addition',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/waking-addition',
      claim: {
        category: 'accuracy/addition',
        summary: 'Adds "and she wakes when the sun moves", which the original does not say.',
        severity: 'major',
        spans: [
          {
            side: 'target',
            nodeId: 'block/0',
            nodeHash: 'fixture',
            startOffset: 0,
            endOffset: 0,
            quotedText: 'and she wakes when the sun moves',
          },
        ],
      },
    },
  ],
  tallies: {},
};

/**
 TRUE addition claim: the original says nothing about salmon.
 */
export const LICENSING_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/salmon-addition',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/salmon-addition',
      claim: {
        category: 'accuracy/addition',
        summary: 'Adds "and she dreams of salmon", which the original does not say.',
        severity: 'major',
        spans: [
          {
            side: 'target',
            nodeId: 'block/0',
            nodeHash: 'fixture',
            startOffset: 0,
            endOffset: 0,
            quotedText: 'and she dreams of salmon',
          },
        ],
      },
    },
  ],
  tallies: {},
};

//endregion Probe sensitivity inputs
