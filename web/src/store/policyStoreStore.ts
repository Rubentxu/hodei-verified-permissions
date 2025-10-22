/**
 * Policy Store - Zustand store for policy store state
 */

import { create } from 'zustand';
import { PolicyStoreFilters } from '../types';

interface PolicyStoreState {
  selectedStoreId: string | null;
  filters: PolicyStoreFilters;
  isLoading: boolean;
  error?: string;

  setSelectedStoreId: (id: string | null) => void;
  setFilters: (filters: PolicyStoreFilters) => void;
  setIsLoading: (loading: boolean) => void;
  setError: (error?: string) => void;
  reset: () => void;
}

export const usePolicyStoreStore = create<PolicyStoreState>((set) => ({
  selectedStoreId: null,
  filters: {},
  isLoading: false,
  error: undefined,

  setSelectedStoreId: (id: string | null) =>
    set({
      selectedStoreId: id,
    }),

  setFilters: (filters: PolicyStoreFilters) =>
    set({
      filters,
    }),

  setIsLoading: (loading: boolean) =>
    set({
      isLoading: loading,
    }),

  setError: (error?: string) =>
    set({
      error,
    }),

  reset: () =>
    set({
      selectedStoreId: null,
      filters: {},
      isLoading: false,
      error: undefined,
    }),
}));
