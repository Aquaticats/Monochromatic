import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BLOCKING_TIMEOUT_ENABLED_ENV,
  BLOCKING_TIMEOUT_MS,
  DISABLE_BACKGROUND_TASKS_ENV,
  disabledBackgroundTasksWarning,
  isBlockingTimeoutEnabled,
  isPlainSleep,
  shouldApplyBlockingTimeout,
} from './blocking-timeout.ts';

const DISABLED_INPUTS = ['0', 'false', 'off',] as const;
const ENABLED_INPUTS = [undefined, '', '1', 'FALSE', 'Off',] as const;
const EXEMPT_SLEEPS = ['sleep 30', '  sleep   0.5\n', 'sleep\t2', 'sleep .5', 'sleep 5.',] as const;
const ELIGIBLE_SLEEP_FORMS = [
  'sleep',
  'sleep 1m',
  'sleep .',
  'sleep 1.2.3',
  'sleep -1',
  'sleep 1 2',
  'sleep 5; ls',
  'sleep 5 && ls',
  'sleep 5 > out.txt',
  'sleeper 5',
  'printf ok',
  'sleep5',
] as const;

function apply(
  overrides: Readonly<{
    agentId?: string;
    command?: string;
    env?: Readonly<NodeJS.ProcessEnv>;
    runInBackground?: boolean;
    timeout?: number;
  }>,
): boolean {
  return shouldApplyBlockingTimeout({
    command: 'printf ok',
    env: {},
    ...overrides,
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'blocking timeout constants',
      children: [
        it({
          name: 'names the enabled environment variable',
          fn: async () => {
            expect(BLOCKING_TIMEOUT_ENABLED_ENV,).toBe('MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED',);
          },
        },),
        it({
          name: 'names the native background-disable variable',
          fn: async () => {
            expect(DISABLE_BACKGROUND_TASKS_ENV,).toBe('CLAUDE_CODE_DISABLE_BACKGROUND_TASKS',);
          },
        },),
        it({
          name: 'uses the owner-selected one-second default',
          fn: async () => {
            expect(BLOCKING_TIMEOUT_MS,).toBe(1_000,);
          },
        },),
        it({
          name: 'warning names the conflict, remedies, and bare-mode limit',
          fn: async () => {
            expect(disabledBackgroundTasksWarning,).toContain(BLOCKING_TIMEOUT_ENABLED_ENV,);
            expect(disabledBackgroundTasksWarning,).toContain('stop instead of moving to the background',);
            expect(disabledBackgroundTasksWarning,).toContain('Bare mode',);
          },
        },),
      ],
    },),
    describe({
      name: isBlockingTimeoutEnabled.name,
      children: [
        ...DISABLED_INPUTS.map(function disabledValue(value,) {
          return it({
            name: `disables for exact lowercase ${value}`,
            fn: async () => {
              expect(isBlockingTimeoutEnabled({ [BLOCKING_TIMEOUT_ENABLED_ENV]: value, },),).toBe(false,);
            },
          },);
        },),
        ...ENABLED_INPUTS.map(function enabledValue(value,) {
          return it({
            name: `enables for ${value === undefined ? 'an unset variable' : JSON.stringify(value)}`,
            fn: async () => {
              expect(isBlockingTimeoutEnabled({ [BLOCKING_TIMEOUT_ENABLED_ENV]: value, },),).toBe(true,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: shouldApplyBlockingTimeout.name,
      children: [
        it({
          name: 'applies to a main-agent foreground call without a timeout',
          fn: async () => {
            expect(apply({},),).toBe(true,);
          },
        },),
        it({
          name: 'does not apply to a native subagent',
          fn: async () => {
            expect(apply({ agentId: 'agent-1', },),).toBe(false,);
          },
        },),
        it({
          name: 'does not apply to an explicit background call',
          fn: async () => {
            expect(apply({ runInBackground: true, },),).toBe(false,);
          },
        },),
        it({
          name: 'applies to an explicit foreground call',
          fn: async () => {
            expect(apply({ runInBackground: false, },),).toBe(true,);
          },
        },),
        ...([0, 500, 600_000,] as const).map(function callerTimeout(timeout,) {
          return it({
            name: `preserves caller timeout ${timeout}`,
            fn: async () => {
              expect(apply({ timeout, },),).toBe(false,);
            },
          },);
        },),
        it({
          name: 'does not apply when disabled by the new variable',
          fn: async () => {
            expect(apply({ env: { [BLOCKING_TIMEOUT_ENABLED_ENV]: 'off', }, },),).toBe(false,);
          },
        },),
        it({
          name: 'ignores the removed legacy variable',
          fn: async () => {
            expect(apply({ env: { MONOCHROMATIC_BASH_FORCE_BACKGROUND: 'off', }, },),).toBe(true,);
          },
        },),
        ...EXEMPT_SLEEPS.map(function exemptSleep(command,) {
          return it({
            name: `exempts bare numeric form ${JSON.stringify(command)}`,
            fn: async () => {
              expect(apply({ command, },),).toBe(false,);
            },
          },);
        },),
        ...ELIGIBLE_SLEEP_FORMS.map(function eligibleSleepForm(command,) {
          return it({
            name: `applies outside the bare numeric exemption for ${JSON.stringify(command)}`,
            fn: async () => {
              expect(apply({ command, },),).toBe(true,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: isPlainSleep.name,
      children: [
        ...EXEMPT_SLEEPS.map(function plainSleep(command,) {
          return it({
            name: `accepts ${JSON.stringify(command)}`,
            fn: async () => {
              expect(isPlainSleep(command,),).toBe(true,);
            },
          },);
        },),
        ...ELIGIBLE_SLEEP_FORMS.map(function nonPlainSleep(command,) {
          return it({
            name: `rejects ${JSON.stringify(command)}`,
            fn: async () => {
              expect(isPlainSleep(command,),).toBe(false,);
            },
          },);
        },),
      ],
    },)
  ],
},);
