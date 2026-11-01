import { readFile } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  COMMIT_DELTA_BOUND_MILLISECONDS,
  FORWARDED_BOUND_MILLISECONDS,
  judge,
} from './native-perf-judge.ts';
import {
  reportLine,
  violationLine,
} from './native-perf-lines.ts';
import {
  scenarioReport,
  type ScenarioKind,
  type ScenarioReport,
} from './native-perf-measure.ts';
import {
  pairedDeltas,
  pairedDeltaSummary,
} from './native-perf-statistics.ts';

/**
 The scenarios of the recorded baseline that are commits, by the names `native-perf.ts` gives its commit arms.
 */
const COMMIT_SCENARIO_PREFIX = 'commit';

/**
 Share of the rounds, taken from the end, that the tail control lifts over the bound; more than the five percent above
 the 95th percentile, so the p95 itself is lifted.
 */
const TAIL_SHARE = 0.1;

/**
 A scenario of a recorded run, as `perf/native-latency-*.json` stores it: its name and the raw samples of both sides.
 */
type RecordedScenario = {
  readonly scenario: string;
  readonly rawDirect: readonly number[];
  readonly rawWrapped: readonly number[];
};

/**
 Scenarios of the recorded baseline, which `native:perf` wrote with 30 samples per side and no failed sample.
 */
const recorded = (JSON.parse(await readFile(
  new URL(
    '../perf/native-latency-2026-10-20.json',
    import.meta.url,
  ),
  'utf8',
)) as { readonly report: readonly RecordedScenario[]; }).report;

/**
 Report of a recorded scenario after `edit` changed its wrapped samples, judged as the kind `native:perf` gives it.

 @param scenario - recorded scenario

 @param edit - change to one wrapped sample, given the direct sample of the same round

 @returns the scenario report built from the edited samples
 */
function rebuilt({
  scenario,
  edit,
}: {
  readonly scenario: RecordedScenario;
  readonly edit: (request: {
    readonly wrapped: number;
    readonly direct: number;
    readonly sample: number;
  }) => number;
}): ScenarioReport {
  /**
   Requirement the scenario is held to.
   */
  const kind: ScenarioKind = scenario.scenario.startsWith(COMMIT_SCENARIO_PREFIX) ? 'commit' : 'forwarded';
  /**
   The rebuilt report, absent only when the recorded samples are empty.
   */
  const entry = scenarioReport({
    scenario: scenario.scenario,
    kind,
    direct: scenario.rawDirect.map(function round(
      elapsed,
      sample,
    ) {
      return {
        sample,
        elapsed,
      };
    }),
    wrapped: scenario.rawWrapped.map(function round(
      elapsed,
      sample,
    ) {
      return {
        sample,
        elapsed: edit({
          wrapped: elapsed,
          direct: scenario.rawDirect[sample] ?? 0,
          sample,
        }),
      };
    }),
  });
  if (entry === undefined)
    throw new Error(`recorded scenario ${scenario.scenario} has no samples`);
  return entry;
}

/**
 The recorded baseline as the judge reads it, with no edit.
 */
function asRecorded(): readonly ScenarioReport[] {
  return recorded.map(function unedited(scenario) {
    return rebuilt({
      scenario,
      edit: ({ wrapped, }) => wrapped,
    });
  });
}

/**
 First recorded commit arm, whose samples the controls edit.
 */
const commitArm = recorded.find(function isCommit(scenario) {
  return scenario.scenario.startsWith(COMMIT_SCENARIO_PREFIX);
});
/**
 First recorded forwarded command.
 */
const forwardedArm = recorded.find(function isForwarded(scenario) {
  return !scenario.scenario.startsWith(COMMIT_SCENARIO_PREFIX);
});
if ((commitArm === undefined) || (forwardedArm === undefined))
  throw new Error('the recorded baseline needs a commit arm and a forwarded command');

