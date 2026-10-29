//region XML attribute coding: escape and unescape JetBrains attribute text

/**
 ASCII code point of the first decimal digit `0`.
 */
const DIGIT_ZERO_CODE_POINT = 48;

/**
 ASCII code point of the last decimal digit `9`.
 */
const DIGIT_NINE_CODE_POINT = 57;

/**
 Decimal radix used when parsing `&#NN;` numeric entities.
 */
const DECIMAL_RADIX = 10;

/**
 Hexadecimal radix used when parsing `&#xNN;` numeric entities.
 */
const HEX_RADIX = 16;

/**
 Checks whether a code point is an ASCII decimal digit.
 
 @param codePoint - Code point to test.
 
 @returns Whether the code point is `0` through `9`.
 
 @example
 ```ts
 isDigitCodePoint({ codePoint: 53 });
 ```
 */
export function isDigitCodePoint({ codePoint, }: { readonly codePoint: number; },): boolean {
  return (codePoint >= DIGIT_ZERO_CODE_POINT) && (codePoint <= DIGIT_NINE_CODE_POINT);
}

/**
 Decodes one numeric XML entity code point.
 
 @param text - Numeric text after entity prefix.
 
 @param radix - Number base used by entity notation.
 
 @returns Decoded code point, or original entity syntax when invalid.
 
 @example
 ```ts
 decodeXmlCodePoint({ text: '10', radix: 10 });
 ```
 */
function decodeXmlCodePoint({
  text,
  radix,
}: {
  readonly radix: number;
  readonly text: string
},): string {
  /**
   Numeric code point parsed from the entity body.
   */
  const codePoint = Number.parseInt(
    text,
    radix,
  );
  if (Number.isNaN(codePoint,)) return `&#${text};`;
  try {
    return String.fromCodePoint(codePoint,);
  }
  catch (codePointError: unknown) {
    void codePointError;
    return `&#${text};`;
  }
}

/**
 Decodes one XML entity body, delegating numeric entities to
 {@link decodeXmlCodePoint}.
 
 @param entity - Entity body between ampersand and semicolon.
 
 @returns Decoded entity, or original entity syntax when unknown.
 
 @example
 ```ts
 decodeXmlEntity({ entity: 'quot' });
 ```
 */
function decodeXmlEntity({ entity, }: { readonly entity: string; },): string {
  if (entity === 'quot') return '"';
  if (entity === 'amp') return '&';
  if (entity === 'lt') return '<';
  if (entity === 'gt') return '>';
  if (entity === 'apos') return "'";
  if (entity.startsWith('#x',)) return decodeXmlCodePoint({
    text: entity.slice(2,),
    radix: HEX_RADIX,
  },);
  if (entity.startsWith('#',)) return decodeXmlCodePoint({
    text: entity.slice(1,),
    radix: DECIMAL_RADIX,
  },);
  return `&${entity};`;
}

/**
 Decodes XML attribute text used by JetBrains persistent-state files, via
 {@link decodeXmlEntity} for each entity found.
 
 @param value - XML attribute value without surrounding quote characters.
 
 @returns Decoded text.
 
 @example
 ```ts
 unescapeXmlAttribute({ value: '&quot;x&quot;' });
 ```
 */
export function unescapeXmlAttribute({ value, }: { readonly value: string; },): string {
  /**
   Accumulated decoded output.
   */
  let output = '';
  /**
   Scan cursor into the source value.
   */
  let cursorIndex = 0;
  while (cursorIndex < value.length) {
    /**
     Index of the next entity ampersand, or -1 when none remain.
     */
    const entityStart = value.indexOf(
      '&',
      cursorIndex,
    );
    if (entityStart === (-1)) {
      output += value.slice(cursorIndex,);
      break;
    }
    output += value.slice(
      cursorIndex,
      entityStart,
    );
    /**
     Index of the entity-terminating semicolon, or -1 when unterminated.
     */
    const entityEnd = value.indexOf(
      ';',
      entityStart + 1,
    );
    if (entityEnd === (-1)) {
      output += value.slice(entityStart,);
      break;
    }
    output += decodeXmlEntity({ entity: value.slice(
      entityStart + 1,
      entityEnd,
    ), },);
    cursorIndex = entityEnd + 1;
  }
  return output;
}

/**
 Encodes text for a double-quoted XML attribute.
 
 @param value - Raw attribute text.
 
 @returns XML-safe attribute value.
 
 @example
 ```ts
 escapeXmlAttribute({ value: '"x"' });
 ```
 */
export function escapeXmlAttribute({ value, }: { readonly value: string; },): string {
  /**
   Accumulated XML-safe output.
   */
  let output = '';
  for (const char of value) {
    if (char === '&') output += '&amp;';
    else if (char === '"') output += '&quot;';
    else if (char === '<') output += '&lt;';
    else if (char === '>') output += '&gt;';
    else if (char === '\n') output += '&#10;';
    else if (char === '\r') output += '&#13;';
    else if (char === '\t') output += '&#9;';
    else output += char;
  }
  return output;
}

/**
 Renders one XML option line.
 
 @param indent - Whitespace prefix for line.
 
 @param name - Option name.
 
 @param value - Raw option value, escaped via {@link escapeXmlAttribute}.
 
 @returns XML option line.
 
 @example
 ```ts
 xmlOptionLine({ indent: '  ', name: 'x', value: 'y' });
 ```
 */
export function xmlOptionLine(
  {
    indent,
    name,
    value,
  }: {
    readonly indent: string;
    readonly name: string;
    readonly value: string
  },
): string {
  return `${indent}<option name="${name}" value="${escapeXmlAttribute({ value, },)}" />`;
}

//endregion XML attribute coding
