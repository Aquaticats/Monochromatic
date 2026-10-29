/**
 The model-facing source makes authored breaks visible without changing the
 canonical source or literals that merely look like Markdown break syntax.

 @module
 */

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { buildTranslateMessages, } from '../dist/final/node/index.mjs';

/** Canonical source text, including render-bearing trailing spaces. */
const SOURCE = '> 猫醒了。  \n> 鸟唱了。';

/**
 Reads the actual model-facing source message for a writer request.

 @param sourceText - exact source passed to the builder
 @returns User message, excluding the instruction that itself mentions br
 */
function displayedSource(sourceText: string,): string {
  /** Explicit absence comes from the calling stage, not an empty-string guess. */
  const plan = buildTranslateMessages({ sourceText, existingText: '', incumbentKind: 'absent', lineStructured: false, },);
  return plan.messages.filter(function isUser(message,): boolean {
    return message.role === 'user';
  },).map(function text(message,): string {
    return messageText({ message, },);
  },).join('\n',);
}

await describe({
  name: 'source break presentation before translation',
  children: [
    it({
      name: 'DISPLAYS parser-confirmed hard breaks as visible br in the existing source block',
      fn: async () => {
        expect(displayedSource(SOURCE,),).toContain('> 猫醒了。<br/>\n> 鸟唱了。',);
        expect(displayedSource(SOURCE,),).not.toContain(SOURCE,);
        expect(SOURCE,).toBe('> 猫醒了。  \n> 鸟唱了。',);
      },
    },),
    it({
      name: 'KEEPS soft wraps, existing br, inline code and fenced code verbatim',
      fn: async () => {
        /** Only the last paragraph has an actual Markdown break node. */
        const sourceText = '猫醒了。\n鸟唱了。\n\n`猫  鸟`<br/>\n\n```txt\n猫  \n鸟\n```\n\n猫睡了。  \n鸟飞了。';
        expect(displayedSource(sourceText,),).toContain('猫醒了。\n鸟唱了。',);
        expect(displayedSource(sourceText,),).toContain('`猫  鸟`<br/>',);
        expect(displayedSource(sourceText,),).toContain('```txt\n猫  \n鸟\n```',);
        expect(displayedSource(sourceText,),).toContain('猫睡了。<br/>\n鸟飞了。',);
      },
    },),
    it({
      name: 'KEEPS archive-backed and unknown-presence source presentation unchanged',
      fn: async () => {
        /** Empty wording alone is insufficient authority to transform this view. */
        const unknown = buildTranslateMessages({ sourceText: SOURCE, existingText: '', },);
        /** Even an absent fallback must not replace real archive authority. */
        const backed = buildTranslateMessages({ sourceText: SOURCE, existingText: 'The cat wakes.', incumbentKind: 'absent', },);
        expect(JSON.stringify(unknown.messages,),).toContain(JSON.stringify(SOURCE,).slice(1, -1,),);
        expect(JSON.stringify(backed.messages,),).toContain(JSON.stringify(SOURCE,).slice(1, -1,),);
      },
    },),
    ...['\n', '\r\n', '\r',].map(function ending(lineEnding,) {
      return it({
        name: `PRESERVES line ending ${JSON.stringify(lineEnding,)} for either hard-break spelling`,
        fn: async () => {
          expect(displayedSource(`猫。  ${lineEnding}鸟。`,),).toContain(`猫。<br/>${lineEnding}鸟。`,);
          expect(displayedSource(`猫。\\${lineEnding}鸟。`,),).toContain(`猫。<br/>${lineEnding}鸟。`,);
        },
      },);
    },),
    it({
      name: 'LEAVES comment and JSX attribute contents untouched while converting actual breaks',
      fn: async () => {
        /** MDX expressions and HTML comments are not prose break nodes. */
        const sourceText = '<!-- 猫  \n鸟 -->\n\n<Card label={`猫  \n鸟`} />\n\n猫。  \n鸟。';
        expect(displayedSource(sourceText,),).toContain('<!-- 猫  \n鸟 -->',);
        expect(displayedSource(sourceText,),).toContain('<Card label={`猫  \n鸟`} />',);
        expect(displayedSource(sourceText,),).toContain('猫。<br/>\n鸟。',);
      },
    },),
    it({
      name: 'LEAVES unreadable source and front matter unchanged rather than guessing offsets',
      fn: async () => {
        expect(displayedSource('<Cat value={',),).toContain('<Cat value={',);
        /** Metadata has its own syntax contract, not a Markdown body view. */
        const plan = buildTranslateMessages({ sourceText: SOURCE, existingText: '', incumbentKind: 'absent', syntax: 'front-matter', },);
        expect(JSON.stringify(plan.messages,),).toContain(JSON.stringify(SOURCE,).slice(1, -1,),);
      },
    },),
  ],
},);
