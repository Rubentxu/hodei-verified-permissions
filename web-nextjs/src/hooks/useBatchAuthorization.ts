import { useMutation } from '@tanstack/react-query';

export interface BatchTestRequest {
  scenarios: Array<{
    name?: string;
    policy_store_id: string;
    principal: {
      entity_type: string;
      entity_id: string;
    };
    action: {
      entity_type: string;
      entity_id: string;
    };
    resource: {
      entity_type: string;
      entity_id: string;
    };
    context?: any;
  }>;
}

export interface BatchTestResult {
  scenario_name: string;
  decision: 'ALLOW' | 'DENY' | 'UNSPECIFIED';
  latency_ms: number;
  determining_policies?: string[];
  errors?: string[];
  success: boolean;
  timestamp: string;
}

export interface BatchTestSummary {
  total: number;
  successful: number;
  failed: number;
  avg_latency_ms: number;
  min_latency_ms: number;
  max_latency_ms: number;
  allow_count: number;
  deny_count: number;
  unspecified_count: number;
}

export interface BatchTestResponse {
  success: boolean;
  results: BatchTestResult[];
  summary: BatchTestSummary;
  timestamp: string;
}

/**
 * Hook para ejecutar tests de autorización en lote
 */
export const useBatchAuthorization = () => {
  return useMutation<BatchTestResponse, Error, BatchTestRequest>({
    mutationFn: async ({ scenarios }) => {
      const response = await fetch('/api/authorize/batch', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ scenarios }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Batch authorization test failed');
      }

      return response.json();
    },
    // Configuración de retry
    retry: 2,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
  });
};

/**
 * Hook para ejecutar múltiples escenarios predefinidos
 */
export const useRunPredefinedScenarios = () => {
  const batchMutation = useBatchAuthorization();

  const runUserAccessTest = () => {
    const scenarios: BatchTestRequest['scenarios'] = [
      {
        name: 'User - View Document',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'alice' },
        action: { entity_type: 'Action', entity_id: 'viewDocument' },
        resource: { entity_type: 'Document', entity_id: 'doc123' },
      },
      {
        name: 'User - Edit Document',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'alice' },
        action: { entity_type: 'Action', entity_id: 'editDocument' },
        resource: { entity_type: 'Document', entity_id: 'doc123' },
      },
      {
        name: 'Admin - Full Access',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'admin' },
        action: { entity_type: 'Action', entity_id: 'fullAccess' },
        resource: { entity_type: 'Resource', entity_id: '*' },
      },
    ];

    return batchMutation.mutate({ scenarios });
  };

  const runRoleBasedTest = () => {
    const scenarios: BatchTestRequest['scenarios'] = [
      {
        name: 'Editor - Edit Own Document',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'bob' },
        action: { entity_type: 'Action', entity_id: 'editDocument' },
        resource: { entity_type: 'Document', entity_id: 'doc456' },
        context: { role: 'editor' },
      },
      {
        name: 'Viewer - View Document',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'charlie' },
        action: { entity_type: 'Action', entity_id: 'viewDocument' },
        resource: { entity_type: 'Document', entity_id: 'doc456' },
        context: { role: 'viewer' },
      },
      {
        name: 'Viewer - Edit Document (Should Deny)',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'charlie' },
        action: { entity_type: 'Action', entity_id: 'editDocument' },
        resource: { entity_type: 'Document', entity_id: 'doc456' },
        context: { role: 'viewer' },
      },
    ];

    return batchMutation.mutate({ scenarios });
  };

  return {
    ...batchMutation,
    runUserAccessTest,
    runRoleBasedTest,
  };
};

/**
 * Hook para exportar resultados a CSV
 */
export const useExportBatchResults = () => {
  const exportToCsv = (results: BatchTestResult[], filename?: string) => {
    const csvHeaders = [
      'Scenario Name',
      'Decision',
      'Latency (ms)',
      'Success',
      'Timestamp',
      'Determining Policies',
      'Errors',
    ];

    const csvRows = results.map(result => [
      result.scenario_name,
      result.decision,
      result.latency_ms.toString(),
      result.success ? 'Yes' : 'No',
      result.timestamp,
      (result.determining_policies || []).join('; '),
      (result.errors || []).join('; '),
    ]);

    const csvContent = [
      csvHeaders.join(','),
      ...csvRows.map(row => 
        row.map(cell => `"${cell.replace(/"/g, '""')}"`).join(',')
      ),
    ].join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    
    if (link.download !== undefined) {
      const url = URL.createObjectURL(blob);
      link.setAttribute('href', url);
      link.setAttribute('download', filename || `batch-test-results-${Date.now()}.csv`);
      link.style.visibility = 'hidden';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
  };

  return { exportToCsv };
};

/**
 * Hook para calcular estadísticas de resultados
 */
export const useBatchResultsStats = (results: BatchTestResult[]) => {
  const stats = React.useMemo(() => {
    const total = results.length;
    const successful = results.filter(r => r.success).length;
    const failed = total - successful;
    
    const decisions = {
      ALLOW: results.filter(r => r.decision === 'ALLOW').length,
      DENY: results.filter(r => r.decision === 'DENY').length,
      UNSPECIFIED: results.filter(r => r.decision === 'UNSPECIFIED').length,
    };

    const latencies = results.map(r => r.latency_ms);
    const avgLatency = latencies.length > 0
      ? Math.round(latencies.reduce((sum, l) => sum + l, 0) / latencies.length)
      : 0;
    const minLatency = latencies.length > 0 ? Math.min(...latencies) : 0;
    const maxLatency = latencies.length > 0 ? Math.max(...latencies) : 0;

    const successRate = total > 0 ? Math.round((successful / total) * 100) : 0;
    const allowRate = total > 0 ? Math.round((decisions.ALLOW / total) * 100) : 0;
    const denyRate = total > 0 ? Math.round((decisions.DENY / total) * 100) : 0;

    return {
      total,
      successful,
      failed,
      decisions,
      latencies: { avg: avgLatency, min: minLatency, max: maxLatency },
      rates: {
        success: successRate,
        allow: allowRate,
        deny: denyRate,
      },
    };
  }, [results]);

  return stats;
};

// Import React
import React from 'react';
