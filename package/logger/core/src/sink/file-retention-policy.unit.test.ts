/**
 Retention bounds stay at their documented defaults and accept only safe overrides. @module
 */
import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _logRetentionPolicyFromEnv as logRetentionPolicyFromEnv,
  DEFAULT_LOG_MAX_AGE_DAYS,
  DEFAULT_LOG_MAX_BYTES,
  DEFAULT_LOG_MAX_FILES,
  type LogRetentionPolicy,
} from '@monochromatic-dev/logger-core/node';

//region One default policy, three independent override knobs

/**
 Policy every missing or invalid override must fall back to.
 */
const DEFAULT_POLICY: LogRetentionPolicy = {
  maxAgeDays: 14,
  maxBytes: 536_870_912,
  maxFiles: 50_000,
};

/**
 Environment keys paired with the policy field each one overrides.
 */
const OVERRIDE_CASES: readonly (readonly [key: string, field: keyof LogRetentionPolicy])[] = [
  ['MONOCHROMATIC_LOG_MAX_AGE_DAYS', 'maxAgeDays',],
  ['MONOCHROMATIC_LOG_MAX_BYTES', 'maxBytes',],
  ['MONOCHROMATIC_LOG_MAX_FILES', 'maxFiles',],
];

/**
 Override texts that are invalid, non-finite, or negative and must fall back.
 */
const REJECTED_VALUES: readonly string[] = [
  'abc',
  'not-a-number',
  '12abc',
  'NaN',
  'Infinity',
  '-Infinity',
  '1e999',
  '-1',
  '-0.5',
];

/**
 Override texts an unset shell or CI variable leaves behind; none may mean zero retention.
 */
const BLANK_VALUES: readonly string[] = [
  '',
  '   ',
  '\t\n',
];

//endregion One default policy, three independent override knobs

await describe({
  name: '',
  children: [
    describe({
      name: logRetentionPolicyFromEnv.name,
      children: [
        //region Defaults

        it({
          name: 'returns 14 days, 536870912 bytes, and 50000 files when no override is set',
          fn: async () => {
            expect(logRetentionPolicyFromEnv({ environment: {}, },),).toEqual(DEFAULT_POLICY,);
          },
        },),

        it({
          name: 'exposes those defaults as exported constants',
          fn: async () => {
            expect(DEFAULT_LOG_MAX_AGE_DAYS,).toBe(14,);
            expect(DEFAULT_LOG_MAX_BYTES,).toBe(536_870_912,);
            expect(DEFAULT_LOG_MAX_FILES,).toBe(50_000,);
          },
        },),

        //endregion Defaults

        //region Overrides

        it({
          name: 'applies every override together',
          fn: async () => {
            expect(logRetentionPolicyFromEnv({
              environment: {
                MONOCHROMATIC_LOG_MAX_AGE_DAYS: '3',
                MONOCHROMATIC_LOG_MAX_BYTES: '4096',
                MONOCHROMATIC_LOG_MAX_FILES: '7',
              },
            },),).toEqual({
              maxAgeDays: 3,
              maxBytes: 4_096,
              maxFiles: 7,
            },);
          },
        },),

        ...OVERRIDE_CASES.map(function overrideCase([key, field,]) {
          return it({
            name: `overrides only ${field} through ${key}`,
            fn: async () => {
              /** Policy produced from one configured override. */
              const policy = logRetentionPolicyFromEnv({ environment: { [key]: '2', }, },);
              expect(policy[field],).toBe(2,);
              for (const [otherKey, otherField,] of OVERRIDE_CASES) {
                if (otherKey !== key)
                  expect(policy[otherField],).toBe(DEFAULT_POLICY[otherField],);
              }
            },
          },);
        },),

        it({
          name: 'accepts a zero bound when it is set explicitly',
          fn: async () => {
            // Zero is finite and nonnegative, so a caller who wants no
            // retained history gets it rather than a silent default.
            expect(logRetentionPolicyFromEnv({ environment: { MONOCHROMATIC_LOG_MAX_AGE_DAYS: '0', }, },),)
              .toEqual({
                maxAgeDays: 0,
                maxBytes: DEFAULT_LOG_MAX_BYTES,
                maxFiles: DEFAULT_LOG_MAX_FILES,
              },);
          },
        },),

        //endregion Overrides

        //region Invalid values fall back

        ...OVERRIDE_CASES.flatMap(function invalidCase([key, field,]) {
          return REJECTED_VALUES.map(function rejectedValue(value,) {
            return it({
              name: `${key}=${JSON.stringify(value,)} falls back to the ${field} default`,
              fn: async () => {
                expect(logRetentionPolicyFromEnv({ environment: { [key]: value, }, },),)
                  .toEqual(DEFAULT_POLICY,);
              },
            },);
          },);
        },),

        it({
          name: 'keeps valid overrides when a sibling override is invalid',
          fn: async () => {
            expect(logRetentionPolicyFromEnv({
              environment: {
                MONOCHROMATIC_LOG_MAX_AGE_DAYS: '3',
                MONOCHROMATIC_LOG_MAX_BYTES: 'nonsense',
                MONOCHROMATIC_LOG_MAX_FILES: '-1',
              },
            },),).toEqual({
              maxAgeDays: 3,
              maxBytes: DEFAULT_LOG_MAX_BYTES,
              maxFiles: DEFAULT_LOG_MAX_FILES,
            },);
          },
        },),

        it({
          name: 'falls back to the default when an override is blank or whitespace-only',
          fn: async () => {
            for (const value of BLANK_VALUES) {
              for (const [key,] of OVERRIDE_CASES) {
                expect(logRetentionPolicyFromEnv({ environment: { [key]: value, }, },),)
                  .toEqual(DEFAULT_POLICY,);
              }
            }
          },
        },),

        //endregion Invalid values fall back
      ],
    },),

    describe({
      name: 'LogRetentionPolicy type shape',
      children: [
        it({
          name: 'is the readonly three-bound policy shape',
          fn: async () => {
            expectTypeOf<LogRetentionPolicy>()
              .toEqualTypeOf<{
                readonly maxAgeDays: number;
                readonly maxBytes: number;
                readonly maxFiles: number;
              }>();
          },
        },),
      ],
    },),
  ],
},);
