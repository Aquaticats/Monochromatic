# Regression-test a CSS refactor with a computed-style diff

Purpose: prove a stylesheet or class-name refactor changes nothing a visitor can see, deterministically,
on a static site served by two local servers (before and after).
Verified 2026-10-18 on the Xinyan Chen portfolio (six pages, 4 widths, about 1,300 element pseudos per page).

## Why not screenshots

Full-page screenshots of the same commit, taken twice, differed in 12 of 66 captures (up to 2 percent of pixels):
lazy images below the fold were sometimes unpainted, and an extra wait for every image to load and decode still left
7 differing captures. A probe that differs from itself cannot show "no difference".
Computed styles do not depend on paint timing, and a cascade flip, which is the refactor's real risk, shows up there
directly as a changed property value. Keep screenshots only to look at a difference the diff already found.

## Method

- Load each page at each width in two `agent-browser` sessions, one per server, and wait for network idle
  (plus any intro animation; computed `opacity` changes while it runs).
- In the page, walk `document.documentElement` and every descendant, and for each element and its `::before`,
  `::after` and `::marker`, serialise every standard property from `getComputedStyle`, sorted.
  Key each element by its DOM position (tag and child index per level), so renamed classes do not change keys.
- Return only a hash per key, then fetch full values for the keys whose hashes differ.
- Skip custom properties: a refactor renames them on purpose, and their effects already show in real properties.
- Strip `location.origin` from values: resolved `url()` values carry the server's port, which differs per side.
- States: `getComputedStyle` cannot see `:hover` or `:focus-visible` unless the browser is in that state, so
  `agent-browser hover <selector>` and `focus <selector>` each candidate element, then read the stateful properties
  of the element and its pseudo-elements. Check the selector exists on both sides first, and do not print the
  action's error text: it names the session, so it differs by side.

## Validate the probe before trusting a zero

1. Run it with the same server on both sides: it must print `summary: 0 differences`.
2. Positive control: copy the site, change one ordinary property (here `padding-block: 2rem` to `calc(2rem + 1px)`
   on a note) and one hover-only colour, serve the copy, and confirm both are reported.

## Chromium pitfall: auto margins read as 0px

In headless Chromium 149, `getComputedStyle(el).margin*` for an `auto`, percentage or `calc()` margin returns a value
stored on the layout fragment, not one derived from geometry.
On about one load in nine (75 of 660 samples, batches from 4 to 20 percent) it reports `0px` while the box is placed
correctly: on the About page at 1920 wide the portrait wave read `0px` against a real 240, and `marginRight` and
`marginInline` read `0px` too.
Two or more `loading="lazy"` images far below the viewport are necessary to trigger it (no lazy image gave no failure
in 100 loads, exactly one gave none in 180; a hand-built page with three lazy images did not reproduce it, so
something else is also needed).
A bad load stays bad: waiting for `document.fonts.ready`, animation frames, a second, scrolling, or forcing layout with
`offsetHeight` does not repair it, while a layout-dirtying style change or a resize does.
`getBoundingClientRect()` and `offsetLeft` are right in every bad load.

The Blink sources (`longhands_custom.cc` `MarginLeft`, `block_node.cc` `UpdateMarginPaddingInfoIfNeeded`, with its
`TODO(crbug.com/1353190)`) suggest a layout-cache path that resets auto margins to 0.
That is a reading of the source, not a traced build, and no matching Chromium issue was found.
The measurements, the variants tried and the open questions are in
`/home/user/temp/agent/chromium-flake/report.md`.

What the tools do about it:

- Read margins through `element.computedStyleMap()`, which returns the declared keyword (`auto`) on every load, and
  record each element's box from `getBoundingClientRect()`. A real shift of an auto-margined box still shows in the box.
- To get a used margin, derive it from geometry:
  `rect.left - (parent rect.left + parent.clientLeft + parent padding-left)`. Flex and grid items and transformed
  elements need their own handling, because the rect is post-transform.

## Chromium tool

