/**
 Deterministic GitHub Issue title generation.
 
 @module
 */

import type { NormalizedFinding, } from './model.ts';

/**
 Maximum complete title size owned by adapter contract.
 */
const TITLE_MAX_BYTES = 256;

/**
 Bytes reserved for truncation ellipsis.
 */
const TITLE_ELLIPSIS_BYTES = 3;

/**
 Visible truncation suffix.
 */
const TITLE_ELLIPSIS = '…';

/**
 Finds first line containing non-whitespace text.
 
 @param text - Candidate summary field.
 
 @returns Trimmed first meaningful line or empty string when none exists.
 
 @example
 ```ts
 firstMeaningfulLine('\n  summary '); // 'summary'
 ```
 */
function firstMeaningfulLine(text: string,): string {
  for (const line of text.split('\n',)) {
    /**
     Candidate line without surrounding whitespace.
     */
    const trimmed = line.trim();
    if (trimmed !== '') {
      return trimmed;
    }
  }
  return '';
}

/**
 Chooses content then existing and suggested code summary fallback.
 
 @param finding - Validated finding with at least one meaningful text line.
 
 @returns First available deterministic title summary.
 
 @throws {@link Error} when caller bypasses validated finding invariant.
 
 @example
 ```ts
 findingSummary({ content: 'Summary' } as NormalizedFinding); // 'Summary'
 ```
 */
function findingSummary(finding: NormalizedFinding,): string {
  for (const text of [
    finding.content,
    finding.existingCode,
    finding.suggestionCode,
  ]) {
    /**
     First meaningful line in current priority field.
     */
    const summary = firstMeaningfulLine(text,);
    if (summary !== '') {
      return summary;
    }
  }
  throw new Error('validated finding has no non-whitespace title source',);
}

/**
 Keeps longest code-point prefix fitting byte budget.
 
 @param text - Complete overlength title.
 
 @param maximumBytes - Prefix budget excluding suffix.
 
 @returns Valid UTF-8 prefix within byte budget.
 
 @example
 ```ts
 utf8Prefix({ text: 'abc', maximumBytes: 2 }); // 'ab'
 ```
 */
function utf8Prefix({
  text,
  maximumBytes,
}: {
  readonly text: string;
  readonly maximumBytes: number;
},): string {
  /**
   Shared UTF-8 encoder for code-point byte measurement.
   */
  const encoder = new TextEncoder();
  /**
   Mutable scan state scoped behind one constant binding.
   */
  const state: {
    readonly accepted: string[];
    bytes: number;
  } = {
    accepted: [],
    bytes: 0,
  };
  for (const character of text) {
    /**
     UTF-8 byte width of current code point.
     */
    const characterBytes = encoder
      .encode(character,)
      .length;
    if ((state.bytes + characterBytes) > maximumBytes) {
      break;
    }
    state.accepted
      .push(character,);
    state.bytes += characterBytes;
  }
  return state.accepted
    .join('',);
}

/**
 Applies adapter-owned complete-title UTF-8 byte cap.
 
 @param title - Generated title including optional triage prefix.
 
 @returns Unchanged title or deterministic ellipsis truncation.
 
 @example
 ```ts
 capIssueTitle('short'); // 'short'
 ```
 */
export function capIssueTitle(title: string,): string {
  /**
   UTF-8 encoder used for complete-title size decision.
   */
  const encoder = new TextEncoder();
  if (encoder.encode(title,)
    .length
    <= TITLE_MAX_BYTES) {
    return title;
  }
  /**
   Longest valid prefix before trailing whitespace removal.
   */
  const prefix = utf8Prefix({
    text: title,
    maximumBytes: TITLE_MAX_BYTES - TITLE_ELLIPSIS_BYTES,
  });
  return `${prefix.trimEnd()}${TITLE_ELLIPSIS}`;
}

/**
 Builds complete deterministic title including label fallback prefix.
 
 @param finding - Validated normalized OCR finding.
 
 @param needsTriageLabel - Whether destination label exists.
 
 @returns Complete capped GitHub Issue title.
 
 @example
 ```ts
 renderIssueTitle({ finding, needsTriageLabel: true });
 ```
 */
export function renderIssueTitle({
  finding,
  needsTriageLabel,
}: {
  readonly finding: NormalizedFinding;
  readonly needsTriageLabel: boolean;
},): string {
  /**
   Category title token including missing-category fallback.
   */
  const category = finding.category ?? 'uncategorized';
  /**
   Existing-label fallback visible when label cannot be applied.
   */
  const triagePrefix = needsTriageLabel ? '' : '[needs-triage] ';
  return capIssueTitle(
    `${triagePrefix}[${category}] ${finding.path}: ${findingSummary(finding,)}`,
  );
}
