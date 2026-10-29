import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { $, } from './index.ts';

await describe({
  name: $.name,
  children: [
    it({
      name: 'creates a simple element with text content',
      fn: async () => {
        expect($({ tag: 'title', text: 'My Feed', },),).toBe('<title>My Feed</title>',);
      },
    },),
    it({
      name: 'self-closes elements with no content',
      fn: async () => {
        expect($({ tag: 'link', },),).toBe('<link />',);
      },
    },),
    it({
      name: 'self-closes elements with attrs but no content',
      fn: async () => {
        expect($({ tag: 'link', attrs: { href: 'https://example.com', }, },),).toBe(
          '<link href="https://example.com" />',
        );
      },
    },),
    it({
      name: 'does not self-close when text is provided',
      fn: async () => {
        expect($({ tag: 'name', text: '', },),).toBe('<name></name>',);
      },
    },),
    it({
      name: 'does not self-close when raw is provided',
      fn: async () => {
        expect($({ tag: 'content', raw: '', },),).toBe('<content></content>',);
      },
    },),
    it({
      name: 'does not self-close when children array is non-empty',
      fn: async () => {
        expect($({ tag: 'items', children: [$({ tag: 'item', text: 'a', },),], },),).toBe(
          '<items><item>a</item></items>',
        );
      },
    },),
    it({
      name: 'self-closes when children array is empty',
      fn: async () => {
        expect($({ tag: 'items', children: [], },),).toBe('<items />',);
      },
    },),
    it({
      name: 'escapes text content',
      fn: async () => {
        expect($({ tag: 'content', text: 'x < y & z > w', },),).toBe(
          '<content>x &lt; y &amp; z &gt; w</content>',
        );
      },
    },),
    it({
      name: 'escapes double quotes in text',
      fn: async () => {
        expect($({ tag: 'title', text: 'say "hello"', },),).toBe(
          '<title>say &quot;hello&quot;</title>',
        );
      },
    },),
    it({
      name: 'escapes single quotes (apostrophes) in text',
      fn: async () => {
        expect($({ tag: 'title', text: "it's", },),).toBe(
          '<title>it&apos;s</title>',
        );
      },
    },),
    it({
      name: 'escapes attribute values',
      fn: async () => {
        expect($({ tag: 'link', attrs: { href: 'a&b=c<d', }, },),).toBe(
          '<link href="a&amp;b=c&lt;d" />',
        );
      },
    },),
    it({
      name: 'does not escape raw content',
      fn: async () => {
        const inner = '<nested>raw & unescaped</nested>';
        expect($({ tag: 'wrapper', raw: inner, },),).toBe(
          `<wrapper>${inner}</wrapper>`,
        );
      },
    },),
    it({
      name: 'renders multiple attributes in order',
      fn: async () => {
        expect($({
          tag: 'entry',
          attrs: { id: '1', type: 'post', },
        },),)
          .toBe('<entry id="1" type="post" />',);
      },
    },),
    it({
      name: 'renders namespaced tag names',
      fn: async () => {
        expect($({ tag: 'atom:title', text: 'Feed', },),).toBe(
          '<atom:title>Feed</atom:title>',
        );
      },
    },),
    it({
      name: 'renders namespaced attributes',
      fn: async () => {
        expect($({
          tag: 'feed',
          attrs: { 'xmlns:atom': 'http://www.w3.org/2005/Atom', },
          children: [$({ tag: 'atom:title', text: 'Feed', },),],
        },),)
          .toBe(
          '<feed xmlns:atom="http://www.w3.org/2005/Atom"><atom:title>Feed</atom:title></feed>',
          );
      },
    },),
    it({
      name: 'concatenates text, raw, and children in order',
      fn: async () => {
        expect($({
          tag: 'mixed',
          text: 'hello',
          raw: '<b>world</b>',
          children: ['<i>!</i>',],
        },),)
          .toBe('<mixed>hello<b>world</b><i>!</i></mixed>',);
      },
    },),
    it({
      name: 'concatenates multiple children',
      fn: async () => {
        expect($({
          tag: 'list',
          children: [
            $({ tag: 'item', text: 'a', },),
            $({ tag: 'item', text: 'b', },),
            $({ tag: 'item', text: 'c', },),
          ],
        },),)
          .toBe('<list><item>a</item><item>b</item><item>c</item></list>',);
      },
    },),
    it({
      name: 'handles deeply nested elements',
      fn: async () => {
        const result = $({
          tag: 'root',
          children: [
            $({
              tag: 'parent',
              children: [
                $({ tag: 'child', text: 'leaf', },),
              ],
            },),
          ],
        },);
        expect(result,).toBe('<root><parent><child>leaf</child></parent></root>',);
      },
    },),
    it({
      name: 'handles RSS-like structure',
      fn: async () => {
        const result = $({
          tag: 'rss',
          attrs: { version: '2.0', },
          children: [
            $({
              tag: 'channel',
              children: [
                $({ tag: 'title', text: 'My Blog', },),
                $({ tag: 'link', attrs: { href: 'https://example.com', }, },),
                $({
                  tag: 'item',
                  children: [
                    $({ tag: 'title', text: 'First Post', },),
                    $({ tag: 'description', text: 'Hello <world>', },),
                  ],
                },),
              ],
            },),
          ],
        },);
        expect(result,).toBe(
          '<rss version="2.0"><channel><title>My Blog</title><link href="https://example.com" /><item><title>First Post</title><description>Hello &lt;world&gt;</description></item></channel></rss>',
        );
      },
    },),
  ],
},);
