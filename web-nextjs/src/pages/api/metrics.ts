import type { NextApiRequest, NextApiResponse } from 'next';
import { grpcClients } from '@/lib/grpc/node-client';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    // Obtener datos reales del backend gRPC
    const [policyStoresResponse] = await Promise.all([
      grpcClients.listPolicyStores({ max_results: 1 }).catch(() => null),
    ]);

    // Calcular métricas de policy stores reales
    const policyStoresCount = policyStoresResponse?.policy_stores?.length || 0;

    // Datos históricos para gráficos (en producción vendría de analytics)
    const last7Days = Array.from({ length: 7 }, (_, i) => {
      const date = new Date();
      date.setDate(date.getDate() - (6 - i));
      return {
        name: date.toLocaleDateString('en-US', { weekday: 'short' }),
        requests: Math.floor(Math.random() * 50) + 20,
        allowed: Math.floor(Math.random() * 40) + 15,
        denied: Math.floor(Math.random() * 10) + 5,
      };
    });

    // Mock data para otras métricas (en producción vendrían de endpoints dedicados)
    const calculateTrend = (current: number, previous: number) => {
      const change = current - previous;
      const percentChange = previous > 0 ? ((change / previous) * 100) : 0;
      return {
        value: Math.abs(parseFloat(percentChange.toFixed(1))),
        isPositive: change >= 0,
      };
    };

    const metrics = {
      policyStores: {
        total: policyStoresCount,
        trend: calculateTrend(policyStoresCount, 10),
      },
      policies: {
        total: 156,
        trend: calculateTrend(156, 148),
      },
      schemas: {
        total: 24,
        trend: calculateTrend(24, 23),
      },
      templates: {
        total: 18,
        trend: calculateTrend(18, 18),
      },
    };

    const totalAllowed = last7Days.reduce((sum, day) => sum + day.allowed, 0);
    const totalDenied = last7Days.reduce((sum, day) => sum + day.denied, 0);

    const chartData = {
      authorizationRequests: last7Days,
      decisions: {
        allowed: totalAllowed,
        denied: totalDenied,
      },
    };

    res.status(200).json({
      success: true,
      metrics,
      chartData,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error('Metrics API error:', error);
    const grpcError = handleGRPCError(error);
    res.status(500).json({
      success: false,
      error: grpcError.message,
    });
  }
}
