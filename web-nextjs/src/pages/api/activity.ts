import type { NextApiRequest, NextApiResponse } from 'next';

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

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    // TODO: En producción, consultar logs reales del backend
    // Por ahora, mock data estructurado basado en actividad típica
    const activities: Activity[] = [
      {
        id: '1',
        type: 'policy',
        action: 'updated',
        resource: 'document-access-policy',
        description: 'Updated document access policy for user roles',
        timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        user: 'admin@example.com',
        changes: ['Added new user role', 'Updated resource permissions'],
      },
      {
        id: '2',
        type: 'schema',
        action: 'created',
        resource: 'User',
        description: 'Added new User entity type with attributes',
        timestamp: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
        user: 'developer@example.com',
        changes: ['Added attributes: department, role, clearance_level'],
      },
      {
        id: '3',
        type: 'policy_store',
        action: 'created',
        resource: 'production-store',
        description: 'Created new policy store for production environment',
        timestamp: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
        user: 'admin@example.com',
      },
      {
        id: '4',
        type: 'policy',
        action: 'deleted',
        resource: 'old-legacy-policy',
        description: 'Removed deprecated legacy policy',
        timestamp: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
        user: 'admin@example.com',
      },
      {
        id: '5',
        type: 'template',
        action: 'updated',
        resource: 'employee-access-template',
        description: 'Updated employee access template with new permissions',
        timestamp: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString(),
        user: 'security@example.com',
        changes: ['Added database read access', 'Removed legacy file permissions'],
      },
      {
        id: '6',
        type: 'schema',
        action: 'updated',
        resource: 'Document',
        description: 'Updated Document entity attributes',
        timestamp: new Date(Date.now() - 4 * 24 * 60 * 60 * 1000).toISOString(),
        user: 'developer@example.com',
        changes: ['Added classification attribute', 'Updated security_level'],
      },
    ];

    // Ordenar por timestamp (más reciente primero)
    activities.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

    // Limitar a los últimos 20 eventos
    const recentActivities = activities.slice(0, 20);

    res.status(200).json({
      success: true,
      activities: recentActivities,
      total: recentActivities.length,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error('Activity API error:', error);
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    });
  }
}
