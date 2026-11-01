import type {
  AdjudicatedIssue,
  SpanAnchor,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { flattenSpace, } from './sheet-line-text.ts';

//region Resolution sheet evidence
// WHAT THE CHECKER SHEET SHOWS BESIDE THE DOCUMENTS (ledger L14, 2026-09-28).
// The sheet showed the ORIGINAL, the REVISED TRANSLATION and each claim's
// category, severity and summary. The panel that accepted the issue also saw
// the declared names with their rules, the cited references and the claim's
// own quotes; the checker saw none of them, while its `worse` ballots strip
// an edit (ledger L3) and roll a rewrite back (ledger L11).

/**
 How a checker reads the quotes under each claim, stated on every checker
 sheet.

 THE TRANSLATION QUOTES ARE THE TEXT BEFORE THE REVISION: the critic quoted
 the archive wording it objected to, which the REVISED TRANSLATION may no
 longer carry, so a checker told nothing would search the revised text for
 it. JSON strings because 349 of 45,860 quotes over every artifact span more
 than one line, and a raw one would open lines of its own in a sheet whose
 issues are told apart by the lines that open them.

 @example
 ```ts
 const system = `${RULES}\n\n${CHECKER_QUOTE_RULE}`;
 ```
 */
export const CHECKER_QUOTE_RULE: string = 'Each claim lists its evidence under it as JSON strings: text quoted from the ORIGINAL,'
  + ' and text quoted from the TRANSLATION as it stood before this revision, which the REVISED TRANSLATION'
  + ' may no longer carry. An insertion point marks where the claim says content was missing.';

/**
 What a checker does with a rendering the DECLARED NAMES block settles,
 stated after the rules every sheet reads the block by.

 @example
 ```ts
 const system = `${DECLARED_IDENTITY_RULES}\n${CHECKER_IDENTITY_RULE}`;
 ```
 */
export const CHECKER_IDENTITY_RULE: string = '- A revision that renders a declared name, handle or place name as the block'
  + ' declares it has not damaged it; a revision that moves one away from its declared value is worse.';

/**
 What a checker does with the cited references.

 THEY NEVER REOPEN AN ACCEPTED ISSUE. The panel judged every claim with the
 references on its sheet, and the reference screen voided each addition claim
 on a detail a reference states word for word (class thirty-seven), so an
 accepted accuracy/addition issue reaching the checkers survived both. The
 checker judges whether the revision fixed what was accepted, on less
 evidence than the panel had, and is no second panel.

 @example
 ```ts
 const system = `${RULES}\n\n${CHECKER_REFERENCE_RULE}`;
 ```
 */
export const CHECKER_REFERENCE_RULE: string = 'Cited references, when that block follows the REVISED TRANSLATION:'
  + ' a detail the REVISED TRANSLATION carries that a cited reference states is not damage, and never a reason'
  + ' for worse. The references never reopen an issue the panel accepted: an accepted accuracy/addition issue is'
  + ' still fixed only by removing its detail.';

/**
 A quote as a JSON string that holds no line break of any kind.

 `JSON.stringify` escapes line feed, carriage return, vertical tab and form
 feed, and leaves next line, line separator and paragraph separator raw; none
 of the 45,860 quotes over every artifact carried one, and a reader taking one
 as a line end would see the quote open a line (ledger L14(d)).

 @param text - quote to encode

 @returns JSON string literal on one line

 @example
 ```ts
 const encoded = jsonLine({ text: 'hunts\u2028at noon', },);
 ```
 */
export function jsonLine({ text, }: { readonly text: string; },): string {
  return JSON.stringify(text,)
    .replaceAll(
    '\u0085',
    String.raw`\u0085`,
    )
    .replaceAll(
    '\u2028',
    String.raw`\u2028`,
    )
    .replaceAll(
    '\u2029',
    String.raw`\u2029`,
    );
}

/**
 One evidence line under a claim.

 @param span - anchored evidence the critic quoted

 @returns Indented line naming where the quote comes from, the quote
 JSON-encoded, or the insertion-point wording

 @example
 ```ts
 const line = resolutionEvidenceLine({ span, },);
 ```
 */
function resolutionEvidenceLine({ span, }: { readonly span: SpanAnchor; },): string {
  /**
   Where the quote comes from, in the sheet's vocabulary.
   */
  const place = (span.side === 'source') ? 'the ORIGINAL' : 'the TRANSLATION before this revision';
  if (span.startOffset === span.endOffset)
    return `  - insertion point in ${place}: the claim says content is missing there`;
  return `  - quoted from ${place}: ${jsonLine({ text: span.quotedText, },)}`;
}

/**
 One numbered issue block: each claim's line, then its evidence.

 @param issue - accepted issue the editors addressed

 @param index - zero-based place in the sheet's numbering

 @returns Block opening with its `ISSUE` line

 @example
 ```ts
 const block = resolutionIssueBlock({ issue, index: 0, },);
 ```
 */
export function resolutionIssueBlock(
  {
    issue,
    index,
  }: {
    readonly issue: AdjudicatedIssue;
    readonly index: number;
  },
): string {
  /**
   Claim lines, each followed by its evidence lines.
   */
  const claimLines = issue.claims
    .flatMap(function toLines(member,) {
      return [
        `- (${member.claim
          .category}, ${issue.severity}): ${flattenSpace({ text: member.claim
            .summary, },)}`,
        ...member.claim
          .spans
          .map(function toLine(span,) {
            return resolutionEvidenceLine({ span, },);
          },),
      ];
    },);

  return `ISSUE ${String(index + 1,)}
${claimLines.join('\n',)}`;
}

//endregion Resolution sheet evidence
