import '@testing-library/jest-dom/vitest'

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: query === '(min-width: 1025px)',
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})
