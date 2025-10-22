/**
 * Domain Types - Business logic types
 */

export interface PolicyStoreFilters {
  search?: string;
  sortBy?: 'name' | 'created' | 'updated';
  sortOrder?: 'asc' | 'desc';
}

export interface PolicyFilters {
  search?: string;
  effect?: 'permit' | 'forbid';
  sortBy?: 'name' | 'created' | 'updated';
  sortOrder?: 'asc' | 'desc';
}

export interface PlaygroundState {
  principal?: string;
  action?: string;
  resource?: string;
  context: Record<string, unknown>;
  policies: string[];
  entities: string;
  schema?: string;
}

export interface EditorState {
  content: string;
  isDirty: boolean;
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

export interface NotificationMessage {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
  duration?: number;
}

export interface PaginationState {
  page: number;
  pageSize: number;
  total: number;
  hasMore: boolean;
}
