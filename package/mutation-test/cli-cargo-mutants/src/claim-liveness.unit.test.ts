import {
  mkdir,
  symlink,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  liveOwnerPid,
  noLiveOwner,
} from './claim-liveness.ts';
import type { OwnedRecord } from './claim-record.ts';
import { scratchDirectory } from './claim-test-support.ts';
import { readBootId } from './process-identity.ts';

/**
 The namespace the cases give this process.
 */
const hereNamespace = 'pid:[4026531836]';

/**
 A namespace another set of processes lives in.
 */
const thereNamespace = 'pid:[4026539999]';

/**
 Start time, in clock ticks, of the owner the cases name.
 */
const ownerTicks = '123456';

/**
 Process id the owner has inside its own namespace.
 */
const ownerPid = 77;

/**
 Process id a host assigns to the same process, outside its namespace.
 */
const hostPid = 4_242;

/**
 How old the claim files of the cases are made, so that no rule can keep an owner alive because its file is young.
 */
const ancientMs = 100 * 3_600_000;

/**
 The namespace text that plants an entry whose link cannot be read.
 */
const unreadableLink = 'unreadable';

/**
 Plant one process in a fake process directory, as the kernel shows it.

 @param root - the fake process directory

 @param directoryId - the id the host sees, the name of the entry

 @param innerPid - the id inside its own namespace, the last id of the `NSpid` line

 @param ticks - the start time

 @param namespace - the namespace link, or {@link unreadableLink} for an entry whose link cannot be read

 @example
 ```ts
 await plantProcess({ root: '/r', directoryId: 4242, innerPid: 77, ticks: '1', namespace: 'pid:[1]' });
 ```
 */
async function plantProcess({
  root,
  directoryId,
  innerPid,
  ticks,
  namespace,
}: {
  readonly root: string;
  readonly directoryId: number;
  readonly innerPid: number;
  readonly ticks: string;
  readonly namespace: string;
}): Promise<void> {
  /**
   The entry of the process.
   */
  const entry = join(
    root,
    String(directoryId),
  );
  await mkdir(
    join(
      entry,
      'ns',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      entry,
      'status',
    ),
    `Name:\tcase\nNSpid:\t${directoryId === innerPid ? String(innerPid) : `${String(directoryId)}\t${String(innerPid)}`}\n`,
  );
  await writeFile(
    join(
      entry,
      'stat',
    ),
    `${String(directoryId)} (case) S 1 ${Array.from(
      { length: 17 },
      () => '0',
    )
      .join(' ')} ${ticks} 0\n`,
  );
  /**
   Where the link goes.
   */
  const linkPath = join(
    entry,
    'ns',
    'pid',
  );
  // A regular file where the link belongs reads as an error that is not "missing", as a link the reader may not read does.
  await (namespace === unreadableLink
    ? writeFile(
      linkPath,
      '',
    )
    : symlink(
      namespace,
      linkPath,
    ));
}

/**
 Make a fake process directory in which this process is in {@link hereNamespace}.

 @param prefix - the start of the directory name

 @returns the directory, removed when disposed

 @example
 ```ts
 await using proc = await fakeProc('liveness-');
 ```
 */
async function fakeProc(prefix: string): Promise<AsyncDisposable & { readonly path: string }> {
  /**
   The directory.
   */
  const directory = await scratchDirectory(prefix);
  await plantProcess({
    root: directory.path,
    directoryId: process.pid,
    innerPid: process.pid,
    ticks: '1',
    namespace: hereNamespace,
  });
  return directory;
}

/**
 The record of the owner the cases name.

 @param parameters - the namespace and boot the record carries; absent for the owner's own, and none

 @returns the record

 @example
 ```ts
 await ownerRecord({ namespace: thereNamespace });
 ```
 */
async function ownerRecord({
  namespace,
  boot,
}: {
  readonly namespace?: string;
  readonly boot?: string;
}): Promise<OwnedRecord> {
  return {
    kind: 'owned',
    pid: ownerPid,
    birth: `linux:${ownerTicks}@${boot ?? await readBootId()}`,
    ...(namespace === undefined ? {} : { namespace }),
  };
}

/**
 Write the claim file of a record, aged so that only evidence can keep its owner alive.

 @param directory - the directory of the file

 @returns its path

 @example
 ```ts
 await agedClaimFile(directory.path);
 ```
 */
async function agedClaimFile(directory: string): Promise<string> {
  /**
   The file.
   */
  const path = join(
    directory,
    'item.owner.1',
  );
  await writeFile(
    path,
    '',
  );
  /**
   A time long ago.
   */
  const longAgo = new Date(Date.now() - ancientMs);
  await utimes(
    path,
    longAgo,
    longAgo,
  );
  return path;
}

