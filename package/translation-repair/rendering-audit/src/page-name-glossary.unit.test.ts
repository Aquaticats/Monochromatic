/**
 Guards class seventy-one (2026-09-19): the author's Han handle reached the
 page as a plain romanization in one line of dialogue while the archive
 renders the same handle as a stylised form in its link text and in every
 other line, because no sheet told the bench how this page renders that name.
 The source's `[铨](url)` and the archive's `[Rendered](url)` share an href,
 and the source's signature lines pair with the archive's, so the page's own
 renderings are readable at preparation and every sheet carries them.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pageNameLines as allNameLines,
  pairedPageNames,
} from '../dist/final/node/index.mjs';

/**
 Lines of the block that pairs links, signatures and headings, which these
 cases are about; the block of names the archive spells inside its sentences
 has cases of its own (`page-name-prose.unit.test.ts`), and a cat fixture's
 link text is a name it spells too.

 @param arguments_ - texts and declared pairs, as `pageNameLines` takes them

 @returns Lines before the heading of the sentence-name block

 @example
 ```ts
 const lines = pageNameLines({ sourceText, targetText, },);
 ```
 */
function pageNameLines(arguments_: Parameters<typeof allNameLines>[0],): readonly string[] {
  /**
   Every line the glossary gives these texts.
   */
  const lines = allNameLines(arguments_,);
  /**
   Where the sentence-name block opens, minus one when it does not.
   */
  const spelled = lines.findIndex(function opensBlock(line,): boolean {
    return line.startsWith('NAMES THE ARCHIVE SPELLS',);
  },);
  return (spelled === (-1)) ? lines : lines.slice(
    0,
    spelled,
  );
}

