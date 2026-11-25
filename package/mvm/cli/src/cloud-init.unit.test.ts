/**
 Unit tests for the template's cloud-init user-data: every systemd guest gets
 the drop-in that keeps an out-of-memory kill of one command from stopping the
 agent, a guest whose SELinux policy confines the QEMU guest agent also gets the
 drop-in that starts the agent through `/usr/bin/env`, and a guest without such
 a policy must not.
 No virtual machine is started.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  GUEST_AGENT_COMMAND_OOM_DROP_IN_PATH,
  GUEST_AGENT_DROP_IN_PATH,
  IMAGES,
  type LinuxImageSpec,
  templateUserData,
} from '../dist/final/node/index.mjs';

/**
 Hostname the user-data is generated for; it only reaches the hostname and the
 cloud-init instance id, so any valid name does.
 */
const TEMPLATE_HOSTNAME = 'template-hostname-for-tests';

/**
 Registry entry for the distro whose policy confines the agent.
 */
const CONFINED_GUEST = IMAGES.fedora as LinuxImageSpec;

/**
 Registry entry for a distro whose policy does not confine the agent.
 */
const UNCONFINED_GUEST = IMAGES.ubuntu as LinuxImageSpec;

/**
 A dollar sign, so an expected systemd placeholder can be written as a template
 literal that interpolates this variable instead of a string that reads like an
 unexpanded TypeScript placeholder.
 */
const DOLLAR = '$';

/**
 Generates the user-data the bake writes for one image.
 
 @param guest - Registry entry to generate for
 
 @returns Cloud-init user-data text
 */
function userDataFor(guest: LinuxImageSpec,): string {
  return templateUserData({
    guest,
    name: TEMPLATE_HOSTNAME,
  },);
}

/**
 Splits user-data into trimmed lines, so assertions can look for the drop-in
 lines without repeating cloud-init's indentation.
 
 @param guest - Registry entry to generate for
 
 @returns One entry per line, without leading or trailing whitespace
 */
function dropInLinesFor(guest: LinuxImageSpec,): readonly string[] {
  return userDataFor(guest,)
    .split('\n',)
    .map((line,) => line.trim(),);
}

/**
 Finds the line that starts the agent through `/usr/bin/env`.
 
 @param guest - Registry entry to generate for
 
 @returns That line, or the empty string when the user-data has none
 */
function wrappedExecStartFor(guest: LinuxImageSpec,): string {
  /**
   First matching line, if the drop-in is there at all.
   */
  const line = dropInLinesFor(guest,)
    .find((candidate,) => candidate.startsWith('ExecStart=/usr/bin/env ',),);
  return line ?? '';
}

await describe({
  name: '',
  children: [
    describe({
      name: templateUserData.name,
      children: [
        it({
          name: 'writes the drop-in that starts a confined guest agent through /usr/bin/env',
          fn: async () => {
            expect(userDataFor(CONFINED_GUEST,),).toContain(
              `  - path: ${GUEST_AGENT_DROP_IN_PATH}`,
            );
            expect(wrappedExecStartFor(CONFINED_GUEST,),).not.toBe('');
          },
        },),
        it({
          name: 'clears the service\'s own ExecStart on the line before the wrapped one',
          fn: async () => {
            /**
             Drop-in lines, trimmed, in the order cloud-init writes them.
             */
            const lines = dropInLinesFor(CONFINED_GUEST,);
            /**
             Line that clears the distro unit's command.
             */
            const clearedAt = lines.indexOf('ExecStart=',);
            expect(clearedAt,).toBeGreaterThan(-1,);
            expect(lines[clearedAt + 1],).toBe(wrappedExecStartFor(CONFINED_GUEST,),);
          },
        },),
        it({
          name: 'mirrors the argument list of the distro unit, placeholders unexpanded',
          fn: async () => {
            /**
             The one wrapped command line.
             */
            const wrapped = wrappedExecStartFor(CONFINED_GUEST,);
            expect(wrapped,).toContain('/usr/bin/qemu-ga --method=virtio-serial');
            expect(wrapped,).toContain('--path=/dev/virtio-ports/org.qemu.guest_agent.0');
            expect(wrapped,).toContain(
              `-F${DOLLAR}{FSFREEZE_HOOK_PATHNAME} ${DOLLAR}QEMU_GA_ARGS`,
            );
          },
        },),
        it({
          name: 'reloads systemd before enabling the wrapped unit',
          fn: async () => {
            /**
             User-data under test.
             */
            const userData = userDataFor(CONFINED_GUEST,);
            /**
             Where the reload is asked for.
             */
            const reloadAt = userData.indexOf('systemctl daemon-reload',);
            expect(reloadAt,).toBeGreaterThan(-1,);
            expect(reloadAt,).toBeLessThan(
              userData.indexOf('systemctl enable --now qemu-guest-agent',),
            );
          },
        },),
        it({
          name: 'installs the agent package for both distros',
          fn: async () => {
            expect(userDataFor(CONFINED_GUEST,),).toContain('  - qemu-guest-agent');
            expect(userDataFor(UNCONFINED_GUEST,),).toContain('  - qemu-guest-agent');
          },
        },),
        it({
          name: 'leaves a distro that does not confine the agent alone',
          fn: async () => {
            expect(userDataFor(UNCONFINED_GUEST,),).not.toContain(
              GUEST_AGENT_DROP_IN_PATH,
            );
            expect(userDataFor(UNCONFINED_GUEST,),).not.toContain('/usr/bin/env');
            expect(dropInLinesFor(UNCONFINED_GUEST,),).toContain(
              '- systemctl enable --now qemu-guest-agent',
            );
          },
        },),
        it({
          name: 'declares the confinement on the Fedora registry entry only',
          fn: async () => {
            expect(CONFINED_GUEST.guestAgentIsSelinuxConfined,).toBe(true,);
            expect(UNCONFINED_GUEST.guestAgentIsSelinuxConfined,).toBeUndefined();
          },
        },),
        it({
          name: 'keeps the agent running when the kernel ends one of its commands',
          fn: async () => {
            for (const guest of [CONFINED_GUEST, UNCONFINED_GUEST]) {
              expect(userDataFor(guest,),).toContain(
                `  - path: ${GUEST_AGENT_COMMAND_OOM_DROP_IN_PATH}`,
              );
              expect(dropInLinesFor(guest,),).toContain('OOMPolicy=continue');
            }
          },
        },),
        it({
          name: 'reloads systemd before enabling the unit, with or without another drop-in',
          fn: async () => {
            for (const guest of [CONFINED_GUEST, UNCONFINED_GUEST]) {
              /**
               User-data under test for this distro.
               */
              const userData = userDataFor(guest,);
              /**
               Where the reload is asked for.
               */
              const reloadAt = userData.indexOf('systemctl daemon-reload',);
              expect(reloadAt,).toBeGreaterThan(-1,);
              expect(reloadAt,).toBeLessThan(
                userData.indexOf('systemctl enable --now qemu-guest-agent',),
              );
            }
          },
        },),
      ],
    },),
  ],
},);