await describe({
  name: liveOwnerPid.name,
  children: [
    describe({
      name: 'an owner in a PID namespace this process can see into',
      children: [
        it({
          name: 'is alive however old its claim file is, when a process with its inner id and start time is there',
          fn: async () => {
            await using proc = await fakeProc('liveness-visible-');
            await using claims = await scratchDirectory('liveness-claims-');
            await plantProcess({
              root: proc.path,
              directoryId: hostPid,
              innerPid: ownerPid,
              ticks: ownerTicks,
              namespace: thereNamespace,
            });
            expect(await liveOwnerPid({
              record: await ownerRecord({ namespace: thereNamespace }),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
        it({
          name: 'is alive when the link of the process that matches cannot be read, because the id and start time still agree',
          fn: async () => {
            await using proc = await fakeProc('liveness-unreadable-');
            await using claims = await scratchDirectory('liveness-claims-');
            await plantProcess({
              root: proc.path,
              directoryId: hostPid,
              innerPid: ownerPid,
              ticks: ownerTicks,
              namespace: unreadableLink,
            });
            expect(await liveOwnerPid({
              record: await ownerRecord({ namespace: thereNamespace }),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
        it({
          name: 'is gone when its inner id belongs to a process of another start time in that namespace, because the id was reused',
          fn: async () => {
            await using proc = await fakeProc('liveness-reused-');
            await using claims = await scratchDirectory('liveness-claims-');
            await plantProcess({
              root: proc.path,
              directoryId: hostPid,
              innerPid: ownerPid,
              ticks: '999',
              namespace: thereNamespace,
            });
            const path = join(
              claims.path,
              'item.owner.1',
            );
            await writeFile(
              path,
              '',
            );
            expect(await liveOwnerPid({
              record: await ownerRecord({ namespace: thereNamespace }),
              path,
              procRoot: proc.path,
            })).toBe(noLiveOwner);
          },
        }),
        it({
          name: 'is gone when every process can be read, the namespace is seen, and none of them has its inner id',
          fn: async () => {
            await using proc = await fakeProc('liveness-complete-');
            await using claims = await scratchDirectory('liveness-claims-');
            await plantProcess({
              root: proc.path,
              directoryId: hostPid,
              innerPid: ownerPid + 1,
              ticks: ownerTicks,
              namespace: thereNamespace,
            });
            expect(await liveOwnerPid({
              record: await ownerRecord({ namespace: thereNamespace }),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(noLiveOwner);
          },
        }),
        it({
          name: 'stays alive when the namespace is seen but a process cannot be read, because the owner may be that process',
          fn: async () => {
            await using proc = await fakeProc('liveness-incomplete-');
            await using claims = await scratchDirectory('liveness-claims-');
            await plantProcess({
              root: proc.path,
              directoryId: hostPid,
              innerPid: ownerPid + 1,
              ticks: ownerTicks,
              namespace: thereNamespace,
            });
            await plantProcess({
              root: proc.path,
              directoryId: hostPid + 1,
              innerPid: ownerPid + 2,
              ticks: ownerTicks,
              namespace: unreadableLink,
            });
            expect(await liveOwnerPid({
              record: await ownerRecord({ namespace: thereNamespace }),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
      ],
    }),
    describe({
      name: 'an owner this process cannot see',
      children: [
        it({
          name: 'is alive however old its claim file is, because nothing here can prove it dead',
          fn: async () => {
            await using proc = await fakeProc('liveness-blind-');
            await using claims = await scratchDirectory('liveness-claims-');
            expect(await liveOwnerPid({
              record: await ownerRecord({ namespace: thereNamespace }),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
        it({
          name: 'is gone when it began in an earlier boot, in any namespace, though its claim file is young',
          fn: async () => {
            await using proc = await fakeProc('liveness-boot-');
            await using claims = await scratchDirectory('liveness-claims-');
            const path = join(
              claims.path,
              'item.owner.1',
            );
            await writeFile(
              path,
              '',
            );
            expect(await liveOwnerPid({
              record: await ownerRecord({
                namespace: thereNamespace,
                boot: 'an-earlier-boot',
              }),
              path,
              procRoot: proc.path,
            })).toBe(noLiveOwner);
          },
        }),
      ],
    }),
    describe({
      name: 'an owner whose record names no namespace, or whose namespace this process cannot read',
      children: [
        it({
          name: 'is alive when no process of this namespace has its id, because the id may belong to another namespace',
          fn: async () => {
            await using proc = await fakeProc('liveness-nameless-');
            await using claims = await scratchDirectory('liveness-claims-');
            expect(await liveOwnerPid({
              record: await ownerRecord({}),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
        it({
          name: 'is alive when an unrelated process of another start time has its id here, because that id may be another namespace\'s',
          fn: async () => {
            await using proc = await fakeProc('liveness-nameless-reused-');
            await using claims = await scratchDirectory('liveness-claims-');
            await plantProcess({
              root: proc.path,
              directoryId: ownerPid,
              innerPid: ownerPid,
              ticks: '999',
              namespace: hereNamespace,
            });
            expect(await liveOwnerPid({
              record: await ownerRecord({}),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
        it({
          name: 'is alive when a process here has its id and start time',
          fn: async () => {
            await using proc = await fakeProc('liveness-nameless-match-');
            await using claims = await scratchDirectory('liveness-claims-');
            await plantProcess({
              root: proc.path,
              directoryId: ownerPid,
              innerPid: ownerPid,
              ticks: ownerTicks,
              namespace: hereNamespace,
            });
            expect(await liveOwnerPid({
              record: await ownerRecord({}),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
        it({
          name: 'is gone when it began in an earlier boot',
          fn: async () => {
            await using proc = await fakeProc('liveness-nameless-boot-');
            await using claims = await scratchDirectory('liveness-claims-');
            expect(await liveOwnerPid({
              record: await ownerRecord({ boot: 'an-earlier-boot' }),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(noLiveOwner);
          },
        }),
        it({
          name: 'is alive when this process cannot read its own namespace and the owner is not in view',
          fn: async () => {
            await using proc = await scratchDirectory('liveness-self-unknown-');
            await using claims = await scratchDirectory('liveness-claims-');
            expect(await liveOwnerPid({
              record: await ownerRecord({ namespace: thereNamespace }),
              path: await agedClaimFile(claims.path),
              procRoot: proc.path,
            })).toBe(ownerPid);
          },
        }),
      ],
    }),
  ],
});
