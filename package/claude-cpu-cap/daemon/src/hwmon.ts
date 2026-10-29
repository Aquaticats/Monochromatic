import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { ABSENT, } from './constants.ts';
import type { Sensors, } from './types.ts';

/**
 One hwmon class entry: its directory and the driver name from its `name` file.
 */
export type HwmonEntry = Readonly<{
  /**
   Directory such as `/sys/class/hwmon/hwmon4`.
   */
  dir: string;
  /**
   Driver name such as `k10temp`.
   */
  name: string;
}>;

/**
 Label file prefix of an energy counter (`energy1_label`).
 */
const ENERGY_LABEL_PREFIX = 'energy';

/**
 Suffix shared by all label files.
 */
const LABEL_SUFFIX = '_label';

/**
 Label of the socket energy counter.
 */
const SOCKET_LABEL = 'Esocket0';

/**
 Label prefix of the per-core energy counters (`Ecore000` to `Ecore015`).
 */
const CORE_LABEL_PREFIX = 'Ecore';

/**
 Lists the hwmon entries with their driver names.

 @param hwmonRoot - hwmon class directory

 @returns entries in directory order

 @example
 ```ts
 await listHwmon('/sys/class/hwmon',); // [{ dir: '/sys/class/hwmon/hwmon0', name: 'nvme', }, ...]
 ```
 */
export async function listHwmon(hwmonRoot: string,): Promise<readonly HwmonEntry[]> {
  /**
   Entry names in the class directory.
   */
  const names = await readdir(hwmonRoot,);
  return Promise.all(names.map(async function toEntry(entry,): Promise<HwmonEntry> {
    /**
     Directory of this entry.
     */
    const dir = join(
      hwmonRoot,
      entry,
    );
    return {
      dir,
      name: (await readFile(
        join(
          dir,
          'name',
        ),
        'utf8',
      )).trim(),
    };
  },),);
}

/**
 Finds the first hwmon directory of a driver.

 @param entries - entries from {@link listHwmon}
 
 @param driver - driver name

 @returns the directory, or {@link ABSENT} when the driver is not loaded

 @example
 ```ts
 findDriverDir({ entries, driver: 'k10temp', },); // '/sys/class/hwmon/hwmon4'
 ```
 */
export function findDriverDir({
  entries,
  driver,
}: Readonly<{
  entries: readonly HwmonEntry[];
  driver: string;
}>,): string {
  return entries.find(function hasDriver(entry,) {
    return entry.name === driver;
  },)
    ?.dir
    ?? ABSENT;
}

/**
 Reads the `energyN_label` files of one `zenergy` directory.

 @param dir - a `zenergy` hwmon directory

 @returns pairs of label (`Esocket0`, `Ecore000`, ...) and the matching `energyN_input` path
 */
async function labelsOf(dir: string,): Promise<readonly (readonly [
  string,
  string
])[]> {
  /**
   Names of the label files in the directory.
   */
  const labelFiles = (await readdir(dir,)).filter(function isLabel(file,) {
    return file.startsWith(ENERGY_LABEL_PREFIX,) && file.endsWith(LABEL_SUFFIX,);
  },);
  return Promise.all(labelFiles.map(async function pairOf(file,): Promise<readonly [
    string,
    string
  ]> {
    return [
      (await readFile(
        join(
          dir,
          file,
        ),
        'utf8',
      )).trim(),
      join(
        dir,
        `${file.slice(
          0,
          -LABEL_SUFFIX.length,
        )}_input`,
      ),
    ];
  },),);
}

/**
 Reads the energy counter labels of every `zenergy` directory.

 @param entries - entries from {@link listHwmon}

 @returns a map from label to its `energyN_input` file

 @example
 ```ts
 (await readEnergyLabels(entries,)).get('Esocket0',); // '/sys/class/hwmon/hwmon6/energy1_input'
 ```
 */
export async function readEnergyLabels(entries: readonly HwmonEntry[],): Promise<ReadonlyMap<string, string>> {
  /**
   Label pairs per `zenergy` directory.
   */
  const perDir = await Promise.all(entries.filter(function isZenergy(entry,) {
    return entry.name === 'zenergy';
  },)
    .map(function readDir(entry,) {
      return labelsOf(entry.dir,);
    },),);
  return new Map(perDir.flat(),);
}