```js
// Computed-style regression between two servers of the portfolio: every element and its ::before, ::after and
// ::marker, every standard property, at each width, plus hover and focus states of interactive elements.
// Usage: node style-diff.mjs <base-url> <candidate-url> [page ...]
// Exit code 1 when anything differs; prints each difference as element, pseudo, property, base and candidate value.
import { execFileSync } from 'node:child_process';

const [base, candidate, ...only] = process.argv.slice(2);
const PAGES = only.length ? only : ['index', 'about', 'contact', 'tales-for-safer-trails', 'menume', 'scotties'];
const WIDTHS = [390, 744, 1440, 1920];
// Session names; set STYLE_DIFF_SESSION to run two comparisons at once.
const BASE_SESSION = `${process.env.STYLE_DIFF_SESSION ?? 'style'}-base`;
const CAND_SESSION = `${process.env.STYLE_DIFF_SESSION ?? 'style'}-cand`;
// Interactive elements whose state rules the refactor rewrites; the first match of each selector is tested.
const STATES = [
  // Selector lists name a control by its old and new class, so renamed controls still match on both sides.
  'a[href="about.html"]:not(#portfolio-primary-navigation a)', '#portfolio-primary-navigation>a',
  '.portfolio-back-to-top-link, .back-to-top', '.phone-navigation-menu-toggle-button, .menu-toggle',
  '.home-page-project-case-study-link, .project .pill', '.contact-page-email-link, .email-link',
  'form button[type=submit]', 'a[href^="mailto:"]', 'a[href$=".pdf"]', 'main a[href$=".html"]', '[popovertarget]',
  'nav ul a', 'a[target="_blank"]', 'input', 'textarea',
];
const browser = (session, ...args) => execFileSync('agent-browser', ['--session', session, ...args], {
  encoding: 'utf8', timeout: 180_000, maxBuffer: 256 * 1024 * 1024,
});

const COLLECT = `(() => {
  const PSEUDOS = ['', '::before', '::after', '::marker'];
  const key = (element) => {
    const parts = [];
    for (let node = element; node && node !== document.documentElement; node = node.parentElement) {
      parts.unshift(node.tagName.toLowerCase() + ':' + [...node.parentElement.children].indexOf(node));
    }
    return parts.join('>');
  };
  const serialise = (element, pseudo) => {
    const style = getComputedStyle(element, pseudo || null);
    // getComputedStyle resolves auto margins to used pixels, and Chromium sometimes reports 0px for them on a laid-out
    // page; the typed computed value keeps auto as auto. The element's box (below) still catches a real shift.
    const typed = pseudo ? null : element.computedStyleMap();
    const out = [];
    for (let index = 0; index < style.length; index += 1) {
      const name = style[index];
      if (name.startsWith('--')) continue;
      const value = typed && name.startsWith('margin') ? String(typed.get(name)) : style.getPropertyValue(name);
      // Resolved url() values carry the server's origin, which differs between the two sides.
      out.push(name + ':' + value.replaceAll(location.origin, ''));
    }
    if (!pseudo) {
      const box = element.getBoundingClientRect();
      out.push('box:' + [box.x, box.y + scrollY, box.width, box.height].map((value) => value.toFixed(2)).join(' '));
    }
    return out.sort().join(';');
  };
  const hash = (text) => {
    let value = 2166136261;
    for (let index = 0; index < text.length; index += 1) value = Math.imul(value ^ text.charCodeAt(index), 16777619);
    return (value >>> 0).toString(36);
  };
  window.__styleDump = {};
  const result = {};
  for (const element of [document.documentElement, ...document.documentElement.querySelectorAll('*')]) {
    const id = element === document.documentElement ? 'html' : key(element);
    for (const pseudo of PSEUDOS) {
      const text = serialise(element, pseudo);
      window.__styleDump[id + pseudo] = text;
      result[id + pseudo] = hash(text);
    }
  }
  return JSON.stringify(result);
})()`;
const detail = (ids) => `JSON.stringify(Object.fromEntries(${JSON.stringify(ids)}.map((id) => [id, window.__styleDump[id]])))`;
const stateProbe = (selector) => `(() => {
  const element = document.querySelector(${JSON.stringify(selector)});
  if (!element) return 'none';
  return [getComputedStyle(element), getComputedStyle(element, '::before'), getComputedStyle(element, '::after')]
    .map((style) => ['color', 'background-color', 'text-decoration-line', 'outline-style', 'outline-color', 'outline-width',
      'outline-offset', 'box-shadow', 'opacity', 'border-color', 'cursor'].map((name) => style.getPropertyValue(name)).join('|'))
    .join(' / ');
})()`;

const load = (session, url, page, width) => {
  browser(session, 'set', 'viewport', String(width), '900', '1');
  browser(session, 'open', `${url}/${page}.html?s=${Date.now()}`);
  browser(session, 'wait', '--load', 'networkidle');
  // The viewport change can land after the page loads; measuring before it does reads the previous width's layout.
  browser(session, 'wait', '--fn', `innerWidth === ${width}`);
  // Home's intro animation ends at 7.25 s; computed opacity changes until then.
  browser(session, 'wait', page === 'index' ? '8500' : '800');
};
const parse = (output) => JSON.parse(JSON.parse(output));

let differences = 0;
for (const page of PAGES) {
  for (const width of WIDTHS) {
    load(BASE_SESSION, base, page, width);
    load(CAND_SESSION, candidate, page, width);
    const a = parse(browser(BASE_SESSION, 'eval', COLLECT));
    const b = parse(browser(CAND_SESSION, 'eval', COLLECT));
    const ids = [...new Set([...Object.keys(a), ...Object.keys(b)])].filter((id) => a[id] !== b[id]);
    if (ids.length) {
      const da = parse(browser(BASE_SESSION, 'eval', detail(ids.slice(0, 40))));
      const db = parse(browser(CAND_SESSION, 'eval', detail(ids.slice(0, 40))));
      for (const id of ids.slice(0, 40)) {
        const pa = new Map((da[id] ?? '').split(';').map((entry) => [entry.slice(0, entry.indexOf(':')), entry.slice(entry.indexOf(':') + 1)]));
        const pb = new Map((db[id] ?? '').split(';').map((entry) => [entry.slice(0, entry.indexOf(':')), entry.slice(entry.indexOf(':') + 1)]));
        for (const name of new Set([...pa.keys(), ...pb.keys()])) {
          if (pa.get(name) !== pb.get(name)) console.log(`${page}@${width} ${id} ${name}: ${pa.get(name)} -> ${pb.get(name)}`);
        }
      }
      if (ids.length > 40) console.log(`${page}@${width} ... ${ids.length - 40} more differing element pseudos`);
    }
    differences += ids.length;
    // Hover and focus states, measured on the live element after the browser applies each state.
    for (const selector of STATES) {
      for (const action of ['hover', 'focus']) {
        const present = [[BASE_SESSION], [CAND_SESSION]].map(([session]) =>
          browser(session, 'eval', `!!document.querySelector(${JSON.stringify(selector)})`).trim());
        if (present[0] !== present[1]) {
          differences += 1;
          console.log(`${page}@${width} ${selector}: present ${present[0]} -> ${present[1]}`);
          continue;
        }
        if (present[0] !== 'true') continue;
        const states = [[BASE_SESSION, base], [CAND_SESSION, candidate]].map(([session]) => {
          try {
            browser(session, action, selector);
          } catch (error) {
            // Report only that the action failed; the message names the session, which differs by side.
            console.error(`${session} ${action} ${selector}: ${String(error.message).split('\n')[0]}`);
            return `not ${action}able`;
          }
          browser(session, 'wait', '300');
          return browser(session, 'eval', stateProbe(selector)).trim();
        });
        if (states[0] !== states[1]) {
          differences += 1;
          console.log(`${page}@${width} ${action} ${selector}: ${states[0]} -> ${states[1]}`);
        }
      }
    }
    console.log(`checked ${page}@${width}: ${Object.keys(a).length} element pseudos, ${ids.length} differ`);
  }
}
browser(BASE_SESSION, 'close');
browser(CAND_SESSION, 'close');
console.log(`summary: ${differences} differences`);
process.exitCode = differences ? 1 : 0;
```

