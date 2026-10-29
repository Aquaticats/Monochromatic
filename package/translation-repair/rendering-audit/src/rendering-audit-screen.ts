import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type AnchoredSpan,
  anchorLocatedSpan,
} from './rendering-audit-anchor.ts';
import { RenderingAuditInvariantError, } from './rendering-audit-invariant.ts';
import {
  CANDIDATE_ONLY_CATEGORIES,
  PAIRED_CATEGORIES,
  RENDERING_AUDIT_CATEGORIES,
  type RenderingAuditCategory,
  type RenderingAuditFindingWire,
  RENDERING_AUDIT_VERDICTS,
  type RenderingAuditReportWire,
  type RenderingAuditVerdict,
  SOURCE_ONLY_CATEGORIES,
} from './rendering-audit-wire.ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Rendering audit screen
// What survives of one auditor's answer once its quotes are checked against the
// texts, deterministically and without asking anybody.
//
// EACH CATEGORY MUST ANCHOR IN THE SIDE IT CAN. Content the candidate never
// rendered has nothing in the candidate to quote, so demanding a candidate span
// for an omission would make omission unfileable, and it is the likeliest
// defect a from-scratch rendering has. An addition nothing supports is the
// mirror. A changed number, actor or polarity is stated by both and anchors in
// both.
//
// THE OBLIGATION RUNS BOTH WAYS, which an earlier version left half-enforced.
// A missing required side was dropped, and a quote on a side the category does
// not use was silently ERASED. That let an `omission` arrive carrying candidate
// text, which is a contradiction in terms, and let a voice escape a paired
// category's evidence obligation by filing the same claim as one-sided. Both
// directions are refused here.
//
// A DROPPED CLAIM IS NOT EVIDENCE OF FIDELITY, and the tally says so separately:
// a voice that reported nothing and a voice whose every claim was dropped are
// different facts, and reading them as one would let a run of unanchored
// answers look like agreement that a rendering is sound.

/**
 What one side of a screened finding rests on.

 NAMED ABSENCE rather than an optional field: a side a category does not use
 is a different thing from a side that happens to be missing, and a reader
 that has to tell them apart from `undefined` will eventually get it wrong.

 @example
 ```ts
 const reading: SideReading = { kind: 'unused', };
 ```
 */
export type SideReading = {
  /**
   Category does not rest on this side at all.
   */
  readonly kind: 'unused';
} | {
  /**
   Category rests on this side, and both spans were located.
   */
  readonly kind: 'anchored';

  /**
   Span that identified which occurrence was meant.
   */
  readonly locator: AnchoredSpan;

  /**
   Smallest span carrying the claimed change, which is what corroboration
   compares.
   */
  readonly focus: AnchoredSpan;
};

/**
 One finding that survived screening.

 @example
 ```ts
 const finding: ScreenedFinding = { category: 'omission', source, candidate: { kind: 'unused', }, reason, };
 ```
 */
export type ScreenedFinding = {
  /**
   Category, narrowed to the vocabulary.
   */
  readonly category: RenderingAuditCategory;

  /**
   What this rests on in the original.
   */
  readonly source: SideReading;

  /**
   What it rests on in the candidate.
   */
  readonly candidate: SideReading;

  /**
   What the auditor said the spans amount to.
   */
  readonly reason: string;
};

/**
 What one auditor's answer amounts to once screened.

 @example
 ```ts
 const screened: ScreenedReport = { verdict: 'defects-found', findings: [], dropped: [], };
 ```
 */
export type ScreenedReport = {
  /**
   Verdict, narrowed to the vocabulary, or `uncertain` when the voice cast one
   this version does not know.
   */
  readonly verdict: RenderingAuditVerdict;

  /**
   Findings that proved themselves.
   */
  readonly findings: readonly ScreenedFinding[];

  /**
   Why each dropped claim was dropped, kept because a voice whose every claim
   fell is a different fact from a voice that claimed nothing.
   */
  readonly dropped: readonly string[];
};

/**
 Which sides a category rests on.

 @param category - category to ask about

 @returns Whether each side is required

 @throws {@link RenderingAuditInvariantError} when a category belongs to no
 anchoring rule, which is a vocabulary that grew without this rule growing
 with it

 @example
 ```ts
 const { needsSource, } = quotesRequired({ category: 'omission', },);
 ```
 */
