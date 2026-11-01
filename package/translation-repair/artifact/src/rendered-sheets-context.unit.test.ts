/**
 Guards ledger B28: a sheet whose text names the DECLARED NAMES block carries
 the page's declared names, and one naming CITED REFERENCES carries the cited
 pages, as the rendered-sheets fixtures render every sheet with the context
 production gives it. Ledger B805 adds the REPAIR PANEL CLAIMS ACCEPTED and ARCHIVE RENDERING
 DISPUTED blocks: the polish gate, the polish writers and the judges of the
 writers' rewrites are all shown the claims the repair panel accepted and the
 note on a disputed archive rendering, and a sheet that names a block carries
 it.

 WHY THIS EXISTS. The house rules every writer and judge carries tell it to
 read a pronoun line and footnote vocabulary in a DECLARED NAMES block. The
 archive block review, coverage, the page title lexicon and the refine slates
 carried those rules and never got the block, and the fixture rendered them
 without it too, so no guard reading the rendered sheets could tell. A new
 sheet carrying the house rules now fails here until its caller threads the
 names and the fixture renders them, or it joins the exemptions with a reason.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ARCHIVE_DISPUTE,
  renderedSheets,
  REPAIR_CLAIMS,
} from './rendered-sheets.test-fixture.ts';
import {
  IDENTITY,
  REFERENCES,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

/**
 Heading every sheet's rules call the declared names by.
 */
const DECLARED_NAMES = 'DECLARED NAMES';

/**
 Heading every sheet's rules call the cited pages by.
 */
const CITED_REFERENCES = 'CITED REFERENCES';

/**
 Heading the block of accepted repair claims carries.
 */
const REPAIR_PANEL_CLAIMS = 'REPAIR PANEL CLAIMS ACCEPTED';

/**
 Heading the note on a disputed archive rendering carries.
 */
const ARCHIVE_RENDERING_DISPUTED = 'ARCHIVE RENDERING DISPUTED';

/**
 Sheets that name the DECLARED NAMES block and rightly carry none, each with
 why.
 */
const WITHOUT_DECLARED_NAMES: Readonly<Record<string, string>> = {
  'house policy': 'the house rules themselves, rendered alone so a guard can read them; every sheet carrying them '
    + 'is checked here',
  'restoration judge': 'a measurement probe over seeded wordings, which asks about planted restorations rather '
    + 'than a page\'s own names, and whose builder takes no declared names',
};

/**
 Sheets that name CITED REFERENCES and rightly carry none, each with why.
 */
const WITHOUT_CITED_REFERENCES: Readonly<Record<string, string>> = {};

/**
 Sheets that name the REPAIR PANEL CLAIMS ACCEPTED block and rightly carry none, each with why.
 */
const WITHOUT_REPAIR_CLAIMS: Readonly<Record<string, string>> = {};

/**
 Sheets that name the ARCHIVE RENDERING DISPUTED block and rightly carry none, each with why.
 */
const WITHOUT_ARCHIVE_DISPUTE: Readonly<Record<string, string>> = {};

/**
 Sheets that must carry the accepted repair claims and the note on a disputed archive rendering, because production
 gives them to the stage they are written for (ledger B731 for the gate, B805 for the writers and for the judges of
 their rewrites). Named here so the scan cannot pass over a sheet that lost the block and with it the only mention of
 the heading.
 */
const MUST_CARRY_POLISH_NOTES: readonly string[] = [
  'polish gate',
  'polish refiner',
  'polish refine slate',
  'polish refine objection slate',
];

/**
 Whether a sheet carries a block's content as the model reads it: written out,
 or inside a JSON string, as the typed decision's state carries its evidence.

 @param text - rendered sheet

 @param content - block content

 @returns Whether either form is in the sheet

 @example
 ```ts
 carries({ text: sheet.text, content: REFERENCES, },);
 ```
 */
function carries({ text, content, }: { readonly text: string; readonly content: string; },): boolean {
  return text.includes(content,) || text.includes(JSON.stringify(content,).slice(1, -1,),);
}

/**
 Names of the rendered sheets that name a block and do not carry it, the
 exempt ones aside.

 @param heading - block heading the sheet's text names

 @param content - block content the fixture hands every sheet that gets it

 @param exempt - sheets that rightly carry none

 @returns Sheet names, in fixture order

 @example
 ```ts
 const missing = namingWithout({ heading: DECLARED_NAMES, content: IDENTITY, exempt: WITHOUT_DECLARED_NAMES, },);
 ```
 */
function namingWithout(
  {
    heading,
    content,
    exempt,
  }: {
    readonly heading: string;
    readonly content: string;
    readonly exempt: Readonly<Record<string, string>>;
  },
): readonly string[] {
  return renderedSheets()
    .filter(function namesButLacks(sheet,): boolean {
      return (sheet.text.includes(heading,))
        && (!carries({ text: sheet.text, content, },))
        && (!Object.hasOwn(exempt, sheet.name,));
    },)
    .map(function nameOf(sheet,): string {
      return sheet.name;
    },);
}

