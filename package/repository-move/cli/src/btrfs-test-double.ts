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
import { pathExists, } from './completion-records.ts';

/**
 Milliseconds the slow send sleeps, long enough that a deadline of a few seconds always ends it.
 */
const SLOW_SEND_MILLISECONDS = 30_000;

/**
 Name of the marker file of a source subvolume.
 */
const SOURCE_MARKER = '.stub-subvolume';

/**
 Name of the metadata file of every other subvolume.
 */
const META_FILE = '.meta';

/**
 Exit status for an unsupported subcommand.
 */
const UNSUPPORTED_STATUS = 2;

/**
 The key and value pairs of one subvolume's metadata.
 */
type Meta = Record<string, string>;

/**
 Path of the metadata file of a subvolume.

 @param folder - the subvolume folder
 
 @returns the path
 */
function metaPath(folder: string,): string {
  return join(
    folder,
    META_FILE,
  );
}

/**
 Reads the metadata of a subvolume, or the implied metadata of a source subvolume.

 @param folder - the subvolume folder
 
 @returns the metadata
 
 @throws Error when the folder is not a subvolume of this double
 */
async function readMeta(folder: string,): Promise<Meta> {
  if (await pathExists(metaPath(folder,),)) {
    return Object.fromEntries((await readFile(
      metaPath(folder,),
      'utf8',
    )).split('\n',)
      .filter(
      function notBlank(line,) {
        return line !== '';
      },
      )
      .map(function toPair(line,) {
        return line.split('=',);
      },),) as Meta;
  }
  /**
   The marker this step searches for.
   */
  const marker = join(
    folder,
    SOURCE_MARKER,
  );
  if (!await pathExists(marker,)) {
    throw new Error(`not a subvolume: ${folder}`,);
  }
  return {
    uuid: (await readFile(
      marker,
      'utf8',
    )).trim(),
    parent: '-',
    received: '-',
    ro: 'false',
  };
}

/**
 Writes the metadata of a subvolume.

 @param folder - the subvolume folder
 
 @param meta - the metadata
 */
async function writeMeta(
  {
    folder,
    meta,
  }: {
    readonly folder: string;
    readonly meta: Readonly<Meta>;
  },
): Promise<void> {
  await writeFile(
    metaPath(folder,),
    `${
      Object.entries(meta,)
        .map(function toLine([key, value,]: readonly [
          string,
          string
        ],) {
        return `${key}=${value}`;
      },)
        .join('\n',)
    }\n`,
  );
}

/**
 Reads all of standard input asynchronously: the sender of a shell pipe may have made the pipe non-blocking, and a
 synchronous read of it then fails with EAGAIN.

 @returns the text
 */
async function readStandardInput(): Promise<string> {
  /**
   The chunks read from standard input.
   */
  const chunks: Buffer[] = [];
  // Touched only here: a named import of `stdin` would open it in every call, and an open stdin keeps the process alive.
  for await (const chunk of process.stdin) {
    chunks.push(chunk as Buffer,);
  }
  return Buffer.concat(chunks,)
    .toString('utf8',);
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
  await mkdir(destination,);
  await writeMeta({
    folder: destination,
    meta: {
      uuid: `u-${basename(destination,)}`,
      parent: (await readMeta(source,)).uuid ?? '-',
      received: '-',
      ro: String(readOnly,),
    },
  },);
  for (const stub of readOnly ? [] : (env.STUBS ?? '').split(',',)
    .filter(function notBlank(name,) {
      return name !== '';
    },)) {
    await mkdir(
      join(
        destination,
        stub,
      ),
      { recursive: true, },
    );
  }
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
  stdout.write(`${
    JSON.stringify({
      uuid: (await readMeta(snapshotPath,)).uuid,
      name,
      parent: options.includes('-p',) ? basename(parent,) : '-',
      clones: clones.join(',',),
    },)
  }\n`,);
}

/**
 Receives a stream into a folder, or an empty partial subvolume when the stream is empty.

 @param args - the arguments after `receive`
 */
async function receive(args: readonly string[],): Promise<void> {
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
  const stream = JSON.parse(input,) as {
    readonly uuid: string;
    readonly name: string;
    readonly parent: string;
    readonly clones: string;
  };
  /**
   The destination of this step.
   */
  const destination = join(
    destinationFolder,
    stream.name,
  );
  await mkdir(destination,);
  await writeMeta({
    folder: destination,
    meta: {
      uuid: `r-${stream.name}`,
      parent: '-',
      received: stream.uuid,
      ro: env.FAIL_RO === stream.name ? 'false' : 'true',
      sentParent: stream.parent,
      sentClones: stream.clones,
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
  else if (key === 'property get') {
    console.log(`ro=${(await readMeta(args[3] ?? '',)).ro}`,);
  }
  else if (key === 'property set') {
    /**
     The path the step writes to or reads from.
     */
    const target = args[3] ?? '';
    await writeMeta({
      folder: target,
      meta: {
        ...await readMeta(target,),
        ro: args[5] ?? 'false',
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
