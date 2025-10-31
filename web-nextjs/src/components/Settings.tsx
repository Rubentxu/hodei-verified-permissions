'use client';

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Settings as SettingsIcon,
  User,
  Bell,
  Code,
  Shield,
  Sliders,
  Moon,
  Sun,
  Monitor,
  Save,
  RotateCcw,
  CheckCircle,
} from 'lucide-react';
import { useSettingsStore } from '@/lib/stores/settings-store';
import { usePolicyStores } from '@/hooks/usePolicyStores';

const Settings = () => {
  const [activeTab, setActiveTab] = useState<'user' | 'system' | 'features'>('user');
  const [saveSuccess, setSaveSuccess] = useState(false);

  const {
    user,
    system,
    features,
    setTheme,
    setLanguage,
    toggleNotification,
    updateEditorPreferences,
    setDefaultPolicyStore,
    toggleAutoSave,
    setAutoSaveInterval,
    toggleConfirmationRequired,
    toggleFeatureFlag,
    resetToDefaults,
  } = useSettingsStore();

  const { data: policyStoresData } = usePolicyStores();

  const handleSave = () => {
    // Settings are automatically persisted via Zustand
    setSaveSuccess(true);
    setTimeout(() => setSaveSuccess(false), 3000);
  };

  const handleReset = () => {
    if (window.confirm('Are you sure you want to reset all settings to defaults?')) {
      resetToDefaults();
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    }
  };

  const tabs = [
    { id: 'user', label: 'User Preferences', icon: User },
    { id: 'system', label: 'System Settings', icon: Settings },
    { id: 'features', label: 'Feature Flags', icon: Sliders },
  ] as const;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Settings</h2>
          <p className="text-gray-600">Customize your workspace and preferences</p>
        </div>
        <div className="flex items-center space-x-2">
          {saveSuccess && (
            <div className="flex items-center space-x-2 text-green-600">
              <CheckCircle className="w-4 h-4" />
              <span className="text-sm">Settings saved</span>
            </div>
          )}
          <Button variant="outline" onClick={handleReset}>
            <RotateCcw className="w-4 h-4 mr-2" />
            Reset to Defaults
          </Button>
          <Button onClick={handleSave}>
            <Save className="w-4 h-4 mr-2" />
            Save Settings
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex space-x-1 bg-gray-100 p-1 rounded-lg w-fit">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center space-x-2 px-4 py-2 rounded-md transition-colors ${
                activeTab === tab.id
                  ? 'bg-white text-blue-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              <Icon className="w-4 h-4" />
              <span>{tab.label}</span>
            </button>
          );
        })}
      </div>

      {/* User Preferences Tab */}
      {activeTab === 'user' && (
        <div className="space-y-6">
          {/* Theme Settings */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Monitor className="w-5 h-5" />
                <span>Theme & Language</span>
              </CardTitle>
              <CardDescription>Customize the appearance and language</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Theme
                </label>
                <div className="grid grid-cols-3 gap-3">
                  <button
                    onClick={() => setTheme('light')}
                    className={`p-3 border rounded-lg flex items-center justify-center space-x-2 ${
                      user.theme === 'light'
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                  >
                    <Sun className="w-5 h-5" />
                    <span>Light</span>
                  </button>
                  <button
                    onClick={() => setTheme('dark')}
                    className={`p-3 border rounded-lg flex items-center justify-center space-x-2 ${
                      user.theme === 'dark'
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                  >
                    <Moon className="w-5 h-5" />
                    <span>Dark</span>
                  </button>
                  <button
                    className={`p-3 border rounded-lg flex items-center justify-center space-x-2 ${
                      'border-gray-200 opacity-50 cursor-not-allowed'
                    }`}
                    disabled
                  >
                    <Monitor className="w-5 h-5" />
                    <span>System</span>
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Language
                </label>
                <select
                  value={user.language}
                  onChange={(e) => setLanguage(e.target.value)}
                  className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                >
                  <option value="en">English</option>
                  <option value="es">Español</option>
                  <option value="fr">Français</option>
                  <option value="de">Deutsch</option>
                </select>
              </div>
            </CardContent>
          </Card>

          {/* Notifications */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Bell className="w-5 h-5" />
                <span>Notifications</span>
              </CardTitle>
              <CardDescription>Choose which notifications you want to receive</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {Object.entries(user.notifications).map(([key, value]) => (
                  <div key={key} className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium text-gray-900">
                        {key === 'email' && 'Email Notifications'}
                        {key === 'browser' && 'Browser Notifications'}
                        {key === 'policyUpdates' && 'Policy Updates'}
                        {key === 'securityAlerts' && 'Security Alerts'}
                      </p>
                      <p className="text-xs text-gray-600">
                        {key === 'email' && 'Receive notifications via email'}
                        {key === 'browser' && 'Receive browser push notifications'}
                        {key === 'policyUpdates' && 'Get notified when policies change'}
                        {key === 'securityAlerts' && 'Important security alerts'}
                      </p>
                    </div>
                    <button
                      onClick={() => toggleNotification(key as any)}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        value ? 'bg-blue-600' : 'bg-gray-200'
                      }`}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          value ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Editor Preferences */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Code className="w-5 h-5" />
                <span>Editor Preferences</span>
              </CardTitle>
              <CardDescription>Customize the code editor behavior</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Font Size
                  </label>
                  <input
                    type="number"
                    min="10"
                    max="24"
                    value={user.editor.fontSize}
                    onChange={(e) =>
                      updateEditorPreferences({ fontSize: parseInt(e.target.value) })
                    }
                    className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Tab Size
                  </label>
                  <select
                    value={user.editor.tabSize}
                    onChange={(e) =>
                      updateEditorPreferences({ tabSize: parseInt(e.target.value) })
                    }
                    className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  >
                    <option value={2}>2 spaces</option>
                    <option value={4}>4 spaces</option>
                    <option value={8}>8 spaces</option>
                  </select>
                </div>
              </div>

              <div className="space-y-3">
                <label className="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    checked={user.editor.wordWrap}
                    onChange={(e) => updateEditorPreferences({ wordWrap: e.target.checked })}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="text-sm text-gray-700">Enable word wrap</span>
                </label>

                <label className="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    checked={user.editor.minimap}
                    onChange={(e) => updateEditorPreferences({ minimap: e.target.checked })}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="text-sm text-gray-700">Show minimap</span>
                </label>

                <label className="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    checked={user.editor.lineNumbers}
                    onChange={(e) =>
                      updateEditorPreferences({ lineNumbers: e.target.checked })
                    }
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="text-sm text-gray-700">Show line numbers</span>
                </label>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* System Settings Tab */}
      {activeTab === 'system' && (
        <div className="space-y-6">
          {/* General Settings */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <SettingsIcon className="w-5 h-5" />
                <span>General Settings</span>
              </CardTitle>
              <CardDescription>Configure system-wide preferences</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Default Policy Store
                </label>
                <select
                  value={system.defaultPolicyStore || ''}
                  onChange={(e) => setDefaultPolicyStore(e.target.value)}
                  className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                >
                  <option value="">Select a policy store</option>
                  {policyStoresData?.policy_stores.map((store) => (
                    <option key={store.policy_store_id} value={store.policy_store_id}>
                      {store.policy_store_id}
                    </option>
                  ))}
                </select>
                <p className="text-xs text-gray-600 mt-1">
                  This policy store will be selected by default
                </p>
              </div>

              <div className="flex items-center justify-between pt-4">
                <div>
                  <p className="text-sm font-medium text-gray-900">Auto Save</p>
                  <p className="text-xs text-gray-600">
                    Automatically save changes to policies and schemas
                  </p>
                </div>
                <button
                  onClick={toggleAutoSave}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    system.autoSave ? 'bg-blue-600' : 'bg-gray-200'
                  }`}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                      system.autoSave ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>

              {system.autoSave && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Auto Save Interval (seconds)
                  </label>
                  <input
                    type="number"
                    min="5"
                    max="300"
                    value={system.autoSaveInterval}
                    onChange={(e) => setAutoSaveInterval(parseInt(e.target.value))}
                    className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  />
                </div>
              )}

              <div className="flex items-center justify-between pt-4 border-t">
                <div>
                  <p className="text-sm font-medium text-gray-900">Require Confirmation</p>
                  <p className="text-xs text-gray-600">
                    Show confirmation dialogs for destructive actions
                  </p>
                </div>
                <button
                  onClick={toggleConfirmationRequired}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    system.confirmationRequired ? 'bg-blue-600' : 'bg-gray-200'
                  }`}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                      system.confirmationRequired ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Feature Flags Tab */}
      {activeTab === 'features' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Sliders className="w-5 h-5" />
                <span>Feature Flags</span>
              </CardTitle>
              <CardDescription>
                Enable or disable experimental and advanced features
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {Object.entries(features).map(([key, value]) => (
                  <div
                    key={key}
                    className="flex items-center justify-between p-4 border border-gray-200 rounded-lg"
                  >
                    <div className="flex-1">
                      <div className="flex items-center space-x-2">
                        <p className="text-sm font-medium text-gray-900">
                          {key === 'experimentalFeatures' && 'Experimental Features'}
                          {key === 'debugMode' && 'Debug Mode'}
                          {key === 'advancedSearch' && 'Advanced Search'}
                          {key === 'batchOperations' && 'Batch Operations'}
                          {key === 'realTimeMetrics' && 'Real-time Metrics'}
                        </p>
                        <Badge
                          variant={value ? 'default' : 'secondary'}
                          className="text-xs"
                        >
                          {value ? 'Enabled' : 'Disabled'}
                        </Badge>
                      </div>
                      <p className="text-xs text-gray-600 mt-1">
                        {key === 'experimentalFeatures' &&
                          'Enable features that are still in development'}
                        {key === 'debugMode' &&
                          'Show additional debug information and logging'}
                        {key === 'advancedSearch' &&
                          'Enable advanced search with filters and regex'}
                        {key === 'batchOperations' &&
                          'Perform operations on multiple items simultaneously'}
                        {key === 'realTimeMetrics' &&
                          'Display live metrics and performance data'}
                      </p>
                    </div>
                    <button
                      onClick={() => toggleFeatureFlag(key as any)}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        value ? 'bg-blue-600' : 'bg-gray-200'
                      }`}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          value ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>
                ))}
              </div>

              <div className="mt-6 p-4 bg-yellow-50 border border-yellow-200 rounded-md">
                <div className="flex items-start space-x-2">
                  <Shield className="w-5 h-5 text-yellow-600 mt-0.5" />
                  <div>
                    <p className="text-sm font-medium text-yellow-800">
                      Use with caution
                    </p>
                    <p className="text-xs text-yellow-700 mt-1">
                      Some features may be unstable or affect performance. Only enable features
                      you understand.
                    </p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
};

export default Settings;
