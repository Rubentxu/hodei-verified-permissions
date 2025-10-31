"use client";

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import {
  Play,
  Pause,
  RotateCcw,
  ChevronDown,
  ChevronRight,
  Bug,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  Info,
} from 'lucide-react';

export interface DebugStep {
  step: number;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
  details?: any;
  timestamp?: string;
  duration_ms?: number;
  errors?: string[];
}

interface DebugPanelProps {
  debugSteps: DebugStep[];
  isRunning?: boolean;
  onStart?: () => void;
  onPause?: () => void;
  onReset?: () => void;
  onStepClick?: (step: DebugStep) => void;
}

const statusConfig = {
  pending: {
    icon: Clock,
    color: 'text-gray-400',
    badge: 'secondary',
    label: 'Pending',
    bgColor: 'bg-gray-50',
    borderColor: 'border-gray-200',
  },
  running: {
    icon: Play,
    color: 'text-blue-500',
    badge: 'default',
    label: 'Running',
    bgColor: 'bg-blue-50',
    borderColor: 'border-blue-200',
  },
  completed: {
    icon: CheckCircle,
    color: 'text-green-500',
    badge: 'default',
    label: 'Completed',
    bgColor: 'bg-green-50',
    borderColor: 'border-green-200',
  },
  failed: {
    icon: XCircle,
    color: 'text-red-500',
    badge: 'destructive',
    label: 'Failed',
    bgColor: 'bg-red-50',
    borderColor: 'border-red-200',
  },
  skipped: {
    icon: AlertCircle,
    color: 'text-yellow-500',
    badge: 'secondary',
    label: 'Skipped',
    bgColor: 'bg-yellow-50',
    borderColor: 'border-yellow-200',
  },
};

