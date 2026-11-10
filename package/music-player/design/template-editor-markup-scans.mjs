//region Character walks the template editor page checks ask for
// The page checks in `template-editor.mjs` ask four questions of one generated HTML string: which script bodies
// it still holds, whether a quoted template call is a conditional, how many form elements it opens and whether it
// references anything it does not carry. Each answer is one walk over the markup's characters, so a reader follows
// the question instead of a pattern, and the cost stays the linear one the page's own size fixes.
// Design evidence only; no production implementation is authorized.

//region Shared character and literal tests
/** ASCII letters, digits and underscore: the characters a word boundary separates from. */
function isWordCharacter(character) {
  return ((character >= '0') && (character <= '9'))
    || ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || (character === '_');
}
/** One ASCII letter, either case. */
function isAsciiLetter(character) {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'));
}
/** Folds one ASCII letter to lower case and leaves every other character alone. */
function lowerAscii(character) {
  return isAsciiLetter(character) ? character.toLowerCase() : character;
}
/** Whether `literal` sits at `at`, folding ASCII case on both sides. */
function startsWithFolded({
  value,
  literal,
  at
}) {
  if ((at + literal.length) > value.length) return false;
  for (let offset = 0; offset < literal.length; offset += 1) {
    if (lowerAscii(value[at + offset]) !== lowerAscii(literal[offset])) return false;
  }
  return true;
}
/** Position of the first `literal` at or after `fromIndex`, folding ASCII case on both sides. */
function indexOfFolded({
  value,
  literal,
  fromIndex
}) {
  for (let at = fromIndex; (at + literal.length) <= value.length; at += 1) {
    if (startsWithFolded({
      value,
      literal,
      at
    })) return at;
  }
  return -1;
}
/** Whether the tag name ending at `afterTagName` is followed by a non-word character or by the end of the page. */
function tagNameEndsAt({
  html,
  afterTagName
}) {
  const next = html[afterTagName];
  return (next === undefined) || !isWordCharacter(next);
}
//endregion

//region Script bodies, removed before the conditional check reads the page
/**
 The page without its `<script>` elements: each opening tag whose name ends there, its body and its closing
 `</script>` are dropped, so a quoted conditional inside a script cannot be mistaken for the template's own.
 
 A `</script>` later in the page closes the element, as the page carries no nested scripts.
 
 @param {object} arguments - the scanned page
 @param {string} arguments.html - the built page
 
 @returns {string} the page with every script element removed
 */
export function scriptBodiesRemoved({ html }) {
  const opening = '<script';
  const closing = '</script>';
  let out = '';
  let from = 0;
  while (from < html.length) {
    const at = html.indexOf(
      opening,
      from
    );
    if (at === (-1)) break;
    const afterOpening = at + opening.length;
    if (!tagNameEndsAt({
      html,
      afterTagName: afterOpening
    })) {
      // Not a script element: keep the text as it stands and look for the next opening.
      out += html.slice(
        from,
        afterOpening
      );
      from = afterOpening;
      continue;
    }
    // An unterminated script element is no element, and no later opening can close either.
    const close = html.indexOf(
      closing,
      afterOpening
    );
    if (close === (-1)) break;
    out += html.slice(
      from,
      at
    );
    from = close + closing.length;
  }
  return out + html.slice(from);
}
//endregion

//region A quoted template call that is a conditional
/**
 Whether the text quotes a template conditional, `$` then characters with no `$` among them then a word boundary
 then `if(`. A conditional in the quoted template is what the decided design removed, so any occurrence is refused.
 
 Only the last `$` before the call matters: the characters between a `$` and the call may hold no `$`, so a `$`
 with another `$` after it and before the call cannot reach the call.
 
 @param {object} arguments - the scanned text
 @param {string} arguments.html - the page, with its script bodies already removed
 
 @returns {boolean} whether a conditional is quoted
 */
export function quotesConditionalTemplateCall({ html }) {
  const call = 'if(';
  let sawDollar = false;
  for (let at = 0; (at + call.length) <= html.length; at += 1) {
    if (sawDollar
      && html.startsWith(
        call,
        at
      )
      && ((at === 0) || !isWordCharacter(html[at - 1]))) return true;
    if (html[at] === '$') sawDollar = true;
  }
  return false;
}
//endregion

//region Form elements the page opens
/**
 How many `<form>` elements the page opens: one self-contained form is the decided shape, so any other count is
 refused.
 
 @param {object} arguments - the scanned page
 @param {string} arguments.html - the built page
 
 @returns {number} the number of form elements opened
 */
export function countFormTags({ html }) {
  const opening = '<form';
  let count = 0;
  let from = 0;
  while (true) {
    const at = html.indexOf(
      opening,
      from
    );
    if (at === (-1)) return count;
    if (tagNameEndsAt({
      html,
      afterTagName: at + opening.length
    })) count += 1;
    from = at + 1;
  }
}
//endregion

//region Elements that would make the page depend on something it does not carry
// Each entry is an opening tag name and the attribute text that makes the page reach outside itself. The tag name
// must end at a word boundary, and the attribute is read only inside that element's own attributes, up to the `>`
// that ends the tag, so a literal `<script` written in the page's prose is not the element itself.
const externalReferences = [
  {
    tagName: '<script',
    written: ['src=']
  },
  {
    tagName: '<link',
    written: ['href=']
  },
  {
    tagName: '<img',
    written: ['src="http:', 'src="https:']
  }
];
/** Whether one opening tag's attributes carry one of the written attribute texts. */
function tagCarriesWritten({
  html,
  tagName,
  written
}) {
  let from = 0;
  while (true) {
    const at = indexOfFolded({
      value: html,
      literal: tagName,
      fromIndex: from
    });
    if (at === (-1)) return false;
    const afterTagName = at + tagName.length;
    if (tagNameEndsAt({
      html,
      afterTagName
    })) {
      for (let position = afterTagName; (position < html.length) && (html[position] !== '>'); position += 1) {
        // An attribute's name begins at a word boundary; the name's first character is a word character.
        if ((position > 0) && !isWordCharacter(html[position - 1])) {
          for (const candidate of written) {
            if (startsWithFolded({
              value: html,
              literal: candidate,
              at: position
            })) return true;
          }
        }
      }
    }
    from = at + 1;
  }
}
/** Whether a leftover `__TEMPLATE_EDITOR_<NAME>__` placeholder survived the build. */
function carriesPlaceholder({ html }) {
  const prefix = '__TEMPLATE_EDITOR_';
  let from = 0;
  while (true) {
    const at = indexOfFolded({
      value: html,
      literal: prefix,
      fromIndex: from
    });
    if (at === (-1)) return false;
    let after = at + prefix.length;
    let letters = 0;
    while (isAsciiLetter(html[after])) {
      letters += 1;
      after += 1;
    }
    if ((letters > 0) && (html.slice(
      after,
      after + 2
    ) === '__')) return true;
    from = at + 1;
  }
}
/**
 Whether the page references anything it does not carry: a leftover template placeholder, a script with a `src`,
 a stylesheet link with an `href`, or an image loaded from the network. Per element the tag name and attribute
 name are compared folded to ASCII case, which is where the page checks folded case too.
 
 @param {object} arguments - the scanned page
 @param {string} arguments.html - the built page
 
 @returns {boolean} whether the page depends on something outside itself
 */
export function hasExternalReference({ html }) {
  if (carriesPlaceholder({ html })) return true;
  return externalReferences.some(entry => tagCarriesWritten({
    html,
    ...entry
  }));
}
//endregion
//endregion
