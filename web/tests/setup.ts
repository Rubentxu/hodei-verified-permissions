import '@testing-library/jest-dom';
import React from 'react';
import { afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';

// Cleanup after each test
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  disconnect() {}
  observe() {}
  takeRecords() {
    return [];
  }
  unobserve() {}
} as any;

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  constructor() {}
  disconnect() {}
  observe() {}
  unobserve() {}
} as any;

// Mock Monaco Editor
vi.mock('@monaco-editor/react', () => ({
  default: vi.fn(({ value, onChange, height = '400px', ...rest }) => {
    return React.createElement('textarea', {
      'data-testid': 'monaco-editor',
      className: 'monaco-editor',
      value,
      onChange: (e: any) => onChange?.(e.target.value),
      style: { height, width: '100%' },
      ...rest,
    });
  }),
}));

// Mock date-fns
vi.mock('date-fns', () => ({
  format: vi.fn((date) => new Date(date).toISOString()),
  formatDistanceToNow: vi.fn(() => 'a few seconds ago'),
}));

// Use real lucide-react icons (no mock)

// Mock React Router
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
  useParams: () => ({}),
  useLocation: () => ({ pathname: '/' }),
  Link: ({ children, to, ...rest }: any) => React.createElement('a', { href: typeof to === 'string' ? to : '#', ...rest }, children),
}));

// (No global mock for React Query) - use real library; tests that need provider should wrap explicitly.

// Mock Zustand
vi.mock('zustand', () => ({
  create: (fn: any) => {
    const state = fn(() => {}, () => {});
    return () => state;
  },
}));

// Mock clsx (provide both default and named export)
vi.mock('clsx', () => {
  const fn = (...args: any[]) => args.filter(Boolean).join(' ');
  return {
    default: fn,
    clsx: fn,
  };
});

// Mock class-variance-authority
vi.mock('class-variance-authority', () => ({
  cva: (base: string) => () => base,
}));

// Setup global fetch mock
global.fetch = vi.fn(() =>
  Promise.resolve({
    ok: true,
    json: () => Promise.resolve({}),
  })
) as any;

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  error: vi.fn(),
  warn: vi.fn(),
  log: vi.fn(),
};

// Mock window.scrollTo
window.scrollTo = vi.fn();

// Mock HTMLElement methods
HTMLElement.prototype.scrollIntoView = vi.fn();

// Setup default test timeout
vi.setConfig({ testTimeout: 10000 });
