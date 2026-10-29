/**
 Tests of file transfer and VM creation through the built `mvm` executable,
 against the stand-in `virsh` and its simulated guest.

 Two routes are covered: the virtiofs shared directory, which must behave as
 it always did, and the guest agent route for hosts where libvirt finds no
 `virtiofsd`. Each test has its own sandbox, so the tests run side by side.

 @module
 */

import {
  createHash,
  randomBytes,
} from 'node:crypto';
import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  GuestFileTransferError,
  pushThroughAgent,
} from '../dist/final/node/index.mjs';
import {
  createSandbox,
  fakeVirshEnv,
  runCli,
  type Sandbox,
  writeExecutable,
} from './fixture.cli-harness.ts';

//region Helpers

/**
 Bytes mvm sends per write command; file sizes here are chosen around it.
 */
const WRITE_CHUNK = 96_000;

/**
 Bytes mvm asks for per read command.
 */
const READ_CHUNK = 1_048_576;

/**
 Writes VM metadata into the sandbox, as `mvm create` would have.
 */
async function writeMeta({
  meta,
  name,
  sandbox,
}: {
  readonly meta: Readonly<Record<string, unknown>>;
  readonly name: string;
  readonly sandbox: Sandbox;
},): Promise<string> {
  const vmDirectory = join(
    sandbox.vms,
    name,
  );
  await mkdir(
    vmDirectory,
    { recursive: true, },
  );
  await writeFile(
    join(
      vmDirectory,
      'meta.json',
    ),
    JSON.stringify({
      createdAt: '2026-10-05T00:00:00.000Z',
      defaultUser: 'ubuntu',
      image: 'ubuntu',
      osFamily: 'linux',
      shell: '/bin/bash',
      ...meta,
    },),
  );
  return vmDirectory;
}

/**
 Creates a directory inside the simulated guest and returns its host path.
 */
async function guestDirectory({
  sandbox,
  segments,
}: {
  readonly sandbox: Sandbox;
  readonly segments: readonly string[];
},): Promise<string> {
  const directory = join(
    sandbox.state,
    'guest',
    ...segments,
  );
  await mkdir(
    directory,
    { recursive: true, },
  );
  return directory;
}

/**
 Writes a host file of `size` random bytes into the sandbox and returns its path and content.
 */
async function hostFile({
  name,
  sandbox,
  size,
}: {
  readonly name: string;
  readonly sandbox: Sandbox;
  readonly size: number;
},): Promise<{ readonly content: Buffer; readonly path: string; }> {
  const content = randomBytes(size,);
  const path = join(
    sandbox.home,
    name,
  );
  await writeFile(
    path,
    content,
  );
  return {
    content,
    path,
  };
}

/**
 SHA-256 of some bytes, so a mismatch prints two short digests instead of megabytes.
 */
function digest(content: Buffer,): string {
  return createHash('sha256',)
    .update(content,)
    .digest('hex',);
}

/**
 Names of the guest agent file handles still open in the simulated guest.
 */
async function openHandleNames(sandbox: Sandbox,): Promise<readonly string[]> {
  return await readdir(join(
    sandbox.state,
    'handles',
  ),);
}

/**
 Prepares what `mvm create` needs besides virsh: a cached template image and a stand-in `qemu-img`.
 */
async function prepareCreate(sandbox: Sandbox,): Promise<void> {
  const images = join(
    sandbox.home,
    '.local',
    'share',
    'mvm',
    'images',
  );
  await mkdir(
    images,
    { recursive: true, },
  );
  await writeFile(
    join(
      images,
      'template-ubuntu.qcow2',
    ),
    '',
  );
  await writeExecutable({
    body: [
      'const { writeFileSync } = require(\'node:fs\');',
      'const args = process.argv.slice(2);',
      'if (args[0] === \'create\') writeFileSync(args.at(-2), \'\');',
    ].join('\n',),
    name: 'qemu-img',
    sandbox,
  },);
}

/**
 Metadata of a Linux VM created on a host without virtiofsd.
 */
const AGENT_ROUTE = { fileTransfer: 'guest-agent', };

/**
 Sets environment variables of this process for the scope, restoring the prior values on disposal.
 */
