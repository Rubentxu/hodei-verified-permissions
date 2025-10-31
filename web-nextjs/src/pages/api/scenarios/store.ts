export interface SavedScenario {
  id: string;
  name: string;
  description: string;
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
  created_at: string;
  updated_at: string;
}

// In-memory storage (en producción usar base de datos)
const scenariosStore: Map<string, SavedScenario> = new Map();

// Seed with some example scenarios
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

export { scenariosStore };
