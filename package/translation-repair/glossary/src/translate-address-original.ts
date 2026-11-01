//region What the original passage writes about its people
// The original side of the second-person floor in `translate-address-drop.ts`:
// where a passage addresses someone, and how many third-person pronouns it
// writes. Split out of that file for its line budget; the reasoning of each
// rule stays beside the rule.

/**
 Characters that address someone in the second person.
 */
const ADDRESS_CHARACTERS: ReadonlySet<string> = new Set([
  '你',
  '您',
]);

/**
 Character that turns an address character into a greeting.
 */
const GREETING_TAIL = '好';

// CLASS ONE HUNDRED EIGHTY-FIVE (TianqiChen66619, 2026-09-27). The page's
// refrain 你看头壳里，她在最狭小的空间中撑起了最完美的世界 opens with 你看,
// the imperative "look", which English writes without "you"; the floor read
// it as an address, refused the standing "Look inside the headpiece: she
// built…" whose "she" rendered the original's own 她, the bench forced a
// "for you" into its proposals to pass, the gate preferred the refused
// standing, and the entry stopped INCOMPLETE at slice 9. 你看 or 你瞧 opening
// a clause, with no complement making the verb its own (被你看到, 替你看管,
// 你看到了吗 stay addresses), is the imperative and is not counted.

/**
 Verbs that, after 你 at the head of a clause, make the imperative "look".
 */
const LOOK_VERBS: ReadonlySet<string> = new Set([
  '看',
  '瞧',
]);

/**
 Characters that, right after 看 or 瞧, give the verb a result, an aspect or
 an object of its own (看到, 看见, 看管, 看书), so the 你 before it is an
 address rather than the imperative.
 */
const LOOK_COMPLEMENTS: ReadonlySet<string> = new Set([
  '到',
  '见',
  '过',
  '着',
  '了',
  '完',
  '懂',
  '清',
  '出',
  '管',
  '待',
  '望',
  '护',
  '守',
  '病',
  '书',
]);

/**
 Characters after which a clause opens: whitespace, sentence and clause
 punctuation, brackets, quotation marks and the markdown marks that open a
 line.
 */
const CLAUSE_OPENERS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
  '\u3000',
  '。',
  '！',
  '？',
  '，',
  '、',
  '；',
  '：',
  '…',
  '—',
  '「',
  '」',
  '『',
  '』',
  '“',
  '”',
  '‘',
  '’',
  '（',
  '）',
  '《',
  '》',
  '【',
  '】',
  '(',
  ')',
  '[',
  ']',
  '"',
  '\'',
  '!',
  '?',
  ',',
  '.',
  ';',
  ':',
  '~',
  '～',
  '>',
  '*',
  '_',
  '-',
]);

/**
 Whether the address character at an offset opens the imperative "look"
 (你看, 你瞧) at the head of a clause, which English writes without "you".

 @param text - original passage, comments already cut

 @param at - offset of the address character

 @returns True for the imperative; false where the verb takes a complement
 or the 你 sits inside a clause

 @example
 ```ts
 opensLookImperative({ text: '。你看窗外', at: 1, },); // true
 opensLookImperative({ text: '被你看到', at: 1, },); // false
 ```
 */
function opensLookImperative(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  if (!LOOK_VERBS.has(text.charAt(at + 1,),))
    return false;
  if (LOOK_COMPLEMENTS.has(text.charAt(at + 2,),))
    return false;
  if (at === 0)
    return true;
  /**
   Unit just before the address character; every clause opener is one unit.
   */
  const before = text.charAt(at - 1,);
  if (CLAUSE_OPENERS.has(before,))
    return true;
  // A doubled imperative (你看你看) opens its second half after the first.
  return LOOK_VERBS.has(before,) && (text.charAt(at - 2,) === '你');
}

// LEDGER F-1 (2026-09-27). Replaying the floor over every archive slice
// against itself refused correct English in seven slices: a closing wish
// rendered as an imperative beside "he" for the original's own 他, an address
// in one block and a pronoun English adds for a subject Chinese drops in
// another, 其 rendered "she", a cry whose "you" English drops beside "her" for
// the original's own 她, and 迷你, 你们好 and 你追我赶, which address nobody.
// The floor now reads block by block where the two texts have as many blocks,
// and refuses only a block whose rendering carries MORE third-person pronouns
// than its original writes: a pronoun that stands where the address stood is a
// surplus, and one rendering the original's own is not.

