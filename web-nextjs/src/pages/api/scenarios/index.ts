import type { NextApiRequest, NextApiResponse } from 'next';
import { grpcClients } from '@/lib/grpc/node-client';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';
import { SavedScenario, scenariosStore } from './store';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method === 'GET') {
    const { policy_store_id } = req.query;

    try {
      let scenarios = Array.from(scenariosStore.values());

      // Filtrar por policy_store_id si se especifica
      if (policy_store_id && typeof policy_store_id === 'string') {
        scenarios = scenarios.filter(s => s.policy_store_id === policy_store_id);
      }

      // Ordenar por updated_at descendente
      scenarios.sort((a, b) => 
        new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
      );

      res.status(200).json(scenarios);
    } catch (error) {
      console.error('Get scenarios error:', error);
      res.status(500).json({ 
        error: error instanceof Error ? error.message : 'Failed to fetch scenarios' 
      });
    }
  } else if (req.method === 'POST') {
    try {
      const {
        name,
        description,
        policy_store_id,
        principal,
        action,
        resource,
        context,
      } = req.body;

      // Validaciones
      if (!name || !policy_store_id || !principal || !action || !resource) {
        return res.status(400).json({
          error: 'Missing required fields: name, policy_store_id, principal, action, resource',
        });
      }

      const scenario: SavedScenario = {
        id: `scenario_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        name,
        description: description || '',
        policy_store_id,
        principal,
        action,
        resource,
        context: context || {},
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };

      scenariosStore.set(scenario.id, scenario);

      res.status(201).json(scenario);
    } catch (error) {
      console.error('Create scenario error:', error);
      res.status(500).json({
        error: error instanceof Error ? error.message : 'Failed to create scenario',
      });
    }
  } else {
    res.status(405).json({ error: 'Method not allowed' });
  }
}
