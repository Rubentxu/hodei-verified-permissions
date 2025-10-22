# 🎨 WEB ARCHITECTURE PLAN - Frontend Implementation

**Fecha:** 22 de Octubre de 2025, 22:45  
**Proyecto:** Hodei Verified Permissions - Web UI  
**Stack:** React 18 + TypeScript + Vite

---

## 📁 ESTRUCTURA DE DIRECTORIOS

```
web/
├── public/                          # Assets estáticos
│   ├── favicon.ico
│   ├── logo.svg
│   └── index.html
│
├── src/
│   ├── main.tsx                     # Entry point
│   ├── App.tsx                      # Root component
│   ├── vite-env.d.ts
│   │
│   ├── api/                         # gRPC client & API
│   │   ├── client.ts                # gRPC-Web client setup
│   │   ├── hooks/                   # API hooks (React Query)
│   │   │   ├── usePolicyStores.ts
│   │   │   ├── usePolicies.ts
│   │   │   ├── useSchemas.ts
│   │   │   ├── usePlayground.ts
│   │   │   └── index.ts
│   │   └── types.ts                 # API types & interfaces
│   │
│   ├── components/                  # Reusable components
│   │   ├── common/                  # Shared components
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Layout.tsx
│   │   │   ├── LoadingSpinner.tsx
│   │   │   ├── ErrorBoundary.tsx
│   │   │   └── index.ts
│   │   │
│   │   ├── editors/                 # Code editors
│   │   │   ├── CodeEditor.tsx       # Monaco wrapper
│   │   │   ├── JsonEditor.tsx
│   │   │   ├── CedarEditor.tsx
│   │   │   └── index.ts
│   │   │
│   │   ├── forms/                   # Form components
│   │   │   ├── PolicyStoreForm.tsx
│   │   │   ├── PolicyForm.tsx
│   │   │   ├── SchemaForm.tsx
│   │   │   ├── PlaygroundForm.tsx
│   │   │   └── index.ts
│   │   │
│   │   ├── tables/                  # Data tables
│   │   │   ├── PolicyStoresTable.tsx
│   │   │   ├── PoliciesTable.tsx
│   │   │   ├── TemplatesTable.tsx
│   │   │   └── index.ts
│   │   │
│   │   └── results/                 # Result displays
│   │       ├── AuthorizationResult.tsx
│   │       ├── ValidationResult.tsx
│   │       └── index.ts
│   │
│   ├── features/                    # Feature modules (Vertical Slices)
│   │   ├── policy-stores/           # Épica 14
│   │   │   ├── pages/
│   │   │   │   ├── PolicyStoresPage.tsx
│   │   │   │   ├── PolicyStoreDetailPage.tsx
│   │   │   │   └── CreatePolicyStorePage.tsx
│   │   │   ├── components/
│   │   │   │   ├── PolicyStoreCard.tsx
│   │   │   │   ├── PolicyStoreList.tsx
│   │   │   │   └── PolicyStoreModal.tsx
│   │   │   ├── hooks/
│   │   │   │   └── usePolicyStores.ts
│   │   │   └── index.ts
│   │   │
│   │   ├── schema-editor/           # Épica 15
│   │   │   ├── pages/
│   │   │   │   └── SchemaEditorPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── SchemaEditor.tsx
│   │   │   │   ├── SchemaValidator.tsx
│   │   │   │   └── SchemaPreview.tsx
│   │   │   ├── hooks/
│   │   │   │   └── useSchemaEditor.ts
│   │   │   └── index.ts
│   │   │
│   │   ├── policy-editor/           # Épica 16
│   │   │   ├── pages/
│   │   │   │   ├── PoliciesPage.tsx
│   │   │   │   ├── CreatePolicyPage.tsx
│   │   │   │   └── EditPolicyPage.tsx
│   │   │   ├── components/
│   │   │   │   ├── PolicyEditor.tsx
│   │   │   │   ├── PolicyList.tsx
│   │   │   │   ├── PolicyValidator.tsx
│   │   │   │   └── PolicyFilters.tsx
│   │   │   ├── hooks/
│   │   │   │   └── usePolicyEditor.ts
│   │   │   └── index.ts
│   │   │
│   │   └── playground/              # Épica 17
│   │       ├── pages/
│   │       │   └── PlaygroundPage.tsx
│   │       ├── components/
│   │       │   ├── PlaygroundForm.tsx
│   │       │   ├── EntityBuilder.tsx
│   │       │   ├── ContextEditor.tsx
│   │       │   ├── ResultsPanel.tsx
│   │       │   └── DecisionVisualization.tsx
│   │       ├── hooks/
│   │       │   └── usePlayground.ts
│   │       └── index.ts
│   │
│   ├── hooks/                       # Custom hooks
│   │   ├── useAuth.ts               # Authentication
│   │   ├── useNotification.ts       # Toast notifications
│   │   ├── useLocalStorage.ts       # Persistence
│   │   ├── useDebounce.ts           # Debouncing
│   │   └── index.ts
│   │
│   ├── store/                       # State management (Zustand)
│   │   ├── authStore.ts             # Auth state
│   │   ├── uiStore.ts               # UI state
│   │   ├── policyStoreStore.ts      # Policy Store state
│   │   └── index.ts
│   │
│   ├── utils/                       # Utility functions
│   │   ├── formatters.ts            # Date, code formatting
│   │   ├── validators.ts            # Input validation
│   │   ├── cedar-helpers.ts         # Cedar-specific helpers
│   │   ├── error-handler.ts         # Error handling
│   │   └── index.ts
│   │
│   ├── styles/                      # Global styles
│   │   ├── globals.css              # Global styles
│   │   ├── variables.css            # CSS variables
│   │   └── tailwind.config.ts       # Tailwind config
│   │
│   ├── types/                       # TypeScript types
│   │   ├── api.ts                   # API types
│   │   ├── domain.ts                # Domain types
│   │   ├── ui.ts                    # UI types
│   │   └── index.ts
│   │
│   └── router/                      # Routing
│       ├── routes.tsx               # Route definitions
│       ├── ProtectedRoute.tsx       # Auth guard
│       └── index.ts
│
├── tests/                           # Tests
│   ├── unit/                        # Unit tests
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── utils/
│   │   └── store/
│   ├── integration/                 # Integration tests
│   │   ├── features/
│   │   └── api/
│   ├── e2e/                         # E2E tests (Playwright)
│   │   ├── policy-stores.spec.ts
│   │   ├── schema-editor.spec.ts
│   │   ├── policy-editor.spec.ts
│   │   └── playground.spec.ts
│   └── setup.ts                     # Test setup
│
├── .env.example                     # Environment variables template
├── .env.local                       # Local environment (gitignored)
├── .eslintrc.json                  # ESLint config
├── .prettierrc                      # Prettier config
├── tsconfig.json                    # TypeScript config
├── vite.config.ts                   # Vite config
├── vitest.config.ts                 # Vitest config
├── playwright.config.ts             # Playwright config
├── package.json
└── README.md
```

