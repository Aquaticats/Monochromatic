import { fakeTestedImageId } from './fake-programs.ts';
import type { ArgumentSet } from './golden-argument-sets.ts';

/**
 Environment of the native git-policy campaign scenarios: skip the planted controls and name the image tag.
 */
export const nativeEnvironment: Readonly<Record<string, string>> = {
  NATIVE_MUTATION_SKIP_PLANTED: '1',
  GIT_POLICY_NATIVE_IMAGE_TAG: 'golden',
};

/**
 Native wrapper campaign scenarios, including every option the wrapper rejects.
 */
export const nativeArgumentSets: readonly ArgumentSet[] = [
  [
    'default',
    [],
    {}
  ],
  [
    'scoped-files-and-timeout',
    [
      '--file',
      'src/native/lock.rs',
      '--file',
      'src/native/push_env.rs',
      '--timeout',
      '60'
    ],
    {}
  ],
  [
    'examine',
    [
      '--examine',
      'replace foo|bar::baz',
      '--file',
      'src/native/a.rs'
    ],
    {}
  ],
  [
    'build-timeout',
    [
      '--build-timeout',
      '600',
      '--timeout',
      '120',
      '--file',
      'src/native/a.rs'
    ],
    {}
  ],
  [
    'container-limits',
    [
      '--memory',
      '4g',
      '--cpus',
      '3',
      '--pids-limit',
      '64',
      '--file',
      'src/native/x.rs'
    ],
    {}
  ],
  [
    'baseline-skip',
    [
      '--file',
      'src/native/a.rs'
    ],
    { NATIVE_MUTATION_BASELINE_SKIP: 'module::tests::name' }
  ],
  [
    'fail-inspect-image',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'inspect-image' }
  ],
  [
    'fail-short-image-id',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'short-image-id' }
  ],
  [
    'fail-build',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'build' }
  ],
  [
    'fail-short-campaign-id',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'short-campaign-id' }
  ],
  [
    'fail-short-container-id',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'short-container-id' }
  ],
  [
    'fail-wrong-command',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'wrong-command' }
  ],
  [
    'fail-start',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'start' }
  ],
  [
    'fail-orphan-removal',
    [
      '--file',
      'src/native/a.rs'
    ],
    { FAKE_FAIL: 'rm' }
  ],
  [
    'reject-build-timeout-zero',
    [
      '--build-timeout',
      '0'
    ],
    {}
  ],
  [
    'reject-build-timeout-large',
    [
      '--build-timeout',
      '3601'
    ],
    {}
  ],
  [
    'reject-timeout-zero',
    [
      '--timeout',
      '0'
    ],
    {}
  ],
  [
    'reject-timeout-large',
    [
      '--timeout',
      '3601'
    ],
    {}
  ],
  [
    'reject-timeout-fraction',
    [
      '--timeout',
      '1.5'
    ],
    {}
  ],
  [
    'reject-timeout-missing',
    ['--timeout'],
    {}
  ],
  [
    'reject-file-outside-native',
    [
      '--file',
      'src/other/x.rs'
    ],
    {}
  ],
  [
    'reject-file-space',
    [
      '--file',
      'src/native/a b.rs'
    ],
    {}
  ],
  [
    'reject-file-missing-glob',
    ['--file'],
    {}
  ],
  [
    'reject-unknown-option',
    [
      '--bogus',
      '1'
    ],
    {}
  ],
  [
    'reject-examine-semicolon',
    [
      '--examine',
      'a;b'
    ],
    {}
  ],
  [
    'reject-memory-zero',
    [
      '--memory',
      '0g'
    ],
    {}
  ],
  [
    'reject-memory-unit',
    [
      '--memory',
      '2x'
    ],
    {}
  ],
  [
    'reject-cpus-zero',
    [
      '--cpus',
      '0'
    ],
    {}
  ],
  [
    'reject-cpus-decimals',
    [
      '--cpus',
      '1.234'
    ],
    {}
  ],
  [
    'reject-pids-zero',
    [
      '--pids-limit',
      '0'
    ],
    {}
  ],
  [
    'reject-baseline-skip',
    [
      '--file',
      'src/native/a.rs'
    ],
    { NATIVE_MUTATION_BASELINE_SKIP: 'Bad' }
  ],
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
      limits: [],
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
];
