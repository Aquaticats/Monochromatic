/**
 The verdict `native-perf.ts` reaches from a finished run.

 `PP6` says a failing process signals through the thrown error, which already exits nonzero, so neither a sample the run
 could not measure nor a judged figure past its bound sets an exit code here: each is the message of a thrown error, and
 the throw is what decides the run's status.
 */
import type { BoundViolation } from './native-perf-judge.ts';
import { violationLine } from './native-perf-lines.ts';
import type { SampleFailure } from './native-perf-measure.ts';

/**
 One failed sample as the thrown message names it: which arm of which scenario, which round, and what Git said.
 
 @param failure - the failed sample
 
 @returns one line naming the sample and its reason
 
 @example
 ```ts
 describeFailure({ scenario: 'forward', wrapped: true, sample: 3, status: 128, stderr: 'fatal' });
 // => 'forward (wrapped) round 3 exited 128: fatal'
 ```
 */
function describeFailure(failure: SampleFailure): string {
  /**
   Which side of the paired run the failed sample belongs to.
   */
  const arm = failure.wrapped
    ? 'wrapped'
    : 'direct';
  /**
   How the failed sample ended, naming the exit status when the command produced one.
   */
  const status = failure.status === undefined
    ? 'exited nonzero'
    : `exited ${String(failure.status,)}`;
  return `${failure.scenario} (${arm}) round ${String(failure.sample,)} ${status}: ${failure.stderr}`;
}

/**
 Samples a finished run could not measure, so every figure beside them is not evidence.
 */
export class PerfSamplesFailedError extends Error {
  /**
   Stable class name that tells an unmeasurable run apart from an unrelated error in a log.
   */
  override name = 'PerfSamplesFailedError';

  /**
   Creates the verdict of a run with unmeasured samples.
   
   @param failures - the failed samples, at least one
   
   @example
   ```ts
   throw new PerfSamplesFailedError(failures);
   ```
   */
  constructor(failures: readonly SampleFailure[]) {
    super([
      `${String(failures.length,)} ${
        failures.length === 1
          ? 'sample'
          : 'samples'
      } failed, so the run measured too little to judge:`,
      ...failures.map(describeFailure,),
    ].join('\n',),);
  }
}

/**
 Requirements a finished run's judged figures broke: the answer the measurement gave is that the wrapper is too slow.
 */
export class PerfBoundsExceededError extends Error {
  /**
   Stable class name that tells a slow wrapper apart from an unrelated error in a log.
   */
  override name = 'PerfBoundsExceededError';

  /**
   Creates the verdict of a run whose measured latency broke a requirement.
   
   @param violations - the broken requirements, at least one
   
   @example
   ```ts
   throw new PerfBoundsExceededError(violations);
   ```
   */
  constructor(violations: readonly BoundViolation[]) {
    super([
      `${String(violations.length,)} ${
        violations.length === 1
          ? 'requirement was'
          : 'requirements were'
      } not met:`,
      ...violations.map(function printViolation(violation,) {
        return violationLine(violation,);
      },),
    ].join('\n',),);
  }
}

/**
 Decides a finished run's status by throwing over what it found, after its report lines are printed.
 
 A sample the run could not measure is reported before a broken requirement, because figures computed beside a failed
 sample are not evidence of anything.
 
 @param failures - samples that could not be measured
 
 @param violations - requirements the judged statistics reached
 
 @throws {@link PerfSamplesFailedError} when any sample could not be measured.
 
 @throws {@link PerfBoundsExceededError} when the run measured, and broke, a requirement.
 
 @example
 ```ts
 assertPerfVerdict({ failures: [], violations: [] });
 ```
 */
export function assertPerfVerdict({
  failures,
  violations,
}: {
  readonly failures: readonly SampleFailure[];
  readonly violations: readonly BoundViolation[];
},): void {
  if (failures.length > 0)
    throw new PerfSamplesFailedError(failures,);

  if (violations.length > 0)
    throw new PerfBoundsExceededError(violations,);
}
