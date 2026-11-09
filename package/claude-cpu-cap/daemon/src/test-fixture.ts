import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { EXPECTED_CORES, } from './constants.ts';
import type { Paths, } from './types.ts';

// The recording logger lives in its own file to keep this one under the line limit; tests keep importing it here.
export {
  recordingLogger,
  type RecordingLogger,
} from './recording-logger.ts';

/**
 Digits in a per-core energy label (`Ecore000`).
 */
const CORE_LABEL_DIGITS = 3;

/**
 One file name with its contents, as `Object.entries()` yields it.
 */
type FileEntry = readonly [
  string,
  string
];

/**
 Writes the files of one hwmon entry.

 @param dir - entry directory
 
 @param files - file names with their contents
 */
async function writeEntry({
  dir,
  files,
}: Readonly<{
  dir: string;
  files: Readonly<Record<string, string>>
}>,): Promise<void> {
  await mkdir(
    dir,
    { recursive: true, },
  );
  await Promise.all(Object.entries(files,)
    .map(async function writeOne([name, text,]: FileEntry,) {
      await writeFile(
        join(
          dir,
          name,
        ),
        text,
      );
    },),);
}

/**
 Creates a disposable host: a hwmon tree resembling the 7950X machine, proc files, a cap file, a cgroup `cpu.max`
 with both weight files (`io.weight` and `io.bfq.weight`, each at the kernel default of 100), and a sample directory,
 so tests never touch the real system. Tests of a kernel that offers one weight file only remove the other.

 @param threads - text written to the cap file
 
 @param cpuPressure - `avg10` written to the CPU pressure file

 @returns the fixture paths and the root directory, which removes itself when the
   caller's scope ends

 @example
 ```ts
 await using host = await makeHost({ threads: '28', cpuPressure: 1, },);
 const { paths, } = host;
 ```
 */
export async function makeHost({
  threads,
  cpuPressure,
}: Readonly<{
  threads: string;
  cpuPressure: number;
}>,): Promise<Readonly<{
  root: string;
  paths: Paths
  [Symbol.asyncDispose]: () => Promise<void>;
}>> {
  /**
   Root of the disposable host.
   */
  const directory = await mkdtempDisposable(join(
    scratchRoot(),
    'cpu-cap-host-',
  ),);
  /**
   Fixture paths are built from a plain path, so cases pass them on unchanged.
   */
  const root = directory.path;
  /**
   Per-core energy counter files, one label and one input each.
   */
  const coreFiles: Record<string, string> = Object.fromEntries(
    Array.from(
      { length: EXPECTED_CORES, },
      function coreEntries(
        _unused,
        core,
      ): readonly (readonly [
        string,
        string
      ])[] {
        return [
          [
            `energy${core + 2}_label`,
            `Ecore${String(core,)
              .padStart(
              CORE_LABEL_DIGITS,
              '0',
            )}`,
          ],
          [
            `energy${core + 2}_input`,
            '1000000',
          ],
        ];
      },
    )
      .flat(),
  );
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon0',
    ),
    files: {
      name: 'k10temp\n',
      temp1_input: '61500\n',
      temp3_input: '58000\n',
      temp4_input: '57000\n',
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon1',
    ),
    files: {
      name: 'zenergy\n',
      energy1_label: 'Esocket0\n',
      energy1_input: '1000000\n',
      ...coreFiles,
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon2',
    ),
    files: {
      name: 'gigabyte_wmi\n',
      temp1_input: '30000\n',
      temp2_input: '31000\n',
      temp3_input: '32000\n',
      temp4_input: '33000\n',
      temp5_input: '34000\n',
      temp6_input: '35000\n',
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon3',
    ),
    files: {
      name: 'acpitz\n',
      temp1_input: '16800\n',
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon4',
    ),
    files: {
      name: 'nvme\n',
      temp1_input: '40000\n',
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon5',
    ),
    files: {
      name: 'amdgpu\n',
      fan1_input: '900\n',
      temp1_input: '45000\n',
      power1_average: '12000000\n',
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon7',
    ),
    files: {
      name: 'nct6687\n',
      fan1_input: '1500\n',
      fan2_input: '0\n',
      fan3_input: '1540\n',
      fan4_input: '0\n',
      fan5_input: '2050\n',
      fan6_input: '0\n',
      fan7_input: '2020\n',
      fan8_input: '2590\n',
      temp1_input: '78000\n',
      temp2_input: '35000\n',
      temp3_input: '40500\n',
      temp4_input: '35500\n',
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'hwmon',
      'hwmon6',
    ),
    files: {
      name: 'amdgpu\n',
      temp1_input: '50000\n',
      power1_input: '3000000\n',
    },
  },);
  await writeEntry({
    dir: join(
      root,
      'proc',
      'pressure',
    ),
    files: {
      cpu: `some avg10=${cpuPressure.toFixed(2,)} avg60=0.00 avg300=0.00 total=0\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n`,
      memory: 'some avg10=0.00 avg60=0.00 avg300=0.00 total=0\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n',
    },
  },);
  await writeFile(
    join(
      root,
      'proc',
      'cpuinfo',
    ),
    'cpu MHz\t\t: 4400.0\ncpu MHz\t\t: 4500.0\n',
  );
  await writeFile(
    join(
      root,
      'proc',
      'loadavg',
    ),
    '10.66 9.00 8.00 1/100 12345\n',
  );
  await mkdir(
    join(
      root,
      'cgroup',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      root,
      'cgroup',
      'cpu.max',
    ),
    'max 100000\n',
  );
  await writeFile(
    join(
      root,
      'cgroup',
      'io.weight',
    ),
    'default 100\n',
  );
  await writeFile(
    join(
      root,
      'cgroup',
      'io.bfq.weight',
    ),
    'default 100\n',
  );
  await writeFile(
    join(
      root,
      'threads',
    ),
    `${threads}\n`,
  );
  return {
    root,
    paths: {
      capFile: join(
        root,
        'threads',
      ),
      cgroupFile: join(
        root,
        'cgroup',
        'cpu.max',
      ),
      ioWeightFile: join(
        root,
        'cgroup',
        'io.weight',
      ),
      ioBfqWeightFile: join(
        root,
        'cgroup',
        'io.bfq.weight',
      ),
      hwmonRoot: join(
        root,
        'hwmon',
      ),
      procRoot: join(
        root,
        'proc',
      ),
      sampleDir: join(
        root,
        'samples',
      ),
    },
    [Symbol.asyncDispose]: async function removeHostFixture(): Promise<void> {
      await directory.remove();
    },
  };
}
