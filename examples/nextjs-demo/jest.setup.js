import '@testing-library/jest-dom'

// Mock Next.js router
jest.mock('next/navigation', () => ({
  useRouter: () => ({
    push: jest.fn(),
    replace: jest.fn(),
    pathname: '/',
    query: {},
    asPath: '/',
  }),
  useSearchParams: () => new URLSearchParams(),
  usePathname: () => '/',
}))

// Mock IntersectionObserver
global.IntersectionObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}))

// Mock ResizeObserver
global.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}))

// Mock window.location
Object.defineProperty(window, 'location', {
  value: {
    href: 'http://localhost:3000',
  },
  writable: true,
})

// Set up global fetch mock for SharedBackendClient tests
const mockFetch = jest.fn(() => Promise.resolve({
  ok: true,
  status: 200,
  headers: new Headers(),
  json: () => Promise.resolve([])
}))

// Ensure the mock always returns a valid Response even when specific mock calls are exhausted
mockFetch.mockImplementation(() => Promise.resolve({
  ok: true,
  status: 200,
  headers: new Headers(),
  json: () => Promise.resolve([])
}))

global.fetch = mockFetch

// Mock Headers constructor if not available
if (typeof Headers === 'undefined') {
  global.Headers = class Headers {
    constructor(init = {}) {
      this.headers = new Map()
      if (typeof init === 'object') {
        Object.entries(init).forEach(([key, value]) => {
          this.headers.set(key, value)
        })
      }
    }

    get(name) {
      return this.headers.get(name.toLowerCase())
    }

    set(name, value) {
      this.headers.set(name.toLowerCase(), value)
    }

    has(name) {
      return this.headers.has(name.toLowerCase())
    }

    delete(name) {
      this.headers.delete(name.toLowerCase())
    }

    entries() {
      return this.headers.entries()
    }
  }
}

// Provide compatibility shim for SharedBackendClient tests
// This makes the test work with both Jest and Bun
globalThis.jest = {
  fn: jest.fn,
  restoreAllMocks: jest.restoreAllMocks,
  spyOn: jest.spyOn
}

// Helper function for tests to create mock error responses
globalThis.createMockErrorResponse = (status, message, code = null) => ({
  ok: false,
  status,
  headers: new Headers(),
  json: () => Promise.resolve({
    message,
    ...(code && { code })
  })
})