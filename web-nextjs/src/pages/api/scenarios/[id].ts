import type { NextApiRequest, NextApiResponse } from 'next';
import { SavedScenario, scenariosStore } from './store';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { id } = req.query;

  if (!id || typeof id !== 'string') {
    return res.status(400).json({ error: 'Invalid scenario ID' });
  }

  if (req.method === 'GET') {
    try {
      const scenario = scenariosStore.get(id);
      
      if (!scenario) {
        return res.status(404).json({ error: 'Scenario not found' });
      }

      res.status(200).json(scenario);
    } catch (error) {
      console.error('Get scenario error:', error);
      res.status(500).json({
        error: error instanceof Error ? error.message : 'Failed to fetch scenario',
      });
    }
  } else if (req.method === 'PUT') {
    try {
      const existingScenario = scenariosStore.get(id);
      
      if (!existingScenario) {
        return res.status(404).json({ error: 'Scenario not found' });
      }

      const updates = req.body;
      const updatedScenario: SavedScenario = {
        ...existingScenario,
        ...updates,
        id, // Don't allow changing ID
        updated_at: new Date().toISOString(),
      };

      scenariosStore.set(id, updatedScenario);

      res.status(200).json(updatedScenario);
    } catch (error) {
      console.error('Update scenario error:', error);
      res.status(500).json({
        error: error instanceof Error ? error.message : 'Failed to update scenario',
      });
    }
  } else if (req.method === 'DELETE') {
    try {
      const existed = scenariosStore.delete(id);
      
      if (!existed) {
        return res.status(404).json({ error: 'Scenario not found' });
      }

      res.status(204).send('');
    } catch (error) {
      console.error('Delete scenario error:', error);
      res.status(500).json({
        error: error instanceof Error ? error.message : 'Failed to delete scenario',
      });
    }
  } else {
    res.status(405).json({ error: 'Method not allowed' });
  }
}
