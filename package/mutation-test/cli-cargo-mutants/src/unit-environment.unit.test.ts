import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseLauncherOptions } from './options.ts';
import { buildUnitEnvironment } from './unit-environment.ts';

/**
 Parse options through the real parser.

 @param argv - arguments after the required task and tag variable

 @param environment - process environment

 @returns the options

 @example
 ```ts
 options({ argv: ['w5', 'a.rs'] });
 ```
 */
function options({
  argv,
  environment = { PATH: '/usr/bin:/bin' },
}: {
  readonly argv: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
}) {
  return parseLauncherOptions({
    argv: ['--task', '//p:t', '--tag-env', 'IMG_TAG', '--image-repository', 'localhost/product-test', '--root', '/w', ...argv],
    environment,
    home: '/h',
    packageDirectory: '/repo/package/mutation-test/cli-cargo-mutants',
  });
}

await describe({
  name: buildUnitEnvironment.name,
  children: [
    it({
      name: 'puts the tag first, then the person\'s entries, then TMPDIR, PATH and HOME',
      fn: async () => {
        expect(buildUnitEnvironment({
          options: options({ argv: ['--setenv', 'CONTAINERS_STORAGE_CONF=/s.conf', 'w5', 'a.rs'] }),
          tag: 'w50',
        })).toEqual([
          'IMG_TAG=w50',
          'CONTAINERS_STORAGE_CONF=/s.conf',
          'TMPDIR=/h/temp/agent/tmp',
          'PATH=/usr/bin:/bin',
          'HOME=/h',
        ]);
      },
    }),
    it({
      name: 'leaves PATH out when the launcher has none',
      fn: async () => {
        expect(buildUnitEnvironment({
          options: options({ argv: ['w5', 'a.rs'], environment: {} }),
          tag: 'w50',
        }).some((entry) => entry.startsWith('PATH='))).toBe(false);
      },
    }),
  ],
});