---

## 🏗️ PATRONES Y BUENAS PRÁCTICAS

### 1. Component Architecture

#### Presentational Components (Dumb)
```typescript
// components/common/Button.tsx
interface ButtonProps {
  label: string;
  onClick: () => void;
  variant?: 'primary' | 'secondary';
  disabled?: boolean;
}

export const Button: React.FC<ButtonProps> = ({ label, onClick, variant = 'primary', disabled }) => (
  <button className={`btn btn-${variant}`} onClick={onClick} disabled={disabled}>
    {label}
  </button>
);
```

#### Container Components (Smart)
```typescript
// features/policy-stores/pages/PolicyStoresPage.tsx
export const PolicyStoresPage: React.FC = () => {
  const { data, isLoading, error } = usePolicyStores();
  
  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorAlert error={error} />;
  
  return <PolicyStoresList stores={data} />;
};
```

### 2. Custom Hooks Pattern

```typescript
// api/hooks/usePolicyStores.ts
export const usePolicyStores = () => {
  return useQuery({
    queryKey: ['policyStores'],
    queryFn: async () => {
      const client = getGrpcClient();
      return client.listPolicyStores(null, null);
    },
  });
};

export const useCreatePolicyStore = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (description: string) => {
      const client = getGrpcClient();
      return client.createPolicyStore(description);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['policyStores'] });
    },
  });
};
```

### 3. State Management (Zustand)

```typescript
// store/policyStoreStore.ts
interface PolicyStoreState {
  selectedStoreId: string | null;
  setSelectedStoreId: (id: string) => void;
  filters: PolicyStoreFilters;
  setFilters: (filters: PolicyStoreFilters) => void;
}

export const usePolicyStoreStore = create<PolicyStoreState>((set) => ({
  selectedStoreId: null,
  setSelectedStoreId: (id) => set({ selectedStoreId: id }),
  filters: {},
  setFilters: (filters) => set({ filters }),
}));
```

### 4. Error Handling

```typescript
// utils/error-handler.ts
export const handleApiError = (error: unknown): string => {
  if (error instanceof GrpcError) {
    return `gRPC Error: ${error.message}`;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return 'Unknown error occurred';
};
```

### 5. Type Safety

```typescript
// types/api.ts
export interface PolicyStore {
  id: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreatePolicyStoreRequest {
  description?: string;
}

export interface ApiResponse<T> {
  data: T;
  error?: string;
  timestamp: string;
}
```

