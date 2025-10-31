"use client";

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import {
  TestTube,
  Play,
  Download,
  CheckCircle,
  XCircle,
  AlertCircle,
  Clock,
  TrendingUp,
} from 'lucide-react';
import {
  useBatchAuthorization,
  useRunPredefinedScenarios,
  useExportBatchResults,
  useBatchResultsStats,
  BatchTestResult,
} from '@/hooks/useBatchAuthorization';

interface BatchTestProps {
  policyStoreId?: string;
}

const BatchTest: React.FC<BatchTestProps> = ({ policyStoreId = 'ps_123' }) => {
  const [results, setResults] = useState<BatchTestResult[]>([]);
  const [customScenarios, setCustomScenarios] = useState(0);
  
  const batchMutation = useBatchAuthorization();
  const predefinedMutation = useRunPredefinedScenarios();
  const { exportToCsv } = useExportBatchResults();
  const stats = useBatchResultsStats(results);

  const runBatchTest = (scenarios: any[]) => {
    batchMutation.mutate(
      { scenarios },
      {
        onSuccess: (data) => {
          setResults(data.results);
        },
      }
    );
  };

  const handleRunPredefined = (type: 'userAccess' | 'roleBased') => {
    if (type === 'userAccess') {
      predefinedMutation.runUserAccessTest();
    } else {
      predefinedMutation.runRoleBasedTest();
    }
    
    // Transfer results from predefined to batch
    setTimeout(() => {
      if (batchMutation.data) {
        setResults(batchMutation.data.results);
      }
    }, 100);
  };

  const handleExport = () => {
    if (results.length > 0) {
      exportToCsv(results, `batch-test-${Date.now()}.csv`);
    }
  };

  const isRunning = batchMutation.isPending || predefinedMutation.isPending;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <TestTube className="w-5 h-5" />
            <span>Batch Authorization Testing</span>
          </CardTitle>
          <CardDescription>
            Test multiple authorization scenarios simultaneously
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {/* Predefined Test Suites */}
            <div>
              <h4 className="text-sm font-medium mb-3">Predefined Test Suites</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                <Button
                  variant="outline"
                  onClick={() => handleRunPredefined('userAccess')}
                  disabled={isRunning}
                  className="justify-start"
                >
                  <Play className="w-4 h-4 mr-2" />
                  User Access Tests (3 scenarios)
                </Button>
                <Button
                  variant="outline"
                  onClick={() => handleRunPredefined('roleBased')}
                  disabled={isRunning}
                  className="justify-start"
                >
                  <Play className="w-4 h-4 mr-2" />
                  Role-Based Tests (3 scenarios)
                </Button>
              </div>
            </div>

            {/* Custom Test Count */}
            <div>
              <h4 className="text-sm font-medium mb-3">Custom Test Suite</h4>
              <div className="flex items-center space-x-3">
                <label className="text-sm">Number of scenarios:</label>
                <input
                  type="number"
                  min="1"
                  max="100"
                  value={customScenarios}
                  onChange={(e) => setCustomScenarios(parseInt(e.target.value) || 0)}
                  className="w-24 px-3 py-2 border border-gray-300 rounded-md text-sm"
                  disabled={isRunning}
                />
                <Button
                  onClick={() => {
                    const scenarios = Array.from({ length: customScenarios }, (_, i) => ({
                      name: `Custom Scenario ${i + 1}`,
                      policy_store_id: policyStoreId,
                      principal: { entity_type: 'User', entity_id: `user${i + 1}` },
                      action: { entity_type: 'Action', entity_id: 'access' },
                      resource: { entity_type: 'Resource', entity_id: `resource${i + 1}` },
                    }));
                    runBatchTest(scenarios);
                  }}
                  disabled={isRunning || customScenarios === 0}
                >
                  <Play className="w-4 h-4 mr-2" />
                  Run Custom Test
                </Button>
              </div>
            </div>

            {/* Export Button */}
            {results.length > 0 && (
              <div className="flex justify-end">
                <Button onClick={handleExport} variant="outline">
                  <Download className="w-4 h-4 mr-2" />
                  Export Results (CSV)
                </Button>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Running State */}
      {isRunning && (
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center space-x-3">
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-700" />
              <span className="text-sm">Running batch test...</span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Error State */}
      {(batchMutation.error || predefinedMutation.error) && (
        <Card className="border-red-200 bg-red-50">
          <CardContent className="pt-6">
            <div className="flex items-center space-x-2 text-red-700">
              <XCircle className="w-5 h-5" />
              <span className="font-medium">Test Failed</span>
            </div>
            <p className="text-sm text-red-600 mt-2">
              {batchMutation.error?.message || predefinedMutation.error?.message}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Results */}
      {results.length > 0 && (
        <>
          {/* Summary Stats */}
          <Card>
            <CardHeader>
              <CardTitle>Test Summary</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="bg-blue-50 p-3 rounded-lg">
                  <div className="text-2xl font-bold text-blue-700">{stats.total}</div>
                  <div className="text-xs text-blue-600">Total Tests</div>
                </div>
                <div className="bg-green-50 p-3 rounded-lg">
                  <div className="text-2xl font-bold text-green-700">{stats.successful}</div>
                  <div className="text-xs text-green-600">Successful</div>
                </div>
                <div className="bg-red-50 p-3 rounded-lg">
                  <div className="text-2xl font-bold text-red-700">{stats.failed}</div>
                  <div className="text-xs text-red-600">Failed</div>
                </div>
                <div className="bg-purple-50 p-3 rounded-lg">
                  <div className="text-2xl font-bold text-purple-700">
                    {stats.latencies.avg}ms
                  </div>
                  <div className="text-xs text-purple-600">Avg Latency</div>
                </div>
              </div>

              {/* Decision Breakdown */}
              <div className="mt-4 grid grid-cols-3 gap-4">
                <div className="flex items-center space-x-2">
                  <CheckCircle className="w-5 h-5 text-green-500" />
                  <span className="text-sm">
                    ALLOW: <strong>{stats.decisions.ALLOW}</strong> ({stats.rates.allow}%)
                  </span>
                </div>
                <div className="flex items-center space-x-2">
                  <XCircle className="w-5 h-5 text-red-500" />
                  <span className="text-sm">
                    DENY: <strong>{stats.decisions.DENY}</strong> ({stats.rates.deny}%)
                  </span>
                  </div>
                <div className="flex items-center space-x-2">
                  <AlertCircle className="w-5 h-5 text-yellow-500" />
                  <span className="text-sm">
                    UNSPECIFIED: <strong>{stats.decisions.UNSPECIFIED}</strong>
                  </span>
                </div>
              </div>

              {/* Latency Stats */}
              <div className="mt-4 flex items-center space-x-4 text-sm text-gray-600">
                <div className="flex items-center space-x-1">
                  <Clock className="w-4 h-4" />
                  <span>Min: {stats.latencies.min}ms</span>
                </div>
                <div className="flex items-center space-x-1">
                  <TrendingUp className="w-4 h-4" />
                  <span>Max: {stats.latencies.max}ms</span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Detailed Results Table */}
          <Card>
            <CardHeader>
              <CardTitle>Test Results</CardTitle>
              <CardDescription>
                Detailed results for each test scenario
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      <th className="text-left py-2 px-3">Scenario</th>
                      <th className="text-left py-2 px-3">Decision</th>
                      <th className="text-left py-2 px-3">Latency</th>
                      <th className="text-left py-2 px-3">Status</th>
                      <th className="text-left py-2 px-3">Policies</th>
                    </tr>
                  </thead>
                  <tbody>
                    {results.map((result, idx) => (
                      <tr key={idx} className="border-b hover:bg-gray-50">
                        <td className="py-2 px-3 font-medium">{result.scenario_name}</td>
                        <td className="py-2 px-3">
                          <Badge
                            variant={
                              result.decision === 'ALLOW'
                                ? 'default'
                                : result.decision === 'DENY'
                                ? 'destructive'
                                : 'secondary'
                            }
                          >
                            {result.decision}
                          </Badge>
                        </td>
                        <td className="py-2 px-3 font-mono">{result.latency_ms}ms</td>
                        <td className="py-2 px-3">
                          {result.success ? (
                            <CheckCircle className="w-4 h-4 text-green-500" />
                          ) : (
                            <XCircle className="w-4 h-4 text-red-500" />
                          )}
                        </td>
                        <td className="py-2 px-3">
                          {result.determining_policies && result.determining_policies.length > 0 ? (
                            <span className="text-xs">
                              {result.determining_policies.length} policy(ies)
                            </span>
                          ) : (
                            <span className="text-xs text-gray-400">None</span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
};

export default BatchTest;