## Safari as the reference browser

Real Safari is the golden browser for the portfolio, and Chromium is not a gate: a rule can compute the same in both
and still differ in Safari.
The Safari tool below compares the same computed values, but drives Safari in a macOS guest through WebDriver, so
its harness differs from the Chromium one in four ways.
Verified 2026-10-18: base commit against itself gives 0 differences on all six pages, a copy with one padding and one
hover colour changed reports exactly those, and the refactor commit against its parent gives 0.

### Iframe harness

Safari's window cannot be phone width, and one WebDriver session is all it allows, so there are no two browsers.
The tool opens a blank harness page (`/@wt-harness/blank.html`) and puts base and candidate in two same-origin
iframes of the target width (390, 744, 1440, 1920), one above the other.
Media queries, `vw` and container units follow the iframe's width, so phone widths work.
Same-origin matters because the script reads each frame's `contentWindow` directly: one forwarded origin
(`http://localhost:4173` inside the guest) serves the main repository at `/` and any snapshot directory
`/home/user/temp/agent/wt-<name>` at `/@wt-<name>/`, so a prefix is all that tells base from candidate.
Prefixes are paths: `""` is the main repository, `@wt-base` a snapshot.
Take a snapshot of a commit without touching the repository: `git archive <commit> | tar -x -C wt-<name>`.
Compare the commit before a refactor, with any fixes the refactor must keep, against the refactor commit;
later intended changes on the same branch otherwise show as differences (comparing a base against `main` after two
unrelated fixes landed reported 1765 differences, all from those two fixes).

