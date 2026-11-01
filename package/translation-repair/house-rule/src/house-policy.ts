import {
  CREDIT_BY_RULE,
  GAME_JARGON_RULE,
  GRAMMATICAL_ENGLISH_RULE,
  IDIOMATIC_ENGLISH_RULE,
  KEPT_SUBJECT_RULE,
  QUOTED_WORDS_REMINDER,
  QUOTED_WORDS_RULE,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { ARCHIVE_NAME_GLOSS_SCOPE, } from './name-form-policy.ts';

//region House policy
// Editorial rules this corpus is written under, restated for the models.
//
// PROVENANCE: these rules are paraphrased from `CODE_OF_CONDUCT.md` (编写原则)
// in the one-among-us/data repository, which is UNLICENSED. The wording here is
// deliberately our own restatement rather than a copy, so nothing in this
// package reproduces that repository's text. Read the source document when
// changing anything here; do not paste from it.
// First-use glossing also follows the owner's inline-gloss decision. Its
// proper-name scope reconciles that rule with established translation-side
// naming authority; it does not remove definitions or invent glossary entries.
//
// Why the pipeline needs this at all: every stage was previously judging a
// memorial corpus with no knowledge of the rules its authors write under, so
// deliberate, policy-required editorial choices looked like defects. The
// clearest case is protective omission. A page that declines to name a suicide
// method is following the corpus's own rule, and a critic ignorant of that rule
// reports it as `accuracy/omission` and the editor "restores" the very detail
// the rule exists to remove. That is worse than a false positive: acting on it
// makes the shipped translation violate the corpus's reader-protection policy.
//
// TWO RULES WERE ADDED 2026-08-18 FROM A GRADED SHEET, both from one passage.
// The archive read "This is her last self-description:" and the pipeline shipped
// "Below is her final self-description image.", which the grader marked as an
// accessibility defect: a listener has no page to look down. The same passage
// carried a bare "()" copied from the original's trailing emoticon, which says
// nothing in English. Both are production rules rather than grounds for
// reopening text that already reads well.

/**
 Reader-protection and voice rules shared by every stage that judges or
 rewrites this corpus.

 Written as prompt-ready lines because every consumer splices it into a system
 prompt. THE LIST OF CONSUMERS IS NOT STATED HERE ANY MORE: it used to name
 the critic, the adjudicator and the editor, and the editor was not one of
 them, which is exactly the stage this block's own header says it exists to
 stop. A comment naming consumers is a claim nothing checks; `rg` on this
 symbol answers it correctly.

 @example
 ```ts
 const system = `${BASE_RULES}\n\n${HOUSE_POLICY_BLOCK}`;
 ```
 */
export const HOUSE_POLICY_BLOCK: string = `House rules this corpus is written under. These describe how these documents are SUPPOSED to read, so a page following them is correct even when following them looks like a defect, and where another rule on the same sheet disagrees with one of them, the house rule wins:
- Reader protection outranks completeness. When a death or an attempt was by suicide, the specific method is deliberately kept vague, whether she died or survived, and when it involved medication, drug names and dosages are deliberately absent. ${
  // OWNER, 2026-10-20 ("Should reader protection cover a suicide means mentioned outside the account of the
  // death or an attempt?"): "Only the death", whose description read: "Scope the rule to how she died or
  // attempted, so her quote keeps the cyanide capsule as the original and the archive have it. Pro: keeps her own
  // words and the meaning of her stance; a spy-trope capsule teaches nobody how to get one. Con: a named substance
  // tied to suicide appears on the page." The rejected option was "Anywhere on the page" (a substance meant for
  // suicide stays vague wherever it appears). XingZ60's page opens with her own words: living is her right, not
  // her duty, so she could even do things with a potassium cyanide capsule in her mouth, and she is preparing to;
  // in dangerous places, where she might be caught, she would do it and never let her enemies get her. It is not
  // how she died. The XingZ6015 run's lane contest chose a repair that dropped the capsule ("could cost
  // me my life"), two of three judges citing this rule.
  ''
}This rule reaches only the account of how a death or an attempt happened, whether the subject's or anyone's that the page tells of. A means named anywhere else (a person's own words about what they might do or carry, a hypothetical, a quoted work, a story) is outside this rule and is rendered as the ORIGINAL states it. Inside that account, a TRANSLATION that omits, softens, or generalizes such a detail is OBEYING this rule. Never report that as an omission, and never restore the detail, even when the ORIGINAL states it plainly. If the ORIGINAL is more specific than this rule allows, the TRANSLATION is right to be vaguer. The test is replicability: what is kept vague is whatever would tell a reader how to do it, a method, a substance, a dose, where to obtain them. An immediate medical cause of death (shock, cardiac arrest, respiratory failure) gives a reader nothing to copy and stays as the ORIGINAL states it, as do the date, the place and the age. Euthanasia or medically assisted dying (安乐死), where the ORIGINAL states it, stays too: it names a legal, medically administered path that is not accessible to a reader, and inaccessibility is the whole reason the vagueness exists. ${
  // OWNER, 2026-09-27: ""took medication" is fine, because it's not
  // replicable. Replicability is the test." The 2026-09-03 sentence had
  // listed it among the method words kept vague.
  ''
}Medication named with no substance, no dose and no source ("took medication", "had taken medication", "overdosed") tells a reader nothing to copy and may stay as the ORIGINAL states it. Method words that tell a reader how (jumped, hanged, cut, breathed in gas) are still a method and stay vague: the page says that she ended her life, or, where she survived, that she attempted suicide, and keeps the night, the room and the dress, not the means. ${
  // CLASS ONE HUNDRED TWENTY-TWO (shi_Yumiaoya23, 2026-09-25): the judges read
  // "three bottles of some kind of sedative-hypnotic" as vague because it named
  // no drug, read the rule as reaching only the medication that was the means,
  // and let "the rooftop of a tall building" stand as the place.
  ''
}A class of drug (a sedative, a sleeping pill, an antidepressant, a painkiller) is a substance and a count (three bottles, a whole box, forty pills) is a dose, named or not: "three bottles of a sedative" names both. This holds for any medication or drug the page ties to a death or an attempt, not only the one it names as the means; the page says at most that she had taken medication. A place that is itself the means (a rooftop, a high floor, a bridge, a railway line, deep water) is a method too: the page keeps where it happened when that tells a reader nothing to copy (a city, a hotel, a hospital), and otherwise says that she ended her life, or attempted to, without saying where she went to do it. 自杀 names the act, not its outcome: where the ORIGINAL goes on to a rescue, a waking or a life that continues (抢救, 醒来, 幸存), the person survived the attempt and the page says that she tried to end her life or attempted suicide; "ended her life" and "took her own life" assert a death, and on an attempt survived they are a factual error, not vagueness. ${
  // CLASS ONE HUNDRED TWENTY-FOUR (shi_Yumiaoya24, 2026-09-25): the rule opened
  // on a death alone, and the judges chose "she swallowed a large amount of
  // medication ... and attempted to end her life", one ballot writing
  // "survived the attempt, so no method named".
  ''
}A survived attempt is no exception: the page says that she attempted suicide or tried to end her life, never by what means ("swallowed a large amount of medication" names the means). The account of how a death or an attempt happened must be said the same way wherever the page tells it; an obituary block that is vague and a postscript that names the means is a page contradicting itself.
- These pages are a small memorial, not an encyclopedia. Both overwrought writing and clinical detachment are wrong for them, so a rendering that carries warmth is closer to correct than one that reads like a reference work.
- Entries narrate the person in the third person. A contributor, a letter or the person's own words written in the first person (我) stay in the first person.
- Tense is chosen once and held. Chinese marks no tense, so English forces a choice, and that choice belongs to the passage rather than to each sentence: a sentence that opens in one tense and finishes in another is wrong however each half reads on its own. These are memorial pages about people who have died, so the narrative of a life (what the person did, said, made, felt) is told in the past tense, except for the person's words quoted as they wrote or said them, which keep the speaker's own tense because they are the speaker's own voice; the present tense is kept only for what is still true as the page is read (the page itself, a surviving work, a standing fact). Where the English already on the page tells the life in the present, that line is not the tense to keep: the ORIGINAL marks no tense and the past is the English for a life that has ended, so bringing it to the past is a correction, never a change to the time. ${QUOTED_WORDS_RULE}
- Pronouns follow the person's own stated wishes, read off the WHOLE ORIGINAL and never off one sentence. A pronoun the ORIGINAL writes is rendered as written where it stands: 她 is she, 他 is he, and TA is singular they, on every page, including a page whose other passages write 她 or 他, since these pages are written by several hands and a writer who chose TA beside a page of 她 chose it. Never "correct" a neutral pronoun into a gendered one, and never neutralize a pronoun the ORIGINAL uses for the person. TA is the neutral pronoun this corpus writes for a person who did not specify, spelled TA, Ta or ta, and English renders it as singular they (they, them, their), with TA 们 as plural they; a Ta or TA left standing in the English is an untranslated word, not a preserved choice. Chinese leaves subjects unstated freely: a sentence with no subject is about the person the passage is about and takes the pronoun the ORIGINAL uses for them elsewhere (她, 他, TA), which a "pronoun" line in the DECLARED NAMES block states when the page has one; that line settles unstated subjects only, never a pronoun the ORIGINAL wrote. Only where the whole ORIGINAL uses a neutral pronoun or avoids pronouns entirely does an unstated subject take the neutral rendering. ${KEPT_SUBJECT_RULE}
- Community and in-group vocabulary is expected. A term rendered by its conventional community meaning is correct even when a literal reading of the characters says otherwise. ${ARCHIVE_NAME_GLOSS_SCOPE} A generic term with no English equivalent (a community term, an institution, an abbreviation local to the person's world, such as 大证 or WER) is kept in English letters (translated literally, or romanized where it is a name), never left in Han, and glossed inline the first time it appears on the page, in parentheses or a following clause, and carried alone after that; not in a footnote, and the same way everywhere on the page. A term that does have an everyday English equivalent is translated rather than kept: 贴贴 is cuddling or snuggling, not 贴贴 with a gloss. 那些秋叶 is this site's own name in Chinese and is rendered "One Among Us" wherever it appears (那些秋叶的成员 are "One Among Us members", 那些秋叶维护组 is "One Among Us Transgender Support"), never as a literal translation of the characters.
- The page is plain written English for a general reader, whatever the register of the ORIGINAL. Chat shorthand and internet slang are rendered by the plain English words, never carried across and never matched by English slang: OD is overdose (overdosing, overdosed), never OD or ODing, except where it is the means of a death or an attempt, which the reader-protection rule keeps unnamed; jk 裙 is a sailor uniform; MtF is trans woman or trans girl, as 药娘 is; a work written as its initials takes its full title (TGT is The Grand Tour). The English adds no slang of its own either (gearhead, slacked off). An abbreviation a general reader would not know (SRS, GAHT, OST, POV, ACG) is spelled out where it first appears, with the abbreviation after it in parentheses only if the page uses it again. Abbreviations a general reader knows (HRT, ICU) stay, and so do names written in Latin letters (a product such as QQ, a handle, a link, a web address), which are names, not shorthand. An identity label with no everyday English equivalent (MtX) is kept and glossed as the community-vocabulary rule says. Quoted speech keeps the speaker's warmth and voice, but its shorthand is spelled out like the rest of the page. A rendering that carries shorthand or slang across is a defect to hold against it, never faithfulness to the ORIGINAL's tone. ${GAME_JARGON_RULE}
- ${IDIOMATIC_ENGLISH_RULE}
- ${GRAMMATICAL_ENGLISH_RULE}
- The page is written in Canadian English (en_CA), in spelling and in vocabulary. ${
  // CLASS ONE HUNDRED THIRTY-TWO (owner, 2026-09-25: "The convention is and
  // should be en_CA. It should have been documented and in the prompts."): no
  // sheet named the variety, so shi_Yumiaoya32 mixed "counselor" and "head of
  // year" with "a petrol car". The spellings follow the Canadian Press
  // Stylebook and the Canadian Oxford as McGill's style guide sets them out.
  ''
}Spelling: -our (colour, behaviour, favourite, honour, neighbour), -re (centre, theatre, and metre for the unit, while a device that measures is a meter), a doubled l before a suffix (travelled, cancelled, counsellor), -ize and -yze (realize, organize, analyze, romanize), and cheque, defence, offence, grey, catalogue, program, advisor, licorice. Dates are written month first (April 29, May 4, March 13, 2024), never day first (29th April, 4 May). A period or comma after quoted words goes inside the closing quotation mark (her “Old Man Chen,” the “nap champion.”); a question or exclamation mark goes inside only when it belongs to the quotation. Vocabulary: where Canadian English has its own word, use it: gas, not petrol; Grade 12, not Year 12 or senior year; math, not maths; mom, not mum; washroom. English already on the page in another variety's spelling is respelled the Canadian way, which is a correction of form and never a change of meaning. Names, titles and quoted English keep their own spelling.
- A person's handle with no declared, archive or corpus rendering (a signer, a cited author, an account name the ORIGINAL writes in Han) is romanized as the handle is read, pinyin for a Chinese handle, in capitalized groups of two syllables with a space between groups and a lone trailing syllable standing by itself (锦心 is Jinxin, 雨狸 is Yuli, 洁澄天奏 is Jiecheng Tianzou), never one joined word, with its literal meaning in parentheses the first time it appears on the page (Jinxin (Brocade Heart)) and the romanization alone after that; never left in Han, never rendered by its meaning alone, and the same way everywhere on the page. A handle the person writes in Latin letters keeps their spelling (HiYku, wing, Mikä), and Latin letters the ORIGINAL writes against a Han handle stay after the romanization (洁澄天奏Official is Jiecheng Tianzou Official).
- A work the ORIGINAL names is called by its official English title when one exists: a published English edition, an official localization, or a title the work carries in English (《活着》 is *To Live*, 《魔法少女小圆》 is *Puella Magi Madoka Magica*). A title that plays on a work's Chinese title takes the play on that work's English title: 「Aiden 的奇幻漂流」 plays on 少年 Pi 的奇幻漂流, *Life of Pi*, so it reads "Life of Aiden". Where the entry establishes vocabulary in a footnote or an editor's comment (a "note" or "editor comment" line in the DECLARED NAMES block), that vocabulary is used. An "editor comment" line says where the comment sits, "under heading X" or "before the first heading"; a comment that speaks of "this title", "this section" or "here" speaks of that heading and its section and no other, and settles nothing about any other heading. A "web lookup" line is evidence about a title, to be read with judgment, not a declaration. Where none of these exists, translate the title and keep it the same everywhere on the page. A title is never left in Han as the work's name: the Han may follow the English title in parentheses the first time it appears, and never stands alone. ${
  // OWNER, 2026-09-27 ("Italics for works"): most archives set a work's
  // title in italics and `archive-italic-title-restore.ts` puts them back,
  // so the rule's quotation marks had judges marking the archive down.
  ''
}In English prose the title of a work (a book, a game, a film, a show, an album) stands in italics (*To Live*), and the title of a song, a poem, an episode or a chapter in quotation marks ("Zero Hour"); never in the Chinese title marks 《》. ${CREDIT_BY_RULE}
- Refer to a passage, image or section by NAMING it, never by its position on the page. Do not use "below", "above", "the following" or "earlier" to point at a place on the page; the same words about time, height or rank (a few days earlier, the following year, below zero, the God Above) are ordinary English. Where the ORIGINAL uses such a word to point at the page, render what it points AT instead: an original reading "the picture below is her last self-description" becomes "this is her last self-description", not "below is her last self-description". A screen reader gives its listener no page to look down, so a positional reference tells them nothing.
- Do not carry punctuation that means nothing in English. A trailing emoticon, a bare pair of empty brackets, or a run of full-width marks that carried tone in Chinese is not rendered by copying the characters across; render the tone in words or leave it out. Corner brackets 「」 are Chinese quotation marks: around a quotation they become English quotation marks; around a name or a term used to refer to its thing they are dropped, so 「那些秋叶」 reads One Among Us; around a name or a word being introduced or talked about (称呼她为「盐田姐姐」) they become English quotation marks, so it reads calling her "Sister Yantian".`;

/**
 What English forces on a translator and Chinese does not, shared by every
 sheet that grades a rendering.

 ONE PARAGRAPH, NO CLOSING SENTENCE. The sentence that used to end it, telling
 a reader to hold a forced choice against a CANDIDATE, names a term only the
 judging sheets define, so each consumer supplies its own close in its own
 vocabulary.

 @example
 ```ts
 const opening = FORCED_DIFFERENCES;
 ```
 */
const FORCED_DIFFERENCES =
  'Obligatory differences between the two languages are never faults. Each language forces choices the other leaves open, and meeting English\'s own requirements is not an addition or an alteration: supplying a pronoun, a number, an article or A TENSE that the ORIGINAL leaves unmarked is REQUIRED. Chinese marks no tense, so rendering it in past or present is a choice English forces, never a change to the time the ORIGINAL refers to.';

/**
 Judge-facing close of `JUDGE_POLICY_BLOCK`.

 SEPARATE FROM `FORCED_DIFFERENCES` because it names a candidate and a
 criterion, and the measuring sheets have neither. They speak of findings,
 issues, regions and the text under review, so a shared close would put two
 undefined terms in front of three checkers.

 ITS PRECEDENCE SENTENCE NAMES ANY RULE, NOT ONLY A CRITERION (ledger S6):
 the contest splices this block and asks two questions rather than ranking
 criteria, so a sentence about criteria alone governed nothing on that sheet.

 @example
 ```ts
 const tail = JUDGE_POLICY_TAIL;
 ```
 */
const JUDGE_POLICY_TAIL = `Hold it against a candidate only when the choice it made is the WRONG one, and say which reading the ORIGINAL supports. WHERE THE FORCED CHOICE IS TENSE, THE ORIGINAL SUPPORTS NEITHER READING AND THE ENGLISH IS THE AUTHORITY INSTEAD, IN THIS ORDER: first the past tense the house rule sets for a life that has ended (a candidate moving a present-tense line of the life to the past has corrected it, and a candidate keeping the life in the present because the text being replaced had it so has kept the wrong tense; there is no rule holding the narration of a life in the present, and the person's words quoted as they wrote or said them are no part of that narration, since they keep the speaker's own tense); then the tense the rest of the passage holds for what is still true as the page is read. Name that instead of the ORIGINAL, and a candidate that disagrees with itself inside one sentence is wrong with no further evidence needed. ${QUOTED_WORDS_REMINDER}

WHERE THE FORCED CHOICE IS A PRONOUN FOR A SUBJECT THE ORIGINAL LEAVES UNSTATED, AN UNSTATED SUBJECT IS NOT A NEUTRAL PRONOUN. A subjectless sentence about the person takes the pronoun the page uses for them, so a candidate supplying that pronoun has invented nothing, and a candidate putting a neutral pronoun there on a page that uses her or his pronoun has made the WRONG choice. Only where the WHOLE ORIGINAL uses a neutral pronoun or avoids pronouns for the person is the neutral rendering of an unstated subject the right one. A TA THE ORIGINAL WRITES IS NOT AN UNSTATED SUBJECT: it is rendered singular they where it stands, whatever pronoun the identity line counts for the rest of the page, and a candidate resolving a written TA to she or he has made the WRONG choice, as has a candidate writing they where the ORIGINAL wrote 她 or 他.

WHERE A CRITERION OR ANY OTHER RULE YOU HAVE BEEN GIVEN DISAGREES WITH A HOUSE RULE, THE HOUSE RULE WINS. A candidate vaguer than the ORIGINAL because reader protection asks for it in the account of how a death or an attempt happened has left nothing out; a candidate vaguer about a means named anywhere else has left out what the ORIGINAL states; and a candidate naming what it points at rather than where it sits on the page is obeying a rule rather than departing from the text.`;

/**
 Measurement-facing close, for sheets that grade or check rather than choose.

 WHY THE PRECEDENCE SENTENCE IS RESTATED RATHER THAN SHARED: a measuring sheet
 has no ranked criteria, so a precedence sentence naming `A CRITERION`
 names something that is not in front of it, and a model reading a rule about
 an absent thing may take the whole block as addressed to someone else.

 SUPPORTED BY THE ORIGINAL IS SAID OUTRIGHT because `introduced-defect-wire`
 carries the opposite rule in as many words: content the AFTER text drops is
 damage only if the ORIGINAL supports it. On a protected detail the ORIGINAL
 DOES support it, which is the entire point of reader protection, so a spliced
 block that did not name the interaction would leave two live rules in
 disagreement.

 VERDICT NAMES ARE NOT HERE. The three consumers have disjoint vocabularies,
 so each splice site adds the one line mapping this principle onto its own
 verdicts. Restating the principle in three places is what would drift; naming
 three different verdicts once each cannot.

 @example
 ```ts
 const tail = MEASUREMENT_POLICY_TAIL;
 ```
 */
const MEASUREMENT_POLICY_TAIL = `Hold a forced choice against the text under review only when the choice it made is the WRONG one, and say which reading the ORIGINAL supports. WHERE THE FORCED CHOICE IS TENSE, THE ORIGINAL SUPPORTS NEITHER READING AND THE ENGLISH IS THE AUTHORITY INSTEAD, IN THIS ORDER: first the past tense the house rule sets for a life that has ended (a line of the life moved from the present to the past is a correction, not a defect, and the tense the text had before the edit is no authority for keeping a life in the present); then the tense the rest of the passage holds for what is still true as the page is read. Name that instead of the ORIGINAL, and a passage that disagrees with itself inside one sentence is wrong with no further evidence needed. ${QUOTED_WORDS_REMINDER}

WHERE A RULE YOU HAVE BEEN GIVEN AND A HOUSE RULE DISAGREE, THE HOUSE RULE WINS. You are measuring, and a measurement that penalizes a page for obeying the rules it was written under reports damage where there is none.

SUPPORTED BY THE ORIGINAL IS NOT ENOUGH TO MAKE SOMETHING MISSING A DEFECT. Reader protection keeps specific detail out of these pages only in the account of how a death or an attempt happened, and there precisely where the ORIGINAL states it, so text vaguer than the ORIGINAL about how a death or an attempt happened, where it happened when the place was the means, or about a medication it involved, is obeying the rule rather than omitting. Never file that as a finding and never count it as damage. Text vaguer than the ORIGINAL about a means named outside that account (a person's own words about what they might do, a hypothetical, a quoted work, a story) has left out what the ORIGINAL states, and that is reported like any other omission.

WHERE AN ISSUE OR FINDING YOU WERE GIVEN ASKS FOR SUCH A DETAIL, THE ISSUE ITSELF IS WRONG. Do not agree that the text should carry it. Where the text under review HAS restored a detail reader protection keeps out, that restoration is new damage rather than a fix, however plainly the ORIGINAL states it.`;

/**
 House rules plus the two things a JUDGE needs that a producer does not.

 WHY JUDGES NEEDED THEIR OWN BLOCK. Every producing sheet splices
 `HOUSE_POLICY_BLOCK`; no judging sheet did. So the stages that decide what
 SHIPS were the only ones that had never been told what this corpus is written
 under, and criterion one, "every proposition of the ORIGINAL is rendered,
 nothing left out", contradicts reader protection outright.

 AND THE TENSE RULE, which existed only in `critic-prompt.ts`. Chinese leaves
 tense unmarked and English cannot. Measured on the consolidation bed: of the
 four slates a judge refused ENTIRELY, three were refused for tense, one of
 them saying every candidate had altered the time reference by rendering a
 tenseless copula as "was". The rule that answers that was in the codebase and
 out of reach.

 PRECEDENCE IS STATED, because a numbered list reads as the standard and
 anything beside it reads as background. Three measured refusals came from
 judges applying a numbered criterion literally.

 @example
 ```ts
 const system = `${task}\n\n${JUDGE_POLICY_BLOCK}`;
 ```
 */
export const JUDGE_POLICY_BLOCK: string = `${HOUSE_POLICY_BLOCK}

${FORCED_DIFFERENCES} ${JUDGE_POLICY_TAIL}`;

/**
 House rules for the sheets that MEASURE rather than write or choose.

 The rendering auditor, the resolution checker and the introduced-defect
 prober all grade text against the ORIGINAL, and all three scored a page
 obeying reader protection as a defect because none had ever been told the
 rule. They were fixed after the deciding sheets on the reading that nothing
 ships from any of them, which does not hold for two: checker ballots decide
 which issues count as resolved, strip an edit voted worse (ledger L3) and
 roll a rewrite back (ledger L11), and a probe claim the screen admits rolls
 a naturalness rewrite back. The rendering auditor decides which defects get
 worked on next.

 @example
 ```ts
 const system = `${task}\n\n${MEASUREMENT_POLICY_BLOCK}`;
 ```
 */
export const MEASUREMENT_POLICY_BLOCK: string = `${HOUSE_POLICY_BLOCK}

${FORCED_DIFFERENCES} ${MEASUREMENT_POLICY_TAIL}`;

//endregion House policy
