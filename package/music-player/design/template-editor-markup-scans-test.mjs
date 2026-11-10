//region Fixtures for the character walks behind the template editor page checks
// Each case names the walk, the page text and the answer the page check must give, so a walk that drifts from what
// the page checks ask fails here instead of letting a page through. The first cases read the real authored template,
// and every synthetic case is a shape that page, or prose beside it, can hold.
// Design evidence only; no production implementation is authorized.

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import {
  countFormTags,
  hasExternalReference,
  quotesConditionalTemplateCall,
  scriptBodiesRemoved
} from './template-editor-markup-scans.mjs';

//region Cases per walk
// A script element is dropped with its body; an opening that is not a script element stays as written.
const scriptCases = [
  {
    subject: 'a script element',
    html: '<p>a</p><script>const x = 1;</script><p>b</p>',
    expected: '<p>a</p><p>b</p>'
  },
  {
    subject: 'a script element with an empty body',
    html: '<script></script>',
    expected: ''
  },
  {
    subject: 'two script elements',
    html: '<script>a</script>between<script>b</script>',
    expected: 'between'
  },
  {
    subject: 'a body that holds markup',
    html: '<script><div class="x">y</div></script>',
    expected: ''
  },
  {
    subject: 'a body that spans lines',
    html: '<script>\nconst a = 1;\n</script>tail',
    expected: 'tail'
  },
  {
    subject: 'an unterminated script element',
    html: 'text<script>const a = 1;',
    expected: 'text<script>const a = 1;'
  },
  {
    subject: 'an opening with no closing tag in the page',
    html: '<script',
    expected: '<script'
  },
  {
    subject: 'a tag whose name only begins with script',
    html: '<scripting>a</scripting>',
    expected: '<scripting>a</scripting>'
  },
  {
    subject: 'an upper case element, which the page checks read as text',
    html: '<SCRIPT>a</SCRIPT>',
    expected: '<SCRIPT>a</SCRIPT>'
  },
  {
    subject: 'a page with no script',
    html: '<p>a</p>',
    expected: '<p>a</p>'
  }
];
// A conditional is a dollar sign, characters with no dollar sign among them, a word boundary and a call.
const conditionalCases = [
  {
    subject: 'a plain conditional',
    html: '$if(mi(peak), a)$',
    expected: true
  },
  {
    subject: 'a conditional after other text',
    html: '$tf(mi(len), m:ss)$ and if(mi(peak), a)$',
    expected: true
  },
  {
    subject: 'a conditional after two dollar signs',
    html: '$$if(mi(peak), a)$',
    expected: true
  },
  {
    subject: 'a conditional whose dollar sign follows a word character',
    html: 'x$if(mi(peak), a)$',
    expected: true
  },
  {
    subject: 'a call with no dollar sign before it',
    html: 'if(mi(peak), a)',
    expected: false
  },
  {
    subject: 'a call whose only dollar sign follows it',
    html: 'if(mi(peak), a)$',
    expected: false
  },
  {
    subject: 'a call preceded by a word character',
    html: '$_if(mi(peak), a)$',
    expected: false
  },
  {
    subject: 'a call whose only dollar sign before it is cut off by another dollar sign',
    html: '$a$bif(mi(peak), a)$',
    expected: false
  },
  {
    subject: 'a call reached across another expression',
    html: '$a$ if(mi(peak), a)$',
    expected: true
  },
  {
    subject: 'the studied templates, which hold no conditional',
    html: ['$tf(mi(len), m:ss)$ $mi(peak)$', '$mi(track)$ of $mi(total)$ $mi(peak)$'].join('\n'),
    expected: false
  },
  {
    subject: 'a page with no template at all',
    html: '<p>a</p>',
    expected: false
  },
  {
    subject: 'an empty page',
    html: '',
    expected: false
  }
];
// A form element is an opening form tag whose name ends there.
const formCases = [
  {
    subject: 'one form',
    html: '<form id="review-form"></form>',
    expected: 1
  },
  {
    subject: 'two forms',
    html: '<form></form><form></form>',
    expected: 2
  },
  {
    subject: 'a form tag at the end of the page',
    html: '<p>a</p><form',
    expected: 1
  },
  {
    subject: 'a closing tag alone',
    html: '</form>',
    expected: 0
  },
  {
    subject: 'a tag whose name only begins with form',
    html: '<formation></formation>',
    expected: 0
  },
  {
    subject: 'an upper case tag',
    html: '<FORM></FORM>',
    expected: 0
  },
  {
    subject: 'the name in prose',
    html: '<p>write a form element</p>',
    expected: 0
  }
];
// A page that reaches outside itself names the element, its attribute and, for an image, the network scheme.
const referenceCases = [
  {
    subject: 'a leftover placeholder',
    html: '<script>const images = __TEMPLATE_EDITOR_IMAGES__;</script>',
    expected: true
  },
  {
    subject: 'a placeholder folded to lower case',
    html: 'const images = __template_editor_images__;',
    expected: true
  },
  {
    subject: 'the prefix with no name between the underscores',
    html: '__TEMPLATE_EDITOR___',
    expected: false
  },
  {
    subject: 'a script with a source',
    html: '<script src="viewer.js"></script>',
    expected: true
  },
  {
    subject: 'a script with a source after another attribute',
    html: '<script type="module" src="viewer.js"></script>',
    expected: true
  },
  {
    subject: 'a script with a source in another case',
    html: '<SCRIPT SRC="viewer.js"></SCRIPT>',
    expected: true
  },
  {
    subject: 'a script with a source spelled across lines',
    html: '<script\n  src="viewer.js"></script>',
    expected: true
  },
  {
    subject: 'a script whose attribute only ends with source',
    html: '<script data-src="viewer.js"></script>',
    expected: true
  },
  {
    subject: 'a script with spacing around the equals sign',
    html: '<script src = "viewer.js"></script>',
    expected: false
  },
  {
    subject: 'a script with no source',
    html: '<script>const a = 1;</script>',
    expected: false
  },
  {
    subject: 'a stylesheet link',
    html: '<link rel="stylesheet" href="page.css">',
    expected: true
  },
  {
    subject: 'a link whose attribute only ends with href',
    html: '<link data-href="page.css">',
    expected: true
  },
  {
    subject: 'a link with no href',
    html: '<link rel="icon">',
    expected: false
  },
  {
    subject: 'an image loaded over https',
    html: '<img alt="a" src="https://example.test/a.png">',
    expected: true
  },
  {
    subject: 'an image loaded over http',
    html: '<img src="http://example.test/a.png">',
    expected: true
  },
  {
    subject: 'an image loaded over https in other cases',
    html: '<IMG ALT="a" SRC="HTTPS://example.test/a.png">',
    expected: true
  },
  {
    subject: 'an image loaded from a scheme relative address',
    html: '<img src="//example.test/a.png">',
    expected: false
  },
  {
    subject: 'an image whose source is carried in the page',
    html: '<img src="data:image/png;base64,iVBORw0KGgo=">',
    expected: false
  },
  {
    subject: 'an image with a placeholder source',
    html: '<img alt="a" src="">',
    expected: false
  },
  {
    subject: 'an image whose attribute only ends with source',
    html: '<img data-src="https://example.test/a.png">',
    expected: true
  },
  {
    subject: 'a network scheme cut off before its colon',
    html: '<img src="https">',
    expected: false
  },
  {
    subject: 'a script element named in prose',
    html: '<p>the page holds one <code>&lt;script</code> element and no src</p>',
    expected: false
  },
  {
    subject: 'a page that carries everything it shows',
    html: '<form><img src="data:image/png;base64,iVBORw0KGgo="></form>',
    expected: false
  }
];
//endregion

