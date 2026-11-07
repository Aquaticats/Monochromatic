#!/usr/bin/env node
/**
 Test double for the `btrfs` program, for the unit tests of the move tools.

 A subvolume is a folder. A source subvolume carries a `.stub-subvolume` file whose text is its UUID; every other
 subvolume this double makes carries a `.meta` file of `key=value` lines. Only the subcommands the move tools call
 exist; anything else exits 2.

 Behavior knobs, by environment variable (each names the folder it applies to): `FAIL_SEND` makes the send of that
 snapshot fail, `SLOW_SEND` makes it sleep thirty seconds, `FAIL_RO` makes the received copy of it writable, and `STUBS`
 lists nested folders that a writable snapshot gets, the way a received parent holds empty stubs of its nested
 subvolumes. `BTRFS_DOUBLE_LOG` names a file that gets one JSON line of argv per call.

 @module
 */

import {
  appendFile,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import {
  argv,
  env,
  exit,
  stdout,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  listSubvolumes,
  type Meta,
  readMeta,
  readSourceMeta,
  writeMeta,
} from './btrfs-test-double-meta.ts';
import { pathExists, } from './completion-records.ts';
import { text, } from 'node:stream/consumers';
import { parseTextFields, } from './json-text-fields.ts';

/**
 Milliseconds the slow send sleeps, long enough that a deadline of a few seconds always ends it.
 */
const SLOW_SEND_MILLISECONDS = 30_000;

/**
 Place of the subject path in the arguments of `property get` and `property set` (`property set -ts <path> ro true`).
 */
const SUBJECT_INDEX = 3;

/**
 Place of the value in the arguments of `property set`.
 */
const VALUE_INDEX = 5;

/**
 Exit status for an unsupported subcommand.
 */
const UNSUPPORTED_STATUS = 2;

/**
 Reads all of standard input asynchronously: the sender of a shell pipe may have made the pipe non-blocking, and a
 synchronous read of it then fails with EAGAIN.

 @returns the text
 */
async function readStandardInput(): Promise<string> {
  // Touched only here: a named import of `stdin` would open it in every call, and an open stdin keeps the process alive.
  return await text(process.stdin,);
}

/**
 Takes a snapshot, read-only or writable.

 @param args - the arguments after `subvolume snapshot`
 */
async function snapshot(args: readonly string[],): Promise<void> {
  /**
   The `ro` value read back from btrfs.
   */
  const readOnly = args[0] === '-r';
  /**
   The parts of the value this function uses, taken out of it.
   */
  const [source = '', destination = '',] = readOnly ? args.slice(1,) : args;
  if ((source === '') || (destination === '')
    || await pathExists(destination,)) {
    console.error(`stub: cannot snapshot ${source} to ${destination}`,);
    exit(1,);
  }
  /**
   The metadata of the source: an empty folder with no marker is an empty subvolume made by `subvolume create`.
   */
  const sourceMeta = await readSourceMeta(source,);
  await mkdir(destination,);
  await writeMeta({
    folder: destination,
    meta: {
      uuid: `u-${basename(destination,)}`,
      parent: sourceMeta.uuid ?? '-',
      received: '-',
      ro: String(readOnly,),
      empty: sourceMeta.empty ?? 'false',
    },
  },);
  await Promise.all((readOnly ? [] : (env.STUBS ?? '').split(',',))
    .filter(function notBlank(name,) {
      return name !== '';
    },)
    .map(async function makeStub(stub,) {
      await mkdir(
        join(
          destination,
          stub,
        ),
        { recursive: true, },
      );
    },),);
}

/**
 Sends a snapshot as one JSON line that names it, its parent and its clone sources.

 @param args - the arguments after `send`
 */
async function send(args: readonly string[],): Promise<void> {
  /**
   The snapshot being sent.
   */
  const snapshotPath = args.at(-1,) ?? '';
  /**
   The name of this step.
   */
  const name = basename(snapshotPath,);
  if (env.SLOW_SEND === name) {
    await wait(SLOW_SEND_MILLISECONDS,);
  }
  if (env.FAIL_SEND === name) {
    console.error('stub: send failed',);
    exit(1,);
  }
  /**
   The options of this invocation.
   */
  const options = args.slice(
    0,
    -1,
  );
  /**
   The parent entry of the subvolume, when it has one.
   */
  const parent = options[options.indexOf('-p',) + 1] ?? '';
  /**
   The earlier copies used as clone sources.
   */
  const clones = options.flatMap(function cloneName(
    option,
    index,
  ) {
    return option === '-c' ? [basename(options[index + 1] ?? '',),] : [];
  },);
  if (options.includes('-p',)) {
    /**
     The metadata of the snapshot being sent.
     */
    const sentOnParent = await readMeta(snapshotPath,);
    /**
     The metadata of the parent.
     */
    const parentMeta = await readMeta(parent,);
    // The real kernel tells two inodes apart by number and generation (`changed_inode` in `fs/btrfs/send.c`), so a
    // parent that is not on the sent subvolume's own history can pair two different inodes made in one transaction. Only
    // a snapshot of the same subvolume, or an empty subvolume (nothing to pair), is safe.
    if ((parentMeta.empty !== 'true') && (parentMeta.parent !== sentOnParent.parent)) {
      console.error(`ERROR: unsafe parent ${basename(parent,)} for ${name}: not a snapshot of the same subvolume and not empty`,);
      exit(1,);
    }
  }
  if ((!options.includes('-p',)) && (clones.length > 0)) {
    /**
     The metadata of the snapshot being sent.
     */
    const sentMeta = await readMeta(snapshotPath,);
    /**
     The metadata of every clone source.
     */
    const cloneMetas = await Promise.all(options.flatMap(function clonePath(
      option,
      index,
    ) {
      return option === '-c' ? [options[index + 1] ?? '',] : [];
    },)
      .map(async function metaOfClone(path,) {
        return await readMeta(path,);
      },),);
    // The real `btrfs send` with clone sources and no `-p` searches for a parent (btrfs-progs `find_good_parent`): the
    // subvolume the snapshot was taken from must be a clone source, or a clone source must be a snapshot of the same
    // subvolume. Anything else ends with "parent determination failed".
    if (!cloneMetas.some(function isRelated(clone: Readonly<Meta>,) {
      return (clone.parent === sentMeta.parent) || (clone.uuid === sentMeta.parent);
    },)) {
      console.error(`ERROR: parent determination failed for ${name}`,);
      exit(1,);
    }
  }
  stdout.write(`${
    JSON.stringify({
      uuid: (await readMeta(snapshotPath,)).uuid,
      name,
      parent: options.includes('-p',) ? basename(parent,) : '-',
      clones: clones.join(',',),
      // The real `--no-data` stream between two snapshots of one subvolume holds a command per change and only a header
      // when nothing changed; `CHANGED_SNAPSHOTS` names the snapshots that changed.
      changed: options.includes('--no-data',)
        && (env.CHANGED_SNAPSHOTS ?? '').split(',',)
          .includes(name,) ? 'yes' : 'no',
    },)
  }\n`,);
}

/**
 Receives a stream into a folder, or an empty partial subvolume when the stream is empty.

 @param args - the arguments after `receive`
 */
async function receive(args: readonly string[],): Promise<void> {
  if (args.includes('--dump',)) {
    /**
     The stream of a `send --no-data`.
     */
    const dumped = parseTextFields((await readStandardInput()).trim(),);
    console.log(`snapshot ./${dumped.name ?? ''} uuid=${dumped.uuid ?? ''}`,);
    if (dumped.changed === 'yes') {
      console.log('write ./file offset=0 len=1',);
    }
    return;
  }
  /**
   The input of this step.
   */
  const input = (await readStandardInput()).trim();
  /**
   The destination folder of this step.
   */
  const destinationFolder = args[0] ?? '';
  if (input === '') {
    await mkdir(join(
      destinationFolder,
      'PARTIAL',
    ),);
    await writeMeta({
      folder: join(
        destinationFolder,
        'PARTIAL',
      ),
      meta: {
        uuid: 'r-PARTIAL',
        parent: '-',
        received: '-',
        ro: 'true',
      },
    },);
    return;
  }
  /**
   The stream of this step.
   */
  const stream = parseTextFields(input,);
  /**
   The name of the sent snapshot.
   */
  const sentName = stream.name ?? '';
  /**
   The destination of this step.
   */
  const destination = join(
    destinationFolder,
    sentName,
  );
  await mkdir(destination,);
  await writeMeta({
    folder: destination,
    meta: {
      uuid: `r-${sentName}`,
      parent: '-',
      received: stream.uuid ?? '',
      ro: env.FAIL_RO === sentName ? 'false' : 'true',
      sentParent: stream.parent ?? '',
      sentClones: stream.clones ?? '',
    },
  },);
}

/**
 Runs the subcommand the arguments name.

 @param args - the arguments after the program name
 */
async function main(args: readonly string[],): Promise<void> {
  if (env.BTRFS_DOUBLE_LOG !== undefined) {
    await appendFile(
      env.BTRFS_DOUBLE_LOG,
      `${JSON.stringify(args,)}\n`,
    );
  }
  /**
   The parts of the value this function uses, taken out of it.
   */
  const [first, second, ...rest] = args;
  /**
   The key of this step.
   */
  const key = `${first ?? ''} ${second ?? ''}`;
  /**
   The folder this step works on.
   */
  const folder = args[2] ?? '';
  if (key === 'subvolume snapshot') {
    await snapshot(rest,);
  }
  else if (key === 'subvolume show') {
    /**
     The meta of this step.
     */
    const meta = await readMeta(folder,);
    console.log(
      `UUID:\t\t\t${meta.uuid}\nParent UUID:\t\t${meta.parent}\nReceived UUID:\t\t${meta.received}`,
    );
  }
  else if (key === 'subvolume create') {
    await mkdir(folder,);
  }
  else if ((key === 'subvolume list') && (env.BTRFS_DOUBLE_ROOT !== undefined)) {
    console.log(await listSubvolumes(env.BTRFS_DOUBLE_ROOT,),);
  }
  else if (key === 'property get') {
    console.log(`ro=${(await readMeta(args[SUBJECT_INDEX] ?? '',)).ro}`,);
  }
  else if (key === 'property set') {
    /**
     The path the step writes to or reads from.
     */
    const target = args[SUBJECT_INDEX] ?? '';
    await writeMeta({
      folder: target,
      meta: {
        ...await readMeta(target,),
        ro: args[VALUE_INDEX] ?? 'false',
      },
    },);
  }
  else if (first === 'send') {
    await send(args.slice(1,),);
  }
  else if (first === 'receive') {
    await receive(args.slice(1,),);
  }
  else {
    console.error(`stub: unsupported ${args.join(' ',)}`,);
    exit(UNSUPPORTED_STATUS,);
  }
}

await main(argv.slice(2,),);
