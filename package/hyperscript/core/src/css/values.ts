/**
 Branded CSS value types and constructor functions.
 
 Replaces raw CSS value strings with type-safe constructors that prevent
 invalid units, disallowed color functions, and named colors at the type level.
 Every constructor returns `CssValue`: a branded string that the strict
 {@link CssDeclarations} type accepts in any property value position.
 
 Constructors are prefixed with `css` to distinguish them from other functions.
 */

//region Branded base type

/**
 Branded CSS value: the base type returned by all value constructors.
 
 Property value types accept `CssValue` alongside specific keyword literals,
 preventing raw strings like `'10px'` or `'red'` from being used directly.
 
 @example
 ```ts
 const gap: CssValue = cssRem(1);    // OK
 const color: CssValue = cssVar('fg'); // OK
 ```
 */
export type CssValue = string & { readonly __cssValue: unique symbol; };

//endregion

//region Type utilities

/**
 Extracts only literal string types from a union, removing `string & {}` escape hatches.
 
 csstype uses `string & {}` in value unions to allow arbitrary strings while
 preserving autocomplete. This utility strips that escape hatch so only
 known keyword literals remain.
 
 @example
 ```ts
 type T = ExtractLiteral<'flex' | 'grid' | (string & {})>;
 // T = 'flex' | 'grid'
 ```
 */
export type ExtractLiteral<T,> = T extends string ? string extends T ? never
  : T
  : T extends number ? number extends T ? never
    : T
    : never;

/**
 CSS named colors (the 148 CSS Color Level 4 keywords).
 
 Excluded from strict property value types to enforce the `color-named: never` rule.
 `transparent` and `currentColor` are NOT named colors per the CSS spec; they are
 special color keywords and remain allowed.
 */
export type CssNamedColor = 'aliceblue' | 'antiquewhite' | 'aqua' | 'aquamarine' | 'azure'
  | 'beige' | 'bisque' | 'black' | 'blanchedalmond' | 'blue' | 'blueviolet' | 'brown'
  | 'burlywood' | 'cadetblue' | 'chartreuse' | 'chocolate' | 'coral' | 'cornflowerblue'
  | 'cornsilk' | 'crimson' | 'cyan' | 'darkblue' | 'darkcyan' | 'darkgoldenrod'
  | 'darkgray' | 'darkgreen' | 'darkgrey' | 'darkkhaki' | 'darkmagenta' | 'darkolivegreen'
  | 'darkorange' | 'darkorchid' | 'darkred' | 'darksalmon' | 'darkseagreen'
  | 'darkslateblue' | 'darkslategray' | 'darkslategrey' | 'darkturquoise' | 'darkviolet'
  | 'deeppink' | 'deepskyblue' | 'dimgray' | 'dimgrey' | 'dodgerblue' | 'firebrick'
  | 'floralwhite' | 'forestgreen' | 'fuchsia' | 'gainsboro' | 'ghostwhite' | 'gold'
  | 'goldenrod' | 'gray' | 'green' | 'greenyellow' | 'grey' | 'honeydew' | 'hotpink'
  | 'indianred' | 'indigo' | 'ivory' | 'khaki' | 'lavender' | 'lavenderblush'
  | 'lawngreen' | 'lemonchiffon' | 'lightblue' | 'lightcoral' | 'lightcyan'
  | 'lightgoldenrodyellow' | 'lightgray' | 'lightgreen' | 'lightgrey' | 'lightpink'
  | 'lightsalmon' | 'lightseagreen' | 'lightskyblue' | 'lightslategray' | 'lightslategrey'
  | 'lightsteelblue' | 'lightyellow' | 'lime' | 'limegreen' | 'linen' | 'magenta'
  | 'maroon' | 'mediumaquamarine' | 'mediumblue' | 'mediumorchid' | 'mediumpurple'
  | 'mediumseagreen' | 'mediumslateblue' | 'mediumspringgreen' | 'mediumturquoise'
  | 'mediumvioletred' | 'midnightblue' | 'mintcream' | 'mistyrose' | 'moccasin'
  | 'navajowhite' | 'navy' | 'oldlace' | 'olive' | 'olivedrab' | 'orange' | 'orangered'
  | 'orchid' | 'palegoldenrod' | 'palegreen' | 'paleturquoise' | 'palevioletred'
  | 'papayawhip' | 'peachpuff' | 'peru' | 'pink' | 'plum' | 'powderblue' | 'purple'
  | 'rebeccapurple' | 'red' | 'rosybrown' | 'royalblue' | 'saddlebrown' | 'salmon'
  | 'sandybrown' | 'seagreen' | 'seashell' | 'sienna' | 'silver' | 'skyblue' | 'slateblue'
  | 'slategray' | 'slategrey' | 'snow' | 'springgreen' | 'steelblue' | 'tan' | 'teal'
  | 'thistle' | 'tomato' | 'turquoise' | 'violet' | 'wheat' | 'white' | 'whitesmoke'
  | 'yellow' | 'yellowgreen';

