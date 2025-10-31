import type { NextApiRequest, NextApiResponse } from 'next';

interface BatchTestRequest {
  scenarios: Array<{
    name?: string;
    policy_store_id: string;
    principal: any;
    action: any;
    resource: any;
    context?: any;
  }>;
}

interface BatchTestResult {
  scenario_name: string;
  decision: 'ALLOW' | 'DENY' | 'UNSPECIFIED';
  latency_ms: number;
  determining_policies?: string[];
  errors?: string[];
  success: boolean;
  timestamp: string;
}

interface BatchTestSummary {
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

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { scenarios }: BatchTestRequest = req.body;

  if (!scenarios || !Array.isArray(scenarios) || scenarios.length === 0) {
    return res.status(400).json({ 
      error: 'Invalid request: scenarios must be a non-empty array' 
    });
  }

  if (scenarios.length > 100) {
    return res.status(400).json({ 
      error: 'Maximum 100 scenarios allowed per batch test' 
    });
  }

  try {
    const results: BatchTestResult[] = [];
    const latencies: number[] = [];
    const decisions = { ALLOW: 0, DENY: 0, UNSPECIFIED: 0 };

    // Process each scenario
    for (let index = 0; index < scenarios.length; index++) {
      const scenario = scenarios[index];
      const startTime = Date.now();
      
      try {
        // Simulate API call to /api/authorize
        const response = await fetch(`${req.headers.origin}/api/authorize`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            policy_store_id: scenario.policy_store_id,
            principal: scenario.principal,
            action: scenario.action,
            resource: scenario.resource,
            context: scenario.context || '{}',
            entities: [],
          }),
        });
        
        const result = await response.json();
        const duration = Date.now() - startTime;
        latencies.push(duration);
        
        const batchResult: BatchTestResult = {
          scenario_name: scenario.name || `Scenario ${index + 1}`,
          decision: result.decision || 'UNSPECIFIED',
          latency_ms: duration,
          determining_policies: result.determining_policies,
          errors: result.errors,
          success: response.ok,
          timestamp: new Date().toISOString(),
        };
        
        results.push(batchResult);
        
        // Count decisions
        if (batchResult.decision in decisions) {
          decisions[batchResult.decision as keyof typeof decisions]++;
        }
        
      } catch (error) {
        const duration = Date.now() - startTime;
        latencies.push(duration);
        
        results.push({
          scenario_name: scenario.name || `Scenario ${index + 1}`,
          decision: 'UNSPECIFIED',
          latency_ms: duration,
          errors: [error instanceof Error ? error.message : 'Unknown error'],
          success: false,
          timestamp: new Date().toISOString(),
        });
      }
    }

    // Calculate summary statistics
    const successful = results.filter(r => r.success).length;
    const failed = results.length - successful;
    const avgLatency = latencies.length > 0 
      ? Math.round(latencies.reduce((sum, l) => sum + l, 0) / latencies.length)
      : 0;
    const minLatency = latencies.length > 0 ? Math.min(...latencies) : 0;
    const maxLatency = latencies.length > 0 ? Math.max(...latencies) : 0;

    const summary: BatchTestSummary = {
      total: results.length,
      successful,
      failed,
      avg_latency_ms: avgLatency,
      min_latency_ms: minLatency,
      max_latency_ms: maxLatency,
      allow_count: decisions.ALLOW,
      deny_count: decisions.DENY,
      unspecified_count: decisions.UNSPECIFIED,
    };

    res.status(200).json({
      success: true,
      results,
      summary,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error('Batch authorization error:', error);
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Batch test failed',
    });
  }
}
