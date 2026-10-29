/**
 Milliseconds in one second, for converting between the loop interval and rates.
 */
export const MS_PER_SECOND = 1_000;

/**
 Micro-units (microjoules, microwatts) in one unit.
 */
export const MICRO_PER_UNIT = 1_000_000;

/**
 Milli-units (millidegrees Celsius) in one unit.
 */
export const MILLI_PER_UNIT = 1_000;

/**
 Milliseconds in one day, for the sample retention cutoff.
 */
export const MS_PER_DAY = 86_400_000;

/**
 Milliseconds between two watcher passes.
 */
export const INTERVAL_MS = 2_000;

/**
 CFS bandwidth period in microseconds; the quota written to `cpu.max` is the thread count times this.
 */
export const PERIOD_USEC = 100_000;

/**
 Thread count used when the cap file is absent or invalid.
 */
export const DEFAULT_THREADS = 30;

/**
 Consecutive bad samples before a metric raises an alert (3 samples, 6 seconds).
 */
export const BAD_AFTER_SAMPLES = 3;

/**
 Consecutive good samples before an alerting metric clears (5 samples, 10 seconds).
 */
export const CLEAR_AFTER_SAMPLES = 5;

/**
 Thread-count step suggested in the alert text when the cap should be toned down.
 */
export const TONE_DOWN_STEP = 2;

/**
 Smallest thread count the alert text will suggest.
 */
export const MIN_THREADS = 1;

/**
 Loop passes between two sample log lines (5 passes, 10 seconds).
 */
export const SAMPLE_EVERY_ITERATIONS = 5;

/**
 Days of sample log files kept before pruning.
 */
export const SAMPLE_RETENTION_DAYS = 14;

/**
 How long, in milliseconds, every pass is logged after a load step (burst mode).
 */
export const BURST_MS = 90_000;

/**
 Chiplet split: cores 0 to 7 are read as the first chiplet, cores 8 to 15 as the second.
 */
export const CORES_PER_CHIPLET = 8;

/**
 Core count of the Ryzen 9 7950X; the per-core energy counters are used only when all of them exist.
 */
export const EXPECTED_CORES = 16;

/**
 Thermal target of Zen 4 in degrees Celsius; the chip regulates clocks to sit here under sustained load.
 */
export const DEFAULT_THERMAL_TARGET_CELSIUS = 95;

/**
 Clock in MHz the 7950X holds on all busy cores with boost disabled (observed 4477 to 4494 MHz in the sample log).
 */
export const DEFAULT_BOOST_OFF_MHZ = 4_490;

/**
 Package power step in watts between two passes that starts burst mode.
 */
export const DEFAULT_BURST_STEP_WATTS = 25;

/**
 How many unlabeled motherboard temperature inputs the `gigabyte_wmi` driver exposes.
 */
export const BOARD_SENSOR_COUNT = 6;

/**
 Numbers of the six unlabeled motherboard temperature inputs (`temp1_input` to `temp6_input`).
 */
export const BOARD_SENSOR_INDEXES: readonly number[] = Array.from(
  { length: BOARD_SENSOR_COUNT, },
  function sensorNumber(
    _unused,
    index,
  ): number {
    return index + 1;
  },
);

/**
 How many columns the sample log keeps for the Corsair controller (six fan speeds and a probe temperature).
 They stay in the header so rows written before 2026-10-18 keep their positions, and are empty in new rows.
 */
export const LEGACY_CORSAIR_COLUMN_COUNT = 7;

/**
 How many fan inputs the `nct6687` chip exposes (`fan1_input` to `fan8_input`), one per motherboard fan header.
 */
export const MB_FAN_COUNT = 8;

/**
 Numbers of the `nct6687` fan inputs, in the order of the sample log columns.
 */
export const MB_FAN_INPUTS: readonly number[] = Array.from(
  { length: MB_FAN_COUNT, },
  function fanNumber(
    _unused,
    index,
  ): number {
    return index + 1;
  },
);

/**
 Number of the first logged `nct6687` temperature input, `temp2_input` (System). `temp1_input` (CPU) repeats
 the CPU temperature and is not logged.
 */
export const MB_FIRST_TEMPERATURE_INPUT = 2;

/**
 How many `nct6687` temperature inputs are logged: System, VRM MOS and PCH (`temp2_input` to `temp4_input`).
 */
export const MB_TEMPERATURE_COUNT = 3;

/**
 Numbers of the logged `nct6687` temperature inputs, in the order of the sample log columns.
 */
export const MB_TEMPERATURE_INPUTS: readonly number[] = Array.from(
  { length: MB_TEMPERATURE_COUNT, },
  function temperatureNumber(
    _unused,
    index,
  ): number {
    return index + MB_FIRST_TEMPERATURE_INPUT;
  },
);

/**
 Marks a sensor directory or file that does not exist on this host; joined with a name it points nowhere,
 so readers return an empty field instead of throwing.
 */
export const ABSENT: string = '';

/**
 Marks a quantity that could not be measured in this pass (no previous reading, counter reset, missing sensor).
 */
export const NOT_MEASURED: number = Number.NaN;

/**
 Header line of the sample log CSV; column order is part of the interface read by `summarize-samples.ts`.
 New columns are appended, never inserted, so older rows keep their positions.
 */
export const SAMPLE_HEADER: string =
  'utc_time,tctl_c,tccd1_c,tccd2_c,package_w,average_mhz,cpu_pressure_some_avg10,load1,cap_threads,'
    + 'fan1_rpm,fan2_rpm,fan3_rpm,fan4_rpm,fan5_rpm,fan6_rpm,corsair_probe_c,'
    + 'chiplet1_w,chiplet2_w,board1_c,board2_c,board3_c,board4_c,board5_c,board6_c,acpi_c,nvme_c,burst,'
    + 'gpu_edge_c,gpu_w,igpu_edge_c,igpu_w,'
    + 'cpu_fan_rpm,pump_fan_rpm,sys_fan1_rpm,sys_fan2_rpm,sys_fan3_rpm,sys_fan4_rpm,sys_fan5_rpm,sys_fan6_rpm,'
    + 'mb_system_c,mb_vrm_c,mb_pch_c\n';