await describe({
  name: judge.name,
  children: [
    it({
      name: 'the recorded baseline breaks neither requirement',
      fn: async () => {
        /**
         Every scenario of the baseline, rebuilt from its raw samples.
         */
        const unedited = asRecorded();
        expect(judge(unedited)).toEqual([]);
      },
    }),
    it({
      name: 'a commit arm whose every paired delta equals the bound is named',
      fn: async () => {
        /**
         The baseline with each wrapped commit sample set to its direct twin plus the bound.
         */
        const violations = judge([
          rebuilt({
            scenario: commitArm,
            edit: ({ direct, }) => direct + COMMIT_DELTA_BOUND_MILLISECONDS,
          }),
        ]);
        expect(violations.map((violation) => violation.scenario)).toEqual([commitArm.scenario]);
        expect(violations.map((violation) => violation.limit)).toEqual([COMMIT_DELTA_BOUND_MILLISECONDS]);
      },
    }),
    it({
      name: 'a commit arm whose every paired delta is one millisecond under the bound is not named',
      fn: async () => {
        expect(judge([
          rebuilt({
            scenario: commitArm,
            edit: ({ direct, }) => (direct + COMMIT_DELTA_BOUND_MILLISECONDS) - 1,
          }),
        ])).toEqual([]);
      },
    }),
    it({
      name: 'a commit arm whose slowest tenth is lifted over the bound is named by its p95',
      fn: async () => {
        /**
         Wrapped commit samples where the last tenth of the rounds is lifted over the bound.
         */
        const tail = rebuilt({
          scenario: commitArm,
          edit: ({
            wrapped,
            direct,
            sample,
          }) => (sample >= Math.floor(commitArm.rawWrapped.length * (1 - TAIL_SHARE))
            ? direct + COMMIT_DELTA_BOUND_MILLISECONDS
            : wrapped),
        });
        expect(tail.pairedDelta.maximum).toBeGreaterThanOrEqual(COMMIT_DELTA_BOUND_MILLISECONDS);
        expect(judge([tail]).map((violation) => violation.scenario)).toEqual([commitArm.scenario]);
      },
    }),
    it({
      name: 'a forwarded command whose added p95 reaches the forwarded bound is named, and not by the commit bound',
      fn: async () => {
        /**
         The forwarded command with each wrapped sample lifted by the forwarded bound.
         */
        const violations = judge([
          rebuilt({
            scenario: forwardedArm,
            edit: ({ wrapped, }) => wrapped + FORWARDED_BOUND_MILLISECONDS,
          }),
        ]);
        expect(violations.map((violation) => violation.scenario)).toEqual([forwardedArm.scenario]);
        expect(violations.map((violation) => violation.limit)).toEqual([FORWARDED_BOUND_MILLISECONDS]);
      },
    }),
    it({
      name: 'a commit that is slower than a forwarded command may be is judged by the commit bound alone',
      fn: async () => {
        /**
         A commit arm lifted by the forwarded bound, which would fail a forwarded command but not a commit.
         */
        const lifted = rebuilt({
          scenario: commitArm,
          edit: ({ wrapped, }) => wrapped + FORWARDED_BOUND_MILLISECONDS,
        });
        expect(lifted.addedP95Milliseconds).toBeGreaterThanOrEqual(FORWARDED_BOUND_MILLISECONDS);
        expect(judge([lifted])).toEqual([]);
      },
    }),
    it({
      name: 'the printed lines carry the paired figures of a commit and the bound it broke, and not for a forwarded command',
      fn: async () => {
        /**
         A commit arm whose every paired delta equals the bound.
         */
        const broken = rebuilt({
          scenario: commitArm,
          edit: ({ direct, }) => direct + COMMIT_DELTA_BOUND_MILLISECONDS,
        });
        /**
         The printed lines of the requirement the broken arm failed.
         */
        const lines = judge([broken]).map(function line(violation) {
          return violationLine(violation);
        });
        expect(reportLine(broken)).toContain('max');
        expect(lines.length).toBe(1);
        expect(lines.join('\n')).toContain(COMMIT_DELTA_BOUND_MILLISECONDS.toFixed(1));
        /**
         The printed figures of a forwarded command.
         */
        const forwardedLine = reportLine(rebuilt({
          scenario: forwardedArm,
          edit: ({ wrapped, }) => wrapped,
        }));
        expect(forwardedLine).not.toContain('paired delta');
      },
    }),
  ],
});

await describe({
  name: pairedDeltas.name,
  children: [
    it({
      name: 'a round that failed on one side leaves the later rounds paired with their own twins',
      fn: async () => {
        /**
         Milliseconds the wrapped side adds in every round.
         */
        const added = 7;
        /**
         Direct samples where round 1 failed.
         */
        const direct = [0, 2, 3].map(function round(sample) {
          return {
            sample,
            elapsed: 10 * (sample + 1),
          };
        });
        /**
         Wrapped samples where round 2 failed.
         */
        const wrapped = [0, 1, 3].map(function round(sample) {
          return {
            sample,
            elapsed: (10 * (sample + 1)) + added,
          };
        });
        expect(pairedDeltas({
          direct,
          wrapped,
        })).toEqual([added, added]);
      },
    }),
    it({
      name: 'the summary reports the median, p95 and maximum of the deltas',
      fn: async () => {
        /**
         Deltas in ascending order, so the maximum is the last.
         */
        const deltas = [1, 2, 3, 4, 100];
        expect(pairedDeltaSummary(deltas)).toEqual({
          median: 3,
          p95: 100,
          maximum: 100,
        });
      },
    }),
  ],
});
