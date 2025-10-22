import '@testing-library/jest-dom';
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
  default: vi.fn(({ value, onChange, height = '400px' }) => {
    return {
      type: 'div',
      props: {
        'data-testid': 'monaco-editor',
        children: [
          {
            type: 'textarea',
            props: {
              value,
              onChange: (e: any) => onChange?.(e.target.value),
              style: { height, width: '100%' },
            },
          },
        ],
      },
    };
  }),
}));

// Mock date-fns
vi.mock('date-fns', () => ({
  format: vi.fn((date) => new Date(date).toISOString()),
  formatDistanceToNow: vi.fn(() => 'a few seconds ago'),
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => {
  const MockIcon = (props: any) => {
    const { className, ...rest } = props;
    return {
      type: 'svg',
      props: { className, ...rest },
    };
  };
  return {
    Plus: MockIcon,
    Edit2: MockIcon,
    Trash2: MockIcon,
    ChevronRight: MockIcon,
    Save: MockIcon,
    X: MockIcon,
    AlertCircle: MockIcon,
    CheckCircle: MockIcon,
    AlertTriangle: MockIcon,
    Info: MockIcon,
  };
});

// Mock React Router
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
  useParams: () => ({}),
  useLocation: () => ({ pathname: '/' }),
  Link: ({ children, to }: any) => children,
}));

// Mock React Query
vi.mock('@tanstack/react-query', () => ({
  useQuery: vi.fn(({ queryFn }) => ({
    data: undefined,
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  })),
  useMutation: vi.fn(() => ({
    mutate: vi.fn(),
    mutateAsync: vi.fn(),
    isPending: false,
    error: null,
    data: undefined,
  })),
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));

// Mock Zustand
vi.mock('zustand', () => ({
  create: (fn: any) => {
    const state = fn(() => {}, () => {});
    return () => state;
  },
}));

// Mock clsx
vi.mock('clsx', () => ({
  default: (...args: any[]) => args.filter(Boolean).join(' '),
}));

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