export function quotesRequired(
  { category, }: { readonly category: RenderingAuditCategory; },
): {
  readonly needsSource: boolean;
  readonly needsCandidate: boolean;
} {
  if (SOURCE_ONLY_CATEGORIES.some(function names(one,): boolean {
    return one === category;
  },)) {
    return {
      needsSource: true,
      needsCandidate: false,
    };
  }

  if (CANDIDATE_ONLY_CATEGORIES.some(function names(one,): boolean {
    return one === category;
  },)) {
    return {
      needsSource: false,
      needsCandidate: true,
    };
  }

  // EVERY REMAINING CATEGORY IS PAIRED, stated as a check rather than assumed,
  // so a category added to the vocabulary and forgotten here is refused instead
  // of quietly treated as paired.
  if (!PAIRED_CATEGORIES.some(function names(one,): boolean {
    return one === category;
  },))
    throw new RenderingAuditInvariantError({
      invariant: `rendering audit category ${category} belongs to no anchoring rule`,
    },);

  return {
    needsSource: true,
    needsCandidate: true,
  };
}

/**
 Reads one side of one claim.

 @param text - side the claim names

 @param locator - span identifying which occurrence is meant

 @param focus - smallest span carrying the claimed change

 @param side - which side this is, for the refusal wording

 @param needed - whether this category rests on this side

 @returns What the side rests on, or why the claim falls

 @example
 ```ts
 const reading = readSide({ text, locator, focus, side: 'source', needed: true, },);
 ```
 */
function readSide(
  {
    text,
    locator,
    focus,
    side,
    needed,
  }: {
    readonly text: string;
    readonly locator: string;
    readonly focus: string;
    readonly side: string;
    readonly needed: boolean;
  },
): SideReading | { readonly dropped: string; } {
  if (!needed) {
    // A QUOTE HERE IS A CONTRADICTION, not a stray field to ignore: this
    // category says the side holds nothing to point at, and the claim points at
    // something anyway, so one of the two is wrong and neither can be trusted.
    // A field that shows a reader nothing quotes nothing, whatever its bytes,
    // and is the empty field the wire asks for here: asked of an empty-string
    // comparison, a space or a zero-width space a model put in the unused
    // field dropped a one-sided claim as two-sided (ledgers B40 and B128).
    if ((!rendersAsNothing({ text: locator, },)) || (!rendersAsNothing({ text: focus, },)))
      return { dropped: `forbidden-side-quote (${side})`, };

    return { kind: 'unused', };
  }

  /**
   Where the claim says it is.
   */
  const anchor = anchorLocatedSpan({
    text,
    locator,
    focus,
    side,
  },);

  if (!anchor.anchored)
    return { dropped: anchor.reason, };

  return {
    kind: 'anchored',
    locator: anchor.locator,
    focus: anchor.focus,
  };
}

/**
 Most UTF-16 units of a model-supplied word a drop reason may repeat; the cut
 ends on a whole character (`wholeOpening`), so a persisted run row never
 carries half an emoji.
 */
const DROPPED_WORD_LIMIT = 32;

/**
 Bounds a word a model supplied to what a drop reason needs.

 THE DIAGNOSTIC EXISTS TO SAY WHICH WORD, not to store prose. A drop reason
 is persisted in every run row, and a model answering the category or verdict
 field with a sentence would put that sentence in the run file whole. One
 token, cut at a fixed length on a whole character, names the word and
 carries nothing after it.

 @param word - category or verdict as the model wrote it

 @returns Its first whitespace-delimited token, at most the limit long

 @example
 ```ts
 const named = boundedWord({ word: 'altered whiskers, and more', },);
 ```
 */
function boundedWord({ word, }: { readonly word: string; },): string {
  /**
   Tokens, after folding newlines and tabs to spaces so a line break cannot
   carry a second one through.
   */
  const tokens = word
    .split('\n',)
    .join(' ',)
    .split('\t',)
    .join(' ',)
    .trim()
    .split(' ',);

  /**
   First of them, empty for an empty word.
   */
  const token = nonNullishOrThrow(tokens[0],);

  return wholeOpening({
    text: token,
    units: DROPPED_WORD_LIMIT,
  },);
}

