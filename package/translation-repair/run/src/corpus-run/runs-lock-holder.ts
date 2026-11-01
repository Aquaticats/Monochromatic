import { hostname, } from 'node:os';

import {
  isJsonRecord,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type HostRead,
  startTicksOf,
} from './process-identity.ts';

//region Runs lock holder
// Who a runs lock names, and whether that process still runs (ledger A16).
// Split out of `runs-lock.ts` when the judgement grew past a process-id check.
//
// A LOCK IS GONE ONLY ON POSITIVE EVIDENCE: the machine booted since, the id is
// free, or the process under the id started at another time. Anything this
// host cannot read reads as held, since taking a lock from a live pass is the
// one outcome the lock must never produce.

/**
 Logger every holder line goes through.
 */
const holderLog = contextRoot({ tag: 'runs-lock', },);

/**
 Which process took a lock, as far as the system that took it could say.

 @example
 ```ts
 const identity: LockIdentity = { kind: 'unrecorded', };
 ```
 */
export type LockIdentity =
  | Readonly<{
    /**
     The taker's system said which process it was.
     */
    kind: 'recorded';

    /**
     Machine the lock was taken on.
     */
    host: string;

    /**
     Boot the lock was taken in.
     */
    bootId: string;

    /**
     Namespace the recorded process id counts in.
     */
    pidNamespace: string;

    /**
     When the taking process started, in clock ticks since boot.
     */
    startTicks: string;
  }>
    | Readonly<{
    /**
     Written by an earlier build, or on a system without `/proc`.
     */
      kind: 'unrecorded';
    }>;

/**
 What a lock file records about its holder.

 @example
 ```ts
 const holder: LockHolder = { pid: 1234, startedAt: '2026-08-15T00:00:00.000Z', token: 'a1b2', identity: { kind: 'unrecorded', }, };
 ```
 */
export type LockHolder = Readonly<{
  /**
   Process holding it.
   */
  pid: number;

  /**
   When it took the lock, for a message a human can act on.
   */
  startedAt: string;

  /**
   Random per-acquisition token, so a release removes only the lock this
   acquisition wrote and never a later holder's. Empty on locks
   written before the token existed, which therefore never read as ours.
   */
  token: string;

  /**
   Which process the id named when the lock was taken.
   */
  identity: LockIdentity;
}>;

/**
 What a lock file turned out to say.

 A named outcome rather than an absent holder, because "no readable holder"
 is a state a refusal has to describe, and a message that cannot say whether
 the lock named nobody or could not be read at all leaves an operator
 guessing.

 @example
 ```ts
 const read: HolderRead = { kind: 'unreadable', };
 ```
 */
export type HolderRead =
  | Readonly<{
    /**
     Lock file named a process.
     */
    kind: 'holder';

    /**
     Who it named.
     */
    holder: LockHolder;
  }>
    | Readonly<{
    /**
     Lock file said nothing this can act on.
     */
      kind: 'unreadable';
    }>;

/**
 How a live holder was judged, which the refusal states so an operator knows
 how far to trust it.

 @example
 ```ts
 const judged: HeldJudgement = 'identity';
 ```
 */
export type HeldJudgement = 'host' | 'identity' | 'namespace' | 'pid' | 'race';

/**
 Whether a lock's holder still runs, and how that was judged.

 @example
 ```ts
 const liveness: Liveness = { state: 'held', judgedBy: 'identity', };
 ```
 */
export type Liveness =
  | Readonly<{
    /**
     The holder may still run; the lock is respected.
     */
    state: 'held';

    /**
     How that was judged.
     */
    judgedBy: Exclude<HeldJudgement, 'race'>;
  }>
    | Readonly<{
    /**
     The holder ended; the lock is taken over.
     */
      state: 'gone';

    /**
     The evidence it ended, for the takeover line.
     */
      judgedBy: 'boot' | 'identity' | 'pid';
    }>;

/**
 What the lock file carries on disk: the holder with its identity laid flat,
 so an earlier build reading it still finds `pid` and `startedAt` where it
 always did.

 @param holder - holder to write

 @returns The file's contents

 @example
 ```ts
 await handle.writeFile(lockFileText({ holder, },),);
 ```
 */
export function lockFileText({ holder, }: { readonly holder: LockHolder; },): string {
  /**
   The holder less its nested identity.
   */
  const {
    identity,
    ...named
  } = holder;
  if (identity.kind === 'unrecorded')
    return `${JSON.stringify(named,)}\n`;
  /**
   The identity less its tag.
   */
  const {
    kind: _recorded,
    ...fields
  } = identity;
  return `${JSON.stringify({
    ...named,
    ...fields,
  },)}\n`;
}

/**
 One string field of a parsed lock file, empty where it is absent or not a
 string.

 @param parsed - lock file contents as parsed

 @param field - field to read

 @returns Its text, or empty

 @example
 ```ts
 const token = stringField({ parsed, field: 'token', },);
 ```
 */
function stringField(
  {
    parsed,
    field,
  }: {
    readonly parsed: object;
    readonly field: string;
  },
): string {
  /**
   The field's value, whatever it is.
   */
  const value: unknown = Object.getOwnPropertyDescriptor(
    parsed,
    field,
  )
    ?.value;
  return ((typeof value) === 'string') ? value : '';
}

/**
 The identity a parsed lock file records.

 @param parsed - lock file contents as parsed

 @returns Recorded when all four fields are there, unrecorded otherwise

 @example
 ```ts
 const identity = identityIn({ parsed, },);
 ```
 */
