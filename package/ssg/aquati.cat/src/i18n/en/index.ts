import type { Label, } from '../labels-types.ts';

/**
 English (base locale) label table.
 */
const en: Record<Label, string> = {
  siteName: 'Aquaticat',
  siteDescription: 'Changing the world, one design at a time',
  chooseALang: 'choose a language',
  searchPlaceholder: 'Search keyword, topic, text',
  noResults: 'No results',
  page: 'page',
  postNotInLang: "Post doesn't exist in specified language",
  redirectingToLangChooser: 'Choose a language for',
  themeToggle: 'Invert theme',
  langSwitcher: 'Switch language',
  published: 'Published',
  updated: 'Updated',
};

export default en;
