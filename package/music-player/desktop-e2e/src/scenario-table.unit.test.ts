/**
 Consistency of the scenario table: every check id the scenario files declare belongs to a session that the filter
 reaches, ids are unique, and the suite keeps its published size (the checks that pass, and the ones that cannot be
 injected).
 
 The ids are read from the scenario sources, because the checks are declared where they run.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import { dirname, join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { VISUAL_VARIANTS, } from './layout.ts';
import {
  sessionsFor,
  type SessionName,
} from './selection.ts';
import { SEEK_STEPS, } from './wide-seek.ts';

/**
 The directory of the sources.
 */
const SOURCE_DIR = import.meta.dirname;

/**
 The scenario files and the session that runs the checks each declares.
 */
const SCENARIO_FILES: readonly { readonly file: string; readonly session: SessionName; }[] = [
  { file: 'wide-focus.ts', session: 'wide', },
  { file: 'wide-playback.ts', session: 'wide', },
  { file: 'wide-seek.ts', session: 'wide', },
  { file: 'wide-browse.ts', session: 'wide', },
  { file: 'wide-track-list.ts', session: 'wide', },
  { file: 'wide-rows.ts', session: 'wide', },
  { file: 'wide-row-menu.ts', session: 'wide', },
  { file: 'wide-menu.ts', session: 'wide', },
  { file: 'wide-search.ts', session: 'wide', },
  { file: 'wide-double-shift.ts', session: 'wide', },
  { file: 'wide-settings.ts', session: 'wide', },
  { file: 'wide-badges.ts', session: 'wide', },
  { file: 'wide-interaction.ts', session: 'wide', },
  { file: 'wide-select.ts', session: 'wide', },
  { file: 'wide-lists.ts', session: 'wide', },
  { file: 'wide-breakpoint.ts', session: 'wide', },
  { file: 'narrow-interaction.ts', session: 'narrow', },
  { file: 'narrow-session.ts', session: 'narrow', },
  { file: 'dialog-session.ts', session: 'dialog', },
  { file: 'auto-play-session.ts', session: 'auto-play', },
  { file: 'text-size-session.ts', session: 'text-size', },
  { file: 'quit-session.ts', session: 'quit', },
  { file: 'states-main.ts', session: 'states', },
  { file: 'states-small.ts', session: 'states', },
  { file: 'states-refresh.ts', session: 'states', },
];

/**
 A declared id with the session that runs it and whether the declaration is a skip.
 */
type Declared = { readonly id: string; readonly session: SessionName; readonly skipped: boolean; };

/**
 Reads the ids a source file declares as `id: 'x/y'`.
 
 @param file - source file name inside the source directory
 
 @param session - the session the file belongs to
 
 @returns the ids in file order
 */
async function declaredIn(file: string, session: SessionName,): Promise<readonly Declared[]> {
  const text = await readFile(join(SOURCE_DIR, file,), 'utf8',);
  const pieces = text.split("id: '",);
  return pieces.slice(1,).map((piece, index,): Declared => ({
    id: piece.slice(0, piece.indexOf("'",),),
    session,
    skipped: (pieces[index] ?? '').trimEnd().endsWith('recorder.skip({',),
  }),);
}

/**
 Reads every declared id of the suite, with the visual ids built from their variants (the focus ring check and the ring-only check of each).
 
 @returns all declarations
 */
async function allDeclared(): Promise<readonly Declared[]> {
  const fromFiles = (await Promise.all(SCENARIO_FILES.map(({ file, session, },) => declaredIn(file, session,),),)).flat();
  const visual = VISUAL_VARIANTS.map(({ scheme, accent, },): Declared => ({
    id: `visual/${scheme}-${accent}`,
    session: 'visual',
    skipped: false,
  }),);
  const rings = VISUAL_VARIANTS.map(({ scheme, accent, },): Declared => ({
    id: `visual/rings-${scheme}-${accent}`,
    session: 'visual',
    skipped: false,
  }),);
  const menus = VISUAL_VARIANTS.map(({ scheme, accent, },): Declared => ({
    id: `visual/menu-${scheme}-${accent}`,
    session: 'visual',
    skipped: false,
  }),);
  return [...fromFiles, ...visual, ...rings, ...menus,];
}

await describe({
  name: 'scenario table',
  children: [
    it({
      name: 'declares 123 checks that run and 1 that is skipped as not injectable',
      fn: async ({ expect, }) => {
        const declared = await allDeclared();
        expect(declared.filter((entry,) => !entry.skipped,),).toHaveLength(123,);
        expect(declared.filter((entry,) => entry.skipped,).map((entry,) => entry.id,).toSorted(),).toEqual([
          'double-shift/repeat',
        ],);
      },
    },),
    it({
      name: 'declares every id once, as group/name',
      fn: async ({ expect, }) => {
        const ids = (await allDeclared()).map((entry,) => entry.id,);
        expect(new Set(ids,).size,).toBe(ids.length,);
        for (const id of ids)
          expect(id.split('/',),).toHaveLength(2,);
      },
    },),
    it({
      name: 'reaches the session of every id when the id is the filter',
      fn: async ({ expect, }) => {
        for (const { id, session, } of await allDeclared())
          expect(sessionsFor({ only: id, },),).toContain(session,);
      },
    },),
    it({
      name: 'reaches the session of every id through its group prefix',
      fn: async ({ expect, }) => {
        for (const { id, session, } of await allDeclared())
          if (id !== 'browse/ctrl-shift-o')
            expect(sessionsFor({ only: id.split('/',)[0] ?? '', },),).toContain(session,);
      },
    },),
    it({
      name: 'declares the four seek chords of the seek table once each',
      fn: async ({ expect, }) => {
        const ids = (await allDeclared()).map((entry,) => entry.id,);
        for (const step of SEEK_STEPS)
          expect(ids.filter((id,) => id === step.id,),).toHaveLength(1,);
      },
    },),
  ],
},);