function identityIn({ parsed, }: { readonly parsed: object; },): LockIdentity {
  /**
   Machine the file names.
   */
  const host = stringField({
    parsed,
    field: 'host',
  },);
  /**
   Boot the file names.
   */
  const bootId = stringField({
    parsed,
    field: 'bootId',
  },);
  /**
   Namespace the file names.
   */
  const pidNamespace = stringField({
    parsed,
    field: 'pidNamespace',
  },);
  /**
   Start time the file names.
   */
  const startTicks = stringField({
    parsed,
    field: 'startTicks',
  },);
  return [
    host,
    bootId,
    pidNamespace,
    startTicks,
  ].includes('',)
    ? { kind: 'unrecorded', }
    : {
      kind: 'recorded',
      host,
      bootId,
      pidNamespace,
      startTicks,
    };
}

/**
 Reads what a lock file claims, or nothing when it claims nothing readable.

 @param path - lock file path

 @returns Holder it records, or that the file is unreadable or malformed

 @example
 ```ts
 const holder = await readHolder({ path, },);
 ```
 */
export async function readHolder(
  { path, }: { readonly path: string; },
): Promise<HolderRead> {
  try {
    /**
     Lock file contents as parsed JSON.
     */
    const parsed: unknown = await readRunJson({ path, },);

    if (!isJsonRecord(parsed,))
      return { kind: 'unreadable', };
    if ((!('pid' in parsed)) || ((typeof parsed.pid) !== 'number'))
      return { kind: 'unreadable', };
    if ((!('startedAt' in parsed)) || ((typeof parsed.startedAt) !== 'string'))
      return { kind: 'unreadable', };

    return {
      kind: 'holder',
      holder: {
        pid: parsed.pid,
        startedAt: parsed.startedAt,
        token: stringField({
          parsed,
          field: 'token',
        },),
        identity: identityIn({ parsed, },),
      },
    };
  }
  catch (error) {
    // A lock file that cannot be read is not a lock anyone can respect, and
    // saying so is better than either honouring it forever or ignoring it
    // silently.
    holderLog.warn(`${path} unreadable (${refusalText({ error, },)})`,);
    return { kind: 'unreadable', };
  }
}

/**
 Whether a process id is in use.

 Signal zero performs the permission and existence checks without delivering
 anything, so it answers exactly this question. A process owned by another
 user answers EPERM, which is still in use.

 @param pid - process id from a lock file

 @returns Whether something is running under it

 @example
 ```ts
 const inUse = isAlive({ pid: 1234, },);
 ```
 */
function isAlive({ pid, }: { readonly pid: number; },): boolean {
  try {
    process.kill(
      pid,
      0,
    );
    return true;
  }
  catch (error) {
    // EPERM means it exists and belongs to someone else, which is held rather
    // than free. Logged rather than swallowed, since taking a lock away from a
    // live process is the one outcome this must never produce silently.
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'EPERM')) {
      holderLog.warn(
        `process ${String(pid,)} exists but is not ours; treating the lock as held`,
      );
      return true;
    }
    return false;
  }
}

/**
 Judges a holder by its process id alone.

 @param pid - process id from the lock file

 @returns Held while the id is in use, gone once it is free

 @example
 ```ts
 return byIdAlone({ pid: holder.pid, },);
 ```
 */
function byIdAlone({ pid, }: { readonly pid: number; },): Liveness {
  return isAlive({ pid, },)
    ? {
      state: 'held',
      judgedBy: 'pid',
    }
    : {
      state: 'gone',
      judgedBy: 'pid',
    };
}

/**
 Judges whether a lock's holder still runs.

 In order: a later boot of this machine means it ended, while another
 machine cannot be judged from here, so its lock holds; so does one from
 another id namespace of this boot; then
 the start time under the id says whether the id still names the holder. The
 boot decides whether the machine is this one, and the hostname is asked only
 where the boots differ, since a hostname can change within a boot. Where the
 start time cannot be read (no `/proc`, a hidden process) or the lock records
 none, the id alone answers, and the refusal says so.

 @param holder - who the lock names

 @param here - this host's identity

 @returns Held or gone, and on what evidence

 @example
 ```ts
 const liveness = await holderLiveness({ holder, here: await hostIdentity(), },);
 ```
 */
export async function holderLiveness(
  {
    holder,
    here,
  }: {
    readonly holder: LockHolder;
    readonly here: HostRead;
  },
): Promise<Liveness> {
  /**
   Which process the lock says it named.
   */
  const { identity, } = holder;
  if (identity.kind === 'unrecorded')
    return byIdAlone({ pid: holder.pid, },);
  /**
   Whether the lock names this machine by name. Asked only where the boots
   differ or cannot be compared: a hostname can change within one boot (DHCP,
   `hostnamectl`), and a boot id, random per boot, already proves the same
   machine.
   */
  const sameName = identity.host === hostname();
  if (here.kind === 'unread') {
    return sameName
      ? byIdAlone({ pid: holder.pid, },)
      : {
        state: 'held',
        judgedBy: 'host',
      };
  }
  /**
   This host's boot and namespace.
   */
  const {
    bootId,
    pidNamespace,
  } = here.here;
  if (identity.bootId !== bootId) {
    return sameName
      ? {
        state: 'gone',
        judgedBy: 'boot',
      }
      : {
        state: 'held',
        judgedBy: 'host',
      };
  }
  if (identity.pidNamespace !== pidNamespace) {
    return {
      state: 'held',
      judgedBy: 'namespace',
    };
  }
  /**
   When the process now under the id started.
   */
  const started = await startTicksOf({ pid: holder.pid, },);
  if (started.kind === 'unread')
    return byIdAlone({ pid: holder.pid, },);
  return (started.startTicks === identity.startTicks)
    ? {
      state: 'held',
      judgedBy: 'identity',
    }
    : {
      state: 'gone',
      judgedBy: 'identity',
    };
}

//endregion Runs lock holder