const DebugPanel: React.FC<DebugPanelProps> = ({
  debugSteps,
  isRunning = false,
  onStart,
  onPause,
  onReset,
  onStepClick,
}) => {
  const [expandedSteps, setExpandedSteps] = useState<Set<number>>(new Set());

  const toggleStepExpansion = (stepNumber: number) => {
    const newExpanded = new Set(expandedSteps);
    if (newExpanded.has(stepNumber)) {
      newExpanded.delete(stepNumber);
    } else {
      newExpanded.add(stepNumber);
    }
    setExpandedSteps(newExpanded);
  };

  const formatDuration = (ms?: number) => {
    if (!ms) return 'N/A';
    return `${ms}ms`;
  };

  const formatTimestamp = (timestamp?: string) => {
    if (!timestamp) return 'N/A';
    return new Date(timestamp).toLocaleTimeString();
  };

  const completedSteps = debugSteps.filter(s => s.status === 'completed').length;
  const failedSteps = debugSteps.filter(s => s.status === 'failed').length;
  const totalSteps = debugSteps.length;
  const successRate = totalSteps > 0 ? Math.round((completedSteps / totalSteps) * 100) : 0;

  return (
    <div className="space-y-4">
      {/* Header Controls */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Bug className="w-5 h-5" />
              <CardTitle>Debug Mode</CardTitle>
              <Badge variant="outline">
                {successRate}% Success Rate
              </Badge>
            </div>
            <div className="flex items-center space-x-2">
              {!isRunning ? (
                <Button onClick={onStart} size="sm" disabled={totalSteps === 0}>
                  <Play className="w-4 h-4 mr-2" />
                  Start Debug
                </Button>
              ) : (
                <Button onClick={onPause} size="sm" variant="outline">
                  <Pause className="w-4 h-4 mr-2" />
                  Pause
                </Button>
              )}
              <Button onClick={onReset} size="sm" variant="outline">
                <RotateCcw className="w-4 h-4 mr-2" />
                Reset
              </Button>
            </div>
          </div>
          <CardDescription>
            Step-by-step authorization analysis with detailed debugging information
          </CardDescription>
        </CardHeader>
      </Card>

      {/* Debug Steps */}
      <Card>
        <CardHeader>
          <CardTitle>Execution Steps</CardTitle>
          <CardDescription>
            {completedSteps} of {totalSteps} steps completed
          </CardDescription>
        </CardHeader>
        <CardContent>
          {debugSteps.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Info className="w-12 h-12 mx-auto mb-4 opacity-50" />
              <p>No debug steps available. Run a test to see debug information.</p>
            </div>
          ) : (
            <div className="space-y-3">
              {debugSteps.map((step) => {
                const config = statusConfig[step.status];
                const Icon = config.icon;
                const isExpanded = expandedSteps.has(step.step);

                return (
                  <div
                    key={step.step}
                    className={`border rounded-lg transition-all ${config.bgColor} ${config.borderColor}`}
                  >
                    {/* Step Header */}
                    <div
                      className="p-4 cursor-pointer hover:bg-opacity-75"
                      onClick={() => {
                        toggleStepExpansion(step.step);
                        onStepClick?.(step);
                      }}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-3">
                          <div className="flex items-center space-x-2">
                            {isExpanded ? (
                              <ChevronDown className="w-4 h-4" />
                            ) : (
                              <ChevronRight className="w-4 h-4" />
                            )}
                            <Badge variant={config.badge as any}>
                              Step {step.step}
                            </Badge>
                          </div>
                          <span className="font-medium">{step.description}</span>
                        </div>

                        <div className="flex items-center space-x-4">
                          {step.duration_ms && (
                            <span className="text-xs text-gray-500 font-mono">
                              {formatDuration(step.duration_ms)}
                            </span>
                          )}
                          <div className="flex items-center space-x-1">
                            <Icon className={`w-4 h-4 ${config.color}`} />
                            <span className={`text-xs font-medium ${config.color}`}>
                              {config.label}
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* Step Metadata */}
                      <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
                        <span>Started: {formatTimestamp(step.timestamp)}</span>
                        {step.status === 'completed' && (
                          <span className="text-green-600 font-medium">
                            ✓ Completed successfully
                          </span>
                        )}
                        {step.status === 'failed' && (
                          <span className="text-red-600 font-medium">
                            ✗ Failed
                          </span>
                        )}
                        {step.status === 'skipped' && (
                          <span className="text-yellow-600 font-medium">
                            ⚠ Skipped
                          </span>
                        )}
                      </div>
                    </div>

                    {/* Step Details (Expandable) */}
                    {isExpanded && (
                      <div className="border-t border-gray-200 p-4 bg-white">
                        {step.details ? (
                          <>
                            <h4 className="text-sm font-medium mb-2">Details:</h4>
                            <div className="bg-gray-900 text-green-400 p-3 rounded text-xs font-mono overflow-x-auto">
                              <pre>{JSON.stringify(step.details, null, 2)}</pre>
                            </div>
                          </>
                        ) : (
                          <p className="text-sm text-gray-500 italic">No details available</p>
                        )}

                        {/* Policies Evaluated */}
                        {step.details?.policies_evaluated && (
                          <div className="mt-4">
                            <h5 className="text-sm font-medium mb-2">Policies Evaluated:</h5>
                            <ul className="space-y-1">
                              {step.details.policies_evaluated.map((policy: string, idx: number) => (
                                <li key={idx} className="text-xs bg-gray-100 p-2 rounded font-mono">
                                  {policy}
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}

                        {/* Entities */}
                        {step.details?.entities && (
                          <div className="mt-4">
                            <h5 className="text-sm font-medium mb-2">Entities:</h5>
                            <div className="text-xs bg-gray-100 p-2 rounded font-mono">
                              <pre>{JSON.stringify(step.details.entities, null, 2)}</pre>
                            </div>
                          </div>
                        )}

                        {/* Errors */}
                        {step.errors && step.errors.length > 0 && (
                          <div className="mt-4">
                            <h5 className="text-sm font-medium mb-2 text-red-600">Errors:</h5>
                            <ul className="space-y-1">
                              {step.errors.map((error: string, idx: number) => (
                                <li key={idx} className="text-xs bg-red-100 text-red-700 p-2 rounded">
                                  {error}
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}

          {/* Summary */}
          {totalSteps > 0 && (
            <div className="mt-6 p-4 bg-gray-50 rounded-lg">
              <h4 className="text-sm font-medium mb-3">Debug Summary:</h4>
              <div className="grid grid-cols-4 gap-4 text-sm">
                <div>
                  <span className="text-gray-600">Total Steps:</span>
                  <div className="font-bold text-lg">{totalSteps}</div>
                </div>
                <div>
                  <span className="text-gray-600">Completed:</span>
                  <div className="font-bold text-lg text-green-600">{completedSteps}</div>
                </div>
                <div>
                  <span className="text-gray-600">Failed:</span>
                  <div className="font-bold text-lg text-red-600">{failedSteps}</div>
                </div>
                <div>
                  <span className="text-gray-600">Success Rate:</span>
                  <div className="font-bold text-lg">{successRate}%</div>
                </div>
              </div>

              {completedSteps === totalSteps && totalSteps > 0 && (
                <div className="mt-3 p-2 bg-green-100 text-green-800 rounded text-sm font-medium">
                  ✓ Debug completed successfully - Authorization flow analyzed
                </div>
              )}

              {failedSteps > 0 && (
                <div className="mt-3 p-2 bg-red-100 text-red-800 rounded text-sm font-medium">
                  ✗ Debug completed with errors - Check failed steps for details
                </div>
              )}

              {isRunning && (
                <div className="mt-3 p-2 bg-blue-100 text-blue-800 rounded text-sm font-medium">
                  ⚡ Debug in progress - Analyzing authorization flow...
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default DebugPanel;
