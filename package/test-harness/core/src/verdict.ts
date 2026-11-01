import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 Logger set whose rightmost tag identifies test-runner outcome.
 
 @example
 ```ts
 const verdictLoggers = createVerdictLoggers({ l, });
 verdictLoggers.pass.info('(1.2ms)');
 ```
 */
export type VerdictLoggers = {
  /**
   Logger tagged with `FAIL`.
   */
  readonly fail: Logger;
  /**
   Logger tagged with `PASS`.
   */
  readonly pass: Logger;
  /**
   Logger tagged with `SKIP`.
   */
  readonly skip: Logger;
};

/**
 Composes outcome tags after existing suite and test hierarchy tags.
 
 @param l - hierarchy-tagged logger receiving outcome wrappers
 
 @returns loggers for each runner outcome
 
 @example
 ```ts
 const verdictLoggers = createVerdictLoggers({ l, });
 verdictLoggers.fail.error('(2.4ms)');
 ```
 */
export function createVerdictLoggers(
  { l, }: { readonly l: Logger; },
): VerdictLoggers {
  return {
    fail: tagged({
      tag: 'FAIL',
      l,
    },),
    pass: tagged({
      tag: 'PASS',
      l,
    },),
    skip: tagged({
      tag: 'SKIP',
      l,
    },),
  };
}
