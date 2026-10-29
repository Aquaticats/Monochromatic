import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  DEFAULT_BOOST_OFF_MHZ,
  DEFAULT_BURST_STEP_WATTS,
  DEFAULT_THERMAL_TARGET_CELSIUS,
} from './constants.ts';
import type {
  Limits,
  Paths,
} from './types.ts';

/**
 Reads a numeric `CPU_WATCH_LIMIT_<name>` override from the environment.

 
 @param env - environment to read
 
 @param name - suffix after `CPU_WATCH_LIMIT_`
 
 @param fallback - value used when the variable is unset

 @returns the override, or the fallback when unset

 @example
 ```ts
 limitFromEnv({ env: { CPU_WATCH_LIMIT_LAG: '100', }, name: 'LAG', fallback: 250, },); // 100
 ```
 */
export function limitFromEnv({
  env,
  name,
  fallback,
}: Readonly<{
  env: Readonly<NodeJS.ProcessEnv>;
  name: string;
  fallback: number;
}>,): number {
  /**
   Raw override text, undefined when the variable is unset.
   */
  const raw = env[`CPU_WATCH_LIMIT_${name}`];
  return raw === undefined ? fallback : Number(raw,);
}

/**
 Builds the alert limits; each metric is bad at or above its limit.
 The throttle limit of 300 MHz sits above everything seen while the chip regulated at 95 degrees for 8 minutes at 133 W
 (125 samples: shortfall median 83, p99 221, max 255); 100 flapped on that normal regulation (2026-10-13).

 @param env - environment with optional `CPU_WATCH_LIMIT_*` overrides

 @returns limits per metric

 @example
 ```ts
 resolveLimits({},)['scheduler-lag-ms']; // 250
 ```
 */
export function resolveLimits(env: Readonly<NodeJS.ProcessEnv>,): Limits {
  return {
    'cpu-pressure-some-avg10': limitFromEnv({
      env,
      name: 'CPU_SOME',
      fallback: 40,
    },),
    'memory-pressure-some-avg10': limitFromEnv({
      env,
      name: 'MEMORY_SOME',
      fallback: 10,
    },),
    'cpu-thermal-throttle-mhz': limitFromEnv({
      env,
      name: 'THROTTLE_MHZ',
      fallback: 300,
    },),
    'scheduler-lag-ms': limitFromEnv({
      env,
      name: 'LAG',
      fallback: 250,
    },),
  };
}

/**
 Reads the thermal target, the boost-off clock and the burst step from the environment.

 @param env - environment with optional `CPU_WATCH_LIMIT_*` overrides

 @returns the three tuning values

 @example
 ```ts
 resolveThermalTuning({},).thermalTargetCelsius; // 95
 ```
 */
export function resolveThermalTuning(env: Readonly<NodeJS.ProcessEnv>,): Readonly<{
  thermalTargetCelsius: number;
  boostOffMhz: number;
  burstStepWatts: number;
}> {
  return {
    thermalTargetCelsius: limitFromEnv({
      env,
      name: 'TEMP',
      fallback: DEFAULT_THERMAL_TARGET_CELSIUS,
    },),
    boostOffMhz: limitFromEnv({
      env,
      name: 'BOOST_OFF_MHZ',
      fallback: DEFAULT_BOOST_OFF_MHZ,
    },),
    burstStepWatts: limitFromEnv({
      env,
      name: 'BURST_STEP_W',
      fallback: DEFAULT_BURST_STEP_WATTS,
    },),
  };
}

/**
 Resolves every path the watcher touches; each `CPU_WATCH_*_FILE`, `_DIR` or `_ROOT` variable overrides one,
 which is how tests point the watcher at fixtures instead of the real cgroup and sample log.

 
 @param env - environment with optional path overrides
 
 @param uid - numeric user id, used in the cgroup path
 
 @param home - home directory, used for the config and sample locations

 @returns all paths

 @example
 ```ts
 resolvePaths({ env: {}, uid: 1000, home: '/home/user', },).capFile; // '/home/user/.config/claude-cpu-cap/threads'
 ```
 */
export function resolvePaths({
  env,
  uid,
  home = homedir(),
}: Readonly<{
  env: Readonly<NodeJS.ProcessEnv>;
  uid: number;
  home?: string;
}>,): Paths {
  return {
    capFile: env.CPU_WATCH_CAP_FILE ?? join(
      home,
      '.config',
      'claude-cpu-cap',
      'threads',
    ),
    cgroupFile: env.CPU_WATCH_CGROUP_FILE
      ?? `/sys/fs/cgroup/user.slice/user-${uid}.slice/user@${uid}.service/app.slice/claude-code-bash/cpu.max`,
    hwmonRoot: env.CPU_WATCH_HWMON_ROOT ?? '/sys/class/hwmon',
    procRoot: env.CPU_WATCH_PROC_ROOT ?? '/proc',
    sampleDir: env.CPU_WATCH_SAMPLE_DIR ?? join(
      home,
      '.local',
      'share',
      'claude-cpu-cap',
      'samples',
    ),
  };
}
