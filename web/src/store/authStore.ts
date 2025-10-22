/**
 * Auth Store - Zustand store for authentication state
 */

import { create } from 'zustand';

interface User {
  id: string;
  email: string;
  name: string;
}

interface AuthStore {
  isAuthenticated: boolean;
  token?: string;
  user?: User;
  setAuth: (token: string, user: User) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthStore>((set) => ({
  isAuthenticated: false,
  token: undefined,
  user: undefined,

  setAuth: (token: string, user: User) =>
    set({
      isAuthenticated: true,
      token,
      user,
    }),

  clearAuth: () =>
    set({
      isAuthenticated: false,
      token: undefined,
      user: undefined,
    }),
}));
