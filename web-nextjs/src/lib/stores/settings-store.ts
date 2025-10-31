import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface EditorPreferences {
  fontSize: number;
  tabSize: number;
  wordWrap: boolean;
  minimap: boolean;
  lineNumbers: boolean;
}

interface UserPreferences {
  theme: 'light' | 'dark';
  language: string;
  notifications: {
    email: boolean;
    browser: boolean;
    policyUpdates: boolean;
    securityAlerts: boolean;
  };
  editor: EditorPreferences;
}

interface SystemSettings {
  defaultPolicyStore?: string;
  autoSave: boolean;
  autoSaveInterval: number; // in seconds
  confirmationRequired: boolean;
}

interface FeatureFlags {
  experimentalFeatures: boolean;
  debugMode: boolean;
  advancedSearch: boolean;
  batchOperations: boolean;
  realTimeMetrics: boolean;
}

interface SettingsState {
  user: UserPreferences;
  system: SystemSettings;
  features: FeatureFlags;

  // Actions
  setTheme: (theme: 'light' | 'dark') => void;
  setLanguage: (language: string) => void;
  toggleNotification: (key: keyof UserPreferences['notifications']) => void;
  updateEditorPreferences: (prefs: Partial<EditorPreferences>) => void;
  setDefaultPolicyStore: (storeId: string) => void;
  toggleAutoSave: () => void;
  setAutoSaveInterval: (interval: number) => void;
  toggleConfirmationRequired: () => void;
  toggleFeatureFlag: (key: keyof FeatureFlags) => void;
  resetToDefaults: () => void;
}

const defaultUserPreferences: UserPreferences = {
  theme: 'light',
  language: 'en',
  notifications: {
    email: true,
    browser: true,
    policyUpdates: true,
    securityAlerts: true,
  },
  editor: {
    fontSize: 14,
    tabSize: 2,
    wordWrap: true,
    minimap: false,
    lineNumbers: true,
  },
};

const defaultSystemSettings: SystemSettings = {
  autoSave: true,
  autoSaveInterval: 30,
  confirmationRequired: true,
};

const defaultFeatureFlags: FeatureFlags = {
  experimentalFeatures: false,
  debugMode: false,
  advancedSearch: true,
  batchOperations: true,
  realTimeMetrics: true,
};

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      user: defaultUserPreferences,
      system: defaultSystemSettings,
      features: defaultFeatureFlags,

      setTheme: (theme) =>
        set((state) => ({
          user: { ...state.user, theme },
        })),

      setLanguage: (language) =>
        set((state) => ({
          user: { ...state.user, language },
        })),

      toggleNotification: (key) =>
        set((state) => ({
          user: {
            ...state.user,
            notifications: {
              ...state.user.notifications,
              [key]: !state.user.notifications[key],
            },
          },
        })),

      updateEditorPreferences: (prefs) =>
        set((state) => ({
          user: {
            ...state.user,
            editor: { ...state.user.editor, ...prefs },
          },
        })),

      setDefaultPolicyStore: (storeId) =>
        set((state) => ({
          system: { ...state.system, defaultPolicyStore: storeId },
        })),

      toggleAutoSave: () =>
        set((state) => ({
          system: { ...state.system, autoSave: !state.system.autoSave },
        })),

      setAutoSaveInterval: (interval) =>
        set((state) => ({
          system: { ...state.system, autoSaveInterval: interval },
        })),

      toggleConfirmationRequired: () =>
        set((state) => ({
          system: {
            ...state.system,
            confirmationRequired: !state.system.confirmationRequired,
          },
        })),

      toggleFeatureFlag: (key) =>
        set((state) => ({
          features: {
            ...state.features,
            [key]: !state.features[key],
          },
        })),

      resetToDefaults: () =>
        set({
          user: defaultUserPreferences,
          system: defaultSystemSettings,
          features: defaultFeatureFlags,
        }),
    }),
    {
      name: 'hodei-settings',
      version: 1,
    }
  )
);
