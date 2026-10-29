/**
 * Configuration constants for the constrained benchmark runner.
 * Defines container resource limits, device paths, and tuning parameters.
 */

import {
  findRoot,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import spawn from 'nano-spawn';
import { realpath, } from 'node:fs/promises';
import { resolve, } from 'node:path';

/**
 * Canonical monorepo root path, normalized to `/var/home` on Fedora ostree
 * where `/home` is a symlink that breaks `readlink -f` resolution.
 */
export const MONOREPO_ROOT: string = await findRoot({
  cwd: resolve(import.meta.dirname,),
  marker: MISE_MONOREPO,
},);

/** Containerfile location */
export const CONTAINERFILE: string = resolve(import.meta.dirname, '..', 'Containerfile',);

/** Container image name */
export const IMAGE_NAME = 'file-enforcer-perf';

/**
 * The user's cheapest VPS scored 1605 events/sec in sysbench cpu.
 * Even the PEAK score across all contending containers must stay below this.
 */
export const VPS_SYSBENCH_BASELINE = 1_605;

/**
 * Number of containers to run simultaneously on the same CPU core.
 * With 5+ containers sharing 1 core of a Ryzen 7 8700F (~5300 events/sec),
 * each gets ~20% throughput (~1060 events/sec), well below the 1605 baseline.
 * Extra containers ensure peak performance during brief idle windows
 * (when some containers are between benchmark phases) stays below baseline.
 */
export const CONTAINER_COUNT = 5;

/**
 * CPU core to pin all containers to via taskset inside the container.
 * All containers compete for this single core via EEVDF scheduler,
 * creating contention at sub-millisecond timeslice granularity
 * rather than CFS bandwidth throttling's 10-100ms stall periods.
 */
export const CPUSET_CPU = '0';

/**
 * Resolve the canonical block device backing a filesystem path.
 *
 * Uses `findmnt -no SOURCE -T <path>` to discover the source device for the
 * mount, strips any btrfs/bind subvolume suffix (e.g. `[/home]`), and
 * canonicalizes mapper symlinks (e.g. `/dev/mapper/luks-xxx -> /dev/dm-N`)
 * so podman's `--device-read/write` cgroup rules attach to the real
 * device-mapper node where the kernel's IO throttle hooks live.
 *
 * @param root0 - Named arguments object.
 * @param root0.path - Filesystem path whose backing device to resolve.
 * @returns Canonical block device path (e.g. `/dev/dm-0`).
 * @throws If `findmnt` returns no source or fails to execute.
 *
 * @example
 * const device = await detectBlockDevice({ path: '/var/home/user/repo', },);
 * // -> '/dev/dm-0'
 */
async function detectBlockDevice({ path, }: { path: string; },): Promise<string> {
  const { stdout, } = await spawn('findmnt', ['-no', 'SOURCE', '-T', path,],);
  const sourceWithSubpath = stdout.trim();
  if (!sourceWithSubpath)
    throw new Error(`findmnt returned empty source for path: ${path}`,);
  const source = sourceWithSubpath.replace(/\[.*$/, '',);
  return await realpath(source,);
}

/**
 * Block device that backs the volume mount, auto-detected at module load.
 *
 * Walks from `MONOREPO_ROOT` through `findmnt` to the source device
 * (e.g. `/dev/mapper/luks-xxx[/home]`), strips the subvolume suffix, and
 * canonicalizes the mapper symlink to the underlying `/dev/dm-N` node.
 * This matches the device-mapper layer where podman's IO throttle cgroup
 * rules attach.
 */
const BLOCK_DEVICE: string = await detectBlockDevice({ path: MONOREPO_ROOT, },);

/**
 * HDD-like IO limits.
 * Cheap shared HDD: ~75-150 random IOPS, ~80-120 MB/s sequential.
 * Using conservative end: 100 IOPS, 80 MB/s.
 */
const READ_BPS = '80mb';
/** Write bytes per second limit for HDD-like IO simulation */
const WRITE_BPS = '80mb';
/** Read IOPS limit for HDD-like IO simulation */
const READ_IOPS = '100';
/** Write IOPS limit for HDD-like IO simulation */
const WRITE_IOPS = '100';

/**
 * Podman flags for memory and IO constraints.
 * CPU pinning is done via taskset inside the container rather than
 * --cpuset-cpus, avoiding the need for sudo or cpuset delegation.
 */
export const RESOURCE_FLAGS: readonly string[] = [
  '--memory=1g',
  `--device-read-bps=${BLOCK_DEVICE}:${READ_BPS}`,
  `--device-write-bps=${BLOCK_DEVICE}:${WRITE_BPS}`,
  `--device-read-iops=${BLOCK_DEVICE}:${READ_IOPS}`,
  `--device-write-iops=${BLOCK_DEVICE}:${WRITE_IOPS}`,
];
