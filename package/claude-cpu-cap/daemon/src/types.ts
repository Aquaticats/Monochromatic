/**
 Metrics the watcher raises alerts for; each is bad at or above its limit.
 */
export type Metric =
  | 'cpu-pressure-some-avg10'
    | 'memory-pressure-some-avg10'
    | 'cpu-thermal-throttle-mhz'
    | 'scheduler-lag-ms';

/**
 Limits per metric, in the unit of the metric.
 */
export type Limits = Readonly<Record<Metric, number>>;

/**
 Locations the watcher reads and writes; every one can be pointed at a fixture directory.
 */
export type Paths = Readonly<{
  /**
   File holding the wanted thread count.
   */
  capFile: string;
  /**
   The `cpu.max` file of the `claude-code-bash` cgroup.
   */
  cgroupFile: string;
  /**
   Directory of the hwmon class (`/sys/class/hwmon`).
   */
  hwmonRoot: string;
  /**
   Directory of the proc filesystem (`/proc`).
   */
  procRoot: string;
  /**
   Directory of the daily sample CSV files.
   */
  sampleDir: string;
}>;

/**
 Sensor files and directories found at start; an empty string ({@link ABSENT}) marks a sensor this host lacks.
 */
export type Sensors = Readonly<{
  /**
   The `k10temp` Tctl input file.
   */
  temperatureFile: string;
  /**
   The `zenergy` socket energy counter file.
   */
  socketEnergyFile: string;
  /**
   The 16 per-core energy counter files, indexed by core, or an empty array.
   */
  coreEnergyFiles: readonly string[];
  /**
   The `gigabyte_wmi` directory.
   */
  boardDir: string;
  /**
   The `nct6687` directory, the motherboard fan headers and board temperatures.
   */
  superIoDir: string;
  /**
   The `acpitz` directory.
   */
  acpiDir: string;
  /**
   The `nvme` directory.
   */
  nvmeDir: string;
  /**
   The discrete `amdgpu` directory (the one with a fan input).
   */
  gpuDir: string;
  /**
   The integrated `amdgpu` directory (no fan input).
   */
  igpuDir: string;
}>;

/**
 Raw energy counters and the time they were read, kept between passes to derive watts.
 */
export type EnergyReading = Readonly<{
  /**
   Socket energy counter in microjoules.
   */
  socketMicrojoules: number;
  /**
   Per-core energy counters in microjoules.
   */
  coreMicrojoules: readonly number[];
  /**
   `performance.now()` at the read, in milliseconds.
   */
  at: number;
}>;

/**
 One pass of power and clock measurements; {@link NOT_MEASURED} marks what could not be derived.
 */
export type PowerSample = Readonly<{
  /**
   Counters to compare against on the next pass.
   */
  reading: EnergyReading;
  /**
   Package watts since the previous pass.
   */
  watts: number;
  /**
   Watts of the first chiplet (cores 0 to 7).
   */
  chiplet1Watts: number;
  /**
   Watts of the second chiplet (cores 8 to 15).
   */
  chiplet2Watts: number;
  /**
   Average clock over all threads in MHz.
   */
  mhz: number;
}>;

/**
 Consecutive bad and good sample counts of one metric and whether it is alerting.
 */
export type Streak = Readonly<{
  /**
   Consecutive samples at or above the limit.
   */
  bad: number;
  /**
   Consecutive samples below the limit.
   */
  good: number;
  /**
   Whether an alert is currently raised.
   */
  alerting: boolean;
}>;

/**
 What a sample did to a streak: raised an alert, cleared one, or nothing.
 */
export type StreakEvent = 'alert' | 'clear' | 'none';

/**
 State carried from one watcher pass to the next.
 */
export type WatchState = Readonly<{
  /**
   Streak per metric.
   */
  streaks: Readonly<Record<Metric, Streak>>;
  /**
   Energy counters from the previous pass.
   */
  reading: EnergyReading;
  /**
   `performance.now()` of the previous tick.
   */
  previousTickAt: number;
  /**
   Number of passes run.
   */
  iteration: number;
  /**
   `performance.now()` until which every pass is sampled (burst mode); 0 when idle.
   */
  burstUntil: number;
  /**
   Package watts of the previous pass, for burst detection.
   */
  previousPassWatts: number;
  /**
   Local date of the last sample file, to prune once per day.
   */
  sampleDay: string;
}>;