await describe({
  name: 'rendered sheets carry the blocks their rules name (ledger B28)',
  children: [
    it({
      name: 'CARRIES THE DECLARED NAMES on every sheet whose text names the block, the exempt ones aside',
      fn: async () => {
        /**
         Sheets that name the block at all, so an empty result means something.
         */
        const naming = renderedSheets().filter(function names(sheet,): boolean {
          return sheet.text.includes(DECLARED_NAMES,);
        },);
        expect(naming.length,).toBeGreaterThan(Object.keys(WITHOUT_DECLARED_NAMES,).length,);
        expect(namingWithout({ heading: DECLARED_NAMES, content: IDENTITY, exempt: WITHOUT_DECLARED_NAMES, },),)
          .toEqual([],);
      },
    },),
    it({
      name: 'CARRIES THE CITED REFERENCES on every sheet whose text names them, the exempt ones aside',
      fn: async () => {
        /**
         Sheets that name the references at all, so an empty result means something.
         */
        const naming = renderedSheets().filter(function names(sheet,): boolean {
          return sheet.text.includes(CITED_REFERENCES,);
        },);
        expect(naming.length,).toBeGreaterThan(Object.keys(WITHOUT_CITED_REFERENCES,).length,);
        expect(namingWithout({ heading: CITED_REFERENCES, content: REFERENCES, exempt: WITHOUT_CITED_REFERENCES, },),)
          .toEqual([],);
      },
    },),
    it({
      name: 'CARRIES THE ACCEPTED REPAIR CLAIMS on every sheet whose text names them, the exempt ones aside, and on the '
        + 'polish gate, the polish writer and the polish writers\' rewrite judges whatever their text names (ledger B805)',
      fn: async () => {
        /**
         Sheets that name the block at all, so an empty result means something.
         */
        const naming = renderedSheets().filter(function names(sheet,): boolean {
          return sheet.text.includes(REPAIR_PANEL_CLAIMS,);
        },);
        expect(naming.length,).toBeGreaterThan(Object.keys(WITHOUT_REPAIR_CLAIMS,).length,);
        expect(namingWithout({ heading: REPAIR_PANEL_CLAIMS, content: REPAIR_CLAIMS, exempt: WITHOUT_REPAIR_CLAIMS, },),)
          .toEqual([],);
        /**
         Sheets production gives the claims that do not show them.
         */
        const lacking = MUST_CARRY_POLISH_NOTES.filter(function lacksClaims(name,): boolean {
          return !renderedSheets().some(function showsThem(sheet,): boolean {
            return (sheet.name === name) && carries({ text: sheet.text, content: REPAIR_CLAIMS, },);
          },);
        },);
        expect(lacking,).toEqual([],);
      },
    },),
    it({
      name: 'CARRIES THE NOTE ON A DISPUTED ARCHIVE RENDERING on every sheet whose text names it, the exempt ones aside, '
        + 'and on the polish gate, the polish writer and the polish writers\' rewrite judges whatever their text names '
        + '(ledger B805)',
      fn: async () => {
        /**
         Sheets that name the block at all, so an empty result means something.
         */
        const naming = renderedSheets().filter(function names(sheet,): boolean {
          return sheet.text.includes(ARCHIVE_RENDERING_DISPUTED,);
        },);
        expect(naming.length,).toBeGreaterThan(Object.keys(WITHOUT_ARCHIVE_DISPUTE,).length,);
        expect(
          namingWithout({ heading: ARCHIVE_RENDERING_DISPUTED, content: ARCHIVE_DISPUTE, exempt: WITHOUT_ARCHIVE_DISPUTE, },),
        ).toEqual([],);
        /**
         Sheets production gives the note that do not show it.
         */
        const lacking = MUST_CARRY_POLISH_NOTES.filter(function lacksNote(name,): boolean {
          return !renderedSheets().some(function showsIt(sheet,): boolean {
            return (sheet.name === name) && carries({ text: sheet.text, content: ARCHIVE_DISPUTE, },);
          },);
        },);
        expect(lacking,).toEqual([],);
      },
    },),
    it({
      name: 'EXEMPTS ONLY SHEETS THAT EXIST AND NAME THE BLOCK, so a renamed or reworded sheet does not keep an '
        + 'exemption it no longer needs',
      fn: async () => {
        /**
         Every rendered sheet, by name.
         */
        const byName = new Map(renderedSheets().map(function entry(sheet,): readonly [string, string] {
          return [sheet.name, sheet.text,];
        },),);
        /**
         Exemptions naming no rendered sheet, or a sheet that does not name the block.
         */
        const stale = [
          ...Object.keys(WITHOUT_DECLARED_NAMES,).filter(function staleNames(name,): boolean {
            return !(byName.get(name,) ?? '').includes(DECLARED_NAMES,);
          },),
          ...Object.keys(WITHOUT_CITED_REFERENCES,).filter(function staleReferences(name,): boolean {
            return !(byName.get(name,) ?? '').includes(CITED_REFERENCES,);
          },),
          ...Object.keys(WITHOUT_REPAIR_CLAIMS,).filter(function staleClaims(name,): boolean {
            return !(byName.get(name,) ?? '').includes(REPAIR_PANEL_CLAIMS,);
          },),
          ...Object.keys(WITHOUT_ARCHIVE_DISPUTE,).filter(function staleDispute(name,): boolean {
            return !(byName.get(name,) ?? '').includes(ARCHIVE_RENDERING_DISPUTED,);
          },),
        ];
        expect(stale,).toEqual([],);
      },
    },),
  ],
},);
