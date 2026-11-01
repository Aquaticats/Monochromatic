/**
 Sets CSS properties on the `:root` element based on URL query parameters.
 
 For each URL query parameter, the parameter name is passed verbatim as the CSS
 property name to `setProperty`, and the value becomes the CSS property value.
 `--`-prefixed keys set CSS custom properties (`--brand`); standard property
 names (`color`, `background-color`) set inline styles on `<html>` itself.
 Keys that are neither valid custom properties nor known CSS properties are
 silently discarded by `setProperty`. Useful for theming, debugging, and
 configuration via shareable URLs.
 
 @param allowedProperties - Optional iterable of CSS property names to filter which
   parameters are processed. If provided, only properties listed in this iterable
   will be applied from the URL parameters. If omitted, all URL parameters are
   processed as CSS properties. Always pass an allowlist when the URL is
   user-controllable, since otherwise a crafted link can overwrite any
   layout-critical property on `:root`.
 
 @example
 ```ts
 // URL: https://example.com/?--brand=red&font-size=16px&margin-inline=1rem
 
 // Apply all URL parameters verbatim
 onLoadSetCssFromUrlParams();
 // Sets: --brand: red (custom property),
 //       font-size: 16px and margin-inline: 1rem (inline styles on <html>)
 
 // Apply only specific allowed properties
 onLoadSetCssFromUrlParams(['--brand', 'font-size']);
 // Sets: --brand: red, font-size: 16px (margin-inline ignored)
 
 // With no matching parameters
 onLoadSetCssFromUrlParams(['--non-existent-prop']);
 // Sets: nothing (no matching parameters found)
 ```
 */
export function onLoadSetCssFromUrlParams(allowedProperties?: Iterable<string>,): void {
  /**
   Parsed URL query, iterated to project each entry onto a CSS custom property.
   */
  const params = new URLSearchParams(globalThis.location
    .search,);

  if (allowedProperties) {
    /**
     Set form of `allowedProperties` so each entry can be matched in O(1).
     */
    const allowedSet = new Set(allowedProperties,);
    for (const [key, value,] of params.entries()) {
      if (allowedSet.has(key,)) {
        document.documentElement
          .style
          .setProperty(
          key,
          value,
          );
      }
    }
  }
  else {
    // No allowedProperties, so process all parameters
    for (const [key, value,] of params.entries()) {
      document.documentElement
        .style
        .setProperty(
        key,
        value,
        );
    }
  }
}
