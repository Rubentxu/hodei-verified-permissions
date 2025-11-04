'use client';

import { create } from 'zustand';

interface PolicyStorePanelState {
  isOpen: boolean;
  content: 'details' | 'create' | 'edit' | null;
  selectedStoreId: string | null;
  isTransitioning: boolean;
  openPanel: (content: 'details' | 'create' | 'edit', storeId?: string) => void;
  closePanel: () => void;
  startTransition: () => void;
  endTransition: () => void;
}

export const usePolicyStorePanelStore = create<PolicyStorePanelState>((set) => ({
  isOpen: false,
  content: null,
  selectedStoreId: null,
  isTransitioning: false,
  openPanel: (content, storeId) => set({ isOpen: true, content, selectedStoreId: storeId || null }),
  closePanel: () => set({ isOpen: false }),
  startTransition: () => set({ isTransitioning: true }),
  endTransition: () => set({ isTransitioning: false }),
}));
