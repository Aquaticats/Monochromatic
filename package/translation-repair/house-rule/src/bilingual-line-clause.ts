//region Bilingual line clause
// CLASS EIGHTY (shi_Yumiaoya, 2026-09-22). The original quotes a film line
// in Chinese with the film's own English directly beside it, and attributes
// it the same way, Chinese line then English line. The line floor counts
// such a pair as one line owed (class forty-seven, `line-structure-guard.ts`)
// and the archive carries the English alone, but every line-structure
// wording on the sheets said "never drop a line" and the judges' criterion
// called a dropped line FAULTY. So the writers rendered the Chinese line a
// second time in English beside the film's own words (shi_Yumiaoya10: "If I
// can never see you again, I wish you good morning, good afternoon, and good
// night." above "And in case I don't see you……good afternoon, good evening,
// and good night."), or kept it in Chinese on an English page (shi_Yumiaoya4
// to 9), and the judges split: "omits the first quoted line" against "keeps
// the original English line after adding a new translation".
//
// STATED FOR THE MODELS, NOT ENFORCED AS A SURPLUS BOUND. Measured over the
// pinned corpus, shi_Yumiaoya's two pairs are the only Chinese lines whose
// English stands beside them; a Chinese line followed by an English line of
// DIFFERENT content occurs on Arita, NIGHT81473140, gqt, zhangyubaka, Y1Ran,
// luxuanwen3, cheonwoomaeng, hakureico, Susiethegamer and XIEPT2 (an
// introduction then a quoted poem, a letter then its signature), and the
// pair count cannot tell the two apart. A mechanical bound would refuse
// faithful renderings on those pages; the clause names the condition and the
// judges read the lines.

/**
 Clause every line-structure wording carries: a line the original gives in
 Chinese and in English beside it is one line, already rendered.

 ONE WORDING FOR THE THREE SHEETS (translators, judges, editors), so a
 rewording of one cannot drift from the others, and so the guard can read it
 off each.

 @example
 ```ts
 const rule = `The ORIGINAL is line-structured. ${BILINGUAL_LINE_CLAUSE}`;
 ```
 */
export const BILINGUAL_LINE_CLAUSE: string = 'A line the ORIGINAL gives twice, once in Chinese and once in English directly beside it '
  + '(a quotation in both languages, and its attribution the same way), is ONE line whose English is already its rendering: '
  + 'carry that English line once, as the ORIGINAL has it; a rendering carrying the English line alone for such a pair has dropped nothing, '
  + 'and one carrying the Chinese line, or a second English wording of it, beside the English has invented a line.';

//endregion Bilingual line clause