await describe({
  name: 'a page names its people and linked titles as the archive renders them (class seventy-one)',
  children: [
    it({
      name: 'READS a same-href link whose Han text the archive renders, and a signature pair, into sheet lines',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '节选自[猫猫](https://example.invalid/maomao)猫日记\n\n——猫猫, 2024 年 12 月 17 日\n',
          targetText: 'Excerpt from [Maomao](https://example.invalid/maomao)’s cat diary\n\n——Maomao, December 17, 2024\n',
        },);
        expect(lines[0],).toContain('NAMES AND LINKED TEXT',);
        expect(lines,).toContain('- 猫猫 (link text, https://example.invalid/maomao): "Maomao"',);
        expect(lines.length,).toBe(2,);
      },
    },),
    it({
      name: 'READS a signature the archive renders under another spelling, once per distinct name',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '它睡了。\n\n——锦猫, 2025 年 2 月 10 日\n\n它醒了。\n\n——锦猫, 2025 年 3 月 1 日\n',
          targetText: 'It sleeps.\n\n——Jinmao, February 10, 2025\n\nIt wakes.\n\n——Jinmao, March 1, 2025\n',
        },);
        expect(lines,).toEqual([
          'NAMES AND LINKED TEXT THE ARCHIVE RENDERS ON THIS PAGE (render the same person or title the same way everywhere; a declared name inside a title takes its declared form):',
          '- 锦猫 (signature): "Jinmao"',
        ],);
      },
    },),
    it({
      name: 'READS a heading the archive renders otherwise, aligned by order when both documents carry the same '
        + 'count, as evidence the judges weigh (owner, 2026-09-21: the judges keep deciding headings; the literal '
        + 'reading winning is a world-knowledge gap to close another way)',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '## 左右\n\n猫在门口犹豫。\n\n## 回见\n\n猫走了。\n',
          targetText: '## Conflict\n\nThe cat hesitates at the door.\n\n## Farewell\n\nThe cat leaves.\n',
        },);
        expect(lines,).toContain('- 左右 (heading): "Conflict"',);
        expect(lines,).toContain('- 回见 (heading): "Farewell"',);
        expect(lines.length,).toBe(3,);
        expect(pageNameLines({
          sourceText: '## 左右\n\n猫在门口犹豫。\n\n## 回见\n\n猫走了。\n',
          targetText: '## Conflict\n\nThe cat hesitates at the door.\n',
        },),).toEqual([],);
        // A heading the archive left in the original's words renders nothing.
        expect(pageNameLines({
          sourceText: '## 左右\n\n猫在门口犹豫。\n',
          targetText: '## 左右\n\nThe cat hesitates at the door.\n',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'READS AN HTML HEADING the archive renders otherwise, paired among the HTML headings alone, so one '
        + 'HTML heading only one side carries costs no Markdown heading its pair (ledger X20)',
      fn: async () => {
        /**
         Page whose archive carries both kinds of heading.
         */
        const both = pageNameLines({
          sourceText: '## 左右\n\n<h3 align="center">猫之歌</h3>\n\n猫唱歌。\n',
          targetText: '## Conflict\n\n<h3 align="center">Song of the Cat</h3>\n\nThe cat sings.\n',
        },);
        /**
         Page whose archive dropped the HTML heading.
         */
        const htmlDropped = pageNameLines({
          sourceText: '## 左右\n\n<h3 align="center">猫之歌</h3>\n\n猫唱歌。\n',
          targetText: '## Conflict\n\nThe cat sings.\n',
        },);
        /**
         Page whose HTML heading stands before its Markdown one.
         */
        const htmlFirst = pageNameLines({
          sourceText: '<h3 align="center">猫之歌</h3>\n\n## 左右\n\n猫唱歌。\n',
          targetText: '<h3 align="center">Song of the Cat</h3>\n\n## Conflict\n\nThe cat sings.\n',
        },);
        expect({
          htmlPaired: both.includes('- 猫之歌 (heading): "Song of the Cat"',),
          markdownPaired: both.includes('- 左右 (heading): "Conflict"',),
          markdownKept: htmlDropped.includes('- 左右 (heading): "Conflict"',),
          htmlUnpaired: htmlDropped.some(function namesSong(line,): boolean {
            return line.startsWith('- 猫之歌',);
          },),
          inPageOrder: htmlFirst.filter(function isHeadingLine(line,): boolean {
            return line.includes(' (heading): ',);
          },),
        },).toEqual({
          htmlPaired: true,
          markdownPaired: true,
          markdownKept: true,
          htmlUnpaired: false,
          inPageOrder: [
            '- 猫之歌 (heading): "Song of the Cat"',
            '- 左右 (heading): "Conflict"',
          ],
        },);
      },
    },),
    it({
      name: 'SAYS a linked title that names the person takes the declared form inside it (class eighty-six, 2026-09-22: the archive\'s other form of the name inside a blog title carried as the title\'s authority against the declared form)',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '[永别了，猫猫。](https://example.invalid/farewell)\n',
          targetText: '[Farewell, Kitty.](https://example.invalid/farewell)\n',
          declared: [{ source: '猫猫', rendering: 'Maomao', },],
        },);
        // The title line still carries the archive's rendering, and now says
        // which name inside it the declared form settles.
        expect(lines,).toContain(
          '- 永别了，猫猫。 (link text, https://example.invalid/farewell): "Farewell, Kitty."; names 猫猫, declared "Maomao": the declared form inside the title, the archive\'s words for the rest',
        );
        expect(lines[0],).toContain('a declared name inside a title takes its declared form',);
      },
    },),
    it({
      name: 'KEEPS a linked title\'s line bare where the archive already renders the declared name inside it, or the title names nobody declared',
      fn: async () => {
        const rendered = pageNameLines({
          sourceText: '[永别了，猫猫。](https://example.invalid/farewell)\n',
          targetText: '[Farewell, Maomao.](https://example.invalid/farewell)\n',
          declared: [{ source: '猫猫', rendering: 'Maomao', },],
        },);
        expect(rendered,).toContain('- 永别了，猫猫。 (link text, https://example.invalid/farewell): "Farewell, Maomao."',);
        const nobody = pageNameLines({
          sourceText: '[永别了，猫猫。](https://example.invalid/farewell)\n',
          targetText: '[Farewell, Kitty.](https://example.invalid/farewell)\n',
          declared: [{ source: '橘猫', rendering: 'Ginger', },],
        },);
        expect(nobody,).toContain('- 永别了，猫猫。 (link text, https://example.invalid/farewell): "Farewell, Kitty."',);
      },
    },),
    it({
      name: 'READS THE NOTE AS THE LINK-NAME FLOOR DOES (ledger B23): a declared form only inside a longer word is '
        + 'not rendered, an escaped handle is, and a Latin source form inside a longer word names no one',
      fn: async () => {
        expect(pageNameLines({
          sourceText: '[\u{65E9}\u{5B89}\u{FF0C}\u{6C64}\u{59C6}\u{3002}](https://example.invalid/tom)\n',
          targetText: '[Good morning, Tomcat.](https://example.invalid/tom)\n',
          declared: [{ source: '\u{6C64}\u{59C6}', rendering: 'Tom', },],
        },),).toContain(
          '- \u{65E9}\u{5B89}\u{FF0C}\u{6C64}\u{59C6}\u{3002} (link text, https://example.invalid/tom): "Good morning, Tomcat."; names \u{6C64}\u{59C6}, declared "Tom": the declared form inside the title, the archive\'s words for the rest',
        );
        expect(pageNameLines({
          sourceText: '[\u{6C38}\u{522B}\u{4E86}\u{FF0C}\u{732B}\u{732B}\u{3002}](https://example.invalid/farewell)\n',
          targetText: `${String.raw`[Farewell, Mittens\_Cat.](https://example.invalid/farewell)`}\n`,
          declared: [{ source: '\u{732B}\u{732B}', rendering: 'Mittens_Cat', },],
        },),).toContain(
          String.raw`- 永别了，猫猫。 (link text, https://example.invalid/farewell): "Farewell, Mittens\_Cat."`,
        );
        expect(pageNameLines({
          sourceText: '[Tomcat\u{7684}\u{65E9}\u{9910}](https://example.invalid/tomcat)\n',
          targetText: '[Tomcat\'s breakfast](https://example.invalid/tomcat)\n',
          declared: [{ source: 'Tom', rendering: 'Thomas', },],
        },),).toContain(
          '- Tomcat\u{7684}\u{65E9}\u{9910} (link text, https://example.invalid/tomcat): "Tomcat\'s breakfast"',
        );
      },
    },),
    it({
      name: 'LEAVES OUT a link the archive keeps in the same words, a sentence-long link text, an href only one side '
        + 'carries, and signatures whose counts differ between the documents',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '[Maomao](https://example.invalid/same) and '
            + '[猫猫说：今天的太阳很暖，很软吧，要是明天还这样晴，一定陪你晒到傍晚](https://example.invalid/long) and '
            + '[白猫](https://example.invalid/source-only)\n\n——橘猫, 2024 年 1 月 1 日\n\n——黑猫, 2024 年 1 月 2 日\n',
          targetText: '[Maomao](https://example.invalid/same) and '
            + '[Maomao said the sun was warm](https://example.invalid/long)\n\n——Ginger, January 1, 2024\n',
        },);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'READS ONLY WHAT THE PAGE SHOWS: a heading or link inside the front matter, an HTML or JSX comment or '
        + 'a code fence neither pairs nor shifts the heading order (ledger E10)',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: [
            '---',
            'title: "[黑猫](https://example.invalid/fm)"',
            '---',
            '',
            '<!--',
            '# 旧标题',
            '[白猫](https://example.invalid/comment)',
            '-->',
            '',
            '{/* [花猫](https://example.invalid/jsx) */}',
            '',
            '```',
            '# 代码里的猫',
            '```',
            '',
            '## 午睡',
            '',
            '猫睡了。',
          ].join('\n',),
          targetText: [
            '---',
            'title: "[Black Cat](https://example.invalid/fm)"',
            '---',
            '',
            '<!--',
            '[Snowball](https://example.invalid/comment)',
            '-->',
            '',
            '{/* [Patches](https://example.invalid/jsx) */}',
            '',
            '## The Nap',
            '',
            'The cat slept.',
          ].join('\n',),
        },);
        expect(lines.slice(1,),).toEqual(['- 午睡 (heading): "The Nap"',],);
      },
    },),
    it({
      name: 'STOPS READING LINKS where a bracket never closes: a link before the break is still read, and the scan '
        + 'never finds a link past an opening bracket with no `]` anywhere after it',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '[小狗](https://example.invalid/dog)\n\n猫 [未闭合',
          targetText: '[Puppy](https://example.invalid/dog)\n\nCat text',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 小狗 (link text, https://example.invalid/dog): "Puppy"',
        ],);
      },
    },),
    it({
      name: 'STOPS READING LINKS where a destination never closes: a link before the break is still read, and the '
        + 'scan never finds a link whose `)` is missing to the end of the text',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '[小狗](https://example.invalid/dog)\n\n[猫猫](https://example.invalid/missing-paren',
          targetText: '[Puppy](https://example.invalid/dog)\n\n[Cat](https://example.invalid/missing-paren',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 小狗 (link text, https://example.invalid/dog): "Puppy"',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT AN IMAGE, reading past its `!` marker to the next real link',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '![猫猫](https://example.invalid/cat)\n\n[小鸟](https://example.invalid/bird)',
          targetText: '![Maomao](https://example.invalid/cat)\n\n[Bird](https://example.invalid/bird)',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 小鸟 (link text, https://example.invalid/bird): "Bird"',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT A LINK WHOSE TEXT CARRIES A NESTED BRACKET, reading only the empty-text link CommonMark '
        + 'finds at the inner bracket, which pairs nothing, and then the next real link',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '[猫[](https://example.invalid/nested)\n\n[小鸟](https://example.invalid/bird)',
          targetText: '[Cat[](https://example.invalid/nested)\n\n[Bird](https://example.invalid/bird)',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 小鸟 (link text, https://example.invalid/bird): "Bird"',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT A LINK WHOSE SOURCE TEXT CARRIES NO HAN, even where the archive renders it otherwise',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '[Tomcat](https://example.invalid/tomcat) and [小猫](https://example.invalid/kitten)',
          targetText: '[Tom](https://example.invalid/tomcat) and [Kitten](https://example.invalid/kitten)',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 小猫 (link text, https://example.invalid/kitten): "Kitten"',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT A SIGNATURE THE ARCHIVE KEEPS UNCHANGED OR WHOSE ORIGINAL NAME CARRIES NO HAN, pairing only '
        + 'the one the archive renders otherwise',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '它伸懒腰。\n\n——阿橘, 2024 年 1 月 1 日\n\nTom 挥手。\n\n——Tom, 2024 年 1 月 2 日\n\n它打盹。\n\n——锦猫, 2024 年 1 月 3 日',
          targetText: 'It stretches.\n\n——阿橘, January 1, 2024\n\nTom waves.\n\n——Thomas, January 2, 2024\n\nIt naps.\n\n——Jinmao, January 3, 2024',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 锦猫 (signature): "Jinmao"',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT A HEADING WHOSE SOURCE TEXT CARRIES NO HAN, pairing only the one the archive renders otherwise',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '## Morning\n\n天亮了。\n\n## 左右\n\n猫在门口犹豫。\n',
          targetText: '## Dawn\n\nIt dawns.\n\n## Conflict\n\nThe cat hesitates at the door.\n',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 左右 (heading): "Conflict"',
        ],);
      },
    },),
    it({
      name: 'STOPS READING AN HTML HEADING THAT NEVER CLOSES, keeping a well-formed heading read before it',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '<h3 align="center">猫之歌</h3>\n\n<h2>未闭合标题',
          targetText: '<h3 align="center">Song of the Cat</h3>\n\n<h2>Unfinished',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 猫之歌 (heading): "Song of the Cat"',
        ],);
      },
    },),
    it({
      name: 'SWALLOWS EVERYTHING AFTER AN UNCLOSED JSX COMMENT to the document\'s end, including a heading inside it',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '## 外面\n\n猫在院子里玩。\n\n{/* 旧内容\n## 旧标题\n\n这些猫不存在。\n',
          targetText: '## Outside\n\nThe cat plays in the yard.\n',
        },);
        expect(lines.slice(1,),).toEqual([
          '- 外面 (heading): "Outside"',
        ],);
      },
    },),
    it({
      name: 'PAIRS A LINK WITH THE ARCHIVE LINK AT ITS PLACE among the links to one href, where the archive links '
        + 'that href under more than one text and as often as the original does',
      fn: async () => {
        const lines = pageNameLines({
          sourceText: '[咪咪](https://example.invalid/mimi)的日记在[这里](https://example.invalid/mimi)。',
          targetText: 'The diary of [Mimi](https://example.invalid/mimi) is [here](https://example.invalid/mimi).',
        },);
        // Keyed by href alone, both links took the archive's last text for it.
        expect(lines.slice(1,),).toEqual([
          '- 咪咪 (link text, https://example.invalid/mimi): "Mimi"',
          '- 这里 (link text, https://example.invalid/mimi): "here"',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT A LINK WHOSE HREF THE ARCHIVE LINKS UNDER MORE THAN ONE TEXT a different number of times '
        + 'than the original, since which of them renders it cannot be read, and READS one the archive links '
        + 'more often under a single text',
      fn: async () => {
        expect(pageNameLines({
          sourceText: '[咪咪](https://example.invalid/mimi)睡了。',
          targetText: '[Mimi](https://example.invalid/mimi) slept; see [her page](https://example.invalid/mimi).',
        },),).toEqual([],);
        expect(pageNameLines({
          sourceText: '[咪咪](https://example.invalid/mimi)睡了。',
          targetText: '[Mimi](https://example.invalid/mimi) slept, and [Mimi](https://example.invalid/mimi) woke.',
        },)
          .slice(1,),).toEqual([
            '- 咪咪 (link text, https://example.invalid/mimi): "Mimi"',
          ],);
      },
    },),
    it({
      name: 'LEAVES OUT A SOURCE TEXT THE ARCHIVE RENDERS TWO WAYS, by two links to different hrefs or by two '
        + 'headings, since the sheet would give one rendering as the page\'s and say nothing of the other, and '
        + 'READS the page\'s other names and their paired set without it',
      fn: async () => {
        /**
         Original naming 猫猫 by two links and 简介 by two headings, and 左右 once.
         */
        const sourceText = '[猫猫](https://example.invalid/a)和[猫猫](https://example.invalid/b)。\n\n'
          + '## 简介\n\n猫睡了。\n\n## 简介\n\n猫醒了。\n\n## 左右\n\n猫走了。\n';
        /**
         Archive rendering each of the repeated texts two ways.
         */
        const targetText = '[Maomao](https://example.invalid/a) and [Mimi](https://example.invalid/b).\n\n'
          + '## Overview\n\nThe cat slept.\n\n## Summary\n\nThe cat woke.\n\n## Conflict\n\nThe cat left.\n';
        expect(pageNameLines({
          sourceText,
          targetText,
        },)
          .slice(1,),).toEqual([
            '- 左右 (heading): "Conflict"',
          ],);
        expect([...pairedPageNames({
          sourceText,
          targetText,
        },),],).toEqual(['左右',],);
      },
    },),
  ],
},);
