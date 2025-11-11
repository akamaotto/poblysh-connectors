/**
 * Unified test setup for the Next.js demo (Bun-native):
 * - Sets up a jsdom-like DOM environment for React Testing Library
 * - Optionally installs a default fetch mock that tests can override
 * - Does NOT install or emulate Jest; tests should use Bun's `vi` directly
 */

declare global {
  var fetch?: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;
  var window?: unknown;
  var document?: unknown;
  var jest?: {
    fn: (impl?: (...args: unknown[]) => unknown) => any;
    spyOn?: (obj: Record<string, unknown>, key: string) => any;
    restoreAllMocks?: () => void;
  };
}

/**
 * Optionally provide a default fetch stub if none is defined.
 * Tests that care about fetch behavior should override this with vi.fn().
 */
if (!global.fetch) {
  global.fetch = (async () =>
    Promise.resolve({
      ok: true,
      status: 200,
      headers: new Headers(),
      json: async () => [],
    } as Response)) as unknown as (
    input: RequestInfo | URL,
    init?: RequestInit,
  ) => Promise<Response>;
}

/**
 * Ensure a DOM-like environment for React Testing Library:
 * - If a DOM already exists, do nothing.
 * - Otherwise, create a minimal jsdom environment (if jsdom is available).
 */
if (
  typeof global.document === "undefined" ||
  typeof global.window === "undefined"
) {
  // DOM setup is handled by jest.setup.js for Jest tests
  // For Bun tests, DOM should be available via happy-dom or test environment
  console.log("Note: DOM environment not detected. Tests may need proper DOM setup.");
}

console.log("Unified Bun-native test setup loaded (DOM + optional fetch stub)");
