import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isAllowed,
  shouldSkip,
} from './validation.ts';

await describe({
  name: 'validation',
  children: [
    describe({
      name: isAllowed.name,
      children: [
        it({
          name: 'allows a normal command starting with a safe char',
          fn: async () => {
            expect(isAllowed('git status',),).toBe(true,);
          },
        },),
        it({
          name: 'rejects the empty string',
          fn: async () => {
            expect(isAllowed('',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a command starting with a disallowed char',
          fn: async () => {
            expect(isAllowed('!history',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      // hasFileRedirect: indexOf('>') walk + nested skipWs recursion
      name: 'shouldSkip: hasFileRedirect',
      children: [
        it({
          name: 'skips a plain file redirect',
          fn: async () => {
            expect(shouldSkip('cat foo > out.txt',),).toBe(true,);
          },
        },),
        it({
          name: 'skips a redirect with no space before the target',
          fn: async () => {
            expect(shouldSkip('cat foo >out.txt',),).toBe(true,);
          },
        },),
        it({
          name: 'does not skip a descriptor redirect 2>&1',
          fn: async () => {
            expect(shouldSkip('printf hi 2>&1',),).toBe(false,);
          },
        },),
        it({
          name: 'skips malformed redirect syntax conservatively',
          fn: async () => {
            expect(shouldSkip('printf hi > | wc',),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      // hasHeredoc: indexOf('<<') walk + nested skipWs recursion
      name: 'shouldSkip: hasHeredoc',
      children: [
        it({
          name: 'skips a <<EOF heredoc',
          fn: async () => {
            expect(shouldSkip('runner <<EOF',),).toBe(true,);
          },
        },),
        it({
          name: 'skips a <<<word here-string',
          fn: async () => {
            expect(shouldSkip('runner <<<word',),).toBe(true,);
          },
        },),
        it({
          name: 'skips a <<- dash-variant heredoc',
          fn: async () => {
            expect(shouldSkip('runner <<-EOF',),).toBe(true,);
          },
        },),
        it({
          name: 'skips malformed heredoc syntax conservatively',
          fn: async () => {
            expect(shouldSkip('runner << ',),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      // hasTtyContainerInvoke: token-index recursion
      name: 'shouldSkip: hasTtyContainerInvoke',
      children: [
        it({
          name: 'skips docker exec with a -it flag',
          fn: async () => {
            expect(shouldSkip('docker exec -it ctr sh',),).toBe(true,);
          },
        },),
        it({
          name: 'skips podman run with a later -t flag',
          fn: async () => {
            expect(shouldSkip('podman run --rm -t img',),).toBe(true,);
          },
        },),
        it({
          name: 'does not skip docker pull (no tty subcommand)',
          fn: async () => {
            expect(shouldSkip('docker pull ubuntu',),).toBe(false,);
          },
        },),
        it({
          name: 'does not skip docker exec without a tty flag',
          fn: async () => {
            expect(shouldSkip('docker exec ctr ls',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      // hasBunBuild: token-index recursion
      name: 'shouldSkip: hasBunBuild',
      children: [
        it({
          name: 'skips bun build',
          fn: async () => {
            expect(shouldSkip('bun build --watch',),).toBe(true,);
          },
        },),
        it({
          name: 'does not skip bun run build',
          fn: async () => {
            expect(shouldSkip('bun run build',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'shouldSkip: long-input scans stay linear',
      children: [
        it({
          name: 'handles a long command with no skip trigger',
          fn: async () => {
            expect(shouldSkip(`echo ${'a'.repeat(200_000,)}`,),).toBe(false,);
          },
        },),
        it({
          name: 'finds a redirect after a long prefix',
          fn: async () => {
            expect(shouldSkip(`echo ${'a'.repeat(200_000,)} > out`,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
