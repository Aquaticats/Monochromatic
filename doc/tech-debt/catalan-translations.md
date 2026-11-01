# Tech debt: Catalan translations in ssg-test

Created 2026-05-12.
 The Catalan content shipped today is intentionally rough;
 it went out under a same-day deadline and has not received the quality bar applied to the English source
or the existing Chinese translations.
 This note records what to revisit when there is time to do the work properly.

## What shipped

Commit `d0ad7a50` translated the 17 ssg-test posts to Catalan and registered `ca` as a typesafe-i18n locale:

- 17 MDX files under `package/ssg/aquati.cat/src/content/ca/` (1444 insertions).
- UI strings bundle at `package/ssg/aquati.cat/src/i18n/ca/index.ts`,
   mirroring the Chinese bundle.
- Regenerated typesafe-i18n outputs (`i18n-types.ts`,
   `i18n-util.ts`,
   `i18n-util.async.ts`,
   `i18n-util.sync.ts`).

Today's follow-up commit `68c3930d` adds a `langSwitcher` UI string for the new dropdown;
 the Catalan string `Canvia d'idioma` was written in the same rushed pass and is subject to the same caveat.

## Why it is dirty

The translations were produced under shipping pressure on the same day,
 without a native or fluent reviewer in the loop.
 Concretely:

- Wording was not vetted by a Catalan speaker.
- Technical terminology (CSS / HTML / web platform terms) was translated literally
  where an established Catalan technical term may exist;
   some phrasings will read as calques.
- Tone,
   register,
   and idiom were not aligned with the original English voice;
   the source posts have a deliberate informal-but-precise voice that the translations probably flatten.
- Cross-post consistency was not enforced:
   a term translated one way in `about.mdx` may render differently in `link-vs-button-quiz.mdx`.
- Quiz and question prose in `link-vs-button-quiz.mdx` (555 lines,
   by far the longest file) is the most likely site of awkward phrasing
   because the question format leans on rhetorical structure
that does not transfer mechanically.
- Frontmatter fields (`title`,
   `description`) were translated in the same rushed pass;
   the descriptions feed search snippets and meta tags,
   so weaknesses there hurt discoverability.

## What "looking back" should cover

When time allows,
 walk this list:

- Native or fluent review pass over all 17 posts.
   Prioritise `link-vs-button-quiz.mdx`,
   `mdx.mdx`,
   `magicbread.mdx`,
   and `portfolio-*.mdx` (longest,
   highest visibility).
- Build a glossary of recurring technical terms (button,
   link,
   popover,
   dropdown,
   focus ring,
   etc.) and apply it consistently across files.
   Promote the glossary into a doc under `doc/` so future translators have a reference.
- Reread frontmatter `title` and `description` for every post;
   these surface in the listing pages,
   meta tags,
   and pagefind snippets.
- Reconcile UI strings in `src/i18n/ca/index.ts` with the post body translations once the glossary is settled.
   Includes `langSwitcher`,
   `themeToggle`,
   `searchPlaceholder`,
   `postNotInLang`,
   `redirectingToLangChooser`,
   `chooseALang`.
- Re-run the SSG build and visually verify each Catalan page renders correctly,
   paying attention to line lengths (Catalan tends to expand vs English)
and any broken inline markup introduced during translation.
- Decide whether to publish the language switcher dropdown's autonym for `ca` as `Català` (current)
  or something else (e.g. `Català (general)` if a dialect distinction ever becomes relevant).
   For now `Català` is fine.

## Out of scope for the look-back

These are deliberate non-goals so the review stays focused:

- Adding new posts in Catalan that do not exist in English.
- Changing the i18n tooling (typesafe-i18n stays).
- Restructuring the per-locale content directory layout.
