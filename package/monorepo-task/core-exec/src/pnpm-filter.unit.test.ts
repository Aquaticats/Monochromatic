import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  filterPnpmOutput,
  isAllowedCycleWarning,
} from '../dist/final/node/testing.mjs';

/** pnpm uses U+2009 THIN SPACE around "WARN" in formatted output. */
const WARN = '\u2009WARN\u2009';

await describe({
  name: '',
  children: [
    describe({
      name: isAllowedCycleWarning.name,
      children: [
        it({
          name: 'returns true for the known build/test utility cycle',
          fn: async () => {
            expect(isAllowedCycleWarning(
              `${WARN} There are cyclic workspace dependencies: /var/home/user/Monochromatic/package/config/tsdown, /var/home/user/Monochromatic/package/test-harness/core, /var/home/user/Monochromatic/package/numeric-format/core, /var/home/user/Monochromatic/package/or-throw/core`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'returns true for the known async-time/core + test-harness/core cycle',
          fn: async () => {
            expect(isAllowedCycleWarning(
              `${WARN} There are cyclic workspace dependencies: /var/home/user/Monochromatic/package/async-time/core, /var/home/user/Monochromatic/package/test-harness/core`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'returns true regardless of monorepo root path',
          fn: async () => {
            expect(isAllowedCycleWarning(
              `${WARN} There are cyclic workspace dependencies: /home/alice/projects/mono/package/async-time/core, /home/alice/projects/mono/package/test-harness/core`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'returns false for unknown packages in the cycle',
          fn: async () => {
            expect(isAllowedCycleWarning(
              `${WARN} There are cyclic workspace dependencies: /var/home/user/Monochromatic/package/foo/bar, /var/home/user/Monochromatic/package/baz/qux`,
            ),)
              .toBe(false,);
          },
        },),
        it({
          name: 'returns false for a mix of known and unknown packages',
          fn: async () => {
            expect(isAllowedCycleWarning(
              `${WARN} There are cyclic workspace dependencies: /var/home/user/Monochromatic/package/async-time/core, /var/home/user/Monochromatic/package/foo/bar`,
            ),)
              .toBe(false,);
          },
        },),
        it({
          name: 'returns false for unrelated warnings',
          fn: async () => {
            expect(isAllowedCycleWarning(
              `${WARN} deprecated package: some-old-pkg@1.0.0`,
            ),)
              .toBe(false,);
          },
        },),
        it({
          name: 'returns false for empty string',
          fn: async () => {
            expect(isAllowedCycleWarning('',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: filterPnpmOutput.name,
      children: [
        it({
          name: 'removes the allowed cycle warning while keeping other lines',
          fn: async () => {
            const output = [
              `${WARN} There are cyclic workspace dependencies: /abs/package/config/tsdown, /abs/package/test-harness/core, /abs/package/numeric-format/core, /abs/package/or-throw/core`,
              `${WARN} deprecated package: old-pkg@1.0.0`,
              '',
            ]
              .join('\n',);

            const result = filterPnpmOutput(output,);
            expect(result,).not.toContain('cyclic workspace dependencies',);
            expect(result,).toContain('deprecated package',);
          },
        },),
        it({
          name: 'returns empty string for empty input',
          fn: async () => {
            expect(filterPnpmOutput('',),).toBe('',);
          },
        },),
        it({
          name: 'preserves all lines when no cycle warning is present',
          fn: async () => {
            const output = 'some output\nanother line\n';
            expect(filterPnpmOutput(output,),).toBe(output,);
          },
        },),
        it({
          name: 'does not filter unknown cycle warnings',
          fn: async () => {
            const output =
              `${WARN} There are cyclic workspace dependencies: /abs/package/foo, /abs/package/bar\n`;
            expect(filterPnpmOutput(output,),).toBe(output,);
          },
        },),
      ],
    },),
  ],
},);
