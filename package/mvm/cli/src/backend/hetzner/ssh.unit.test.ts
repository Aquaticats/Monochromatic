/**
 Unit tests for the ssh/scp argument builders.
 
 STB boundary: the remote command and remote paths must reach ssh/scp as
 single argv elements, verbatim, never shell-quoted or split, so no remote
 shell can act on adversarial characters. No process spawned.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  scpPullArgs,
  scpPushArgs,
  sshBaseOpts,
  sshExecArgs,
} from '@monochromatic-dev/mvm-cli/ts/backend/hetzner/ssh.ts';

/**
 Adversarial fragments: shell metacharacters, quotes, substitution, newline,
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
 Test server IPv4.
 */
const IP = '203.0.113.7';

await describe({
  name: 'hetzner ssh arg builders',
  children: [
    it({
      name: 'sshBaseOpts pins the managed key and disables host-key persistence',
      fn: async () => {
        const opts = sshBaseOpts();
        expect(opts,).toContain('UserKnownHostsFile=/dev/null',);
        expect(opts,).toContain('StrictHostKeyChecking=no',);
        expect(opts,).toContain('-i',);
      },
    },),
    it({
      name: 'sshExecArgs passes the command as a single trailing argv element',
      fn: async () => {
        for (const command of ADVERSARIAL) {
          const args = sshExecArgs({ command, ip: IP, },);
          expect(args.at(-1,),).toBe(command,);
          expect(args.at(-2,),).toBe(`root@${IP}`,);
          expect(args.filter(function isCommand(part,) {
            return part === command;
          },).length,).toBe(1,);
        }
      },
    },),
    it({
      name: 'scpPushArgs keeps the remote target one verbatim, unquoted argv element',
      fn: async () => {
        for (const guestPath of ADVERSARIAL) {
          const args = scpPushArgs({ guestPath, hostPath: '/tmp/local', ip: IP, },);
          expect(args.at(-1,),).toBe(`root@${IP}:${guestPath}`,);
          expect(args.at(-2,),).toBe('/tmp/local',);
        }
      },
    },),
    it({
      name: 'scpPullArgs keeps the remote source one verbatim, unquoted argv element',
      fn: async () => {
        for (const guestPath of ADVERSARIAL) {
          const args = scpPullArgs({ guestPath, ip: IP, localPath: '/tmp/out', },);
          expect(args.at(-2,),).toBe(`root@${IP}:${guestPath}`,);
          expect(args.at(-1,),).toBe('/tmp/out',);
        }
      },
    },),
  ],
},);
