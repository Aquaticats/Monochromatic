import type { FileOperations, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  computeFileLists,
  formatFileOperations,
} from './file-tracking.ts';

await describe({
  name: '',
  children: [
    describe({
      name: computeFileLists.name,
      children: [
        it({
          name: 'separates read-only from modified files',
          fn: async () => {
            const fileOps: FileOperations = {
              read: new Set([
                'a.ts',
                'b.ts',
                'c.ts',
              ],),
              written: new Set(['d.ts',],),
              edited: new Set([
                'b.ts',
                'e.ts',
              ],),
            };
            const {
              readFiles,
              modifiedFiles,
            } = computeFileLists(fileOps,);
            // b.ts is in both read and edited -> modified, not read
            expect(readFiles.includes('b.ts',),).toBe(false,);
            expect(modifiedFiles.includes('b.ts',),).toBe(true,);
            expect(readFiles.includes('a.ts',),).toBe(true,);
            expect(readFiles.includes('c.ts',),).toBe(true,);
            expect(modifiedFiles.includes('d.ts',),).toBe(true,);
            expect(modifiedFiles.includes('e.ts',),).toBe(true,);
          },
        },),
        it({
          name: 'returns empty lists for empty file ops',
          fn: async () => {
            const fileOps: FileOperations = {
              read: new Set(),
              written: new Set(),
              edited: new Set(),
            };
            const {
              readFiles,
              modifiedFiles,
            } = computeFileLists(fileOps,);
            expect(readFiles,).toHaveLength(0,);
            expect(modifiedFiles,).toHaveLength(0,);
          },
        },),
        it({
          name: 'sorts file paths',
          fn: async () => {
            const fileOps: FileOperations = {
              read: new Set([
                'z.ts',
                'a.ts',
                'm.ts',
              ],),
              written: new Set(),
              edited: new Set(),
            };
            const { readFiles, } = computeFileLists(fileOps,);
            expect(readFiles,).toEqual([
              'a.ts',
              'm.ts',
              'z.ts',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: formatFileOperations.name,
      children: [
        it({
          name: 'produces XML with both sections',
          fn: async () => {
            const xml = formatFileOperations({
              readFiles: [
                'a.ts',
                'b.ts',
              ],
              modifiedFiles: ['c.ts',],
            },);
            expect(xml,).toContain('<read-files>',);
            expect(xml,).toContain('</read-files>',);
            expect(xml,).toContain('<modified-files>',);
            expect(xml,).toContain('</modified-files>',);
            expect(xml,).toContain('a.ts',);
            expect(xml,).toContain('c.ts',);
          },
        },),
        it({
          name: 'omits read-files when only modified files exist',
          fn: async () => {
            const xml = formatFileOperations({
              readFiles: [],
              modifiedFiles: ['changed.ts',],
            },);
            expect(xml,).not.toContain('<read-files>',);
            expect(xml,).toContain('<modified-files>',);
          },
        },),
        it({
          name: 'returns empty string for no files',
          fn: async () => {
            const xml = formatFileOperations({
              readFiles: [],
              modifiedFiles: [],
            },);
            expect(xml,).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
