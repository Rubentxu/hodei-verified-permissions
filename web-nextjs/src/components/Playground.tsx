"use client";

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Badge } from './ui/badge';
import { Input } from './ui/input';
import { Label } from './ui/label';
import {
  TestTube,
  Play,
  Save,
  FolderOpen,
  CheckCircle,
  XCircle,
  AlertCircle,
  Bug,
  GitBranch,
  Download,
  Plus,
  Trash2,
} from 'lucide-react';
import DebugPanel, { DebugStep } from './DebugPanel';
import BatchTest from './BatchTest';
import {
  useSavedScenarios,
  useSaveScenario,
  useDeleteScenario,
  SavedScenario,
} from '@/hooks/useSavedScenarios';

const Playground = () => {
  // Core test state
  const [testResult, setTestResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  // Scenario management
  const [currentScenario, setCurrentScenario] = useState({
    name: 'New Test Scenario',
    description: '',
    policy_store_id: 'ps_123',
    principal: { entity_type: 'User', entity_id: 'alice' },
    action: { entity_type: 'Action', entity_id: 'viewDocument' },
    resource: { entity_type: 'Document', entity_id: 'doc123' },
    context: '{}',
  });

  // Debug mode state
  const [debugMode, setDebugMode] = useState(false);
  const [debugSteps, setDebugSteps] = useState<DebugStep[]>([]);

  // Scenarios data
  const { data: savedScenarios } = useSavedScenarios();
  const saveScenarioMutation = useSaveScenario();
  const deleteScenarioMutation = useDeleteScenario();

  // Batch test tab
  const [activeTab, setActiveTab] = useState<'single' | 'batch'>('single');

  const runAuthorizationTest = async () => {
    setLoading(true);
    try {
      // Run debug steps if debug mode is enabled
      if (debugMode) {
        await runDebugMode();
      }

      const response = await fetch('/api/authorize', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(currentScenario),
      });

      const result = await response.json();
      setTestResult({
        status: response.status,
        data: result,
      });
    } catch (error) {
      setTestResult({
        status: 500,
        error: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setLoading(false);
    }
  };

  const runDebugMode = async () => {
    const steps: DebugStep[] = [
      {
        step: 1,
        description: 'Parse authorization request',
        status: 'pending',
      },
      {
        step: 2,
        description: 'Load policy store configuration',
        status: 'pending',
      },
      {
        step: 3,
        description: 'Fetch relevant policies',
        status: 'pending',
      },
      {
        step: 4,
        description: 'Evaluate policies against entities',
        status: 'pending',
      },
      {
        step: 5,
        description: 'Determine final authorization decision',
        status: 'pending',
      },
    ];

    setDebugSteps(steps);

    // Simulate debug steps
    for (let i = 0; i < steps.length; i++) {
      await new Promise(resolve => setTimeout(resolve, 500));
      setDebugSteps(prev => 
        prev.map((step, idx) => 
          idx === i 
            ? { ...step, status: 'completed', timestamp: new Date().toISOString() }
            : step
        )
      );
    }
  };

  const handleSaveScenario = async () => {
    try {
      await saveScenarioMutation.mutateAsync(currentScenario);
      alert('Scenario saved successfully!');
    } catch (error) {
      alert(`Failed to save scenario: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  };

  const handleLoadScenario = (scenario: SavedScenario) => {
    setCurrentScenario({
      name: scenario.name,
      description: scenario.description,
      policy_store_id: scenario.policy_store_id,
      principal: scenario.principal,
      action: scenario.action,
      resource: scenario.resource,
      context: JSON.stringify(scenario.context),
    });
  };

  const handleDeleteScenario = async (scenarioId: string) => {
    if (confirm('Are you sure you want to delete this scenario?')) {
      await deleteScenarioMutation.mutateAsync(scenarioId);
    }
  };

  const getDecisionIcon = (decision: string) => {
    switch (decision) {
      case 'ALLOW':
        return <CheckCircle className="w-5 h-5 text-green-500" />;
      case 'DENY':
        return <XCircle className="w-5 h-5 text-red-500" />;
      default:
        return <AlertCircle className="w-5 h-5 text-yellow-500" />;
    }
  };

  const getDecisionBadge = (decision: string) => {
    switch (decision) {
      case 'ALLOW':
        return <Badge className="bg-green-100 text-green-800">ALLOW</Badge>;
      case 'DENY':
        return <Badge className="bg-red-100 text-red-800">DENY</Badge>;
      default:
        return <Badge className="bg-yellow-100 text-yellow-800">UNSPECIFIED</Badge>;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Authorization Playground</h2>
          <p className="text-gray-600">Test, debug, and analyze authorization requests</p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant={activeTab === 'single' ? 'default' : 'outline'}
            onClick={() => setActiveTab('single')}
          >
            Single Test
          </Button>
          <Button
            variant={activeTab === 'batch' ? 'default' : 'outline'}
            onClick={() => setActiveTab('batch')}
          >
            Batch Test
          </Button>
        </div>
      </div>

      {/* Single Test Tab */}
      {activeTab === 'single' && (
        <>
          {/* Test Configuration */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center space-x-2">
                  <TestTube className="w-5 h-5" />
                  <span>Test Configuration</span>
                </CardTitle>
                <div className="flex items-center space-x-2">
                  <Button
                    variant={debugMode ? 'default' : 'outline'}
                    onClick={() => setDebugMode(!debugMode)}
                  >
                    <Bug className="w-4 h-4 mr-2" />
                    {debugMode ? 'Disable' : 'Enable'} Debug
                  </Button>
                  <Button variant="outline" onClick={handleSaveScenario}>
                    <Save className="w-4 h-4 mr-2" />
                    Save Scenario
                  </Button>
                </div>
              </div>
              <CardDescription>
                Configure and test authorization scenarios
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Left Column - Scenario Config */}
                <div className="space-y-4">
                  <div>
                    <Label htmlFor="scenario-name">Scenario Name</Label>
                    <Input
                      id="scenario-name"
                      value={currentScenario.name}
                      onChange={(e) =>
                        setCurrentScenario({ ...currentScenario, name: e.target.value })
                      }
                    />
                  </div>

                  <div>
                    <Label htmlFor="scenario-description">Description</Label>
                    <textarea
                      id="scenario-description"
                      name="description"
                      value={currentScenario.description}
                      onChange={(e) =>
                        setCurrentScenario({ ...currentScenario, description: e.target.value })
                      }
                      className="w-full p-2 border border-gray-300 rounded-md text-sm"
                      rows={2}
                    />
                  </div>

                  <div>
                    <Label htmlFor="policy-store">Policy Store ID</Label>
                    <Input
                      id="policy-store"
                      value={currentScenario.policy_store_id}
                      onChange={(e) =>
                        setCurrentScenario({
                          ...currentScenario,
                          policy_store_id: e.target.value,
                        })
                      }
                    />
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <Label>Principal Type</Label>
                      <Input
                        value={currentScenario.principal.entity_type}
                        onChange={(e) =>
                          setCurrentScenario({
                            ...currentScenario,
                            principal: { ...currentScenario.principal, entity_type: e.target.value },
                          })
                        }
                      />
                    </div>
                    <div>
                      <Label>Principal ID</Label>
                      <Input
                        value={currentScenario.principal.entity_id}
                        onChange={(e) =>
                          setCurrentScenario({
                            ...currentScenario,
                            principal: { ...currentScenario.principal, entity_id: e.target.value },
                          })
                        }
                      />
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <Label>Action Type</Label>
                      <Input
                        value={currentScenario.action.entity_type}
                        onChange={(e) =>
                          setCurrentScenario({
                            ...currentScenario,
                            action: { ...currentScenario.action, entity_type: e.target.value },
                          })
                        }
                      />
                    </div>
                    <div>
                      <Label>Action ID</Label>
                      <Input
                        value={currentScenario.action.entity_id}
                        onChange={(e) =>
                          setCurrentScenario({
                            ...currentScenario,
                            action: { ...currentScenario.action, entity_id: e.target.value },
                          })
                        }
                      />
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <Label>Resource Type</Label>
                      <Input
                        value={currentScenario.resource.entity_type}
                        onChange={(e) =>
                          setCurrentScenario({
                            ...currentScenario,
                            resource: { ...currentScenario.resource, entity_type: e.target.value },
                          })
                        }
                      />
                    </div>
                    <div>
                      <Label>Resource ID</Label>
                      <Input
                        value={currentScenario.resource.entity_id}
                        onChange={(e) =>
                          setCurrentScenario({
                            ...currentScenario,
                            resource: { ...currentScenario.resource, entity_id: e.target.value },
                          })
                        }
                      />
                    </div>
                  </div>

                  <div>
                    <Label htmlFor="context">Context (JSON)</Label>
                    <textarea
                      id="context"
                      value={currentScenario.context}
                      onChange={(e) =>
                        setCurrentScenario({ ...currentScenario, context: e.target.value })
                      }
                      className="w-full p-2 border border-gray-300 rounded-md font-mono text-sm"
                      rows={3}
                    />
                  </div>

                  <Button 
                    onClick={runAuthorizationTest} 
                    disabled={loading}
                    className="w-full"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    {loading ? 'Running...' : 'Run Test'}
                  </Button>
                </div>

                {/* Right Column - Saved Scenarios */}
                <div>
                  <div className="flex items-center justify-between mb-3">
                    <Label>Saved Scenarios</Label>
                  </div>
                  <div className="space-y-2 max-h-96 overflow-y-auto">
                    {savedScenarios && savedScenarios.length > 0 ? (
                      savedScenarios.map((scenario) => (
                        <div
                          key={scenario.id}
                          data-testid="scenario-card"
                          className="border rounded p-3 hover:bg-gray-50"
                        >
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <h4 className="font-medium text-sm">{scenario.name}</h4>
                              <p className="text-xs text-gray-600 mt-1">
                                {scenario.description || 'No description'}
                              </p>
                              <div className="flex items-center space-x-2 mt-2 text-xs text-gray-500">
                                <span>{scenario.principal.entity_type}:{scenario.principal.entity_id}</span>
                                <span>•</span>
                                <span>{scenario.action.entity_id}</span>
                                <span>•</span>
                                <span>{scenario.resource.entity_type}:{scenario.resource.entity_id}</span>
                              </div>
                              <div className="flex items-center space-x-2 mt-1 text-xs text-gray-400">
                                <span>
                                  Updated {new Date(scenario.updated_at).toLocaleString()} ({Math.floor((Date.now() - new Date(scenario.updated_at).getTime()) / (1000 * 60))} minutes ago)
                                </span>
                              </div>
                            </div>
                            <div className="flex items-center space-x-1 ml-2">
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => handleLoadScenario(scenario)}
                                aria-label="Load"
                                title="Load scenario"
                              >
                                <FolderOpen className="w-4 h-4" />
                              </Button>
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => handleDeleteScenario(scenario.id)}
                                aria-label="Delete"
                                title="Delete scenario"
                              >
                                <Trash2 className="w-4 h-4" />
                              </Button>
                            </div>
                          </div>
                        </div>
                      ))
                    ) : (
                      <p className="text-sm text-gray-500 text-center py-4">
                        No saved scenarios
                      </p>
                    )}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Test Results */}
          <Card>
            <CardHeader>
              <CardTitle>Test Results</CardTitle>
              <CardDescription>
                Authorization decision and details
              </CardDescription>
            </CardHeader>
            <CardContent>
              {testResult ? (
                <div className="space-y-4">
                  {testResult.status === 200 ? (
                    <>
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-2">
                          {getDecisionIcon(testResult.data.decision)}
                          <span className="font-medium">Decision:</span>
                        </div>
                        {getDecisionBadge(testResult.data.decision)}
                      </div>

                      {testResult.data.determining_policies?.length > 0 && (
                        <div>
                          <h4 className="font-medium mb-2">Determining Policies:</h4>
                          <ul className="space-y-1">
                            {testResult.data.determining_policies.map((policy: string, index: number) => (
                              <li key={index} className="text-sm text-gray-600 bg-gray-50 p-2 rounded font-mono">
                                {policy}
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}

                      {testResult.data.errors?.length > 0 && (
                        <div>
                          <h4 className="font-medium mb-2 text-red-600">Errors:</h4>
                          <ul className="space-y-1">
                            {testResult.data.errors.map((error: string, index: number) => (
                              <li key={index} className="text-sm text-red-600 bg-red-50 p-2 rounded">
                                {error}
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}
                    </>
                  ) : (
                    <div className="text-center py-4">
                      <XCircle className="w-8 h-8 text-red-500 mx-auto mb-2" />
                      <p className="text-red-600 font-medium">Test Failed</p>
                      <p className="text-sm text-gray-600">
                        {testResult.error || 'Server error occurred'}
                      </p>
                    </div>
                  )}
                </div>
              ) : (
                <div className="text-center py-8">
                  <TestTube className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                  <h3 className="text-lg font-medium text-gray-900 mb-2">
                    No Test Results
                  </h3>
                  <p className="text-gray-600">
                    Configure your test case and click "Run Test" to see results
                  </p>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Debug Panel */}
          {debugMode && (
            <DebugPanel
              debugSteps={debugSteps}
              isRunning={loading}
              onStart={runDebugMode}
            />
          )}
        </>
      )}

      {/* Batch Test Tab */}
      {activeTab === 'batch' && (
        <BatchTest policyStoreId={currentScenario.policy_store_id} />
      )}
    </div>
  );
};

export default Playground;
