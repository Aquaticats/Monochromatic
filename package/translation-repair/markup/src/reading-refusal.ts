import {
  isLatinLetter,
  isLatinLetterOrMark,
} from './latin-letters.ts';
import { carriesWord, } from './word-bounds.ts';

//region Reading refusal
// WHETHER A MODEL DECLINED TO READ A PICTURE, rather than read it.
//
// WHY THIS IS ITS OWN SCREEN AND NOT A LONGER PHRASE LIST. A fixed list of
// wordings refuses only what somebody thought to write down, and real traffic on
// 2026-08-19 showed how narrow that is. Two readers were sent the same picture
// and answered:
//
//     Qwen3.6-27B    "There is no text visible in this image."
//     Kimi-K3        "No legible text is visible."
//
// The list carried `no text is visible` and `no visible text`. The first reply
// misses by word order, the second by the word `legible` sitting between `no`
// and `text`. Both passed the screen, and then CORROBORATED EACH OTHER at 0.565
// trigram overlap, because two ways of saying "there is nothing here" share
// their vocabulary exactly as two transcriptions of one passage do.
//
// THAT IS THE FAILURE THIS EXISTS TO STOP. A corroborated refusal is worse than
// no reading, because a refused picture is silent while a corroborated one
// travels to the translator and the judge under the heading "WHAT THE PICTURES
// HERE SAY, transcribed by two readers that agreed". The sheets would then
// assert that a picture says `There is no text visible in this image.`
//
// SHAPE RATHER THAN WORDING. A refusal talks ABOUT the picture: it negates, it
// names the picture or its text, and it is a sentence rather than a passage. A
// transcription reproduces what the picture holds and does none of those. So the
// test is a negation word plus a picture word inside a short reading, which no
// wording of "there is nothing to read" escapes and no transcription of a
// Chinese passage trips.
//
// MEASURED ON WHAT WAS AVAILABLE, both directions. The six real transcriptions
// kept from the 2026-08-19 boundary probe run 976 to 1520 bytes and contain ZERO
// English negation words and ZERO picture words between them, so the rule cannot
// reach them on either test. The two refusals the WHY THIS IS ITS OWN SCREEN paragraph quotes carry both, at 27 and 41
// characters. The separation is not marginal on this sample, and the length
// bound is a second margin rather than the main one: an English transcription
// long enough to discuss text and negate something is past it before either word
// list is consulted.

/**
 Longest a refusal runs, in characters after trimming.

 A REFUSAL IS A SENTENCE. The two measured run 27 and 41 characters, and the
 six real transcriptions run 976 and up, so this sits far from both rather
 than between two crowded populations. It exists so a long reading that
 genuinely transcribes a passage about a picture is never reached by the word
 tests at all.
 */
const REFUSAL_MAX_CHARS = 160;

/**
 Words that negate, lowercased.

 Matched as whole words, so `not` does not fire inside `note` and `no` does
 not fire inside `nothing`, which is listed separately in its own right.
 */
const NEGATION_WORDS: readonly string[] = [
  'no',
  'not',
  'none',
  'nothing',
  'cannot',
  'can\'t',
  'unable',
  'without',
  'doesn\'t',
  'isn\'t',
  'aren\'t',
  'unreadable',
  'illegible',
];

/**
 Words naming the picture or its text rather than reproducing either.

 A TRANSCRIPTION DOES NOT NAME ITS MEDIUM. Somebody's letter says what it
 says; it does not call itself an image. These are the words a model reaches
 for when it is describing its own situation instead of the picture's
 contents.
 */
const PICTURE_WORDS: readonly string[] = [
  'text',
  'image',
  'picture',
  'photo',
  'photograph',
  'writing',
  'written',
  'character',
  'characters',
  'legible',
  'readable',
  'caption',
  'content',
  'contents',
  'ocr',
  'transcribe',
  'transcription',
  'visible',
];

