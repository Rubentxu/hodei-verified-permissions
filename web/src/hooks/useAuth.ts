/**
 * useAuth - Authentication hook
 */

import { useState, useCallback, useEffect } from 'react';

interface AuthState {
  isAuthenticated: boolean;
  token?: string;
  user?: {
    id: string;
    email: string;
    name: string;
  };
}

export const useAuth = () => {
  const [authState, setAuthState] = useState<AuthState>({
    isAuthenticated: false,
  });

  // Check if token exists in localStorage on mount
  useEffect(() => {
    const token = localStorage.getItem('auth_token');
    if (token) {
      setAuthState({
        isAuthenticated: true,
        token,
        user: JSON.parse(localStorage.getItem('auth_user') || '{}'),
      });
    }
  }, []);

  const login = useCallback((token: string, user: AuthState['user']) => {
    localStorage.setItem('auth_token', token);
    localStorage.setItem('auth_user', JSON.stringify(user));
    setAuthState({
      isAuthenticated: true,
      token,
      user,
    });
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('auth_user');
    setAuthState({
      isAuthenticated: false,
    });
  }, []);

  return {
    ...authState,
    login,
    logout,
  };
};
