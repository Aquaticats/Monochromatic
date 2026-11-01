import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  limitFromEnv,
  resolveLimits,
  resolvePaths,
  resolveThermalTuning,
} from './paths.ts';

await describe({
  name: 'paths',
  children: [
    describe({
      name: limitFromEnv.name,
      children: [
        it({
          name: 'returns the override as a number or the fallback when unset',
          fn: async function overrides() {
            expect(limitFromEnv({ env: { CPU_WATCH_LIMIT_LAG: '100', }, name: 'LAG', fallback: 250, },),).toBe(100,);
            expect(limitFromEnv({ env: {}, name: 'LAG', fallback: 250, },),).toBe(250,);
          },
        },),
      ],
    },),
    describe({
      name: resolveLimits.name,
      children: [
        it({
          name: 'defaults to the documented limits and honours each override name',
          fn: async function limits() {
            expect(resolveLimits({},),).toEqual({
              'cpu-pressure-some-avg10': 40,
              'memory-pressure-some-avg10': 10,
              'cpu-thermal-throttle-mhz': 300,
              'scheduler-lag-ms': 250,
            },);
            expect(resolveLimits({
              CPU_WATCH_LIMIT_CPU_SOME: '0',
              CPU_WATCH_LIMIT_MEMORY_SOME: '1',
              CPU_WATCH_LIMIT_THROTTLE_MHZ: '2',
              CPU_WATCH_LIMIT_LAG: '3',
            },),).toEqual({
              'cpu-pressure-some-avg10': 0,
              'memory-pressure-some-avg10': 1,
              'cpu-thermal-throttle-mhz': 2,
              'scheduler-lag-ms': 3,
            },);
          },
        },),
      ],
    },),
    describe({
      name: resolveThermalTuning.name,
      children: [
        it({
          name: 'defaults to 95 degrees, 4490 MHz and 25 W and honours overrides',
          fn: async function tuning() {
            expect(resolveThermalTuning({},),).toEqual({ thermalTargetCelsius: 95, boostOffMhz: 4_490, burstStepWatts: 25, },);
            expect(resolveThermalTuning({
              CPU_WATCH_LIMIT_TEMP: '90',
              CPU_WATCH_LIMIT_BOOST_OFF_MHZ: '4000',
              CPU_WATCH_LIMIT_BURST_STEP_W: '10',
            },),).toEqual({ thermalTargetCelsius: 90, boostOffMhz: 4_000, burstStepWatts: 10, },);
          },
        },),
      ],
    },),
    describe({
      name: resolvePaths.name,
      children: [
        it({
          name: 'derives the documented locations from the home directory and user id',
          fn: async function defaultPaths() {
            expect(resolvePaths({ env: {}, uid: 1_234, home: '/home/x', },),).toEqual({
              capFile: '/home/x/.config/claude-cpu-cap/threads',
              cgroupFile: '/sys/fs/cgroup/user.slice/user-1234.slice/user@1234.service/app.slice/claude-code-bash/cpu.max',
              ioWeightFile: '/sys/fs/cgroup/user.slice/user-1234.slice/user@1234.service/app.slice/claude-code-bash/io.weight',
              hwmonRoot: '/sys/class/hwmon',
              procRoot: '/proc',
              sampleDir: '/home/x/.local/share/claude-cpu-cap/samples',
            },);
          },
        },),
        it({
          name: 'lets every CPU_WATCH_* variable override its location',
          fn: async function overriddenPaths() {
            expect(resolvePaths({
              env: {
                CPU_WATCH_CAP_FILE: '/a',
                CPU_WATCH_CGROUP_FILE: '/b',
                CPU_WATCH_IO_WEIGHT_FILE: '/f',
                CPU_WATCH_HWMON_ROOT: '/c',
                CPU_WATCH_PROC_ROOT: '/d',
                CPU_WATCH_SAMPLE_DIR: '/e',
              },
              uid: 1,
              home: '/home/x',
            },),).toEqual({
              capFile: '/a',
              cgroupFile: '/b',
              ioWeightFile: '/f',
              hwmonRoot: '/c',
              procRoot: '/d',
              sampleDir: '/e',
            },);
          },
        },),
      ],
    },),
  ],
},);
