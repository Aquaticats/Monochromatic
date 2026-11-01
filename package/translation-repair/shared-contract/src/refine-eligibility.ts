import type { DocumentNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RepairDocument, } from './parse-document.ts';

//region Refinement eligibility
// Which paragraphs of a repaired slice the naturalness lane may rewrite at all.
//
// This is an ELIGIBILITY FILTER and deliberately not called a verse detector.
// Nothing in the parsed model identifies poetry: an mdast `break`, a soft
// source wrap inside a node's text, and an HTML or MDX `<br>` are three
// different things and none of them means verse. Single-line poetry still
// passes this filter, which is the asymmetry that makes it honest as a filter
// and dishonest as a detector.
//
// It once excluded EVERY paragraph containing a newline, which collapsed the
// very distinction the ELIGIBILITY FILTER paragraph draws: a soft wrap renders as a space
// and carries no authored structure, while a hard break does. Measured over
// the 92 entries at the pinned corpus commit, 811 of 2067 prose paragraphs
// carry an internal newline and only 29 carry a hard break, so that rule
// discarded 782 ordinary wrapped paragraphs to protect 29, and the lane
// reported zero eligible paragraphs on 175 chunks of the run. It now excludes
// authored line structure only.
//
// Everything here reads the REPAIRED slice, never the original target. Accuracy
// edits shift offsets and can change block structure, so eligibility computed
// against the pre-repair document would be describing text that no longer
// exists.

/**
 Shortest paragraph worth spending a rewrite on;
 below this there is not enough prose for naturalness to be the problem.
 */
const MIN_REFINE_CHARS = 120;

/**
 Longest paragraph the lane will attempt.

 A cap rather than an optimization: every protected atom in a paragraph has to
 survive in order, so a very long paragraph multiplies the ways a rewrite can
 silently drop one while still reading well.
 */
const MAX_REFINE_CHARS = 1_200;

/**
 Block kind the lane rewrites. Headings, block quotes, code, tables, and
 lists are excluded by not appearing here rather than by a deny list, so a
 new block kind is ineligible until someone decides otherwise.
 */
const ELIGIBLE_KIND = 'paragraph';

/**
 Parse findings that make the tree a LESS faithful account of the bytes.

 `invisible-line-masked` is deliberately absent, and its absence is the whole
 point of naming kinds rather than counting them. Blanking a line that showed
 a reader nothing makes the parse more faithful, not less: it restores the
 paragraph break the author wrote, which the byte-order mark had welded shut.
 Treating it as degradation disqualified a slice for having been repaired.
 */
const DEGRADING_FINDINGS: ReadonlySet<string> = new Set([
  'mdx-downgraded',
  'html-comment-skipped',
  'unterminated-html-comment',
],);

/**
 Markup that can carry a line break or a structural element without a newline
 appearing in the node text, which is what makes the single-line check alone
 insufficient.
 */
const MARKUP_MARKERS = [
  '<',
  '>',
  '{',
  '}',
] as const;

/**
 Why one paragraph was skipped, in scorecard-stable wording.

 @example
 ```ts
 const reason: IneligibleReason = 'hard-break';
 ```
 */
export type IneligibleReason =
  | 'not-a-paragraph'
    | 'hard-break'
    | 'carries-markup'
    | 'too-short'
    | 'too-long'
    | 'parse-degraded';

/**
 One paragraph's eligibility verdict.

 @example
 ```ts
 const verdict: ParagraphEligibility = { node, eligible: true, };
 ```
 */
export type ParagraphEligibility =
  | {
    readonly eligible: true;

    /**
     Block the lane may rewrite.
     */
    readonly node: DocumentNode;
  }
    | {
      readonly eligible: false;

    /**
     Block that was skipped.
     */
      readonly node: DocumentNode;

    /**
     First rule that excluded it, in check order.
     */
      readonly reason: IneligibleReason;
    };

/**
 Builds a skip verdict, so the rule chain reads as one expression per rule
 rather than as a reason threaded through an absent value.

 @param node - block being skipped

 @param reason - rule that excluded it

 @returns Skip verdict

 @example
 ```ts
 return skipped({ node, reason: 'too-short', },);
 ```
 */
function skipped(
  {
    node,
    reason,
  }: {
    readonly node: DocumentNode;
    readonly reason: IneligibleReason;
  },
): ParagraphEligibility {
  return {
    eligible: false,
    node,
    reason,
  };
}

