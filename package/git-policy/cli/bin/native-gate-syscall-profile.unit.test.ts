import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  derivedSeccompProfile,
  statmountAllowedVariable,
  theCallsTheMountSweepAsks,
} from './native-gate-syscall-profile.ts';

/**
 A profile shaped as the containers-common default is: one unconditional allow rule naming the mount family, one narrowed
 rule, and the default action that answers every unlisted call with `ENOSYS`.
 */
const hostProfileText = `${JSON.stringify({
  defaultAction: 'SCMP_ACT_ERRNO',
  defaultErrnoRet: 38,
  syscalls: [
    {
      names: [
        'mknodat',
        'mount',
        'mount_setattr',
        'move_mount',
        'mprotect',
      ],
      action: 'SCMP_ACT_ALLOW',
      args: [],
      includes: {},
      excludes: {},
    },
    {
      names: ['perf_event_open'],
      action: 'SCMP_ACT_ALLOW',
      args: [],
      includes: { caps: ['CAP_PERFMON'] },
      excludes: {},
    },
  ],
}, undefined, '\t',)}\n`;

await describe({
  name: 'the seccomp profile the gate\'s containers are given',
  children: [
    it({
      name: 'adds statmount and listmount to the unconditional rule that names mount',
      fn: async () => {
        const derived = derivedSeccompProfile({
          profileText: hostProfileText,
          profilePath: '/usr/share/containers/seccomp.json',
        },);
        expect(derived.added,).toEqual([
          'listmount',
          'statmount',
        ],);
        const parsed = JSON.parse(derived.text,) as { readonly syscalls: readonly { readonly names?: readonly string[]; readonly action?: string; }[]; };
        const allowRule = parsed.syscalls.find(function allowsMount(rule,) {
          return (rule.action === 'SCMP_ACT_ALLOW') && (rule.names
            ?.includes('mount',)
            === true);
        },);
        expect(allowRule?.names,).toEqual([
          'listmount',
          'mknodat',
          'mount',
          'mount_setattr',
          'move_mount',
          'mprotect',
          'statmount',
        ],);
      },
    }),
    it({
      name: 'keeps the rule narrowed by a capability out of the derivation',
      fn: async () => {
        const derived = derivedSeccompProfile({
          profileText: hostProfileText,
          profilePath: '/usr/share/containers/seccomp.json',
        },);
        const parsed = JSON.parse(derived.text,) as { readonly syscalls: readonly { readonly names?: readonly string[]; readonly includes?: { readonly caps?: readonly string[]; }; }[]; };
        const narrowed = parsed.syscalls.find(function requiresPerfmon(rule,) {
          return rule.includes?.caps?.includes('CAP_PERFMON',) === true;
        },);
        expect(narrowed?.names,).toEqual(['perf_event_open'],);
      },
    }),
    it({
      name: 'keeps every other fact of the host\'s profile, including its default action',
      fn: async () => {
        const derived = derivedSeccompProfile({
          profileText: hostProfileText,
          profilePath: '/usr/share/containers/seccomp.json',
        },);
        const parsed = JSON.parse(derived.text,) as { readonly defaultAction?: string; readonly defaultErrnoRet?: number; };
        expect(parsed.defaultAction,).toBe('SCMP_ACT_ERRNO',);
        expect(parsed.defaultErrnoRet,).toBe(38,);
      },
    }),
    it({
      name: 'leaves a profile that already allows both calls as the host wrote it',
      fn: async () => {
        const already = hostProfileText.replace(
          '"mount",',
          '"listmount",\n\t\t\t\t\t"mount",\n\t\t\t\t\t"statmount",',
        );
        const derived = derivedSeccompProfile({
          profileText: already,
          profilePath: '/usr/share/containers/seccomp.json',
        },);
        expect(derived.added,).toEqual([],);
        expect(derived.text,).toBe(already,);
      },
    }),
    it({
      name: 'refuses a profile whose rule naming mount would allow the calls only under a capability',
      fn: async () => {
        const narrowedMount = `${JSON.stringify({
          defaultAction: 'SCMP_ACT_ERRNO',
          syscalls: [
            {
              names: ['mount'],
              action: 'SCMP_ACT_ALLOW',
              includes: { caps: ['CAP_SYS_ADMIN'] },
            },
          ],
        },)}\n`;
        expect(function derive(): unknown {
          return derivedSeccompProfile({
            profileText: narrowedMount,
            profilePath: '/host/containers/seccomp.json',
          },);
        },).toThrow('/host/containers/seccomp.json',);
      },
    }),
    it({
      name: 'refuses text that is not a profile, naming the file it came from',
      fn: async () => {
        expect(function derive(): unknown {
          return derivedSeccompProfile({
            profileText: 'defaultAction: allow\n',
            profilePath: '/host/containers/seccomp.json',
          },);
        },).toThrow('/host/containers/seccomp.json',);
      },
    }),
    it({
      name: 'names the calls the sweep asks and the variable the controls read',
      fn: async () => {
        expect(theCallsTheMountSweepAsks,).toEqual([
          'listmount',
          'statmount',
        ],);
        expect(statmountAllowedVariable,).toBe('CLI_GIT_STATMOUNT_ALLOWED_IN_CONTAINERS',);
      },
    }),
  ],
},);
