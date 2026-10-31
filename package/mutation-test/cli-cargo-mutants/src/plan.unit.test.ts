import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import {
  type Environment,
  type LauncherOptions,
  parseLauncherOptions,
} from './options.ts';
import { planUnits } from './plan.ts';

/**
 Build options through the real parser, so the plan is checked against what a command line produces.

 @param argv - arguments after the required task and tag variable
 @param environment - process environment
 @returns parsed options
 */
function options({
  argv,
  environment = { PATH: '/usr/bin:/bin' },
}: {
  readonly argv: readonly string[];
  readonly environment?: Environment;
}): LauncherOptions {
  return parseLauncherOptions({
    argv: ['--task', '//p:t', '--tag-env', 'IMG_TAG', '--root', '/w', ...argv],
    environment,
    home: '/h',
    packageDirectory: '/repo/package/mutation-test/cli-cargo-mutants',
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: planUnits.name,
      children: [
        it({
          name: 'plans one unit per file, in order, each with its own tag, log and unit name',
          fn: async () => {
            const plans = planUnits(options({ argv: ['w5', 'push_env.rs', 'lock.rs'] }));
            expect(plans.map((plan) => plan.unit)).toEqual(['mutpar-w5-push-env', 'mutpar-w5-lock']);
            expect(plans.map((plan) => plan.tag)).toEqual(['w50', 'w51']);
            expect(plans.map((plan) => plan.logBase)).toEqual(['/h/temp/agent/mutpar-w5-push_env.log', '/h/temp/agent/mutpar-w5-lock.log']);
          },
        }),
        it({
          name: 'sets the tag variable, extra environment, TMPDIR, PATH and HOME in that order',
          fn: async () => {
            const [plan] = planUnits(options({ argv: ['--setenv', 'SKIP=1', 'w5', 'a.rs'] }));
            expect(plan?.systemdRunArguments.filter((argument) => argument.startsWith('--setenv='))).toEqual([
              '--setenv=IMG_TAG=w50',
              '--setenv=SKIP=1',
              '--setenv=TMPDIR=/h/temp/agent/tmp',
              '--setenv=PATH=/usr/bin:/bin',
              '--setenv=HOME=/h',
            ]);
          },
        }),
        it({
          name: 'leaves PATH unset rather than empty when the launcher has none',
          fn: async () => {
            const [plan] = planUnits(options({ argv: ['w5', 'a.rs'], environment: {} }));
            expect(plan?.systemdRunArguments.some((argument) => argument.startsWith('--setenv=PATH='))).toBe(false);
          },
        }),
        it({
          name: 'applies the scope prefix and caps to the unit',
          fn: async () => {
            const [plan] = planUnits(options({
              argv: ['--scope-prefix', 'src/native/', '--unit-memory-max', '8G', '--unit-cpu-quota', '300%', 'w5', 'a.rs'],
            }));
            expect(plan?.systemdRunArguments).toContain('--property=MemoryMax=8G');
            expect(plan?.systemdRunArguments).toContain('--property=CPUQuota=300%');
            expect(plan?.systemdRunArguments.at(-1)).toContain(`'--file' 'src/native/a.rs'`);
          },
        }),
        it({
          name: 'forwards no container limit to the task when none is given',
          fn: async () => {
            const [plan] = planUnits(options({ argv: ['w5', 'a.rs'] }));
            expect(plan?.systemdRunArguments.at(-1)).not.toContain('--memory');
            expect(plan?.systemdRunArguments.at(-1)).not.toContain('--cpus');
          },
        }),
        it({
          name: 'forwards the container limits to the task, after the file scope, when given',
          fn: async () => {
            const [plan] = planUnits(options({ argv: ['--memory', '4g', '--cpus', '3', 'w5', 'a.rs'] }));
            expect(plan?.systemdRunArguments.at(-1)).toContain(`'--file' 'a.rs' '--memory' '4g' '--cpus' '3' >> `);
          },
        }),
        it({
          name: 'quotes a file name with shell metacharacters in the body',
          fn: async () => {
            const [plan] = planUnits(options({ argv: ['w5', `x'; touch pwned; '.rs`] }));
            expect(plan?.systemdRunArguments.at(-1)).toContain(String.raw`'x'\''; touch pwned; '\''.rs'`);
          },
        }),
        it({
          name: 'rejects two files that give one unit name',
          fn: async () => {
            expect(() => planUnits(options({ argv: ['w5', 'a b.rs', 'a-b.rs'] }))).toThrow(CargoMutantsToolingError);
          },
        }),
        it({
          name: 'rejects a tag template that would share one image between campaigns',
          fn: async () => {
            expect(() => planUnits(options({ argv: ['--tag-template', '{label}', 'w5', 'a.rs'] }))).toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
  ],
});
