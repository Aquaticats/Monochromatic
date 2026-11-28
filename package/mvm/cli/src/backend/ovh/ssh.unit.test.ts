/**
 Unit tests for the ovh backend's ssh and scp argument builders.

 STB boundary: the remote command and remote paths must reach ssh and scp as
 single argv elements, verbatim, never shell-quoted or split, so no remote shell
 can act on adversarial characters. The login target must carry the account the
 image names, never a fixed one. No process is spawned and nothing here talks to
 a host.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ovhConnectionTarget,
  ovhScpPullArgs,
  ovhScpPushArgs,
  ovhSshBaseOpts,
  ovhSshExecArgs,
} from '../../../dist/final/node/index.mjs';

/**
 Adversarial fragments: shell metacharacters, quotes, substitution, a newline,
 spaces, and traversal that must survive verbatim as one argv element.
 */
const ADVERSARIAL = [
  'a;b',
  'a && rm -rf /',
  'a\'b',
  'a"b',
  'a$(reboot)',
  'a`reboot`',
  'a\nb',
  'a b',
  '../../etc/passwd',
];

/**
 Address the tests connect to.
 */
const IP = '192.0.2.10';

/**
 Login account the tests connect as.
 */
const USER = 'debian';

await describe({
  name: 'ovh ssh arg builders',
  children: [
    it({
      name: 'ovhConnectionTarget puts the image own account in front of the address',
      fn: async () => {
        expect(ovhConnectionTarget({
          ip: IP,
          user: USER,
        },),).toBe('debian@192.0.2.10',);
        expect(ovhConnectionTarget({
          ip: IP,
          user: 'root',
        },),).toBe('root@192.0.2.10',);
      },
    },),
    it({
      name: 'ovhSshBaseOpts pins the managed key and disables host-key persistence',
      fn: async () => {
        const opts = ovhSshBaseOpts();
        expect(opts,).toContain('UserKnownHostsFile=/dev/null',);
        expect(opts,).toContain('StrictHostKeyChecking=no',);
        expect(opts,).toContain('-i',);
        expect(
          opts.some(function isKeyPath(option,) {
            return option.endsWith('id_ed25519',);
          },),
        ).toBe(true,);
      },
    },),
    it({
      name: 'ovhSshExecArgs passes the command as a single trailing argv element',
      fn: async () => {
        for (const command of ADVERSARIAL) {
          const args = ovhSshExecArgs({
            command,
            ip: IP,
            user: USER,
          },);
          expect(args.at(-1,),).toBe(command,);
          expect(args.at(-2,),).toBe('debian@192.0.2.10',);
          expect(args.filter(function isCommand(part,) {
            return part === command;
          },).length,).toBe(1,);
        }
      },
    },),
    it({
      name: 'ovhScpPushArgs keeps the remote target one verbatim, unquoted argv element',
      fn: async () => {
        for (const guestPath of ADVERSARIAL) {
          const args = ovhScpPushArgs({
            guestPath,
            hostPath: '/home/user/local',
            ip: IP,
            user: USER,
          },);
          expect(args.at(-1,),).toBe(`debian@192.0.2.10:${guestPath}`,);
          expect(args.at(-2,),).toBe('/home/user/local',);
        }
      },
    },),
    it({
      name: 'ovhScpPullArgs keeps the remote source one verbatim, unquoted argv element',
      fn: async () => {
        for (const guestPath of ADVERSARIAL) {
          const args = ovhScpPullArgs({
            guestPath,
            ip: IP,
            localPath: '/home/user/out',
            user: USER,
          },);
          expect(args.at(-2,),).toBe(`debian@192.0.2.10:${guestPath}`,);
          expect(args.at(-1,),).toBe('/home/user/out',);
        }
      },
    },),
  ],
},);
