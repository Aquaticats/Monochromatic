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
      name: 'one commit round whose paired delta equals the bound is named although the p95 stays under it',
      fn: async () => {
        /**
         Round that is lifted; every other round stays as recorded.
         */
        const liftedRound = 0;
        /**
         Wrapped commit samples where only one round is lifted to the bound.
         */
        const single = rebuilt({
          scenario: commitArm,
          edit: ({
            wrapped,
            direct,
            sample,
          }) => (sample === liftedRound
            ? direct + COMMIT_DELTA_BOUND_MILLISECONDS
            : wrapped),
        });
        expect(single.pairedDelta.p95).toBeLessThan(COMMIT_DELTA_BOUND_MILLISECONDS);
        expect(single.pairedDelta.maximum).toBe(COMMIT_DELTA_BOUND_MILLISECONDS);
        expect(judge([single]).map((violation) => violation.scenario)).toEqual([commitArm.scenario]);
      },
    }),
    it({
      name: 'one commit round one millisecond under the bound is not named',
      fn: async () => {
        /**
         Wrapped commit samples where only the first round is lifted to just under the bound.
         */
        const almost = rebuilt({
          scenario: commitArm,
          edit: ({
            wrapped,
            direct,
            sample,
          }) => (sample === 0
            ? (direct + COMMIT_DELTA_BOUND_MILLISECONDS) - 1
            : wrapped),
        });
        expect(judge([almost])).toEqual([]);
      },
    }),
    it({
      name: 'a commit round that adds more than five seconds over a slow real Git is not named while twice that time covers it',
      fn: async () => {
        /**
         A round whose real Git took four seconds and whose wrapper took ten, so it adds six seconds against a bound of
         eight.
         */
        const slow = scenarioReport({
          scenario: 'commit-explicit',
          kind: 'commit',
          direct: [
            {
              sample: 0,
              elapsed: 4_000,
            },
          ],
          wrapped: [
            {
              sample: 0,
              elapsed: 10_000,
            },
          ],
        });
        expect(judge([slow])).toEqual([]);
      },
    }),
    it({
      name: "a commit round that reaches twice a slow real Git's own time is named with that round's bound",
      fn: async () => {
        /**
         A round whose real Git took four seconds and whose wrapper took twelve, which is the bound of that round.
         */
        const slow = scenarioReport({
          scenario: 'commit-explicit',
          kind: 'commit',
          direct: [
            {
              sample: 0,
              elapsed: 4_000,
            },
          ],
          wrapped: [
            {
              sample: 0,
              elapsed: 12_000,
            },
          ],
        });
        /**
         The requirement the round broke.
         */
        const violations = judge([slow]);
        expect(violations.map((violation) => violation.scenario)).toEqual(['commit-explicit']);
        expect(violations.map((violation) => violation.value)).toEqual([8_000]);
        expect(violations.map((violation) => violation.limit)).toEqual([8_000]);
      },
    }),
    it({
      name: 'the commit round printed is the one furthest past its own bound, and a larger delta that stays inside its bound is not printed',
      fn: async () => {
        /**
         Two rounds: one adds six seconds where its own bound is eight, the other adds 5,100 ms where its own bound is
         five seconds. The second is over its bound although the first is the larger delta.
         */
        const entry = scenarioReport({
          scenario: 'commit-explicit',
          kind: 'commit',
          direct: [
            {
              sample: 0,
              elapsed: 4_000,
            },
            {
              sample: 1,
              elapsed: 100,
            },
          ],
          wrapped: [
            {
              sample: 0,
              elapsed: 10_000,
            },
            {
              sample: 1,
              elapsed: 5_200,
            },
          ],
        });
        /**
         The requirement the report broke.
         */
        const violations = judge([entry]);
        expect(violations.map((violation) => violation.value)).toEqual([5_100]);
        expect(violations.map((violation) => violation.limit)).toEqual([COMMIT_DELTA_BOUND_MILLISECONDS]);
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
