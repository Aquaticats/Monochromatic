# Ottawa Public Health; Common Look and Feel audit

> **Deprecated.** This audit package is no longer maintained and has moved to
> `packages-deprecated/audit/oph-common-look-and-feel/` for reference. It is a
> private static document (never published); the source stays for historical
> reference only.

Exhaustive design audit of [ottawapublichealth.ca](https://www.ottawapublichealth.ca/en/index.aspx)
as a reference for building new pages that conform to the site's existing visual language and component patterns.

## What this contains

- **Design tokens**: complete color palette, typography scale, spacing,
  breakpoints, elevation, z-index layers, motion, borders
- **Tech stack analysis**: CMS platform (iCreate/GHD Digital), jQuery ecosystem,
  analytics, search provider, CSS architecture
- **Page layouts**: documented homepage and content page structures with ASCII diagrams
- **Component inventory**: 16 component types with properties, states, and CSS values
- **Accessibility audit**: landmark usage, ARIA patterns, contrast ratios, focus handling
- **Screenshots**: desktop and mobile captures of homepage, services, content pages, footer
- **Web component recreations**: 7 interactive web components
  (`oph-button`, `oph-accordion`, `oph-alert`, `oph-callout`,
  `oph-breadcrumb`, `oph-table`, `oph-feature-tile`)

## Viewing

Open `src/index.html` directly in a browser.
All screenshots are embedded as base64 AVIF data URIs: no server required.
