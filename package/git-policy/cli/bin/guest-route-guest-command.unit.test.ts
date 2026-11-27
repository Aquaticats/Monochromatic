/**
 Guards for the command `mvm exec` is handed: the guest's hostname guard, the guest's own `timeout` sized ahead of
 `mvm`'s kill, and the script quoted whole behind both.
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  GUEST_TIMEOUT_MARGIN_SECONDS,
  guestCommandLine,
} from './guest-route-guest-command.ts';

/** The guest every case below runs in. */
const GUEST = 'issue-6581-part3-guest';

/** Milliseconds in one second, stated here so the cases read their own arithmetic rather than the module's. */
const MILLISECONDS_PER_SECOND = 1_000;

await describe({
  name: '',
  children: [
    describe({
      name: guestCommandLine.name,
      children: [
        it({
          name: 'runs the script only where the guest reports its own hostname',
          fn: async () => {
            expect(guestCommandLine({
              name: GUEST,
              script: 'true',
              timeoutMilliseconds: 900_000,
            },),).toContain(`test "$(hostname)" = ${GUEST} && `,);
          },
        },),
        it({
          name: 'gives the guest a timeout the margin ahead of the bound mvm is given',
          fn: async () => {
            /** The bound one arm is given, in milliseconds. */
            const bound = 900_000;
            /** The seconds the guest's own timeout allows. */
            const guestSeconds = (bound / MILLISECONDS_PER_SECOND) - GUEST_TIMEOUT_MARGIN_SECONDS;
            expect(guestCommandLine({
              name: GUEST,
              script: 'true',
              timeoutMilliseconds: bound,
            },),).toContain(` timeout ${String(guestSeconds,)} sh -ec `,);
          },
        },),
        it({
          name: 'rounds a bound that is not a whole number of seconds down, so the guest never outlasts mvm',
          fn: async () => {
            /** The bound one arm is given, in milliseconds, which is 900.999 seconds. */
            const bound = 900_999;
            /** The seconds the guest's own timeout allows, rounded down the way the drivers always rounded. */
            const guestSeconds = Math.floor(bound / MILLISECONDS_PER_SECOND,) - GUEST_TIMEOUT_MARGIN_SECONDS;
            expect(guestCommandLine({
              name: GUEST,
              script: 'true',
              timeoutMilliseconds: bound,
            },),).toContain(` timeout ${String(guestSeconds,)} sh -ec `,);
          },
        },),
        it({
          name: 'quotes the script whole, so a quote inside it cannot end the command mvm reads',
          fn: async () => {
            expect(guestCommandLine({
              name: GUEST,
              script: `grep --quiet 'SKIPPED' /root/arm.log`,
              timeoutMilliseconds: 900_000,
            },),).toContain(`sh -ec 'grep --quiet '\\''SKIPPED'\\'' /root/arm.log'`,);
          },
        },),
        it({
          name: 'sizes the guest timeout even when the bound the caller gave is below the margin',
          fn: async () => {
            // The drivers pass a bound of minutes, so this boundary is only about the arithmetic staying the one they
            // always did: a bound below the margin leaves the guest a negative timeout, which `timeout` refuses at once.
            /** The bound one arm is given, in milliseconds, which is below the margin's own seconds. */
            const bound = 30_000;
            /** The seconds the guest's own timeout allows. */
            const guestSeconds = (bound / MILLISECONDS_PER_SECOND) - GUEST_TIMEOUT_MARGIN_SECONDS;
            expect(guestCommandLine({
              name: GUEST,
              script: 'true',
              timeoutMilliseconds: bound,
            },),).toContain(` timeout ${String(guestSeconds,)} sh -ec `,);
          },
        },),
      ],
    },),
  ],
},);
