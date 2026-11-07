import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { launchUnits } from './launch-units.ts';
import type { UnitPlan } from './plan.ts';
import type { StartedUnit } from './start-units.ts';

/**
 Plans for files with the given names, each with its own unit.

 @param files - file names

 @returns one plan per file
 */
function plansOf(files: readonly string[]): readonly UnitPlan[] {
  return files.map(function planOf(file) {
    return {
      file,
      unit: `u-${file}`,
      logBase: `/logs/${file}.log`,
      tag: `t-${file}`,
      systemdRunArguments: [`--unit=u-${file}`],
    };
  });
}

/**
 A starter that refuses the named units with an error text and accepts the others, and records every unit it was asked to
 start.

 @param refused - unit names `systemd-run` refuses

 @returns the starter and its record
 */
function starting(refused: readonly string[]): {
  readonly start: (call: { readonly unit: string; readonly systemdRunArguments: readonly string[] }) => Promise<StartedUnit>;
  readonly asked: string[];
} {
  const asked: string[] = [];
  return {
    asked,
    async start({ unit }) {
      asked.push(unit);
      return refused.includes(unit)
        ? {
          unit,
          started: false,
          line: `${unit}: Failed to start transient service unit: fake failure`,
        }
        : {
          unit,
          started: true,
          line: `${unit}: started`,
        };
    },
  };
}

await describe({
  name: launchUnits.name,
  children: [
    it({
      name: 'exits 0 and prints every unit and the wait command when every unit started',
      fn: async () => {
        const { start } = starting([]);
        const outcome = await launchUnits({
          plans: plansOf(['a.rs', 'b.rs']),
          start,
        });
        expect(outcome.exitCode).toBe(0);
        expect(outcome.started).toEqual(['u-a.rs', 'u-b.rs']);
        expect(outcome.failed).toEqual([]);
        expect(outcome.lines.slice(0, 2)).toEqual(['u-a.rs: started', 'u-b.rs: started']);
        expect(outcome.lines.at(-1)).toBe('wait with: systemctl --user is-active u-a.rs u-b.rs');
      },
    }),
    it({
      name: 'tries every unit even when one fails, because the units are independent',
      fn: async () => {
        const { start, asked } = starting(['u-a.rs']);
        await launchUnits({
          plans: plansOf(['a.rs', 'b.rs', 'c.rs']),
          start,
        });
        expect(asked).toEqual(['u-a.rs', 'u-b.rs', 'u-c.rs']);
      },
    }),
    it({
      name: 'exits nonzero when a unit failed, naming it, its file and the systemd-run error, and waits only for the started ones',
      fn: async () => {
        const { start } = starting(['u-b.rs']);
        const outcome = await launchUnits({
          plans: plansOf(['a.rs', 'b.rs', 'c.rs']),
          start,
        });
        expect(outcome.exitCode).toBe(1);
        expect(outcome.started).toEqual(['u-a.rs', 'u-c.rs']);
        expect(outcome.failed).toEqual([{ unit: 'u-b.rs', file: 'b.rs' }]);
        const text = outcome.lines.join('\n');
        expect(text).toContain('u-b.rs: Failed to start transient service unit: fake failure');
        expect(text).toContain('u-a.rs: started');
        expect(text).toContain('b.rs (u-b.rs)');
        expect(text).toContain('keep running');
        expect(outcome.lines.at(-1)).toBe('wait with: systemctl --user is-active u-a.rs u-c.rs');
      },
    }),
    it({
      name: 'says to relaunch the failed files under another label, because a tag belongs to its position',
      fn: async () => {
        const { start } = starting(['u-b.rs']);
        const outcome = await launchUnits({
          plans: plansOf(['a.rs', 'b.rs']),
          start,
        });
        expect(outcome.lines.join('\n')).toContain('another label');
      },
    }),
    it({
      name: 'exits nonzero and prints no wait command when no unit started',
      fn: async () => {
        const { start } = starting(['u-a.rs', 'u-b.rs']);
        const outcome = await launchUnits({
          plans: plansOf(['a.rs', 'b.rs']),
          start,
        });
        expect(outcome.exitCode).toBe(1);
        expect(outcome.started).toEqual([]);
        expect(outcome.lines.some(function isHint(line) {
          return line.startsWith('wait with');
        })).toBe(false);
      },
    }),
    it({
      name: 'puts the units a product started itself before the file units in the wait command',
      fn: async () => {
        const { start } = starting([]);
        const outcome = await launchUnits({
          plans: plansOf(['a.rs']),
          start,
          alsoWaitFor: ['controls'],
        });
        expect(outcome.lines.at(-1)).toBe('wait with: systemctl --user is-active controls u-a.rs');
      },
    }),
  ],
});
