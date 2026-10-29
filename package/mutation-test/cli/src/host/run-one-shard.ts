/**
 One shard container round trip: write manifest, run, read report.
 
 @example
 ```ts
 const report = await runOneShard({ manifest, image, repoRoot, resources, selinuxRelabel: false });
 ```
 */

import {
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  buildShardArgs,
  runShardContainer,
  type ShardResources,
} from './podman.ts';
import type {
  ShardManifest,
  ShardReport,
} from '../shard-schema.ts';

/**
 Module logger for shard round trips.
 */
const l = tagged({ tag: 'mutation-test', },);

/**
 Manifest file name mounted into containers (mirrors container/main.ts).
 */
const MANIFEST_FILE_NAME = 'shard-manifest.json';

/**
 Runs one shard container, returning a synthetic all-unrun report when
 the container fails without producing a usable one.
 
 @param options - Manifest, image, and container parameters.
 
 @returns Shard report, real or synthetic.
 
 @mutates options - `JSON.stringify` may invoke hooks on manifest stored in options.
 
 @example
 ```ts
 const report = await runOneShard({ manifest, image, repoRoot, resources, selinuxRelabel: false });
 ```
 */
export async function runOneShard(options: {
  manifest: ShardManifest;
  readonly image: string;
  readonly repoRoot: string;
  readonly resources: ShardResources;
  readonly selinuxRelabel: boolean;
},): Promise<ShardReport> {
  /**
   Logger scoped to this shard.
   */
  const rl = tagged({
    tag: runOneShard.name,
    l,
  },);
  /**
   Host directory holding this shard's manifest.
   */
  const manifestDir = await mkdtemp(join(
    tmpdir(),
    'mutation-manifest-',
  ),);
  /**
   Host directory receiving this shard's report.
   */
  const reportDir = await mkdtemp(join(
    tmpdir(),
    'mutation-report-',
  ),);
  await writeFile(
    join(
      manifestDir,
      MANIFEST_FILE_NAME,
    ),
    `${JSON.stringify(
      options.manifest,
      null,
      2,
    )}\n`,
    'utf8',
  );

  try {
    return await runShardContainer({
      args: buildShardArgs({
        repoRoot: options.repoRoot,
        image: options.image,
        manifestDir,
        reportDir,
        resources: options.resources,
        selinuxRelabel: options.selinuxRelabel,
      },),
      reportDir,
    },);
  }
  catch (error) {
    rl.error(`shard ${options.manifest
      .shardId} container failed: ${String(error,)}`,);
    return {
      schemaVersion: options.manifest
        .schemaVersion,
      shardId: options.manifest
        .shardId,
      baseline: {
        green: true,
        testsMs: 0,
        tsgoMs: 0,
        detail: '',
      },
      results: [],
      unrun: options.manifest
        .mutants
        .map(function toId(mutant,): string {
          return mutant.id;
        },),
      anomaly: `container failed without report: ${String(error,)}`,
    };
  }
}
