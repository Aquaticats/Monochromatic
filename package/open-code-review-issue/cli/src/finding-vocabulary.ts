/**
 OCR category and severity runtime vocabulary.
 
 @module
 */

import { InputValidationError, } from './input-validation-error.ts';
import type {
  FindingCategory,
  FindingSeverity,
} from './model.ts';

/**
 Accepted OCR categories for runtime narrowing.
 */
const FINDING_CATEGORIES: ReadonlySet<string> = new Set([
  'bug',
  'security',
  'performance',
  'maintainability',
  'test',
  'style',
  'documentation',
  'other',
],);

/**
 Accepted OCR severities for runtime narrowing.
 */
const FINDING_SEVERITIES: ReadonlySet<string> = new Set([
  'critical',
  'high',
  'medium',
  'low',
],);

/**
 Narrows validated category text to OCR's declared category union.
 
 @param value - Category candidate from one comment.
 
 @returns Whether candidate belongs to supported category vocabulary.
 
 @example
 ```ts
 isFindingCategory('bug'); // true
 ```
 */
function isFindingCategory(value: string,): value is FindingCategory {
  return FINDING_CATEGORIES.has(value,);
}

/**
 Narrows validated severity text to OCR's declared severity union.
 
 @param value - Severity candidate from one comment.
 
 @returns Whether candidate belongs to supported severity vocabulary.
 
 @example
 ```ts
 isFindingSeverity('high'); // true
 ```
 */
function isFindingSeverity(value: string,): value is FindingSeverity {
  return FINDING_SEVERITIES.has(value,);
}

/**
 Normalizes optional category after case and whitespace folding.
 
 @param record - Comment record carrying category.
 
 @param positionLabel - Input position for diagnostic evidence.
 
 @returns Supported category or absence.
 
 @throws {@link InputValidationError} when supplied category is unsupported.
 
 @example
 ```ts
 normalizeCategory({ record: { category: ' BUG ' }, positionLabel: 'record 1' });
 ```
 */
export function normalizeCategoryMetadata({
  record,
  positionLabel,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly positionLabel: string;
},): { readonly category?: FindingCategory; } {
  /**
   Untrusted optional category value.
   */
  const value = record.category;
  if ((value === undefined) || (value === '')) {
    return {};
  }
  if ((typeof value) !== 'string') {
    throw new InputValidationError(
      `${positionLabel} property category must be a string`,
    );
  }
  /**
   Case-folded category used by policy comparisons.
   */
  const normalized = value.trim()
    .toLowerCase();
  if (!isFindingCategory(normalized,)) {
    throw new InputValidationError(
      `${positionLabel} has unsupported category ${value}`,
    );
  }
  return { category: normalized, };
}

/**
 Normalizes optional severity after case and whitespace folding.
 
 @param record - Comment record carrying severity.
 
 @param positionLabel - Input position for diagnostic evidence.
 
 @returns Supported severity or absence.
 
 @throws {@link InputValidationError} when supplied severity is unsupported.
 
 @example
 ```ts
 normalizeSeverity({ record: { severity: ' HIGH ' }, positionLabel: 'record 1' });
 ```
 */
export function normalizeSeverityMetadata({
  record,
  positionLabel,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly positionLabel: string;
},): { readonly severity?: FindingSeverity; } {
  /**
   Untrusted optional severity value.
   */
  const value = record.severity;
  if ((value === undefined) || (value === '')) {
    return {};
  }
  if ((typeof value) !== 'string') {
    throw new InputValidationError(
      `${positionLabel} property severity must be a string`,
    );
  }
  /**
   Case-folded severity used by display and policy.
   */
  const normalized = value.trim()
    .toLowerCase();
  if (!isFindingSeverity(normalized,)) {
    throw new InputValidationError(
      `${positionLabel} has unsupported severity ${value}`,
    );
  }
  return { severity: normalized, };
}
