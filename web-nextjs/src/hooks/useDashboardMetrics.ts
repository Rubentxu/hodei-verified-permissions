import { useQuery } from '@tanstack/react-query';

interface Trend {
  value: number;
  isPositive: boolean;
}

interface Metrics {
  policyStores: {
    total: number;
    trend: Trend;
  };
  policies: {
    total: number;
    trend: Trend;
  };
  schemas: {
    total: number;
    trend: Trend;
  };
  templates: {
    total: number;
    trend: Trend;
  };
}

interface ChartData {
  authorizationRequests: Array<{
    name: string;
    requests: number;
    allowed: number;
    denied: number;
  }>;
  decisions: {
    allowed: number;
    denied: number;
  };
}

interface DashboardMetricsResponse {
  success: boolean;
  metrics: Metrics;
  chartData: ChartData;
  timestamp: string;
}

interface Activity {
  id: string;
  type: 'policy' | 'schema' | 'policy_store' | 'template';
  action: 'created' | 'updated' | 'deleted';
  resource: string;
  description: string;
  timestamp: string;
  user: string;
  changes?: string[];
}

interface ActivityFeedResponse {
  success: boolean;
  activities: Activity[];
  total: number;
  timestamp: string;
}

interface HealthStatus {
  grpc_server: 'connected' | 'disconnected';
  database?: 'connected' | 'disconnected';
  last_check?: string;
}

/**
 * Hook para obtener métricas del dashboard desde el backend
 */
export const useDashboardMetrics = () => {
  return useQuery<DashboardMetricsResponse>({
    queryKey: ['dashboard', 'metrics'],
    queryFn: async () => {
      const response = await fetch('/api/metrics');
      
      if (!response.ok) {
        throw new Error(`Failed to fetch metrics: ${response.statusText}`);
      }
      
      return response.json();
    },
    refetchInterval: 30000, // Actualizar cada 30 segundos
    staleTime: 25000, // 25 segundos
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
};

/**
 * Hook para obtener el feed de actividad reciente
 */
export const useActivityFeed = () => {
  return useQuery<ActivityFeedResponse>({
    queryKey: ['dashboard', 'activity'],
    queryFn: async () => {
      const response = await fetch('/api/activity');
      
      if (!response.ok) {
        throw new Error(`Failed to fetch activity: ${response.statusText}`);
      }
      
      return response.json();
    },
    refetchInterval: 60000, // Actualizar cada 60 segundos
    staleTime: 50000, // 50 segundos
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
};

/**
 * Hook para obtener el estado de salud del sistema
 */
export const useHealthStatus = () => {
  return useQuery<HealthStatus>({
    queryKey: ['dashboard', 'health'],
    queryFn: async () => {
      const response = await fetch('/api/health');
      
      if (!response.ok) {
        throw new Error(`Failed to fetch health status: ${response.statusText}`);
      }
      
      const data = await response.json();
      
      return {
        grpc_server: data?.grpc_server === 'connected' ? 'connected' : 'disconnected',
        database: 'connected', // TODO: Obtener de backend real
        last_check: data?.timestamp,
      };
    },
    refetchInterval: 10000, // Health check cada 10 segundos
    staleTime: 5000, // 5 segundos
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
  });
};

/**
 * Hook combinado para obtener todos los datos del dashboard
 */
export const useDashboardData = () => {
  const metrics = useDashboardMetrics();
  const activity = useActivityFeed();
  const health = useHealthStatus();

  return {
    metrics,
    activity,
    health,
    isLoading: metrics.isLoading || activity.isLoading || health.isLoading,
    isError: metrics.isError || activity.isError || health.isError,
    error: metrics.error || activity.error || health.error,
    refetch: async () => {
      // Refetch all queries in parallel
      await Promise.all([
        metrics.refetch(),
        activity.refetch(),
        health.refetch(),
      ]);
    },
  };
};
