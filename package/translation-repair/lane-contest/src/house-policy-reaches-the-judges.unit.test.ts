/**
 Tests that the stages deciding what SHIPS are told what this corpus is
 written under.

 WHY THIS FILE EXISTS. Every producing sheet splices `HOUSE_POLICY_BLOCK`, and
 for a long time no judging sheet did. The judges were therefore the only
 stages that had never been told about reader protection, while criterion one
 asks them for every proposition of the original with nothing left out. A
 candidate keeping a suicide method vague, exactly as the corpus's own rule
 demands, reads as an omission to a judge with only the numbered list.

 AND THE TENSE HALF, which was measured rather than imagined. Of the four
 slates a judge refused ENTIRELY on the consolidation bed, three were refused
 over tense, one saying every candidate had altered the time reference by
 rendering a tenseless Chinese copula in the past. The rule that answers that
 lived in the critic's sheet and reached no judge.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildCandidateSelectMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import { CONTEST_POLICY, } from '../dist/final/node/index.mjs';

/**
 System half of one judge exchange, which is where standing rules live.
 */
const system = buildCandidateSelectMessages({
  task: 'Each candidate is a rendering of the passage below.',
  criteria: ['Complete coverage: every proposition of the ORIGINAL is rendered, nothing left out.',],
  evidence: [
    {
      label: 'ORIGINAL (Chinese)',
      text: '猫在窗台上睡觉。',
    },
  ],
  rendered: ['The cat sleeps on the windowsill.', 'The cat naps on the sill.',],
},)
  .filter(function isSystem(message,): boolean {
    return message.role === 'system';
  },)
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

