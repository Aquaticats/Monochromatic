import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { SAMPLE_HEADER, } from './constants.ts';
import { discoverSensors, } from './hwmon.ts';
import { samplePower, NO_READING, } from './power.ts';
import {
  buildSampleFields,
  ensureHeader,
  fixedField,
  pruneSamples,
  writeSample,
} from './samples.ts';
import { makeHost, } from './test-fixture.ts';

/**
 Number of columns in the sample log header.
 */
const COLUMN_COUNT = SAMPLE_HEADER.trimEnd().split(',',).length;

await describe({
  name: 'samples',
  children: [
    it({
      name: 'fixedField formats numbers and leaves unmeasured values empty',
      fn: async function formatsFields() {
        expect(fixedField({ value: 74.84, digits: 1, },),).toBe('74.8',);
        expect(fixedField({ value: Number.NaN, digits: 1, },),).toBe('',);
      },
    },),
    it({
      name: 'the header has 42 columns, the 31 older ones first so older rows keep their positions',
      fn: async function headerWidth() {
        expect(COLUMN_COUNT,).toBe(42,);
        expect(SAMPLE_HEADER.split(',',).slice(0, 31,).at(-1,),).toBe('igpu_w',);
        expect(SAMPLE_HEADER.split(',',).slice(31,).at(0,),).toBe('cpu_fan_rpm',);
      },
    },),
    it({
      name: 'buildSampleFields yields one field per header column, in order, from the fixture sensors',
      fn: async function buildsRow() {
        /**
         Disposable host.
         */
        const { paths, } = await makeHost({ threads: '28', cpuPressure: 1.5, },);
        /**
         Sensors found in the fixture.
         */
        const sensors = await discoverSensors(paths.hwmonRoot,);
        /**
         Power sample from the fixture counters.
         */
        const power = await samplePower({ sensors, procRoot: paths.procRoot, previous: NO_READING, at: 0, },);
        /**
         The row's fields.
         */
        const fields = await buildSampleFields({
          time: '2026-10-13T00:00:00.000Z',
          cap: 28,
          power,
          burst: true,
          cpuPressure: 1.5,
          load1: '10.66',
          sensors,
        },);
        expect(fields,).toHaveLength(COLUMN_COUNT,);
        expect(fields.slice(0, 9,),).toEqual(['2026-10-13T00:00:00.000Z', '61.5', '58.0', '57.0', '', '4450', '1.5', '10.66', '28',],);
        expect(fields.slice(9, 16,),).toEqual(['', '', '', '', '', '', '',],);
        expect(fields.slice(18, 24,),).toEqual(['30.0', '31.0', '32.0', '33.0', '34.0', '35.0',],);
        expect(fields.slice(24, 31,),).toEqual(['16.8', '40.0', '1', '45.0', '12.0', '50.0', '3.0',],);
        expect(fields.slice(31, 39,),).toEqual(['1500', '0', '1540', '0', '2050', '0', '2020', '2590',],);
        expect(fields.slice(39,),).toEqual(['35.0', '40.5', '35.5',],);
      },
    },),
    describe({
      name: ensureHeader.name,
      children: [
        it({
          name: 'writes the header into a new file',
          fn: async function newFile() {
            /**
             Disposable host.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            await mkdir(paths.sampleDir, { recursive: true, },);
            /**
             Path of the new file.
             */
            const file = join(paths.sampleDir, 'new.csv',);
            await ensureHeader(file,);
            expect(await readFile(file, 'utf8',),).toBe(SAMPLE_HEADER,);
          },
        },),
        it({
          name: 'replaces an older header and keeps the rows',
          fn: async function oldHeader() {
            /**
             Disposable host.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            await mkdir(paths.sampleDir, { recursive: true, },);
            /**
             Path of the file with an older header.
             */
            const file = join(paths.sampleDir, 'old.csv',);
            await writeFile(file, 'utc_time,tctl_c\nrow-one\nrow-two\n',);
            await ensureHeader(file,);
            expect(await readFile(file, 'utf8',),).toBe(`${SAMPLE_HEADER}row-one\nrow-two\n`,);
          },
        },),
        it({
          name: 'leaves a file with the current header untouched',
          fn: async function currentHeader() {
            /**
             Disposable host.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            await mkdir(paths.sampleDir, { recursive: true, },);
            /**
             Path of the file with the current header and one row.
             */
            const file = join(paths.sampleDir, 'current.csv',);
            await writeFile(file, `${SAMPLE_HEADER}row-one\n`,);
            await ensureHeader(file,);
            expect(await readFile(file, 'utf8',),).toBe(`${SAMPLE_HEADER}row-one\n`,);
          },
        },),
      ],
    },),
    describe({
      name: pruneSamples.name,
      children: [
        it({
          name: 'removes csv files older than 14 days and keeps newer ones and other files',
          fn: async function prunesOld() {
            /**
             Disposable host.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            await mkdir(paths.sampleDir, { recursive: true, },);
            await Promise.all(['2026-09-01.csv', '2026-10-12.csv', 'notes.txt',].map(async function createFile(name,) {
              await writeFile(join(paths.sampleDir, name,), 'x',);
            },),);
            await pruneSamples({ sampleDir: paths.sampleDir, now: Date.parse('2026-10-13T12:00:00',), },);
            expect((await readdir(paths.sampleDir,)).toSorted(),).toEqual(['2026-10-12.csv', 'notes.txt',],);
          },
        },),
      ],
    },),
    describe({
      name: writeSample.name,
      children: [
        it({
          name: "appends a 42-field row to today's file, creating the directory and header, and returns today's date",
          fn: async function appendsRow() {
            /**
             Disposable host.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 1.5, },);
            /**
             Sensors found in the fixture.
             */
            const sensors = await discoverSensors(paths.hwmonRoot,);
            /**
             Power sample from the fixture counters.
             */
            const power = await samplePower({ sensors, procRoot: paths.procRoot, previous: NO_READING, at: 0, },);
            /**
             Local date returned by the write.
             */
            const day = await writeSample({
              paths,
              sensors,
              power,
              cap: 28,
              burst: false,
              sampleDay: '',
            },);
            /**
             Lines of the written file.
             */
            const lines = (await readFile(join(paths.sampleDir, `${day}.csv`,), 'utf8',)).trimEnd().split('\n',);
            expect(lines[0],).toBe(SAMPLE_HEADER.trimEnd(),);
            expect(lines,).toHaveLength(2,);
            expect(lines[1]?.split(',',),).toHaveLength(COLUMN_COUNT,);
          },
        },),
      ],
    },),
  ],
},);