/**
 Whether a character can sit inside one of the `PICTURE_WORDS`: a Latin letter,
 accented or not, a combining mark, or an apostrophe. ASCII letters alone cut
 `café` to `caf` (ledger B18).

 APOSTROPHE INCLUDED so `can't` and `isn't` stay single words rather than
 splitting into a fragment that matches nothing. Both the straight and the
 typographic apostrophe count, since a model writing prose reaches for either.

 @param character - character to weigh

 @returns Whether it continues a word

 @example
 ```ts
 const inside = continuesWord({ character: 'a', },);
 ```
 */
function continuesWord({ character, }: { readonly character: string; },): boolean {
  return isLatinLetterOrMark({ character, },)
    || (character === '\'')
    || (character === '’');
}

/**
 Whether a character can open one of the `PICTURE_WORDS`: a Latin letter or an
 apostrophe, never a combining mark, which belongs to the letter before it.

 @param character - character to weigh

 @returns Whether a word may start with it

 @example
 ```ts
 const opens = opensWord({ character: 'N', },);
 ```
 */
function opensWord({ character, }: { readonly character: string; },): boolean {
  return isLatinLetter({ character, },)
    || (character === '\'')
    || (character === '’');
}

/**
 Splits text into lowercased Latin words.

 A LINEAR SCAN rather than a pattern, per `RG1`: the rule is "runs of Latin
 letters, combining marks and apostrophes are words, everything else
 separates them", which a scan
 states directly, runs in one pass, and cannot backtrack. The typographic
 apostrophe is folded onto the straight one so a word list needs only one
 spelling of each contraction.

 @param text - reading to split

 @returns Every word it holds, lowercased, in order

 @example
 ```ts
 const words = latinWords({ text: 'No text.', },);
 ```
 */
