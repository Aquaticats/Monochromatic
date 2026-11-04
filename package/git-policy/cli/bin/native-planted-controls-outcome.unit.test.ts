import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { judgePlantedRun, } from './native-planted-controls-outcome.ts';

/**
 Test the named control stands for, in the form nextest prints it after the binary ID.
 */
const failing = 'entry::stop_tests::forward_target_naming_this_executable_stops';

/**
 Output of a build that Cargo stopped, as the stale-image case printed it: the planted file no longer fit the sibling
 files baked into the image, so no test ran and no test line exists.
 */
const compileFailure = [
  '   Compiling monochromatic-git-policy-cli v0.4.6 (/work/package/git-policy/cli)',
  'error[E0432]: unresolved import `super::lfs_upload_plan::settings_from_snapshot`',
  '  --> src/native/invocation/entry.rs:24:5',
  'warning: unused import: `super::real_git_candidate::same_file`',
  'error: could not compile `monochromatic-git-policy-cli` (lib test) due to 2 previous errors; 2 warnings emitted',
  'error: command `cargo test --no-run` exited with code 101',
].join('\n');

/**
 Output of a run in which the named test failed among others.
 */
const noticedRun = [
  '    Starting 2956 tests across 5 binaries (7 tests skipped)',
  `        FAIL [   0.007s] ( 859/2956) monochromatic-git-policy-cli ${failing}`,
  '     Summary [ 285.407s] 2956 tests run: 2955 passed, 1 failed, 7 skipped',
].join('\n');

await describe({
  name: judgePlantedRun.name,
  children: [
    it({
      name: 'a failed named test is noticed',
      fn: async () => {
        expect(judgePlantedRun({
          output: noticedRun,
          status: 100,
          failing,
        })).toEqual({ kind: 'noticed', });
      },
    }),
    it({
      name: 'a leaking failed named test is noticed',
      fn: async () => {
        expect(judgePlantedRun({
          output: `    Starting 3 tests across 1 binary\n        FAIL + LEAK [   5.056s] (1/3) crate::bin ${failing}`,
          status: 100,
          failing,
        })).toEqual({ kind: 'noticed', });
      },
    }),
    it({
      name: 'a named test the slow-timeout stopped is noticed',
      fn: async () => {
        expect(judgePlantedRun({
          output: `    Starting 3 tests across 1 binary\n     TIMEOUT [ 300.002s] (1/3) crate::bin ${failing}`,
          status: 100,
          failing,
        })).toEqual({ kind: 'noticed', });
      },
    }),
    it({
      name: 'a run where only another test was stopped by the slow-timeout is not noticed',
      fn: async () => {
        expect(judgePlantedRun({
          output: `    Starting 3 tests across 1 binary\n     TIMEOUT [ 300.002s] (1/3) crate::bin other::test`,
          status: 100,
          failing,
        })).toEqual({ kind: 'not-noticed', });
      },
    }),
    it({
      name: 'a run where other tests failed but not the named one is not noticed',
      fn: async () => {
        expect(judgePlantedRun({
          output: `    Starting 3 tests across 1 binary\n        FAIL [   0.1s] (1/3) crate::bin other::test`,
          status: 100,
          failing,
        })).toEqual({ kind: 'not-noticed', });
      },
    }),
    it({
      name: 'a named test with a longer name sharing its ending is not noticed',
      fn: async () => {
        expect(judgePlantedRun({
          output: `    Starting 1 test across 1 binary\n        FAIL [   0.1s] (1/1) crate::bin extra_${failing}`,
          status: 100,
          failing,
        })).toEqual({ kind: 'not-noticed', });
      },
    }),
    it({
      name: 'a passing run is not noticed even when it names the test',
      fn: async () => {
        expect(judgePlantedRun({
          output: `    Starting 1 test across 1 binary\n        FAIL [   0.1s] (1/1) crate::bin ${failing}`,
          status: 0,
          failing,
        })).toEqual({ kind: 'not-noticed', });
      },
    }),
    it({
      name: 'a run killed by a signal is not noticed',
      fn: async () => {
        expect(judgePlantedRun({
          output: noticedRun,
          failing,
        })).toEqual({ kind: 'not-noticed', });
      },
    }),
    it({
      name: 'a build that did not compile reports the compiler errors instead of a missed guard',
      fn: async () => {
        expect(judgePlantedRun({
          output: compileFailure,
          status: 101,
          failing,
        })).toEqual({
          kind: 'build-failed',
          detail: [
            'error[E0432]: unresolved import `super::lfs_upload_plan::settings_from_snapshot`',
            'error: could not compile `monochromatic-git-policy-cli` (lib test) due to 2 previous errors; 2 warnings emitted',
          ].join('\n'),
        });
      },
    }),
    it({
      name: 'a build failure wins even when the output also names the test',
      fn: async () => {
        expect(judgePlantedRun({
          output: `${noticedRun}\n${compileFailure}`,
          status: 100,
          failing,
        }).kind).toBe('build-failed');
      },
    }),
  ],
});
