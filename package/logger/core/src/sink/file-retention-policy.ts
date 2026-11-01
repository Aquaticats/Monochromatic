/**
 Retention bounds for JSONL log trees, and their environment overrides. @module
 */

//region Policy names the bounds, never the sweep mechanics.
/**
 Bounds the log tree one destination directory may hold.
 */
export type LogRetentionPolicy = {
  /**
   Whole days of history kept before day directories expire.
   */
  readonly maxAgeDays: number;
  /**
   Total bytes of kept files before the oldest files go.
   */
  readonly maxBytes: number;
  /**
   Total kept files before the oldest files go.
   */
  readonly maxFiles: number;
};

/**
 Two weeks of history,
 the window the issue 576 investigation itself spanned
 (2026-09-26 to 2026-10-20).
 */
export const DEFAULT_LOG_MAX_AGE_DAYS = 14;

/**
 512 MiB of JSONL per destination directory,
 several times the measured peak day's payload
 (107,082 files at a 209 byte median).
 */
export const DEFAULT_LOG_MAX_BYTES = 536_870_912;

/**
 50,000 files per destination directory,
 below the measured peak day
 (107,082 files on 2026-10-12)
 so one busy day cannot park the whole budget in the live directory.
 */
export const DEFAULT_LOG_MAX_FILES = 50_000;

/**
 Opt-out sentinel for sinks whose caller-owned directories stay unswept.
 A unique symbol rather than a falsy literal,
 so `retention` keeps one honest value type.
 */
export const NO_LOG_RETENTION_SWEEP: unique symbol = Symbol('retention sweep skipped for a caller-owned destination directory');

/**
 Environment override accepted only when finite and nonnegative,
 so a typo cannot disable retention or make it delete everything.

 @param environment - runtime environment read for the override

 @param key - variable name holding the numeric override

 @param fallback - default used for a missing or invalid override

 @returns Effective numeric bound

 @example
 ```ts
 const maxAgeDays = retentionBound({ environment, key: 'MONOCHROMATIC_LOG_MAX_AGE_DAYS', fallback: 14 });
 ```
 */
function retentionBound({
  environment,
  key,
  fallback,
}: {
  readonly environment: Readonly<NodeJS.ProcessEnv>;
  readonly key: string;
  readonly fallback: number;
},): number {
  /**
   Configured override text, absent when the variable is unset or blank.
   */
  const configured = environment[key];
  if (configured === undefined)
    return fallback;
  if (configured.trim() === '')
    return fallback;
  /**
   Numeric reading of the override, rejecting text that is not a finite number.
   */
  const parsed = Number(configured,);
  return (Number.isFinite(parsed,) && (parsed >= 0))
    ? parsed
    : fallback;
}

/**
 Environment overrides for the retention bounds,
 named like `MONOCHROMATIC_LOG_DIRECTORY`.

 @param environment - runtime environment read for the overrides

 @returns Policy whose missing or invalid overrides fall back to defaults

 @example
 ```ts
 const policy = logRetentionPolicyFromEnv();
 ```
 */
export function logRetentionPolicyFromEnv({
  environment = process.env,
}: Readonly<{ environment?: Readonly<NodeJS.ProcessEnv> }> = {},): LogRetentionPolicy {
  return {
    maxAgeDays: retentionBound({
      environment,
      key: 'MONOCHROMATIC_LOG_MAX_AGE_DAYS',
      fallback: DEFAULT_LOG_MAX_AGE_DAYS,
    },),
    maxBytes: retentionBound({
      environment,
      key: 'MONOCHROMATIC_LOG_MAX_BYTES',
      fallback: DEFAULT_LOG_MAX_BYTES,
    },),
    maxFiles: retentionBound({
      environment,
      key: 'MONOCHROMATIC_LOG_MAX_FILES',
      fallback: DEFAULT_LOG_MAX_FILES,
    },),
  };
}
//endregion Policy names the bounds, never the sweep mechanics.
