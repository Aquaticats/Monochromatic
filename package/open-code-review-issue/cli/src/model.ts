/**
 Normalized OCR finding and input contracts.
 
 @module
 */

//region Finding metadata

/**
 OCR finding categories accepted by the adapter.
 
 @example
 ```ts
 const category: FindingCategory = 'security';
 ```
 */
export type FindingCategory =
  | 'bug'
    | 'security'
    | 'performance'
    | 'maintainability'
    | 'test'
    | 'style'
    | 'documentation'
    | 'other';

/**
 OCR finding severities accepted by the adapter.
 
 @example
 ```ts
 const severity: FindingSeverity = 'high';
 ```
 */
export type FindingSeverity = 'critical' | 'high' | 'medium' | 'low';

/**
 Source position used for diagnostics without repeating finding content.
 
 @example
 ```ts
 const position: InputPosition = { kind: 'record', value: 1 };
 ```
 */
export type InputPosition = {
  readonly kind: 'record' | 'line';
  readonly value: number;
};

/**
 Validated finding independent of OCR's input envelope.
 
 @example
 ```ts
 const finding: NormalizedFinding = {
   position: { kind: 'record', value: 1 },
   path: 'src/example.ts',
   content: 'Boundary handling is incorrect.',
   existingCode: '',
   suggestionCode: '',
   startLine: 4,
   endLine: 4,
   category: 'bug',
   severity: 'high',
 };
 ```
 */
export type NormalizedFinding = {
  readonly position: InputPosition;
  readonly path: string;
  readonly content: string;
  readonly existingCode: string;
  readonly suggestionCode: string;
  readonly startLine: number;
  readonly endLine: number;
  readonly category?: FindingCategory;
  readonly severity?: FindingSeverity;
};

//endregion Finding metadata

//region Input envelope

/**
 Recognized OCR envelope after atomic validation.
 
 @example
 ```ts
 const input: NormalizedInput = {
   inputKind: 'comments',
   findings: [],
 };
 ```
 */
export type NormalizedInput = {
  readonly inputKind: 'result' | 'comments' | 'jsonl';
  readonly resolvedHead?: string;
  readonly findings: readonly NormalizedFinding[];
};

//endregion Input envelope
