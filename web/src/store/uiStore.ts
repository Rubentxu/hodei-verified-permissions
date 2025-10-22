/**
 * UI Store - Zustand store for UI state
 */

import { create } from 'zustand';

interface UIStore {
  sidebarOpen: boolean;
  theme: 'light' | 'dark';
  notifications: Array<{
    id: string;
    type: 'success' | 'error' | 'warning' | 'info';
    message: string;
  }>;

  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  setTheme: (theme: 'light' | 'dark') => void;
  addNotification: (
    message: string,
    type: 'success' | 'error' | 'warning' | 'info'
  ) => void;
  removeNotification: (id: string) => void;
  clearNotifications: () => void;
}

export const useUIStore = create<UIStore>((set) => ({
  sidebarOpen: true,
  theme: 'light',
  notifications: [],

  toggleSidebar: () =>
    set((state) => ({
      sidebarOpen: !state.sidebarOpen,
    })),

  setSidebarOpen: (open: boolean) =>
    set({
      sidebarOpen: open,
    }),

  setTheme: (theme: 'light' | 'dark') =>
    set({
      theme,
    }),

  addNotification: (message: string, type: 'success' | 'error' | 'warning' | 'info') =>
    set((state) => ({
      notifications: [
        ...state.notifications,
        {
          id: `${Date.now()}-${Math.random()}`,
          type,
          message,
        },
      ],
    })),

  removeNotification: (id: string) =>
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    })),

  clearNotifications: () =>
    set({
      notifications: [],
    }),
}));
