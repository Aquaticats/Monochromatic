/**
 Tests for distro-to-package-manager mapping, prerequisite commands,
 and user creation commands.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  detectPackageManager,
  prerequisiteCommand,
  runtimeDependencyCommand,
  userCreationCommand,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'distro',
  children: [
    //region detectPackageManager

    describe({
      name: detectPackageManager.name,
      children: [
        it({
          name: 'ubuntu resolves to apt',
          fn: async () => {
            expect(detectPackageManager('ubuntu',),).toBe('apt',);
          },
        },),

        it({
          name: 'debian resolves to apt',
          fn: async () => {
            expect(detectPackageManager('debian',),).toBe('apt',);
          },
        },),

        it({
          name: 'fedora resolves to dnf',
          fn: async () => {
            expect(detectPackageManager('fedora',),).toBe('dnf',);
          },
        },),

        it({
          name: 'centos resolves to dnf',
          fn: async () => {
            expect(detectPackageManager('centos',),).toBe('dnf',);
          },
        },),

        it({
          name: 'rhel resolves to dnf',
          fn: async () => {
            expect(detectPackageManager('rhel',),).toBe('dnf',);
          },
        },),

        it({
          name: 'rocky resolves to dnf',
          fn: async () => {
            expect(detectPackageManager('rocky',),).toBe('dnf',);
          },
        },),

        it({
          name: 'alma resolves to dnf',
          fn: async () => {
            expect(detectPackageManager('alma',),).toBe('dnf',);
          },
        },),

        it({
          name: 'alpine resolves to apk',
          fn: async () => {
            expect(detectPackageManager('alpine',),).toBe('apk',);
          },
        },),

        it({
          name: 'arch resolves to pacman',
          fn: async () => {
            expect(detectPackageManager('arch',),).toBe('pacman',);
          },
        },),

        it({
          name: 'cachyos resolves to pacman',
          fn: async () => {
            expect(detectPackageManager('cachyos',),).toBe('pacman',);
          },
        },),

        it({
          name: 'cachyos with a tag resolves to pacman',
          fn: async () => {
            expect(detectPackageManager('cachyos:latest',),).toBe('pacman',);
          },
        },),

        it({
          name: 'strips tag suffix before lookup',
          fn: async () => {
            expect(detectPackageManager('fedora:39',),).toBe('dnf',);
          },
        },),

        it({
          name: 'throws on unknown distro',
          fn: async () => {
            expect(function unknownDistro() {
              detectPackageManager('gentoo',);
            },)
              .toThrow('Unknown distro "gentoo"',);
          },
        },),
      ],
    },),

    //endregion detectPackageManager

    //region prerequisiteCommand

    describe({
      name: prerequisiteCommand.name,
      children: [
        it({
          name: 'apt root installs curl and unzip without sudo',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'apt',
              user: 'root',
            },);
            expect(cmd,).toContain('curl',);
            expect(cmd,).toContain('unzip',);
            expect(cmd,).not.toContain('sudo',);
          },
        },),

        it({
          name: 'apt non-root installs sudo',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'apt',
              user: 'user',
            },);
            expect(cmd,).toContain('sudo',);
          },
        },),

        it({
          name: 'dnf root uses dnf install',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'dnf',
              user: 'root',
            },);
            expect(cmd.startsWith('dnf install',),).toBe(true,);
          },
        },),

        it({
          name: 'apk root uses apk add',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'apk',
              user: 'root',
            },);
            expect(cmd.startsWith('apk add',),).toBe(true,);
            expect(cmd,).toContain('bash',);
          },
        },),

        it({
          name: 'pacman root syncs, upgrades and installs without the hook sandbox',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'pacman',
              user: 'root',
            },);
            expect(cmd.startsWith('pacman --disable-sandbox --sync --refresh --sysupgrade',),).toBe(true,);
          },
        },),

        it({
          name: 'dnf non-root includes sudo',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'dnf',
              user: 'user',
            },);
            expect(cmd,).toContain('sudo',);
          },
        },),

        it({
          name: 'apk non-root includes sudo',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'apk',
              user: 'user',
            },);
            expect(cmd,).toContain('sudo',);
          },
        },),

        it({
          name: 'pacman non-root includes sudo',
          fn: async () => {
            const cmd = prerequisiteCommand({
              manager: 'pacman',
              user: 'user',
            },);
            expect(cmd,).toContain('sudo',);
          },
        },),
      ],
    },),

    //endregion prerequisiteCommand

    //region runtimeDependencyCommand

    describe({
      name: runtimeDependencyCommand.name,
      children: [
        it({
          name: 'node on apt installs libatomic1 and jq',
          fn: async () => {
            const cmd = runtimeDependencyCommand({
              manager: 'apt',
              runtime: 'node',
            },);
            expect(cmd,).toBe('apt-get install -y libatomic1 jq',);
          },
        },),

        it({
          name: 'node on dnf installs libatomic and jq',
          fn: async () => {
            const cmd = runtimeDependencyCommand({
              manager: 'dnf',
              runtime: 'node',
            },);
            expect(cmd,).toBe('dnf install -y libatomic jq',);
          },
        },),

        it({
          name: 'node on pacman installs gcc-libs and jq',
          fn: async () => {
            const cmd = runtimeDependencyCommand({
              manager: 'pacman',
              runtime: 'node',
            },);
            expect(cmd,).toBe('pacman --disable-sandbox --sync --refresh --sysupgrade --noconfirm --needed gcc-libs jq',);
          },
        },),

        it({
          name: 'bun needs no system package',
          fn: async () => {
            expect(runtimeDependencyCommand({
              manager: 'apt',
              runtime: 'bun',
            },),)
              .toBe('',);
          },
        },),

        it({
          name: 'deno needs no system package',
          fn: async () => {
            expect(runtimeDependencyCommand({
              manager: 'apt',
              runtime: 'deno',
            },),)
              .toBe('',);
          },
        },),

        it({
          name: 'node on apk throws a glibc diagnostic',
          fn: async () => {
            expect(function alpineNode() {
              runtimeDependencyCommand({
                manager: 'apk',
                runtime: 'node',
              },);
            },)
              .toThrow('official Node.js Linux builds require glibc',);
          },
        },),
      ],
    },),

    //endregion runtimeDependencyCommand

    //region userCreationCommand

    describe({
      name: userCreationCommand.name,
      children: [
        it({
          name: 'root returns empty string',
          fn: async () => {
            expect(userCreationCommand({
              manager: 'apt',
              user: 'root',
            },),)
              .toBe('',);
          },
        },),

        it({
          name: 'non-root with apt uses useradd',
          fn: async () => {
            const cmd = userCreationCommand({
              manager: 'apt',
              user: 'user',
            },);
            expect(cmd,).toContain('useradd -m testuser',);
            expect(cmd,).toContain('NOPASSWD:ALL',);
          },
        },),

        it({
          name: 'non-root with dnf uses useradd',
          fn: async () => {
            const cmd = userCreationCommand({
              manager: 'dnf',
              user: 'user',
            },);
            expect(cmd,).toContain('useradd -m testuser',);
          },
        },),

        it({
          name: 'non-root with apk uses adduser -D',
          fn: async () => {
            const cmd = userCreationCommand({
              manager: 'apk',
              user: 'user',
            },);
            expect(cmd,).toContain('adduser -D testuser',);
            expect(cmd,).toContain('NOPASSWD:ALL',);
          },
        },),

        it({
          name: 'non-root with pacman uses useradd',
          fn: async () => {
            const cmd = userCreationCommand({
              manager: 'pacman',
              user: 'user',
            },);
            expect(cmd,).toContain('useradd -m testuser',);
          },
        },),
      ],
    },),
    //endregion userCreationCommand
  ],
},);