/**
 Screens one claimed finding against both texts.

 @param finding - claim as the auditor sent it

 @param sourceText - original

 @param candidateText - rendering under audit

 @returns Finding with the texts' own spans, or why it was dropped

 @example
 ```ts
 const screened = screenFinding({ finding, sourceText, candidateText, },);
 ```
 */
function screenFinding(
  {
    finding,
    sourceText,
    candidateText,
  }: {
    readonly finding: RenderingAuditFindingWire;
    readonly sourceText: string;
    readonly candidateText: string;
  },
): ScreenedFinding | { readonly dropped: string; } {
  /**
   Category, when it is one this version names.

   FOUND RATHER THAN ASSERTED: the member comes back out of the vocabulary
   itself, so nothing here claims a string is a category it never checked.
   */
  const category = RENDERING_AUDIT_CATEGORIES.find(function isNamed(one,): boolean {
    return one === finding.category;
  },);

  if (category === undefined)
    return { dropped: `unknown-category (${boundedWord({ word: finding.category, },)})`, };

  /**
   Which sides this category rests on.
   */
  const {
    needsSource,
    needsCandidate,
  } = quotesRequired({ category, },);

  /**
   What the original side amounts to.
   */
  const source = readSide({
    text: sourceText,
    locator: finding.sourceLocator,
    focus: finding.sourceFocus,
    side: 'source',
    needed: needsSource,
  },);

  if ('dropped' in source)
    return source;

  /**
   What the candidate side amounts to.
   */
  const candidate = readSide({
    text: candidateText,
    locator: finding.candidateLocator,
    focus: finding.candidateFocus,
    side: 'candidate',
    needed: needsCandidate,
  },);

  if ('dropped' in candidate)
    return candidate;

  return {
    category,
    source,
    candidate,
    reason: finding.reason,
  };
}

/**
 Screens one auditor's whole answer.

 @param report - reply as the wire guard accepted it

 @param sourceText - original, whose own comments stay in the candidate as the
 sheet showed it

 @param candidateText - rendering under audit as stored, read as the sheet
 showed it: without the notes between translators

 @returns Findings that proved themselves, and why the rest fell

 @example
 ```ts
 const screened = screenRenderingAudit({ report, sourceText, candidateText, },);
 ```
 */
export function screenRenderingAudit(
  {
    report,
    sourceText,
    candidateText,
  }: {
    readonly report: RenderingAuditReportWire;
    readonly sourceText: string;
    readonly candidateText: string;
  },
): ScreenedReport {
  // THE AUDITOR'S LOCATORS ARE READ IN THE CANDIDATE AS THE SHEET SHOWED IT, without the notes between
  // translators, cut against the original as `buildRenderingAuditMessages` cuts it: a locator across the place
  // a note was cut from stands in no stored candidate. The spans found are offsets of that shown text, and
  // corroboration compares them with each other only, all in the one text.
  /**
   The candidate as the auditor read it.
   */
  const candidateShown = withoutTranslatorsNotes({
    text: candidateText,
    original: sourceText,
  },);

  /**
   Every claim, screened.
   */
  const screened = report.findings
    .map(function screenOne(finding,) {
      return screenFinding({
        finding,
        sourceText,
        candidateText: candidateShown,
      },);
    },);

  /**
   Verdict this version knows, or the absence of one.

   A VERDICT THIS VERSION DOES NOT KNOW READS AS `uncertain` rather than as a
   refusal of the whole answer: the findings underneath it are checked against
   the texts either way, and a mis-cast verdict is not a reason to discard
   evidence that anchors. The substitution is RECORDED in the drop list, since
   an unknown verdict is a protocol failure and silently reading it as an
   epistemic state would hide one instrument defect behind a vocabulary word.
   */
  const cast = RENDERING_AUDIT_VERDICTS.find(function isCast(one,): boolean {
    return one === report.verdict;
  },);

  return {
    verdict: cast ?? 'uncertain',
    findings: screened.filter(function survived(one,): one is ScreenedFinding {
      return !('dropped' in one);
    },),
    dropped: [
      ...((cast === undefined) ? [`unknown-verdict (${boundedWord({ word: report.verdict, },)})`,] : []),
      ...screened
        .filter(function fell(one,): one is { readonly dropped: string; } {
          return 'dropped' in one;
        },)
        .map(function toReason(one,): string {
          return one.dropped;
        },),
    ],
  };
}

//endregion Rendering audit screen