/**
 Idioms that pair 你 with 我 to mean "each other" or "back and forth" and
 address nobody.
 */
const MUTUAL_IDIOMS: readonly string[] = [
  '你追我赶',
  '你来我往',
  '你死我活',
  '你争我夺',
  '你推我让',
  '你侬我侬',
  '你情我愿',
  '你中有我',
  '你一言我一语',
];

/**
 Whether the address character at an offset addresses nobody: the 你 of 迷你
 (mini), of the greeting 你们好, or of an idiom meaning "each other".

 @param text - original passage, comments already cut

 @param at - offset of the address character

 @returns True where no one is addressed

 @example
 ```ts
 addressesNobody({ text: '迷你猫窝', at: 1, },); // true
 ```
 */
function addressesNobody(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  if (text.charAt(at - 1,) === '迷')
    return true;
  if (text.startsWith(
    '们好',
    at + 1,
  ))
    return true;
  return MUTUAL_IDIOMS.some(function opensIdiom(idiom,): boolean {
    return text.startsWith(
      idiom,
      at,
    );
  },);
}

/**
 How many times a passage addresses someone in the second person, greetings,
 the clause-opening imperative "look" and a 你 addressing nobody left out.

 @param text - original passage, comments already cut

 @returns Count of address characters that address someone

 @example
 ```ts
 addressCount({ text: '你好，你来了。你看窗外。', },); // 1
 ```
 */
export function addressCount({ text, }: { readonly text: string; },): number {
  /**
   Addresses counted so far.
   */
  let count = 0;
  for (let at = 0; at < text.length; at += 1) {
    if (!ADDRESS_CHARACTERS.has(text.charAt(at,),))
      continue;
    if (text.charAt(at + 1,) === GREETING_TAIL)
      continue;
    if (opensLookImperative({
      text,
      at,
    },))
      continue;
    if (addressesNobody({
      text,
      at,
    },))
      continue;
    count += 1;
  }
  return count;
}

/**
 Characters that write a person in the third person: 他, 她, 祂, and the
 literary 其 (其在 Telegram 留下的回忆, "the memories she left").
 */
const THIRD_PERSON_CHARACTERS: ReadonlySet<string> = new Set([
  '他',
  '她',
  '祂',
  '其',
]);

/**
 Words in which one of those characters names no person (其他 "other",
 他人 "others", 其实 "in fact" and the like), so it is not counted.
 */
const NON_PERSON_WORDS: readonly string[] = [
  '其他',
  '其它',
  '他人',
  '他乡',
  '他日',
  '他处',
  '他国',
  '他者',
  '其实',
  '其中',
  '其次',
  '其余',
  '其间',
  '尤其',
  '极其',
  '与其',
];

/**
 Whether the character at an offset sits inside a word where it names no
 person.

 @param text - original passage

 @param at - offset of a third-person character

 @returns True inside such a word

 @example
 ```ts
 insideNonPersonWord({ text: '其他猫', at: 1, },); // true
 ```
 */
function insideNonPersonWord(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  return NON_PERSON_WORDS.some(function covers(word,): boolean {
    /**
     Where the character sits in the word, -1 where it is not in it.
     */
    const offset = word.indexOf(text.charAt(at,),);
    // Absent from the word, or the word would start before the passage does.
    if ((offset === (-1)) || (offset > at))
      return false;
    return text.startsWith(
      word,
      at - offset,
    );
  },);
}

/**
 How many third-person pronouns an original passage writes in Han
 characters: 他, 她, 祂 and 其 where they name a person.

 @param text - original passage, comments already cut

 @returns Count of those pronouns

 @example
 ```ts
 hanThirdPersonCount({ text: '她说她的其他猫', },); // 2
 ```
 */
export function hanThirdPersonCount({ text, }: { readonly text: string; },): number {
  /**
   Pronouns counted so far.
   */
  let count = 0;
  for (let at = 0; at < text.length; at += 1) {
    if (!THIRD_PERSON_CHARACTERS.has(text.charAt(at,),))
      continue;
    if (insideNonPersonWord({
      text,
      at,
    },))
      continue;
    count += 1;
  }
  return count;
}

//endregion What the original passage writes about its people