export function latinWords({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Words closed so far.
   */
  const words: string[] = [];

  /**
   Characters of the word being built, empty between words; joined once when
   the word closes rather than rebuilt per character.
   */
  const current: string[] = [];

  /**
   Closes the word being built, if any.
   */
  function closeWord(): void {
    if (current.length === 0)
      return;
    words.push(current.join('',)
      .toLowerCase(),);
    current.length = 0;
  }

  for (const character of text) {
    /**
     Whether the character opens a word or goes on with the one being built.
     */
    const inWord = (current.length === 0)
      ? opensWord({ character, },)
      : continuesWord({ character, },);
    if (inWord) {
      current.push((character === '’') ? '\'' : character,);
      continue;
    }
    closeWord();
  }
  closeWord();
  return words;
}

/**
 Whether a reading is a model declining to read rather than a reading.

 SHORT, NEGATING, AND ABOUT THE PICTURE. All three are required, so a long
 transcription is never tested, a description with no negation passes, and a
 negation about something other than the picture passes. Each condition alone
 would refuse real readings; together they describe only the shape a refusal
 takes.

 @param reading - what model returned for image, whitespace and all

 @returns Whether it declines rather than transcribes

 @example
 ```ts
 const declined = readsAsRefusal({ reading: 'No legible text is visible.', },);
 ```
 */
export function readsAsRefusal({ reading, }: { readonly reading: string; },): boolean {
  /**
   Reading without its surrounding whitespace, which is what the length bound
   is about: a reply padded with newlines is still a sentence.
   */
  const trimmed = reading.trim();
  if (trimmed.length > REFUSAL_MAX_CHARS)
    return false;

  /**
   Every Latin word it holds, lowercased.
   */
  const words = latinWords({ text: trimmed, },);

  return NEGATION_WORDS.some(function negates(word,): boolean {
    return words.includes(word,);
  },)
    && PICTURE_WORDS.some(function namesPicture(word,): boolean {
      return words.includes(word,);
    },);
}

/**
 Words naming text itself, which an absence report negates: "no visible
 text", "no words", "nothing written", "no legible characters".

 MEASURED ON THE PROBE OF 2026-09-04 over the five pictures every run log had
 ended `unavailable`: 14 of 15 refused replies named text this way, and the
 one that did not ("I cannot read the image.") was the reply of a model told
 to say so plainly when it cannot read.
 */
const TEXT_WORDS: readonly string[] = [
  'text',
  'word',
  'words',
  'writing',
  'written',
  'character',
  'characters',
  'lettering',
  'inscription',
  'inscriptions',
  'signature',
  'signatures',
  'label',
  'labels',
  'caption',
  'captions',
  'legible',
];

/**
 Phrases about the picture's quality or the model's access to it, lowercased,
 any of which makes a refusal an inability rather than an absence report:
 text the model could not make out is text, not nothing.

 EACH IS READ AS THE START OF A WORD (ledger B23): "load" names "loaded" and
 "loading" but not "workload", and "quality" is not found in "equality".

 A PREFIXED FORM THAT STILL NAMES ACCESS IS LISTED ON ITS OWN, taken from
 every word of `/usr/share/dict/words` that carries a marker past its start,
 plus the two software words that list lacks (`unprocessable`, `unloadable`).
 "download", "upload" and "reload" stay inability although an absence report
 could name a download icon, because the costs differ: a reader that never
 received the picture says "please upload it again", two such replies read
 as absence would confirm a picture textless in silence, while an absence
 read as inability leaves the picture unread, and the visual evidence check
 then pauses the entry and says why. `inaccessib` and `unaccessib` are stems,
 since the words part at them ("inaccessible", "inaccessibly",
 "inaccessibility").
 */
const INABILITY_MARKERS: readonly string[] = [
  'unclear',
  'blurry',
  'blurred',
  'resolution',
  'make out',
  'process',
  'unprocessable',
  'access',
  'inaccessib',
  'unaccessib',
  'load',
  'download',
  'upload',
  'reload',
  'unloadable',
  'corrupt',
  'distorted',
  'quality',
  'illegible',
  'unreadable',
  'too small',
  'too dark',
  'obscured',
  'cut off',
  'cropped',
];

/**
 Whether a reading negates anything at all.

 @param reading - what model returned for image

 @returns Whether any negation word stands as a word of its own in it

 @example
 ```ts
 const negated = negatesSomething({ reading: 'I can't.', },);
 ```
 */
export function negatesSomething({ reading, }: { readonly reading: string; },): boolean {
  /**
   Every Latin word it holds, lowercased.
   */
  const words = latinWords({ text: reading, },);
  return NEGATION_WORDS.some(function negates(word,): boolean {
    return words.includes(word,);
  },);
}

/**
 Whether a refusal reports that the picture carries no text, as opposed to
 declining to read it.

 ASKED OF A REFUSAL ONLY. This does not decide whether a reply is a refusal;
 `readsAsRefusal` and the phrase list do. It decides which kind: an absence
 report names text and says nothing about the picture's quality or the
 model's access to it, and an inability does the reverse, or names neither.
 The two are answers to different questions, and the pair stage treats them
 differently: two absence reports confirm a textless picture, while an
 inability is asked again and, if it holds, leaves the picture unread.

 @param reading - refused reply, whitespace and all

 @returns Whether it reports absence rather than inability

 @example
 ```ts
 const absent = refusalReportsAbsence({ reading: 'There is no visible text in this image.', },);
 ```
 */
export function refusalReportsAbsence({ reading, }: { readonly reading: string; },): boolean {
  /**
   Reply without its surrounding whitespace.
   */
  const trimmed = reading.trim();

  /**
   Reply lowercased, where the markers are matched as phrases.
   */
  const lowered = trimmed.toLowerCase();
  if (INABILITY_MARKERS.some(function marks(marker,): boolean {
    return carriesWord({
      text: lowered,
      needle: marker,
      end: 'open',
    },);
  },))
    return false;

  /**
   Every Latin word it holds, lowercased.
   */
  const words = latinWords({ text: lowered, },);
  return TEXT_WORDS.some(function namesText(word,): boolean {
    return words.includes(word,);
  },);
}

//endregion Reading refusal
