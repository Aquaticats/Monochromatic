import { rm, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ABSENT, } from './constants.ts';
import {
  coreEnergyFilesFrom,
  discoverSensors,
  findDriverDir,
  listHwmon,
} from './hwmon.ts';
import { makeHost, } from './test-fixture.ts';

await describe({
  name: 'hwmon',
  children: [
    it({
      name: 'lists every entry with its driver name',
      fn: async function listsEntries() {
        /**
         Disposable host with eight hwmon entries.
         */
        const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
        /**
         Entries read from the fixture.
         */
        const entries = await listHwmon(paths.hwmonRoot,);
        expect(entries.map(function nameOf(entry,) {
          return entry.name;
        },).toSorted(),).toEqual(['acpitz', 'amdgpu', 'amdgpu', 'gigabyte_wmi', 'k10temp', 'nct6687', 'nvme', 'zenergy',],);
        expect(findDriverDir({ entries, driver: 'k10temp', },),).toContain('hwmon0',);
        expect(findDriverDir({ entries, driver: 'missing', },),).toBe(ABSENT,);
      },
    },),
    it({
      name: 'discovers the sensors, telling the discrete GPU (has a fan) from the integrated one',
      fn: async function discoversSensors() {
        /**
         Disposable host.
         */
        const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
        /**
         Sensors found in the fixture.
         */
        const sensors = await discoverSensors(paths.hwmonRoot,);
        expect(sensors.temperatureFile,).toMatch('hwmon0/temp1_input',);
        expect(sensors.socketEnergyFile,).toMatch('hwmon1/energy1_input',);
        expect(sensors.coreEnergyFiles,).toHaveLength(16,);
        expect(sensors.coreEnergyFiles[0],).toMatch('hwmon1/energy2_input',);
        expect(sensors.coreEnergyFiles[15],).toMatch('hwmon1/energy17_input',);
        expect(sensors.gpuDir,).toMatch('hwmon5',);
        expect(sensors.igpuDir,).toMatch('hwmon6',);
        expect(sensors.boardDir,).toMatch('hwmon2',);
        expect(sensors.superIoDir,).toMatch('hwmon7',);
      },
    },),
    it({
      name: 'answers ABSENT for drivers the host lacks',
      fn: async function absentDrivers() {
        /**
         Disposable host.
         */
        const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
        await Promise.all(['hwmon0', 'hwmon1', 'hwmon5', 'hwmon6', 'hwmon7',].map(async function removeEntry(entry,) {
          await rm(join(paths.hwmonRoot, entry,), { recursive: true, },);
        },),);
        /**
         Sensors found after the CPU, energy, GPU and fan drivers were removed.
         */
        const sensors = await discoverSensors(paths.hwmonRoot,);
        expect(sensors.temperatureFile,).toBe(ABSENT,);
        expect(sensors.socketEnergyFile,).toBe(ABSENT,);
        expect(sensors.coreEnergyFiles,).toEqual([],);
        expect(sensors.gpuDir,).toBe(ABSENT,);
        expect(sensors.igpuDir,).toBe(ABSENT,);
        expect(sensors.superIoDir,).toBe(ABSENT,);
        expect(sensors.boardDir,).not.toBe(ABSENT,);
      },
    },),
    it({
      name: 'orders core files by core number and ignores non-core labels',
      fn: async function ordersCores() {
        expect(
          coreEnergyFilesFrom(new Map([['Ecore001', '/b',], ['Esocket0', '/s',], ['Ecore000', '/a',],],),),
        ).toEqual(['/a', '/b',],);
      },
    },),
  ],
},);
