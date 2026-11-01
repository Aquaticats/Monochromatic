import {
  extractDeclaredIdentity,
  formsOf,
  isDeclared,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Subject names
// THE NAMES THE TENSE DETECTOR READS AS THE PAGE'S SUBJECT: the `name` the
// translation declares and every alias it declares in `info.alias`. A memorial
// page names its subject by a handle as often as by the display name, and a
// sentence that opens on the handle is as much a narration of the subject as
// one that opens on the name.
//
// AN ALIAS IS A STRING OR A LIST. The corpus joins several aliases into one
// comma-joined string (`extractDeclaredIdentity` reads that form), and a page
// written by hand may give a YAML list instead; the identity block keeps its
// existing reading of the string form, and this module reads both.

/**
 Names a translation's front matter declares for its subject: the `name` and
 every alias in `info.alias`, an alias given as a string split at the commas the
 corpus joins aliases with and an alias given as a list read item by item.

 @param data - parsed front matter of the translation, `unknown` because corpus metadata is hand-edited

 @returns Names in declaration order, each once, empty when the front matter declares none

 @example
 ```ts
 const names = declaredSubjectNames({ data: { name: 'Mittens', info: { alias: 'Whiskers, Tom', }, }, },);
 ```
 */
export function declaredSubjectNames(
  { data, }: { readonly data: unknown; },
): readonly string[] {
  if (!isJsonRecord(data,))
    return [];

  /**
   Name and string alias by the rule every identity field follows.
   */
  const declared = extractDeclaredIdentity({ data, },);

  /**
   Nested `info` record, where the alias lives.
   */
  const info = isJsonRecord(data.info,)
    ? data.info
    : {};

  /**
   Aliases the front matter declares, whichever form it wrote them in.
   */
  const aliases = isJsonArray(info.alias,)
    ? info.alias
      .filter(function declaredItem(item,): item is string {
        return isDeclared(item,);
      },)
      .map(function trimmed(item,): string {
        return item.trim();
      },)
    : formsOf({ value: declared.alias ?? '', },);

  /**
   Name first, then the aliases. The reading of each already leaves out a blank
   one; the set leaves out a repeat.
   */
  const candidates = [
    ...((declared.name === undefined) ? [] : [declared.name,]),
    ...aliases,
  ];
  return [...new Set(candidates,),];
}

//endregion Subject names
