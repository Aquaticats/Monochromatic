/**
 Consistency of the scenario table: every check id the scenario files declare belongs to a session that the filter
 reaches, ids are unique, and the suite keeps its published size (the checks that pass, and the ones that cannot be
 injected).
 
 The ids are read from the scenario sources, because the checks are declared where they run. The scenario sources
 themselves are the source files that declare ids, read from the source directory, so a scenario file added there is
 covered even before it is registered: the registration test below fails with its name until the session attribution
 table names it, and with the name of a table entry that declares no id.
 
 @module
 */

import { readdir, readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
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
 Declares whether a source's text declares a check id: one of its lines, after leading blanks, starts the declaration
 `id: 'group/name'`, the way the scenario sources declare their checks.
 
 @param text - the source file text
 
 @returns true when the text declares at least one check id
 */
function declaresCheckId(text: string,): boolean {
  return text.split('\n',).some((line,) => line.trimStart().startsWith("id: '",));
}

/**
 A scenario source file and the session that runs its checks.
 */
type ScenarioSource = { readonly file: string; readonly session: SessionName; };

/**
 The scenario files and the session that runs the checks each declares.
 
 This hand-written table is the session attribution: a file name does not say which session launches the app for it, so
 that knowledge stays here. The registration test keeps the files it names in sync with the source directory.
 */
const SCENARIO_FILES: readonly ScenarioSource[] = [
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
 The session of every registered scenario file, for finding a file derived from the directory by its name.
 */
const SESSION_OF_FILE: ReadonlyMap<string, SessionName> = new Map(
  SCENARIO_FILES.map(({ file, session, },) => [file, session,] as const,),
);

/**
 Reads the names of the scenario sources in the source directory: every source file whose text declares a check id.
 
 A test file declares fixture ids rather than checks, so it is not a scenario source.
 
 @returns the file names, sorted
 */
async function declaredScenarioFiles(): Promise<readonly string[]> {
  const names = await readdir(SOURCE_DIR,);
  const sources = names.filter((name,) => (name.endsWith('.ts',)) && (!name.endsWith('.test.ts',)),);
  const declarations = await Promise.all(sources.map(async (name,) => {
    const text = await readFile(join(SOURCE_DIR, name,), 'utf8',);
    return declaresCheckId(text,) ? name : undefined;
  },),);
  return declarations.filter((name,): name is string => name !== undefined,).toSorted();
}

/**
 The scenario sources of the suite: the file set from the source directory, the session from the attribution table.
 
 The directory is the authority on which scenario sources exist, so a declaring file added without a table entry fails
 here with its name instead of escaping every id-coverage check below.
 
 @returns the files and their sessions, sorted by file name
 
 @throws Error listing every declaring file the table does not register, and every table entry that declares no id
 */
async function scenarioSources(): Promise<readonly ScenarioSource[]> {
  const declaring = await declaredScenarioFiles();
  const unregistered = declaring.filter((file,) => !SESSION_OF_FILE.has(file,),);
  const silent = SCENARIO_FILES.map(({ file, },) => file,).filter((file,) => !declaring.includes(file,));
  if ((unregistered.length > 0) || (silent.length > 0))
    throw new Error(
      `the scenario table is out of date: declaring but not registered: ${unregistered.join(', ',) || '(none)'}; registered but declaring no check id: ${silent.join(', ',) || '(none)'}`,
    );
  return declaring.map((file,) => ({ file, session: nonNullishOrThrow(SESSION_OF_FILE.get(file,),), }),);
}

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
  const sources = await scenarioSources();
  const fromFiles = (await Promise.all(sources.map(({ file, session, },) => declaredIn(file, session,),),)).flat();
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
      name: 'registers every scenario source that declares a check id, and only those',
      fn: async ({ expect, }) => {
        const declaring = await declaredScenarioFiles();
        const registered = SCENARIO_FILES.map(({ file, },) => file,).toSorted();
        expect(declaring.filter((file,) => !registered.includes(file,),),).toEqual([],);
        expect(registered.filter((file,) => !declaring.includes(file,),),).toEqual([],);
      },
    },),
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
