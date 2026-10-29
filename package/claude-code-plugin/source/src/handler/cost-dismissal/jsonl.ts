/**
 Shared sentinel for JSONL parsing across the cost-dismissal modules.

 Absence is modeled with {@link NOT_PARSED},
 never with a `| undefined` union.

 @module
 */

/**
 Sentinel returned when a line does not parse into its record shape.
 */
const NOT_PARSED = 'not-parsed' as const;

export { NOT_PARSED, };
