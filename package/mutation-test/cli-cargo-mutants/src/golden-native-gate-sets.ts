import { fakeTestedImageId } from './fake-programs.ts';

/**
 Container arguments of a gate run at the defaults, as the gate's manifest records them.
 */
const gateLimits = [
  '--rm',
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=512',
];

/**
 Files of a gate run the way the native gate leaves them, so a campaign can find proof that the gate passed.

 @param image - full image ID the manifest records

 @param passed - whether the gate got as far as writing its passed record

 @returns the files by path below the working directory

 @example
 ```ts
 gateFiles({ image: fakeTestedImageId, passed: true });
 ```
 */
function gateFiles({
  image,
  passed,
}: {
  readonly image: string;
  readonly passed: boolean;
}): Readonly<Record<string, string>> {
  return {
    'target/verification/native-gate01/manifest.json': `${JSON.stringify({
      base: 'c'.repeat(fakeTestedImageId.length),
      image,
      limits: gateLimits,
    })}\n`,
    ...(passed ? {
      'target/verification/native-gate01/passed.json': `${JSON.stringify({
        tests: true,
        clippy: true,
        image,
      })}\n`,
    } : {}),
  };
}

/**
 Native wrapper campaign scenarios that start with gate evidence in the working directory.
 */
export const nativeGateScenarios: readonly {
  readonly name: string;
  readonly args: readonly string[];
  readonly files: Readonly<Record<string, string>>;
  readonly environment?: Readonly<Record<string, string>>;
}[] = [
  {
    name: 'baseline-proof',
    args: [
      '--file',
      'src/native/a.rs'
    ],
    files: gateFiles({
      image: fakeTestedImageId,
      passed: true,
    }),
  },
  {
    name: 'baseline-proof-other-image',
    args: [
      '--file',
      'src/native/a.rs'
    ],
    files: gateFiles({
      image: 'd'.repeat(fakeTestedImageId.length),
      passed: true,
    }),
  },
  {
    name: 'baseline-proof-gate-not-passed',
    args: [
      '--file',
      'src/native/a.rs'
    ],
    files: gateFiles({
      image: fakeTestedImageId,
      passed: false,
    }),
  },
  {
    name: 'baseline-proof-lower-pids-limit',
    args: [
      '--pids-limit',
      '128',
      '--file',
      'src/native/a.rs'
    ],
    files: gateFiles({
      image: fakeTestedImageId,
      passed: true,
    }),
  },
];

/**
 A small crate in the working directory, shaped like the native wrapper: `b` imports `a`, `c` imports nothing.
 The two-pass scenarios point `NATIVE_MUTATION_CRATE_DIRECTORY` at it, so their pinned outcome does not depend on the
 real crate's sources.
 */
const fixtureCrateFiles: Readonly<Record<string, string>> = {
  'crate/Cargo.toml': '[package]\nname = "fixture-crate"\nversion = "0.1.0"\n\n[lib]\nname = "fixture_lib"\npath = "src/native/lib.rs"\n\n[[test]]\nname = "native_binary"\npath = "src/native/binary_tests.rs"\n',
  'crate/src/native/lib.rs': 'pub mod a;\npub mod b;\npub mod c;\n',
  'crate/src/native/a.rs': 'pub fn f() {}\n',
  'crate/src/native/b.rs': 'use super::a::f;\npub fn g() { f() }\n',
  'crate/src/native/c.rs': 'pub fn h() {}\n',
  'crate/src/native/binary_tests.rs': 'use fixture_lib::a::f;\n',
};

/**
 Environment of the two-pass scenarios: the fixture crate, and mutants of which pass 1 misses the second.
 */
const twoPassEnvironment: Readonly<Record<string, string>> = {
  NATIVE_MUTATION_CRATE_DIRECTORY: 'crate',
  FAKE_MUTANTS: 'm1,m2',
  FAKE_PASS_ONE_MISSED: 'm2',
};

/**
 Native wrapper campaign scenarios with `--passes`.
 */
export const nativePassScenarios: readonly {
  readonly name: string;
  readonly args: readonly string[];
  readonly files: Readonly<Record<string, string>>;
  readonly environment: Readonly<Record<string, string>>;
}[] = [
  {
    name: 'two-pass',
    args: [
      '--passes',
      '2',
      '--file',
      'src/native/a.rs'
    ],
    files: {
      ...fixtureCrateFiles,
      ...gateFiles({
        image: fakeTestedImageId,
        passed: true,
      }),
    },
    environment: twoPassEnvironment,
  },
  {
    name: 'reject-passes-without-gate-proof',
    args: [
      '--passes',
      '2',
      '--file',
      'src/native/a.rs'
    ],
    files: fixtureCrateFiles,
    environment: twoPassEnvironment,
  },
  {
    name: 'reject-passes-gate-limits-lower',
    args: [
      '--passes',
      '2',
      '--pids-limit',
      '128',
      '--file',
      'src/native/a.rs'
    ],
    files: {
      ...fixtureCrateFiles,
      ...gateFiles({
        image: fakeTestedImageId,
        passed: true,
      }),
    },
    environment: twoPassEnvironment,
  },
  {
    name: 'one-pass-by-passes-1',
    args: [
      '--passes',
      '1',
      '--file',
      'src/native/a.rs'
    ],
    files: fixtureCrateFiles,
    environment: twoPassEnvironment,
  },
  {
    name: 'reject-passes-glob',
    args: [
      '--passes',
      '2',
      '--file',
      'src/native/a*.rs'
    ],
    files: fixtureCrateFiles,
    environment: twoPassEnvironment,
  },
  {
    name: 'reject-passes-without-file',
    args: [
      '--passes',
      '2'
    ],
    files: fixtureCrateFiles,
    environment: twoPassEnvironment,
  },
  {
    name: 'reject-passes-unknown-module',
    args: [
      '--passes',
      '2',
      '--file',
      'src/native/zz.rs'
    ],
    files: fixtureCrateFiles,
    environment: twoPassEnvironment,
  },
  {
    name: 'reject-passes-value',
    args: [
      '--passes',
      '3',
      '--file',
      'src/native/a.rs'
    ],
    files: fixtureCrateFiles,
    environment: twoPassEnvironment,
  },
];
