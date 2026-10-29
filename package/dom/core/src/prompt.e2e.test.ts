/**
 E2e tests for `prompt`.
 
 Drives the live `<dialog>` element in a real browser via Playwright,
 since happy-dom and jsdom do not implement `<dialog>` semantics fully.
 
 @module
 */
import {
  expect,
  test,
} from '@playwright/test';

import { loadHarness, } from './test-setup.ts';

declare global {
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- `string | null` mirrors the `prompt` return type, which mirrors the native `globalThis.prompt` DOM API (string on OK, `null` on cancel)
  var pendingPrompt: Promise<string | null>;
}

test.describe('prompt', () => {
  test('OK with text resolves to the entered string', async ({ page, },) => {
    await loadHarness({ page, },);

    await page.evaluate(function openPrompt() {
      globalThis.pendingPrompt = globalThis.moduleDom.prompt({ message: 'Name?', },);
    },);

    await page.locator('dialog.prompt-polyfill-dialog',).waitFor();
    await page.locator('dialog.prompt-polyfill-dialog input',).fill('alice',);
    await page.locator('button.prompt-polyfill-ok',).click();

    const result = await page.evaluate(async function awaitPrompt() {
      return globalThis.pendingPrompt;
    },);

    expect(result,).toBe('alice',);
  });

  test('OK with empty input resolves to empty string (globalThis.prompt parity)', async ({ page, },) => {
    await loadHarness({ page, },);

    await page.evaluate(function openPrompt() {
      globalThis.pendingPrompt = globalThis.moduleDom.prompt({ message: 'Name?', },);
    },);

    await page.locator('dialog.prompt-polyfill-dialog',).waitFor();
    // input.value starts empty (no defaultValue passed); submit without typing.
    await page.locator('button.prompt-polyfill-ok',).click();

    const result = await page.evaluate(async function awaitPrompt() {
      return globalThis.pendingPrompt;
    },);

    expect(result,).toBe('',);
  });

  test('Cancel button resolves to null', async ({ page, },) => {
    await loadHarness({ page, },);

    await page.evaluate(function openPrompt() {
      globalThis.pendingPrompt = globalThis.moduleDom.prompt({ message: 'Name?', },);
    },);

    await page.locator('dialog.prompt-polyfill-dialog',).waitFor();
    await page.locator('button.prompt-polyfill-cancel',).click();

    const result = await page.evaluate(async function awaitPrompt() {
      return globalThis.pendingPrompt;
    },);

    expect(result,).toBeNull();
  });

  test('Esc resolves to null even when input has text', async ({ page, },) => {
    await loadHarness({ page, },);

    await page.evaluate(function openPrompt() {
      globalThis.pendingPrompt = globalThis.moduleDom.prompt({ message: 'Name?', },);
    },);

    await page.locator('dialog.prompt-polyfill-dialog',).waitFor();
    await page.locator('dialog.prompt-polyfill-dialog input',).fill('bob',);
    await page.keyboard.press('Escape',);

    const result = await page.evaluate(async function awaitPrompt() {
      return globalThis.pendingPrompt;
    },);

    expect(result,).toBeNull();
  });

  test('default class names are applied when classes option is omitted', async ({ page, },) => {
    await loadHarness({ page, },);

    await page.evaluate(function openPrompt() {
      globalThis.pendingPrompt = globalThis.moduleDom.prompt({ message: 'Name?', },);
    },);

    await page.locator('dialog.prompt-polyfill-dialog',).waitFor();
    await expect(page.locator('dialog.prompt-polyfill-dialog',),).toBeVisible();
    await expect(page.locator('button.prompt-polyfill-cancel',),).toBeVisible();
    await expect(page.locator('button.prompt-polyfill-ok',),).toBeVisible();

    await page.locator('button.prompt-polyfill-cancel',).click();
    await page.evaluate(async function awaitPrompt() {
      return globalThis.pendingPrompt;
    },);
  });

  test('classes option overrides default class names per call', async ({ page, },) => {
    await loadHarness({ page, },);

    await page.evaluate(function openPrompt() {
      globalThis.pendingPrompt = globalThis.moduleDom.prompt({
        message: 'Rename file',
        classes: { dialog: 'rename-dialog', ok: 'rename-ok', },
      },);
    },);

    await page.locator('dialog.rename-dialog',).waitFor();
    await expect(page.locator('dialog.rename-dialog',),).toBeVisible();
    await expect(page.locator('button.rename-ok',),).toBeVisible();
    // Cancel button still uses the default class since classes.cancel was not supplied.
    await expect(page.locator('button.prompt-polyfill-cancel',),).toBeVisible();

    await page.locator('button.prompt-polyfill-cancel',).click();
    await page.evaluate(async function awaitPrompt() {
      return globalThis.pendingPrompt;
    },);
  });
});