/**
 Deprecated CSS system colors (e.g. `ActiveBorder`, `ButtonHighlight`).
 
 Excluded alongside named colors for strictness.
 */
export type CssDeprecatedSystemColor = 'ActiveBorder' | 'ActiveCaption' | 'AppWorkspace'
  | 'Background' | 'ButtonHighlight' | 'ButtonShadow' | 'CaptionText' | 'InactiveBorder'
  | 'InactiveCaption' | 'InactiveCaptionText' | 'InfoBackground' | 'InfoText' | 'Menu'
  | 'MenuText' | 'Scrollbar' | 'ThreeDDarkShadow' | 'ThreeDFace' | 'ThreeDHighlight'
  | 'ThreeDLightShadow' | 'ThreeDShadow' | 'Window' | 'WindowFrame' | 'WindowText';

/**
 Converts a csstype property value type to a strict value type.
 
 - Strips `string & {}` (the any-string escape hatch) via `ExtractLiteral`
 - Removes named colors and deprecated system colors
 - Adds `CssValue` (branded constructor return type) as a valid alternative
 - Preserves plain `number` for properties that accept any numeric value
   (e.g. `opacity`, `flex-grow`, `z-index`) while keeping it excluded from
   length properties (where only the `0` literal from `TLength` survives)
 
 csstype encodes the distinction: length properties use `TLength = (string & {}) | 0`
 (only literal `0` without units), while number properties use `(number & {})` directly
 (any number is valid CSS). The `[number] extends [T]` check detects the wide number
 case without distributing over the union.
 
 @example
 ```ts
 // csstype: Property.Display = 'flex' | 'grid' | ... | (string & {})
 // StrictValue<Property.Display> = 'flex' | 'grid' | ... | CssValue
 //
 // csstype: Property.Opacity = Globals | (number & {}) | (string & {})
 // StrictValue<Property.Opacity> = Globals | number | CssValue
 //
 // csstype: Property.Gap<TLength> = Globals | TLength | (string & {})
 // StrictValue<Property.Gap> = Globals | 0 | CssValue  (number excluded)
 ```
 */
export type StrictValue<T,> =
  | Exclude<ExtractLiteral<T>, CssNamedColor | CssDeprecatedSystemColor>
    | CssValue
    | ([number,] extends [T,] ? number : never);

//endregion

//region Re-exports from constructors
export {
  cssAnchor,
  cssCalc,
  cssCh,
  cssClamp,
  cssColorFn,
  cssCommaList,
  cssCompounded,
  cssCqb,
  cssCqi,
  cssCubicBezier,
  cssDvb,
  cssDvi,
  cssEm,
  cssFr,
  cssInt,
  cssLh,
  cssMax,
  cssMin,
  cssNum,
  cssOklch,
  cssOklchFrom,
  cssPercent,
  cssRandom,
  cssRem,
  cssRotate,
  cssS,
  cssScale,
  cssTranslateX,
  cssTranslateY,
  cssTurn,
  cssVar,
  cssVb,
  cssVi,
} from './values.constructors.ts';
//endregion
