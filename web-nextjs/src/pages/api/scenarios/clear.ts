import type { NextApiRequest, NextApiResponse } from 'next';
import { scenariosStore } from './store';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method === 'POST') {
    try {
      // Clear all scenarios (for testing purposes)
      scenariosStore.clear();

      // Re-seed with example scenarios
      scenariosStore.set('scenario_1', {
        id: 'scenario_1',
        name: 'User Document Access',
        description: 'Test basic user access to document resources',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'alice' },
        action: { entity_type: 'Action', entity_id: 'viewDocument' },
        resource: { entity_type: 'Document', entity_id: 'doc123' },
        context: {},
        created_at: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
        updated_at: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
      });

      scenariosStore.set('scenario_2', {
        id: 'scenario_2',
        name: 'Admin Full Access',
        description: 'Test admin privileges on all resources',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'admin' },
        action: { entity_type: 'Action', entity_id: 'fullAccess' },
        resource: { entity_type: 'Resource', entity_id: '*' },
        context: {},
        created_at: new Date(Date.now() - 48 * 60 * 60 * 1000).toISOString(),
        updated_at: new Date(Date.now() - 12 * 60 * 60 * 1000).toISOString(),
      });

      scenariosStore.set('scenario_3', {
        id: 'scenario_3',
        name: 'Role-Based Access',
        description: 'Test role-based permissions',
        policy_store_id: 'ps_123',
        principal: { entity_type: 'User', entity_id: 'bob' },
        action: { entity_type: 'Action', entity_id: 'editDocument' },
        resource: { entity_type: 'Document', entity_id: 'doc456' },
        context: { role: 'editor' },
        created_at: new Date(Date.now() - 72 * 60 * 60 * 1000).toISOString(),
        updated_at: new Date(Date.now() - 6 * 60 * 60 * 1000).toISOString(),
      });

      res.status(200).json({ message: 'Scenarios cleared and re-seeded' });
    } catch (error) {
      console.error('Clear scenarios error:', error);
      res.status(500).json({
        error: error instanceof Error ? error.message : 'Failed to clear scenarios',
      });
    }
  } else {
    res.status(405).json({ error: 'Method not allowed' });
  }
}
