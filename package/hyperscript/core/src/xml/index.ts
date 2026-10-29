/**
 String-returning hyperscript factory for XML generation.
 
 Same API shape as the HTML `h()` (`t string/t html/.../$`) but produces
 well-formed XML instead of HTML. Key differences from `h-html`:
 
 - No void elements: childless elements self-close (`<tag />`)
 - No `class` or `style` shortcuts (HTML/CSS concepts, not XML)
 - Supports XML namespace prefixes in tag names and attributes
 
 Useful for generating RSS/Atom feeds, XML configuration files,
 SOAP payloads, and other structured XML documents.
 
 @param options - Named parameters describing the element
 @param options.tag - XML tag name, may include namespace prefix (e.g. `'atom:link'`)
 @param options.text - Text content (XML-escaped automatically)
 @param options.raw - Raw inner XML (NOT escaped; caller is responsible for well-formedness)
 @param options.attrs - Record of attributes (values are XML-escaped)
 @param options.children - Child XML strings to concatenate inside the element
 @returns XML string
 
 @example Simple element
 ```ts
 const xml = $({ tag: 'title', text: 'My Feed' });
 // '<title>My Feed</title>'
 ```
 
 @example Self-closing element
 ```ts
 const xml = $({ tag: 'link', attrs: { href: 'https://example.com' } });
 // '<link href="https://example.com" />'
 ```
 
 @example Namespaced elements
 ```ts
 const xml = $({
   tag: 'atom:feed',
   attrs: { 'xmlns:atom': 'http://www.w3.org/2005/Atom' },
   children: [
     $({ tag: 'atom:title', text: 'My Feed' }),
     $({ tag: 'atom:link', attrs: { href: 'https://example.com' } }),
   ],
 });
 ```
 
 @example Text escaping prevents malformed XML
 ```ts
 const xml = $({ tag: 'content', text: 'x < y & z > w' });
 // '<content>x &lt; y &amp; z &gt; w</content>'
 ```
 */

/**
 Escapes characters that have special meaning in XML attribute values and text content.
 
 Handles the five predefined XML entities.
 
 @param raw - unescaped string
 
 @returns escaped string safe for XML text content and attribute values
 
 @example
 ```ts
 escapeXml('x < y & z') // 'x &lt; y &amp; z'
 ```
 */
function escapeXml(raw: string,): string {
  return raw
    .replaceAll(
    '&',
    '&amp;',
    )
    .replaceAll(
    '<',
    '&lt;',
    )
    .replaceAll(
    '>',
    '&gt;',
    )
    .replaceAll(
    '"',
    '&quot;',
    )
    .replaceAll(
    "'",
    '&apos;',
    );
}

/**
 Named parameters for XML element creation.
 
 Intentionally omits `class`, `style`, and event listeners; those are
 HTML/DOM concepts with no meaning in generic XML.
 */
type XmlOptions = {
  /**
   XML tag name, may include namespace prefix (e.g. `'atom:link'`)
   */
  readonly tag: string;
  /**
   Text content (XML-escaped automatically)
   */
  readonly text?: string;
  /**
   Raw inner XML (NOT escaped; caller is responsible for well-formedness)
   */
  readonly raw?: string;
  /**
   Attributes set as key="value" pairs (values are XML-escaped)
   */
  readonly attrs?: Readonly<Record<string, string>>;
  /**
   Child XML strings to concatenate inside the element
   */
  readonly children?: readonly string[];
};

/**
 Creates an XML element string from declarative options.
 
 Elements with no content (no `text`, `raw`, or `children`) produce
 self-closing tags (`<tag />`). Otherwise produces `<tag>...</tag>`.
 Attribute values and text content pass through {@link escapeXml}.
 
 @param tag - XML element name
 
 @param text - text content (XML-escaped automatically)
 
 @param raw - raw inner XML (NOT escaped)
 
 @param attrs - attributes as key="value" pairs
 
 @param children - child XML strings to concatenate
 
 @returns well-formed XML element string
 
 @example
 ```ts
 $({ tag: 'item', attrs: { id: '1' }, text: 'hello' });
 // '<item id="1">hello</item>'
 ```
 */
/* @__NO_SIDE_EFFECTS__ */ export function $(
  {
    tag,
    text,
    raw,
    attrs,
    children,
  }: XmlOptions,
): string {
  /**
   Accumulates serialization fragments so they can be joined once at the end without intermediate string concatenations.
   */
  const parts: string[] = [`<${tag}`,];

  if (attrs !== undefined) {
    for (const [key, value,] of Object.entries(attrs,))
      parts.push(` ${key}="${escapeXml(value,)}"`,);
  }

  //region Self-closing check
  // XML has no void elements; instead, childless elements self-close.
  /**
   Flags whether the element has body content so the caller can choose between self-closing and the open/close pair.
   */
  const hasContent = (text !== undefined)
    || (raw !== undefined)
    || ((children !== undefined) && (children.length
      > 0));

  if (!hasContent) {
    parts.push(' />',);
    return parts.join('',);
  }
  //endregion

  parts.push('>',);

  if (text !== undefined)
    parts.push(escapeXml(text,),);

  if (raw !== undefined)
    parts.push(raw,);

  if (children !== undefined) {
    for (const child of children)
      parts.push(child,);
  }

  parts.push(`</${tag}>`,);

  return parts.join('',);
}