/**
 Whether a paragraph's line breaks are AUTHORED rather than incidental.

 The distinction this draws is the one the module header names and the code
 previously collapsed. A soft source wrap inside a paragraph is insignificant
 whitespace that renders as a space, so rewriting across it changes nothing a
 reader sees. A hard break is authored line structure, which is what verse
 uses and what a rewrite must never flatten.

 Markdown spells a hard break two ways: a line ending in two or more spaces,
 or a line ending in a backslash. An HTML `<br>` is a third, and it needs no
 check here because `MARKUP_MARKERS` already excludes any paragraph
 containing `<`.

 MEASURED before changing the rule, over the 92 entries at the pinned corpus
 commit: 811 of 2067 prose paragraphs carry an internal newline, and 29 of
 those carry a hard break. Rejecting every multi-line paragraph therefore
 discarded 782 ordinary wrapped paragraphs to protect 29, which is why the
 lane reported zero eligible paragraphs on 175 chunks.

 A linear scan rather than a pattern, since the rule is positional (does THIS
 line, which is not the last, end in a break marker) and reads plainly as a
 loop.

 @param text - paragraph source text, offsets intact

 @returns Whether any non-final line ends in a hard-break marker

 @example
 ```ts
 const authored = carriesHardBreak({ text: node.text, },);
 ```
 */
function carriesHardBreak({ text, }: { readonly text: string; },): boolean {
  /**
   Source lines; only the breaks BETWEEN them can be hard, so the last line
   is never examined.
   */
  const lines = text.split('\n',);

  return lines
    .slice(
    0,
    -1,
    )
    .some(function endsInBreak(line,): boolean {
      return line.endsWith('  ',) || line.endsWith('\\',);
    },);
}

/**
 Judges one block against every eligibility rule, reporting the first that
 excluded it.

 @param node - block under consideration

 @param degraded - whether parsing this slice reported findings, which makes
 every block in it ineligible

 @param minimumChars - shortest paragraph eligible in current refinement role

 @returns Verdict carrying the excluding rule when there is one

 @throws Error when a paragraph node stands outside the body zone, which no parse emits

 @example
 ```ts
 const verdict = judgeParagraph({ node, degraded: false, minimumChars, },);
 ```
 */
function judgeParagraph(
  {
    node,
    degraded,
    minimumChars,
  }: {
    readonly node: DocumentNode;
    readonly degraded: boolean;
    readonly minimumChars: number;
  },
): ParagraphEligibility {
  if (node.kind !== ELIGIBLE_KIND)
    return skipped({
      node,
      reason: 'not-a-paragraph',
    },);

  // `parseDocument` gives a node the zone `footnote-definition` exactly when
  // its kind is `footnoteDefinition`, and the kind test refuses that kind
  // first, so a paragraph outside the body zone is a node no parse emits.
  if (node.zone !== 'body')
    throw new Error('unreachable: a paragraph node stands outside the body zone, and the parse gives that zone to footnote definitions alone',);

  // A degraded parse means the mdast this filter reasons about is not a
  // faithful account of the bytes, so no block in the slice is eligible.
  if (degraded)
    return skipped({
      node,
      reason: 'parse-degraded',
    },);
  if (carriesHardBreak({ text: node.text, },))
    return skipped({
      node,
      reason: 'hard-break',
    },);
  if (MARKUP_MARKERS.some(function present(marker,) {
    return node.text
      .includes(marker,);
  },))
    return skipped({
      node,
      reason: 'carries-markup',
    },);
  if (node.text
    .length
    < minimumChars)
    return skipped({
      node,
      reason: 'too-short',
    },);
  if (node.text
    .length
    > MAX_REFINE_CHARS)
    return skipped({
      node,
      reason: 'too-long',
    },);
  return {
    eligible: true,
    node,
  };
}

/**
 Selects the paragraphs of one repaired slice the naturalness lane may
 rewrite, keeping every skip with its reason so the lane's yield is
 explainable rather than merely observed.

 @param document - REPAIRED slice, parsed after accuracy edits landed

 @param minimumChars - shortest eligible paragraph; defaults to measured
 repair-lane window while final polish may use narrower sentence-scale window

 @returns Verdict per block in document order

 @example
 ```ts
 const verdicts = selectRefinableParagraphs({ document, },);
 ```
 */
export function selectRefinableParagraphs(
  {
    document,
    minimumChars = MIN_REFINE_CHARS,
  }: {
    readonly document: RepairDocument;
    readonly minimumChars?: number;
  },
): readonly ParagraphEligibility[] {
  /**
   Whether the tree is a less faithful account of the bytes than usual, which
   disqualifies the whole slice rather than the offending block alone: a
   downgrade to plain markdown or a blanked comment changes how every block
   was read.

   Asked of the finding KIND rather than of the count, because not every
   finding is a loss. See {@link DEGRADING_FINDINGS}.
   */
  const degraded = document.parseFindings
    .some(function isLoss(finding,) {
      return DEGRADING_FINDINGS.has(finding.kind,);
    },);
  return document.nodes
    .map(function toVerdict(node,) {
      return judgeParagraph({
        node,
        degraded,
        minimumChars,
      },);
    },);
}

//endregion Refinement eligibility
