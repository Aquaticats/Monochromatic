import { splitFrontMatter, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { FrontMatterAuthority, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  directoryIdNameStands,
  namesDirectoryId,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Archive front matter authority
// WHOSE FRONT MATTER THE PAGE CARRIES, decided by the owner on 2026-09-08
// after the ninth hakureico pass renamed the person in slice zero (`name:
// Kagurazaka Chika`, six ballots to three, endorsed nine of nine) while the
// body kept the archive's `Hanasaka`, and a census of the shipped pages found
// nine of the last ten read pages rewriting `desc` or `alias` ("We're not
// supposed to change front matter though?"). The rule: the archive's front
// matter is published as it stands, and the lanes render it only where the
// archive never translated it, which is the directory-id shape the publication guard
// already names: the visible name is the directory id while the source names
// the person, and neither the pinyin reading nor an alias makes the id stand.
//
// DECIDED BEFORE PREPARATION AND CHECKED AGAIN AT PUBLICATION. The
// preparation omits slice zero when the archive stands, so no lane spends on
// it and no judge is asked to prefer a rendering over the archive's editorial
// choice; the publication guard recomputes the same answer from the two
// documents and refuses a page whose front matter differs from the archive's
// where it stands, so a preparation that forgot the rule cannot ship.

/**
 Whether the archive's front matter stands as published, so the lanes leave
 it alone.

 @param entryId - directory id of the entry

 @param sourceText - whole original page

 @param archiveText - whole archive page before any lane ran

 @returns Whether the page carries the archive's front matter byte for byte

 @example
 ```ts
 const stands = archiveFrontMatterStands({ entryId: 'hakureico', sourceText, archiveText, },);
 ```
 */
export function archiveFrontMatterStands(
  {
    entryId,
    sourceText,
    archiveText,
  }: {
    readonly entryId: string;
    readonly sourceText: string;
    readonly archiveText: string;
  },
): boolean {
  /**
   Archive metadata, when the archive declares any.
   */
  const { frontMatter: archive, } = splitFrontMatter({ text: archiveText, },);
  // NOTHING OF THE ARCHIVE'S TO KEEP: a source-only front matter is inserted
  // by the lanes as it always was.
  if (archive === undefined)
    return false;
  /**
   Source metadata, when the original declares any.
   */
  const { frontMatter: source, } = splitFrontMatter({ text: sourceText, },);
  // The publication guard already requires the page to carry the archive's
  // bytes where the source declares none.
  if (source === undefined)
    return true;
  // A NAME OF ITS OWN: the archive rendered the person's name, so its
  // metadata was translated and is the memorial's editorial choice.
  if (!namesDirectoryId({
    metadata: archive,
    entryId,
  },))
    return true;
  // THE HANDLE IS THE NAME: the source names the person by the directory id
  // too, so the archive's id-equal name is a rendering of it.
  if (namesDirectoryId({
    metadata: source,
    entryId,
  },))
    return true;
  // THE ID STANDS AS A RENDERING (the owner's decision of 2026-09-07): the
  // pinyin of the source's name, the source's own alias, or a Latin-script
  // alias beside it. What is left is the directory-id shape, which the lanes render.
  return directoryIdNameStands({
    entryId,
    source,
    page: archive,
    archives: [archive,],
  },);
}

/**
 Authority the page's front matter has for one entry.

 @param entryId - directory id of the entry

 @param sourceText - whole original page

 @param archiveText - whole archive page before any lane ran

 @returns `archive` where the archive's front matter stands, `rendered`
 where the lanes must render it

 @example
 ```ts
 const authority = frontMatterAuthorityOf({ entryId: 'hakureico', sourceText, archiveText, },);
 ```
 */
export function frontMatterAuthorityOf(
  {
    entryId,
    sourceText,
    archiveText,
  }: {
    readonly entryId: string;
    readonly sourceText: string;
    readonly archiveText: string;
  },
): FrontMatterAuthority {
  return archiveFrontMatterStands({
    entryId,
    sourceText,
    archiveText,
  },)
    ? 'archive'
    : 'rendered';
}

//endregion Archive front matter authority
