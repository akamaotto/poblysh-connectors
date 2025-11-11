/**
 * Unified test setup for the Next.js demo (Bun-native):
 * - Sets up a jsdom-like DOM environment for React Testing Library
 * - Optionally installs a default fetch mock that tests can override
 * - Does NOT install or emulate Jest; tests should use Bun's `vi` directly
 */

export {};

console.log("Test setup loaded");
