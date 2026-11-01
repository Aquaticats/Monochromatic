/**
 Tests for sinon sandbox via TestContext and asymmetric matchers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

//region Shared prototype stub target

/* oxlint-disable no-restricted-syntax/no-class -- sinon prototype-stub isolation tests need a real `Greeter.prototype.greet` as the stub target; a factory returning a frozen object has no shared prototype to stub, so a class is the required fixture. */
/**
 Toy class whose prototype methods serve as safe stub targets
 for concurrent-isolation tests. Using a synthetic class avoids
 stubbing real third-party prototypes, which can leak into other
 test suites if a sandbox restore is missed.
 */
class Greeter {
  greet(_name: string,): string {
    return 'hi';
  }
}
/* oxlint-enable no-restricted-syntax/no-class */

//endregion

await describe({
  name: 'sinon via context',
  children: [
    it({
      name: 'stub and spy work with expect matchers',
      fn: async ({ sinon, },) => {
        const obj = { greet: (_name: string,): string => 'hi', };
        const stub = sinon.stub(obj, 'greet',).returns('hello',);

        obj.greet('world',);

        expect(stub,).toHaveBeenCalled();
        expect(stub,).toHaveBeenCalledTimes(1,);
        expect(stub,).toHaveBeenCalledWith('world',);
      },
    },),

    it({
      name: 'sandbox restores automatically between tests',
      fn: async ({ sinon, },) => {
        const obj = { getValue: (): number => 42, };
        const ORIGINAL = 42;
        const STUBBED = 99;

        sinon.stub(obj, 'getValue',).returns(STUBBED,);
        expect(obj.getValue(),).toBe(STUBBED,);

        // restore manually to verify the mechanism works within a test
        sinon.restore();
        expect(obj.getValue(),).toBe(ORIGINAL,);
      },
    },),

    it({
      name: 'asymmetric matchers work inside toHaveBeenCalledWith',
      fn: async ({ sinon, },) => {
        const spy = sinon.spy();

        spy('hello world', { id: 1, name: 'test', },);

        expect(spy,).toHaveBeenCalledWith(
          expect.stringContaining('hello',),
          expect.objectContaining({ id: 1, },),
        );
      },
    },),

    it({
      name: 'expect.anything matches any value',
      fn: async ({ sinon, },) => {
        const spy = sinon.spy();

        spy('specific', { deeply: 'nested', },);

        expect(spy,).toHaveBeenCalledWith('specific', expect.anything(),);
      },
    },),

    it({
      name: 'expect.any matches by constructor',
      fn: async ({ sinon, },) => {
        const spy = sinon.spy();

        spy(new Error('test',),);

        expect(spy,).toHaveBeenCalledWith(expect.any(Error,),);
      },
    },),

    it({
      name: 'expect.arrayContaining matches subset',
      fn: async ({ sinon, },) => {
        const spy = sinon.spy();

        spy([1, 2, 3, 4, 5,],);

        expect(spy,).toHaveBeenCalledWith(expect.arrayContaining([2, 4,],),);
      },
    },),

    it({
      name: 'expect.stringMatching matches by regex',
      fn: async ({ sinon, },) => {
        const spy = sinon.spy();

        spy('hello world',);

        // oxlint-disable-next-line no-restricted-syntax/no-regex -- this test exercises `expect.stringMatching` which is a regex matcher; the regex IS the test fixture.
        expect(spy,).toHaveBeenCalledWith(expect.stringMatching(/^hello/u,),);
      },
    },),

    it({
      name: 'toHaveReturnedWith checks return value',
      fn: async ({ sinon, },) => {
        const RETURN_VALUE = 42;
        const obj = { getValue: (): number => RETURN_VALUE, };
        const spy = sinon.spy(obj, 'getValue',);

        obj.getValue();

        expect(spy,).toHaveReturnedWith(RETURN_VALUE,);
      },
    },),
  ],
},);

await describe({
  name: 'sinon prototype stub isolation',
  // Sequential execution required: tests stub the same
  // prototype method and would fail under concurrent execution
  // because sinon refuses to wrap an already-wrapped method.
  concurrency: 1,
  children: [
    it({
      name: 'restores prototype stub before next test resolves (first test)',
      fn: async ({ sinon, },) => {
        sinon.stub(Greeter.prototype, 'greet',).returns('stubbed-a',);
        const instance = new Greeter();
        expect(instance.greet('x',),).toBe('stubbed-a',);
      },
    },),
    it({
      name: 'restores prototype stub before next test resolves (second test)',
      fn: async ({ sinon, },) => {
        // If the previous test's sandbox was not restored
        // before this test started, this stub call would
        // throw "Attempted to wrap greet which is already wrapped"
        sinon.stub(Greeter.prototype, 'greet',).returns('stubbed-b',);
        const instance = new Greeter();
        expect(instance.greet('y',),).toBe('stubbed-b',);
      },
    },),
    it({
      name: 'prototype is restored to original after all stub tests',
      fn: async () => {
        const instance = new Greeter();
        expect(instance.greet('z',),).toBe('hi',);
      },
    },),
  ],
},);
