/**
 Checks that every fixture configuration defines every shared variable its copied root sections interpolate.

 Both process-boundary fixtures copy root `[vars]` helpers and root task sections verbatim, so a helper that gains an
 interpolation reaches them without its new variable: mise then fails to render the whole file, the task or variable
 being measured never runs, and the cases see no events instead of a message naming the missing field. That happened
 twice, once when the `buildAndTest` task body began interpolating the raised task-list cap and once when the
 `fanout_packages` helper did.

 A `[vars]` entry is rendered with only the entries above it in scope, so the definition must also come before its first
 use. The checks read the generated configuration as text, which is what mise renders.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fixtureConfig } from './build-and-test-fixture-config.ts';
import { fixtureConfiguration } from './root-test-aggregate-fixture-config.ts';
import { testUnitFixtureConfig } from './test-unit-fixture-config.ts';

//region Reading a generated configuration

/**
 One fixture configuration: the name a message uses and its generated text.
 */
type FixtureConfiguration = {
  readonly name: string;
  readonly text: string;
};

/**
 Every fixture configuration this package generates.
 */
const configurations: readonly FixtureConfiguration[] = [
  { name: 'buildAndTest fixture', text: fixtureConfig },
  { name: 'root test aggregate fixture', text: fixtureConfiguration() },
  { name: 'test:unit fixture', text: testUnitFixtureConfig },
];

/**
 Marks the start of an interpolation of a shared variable.
 */
const interpolationStart = '{{vars.';

/**
 Names of the shared variables a configuration interpolates, in the order they first appear.

 @param text - generated configuration

 @returns each interpolated variable name once

 @example
 ```ts
 interpolated({ text: '{{vars.fanout}}' });
 ```
 */
function interpolated({ text, }: { readonly text: string }): readonly string[] {
  const names: string[] = [];
  for (let index = text.indexOf(interpolationStart,); index !== (-1);) {
    const from = index + interpolationStart.length;
    const to = text.indexOf('}}', from,);
    if (to === (-1)) break;
    const name = text.slice(from, to,);
    if (!names.includes(name,)) names.push(name,);
    index = text.indexOf(interpolationStart, to,);
  }
  return names;
}

/**
 Where a variable assignment begins, and nothing when the configuration assigns it nowhere.

 @param options - generated configuration and the variable name

 @returns index of `NAME = """`, or -1

 @example
 ```ts
 assignmentAt({ text: 'a = """\nx\n"""', name: 'a' });
 ```
 */
function assignmentAt({ text, name, }: {
  readonly text: string;
  readonly name: string;
}): number {
  return text.indexOf(`${name} = """`,);
}

//endregion

await describe({
  name: 'fixture configurations',
  children: [
    it({
      name: 'interpolate shared variables and assign some, so the checks below cannot pass on an empty configuration',
      fn: async function hasSubjects() {
        for (const configuration of configurations) {
          expect(interpolated({ text: configuration.text, },).length,).toBeGreaterThan(0,);
          expect(assignmentAt({ text: configuration.text, name: 'run_test_files', },),).toBeGreaterThan((-1),);
        }
      },
    },),
    it({
      name: 'assign every shared variable they interpolate, and assign it above its first use',
      fn: async function assignsWhatItInterpolates() {
        const missing: string[] = [];
        for (const configuration of configurations)
          for (const name of interpolated({ text: configuration.text, },)) {
            const assignedAt = assignmentAt({ text: configuration.text, name, },);
            const usedAt = configuration.text.indexOf(`${interpolationStart}${name}}}`,);
            if (assignedAt === (-1))
              missing.push(`${configuration.name}: ${name} is used but not assigned`);
            else if (assignedAt > usedAt)
              missing.push(`${configuration.name}: ${name} is assigned after its first use`);
          }
        expect(missing,).toEqual([],);
      },
    },),
  ],
},);
