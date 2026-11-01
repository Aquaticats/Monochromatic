/**
 Client-side entry point for the syllable break demo.
 
 Processes user-entered text through TeX hyphenation patterns,
 inserting zero-width spaces at syllable boundaries instead of soft hyphens.
 The browser then wraps at those boundaries without showing a hyphen character.
 */
import { hyphenateSync, } from 'hyphen/en-us';

/**
 Zero-width space character used as an invisible break opportunity.
 */
const ZWS = '\u200B';

/**
 Inserts zero-width spaces at syllable boundaries in the given text.
 
 Uses TeX hyphenation patterns (via {@link hyphenateSync}) to find syllable
 boundaries, but replaces the default soft hyphen with a {@link ZWS} so the
 browser can break there without rendering a visible hyphen.
 
 @param text - plain text to process
 
 @returns text with zero-width spaces at syllable boundaries
 
 @example
 ```ts
 insertBreakOpportunities('extraordinary');
 // 'ex\u200Btra\u200Bor\u200Bdi\u200Bnary'
 ```
 */
function insertBreakOpportunities(text: string,): string {
  return hyphenateSync(
    text,
    { hyphenChar: ZWS, },
  );
}

/**
 Updates all output containers with hyphenated (via {@link insertBreakOpportunities})
 and plain versions of the input text.
 
 @param text - raw user input
 */
function updateOutputs(text: string,): void {
  /**
   Hyphenated version of the input, computed once and shared across all output containers.
   */
  const processed = insertBreakOpportunities(text,);

  /**
   Output container that renders text with zero-width spaces inserted at syllable boundaries.
   */
  const zwsOutput = document.querySelector<HTMLElement>('#output-zws',);
  /**
   Output container that renders raw text styled with CSS hyphens: auto, for comparison.
   */
  const hyphensAutoOutput = document.querySelector<HTMLElement>('#output-hyphens-auto',);
  /**
   Output container that renders raw text without any break opportunities, as a control.
   */
  const plainOutput = document.querySelector<HTMLElement>('#output-plain',);
  /**
   Debug view that surfaces the otherwise-invisible zero-width spaces as middle-dot markers.
   */
  const processedDisplay = document.querySelector<HTMLElement>('#processed-text',);

  if (zwsOutput)
    zwsOutput.textContent = processed;
  if (hyphensAutoOutput)
    hyphensAutoOutput.textContent = text;
  if (plainOutput)
    plainOutput.textContent = text;
  if (processedDisplay) {
    /**
     Show the zero-width spaces as visible markers for debugging
     */
    processedDisplay.textContent = processed.replaceAll(
      ZWS,
      '\u00B7',
    );
  }
}

/**
 Reads the width slider value and applies it to all output containers.
 */
function updateWidth(): void {
  /**
   Width slider that drives the inline-size of every output container.
   */
  const slider = document.querySelector<HTMLInputElement>('#width-slider',);
  /**
   Label that mirrors the current slider value next to the slider control.
   */
  const widthLabel = document.querySelector<HTMLElement>('#width-label',);
  if (!slider)
    return;

  /**
   Slider position formatted as a ch-unit length, shared by every output container and the label.
   */
  const value = `${slider.value}ch`;
  for (const id of [
    'output-zws',
    'output-hyphens-auto',
    'output-plain',
  ]) {
    /**
     Current output container in the iteration, resolved from its id.
     */
    const el = document.querySelector<HTMLElement>(`#${id}`,);
    if (el)
      el.style
        .inlineSize = value;
  }
  if (widthLabel)
    widthLabel.textContent = value;
}

/**
 Default sample text demonstrating scientific terminology that CSS hyphens: auto may not cover.
 */
const DEFAULT_TEXT = 'Ribulose-1,5-bisphosphate carboxylase/oxygenase';

/**
 Input textarea where the user types or pastes the text to process.
 */
const textarea = document.querySelector<HTMLTextAreaElement>('#input-text',);
/**
 Range slider that adjusts the width of the output containers.
 */
const slider = document.querySelector<HTMLInputElement>('#width-slider',);

if (textarea) {
  textarea.value = DEFAULT_TEXT;
  textarea.addEventListener(
    'input',
    function handleInput(): void {
      updateOutputs(textarea.value,);
    },
  );
}

if (slider) {
  slider.addEventListener(
    'input',
    function handleSlider(): void {
      updateWidth();
    },
  );
}

updateOutputs(DEFAULT_TEXT,);
updateWidth();
