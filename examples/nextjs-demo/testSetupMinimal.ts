/**
 * Fixed test setup that doesn't hang
 *
 * This file now only provides a minimal Jest-style shim for environments
 * that do not already have `jest`. The main test setup logic lives in
 * `testSetup.ts` to avoid duplicate and conflicting shims.
 */

// Use global type from testSetup.ts - no need to redeclare

interface JestMockFunction {
  (...args: unknown[]): unknown;
  mock: {
    calls: unknown[][];
  };
  mockReturnValue(value: unknown): JestMockFunction;
  mockResolvedValue(value: unknown): JestMockFunction;
  mockRejectedValue(value: unknown): JestMockFunction;
  mockImplementation(fn: (...args: unknown[]) => unknown): JestMockFunction;
  mockReset(): void;
  mockRestore(): void;
}

// Only install a shim if `jest` is not already defined
if (!global.jest) {
  const jestShim = {
    fn(impl?: (...args: unknown[]) => unknown): JestMockFunction {
      const mockCalls: unknown[][] = [];

      const mockFn = (...args: unknown[]): unknown => {
        mockCalls.push(args);
        return impl ? impl(...args) : undefined;
      };

      const mockFnWithMethods = mockFn as JestMockFunction;
      mockFnWithMethods.mock = { calls: mockCalls };

      mockFnWithMethods.mockReturnValue = (
        value: unknown,
      ): JestMockFunction => {
        return jestShim.fn(() => value);
      };

      mockFnWithMethods.mockResolvedValue = (
        value: unknown,
      ): JestMockFunction => {
        return jestShim.fn(() => Promise.resolve(value));
      };

      mockFnWithMethods.mockRejectedValue = (
        value: unknown,
      ): JestMockFunction => {
        return jestShim.fn(() => Promise.reject(value));
      };

      mockFnWithMethods.mockImplementation = (
        fn: (...args: unknown[]) => unknown,
      ): JestMockFunction => {
        return jestShim.fn(fn);
      };

      mockFnWithMethods.mockReset = (): void => {
        mockCalls.length = 0;
      };

      mockFnWithMethods.mockRestore = (): void => {
        mockCalls.length = 0;
      };

      return mockFnWithMethods;
    },

    spyOn(obj: Record<string, unknown>, key: string): JestMockFunction {
      const original = obj[key];
      const mockFn = jestShim.fn();
      obj[key] = mockFn as unknown;

      mockFn.mockRestore = () => {
        obj[key] = original;
      };

      return mockFn;
    },

    restoreAllMocks(): void {
      // No-op here; individual mocks should use mockRestore where needed.
    },
  };

  global.jest = jestShim as any; // Type assertion to avoid missing properties error
}

console.log("Fixed minimal test setup shim loaded");