/**
 Picks the per-core counter files out of the label map, indexed by core number.

 @param labels - map from {@link readEnergyLabels}

 @returns files for cores 0 to N, or an empty array when no core counter exists

 @example
 ```ts
 coreEnergyFilesFrom(new Map([['Ecore001', '/x/energy3_input',],],),); // [<empty>, '/x/energy3_input']
 ```
 */
export function coreEnergyFilesFrom(labels: ReadonlyMap<string, string>,): readonly string[] {
  /**
   Counter files by core number.
   */
  const byCore = new Map(
    labels.entries()
      .filter(function isCore([label,],) {
        return label.startsWith(CORE_LABEL_PREFIX,);
      },)
      .map(function coreEntry([label, file,],): readonly [
        number,
        string
      ] {
        return [
          Number(label.slice(CORE_LABEL_PREFIX.length,),),
          file,
        ];
      },),
  );
  return Array.from(
    { length: byCore.size, },
    function fileOfCore(
      _unused,
      core,
    ): string {
      return byCore.get(core,) ?? ABSENT;
    },
  );
}

/**
 Finds the discrete GPU directory (the `amdgpu` entry with a fan input) and the integrated one (without).

 @param entries - entries from {@link listHwmon}

 @returns the two directories, each {@link ABSENT} when missing

 @example
 ```ts
 await findGpuDirs(entries,); // { gpuDir: '/sys/class/hwmon/hwmon2', igpuDir: '/sys/class/hwmon/hwmon3', }
 ```
 */
export async function findGpuDirs(entries: readonly HwmonEntry[],): Promise<Readonly<{
  gpuDir: string;
  igpuDir: string;
}>> {
  /**
   The `amdgpu` entries.
   */
  const amdgpu = entries.filter(function isAmdgpu(entry,) {
    return entry.name === 'amdgpu';
  },);
  /**
   Whether each `amdgpu` entry has a fan input, in the same order.
   */
  const hasFan = await Promise.all(amdgpu.map(async function fanExists(entry,): Promise<boolean> {
    return (await readdir(entry.dir,)).includes('fan1_input',);
  },),);
  return {
    gpuDir: amdgpu.find(function discrete(
      _entry,
      index,
    ) {
      return hasFan[index] === true;
    },)
      ?.dir
      ?? ABSENT,
    igpuDir: amdgpu.find(function integrated(
      _entry,
      index,
    ) {
      return hasFan[index] === false;
    },)
      ?.dir
      ?? ABSENT,
  };
}

/**
 Discovers every sensor the watcher reads.

 @param hwmonRoot - hwmon class directory

 @returns sensor locations, with {@link ABSENT} for the ones this host lacks

 @example
 ```ts
 (await discoverSensors('/sys/class/hwmon',)).temperatureFile; // '/sys/class/hwmon/hwmon4/temp1_input'
 ```
 */
export async function discoverSensors(hwmonRoot: string,): Promise<Sensors> {
  /**
   Every hwmon entry.
   */
  const entries = await listHwmon(hwmonRoot,);
  /**
   Energy counter files by label.
   */
  const labels = await readEnergyLabels(entries,);
  /**
   Directory of the CPU temperature driver.
   */
  const k10temp = findDriverDir({
    entries,
    driver: 'k10temp',
  },);
  return {
    temperatureFile: k10temp === ABSENT ? ABSENT : join(
      k10temp,
      'temp1_input',
    ),
    socketEnergyFile: labels.get(SOCKET_LABEL,) ?? ABSENT,
    coreEnergyFiles: coreEnergyFilesFrom(labels,),
    boardDir: findDriverDir({
      entries,
      driver: 'gigabyte_wmi',
    },),
    superIoDir: findDriverDir({
      entries,
      driver: 'nct6687',
    },),
    acpiDir: findDriverDir({
      entries,
      driver: 'acpitz',
    },),
    nvmeDir: findDriverDir({
      entries,
      driver: 'nvme',
    },),
    ...await findGpuDirs(entries,),
  };
}
