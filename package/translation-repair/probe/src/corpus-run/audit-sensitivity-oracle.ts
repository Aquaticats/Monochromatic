import type {
  AuditVoiceRow,
  ScreenedFinding,
  SideReading,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  ORACLE_CANDIDATE_SPAN,
  ORACLE_SOURCE_SPAN,
} from './audit-sensitivity-input.ts';

//region Audit sensitivity oracle
// Scores what an auditor pointed at against the span the defect was planted
// in, loosely on purpose: whether the auditor looked in the right place at
// all, which is a different question from the matcher's.

/**
 Wording one side of a finding rests on, empty where it rests on none.

 @param reading - one side of a screened finding

 @returns Focus wording, or empty for a side the category does not use

 @example
 ```ts
 const quoted = focusText({ reading: finding.source, },);
 ```
 */
function focusText({ reading, }: { readonly reading: SideReading; },): string {
  if (reading.kind !== 'anchored')
    return '';

  return reading.focus
    .text;
}

/**
 Whether one quoted span and the oracle span are about the same wording.

 @param span - oracle wording

 @param quoted - wording the auditor pointed at

 @returns Whether either contains the other

 @example
 ```ts
 const near = meetsOracle({ span: ORACLE_SOURCE_SPAN, quoted, },);
 ```
 */
function meetsOracle(
  {
    span,
    quoted,
  }: {
    readonly span: string;
    readonly quoted: string;
  },
): boolean {
  if (quoted === '')
    return false;

  return span.includes(quoted,) || quoted.includes(span,);
}

/**
 Wording to print for one side, or a dash where it rests on none.

 @param reading - one side of a screened finding

 @returns Focus wording, or a dash

 @example
 ```ts
 const shown = shownText({ reading: defect.source, },);
 ```
 */
export function shownText({ reading, }: { readonly reading: SideReading; },): string {
  /**
   What this side rests on.
   */
  const quoted = focusText({ reading, },);

  return (quoted === '') ? '-' : quoted;
}

/**
 Whether one screened finding points at the planted defect.

 BY CONTAINMENT EITHER WAY, deliberately loose: this is not the matcher and
 must not inherit its strictness. The question here is whether the auditor
 looked in the right place at all, so a voice quoting the whole clause and one
 quoting the negator both count.

 @param finding - claim to check

 @returns Whether either side's focus meets the oracle span

 @example
 ```ts
 const hit = pointsAtOracle({ finding, },);
 ```
 */
export function pointsAtOracle({ finding, }: { readonly finding: ScreenedFinding; },): boolean {
  /**
   Whether the original side names the planted clause.
   */
  const source = meetsOracle({
    span: ORACLE_SOURCE_SPAN,
    quoted: focusText({ reading: finding.source, },),
  },);

  /**
   Whether the candidate side names it.
   */
  const candidate = meetsOracle({
    span: ORACLE_CANDIDATE_SPAN,
    quoted: focusText({ reading: finding.candidate, },),
  },);

  return source || candidate;
}

/**
 Voices that pointed at the planted defect at least once.

 @param rows - every auditor's screened answer

 @returns The voices that did

 @example
 ```ts
 const sighted = sightedVoices({ rows: report.rows, },);
 ```
 */
export function sightedVoices({ rows, }: { readonly rows: readonly AuditVoiceRow[]; },): readonly AuditVoiceRow[] {
  return rows
    .filter(function sawIt(row,): boolean {
      return row.findings
        .some(function atOracle(finding,): boolean {
          return pointsAtOracle({ finding, },);
        },);
    },);
}

//endregion Audit sensitivity oracle
