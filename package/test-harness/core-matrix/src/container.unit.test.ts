/**
 Tests for OS specification parsing and container command building.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildContainerCommand,
  parseOs,
  resolveImage,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'container',
  children: [
    //region parseOs

    describe({
      name: parseOs.name,
      children: [
        it({
          name: 'parses container:ubuntu',
          fn: async () => {
            const result = parseOs('container:ubuntu',);
            expect(result.protocol,).toBe('container',);
            expect(result.distro,).toBe('ubuntu',);
          },
        },),

        it({
          name: 'parses container:fedora:39 preserving tag in distro',
          fn: async () => {
            const result = parseOs('container:fedora:39',);
            expect(result.protocol,).toBe('container',);
            expect(result.distro,).toBe('fedora:39',);
          },
        },),

        it({
          name: 'parses host: with empty distro',
          fn: async () => {
            const result = parseOs('host:',);
            expect(result.protocol,).toBe('host',);
            expect(result.distro,).toBe('',);
          },
        },),

        it({
          name: 'parses vm: protocol',
          fn: async () => {
            const result = parseOs('vm:fedora',);
            expect(result.protocol,).toBe('vm',);
            expect(result.distro,).toBe('fedora',);
          },
        },),

        it({
          name: 'throws on missing protocol prefix',
          fn: async () => {
            expect(function noProtocol() {
              parseOs('ubuntu',);
            },)
              .toThrow('must have a protocol prefix',);
          },
        },),

        it({
          name: 'throws on unknown protocol',
          fn: async () => {
            expect(function unknownProtocol() {
              parseOs('docker:ubuntu',);
            },)
              .toThrow('Unknown protocol "docker"',);
          },
        },),
      ],
    },),

    //endregion parseOs

    //region resolveImage

    describe({
      name: resolveImage.name,
      children: [
        it({
          name: 'appends latest to a bare distro name',
          fn: async () => {
            expect(resolveImage({ distro: 'ubuntu', },),).toBe('ubuntu:latest',);
          },
        },),

        it({
          name: 'keeps an explicit tag on an unmapped distro',
          fn: async () => {
            expect(resolveImage({ distro: 'fedora:39', },),).toBe('fedora:39',);
          },
        },),

        it({
          name: 'a bare cachyos uses the image the matrix resolved for the host CPU level',
          fn: async () => {
            expect(resolveImage({
              distro: 'cachyos',
              cachyosImage: 'docker.io/cachyos/cachyos-v4:latest',
            },),).toBe('docker.io/cachyos/cachyos-v4:latest',);
          },
        },),

        it({
          name: 'a bare cachyos without a pre-resolved image is refused',
          fn: async () => {
            expect(function bareCachyos() {
              resolveImage({ distro: 'cachyos', },);
            },).toThrow('pass cachyosImage from resolveBaseImage',);
          },
        },),

        it({
          name: 'cachyos with an explicit tag skips the ladder and keeps the tag on the plain repository',
          fn: async () => {
            expect(resolveImage({
              distro: 'cachyos:latest',
              cachyosImage: 'docker.io/cachyos/cachyos-v4:latest',
            },),).toBe('docker.io/cachyos/cachyos:latest',);
            expect(resolveImage({ distro: 'cachyos:testing', },),).toBe('docker.io/cachyos/cachyos:testing',);
          },
        },),

        it({
          name: 'a name that only starts with cachyos is not remapped',
          fn: async () => {
            expect(resolveImage({ distro: 'cachyos-v3', },),).toBe('cachyos-v3:latest',);
          },
        },),

        it({
          name: 'a name inherited from Object.prototype is treated as an ordinary distro',
          fn: async () => {
            expect(resolveImage({ distro: 'constructor', },),).toBe('constructor:latest',);
          },
        },),
      ],
    },),

    //endregion resolveImage

    //region buildContainerCommand

    describe({
      name: buildContainerCommand.name,
      children: [
        it({
          name: 'root user builds a direct command chain',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/home/user/Mono/packages/foo/test.ts',
                os: 'container:ubuntu',
                user: 'root',
                runtime: 'bun',
              },
              monorepoRoot: '/home/user/Mono',
            },);

            /** Should be a && chain without sudo -u or heredoc. */
            expect(cmd,).toContain('apt-get',);
            expect(cmd,).toContain('curl',);
            expect(cmd,).toContain('.bun/bin/bun',);
            expect(cmd,).toContain('/workspace/packages/foo/test.ts',);
            expect(cmd,).not.toContain('sudo -u',);
            expect(cmd,).not.toContain('run-test.sh',);
          },
        },),

        it({
          name: 'non-root user writes heredoc script and uses sudo -u',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/home/user/Mono/packages/foo/test.ts',
                os: 'container:ubuntu',
                user: 'user',
                runtime: 'bun',
              },
              monorepoRoot: '/home/user/Mono',
            },);

            expect(cmd,).toContain('sudo -u testuser',);
            expect(cmd,).toContain('run-test.sh',);
            expect(cmd,).toContain('useradd',);
            expect(cmd,).toContain('NOPASSWD',);
          },
        },),

        it({
          name: 'non-root heredoc terminator sits on its own line',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/home/user/Mono/packages/foo/test.ts',
                os: 'container:ubuntu',
                user: 'user',
                runtime: 'bun',
              },
              monorepoRoot: '/home/user/Mono',
            },);

            /** A delimiter sharing a line with `&& chmod` never terminates the heredoc. */
            expect(cmd,).toContain('TESTSCRIPT\nchmod +x /tmp/run-test.sh',);
            expect(cmd,).not.toContain('TESTSCRIPT &&',);
            expect(cmd,).toContain('sudo -u testuser -i /tmp/run-test.sh',);
          },
        },),

        it({
          name: 'converts host path to /workspace container path',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/root/packages/bar/src/test.ts',
                os: 'container:fedora',
                user: 'root',
                runtime: 'bun',
              },
              monorepoRoot: '/repo/root',
            },);

            expect(cmd,).toContain('/workspace/packages/bar/src/test.ts',);
            expect(cmd,).not.toContain('/repo/root',);
          },
        },),

        it({
          name: 'uses dnf for fedora',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:fedora',
                user: 'root',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('dnf install',);
          },
        },),

        it({
          name: 'uses pacman for cachyos',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:cachyos',
                user: 'root',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('pacman -Sy --noconfirm curl unzip',);
            expect(cmd,).not.toContain('sudo',);
          },
        },),

        it({
          name: 'cachyos non-root installs sudo with pacman and creates the user',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:cachyos',
                user: 'user',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('pacman -Sy --noconfirm curl unzip sudo',);
            expect(cmd,).toContain('useradd -m testuser',);
          },
        },),

        it({
          name: 'node on cachyos installs gcc-libs and jq with pacman',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:cachyos',
                user: 'root',
                runtime: 'node',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('pacman -Sy --noconfirm gcc-libs jq',);
          },
        },),

        it({
          name: 'uses apk for alpine',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:alpine',
                user: 'root',
                runtime: 'deno',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('apk add',);
            expect(cmd,).toContain('.deno/bin/deno',);
          },
        },),

        it({
          name: 'alpine non-root uses adduser -D instead of useradd',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:alpine',
                user: 'user',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('adduser -D testuser',);
            expect(cmd,).not.toContain('useradd',);
          },
        },),

        it({
          name: 'deno exec includes --allow-all flag',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'root',
                runtime: 'deno',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('--allow-all',);
          },
        },),

        it({
          name: 'non-root heredoc includes cd /workspace',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'user',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('cd /workspace',);
          },
        },),

        it({
          name: 'root command includes cd /workspace',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'root',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('cd /workspace',);
          },
        },),

        it({
          name: 'node root install installs jq and libatomic, then reads the locked Node from toolchain.lock.json',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'root',
                runtime: 'node',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('apt-get install -y libatomic1 jq',);
            expect(cmd,).toContain('toolchain.lock.json',);
            expect(cmd,).toContain('sha256sum --check',);
            expect(cmd.indexOf('cd /workspace',),)
              .toBeLessThan(cmd.indexOf('toolchain.lock.json',),);
            expect(cmd.indexOf('toolchain.lock.json',),)
              .toBeLessThan(cmd.indexOf('tar --extract',),);
          },
        },),

        it({
          name: 'node root exec runs the workspace file through the locked node binary',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/home/user/Mono/packages/foo/src/bar.ts',
                os: 'container:ubuntu',
                user: 'root',
                runtime: 'node',
              },
              monorepoRoot: '/home/user/Mono',
            },);

            expect(cmd,).toContain(
              '$HOME/node/bin/node'
                + ' /workspace/packages/foo/src/bar.ts',
            );
          },
        },),

        it({
          name: 'node non-root installs jq and libatomic as root and cds before install in heredoc',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'user',
                runtime: 'node',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain('sudo -u testuser',);
            expect(cmd,).toContain('apt-get install -y libatomic1 jq',);
            expect(cmd,).toContain('cd /workspace',);
            expect(cmd,).toContain('toolchain.lock.json',);
            expect(cmd.indexOf('cd /workspace',),)
              .toBeLessThan(cmd.indexOf('toolchain.lock.json',),);
          },
        },),

        it({
          name: 'node non-root installs each root-only step exactly once',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'user',
                runtime: 'node',
              },
              monorepoRoot: '/repo',
            },);

            /** The heredoc body runs as testuser, so root-only steps stay outside it. */
            const heredoc = cmd.slice(cmd.indexOf('TESTSCRIPT',),);

            expect(cmd.split('apt-get install -y libatomic1 jq',).length
              - 1,)
              .toBe(1,);
            expect(cmd.split('tar --extract',).length
              - 1,)
              .toBe(1,);
            expect(heredoc,).not.toContain('libatomic',);
          },
        },),

        it({
          name: 'non-root bun runtime install appears once, in the heredoc',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'user',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd.split('bun.sh/install',).length
              - 1,)
              .toBe(1,);
          },
        },),

        it({
          name: 'node exec runs the locked node binary inside the non-root heredoc',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'user',
                runtime: 'node',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).toContain(
              '$HOME/node/bin/node /workspace/test.ts',
            );
          },
        },),

        it({
          name: 'node on alpine throws instead of generating a musl-incompatible command',
          fn: async () => {
            expect(function buildAlpineNode() {
              buildContainerCommand({
                combination: {
                  file: '/repo/test.ts',
                  os: 'container:alpine',
                  user: 'root',
                  runtime: 'node',
                },
                monorepoRoot: '/repo',
              },);
            },)
              .toThrow('official Node.js Linux builds require glibc',);
          },
        },),

        it({
          name: 'bun keeps its command free of node runtime system packages',
          fn: async () => {
            const cmd = buildContainerCommand({
              combination: {
                file: '/repo/test.ts',
                os: 'container:ubuntu',
                user: 'root',
                runtime: 'bun',
              },
              monorepoRoot: '/repo',
            },);

            expect(cmd,).not.toContain('libatomic',);
            expect(cmd,).not.toContain('mise',);
          },
        },),
      ],
    },),
    //endregion buildContainerCommand
  ],
},);