### Margins through Typed OM

Margins are read with `element.computedStyleMap().get('margin-left')` and so on, which keeps `auto` as the keyword,
and each element's `getBoundingClientRect()` box is added as a `box` property.
This is the same defence as in the Chromium pitfall: the used margin is not read from `getComputedStyle`, and a real
shift of an auto-margined box still shows in the box.
A change to one element's padding also moves its ancestors' `height`, `block-size`, `box`, `perspective-origin` and
`transform-origin`; read the first differing element of a run, not the last.

### Focus and hover probes

Each control is probed in each frame in turn: WebDriver scrolls it into view, hovers it with a pointer move, or focuses
it with `window.focus()` and `focus({ focusVisible: true })`, and the stateful properties are read back.
Three pitfalls made the first version report differences between two identical frames.

#### Stale pointer

After a hover probe the pointer stays where it was, and the next probe scrolls its control under
it. The second frame's focused link was also hovered (darker colour, underline), so the two frames differed. The tool
now parks the pointer on a 40 px pad fixed in the harness page's corner before every probe, and each probe records
`hover:`, `focus:`, `focus-visible:` and `hasFocus:` so the state is visible. A focus probe whose control is still
hovered is printed as `PROBE INVALID`.

#### Frame without focus

A frame occasionally does not hold the page's focus after switching to it (`hasFocus:false`),
and then even `focus({ focusVisible: true })` shows no outline. The tool focuses again, up to three times, and
reports `PROBE INVALID` if the frame never takes focus.

#### Unreachable controls

A control hidden at a width (the phone menu on a wide page) answers hover with
`element not interactable`. The error text is the same on both sides, so it compares equal, and a state rule on a
control hidden at that width is untested. Run `DEBUG_STATES=1` to print every probe and check which ones ran.

Environment switches: `WIDTHS=1920,390`, `ONLY_STATE='<substring of a selector>'`, `DEBUG_STATES=1` and
`MAX_ELEMENTS=<n>` (differing elements printed per page and width, 60 by default; the rest are only counted).

### Lock and drivers

Safari allows one WebDriver session at a time, and other sessions use it too, so every run holds a lock for the run and
no longer:

```sh
# run from the Linux host; the lock file is created on first use
flock /home/user/temp/agent/vm-safari.lock node safari-style-diff.mjs "@wt-base" "@wt-refactor"
```

The driver is reached on Linux port 4446. A run of all six pages takes 15 to 30 minutes, so start it in the
background and wait for it in one command that ends when the output has its `summary:` line.
The tool sets the guest display mode first (`displayplacer` over `ssh m1`), so the window is the size it expects.
Safari Technology Preview in the same guest has its own driver on Linux port 4450 and runs next to the 4446 session
(proven with a session that loaded the home page while six comparisons used 4446). It needed `safaridriver --enable`
run once in the guest, reports `browserName` "Safari Technology Preview", version 27.0 (platform build 26A428), and
wants its own lock, `/home/user/temp/agent/vm-stp.lock`.
The tool uses port 4446 and `browserName: 'safari'`; changing both constants should be all a Technology Preview run
needs (not yet tried with the tool).
Technology Preview is not the golden browser; use it for extra coverage, not for the gate.

## Safari tool

