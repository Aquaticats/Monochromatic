/**
 Podman argv construction and shard container execution.
 
 @example
 ```ts
 const args = buildShardArgs({ repoRoot, image, manifestDir, reportDir, resources, selinuxRelabel: false });
 ```
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  BAKED_ENTRYPOINT,
  MANIFEST_MOUNT,
  REPORT_MOUNT,
  SOURCE_MOUNT,
  WORK_MOUNT,
} from '../mounts.ts';
import { isRecord, } from '../is-record.ts';
import {
  SHARD_SCHEMA_VERSION,
  type ShardReport,
} from '../shard-schema.ts';

/**
 Module logger for host-side container execution.
 */
const l = tagged({ tag: 'mutation-test', },);

/**
 Report file name the container writes (mirrors container/main.ts).
 */
export const REPORT_FILE_NAME = 'shard-report.json';

/**
 Resource caps applied to each shard container.
 */
export type ShardResources = {
  readonly memory: string;
  readonly cpus: string;
  readonly pidsLimit: number;
  readonly sessionTimeoutSeconds: number;
  readonly workTmpfsSize: string;
};

/**
 Formats one volume mount argument.
 
 @param options - Host path, container path, mode, and SELinux toggle.
 
 @returns Volume argument value.
 
 @example
 ```ts
 volumeMount({ hostPath: '/repo', containerPath: '/src-ro', mode: 'ro', selinuxRelabel: false });
 // '/repo:/src-ro:ro'
 ```
 */
function volumeMount(options: {
  readonly hostPath: string;
  readonly containerPath: string;
  readonly mode: 'ro' | 'rw';
  readonly selinuxRelabel: boolean;
},): string {
  /**
   Mode string with optional SELinux relabel suffix.
   */
  const mode = options.selinuxRelabel ? `${options.mode},z` : options.mode;
  return `${options.hostPath}:${options.containerPath}:${mode}`;
}

/**
 Builds the podman argv for one shard container.
 
 Hardened flags ported from the proven per-file container setup:
 no network, read-only root, dropped capabilities, tmpfs work tree.
 
 @param options - Paths, image, and resource caps.
 
 @returns Full podman argv (starting with `run`).
 
 @example
 ```ts
 buildShardArgs({ repoRoot, image, manifestDir, reportDir, resources, selinuxRelabel: false });
 ```
 */
export function buildShardArgs(options: {
  readonly repoRoot: string;
  readonly image: string;
  readonly manifestDir: string;
  readonly reportDir: string;
  readonly resources: ShardResources;
  readonly selinuxRelabel: boolean;
},): readonly string[] {
  return [
    'run',
    '--rm',
    '--pull=never',
    '--network=none',
    '--read-only',
    '--cap-drop=ALL',
    '--security-opt=no-new-privileges',
    '--security-opt=label=disable',
    '--memory',
    options.resources
      .memory,
    '--cpus',
    options.resources
      .cpus,
    '--pids-limit',
    String(options.resources
      .pidsLimit,),
    '--timeout',
    String(options.resources
      .sessionTimeoutSeconds,),
    '--tmpfs',
    '/tmp:rw,exec,size=1g',
    '--tmpfs',
    `${WORK_MOUNT}:rw,exec,size=${options.resources
      .workTmpfsSize}`,
    '--env',
    'HOME=/tmp',
    '--env',
    'TMPDIR=/tmp',
    '--env',
    'XDG_CACHE_HOME=/tmp/.cache',
    '--volume',
    volumeMount({
      hostPath: options.repoRoot,
      containerPath: SOURCE_MOUNT,
      mode: 'ro',
      selinuxRelabel: options.selinuxRelabel,
    },),
    '--volume',
    volumeMount({
      hostPath: options.manifestDir,
      containerPath: MANIFEST_MOUNT,
      mode: 'ro',
      selinuxRelabel: options.selinuxRelabel,
    },),
    '--volume',
    volumeMount({
      hostPath: options.reportDir,
      containerPath: REPORT_MOUNT,
      mode: 'rw',
      selinuxRelabel: options.selinuxRelabel,
    },),
    '--workdir',
    WORK_MOUNT,
    options.image,
    'node',
    BAKED_ENTRYPOINT,
  ];
}

/**
 Returns whether a parsed JSON value has the shard report shape.
 
 @param value - Parsed JSON value.
 
 @returns Whether value is a usable report.
 
 @example
 ```ts
 isShardReport(JSON.parse(raw));
 ```
 */
function isShardReport(value: unknown,): value is ShardReport {
  if (!isRecord(value,))
    return false;

  /**
   Record view over the candidate report.
   */
  const record = value;
  return (record.schemaVersion === SHARD_SCHEMA_VERSION)
    && ((typeof record.shardId) === 'string')
    && Array.isArray(record.results,)
    && Array.isArray(record.unrun,);
}

/**
 Runs one shard container and reads back its report.
 
 @param options - Podman argv pieces and the report directory.
 
 @returns Parsed shard report.
 
 @throws Error when the container fails or the report is unreadable.
 
 @example
 ```ts
 const report = await runShardContainer({ args, reportDir });
 ```
 */
export async function runShardContainer(options: {
  readonly args: readonly string[];
  readonly reportDir: string;
},): Promise<ShardReport> {
  /**
   Logger scoped to this container run.
   */
  const rl = tagged({
    tag: runShardContainer.name,
    l,
  },);
  await spawn(
    'podman',
    options.args,
    {
      stdout: 'inherit',
      stderr: 'inherit',
    },
  );

  /**
   Raw report text written by the container.
   */
  const raw = await readFile(
    join(
      options.reportDir,
      REPORT_FILE_NAME,
    ),
    'utf8',
  );
  /**
   Parsed report before shape validation.
   */
  const parsed: unknown = JSON.parse(raw,);

  if (!isShardReport(parsed,))
    throw new Error(`shard report in ${options.reportDir} has unsupported shape`,);

  rl.debug(`report ${parsed.shardId}: ${String(parsed.results
    .length,)} results`,);
  return parsed;
}
