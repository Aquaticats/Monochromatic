import {
  isRecord,
  parseJson,
} from './json-value.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import { parseRunCreatedAt } from './run-results.ts';

/**
 Status a run and each of its jobs reports once it has ended; every other status means the run is still going, whether
 it is queued, waiting for a runner or running a job.
 */
export const completedStatus = 'completed';

/**
 Conclusion of a run or a job that passed.
 */
export const successConclusion = 'success';

/**
 Conclusions a job may hold without having failed its run: a skipped job never ran, and a neutral one reported nothing
 to fail. Every other conclusion of a finished job names what stopped the run.
 */
export const passingConclusions: readonly string[] = [
  'success',
  'skipped',
  'neutral',
];

/**
 One step of a job, as `gh run view --json jobs` prints it.
 */
export type RunStep = {
  readonly name: string;
  readonly number: number;
  readonly status: string;
  readonly conclusion: string;
};

/**
 One job of a run, as `gh run view --json jobs` prints it.
 */
export type RunJob = {
  readonly name: string;
  readonly status: string;
  readonly conclusion: string;
  readonly steps: readonly RunStep[];
};

/**
 What a run itself says about itself: its creation time, how far it got and what each of its jobs concluded.

 This is the only verdict the release helpers accept. `gh run watch --exit-status` exits 1 both for a run that concluded
 otherwise than success and for a failure of the watcher itself, such as an API call that errored while the run was
 still going, so its status says nothing about the run.
 */
export type RunOutcome = {
  readonly createdAt: string;
  readonly status: string;
  readonly conclusion: string;
  readonly jobs: readonly RunJob[];
};

/**
 Read a conclusion that may be absent, which `gh` prints as an empty text.

 @param value - the field as the answer held it

 @returns the conclusion, or an empty text when the run has ended in no conclusion yet

 @example
 ```ts
 conclusionOf(null);
 ```
 */
function conclusionOf(value: unknown): string {
  return ((typeof value) === 'string') ? value : '';
}

/**
 Read the steps of one job.

 @param job - one job as the answer held it

 @returns the steps in the order the answer lists them; empty when the job reports none

 @example
 ```ts
 parseSteps({ steps: [] });
 ```
 */
function parseSteps(job: Readonly<Record<string, unknown>>): readonly RunStep[] {
  if (!Array.isArray(job.steps))
    return [];
  return (job.steps as readonly unknown[])
    .flatMap(function toStep(step) {
      return isRecord(step)
        && ((typeof step.name) === 'string')
        && ((typeof step.status) === 'string')
        ? [{
          name: step.name as string,
          number: ((typeof step.number) === 'number') ? step.number as number : 0,
          status: step.status as string,
          conclusion: conclusionOf(step.conclusion),
        }]
        : [];
    });
}

/**
 Read the jobs of a run.

 @param jobs - the `jobs` field of the answer

 @returns the jobs the answer lists; empty when it lists none

 @example
 ```ts
 parseJobs([]);
 ```
 */
function parseJobs(jobs: unknown): readonly RunJob[] {
  if (!Array.isArray(jobs))
    return [];
  return (jobs as readonly unknown[])
    .flatMap(function toJob(job) {
      return isRecord(job)
        && ((typeof job.name) === 'string')
        && ((typeof job.status) === 'string')
        ? [{
          name: job.name as string,
          status: job.status as string,
          conclusion: conclusionOf(job.conclusion),
          steps: parseSteps(job),
        }]
        : [];
    });
}

/**
 Read what a run says about itself from the answer of `gh run view`.

 @param body - JSON text printed by `gh run view --json createdAt,status,conclusion,jobs`

 @returns the outcome, or a miss that says what the answer lacks

 @example
 ```ts
 parseRunOutcome('{"createdAt":"2026-11-11T00:55:54Z","status":"completed","conclusion":"success","jobs":[]}');
 ```
 */
export function parseRunOutcome(body: string): Lookup<RunOutcome> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  if (!isRecord(answer.value))
    return missingValue('The run answer is not a JSON object.');
  /**
   Creation time of the run, which the callers order runs by.
   */
  const createdAt = parseRunCreatedAt(body);
  if (!createdAt.found)
    return createdAt;
  /**
   How the run stands, as its own answer reports it.
   */
  const {
    status,
    conclusion,
    jobs
  } = answer.value;
  if ((typeof status) !== 'string')
    return missingValue('The run answer holds no status.');
  return foundValue({
    createdAt: createdAt.value,
    status,
    conclusion: conclusionOf(conclusion),
    jobs: parseJobs(jobs),
  });
}

/**
 Say whether a run has ended.

 @param outcome - what the run says about itself

 @returns true when the run has ended, whatever it concluded

 @example
 ```ts
 isRunCompleted(outcome);
 ```
 */
export function isRunCompleted(outcome: RunOutcome): boolean {
  return outcome.status === completedStatus;
}

/**
 Say whether a run ended successfully.

 @param outcome - what the run says about itself

 @returns true only for a run that ended with the success conclusion

 @example
 ```ts
 runPassed(outcome);
 ```
 */
export function runPassed(outcome: RunOutcome): boolean {
  return outcome.conclusion === successConclusion;
}

/**
 List the jobs of a run that have not ended, with their status.

 @param outcome - what the run says about itself

 @returns texts such as `gpc-build-binary (in_progress)`; empty when no job is going

 @example
 ```ts
 jobsStillGoing(outcome);
 ```
 */
export function jobsStillGoing(outcome: RunOutcome): readonly string[] {
  return outcome.jobs
    .filter(function isGoing(job) {
      return job.status !== completedStatus;
    })
    .map(function describe(job) {
      return `${job.name} (${job.status})`;
    });
}

/**
 Describe one run in one line, from what the run itself says.

 @param workflow - workflow file of the run

 @param runId - numeric run identifier

 @param outcome - what the run says about itself

 @returns a line that says whether the run ended and how, or which jobs it is still running

 @example
 ```ts
 describeRunOutcome({ workflow: 'cargo-publish.yml', runId: '42', outcome });
 ```
 */
export function describeRunOutcome({
  workflow,
  runId,
  outcome,
}: {
  readonly workflow: string;
  readonly runId: string;
  readonly outcome: RunOutcome;
}): string {
  if (isRunCompleted(outcome))
    return `${workflow} run ${runId}: completed with ${outcome.conclusion === '' ? 'no conclusion' : outcome.conclusion}`;
  /**
   Jobs the run has not ended, when it reports any.
   */
  const going = jobsStillGoing(outcome);
  return going.length === 0
    ? `${workflow} run ${runId} is still going: ${outcome.status}`
    : `${workflow} run ${runId} is still going: ${going.join(', ')}`;
}

/**
 List the jobs of a run that did not pass, each with the step that did not pass.

 @param outcome - what the run says about itself

 @returns one line per such job; empty when every job passed, was skipped or was neutral

 @example
 ```ts
 describeRunFailures(outcome);
 ```
 */
export function describeRunFailures(outcome: RunOutcome): readonly string[] {
  return outcome.jobs
    .filter(function didNotPass(job) {
      return (job.conclusion !== '') && (!passingConclusions.includes(job.conclusion));
    })
    .map(function describe(job) {
      /**
       First step of the job that did not pass, in the order the answer lists them.
       */
      const step = job.steps
        .find(function didNotPassEither(candidate) {
          return (candidate.conclusion !== '') && (!passingConclusions.includes(candidate.conclusion));
        });
      return step === undefined
        ? `job ${job.name} concluded ${job.conclusion}`
        : `job ${job.name} concluded ${job.conclusion} at step ${JSON.stringify(step.name)}`;
    });
}