```js
// Computed-style regression in real Safari (the UTM macOS guest, through WebDriver on local port 4446).
// Base and candidate load side by side as same-origin iframes of the target width inside one harness page, so phone
// widths work although Safari's window cannot be that narrow; media queries and vw follow the iframe's width.
// Usage: node safari-style-diff.mjs <base-prefix> <candidate-prefix> [page ...]
//   prefixes are paths on the forwarded origin http://localhost:4173, for example "" (the main repo) or "@wt-layer"
// Prints each difference as page@width, element, pseudo, property, base and candidate value; exit code 1 if any.
import { execFileSync } from 'node:child_process';
import { setTimeout as wait } from 'node:timers/promises';

const DRIVER = 'http://127.0.0.1:4446';
const ORIGIN = 'http://localhost:4173';
const GUEST_DISPLAY = 'id:35B36D67-6BAC-47B7-98B5-07877849011C';
const [basePrefix, candidatePrefix, ...only] = process.argv.slice(2);
const PAGES = only.length ? only : ['index', 'about', 'contact', 'tales-for-safer-trails', 'menume', 'scotties'];
// How many differing element pseudos print per page and width (the count of the rest is always printed).
const LIMIT = Number(process.env.MAX_ELEMENTS ?? 60);
const WIDTHS = process.env.WIDTHS ? process.env.WIDTHS.split(',').map(Number) : [390, 744, 1440, 1920];
// Selector lists name a control by its old and new class, so renamed controls still match on both sides.
const STATES = [
  'a[href="about.html"]:not(#portfolio-primary-navigation a)', '#portfolio-primary-navigation>a',
  '.portfolio-back-to-top-link, .back-to-top', '.phone-navigation-menu-toggle-button, .menu-toggle',
  '.home-page-project-case-study-link, .project .pill', '.contact-page-email-link, .email-link',
  'form button[type=submit]', 'a[href$=".pdf"]', 'main a[href$=".html"]', '[popovertarget]', 'nav ul a', 'input:not([type=hidden])',
  'textarea',
].filter((selector) => !process.env.ONLY_STATE || selector.includes(process.env.ONLY_STATE));

// A fixed pad above both frames, in the harness page's corner: the pointer rests on it between probes, so no frame
// keeps :hover from the previous probe (a stale pointer hovered the second frame's control during its focus probe).
const PARK = `
document.getElementById('park')?.remove();
const pad = document.createElement('div');
pad.id = 'park';
pad.style.cssText = 'position:fixed;inset-inline-end:0;inset-block-end:0;width:40px;height:40px;z-index:2147483647';
document.body.append(pad);
window.scrollTo(0, 0);
`;
async function park(session) {
  await call('POST', `${session}/execute/sync`, { script: PARK, args: [] });
  const pad = Object.values(await call('POST', `${session}/element`, { using: 'css selector', value: '#park' }))[0];
  await call('POST', `${session}/actions`, { actions: [{ type: 'pointer', id: 'mouse', parameters: { pointerType: 'mouse' }, actions: [
    { type: 'pointerMove', duration: 0, origin: { 'element-6066-11e4-a52e-4f735466cecf': pad }, x: 0, y: 0 },
  ] }] });
  await call('DELETE', `${session}/actions`);
}

async function call(method, path, body) {
  const response = await fetch(`${DRIVER}${path}`, {
    method, headers: { 'Content-Type': 'application/json' }, body: body === undefined ? undefined : JSON.stringify(body),
  });
  const json = await response.json();
  if (json.value?.error) throw new Error(`${method} ${path}: ${json.value.error}: ${json.value.message}`);
  return json.value;
}

// Runs in the harness page: loads both builds into iframes, then hashes every element's computed style in each.
const LOAD_AND_COMPARE = `
const [baseUrl, candidateUrl, width, settleMs, done] = arguments;
const frames = [baseUrl, candidateUrl].map((url, index) => {
  document.getElementById('f' + index)?.remove();
  const frame = document.createElement('iframe');
  frame.id = 'f' + index;
  frame.style.top = (index * 1000) + 'px';
  frame.style.width = width + 'px';
  frame.style.height = '900px';
  frame.src = url;
  document.body.append(frame);
  return new Promise((resolve) => frame.addEventListener('load', () => resolve(frame), { once: true }));
});
const collect = (win) => {
  const doc = win.document;
  const PSEUDOS = ['', '::before', '::after', '::marker'];
  const key = (element) => {
    const parts = [];
    for (let node = element; node && node !== doc.documentElement; node = node.parentElement) {
      parts.unshift(node.tagName.toLowerCase() + ':' + [...node.parentElement.children].indexOf(node));
    }
    return parts.join('>') || 'html';
  };
  const out = {};
  for (const element of [doc.documentElement, ...doc.documentElement.querySelectorAll('*')]) {
    const id = key(element);
    for (const pseudo of PSEUDOS) {
      const style = win.getComputedStyle(element, pseudo || null);
      const typed = !pseudo && element.computedStyleMap ? element.computedStyleMap() : null;
      const values = [];
      for (let index = 0; index < style.length; index += 1) {
        const name = style[index];
        if (name.startsWith('--')) continue;
        let value = style.getPropertyValue(name);
        if (typed && name.startsWith('margin')) {
          try { value = String(typed.get(name)); } catch (error) { value = value + ' (typed: ' + error.name + ')'; }
        }
        // Resolved url() values name the build's path prefix, which differs between the two sides.
        values.push(name + ':' + value.replaceAll(win.location.origin, '').replace(/\\/@wt-[a-z0-9-]+\\//g, '/'));
      }
      if (!pseudo) {
        const box = element.getBoundingClientRect();
        values.push('box:' + [box.x, box.y + win.scrollY, box.width, box.height].map((value) => value.toFixed(2)).join(' '));
      }
      out[id + pseudo] = values.sort().join(';');
    }
  }
  return out;
};
Promise.all(frames).then(async (loaded) => {
  for (const frame of loaded) {
    const win = frame.contentWindow;
    for (const image of win.document.images) image.loading = 'eager';
    await win.document.fonts.ready;
    await Promise.all([...win.document.images].map((image) => image.decode().catch(() => {})));
  }
  await new Promise((resolve) => setTimeout(resolve, settleMs));
  const [a, b] = loaded.map((frame) => collect(frame.contentWindow));
  const ids = [...new Set([...Object.keys(a), ...Object.keys(b)])].filter((id) => a[id] !== b[id]);
  const lines = [];
  for (const id of ids.slice(0, ${LIMIT})) {
    const split = (text) => new Map((text ?? '').split(';').map((entry) => [entry.slice(0, entry.indexOf(':')), entry.slice(entry.indexOf(':') + 1)]));
    const [pa, pb] = [split(a[id]), split(b[id])];
    for (const name of new Set([...pa.keys(), ...pb.keys()])) {
      if (pa.get(name) !== pb.get(name)) lines.push(id + ' ' + name + ': ' + pa.get(name) + ' -> ' + pb.get(name));
    }
  }
  done({
    elements: Object.keys(a).length, differing: ids.length, lines,
    innerWidths: loaded.map((frame) => frame.contentWindow.innerWidth),
    typedOm: typeof loaded[0].contentDocument.body.computedStyleMap,
  });
});
`;
const STATE_PROBE = `
const element = document.querySelector(arguments[0]);
if (!element) return 'none';
const flags = ' hover:' + element.matches(':hover') + (arguments[1] === 'focus'
  ? ' focus:' + element.matches(':focus') + ' focus-visible:' + element.matches(':focus-visible') + ' hasFocus:' + document.hasFocus() : '');
return [getComputedStyle(element), getComputedStyle(element, '::before'), getComputedStyle(element, '::after')]
  .map((style) => ['color', 'background-color', 'text-decoration-line', 'outline-style', 'outline-color', 'outline-width',
    'outline-offset', 'box-shadow', 'opacity', 'border-color', 'cursor'].map((name) => style.getPropertyValue(name)).join('|'))
  .join(' / ') + flags;
`;

execFileSync('ssh', ['-o', 'BatchMode=yes', 'm1',
  `ssh -o BatchMode=yes -i ~/.ssh/id_ed25519_vm user@192.168.64.2 '/usr/local/bin/displayplacer "${GUEST_DISPLAY} mode:24 origin:(0,0) degree:0"'`]);
await wait(2000);
const { sessionId } = await call('POST', '/session', { capabilities: { alwaysMatch: { browserName: 'safari' } } });
const session = `/session/${sessionId}`;
let differences = 0;
try {
  await call('POST', `${session}/timeouts`, { script: 120_000 });
  await call('POST', `${session}/window/rect`, { x: 0, y: 0, width: 1920, height: 1080 });
  await call('POST', `${session}/url`, { url: `${ORIGIN}/@wt-harness/blank.html` });
  const prefix = (value) => (value ? `/${value.replace(/^\/|\/$/g, '')}` : '');
  for (const page of PAGES) {
    for (const width of WIDTHS) {
      const urls = [basePrefix, candidatePrefix].map((value) => `${ORIGIN}${prefix(value)}/${page}.html?s=${Date.now()}`);
      // Home's intro animation ends at 7.25 s; computed opacity changes until then.
      const result = await call('POST', `${session}/execute/async`, {
        script: LOAD_AND_COMPARE, args: [...urls, width, page === 'index' ? 8500 : 1500],
      });
      for (const line of result.lines) console.log(`${page}@${width} ${line}`);
      if (result.differing > LIMIT) console.log(`${page}@${width} ... ${result.differing - LIMIT} more differing element pseudos`);
      differences += result.differing;
      // Hover and focus: WebDriver pointer moves and element focus inside each frame.
      const frames = await call('POST', `${session}/execute/sync`, { script: 'return [...document.querySelectorAll("iframe")]', args: [] });
      for (const selector of STATES) {
        for (const action of ['hover', 'focus']) {
          const states = [];
          for (const frame of frames) {
            await park(session);
            await wait(150);
            await call('POST', `${session}/frame`, { id: frame });
            let state;
            try {
              const element = await call('POST', `${session}/element`, { using: 'css selector', value: selector });
              const reference = Object.values(element)[0];
              // A frame sometimes loses the page's focus as it is switched to; then it cannot show :focus-visible at
              // all, so a focus probe focuses again (up to three times) until the frame reports hasFocus.
              for (let attempt = 0; attempt < (action === 'focus' ? 3 : 1); attempt += 1) {
                await call('POST', `${session}/execute/sync`, {
                  // A frame only shows :focus-visible while it holds the page's focus, so focus its window first.
                  script: 'arguments[0].scrollIntoView({ block: "center" }); if (arguments[1] === "focus") { window.focus(); arguments[0].focus({ focusVisible: true }); }',
                  args: [element, action],
                });
                if (action === 'hover') {
                  await call('POST', `${session}/actions`, { actions: [{ type: 'pointer', id: 'mouse', parameters: { pointerType: 'mouse' }, actions: [{ type: 'pointerMove', duration: 0, origin: { 'element-6066-11e4-a52e-4f735466cecf': reference }, x: 0, y: 0 }] }] });
                }
                await wait(300);
                state = await call('POST', `${session}/execute/sync`, { script: STATE_PROBE, args: [selector, action] });
                if (action !== 'focus' || !state.includes('hasFocus:false')) break;
                await call('POST', `${session}/execute/sync`, { script: 'document.activeElement?.blur()', args: [] });
                await wait(300);
              }
              if (action === 'hover') await call('DELETE', `${session}/actions`);
              if (action === 'focus') await call('POST', `${session}/execute/sync`, { script: 'document.activeElement?.blur()', args: [] });
            } catch (error) {
              state = /no such element/.test(error.message) ? 'none' : `error: ${error.message.split('\n')[0].slice(0, 120)}`;
            }
            states.push(state);
            await call('POST', `${session}/frame/parent`, {});
          }
          if (process.env.DEBUG_STATES) console.log(`  ${page}@${width} ${action} ${selector}: ${states.map((state) => state.slice(0, 60) + ' ... ' + state.slice(-60)).join(' | ')}`);
          // A focus probe must see no :hover and a hover probe no :focus; otherwise the probe itself is invalid.
          const unclean = states.filter((state) => action === 'focus' && (state.includes(' hover:true') || state.includes('hasFocus:false')));
          if (unclean.length) console.log(`${page}@${width} PROBE INVALID, the frame lacks focus or the pointer hovers the control: ${selector}`);
          if (states[0] !== states[1] || unclean.length) {
            differences += 1;
            console.log(`${page}@${width} ${action} ${selector}: ${states[0]} -> ${states[1]}`);
          }
        }
      }
      console.log(`checked ${page}@${width}: ${result.elements} element pseudos, ${result.differing} differ, inner widths ${result.innerWidths}, typed OM ${result.typedOm}`);
    }
  }
} finally {
  await call('DELETE', session);
}
console.log(`summary: ${differences} differences`);
process.exitCode = differences ? 1 : 0;
```
