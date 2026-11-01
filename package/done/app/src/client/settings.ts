/**
 Client entry script for the Settings page.
 
 Same hydration pattern as inbox.ts: injectCSS → readPageData → build DOM into #app.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { injectCSS, } from './lib/inject-css.ts';
import { readPageData, } from './lib/page-data.ts';
import { globalStyles, } from './styles.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/side-drawer.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/top-nav.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import './component/setting-group.ts';

/**
 Shape of the JSON blob embedded in the settings page by the server.
 */
type SettingsPageData = {
  message: string;
};

injectCSS(globalStyles,);

/**
 Deserialized page data (currently unused but kept for future features).
 */
const _pageData = readPageData<SettingsPageData>();

/**
 Raw DOM element for the `#app` container.
 */
const appElement = document.querySelector<HTMLElement>('#app',);
if (!(appElement instanceof HTMLElement))
  throw new Error('Missing app element',);

/**
 Validated `#app` container element.
 */
const app = appElement;

//region Calendar connect
app.append(
  h({
    tag: 'setting-group',
    attrs: {
      label: 'System calendar',
      description: 'Connect to start two-way auto-syncing.',
      mode: 'button',
    },
  },),
);
//endregion Calendar connect

//region Data privacy
app.append(
  h({
    tag: 'setting-group',
    attrs: {
      label: 'Data privacy',
      description: 'We never sell your data, and we never will.',
      on: '',
    },
  },),
);
//endregion Data privacy

//region Dark theme
app.append(
  h({
    tag: 'setting-group',
    attrs: {
      label: 'Dark theme',
      description: 'Unlock the dark side.',
    },
  },),
);
//endregion Dark theme
