import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  recordedCargoPublishRunPassed,
  recordedReleaseRunFailed,
  recordedReleaseRunPassed,
  recordedRunStillGoing,
} from './fixture/recorded-run-answers.ts';
import {
  describeRunFailures,
  describeRunOutcome,
  isRunCompleted,
  jobsStillGoing,
  parseRunOutcome,
  runPassed,
} from './run-outcome.ts';

/**
 Read a recorded run answer the way the wait reads it.

 @param recorded - the answer as the fixture holds it
 @returns the run's own account of itself, which must parse
 */
function parseRecorded(recorded: unknown) {
  const parsed = parseRunOutcome(JSON.stringify(recorded));
  if (!parsed.found)
    throw new Error(`the recorded answer did not parse: ${parsed.reason}`);
  return parsed.value;
}

await describe({
  name: '',
  children: [
    describe({
      name: parseRunOutcome.name,
      children: [
        it({
          name: 'reads a real release run that completed with success, with both of its jobs',
          fn: async () => {
            const outcome = parseRecorded(recordedReleaseRunPassed);
            expect(outcome.status).toBe('completed');
            expect(outcome.conclusion).toBe('success');
            expect(outcome.createdAt).toBe('2026-11-11T00:55:54Z');
            expect(outcome.jobs.map(function nameOf(job) {
              return `${job.name} ${job.status} ${job.conclusion}`;
            })).toEqual(['release completed success', 'publish completed success']);
            expect(isRunCompleted(outcome)).toBe(true);
            expect(runPassed(outcome)).toBe(true);
            expect(jobsStillGoing(outcome)).toEqual([]);
            expect(describeRunFailures(outcome)).toEqual([]);
          },
        }),
        it({
          name: 'reads a real release run that failed, and names the step of the job that failed',
          fn: async () => {
            const outcome = parseRecorded(recordedReleaseRunFailed);
            expect(outcome.conclusion).toBe('failure');
            expect(runPassed(outcome)).toBe(false);
            // The publish job was skipped because the release job never pushed, and a skipped job is no failure.
            expect(describeRunFailures(outcome)).toEqual([
              'job release concluded failure at step "Install Node, pnpm and the workspace dependencies"',
            ]);
          },
        }),
        it({
          name: 'reads the real cargo-publish run that built the archives, whose skipped jobs are no failures',
          fn: async () => {
            const outcome = parseRecorded(recordedCargoPublishRunPassed);
            expect(outcome.conclusion).toBe('success');
            expect(runPassed(outcome)).toBe(true);
            expect(outcome.jobs.length).toBeGreaterThan(1);
            expect(outcome.jobs.some(function isSkipped(job) {
              return job.conclusion === 'skipped';
            })).toBe(true);
            expect(outcome.jobs.some(function builtArchives(job) {
              return job.name.startsWith('gpc-build-binary')
                && (job.conclusion === 'success');
            })).toBe(true);
            expect(describeRunFailures(outcome)).toEqual([]);
          },
        }),
        it({
          name: 'reads a real run that is still going and lists the jobs it has not ended',
          fn: async () => {
            const outcome = parseRecorded(recordedRunStillGoing);
            expect(isRunCompleted(outcome)).toBe(false);
            expect(runPassed(outcome)).toBe(false);
            expect(jobsStillGoing(outcome).length).toBeGreaterThan(0);
            expect(describeRunOutcome({
              workflow: 'forbidden-strings.yml',
              runId: '7',
              outcome
            })).toContain('is still going: ');
          },
        }),
        it({
          name: 'reports a run that ended in no conclusion as ended, without calling it a pass',
          fn: async () => {
            const parsed = parseRunOutcome(JSON.stringify({
              createdAt: '2026-11-11T00:00:00Z',
              status: 'completed',
              conclusion: null,
              jobs: [],
            }));
            expect(parsed.found).toBe(true);
            if (!parsed.found)
              return;
            expect(parsed.value.conclusion).toBe('');
            expect(runPassed(parsed.value)).toBe(false);
            expect(describeRunOutcome({
              workflow: 'cargo-publish.yml',
              runId: '42',
              outcome: parsed.value
            })).toBe('cargo-publish.yml run 42: completed with no conclusion');
          },
        }),
        it({
          name: 'refuses an answer that is not JSON, and says so',
          fn: async () => {
            const parsed = parseRunOutcome('watch exit 1');
            expect(parsed.found).toBe(false);
            if (parsed.found)
              return;
            expect(parsed.reason).toContain('not JSON');
          },
        }),
        it({
          name: 'refuses an answer without a status or without a creation time',
          fn: async () => {
            const noStatus = parseRunOutcome('{"createdAt":"2026-11-11T00:00:00Z","jobs":[]}');
            const noCreatedAt = parseRunOutcome('{"status":"completed","jobs":[]}');
            expect(noStatus.found).toBe(false);
            expect(noCreatedAt.found).toBe(false);
          },
        }),
        it({
          name: 'keeps the jobs of an answer that reports none, so a run without jobs is still read',
          fn: async () => {
            const parsed = parseRunOutcome('{"createdAt":"2026-11-11T00:00:00Z","status":"in_progress"}');
            expect(parsed.found).toBe(true);
            if (!parsed.found)
              return;
            expect(parsed.value.jobs).toEqual([]);
            expect(describeRunOutcome({
              workflow: 'cargo-publish.yml',
              runId: '42',
              outcome: parsed.value
            })).toBe('cargo-publish.yml run 42 is still going: in_progress');
          },
        }),
      ],
    }),
    describe({
      name: describeRunOutcome.name,
      children: [
        it({
          name: 'names the workflow, the run and what the run concluded',
          fn: async () => {
            expect(describeRunOutcome({
              workflow: 'cargo-publish.yml',
              runId: '42935721432',
              outcome: parseRecorded(recordedCargoPublishRunPassed)
            })).toBe('cargo-publish.yml run 42935721432: completed with success');
          },
        }),
      ],
    }),
    describe({
      name: describeRunFailures.name,
      children: [
        it({
          name: 'names each job that did not pass, with the step that did not pass when it reports one',
          fn: async () => {
            const parsed = parseRunOutcome(JSON.stringify({
              createdAt: '2026-11-11T00:00:00Z',
              status: 'completed',
              conclusion: 'failure',
              jobs: [
                {
                  name: 'release',
                  status: 'completed',
                  conclusion: 'cancelled',
                  steps: []
                },
                {
                  name: 'publish',
                  status: 'completed',
                  conclusion: 'failure',
                  steps: [
                    {
                      name: 'Start cargo-publish for the crate on main',
                      number: 2,
                      status: 'completed',
                      conclusion: 'timed_out'
                    },
                  ]
                },
                {
                  name: 'detect',
                  status: 'completed',
                  conclusion: 'skipped',
                  steps: []
                },
              ],
            }));
            expect(parsed.found).toBe(true);
            if (!parsed.found)
              return;
            expect(describeRunFailures(parsed.value)).toEqual([
              'job release concluded cancelled',
              'job publish concluded failure at step "Start cargo-publish for the crate on main"',
            ]);
          },
        }),
      ],
    }),
  ],
});
