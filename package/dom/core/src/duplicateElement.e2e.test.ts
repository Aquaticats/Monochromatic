/**
 E2e tests for `replicateElementAsParentContent`, `replicateElementAsContentOf`,
 and `deepCloneNode`.
 
 @module
 */
import {
  expect,
  test,
} from '@playwright/test';

import { loadHarness, } from './test-setup.ts';

test.describe('replicateElementAsParentContent', () => {
  test("replaces the parent's children with the requested number of clones", async ({ page, },) => {
    await loadHarness({ page, },);

    const result = await page.evaluate(function exerciseHappyPath() {
      const container = document.createElement('div',);
      container.innerHTML =
        '<span>other</span><p class="tpl">orig</p><span>another</span>';
      document.body.append(container,);

      const template = container.querySelector<HTMLElement>('.tpl',);
      if (template === null)
        throw new Error('test setup: template not found',);

      globalThis.moduleDom.replicateElementAsParentContent({
        templateElement: template,
        targetCount: 3,
      },);

      return {
        total: container.children.length,
        allMatchTemplate: [...container.children,].every(function isTpl(child,) {
          return child.classList.contains('tpl',);
        },),
      };
    },);

    expect(result.total,).toBe(3,);
    expect(result.allMatchTemplate,).toBe(true,);
  });

  test('throws when the template has no parent', async ({ page, },) => {
    await loadHarness({ page, },);

    const errored = await page.evaluate(function callDetached() {
      const detached = document.createElement('p',);
      detached.textContent = 'detached';
      try {
        globalThis.moduleDom.replicateElementAsParentContent({
          templateElement: detached,
          targetCount: 1,
        },);
        return false;
      }
      catch (error) {
        return (Error.isError(error,)) && error.message.includes('has no parent',);
      }
    },);

    expect(errored,).toBe(true,);
  });

  test('empties the parent when targetCount is 0', async ({ page, },) => {
    await loadHarness({ page, },);

    const remaining = await page.evaluate(function callZero() {
      const container = document.createElement('div',);
      container.innerHTML = '<span>a</span><p class="tpl">orig</p><span>b</span>';
      document.body.append(container,);
      const template = container.querySelector<HTMLElement>('.tpl',);
      if (template === null)
        throw new Error('test setup: template not found',);

      globalThis.moduleDom.replicateElementAsParentContent({
        templateElement: template,
        targetCount: 0,
      },);

      return container.children.length;
    },);

    expect(remaining,).toBe(0,);
  });
});

test.describe('replicateElementAsContentOf', () => {
  test('fills the explicit parent without moving the template', async ({ page, },) => {
    await loadHarness({ page, },);

    const result = await page.evaluate(function exerciseHappyPath() {
      const templateHost = document.createElement('div',);
      templateHost.id = 'template-host';
      templateHost.innerHTML = '<li class="row">tpl</li>';
      document.body.append(templateHost,);

      const list = document.createElement('ul',);
      list.id = 'list';
      document.body.append(list,);

      const template = templateHost.querySelector<HTMLElement>('.row',);
      if (template === null)
        throw new Error('test setup: template not found',);

      globalThis.moduleDom.replicateElementAsContentOf({
        templateElement: template,
        parentElement: list,
        targetCount: 3,
      },);

      return {
        listChildren: list.children.length,
        templateStillInHost: templateHost.contains(template,),
      };
    },);

    expect(result.listChildren,).toBe(3,);
    expect(result.templateStillInHost,).toBe(true,);
  });
});

test.describe('deepCloneNode', () => {
  test('preserves descendants and attributes', async ({ page, },) => {
    await loadHarness({ page, },);

    const result = await page.evaluate(function exerciseClone() {
      const original = document.createElement('a',);
      original.href = 'https://example.com/path';
      original.className = 'primary';
      original.innerHTML = '<span data-flag="x">child</span>';

      const clone = globalThis.moduleDom.deepCloneNode(original,);

      return {
        sameType: clone instanceof HTMLAnchorElement,
        notSameRef: clone !== original,
        href: clone.href,
        cls: clone.className,
        innerChildCount: clone.children.length,
        innerSpanFlag: (clone.children[0] as HTMLElement).dataset.flag,
      };
    },);

    expect(result.sameType,).toBe(true,);
    expect(result.notSameRef,).toBe(true,);
    expect(result.href,).toBe('https://example.com/path',);
    expect(result.cls,).toBe('primary',);
    expect(result.innerChildCount,).toBe(1,);
    expect(result.innerSpanFlag,).toBe('x',);
  });
});