### 6. Feature-Based Organization

Cada feature (Épica) es independiente:
- Tiene sus propias páginas
- Tiene sus propios componentes
- Tiene sus propios hooks
- Puede ser desarrollada en paralelo

### 7. Testing Strategy

```typescript
// tests/unit/components/Button.test.tsx
describe('Button Component', () => {
  it('should render with label', () => {
    render(<Button label="Click me" onClick={vi.fn()} />);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('should call onClick when clicked', () => {
    const onClick = vi.fn();
    render(<Button label="Click me" onClick={onClick} />);
    fireEvent.click(screen.getByText('Click me'));
    expect(onClick).toHaveBeenCalled();
  });
});
```

---

## 📦 DEPENDENCIES

### Core
- `react@18.2.0`
- `react-dom@18.2.0`
- `typescript@5.2.0`
- `vite@4.4.0`

### UI & Styling
- `@shadcn/ui` - Component library
- `tailwindcss@3.3.0` - Styling
- `lucide-react` - Icons
- `@monaco-editor/react` - Code editor

### State Management
- `zustand@4.4.0` - State management
- `@tanstack/react-query@4.32.0` - Data fetching

### gRPC
- `@grpc-web/grpc-web@1.4.0` - gRPC-Web client
- `google-protobuf@3.21.0` - Protocol Buffers

### Forms & Validation
- `react-hook-form@7.45.0` - Form management
- `zod@3.22.0` - Schema validation

### Utilities
- `date-fns@2.30.0` - Date utilities
- `clsx@2.0.0` - Class name utility
- `axios@1.5.0` - HTTP client

### Development
- `@vitejs/plugin-react@4.0.0`
- `@types/react@18.2.0`
- `@types/react-dom@18.2.0`
- `eslint@8.48.0`
- `prettier@3.0.0`
- `vitest@0.34.0`
- `@testing-library/react@14.0.0`
- `@testing-library/jest-dom@6.1.0`
- `playwright@1.38.0`

---

## 🎯 CONVENCIONES DE CÓDIGO

### Naming
- **Components:** PascalCase (PolicyStoreCard.tsx)
- **Hooks:** camelCase con prefijo `use` (usePolicyStores.ts)
- **Utils:** camelCase (formatters.ts)
- **Types:** PascalCase (PolicyStore.ts)
- **Constants:** UPPER_SNAKE_CASE (API_ENDPOINT)

### File Organization
- Un componente principal por archivo
- Componentes relacionados en carpetas
- Tests junto a código (*.test.tsx)
- Exports en index.ts para facilitar imports

### Code Style
- ESLint + Prettier
- TypeScript strict mode
- No `any` types
- Prefer `const` over `let`
- Arrow functions para callbacks

### Comments
- JSDoc para funciones públicas
- Explicar el "por qué", no el "qué"
- TODO comments con contexto

---

## 🚀 FASES DE IMPLEMENTACIÓN

### Fase 1: Setup (2-3 horas)
- [ ] Crear proyecto Vite + React
- [ ] Configurar TypeScript
- [ ] Instalar dependencias
- [ ] Configurar ESLint + Prettier
- [ ] Configurar gRPC-Web

### Fase 2: Componentes Base (3-4 horas)
- [ ] Layout (Header, Sidebar)
- [ ] Componentes comunes (Button, Input, etc.)
- [ ] Code editors (Monaco)
- [ ] Error handling

### Fase 3: Épica 14 - Policy Stores (2-3 horas)
- [ ] Lista de Policy Stores
- [ ] Crear Policy Store
- [ ] Detalles de Policy Store

### Fase 4: Épica 15 - Schema Editor (3-4 horas)
- [ ] Editor de esquema
- [ ] Validación en tiempo real
- [ ] Vista previa

### Fase 5: Épica 16 - Policy Editor (4-5 horas)
- [ ] Lista de políticas
- [ ] Editor de políticas
- [ ] Validación contra esquema
- [ ] Filtros

### Fase 6: Épica 17 - Playground (3-4 horas)
- [ ] Formulario PARC
- [ ] Entity builder
- [ ] Visualización de resultados

### Fase 7: Testing & Deployment (2-3 horas)
- [ ] Unit tests
- [ ] E2E tests
- [ ] Docker setup
- [ ] CI/CD

**Total estimado:** 20-26 horas

---

## 📝 CONCLUSIÓN

Esta estructura proporciona:
- ✅ Escalabilidad
- ✅ Mantenibilidad
- ✅ Testabilidad
- ✅ Separación de concerns
- ✅ Reutilización de código
- ✅ Fácil onboarding

---

**Listo para comenzar la implementación.** 🚀