//region Fixture runner
let checked = 0;
/** Records one comparison, failing with both values so a drifted walk names what it answered. */
function check({
  subject,
  actual,
  expected
}) {
  if (Object.is(actual, expected)) {
    checked += 1;
    return;
  }
  throw new Error(`${subject}: expected ${JSON.stringify(expected)}, answered ${JSON.stringify(actual)}`);
}
//endregion

//region Cases over the real authored template
const template = readFileSync(
  join(
    import.meta.dirname,
    'questions',
    'template-editor.template.html'
  ),
  'utf8'
);
const templateWithoutScripts = scriptBodiesRemoved({ html: template });
check({
  subject: 'the authored template drops the script element it holds',
  actual: templateWithoutScripts.includes('<script'),
  expected: false
});
check({
  subject: 'the authored template is shorter without its script',
  actual: templateWithoutScripts.length < template.length,
  expected: true
});
check({
  subject: 'the authored template opens one form',
  actual: countFormTags({ html: template }),
  expected: 1
});
check({
  subject: 'the authored template still carries its build placeholders',
  actual: hasExternalReference({ html: template }),
  expected: true
});
check({
  subject: 'the authored template quotes no conditional once its script is read out',
  actual: quotesConditionalTemplateCall({ html: templateWithoutScripts }),
  expected: false
});
//endregion

//region Every case in turn
for (const fixture of scriptCases) {
  check({
    ...fixture,
    actual: scriptBodiesRemoved({ html: fixture.html })
  });
}
for (const fixture of conditionalCases) {
  check({
    ...fixture,
    actual: quotesConditionalTemplateCall({ html: fixture.html })
  });
}
for (const fixture of formCases) {
  check({
    ...fixture,
    actual: countFormTags({ html: fixture.html })
  });
}
for (const fixture of referenceCases) {
  check({
    ...fixture,
    actual: hasExternalReference({ html: fixture.html })
  });
}
console.log(`Template editor markup scans: ${checked} cases answered as their page checks require.`);
//endregion
//endregion