await describe({
  name: 'house policy reaches the judges',
  children: [
    it({
      name: 'TELLS a judge that reader protection outranks completeness',
      fn: async () => {
        // The contradiction this closes: criterion one asks for every
        // proposition, and the corpus deliberately keeps some of them vague.
        expect(system.includes('Reader protection outranks completeness',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge that the test is replicability: an immediate medical cause such as shock gives '
        + 'a reader nothing to copy and stays, as do the date, place and age (the owner, 2026-09-02, '
        + 'correcting the first reading of the Toka_ls page\'s "hemorrhagic shock")',
      fn: async () => {
        expect(system.includes('The test is replicability',),).toBe(true,);
        expect(system.includes('An immediate medical cause of death (shock, cardiac arrest, respiratory failure) gives a reader nothing to copy',),).toBe(true,);
        expect(system.includes('as do the date, the place and the age',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge that euthanasia stays as the original states it, because it names a legal, '
        + 'medically administered path a reader cannot take (the owner, 2026-09-02, on XIEPT2\'s 「安乐死」)',
      fn: async () => {
        expect(system.includes('Euthanasia or medically assisted dying',),).toBe(true,);
        expect(system.includes('inaccessibility is the whole reason the vagueness exists',),).toBe(true,);
      },
    },),
    it({
      name: 'RANKS a house rule above a numbered criterion',
      fn: async () => {
        // Judges apply the numbered list literally: three whole-slate refusals
        // were measured doing exactly that, so precedence has to be stated
        // rather than implied by adjacency.
        expect(system.includes('THE HOUSE RULE WINS',),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES to let a forced tense count as a change of time',
      fn: async () => {
        // Chinese marks no tense and English cannot avoid one, so the choice is
        // required rather than added.
        expect(system.includes('Chinese marks no tense',),).toBe(true,);
        expect(system.includes('never a change to the time',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge that an unstated subject is not a neutral pronoun (Toka_ls slice 9, read '
        + '2026-09-02: eight of eight ballots chose "they" for a subjectless sentence on a page that says '
        + 'she, and the contest called the gendered pronoun an invention)',
      fn: async () => {
        expect(system.includes('AN UNSTATED SUBJECT IS NOT A NEUTRAL PRONOUN',),).toBe(true,);
        expect(system.includes('has invented nothing',),).toBe(true,);
        expect(system.includes('a neutral pronoun there on a page that uses her or his pronoun has made the WRONG choice',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge what English makes of the neutral pronoun TA, Ta or ta (SS3B_0016 slice 5, '
        + 'read 2026-09-04: three of five contest ballots kept a bare "Ta" as "the original\'s neutral '
        + 'Ta" on a page that says they everywhere else)',
      fn: async () => {
        expect(system.includes('spelled TA, Ta or ta',),).toBe(true,);
        expect(system.includes('English renders it as singular they',),).toBe(true,);
        expect(system.includes('an untranslated word, not a preserved choice',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge that a pronoun the ORIGINAL writes is rendered as written where it stands, and that a '
        + 'written TA is not an unstated subject (class eighty-one, XingZ621 slice 14, 2026-09-22: Part One '
        + 'writes TA forty times on a page whose other parts write 她 fifty-one times, and the bench split '
        + 'between "she" on the identity line and "they" on the TA rule until the entry stopped)',
      fn: async () => {
        expect(system.includes('rendered as written where it stands',),).toBe(true,);
        expect(system.includes('a writer who chose TA beside a page of 她 chose it',),).toBe(true,);
        expect(system.includes('A TA THE ORIGINAL WRITES IS NOT AN UNSTATED SUBJECT',),).toBe(true,);
        expect(system.includes('resolving a written TA to she or he has made the WRONG choice',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge how a handle with no declared, archive or corpus rendering is written: pinyin in '
        + 'capitalized groups of two syllables (owner, 2026-09-22), with its literal meaning in parentheses at its '
        + 'first appearance (class eighty-three, '
        + 'XingZ622, 2026-09-22: 锦心 shipped in Han, 洁澄天奏 as "Jiecheng Tianzou" and "洁澄天奏Official", 雨狸 as '
        + '"Yu Li" and "雨狸" on one page; the judges filled the gap with "keep the original form")',
      fn: async () => {
        expect(system.includes('romanized as the handle is read',),).toBe(true,);
        expect(system.includes('capitalized groups of two syllables',),).toBe(true,);
        expect(system.includes('never one joined word',),).toBe(true,);
        expect(system.includes('literal meaning in parentheses the first time it appears on the page',),).toBe(true,);
        expect(system.includes('never left in Han, never rendered by its meaning alone',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge that 那些秋叶 is the site\'s own name and reads One Among Us (SS3B_0016 slice 5, read '
        + '2026-09-04: the gate kept the name 4 to 3 over a literal rendering, arguing from the archive alone; '
        + 'the archives of all five entries naming it agree)',
      fn: async () => {
        expect(system.includes('那些秋叶 is this site\'s own name in Chinese',),).toBe(true,);
        expect(system.includes('rendered "One Among Us" wherever it appears',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge what corner brackets become (SS3B_0016 slice 5 shipped 「One Among Us」 with the '
        + 'source\'s brackets around the English name)',
      fn: async () => {
        expect(system.includes('Corner brackets 「」 are Chinese quotation marks',),).toBe(true,);
        expect(system.includes('around a name or a term used to refer to its thing they are dropped',),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS a judge that a life on these pages is told in the past tense, and that an archive line left in '
        + 'the present is not the tense to keep (owner, 2026-09-21, on CuspariaKLSY\'s drift: "these people are '
        + 'already dead and obviously past tense is better English here")',
      fn: async () => {
        expect(system.includes('told in the past tense',),).toBe(true,);
        expect(system.includes('not the tense to keep',),).toBe(true,);
      },
    },),
    it({
      name: 'NEVER names the replaced text\'s own tense as the authority over the past-tense rule (class '
        + 'seventy-six, CuspariaKLSY4 slice 0: a contest judge kept the archive\'s present-tense life citing '
        + '"the tense rule that holds narrative in the present", reading the judge tail\'s "the tense the '
        + 'text being replaced had already chosen" over the house rule; the contest split 2 to 2 to 1 and the '
        + 'present tense shipped unendorsed)',
      fn: async () => {
        expect(system.includes('the text being replaced had already chosen',),).toBe(false,);
        expect(system.includes('the past tense the house rule sets for a life',),).toBe(true,);
        expect(CONTEST_POLICY.includes('the text being replaced had already chosen',),).toBe(false,);
        expect(CONTEST_POLICY.includes('the past tense the house rule sets for a life',),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS the criteria ahead of the policy in the sheet',
      fn: async () => {
        // The criteria decide; the policy qualifies them. A block arriving
        // first reads as the task.
        const criteria = system.indexOf('Complete coverage',);
        const policy = system.indexOf('Reader protection outranks completeness',);
        expect(criteria,).toBeGreaterThan(-1,);
        expect(policy,).toBeGreaterThan(criteria,);
      },
    },),
  ],
},);