function withProcessEnv(variables: Readonly<Record<string, string>>,): Disposable {
  const prior = Object.keys(variables,).map(function remember(key,) {
    return [
      key,
      process.env[key],
    ] as const;
  },);
  Object.assign(
    process.env,
    variables,
  );
  return {
    [Symbol.dispose]() {
      for (
        const [
          key,
          value,
        ] of prior
      ) {
        if (value === undefined)
          Reflect.deleteProperty(process.env, key,);
        else
          process.env[key] = value;
      }
    },
  };
}

/**
 Runs `operation` and returns what it threw; fails when it did not throw.
 */
async function caught(operation: () => unknown,): Promise<unknown> {
  try {
    await operation();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the operation to throw',);
}

//endregion Helpers

await describe({
  name: 'mvm file transfer and creation',
  children: [
    //region Shared directory route: behavior from before the guest agent route existed

    describe({
      name: 'with the virtiofs shared directory',
      children: [
        it({
          name: 'push copies the file into the shared directory and reports the guest mount path, without virsh',
          fn: async () => {
            await using sandbox = await createSandbox({},);
            const vmDirectory = await writeMeta({
              meta: {},
              name: 'dev',
              sandbox,
            },);
            const source = await hostFile({
              name: 'setup.sh',
              sandbox,
              size: 5_000,
            },);
            const result = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                '/some/dir/renamed.sh',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(result.exitCode,).toBe(0,);
            expect(result.stdout,).toContain('/mnt/shared/renamed.sh',);
            const shared = await readFile(join(
              vmDirectory,
              'shared',
              'renamed.sh',
            ),);
            expect(digest(shared,),).toBe(digest(source.content,),);
            expect(await readdir(sandbox.state,),).not.toContain('calls.jsonl',);
          },
        },),

        it({
          name: 'push to a Windows VM reports the drive of the shared mount',
          fn: async () => {
            await using sandbox = await createSandbox({},);
            await writeMeta({
              meta: {
                fileTransfer: 'virtiofs',
                osFamily: 'windows',
                shell: 'powershell.exe',
              },
              name: 'win',
              sandbox,
            },);
            const source = await hostFile({
              name: 'tool.zip',
              sandbox,
              size: 100,
            },);
            const result = await runCli({
              args: [
                'push',
                'win',
                source.path,
                'tool.zip',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(result.exitCode,).toBe(0,);
            expect(result.stdout,).toContain(String.raw`Z:\tool.zip`,);
          },
        },),

        it({
          name: 'pull reads the file from the shared directory',
          fn: async () => {
            await using sandbox = await createSandbox({},);
            const vmDirectory = await writeMeta({
              meta: {},
              name: 'dev',
              sandbox,
            },);
            const content = randomBytes(7_000,);
            await mkdir(join(
              vmDirectory,
              'shared',
            ),);
            await writeFile(
              join(
                vmDirectory,
                'shared',
                'report.bin',
              ),
              content,
            );
            const destination = join(
              sandbox.home,
              'pulled.bin',
            );
            const result = await runCli({
              args: [
                'pull',
                'dev',
                '/mnt/shared/report.bin',
                destination,
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(result.exitCode,).toBe(0,);
            expect(
              digest(await readFile(destination,),),
            ).toBe(digest(content,),);
          },
        },),

        it({
          name: 'create defines the domain with the share and its memory backing when libvirt can serve it',
          fn: async () => {
            await using sandbox = await createSandbox({ virtiofsd: true, },);
            await prepareCreate(sandbox,);
            const result = await runCli({
              args: [
                'create',
                'dev',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(result.exitCode,).toBe(0,);
            const vmDirectory = join(
              sandbox.vms,
              'dev',
            );
            const xml = await readFile(
              join(
                vmDirectory,
                'domain.xml',
              ),
              'utf8',
            );
            expect(xml,).toContain(
              `<filesystem type="mount" accessmode="passthrough"><driver type="virtiofs" /><source dir="${
                join(
                  vmDirectory,
                  'shared',
                )
              }" /><target dir="mvm-shared" /></filesystem>`,
            );
            expect(xml,).toContain(
              '<memoryBacking><source type="memfd" /><access mode="shared" /></memoryBacking>',
            );
            const meta = JSON.parse(await readFile(
              join(
                vmDirectory,
                'meta.json',
              ),
              'utf8',
            ),) as { fileTransfer: string; };
            expect(meta.fileTransfer,).toBe('virtiofs',);
            const entries = await readdir(vmDirectory,);
            expect(entries,).toContain('shared',);
            expect(entries,).not.toContain('virtiofs-probe.xml',);
            const seed = await readFile(join(
              vmDirectory,
              'seed.iso',
            ),);
            expect(seed.includes('[mvm-shared, /mnt/shared, virtiofs,',),).toBe(true,);
          },
        },),

        it({
          name: 'create keeps the share when the check fails for a reason other than a missing virtiofsd',
          fn: async () => {
            await using sandbox = await createSandbox({
              probeFailure: 'error: this function is not supported by the connection driver',
              virtiofsd: true,
            },);
            await prepareCreate(sandbox,);
            const result = await runCli({
              args: [
                'create',
                'dev',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(result.exitCode,).toBe(0,);
            const xml = await readFile(
              join(
                sandbox.vms,
                'dev',
                'domain.xml',
              ),
              'utf8',
            );
            expect(xml,).toContain('<driver type="virtiofs" />',);
          },
        },),
      ],
    },),

    //endregion Shared directory route

    //region Guest agent route

    describe({
      name: 'without virtiofsd',
      children: [
        it({
          name: 'create defines the domain without the share or its memory backing and records the guest agent route',
          fn: async () => {
            await using sandbox = await createSandbox({},);
            await prepareCreate(sandbox,);
            const result = await runCli({
              args: [
                'create',
                'dev',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            // The stand-in refuses to start a domain with a share, as libvirt does here, so success alone shows the share is gone.
            expect(result.exitCode,).toBe(0,);
            const vmDirectory = join(
              sandbox.vms,
              'dev',
            );
            const xml = await readFile(
              join(
                vmDirectory,
                'domain.xml',
              ),
              'utf8',
            );
            expect(xml,).not.toContain('filesystem',);
            expect(xml,).not.toContain('memoryBacking',);
            expect(xml,).toContain('org.qemu.guest_agent.0',);
            const meta = JSON.parse(await readFile(
              join(
                vmDirectory,
                'meta.json',
              ),
              'utf8',
            ),) as { fileTransfer: string; };
            expect(meta.fileTransfer,).toBe('guest-agent',);
            const entries = await readdir(vmDirectory,);
            expect(entries,).not.toContain('shared',);
            expect(entries,).not.toContain('virtiofs-probe.xml',);
            const seed = await readFile(join(
              vmDirectory,
              'seed.iso',
            ),);
            expect(seed.includes('virtiofs',),).toBe(false,);
            expect(seed.includes('hostname: dev',),).toBe(true,);
          },
        },),

        it({
          name: 'pushes and pulls a binary file whose guest path has spaces and non-ASCII characters',
          fn: async () => {
            await using sandbox = await createSandbox({},);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            const guestDirectoryPath = await guestDirectory({
              sandbox,
              segments: [
                'tmp',
                'dir with space',
              ],
            },);
            const source = await hostFile({
              name: 'payload.bin',
              sandbox,
              size: (3 * WRITE_CHUNK) + 12_345,
            },);
            const guestPath = '/tmp/dir with space/r\u00E9sum\u00E9 \u6F22\u5B57 \u{1F600}.bin';
            const pushed = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                guestPath,
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).toBe(0,);
            expect(pushed.stdout,).toContain(guestPath,);
            const inGuest = await readFile(join(
              guestDirectoryPath,
              'r\u00E9sum\u00E9 \u6F22\u5B57 \u{1F600}.bin',
            ),);
            expect(digest(inGuest,),).toBe(digest(source.content,),);

            const destination = join(
              sandbox.home,
              'pulled back.bin',
            );
            const pulled = await runCli({
              args: [
                'pull',
                'dev',
                guestPath,
                destination,
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pulled.exitCode,).toBe(0,);
            expect(
              digest(await readFile(destination,),),
            ).toBe(digest(source.content,),);
            expect(await openHandleNames(sandbox,),).toEqual([],);
          },
        },),

        it({
          name: 'pushes and pulls through a Windows path, where the end of the file shows only as an empty read',
          fn: async () => {
            await using sandbox = await createSandbox({ readEof: 'windows', },);
            await writeMeta({
              meta: {
                ...AGENT_ROUTE,
                osFamily: 'windows',
                shell: 'powershell.exe',
              },
              name: 'win',
              sandbox,
            },);
            const guestDirectoryPath = await guestDirectory({
              sandbox,
              segments: [
                'C',
                'Temp',
                'my tools',
              ],
            },);
            const source = await hostFile({
              name: 'tool.bin',
              sandbox,
              size: WRITE_CHUNK + 1,
            },);
            const guestPath = 'C:\\Temp\\my tools\\t\u00F6\u00F6l.bin';
            const pushed = await runCli({
              args: [
                'push',
                'win',
                source.path,
                guestPath,
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).toBe(0,);
            expect(
              digest(
                await readFile(join(
                  guestDirectoryPath,
                  't\u00F6\u00F6l.bin',
                ),),
              ),
            ).toBe(digest(source.content,),);
            const destination = join(
              sandbox.home,
              'tool.back',
            );
            const pulled = await runCli({
              args: [
                'pull',
                'win',
                guestPath,
                destination,
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pulled.exitCode,).toBe(0,);
            expect(
              digest(await readFile(destination,),),
            ).toBe(digest(source.content,),);
          },
        },),

        ...[
          {
            label: 'an empty file',
            size: 0,
          },
          {
            label: 'a file of exactly two write chunks',
            size: 2 * WRITE_CHUNK,
          },
          {
            label: 'a file of exactly one read chunk',
            size: READ_CHUNK,
          },
          {
            label: 'a file spanning several read chunks',
            size: (2 * READ_CHUNK) + 77,
          },
        ].map(function roundTrips({
          label,
          size,
        },) {
          return it({
            name: `round-trips ${label}`,
            fn: async () => {
              await using sandbox = await createSandbox({},);
              await writeMeta({
                meta: AGENT_ROUTE,
                name: 'dev',
                sandbox,
              },);
              const guestDirectoryPath = await guestDirectory({
                sandbox,
                segments: ['tmp',],
              },);
              const source = await hostFile({
                name: 'source.bin',
                sandbox,
                size,
              },);
              const destination = join(
                sandbox.home,
                'back.bin',
              );
              const pushed = await runCli({
                args: [
                  'push',
                  'dev',
                  source.path,
                  '/tmp/file.bin',
                ],
                env: fakeVirshEnv(sandbox,),
                sandbox,
              },);
              expect(pushed.exitCode,).toBe(0,);
              const inGuest = await readFile(join(
                guestDirectoryPath,
                'file.bin',
              ),);
              expect(digest(inGuest,),).toBe(digest(source.content,),);
              const pulled = await runCli({
                args: [
                  'pull',
                  'dev',
                  '/tmp/file.bin',
                  destination,
                ],
                env: fakeVirshEnv(sandbox,),
                sandbox,
              },);
              expect(pulled.exitCode,).toBe(0,);
              const back = await readFile(destination,);
              expect(back.length,).toBe(size,);
              expect(digest(back,),).toBe(digest(source.content,),);
            },
            timeout: 300_000,
          },);
        },),

        it({
          name: 'push repeats a write that got no answer although the guest applied it, without shifting the data',
          fn: async () => {
            await using sandbox = await createSandbox({ writeUnansweredAtOffset: WRITE_CHUNK, },);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            const guestDirectoryPath = await guestDirectory({
              sandbox,
              segments: ['tmp',],
            },);
            const source = await hostFile({
              name: 'source.bin',
              sandbox,
              size: (3 * WRITE_CHUNK) + 5,
            },);
            const pushed = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                '/tmp/file.bin',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).toBe(0,);
            const inGuest = await readFile(join(
              guestDirectoryPath,
              'file.bin',
            ),);
            expect(inGuest.length,).toBe(source.content.length,);
            expect(digest(inGuest,),).toBe(digest(source.content,),);
            expect(await readdir(sandbox.state,),).toContain('write-unanswered.used',);
          },
        },),

        it({
          name: 'pull repeats a read that got no answer although the guest advanced, without skipping data',
          fn: async () => {
            await using sandbox = await createSandbox({ readUnansweredAtOffset: 0, },);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            const guestDirectoryPath = await guestDirectory({
              sandbox,
              segments: ['tmp',],
            },);
            const content = randomBytes(50_000,);
            await writeFile(
              join(
                guestDirectoryPath,
                'log.bin',
              ),
              content,
            );
            const destination = join(
              sandbox.home,
              'log.back',
            );
            const pulled = await runCli({
              args: [
                'pull',
                'dev',
                '/tmp/log.bin',
                destination,
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pulled.exitCode,).toBe(0,);
            expect(
              digest(await readFile(destination,),),
            ).toBe(digest(content,),);
            expect(await readdir(sandbox.state,),).toContain('read-unanswered.used',);
          },
        },),

        it({
          name: 'push keeps asking while the guest agent stays silent for several commands, then finishes without shifting the data',
          fn: async () => {
            await using sandbox = await createSandbox({
              silentCommandsAfterUnanswered: 4,
              writeUnansweredAtOffset: WRITE_CHUNK,
            },);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            const guestDirectoryPath = await guestDirectory({
              sandbox,
              segments: ['tmp',],
            },);
            const source = await hostFile({
              name: 'source.bin',
              sandbox,
              size: (3 * WRITE_CHUNK) + 5,
            },);
            const pushed = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                '/tmp/file.bin',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).toBe(0,);
            expect(pushed.stdout,).toContain('asking the guest agent in mvm-dev again',);
            const inGuest = await readFile(join(
              guestDirectoryPath,
              'file.bin',
            ),);
            expect(digest(inGuest,),).toBe(digest(source.content,),);
            expect(
              await readFile(
                join(
                  sandbox.state,
                  'silent-commands.remaining',
                ),
                'utf8',
              ),
            ).toBe('0',);
          },
          timeout: 120_000,
        },),

        it({
          name: 'push stops at once when the domain shut off while the guest agent was silent',
          fn: async () => {
            await using sandbox = await createSandbox({
              domstate: 'shut off',
              silentCommandsAfterUnanswered: 50,
              writeUnansweredAtOffset: WRITE_CHUNK,
            },);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            await guestDirectory({
              sandbox,
              segments: ['tmp',],
            },);
            const source = await hostFile({
              name: 'source.bin',
              sandbox,
              size: 3 * WRITE_CHUNK,
            },);
            const pushed = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                '/tmp/file.bin',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).not.toBe(0,);
            expect(pushed.stderr,).toContain('The domain mvm-dev is now shut off',);
            expect(pushed.stderr,).toContain(
              `stopped after ${String(WRITE_CHUNK,)} of ${String(3 * WRITE_CHUNK,)} bytes`,
            );
            const repositions = (await sandbox.calls()).filter((call,) =>
              (call.at(-1,) ?? '').includes('guest-file-seek',)
            );
            expect(repositions.length,).toBe(0,);
          },
        },),

        it({
          name: 'push gives up when the guest agent stays silent for the limit, and says for how long',
          fn: async () => {
            await using sandbox = await createSandbox({
              silentCommandsAfterUnanswered: 50,
              writeUnansweredAtOffset: 0,
            },);
            await guestDirectory({
              sandbox,
              segments: ['tmp',],
            },);
            const source = await hostFile({
              name: 'source.bin',
              sandbox,
              size: 10,
            },);
            using _environment = withProcessEnv(fakeVirshEnv(sandbox,),);
            const error = await caught(() =>
              pushThroughAgent({
                domain: 'mvm-dev',
                guestPath: '/tmp/file.bin',
                hostPath: source.path,
                limits: {
                  retryPauseMs: 0,
                  unresponsiveLimitMs: 0,
                },
              },)
            );
            expect(error,).toBeInstanceOf(GuestFileTransferError,);
            expect((error as Error).message,).toContain('has not answered a file command for 0 seconds',);
            expect((error as Error).message,).toContain('while the domain is running',);
            expect((error as InstanceType<typeof GuestFileTransferError>).bytesTransferred,).toBe(0,);
          },
        },),

        it({
          name: 'push that the guest refuses midway says how far it got and that the guest file is incomplete',
          fn: async () => {
            await using sandbox = await createSandbox({ writeErrorAtOffset: 2 * WRITE_CHUNK, },);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            await guestDirectory({
              sandbox,
              segments: ['tmp',],
            },);
            const source = await hostFile({
              name: 'source.bin',
              sandbox,
              size: 4 * WRITE_CHUNK,
            },);
            const pushed = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                '/tmp/file.bin',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).not.toBe(0,);
            expect(pushed.stderr,).toContain(
              `stopped after ${String(2 * WRITE_CHUNK,)} of ${String(4 * WRITE_CHUNK,)} bytes`,
            );
            expect(pushed.stderr,).toContain('No space left on device',);
            expect(pushed.stderr,).toContain('/tmp/file.bin is incomplete in the guest',);
            expect(await openHandleNames(sandbox,),).toEqual([],);
          },
        },),

        it({
          name: 'push reports a write the guest cut short',
          fn: async () => {
            await using sandbox = await createSandbox({ shortWriteAtOffset: 0, },);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            await guestDirectory({
              sandbox,
              segments: ['tmp',],
            },);
            const source = await hostFile({
              name: 'source.bin',
              sandbox,
              size: WRITE_CHUNK,
            },);
            const pushed = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                '/tmp/file.bin',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).not.toBe(0,);
            expect(pushed.stderr,).toContain(
              `The guest wrote ${String(WRITE_CHUNK / 2,)} of ${String(WRITE_CHUNK,)} bytes at offset 0`,
            );
            expect(await openHandleNames(sandbox,),).toEqual([],);
          },
        },),

        it({
          name: 'push into a directory that does not exist in the guest says the directory must exist',
          fn: async () => {
            await using sandbox = await createSandbox({},);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            const source = await hostFile({
              name: 'source.bin',
              sandbox,
              size: 10,
            },);
            const pushed = await runCli({
              args: [
                'push',
                'dev',
                source.path,
                '/no/such/dir/file.bin',
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pushed.exitCode,).not.toBe(0,);
            expect(pushed.stderr,).toContain('No such file or directory',);
            expect(pushed.stderr,).toContain('check that its directory exists in the guest',);
          },
        },),

        it({
          name: 'pull of a file that does not exist in the guest fails and writes nothing on the host',
          fn: async () => {
            await using sandbox = await createSandbox({},);
            await writeMeta({
              meta: AGENT_ROUTE,
              name: 'dev',
              sandbox,
            },);
            const pulled = await runCli({
              args: [
                'pull',
                'dev',
                '/tmp/absent.bin',
                join(
                  sandbox.home,
                  'absent.back',
                ),
              ],
              env: fakeVirshEnv(sandbox,),
              sandbox,
            },);
            expect(pulled.exitCode,).not.toBe(0,);
            expect(pulled.stderr,).toContain('Pulling /tmp/absent.bin from mvm-dev stopped after 0 bytes',);
            expect(await readdir(sandbox.home,),).not.toContain('absent.back',);
          },
        },),

        ...[
          {
            guestPath: 'setup.sh',
            meta: AGENT_ROUTE,
            suggestion: '/tmp/setup.sh',
          },
          {
            guestPath: String.raw`Temp\setup.ps1`,
            meta: {
              ...AGENT_ROUTE,
              osFamily: 'windows',
              shell: 'powershell.exe',
            },
            suggestion: 'C:\\Windows\\Temp\\',
          },
        ].map(function rejectsRelative({
          guestPath,
          meta,
          suggestion,
        },) {
          return it({
            name: `refuses the guest path ${guestPath}, which is not absolute, before touching the guest`,
            fn: async () => {
              await using sandbox = await createSandbox({},);
              await writeMeta({
                meta,
                name: 'dev',
                sandbox,
              },);
              const source = await hostFile({
                name: 'source.bin',
                sandbox,
                size: 10,
              },);
              const pushed = await runCli({
                args: [
                  'push',
                  'dev',
                  source.path,
                  guestPath,
                ],
                env: fakeVirshEnv(sandbox,),
                sandbox,
              },);
              expect(pushed.exitCode,).not.toBe(0,);
              expect(pushed.stderr,).toContain('has no shared directory',);
              expect(pushed.stderr,).toContain(suggestion,);
              expect(await readdir(sandbox.state,),).not.toContain('calls.jsonl',);
            },
          },);
        },),
      ],
    },),

    //endregion Guest agent route
  ],
},);
