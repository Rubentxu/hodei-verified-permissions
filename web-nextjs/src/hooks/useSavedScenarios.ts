import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

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

interface CreateScenarioData {
  name: string;
  description?: string;
  policy_store_id: string;
  principal: { entity_type: string; entity_id: string };
  action: { entity_type: string; entity_id: string };
  resource: { entity_type: string; entity_id: string };
  context?: any;
}

interface UpdateScenarioData {
  name?: string;
  description?: string;
  policy_store_id?: string;
  principal?: { entity_type: string; entity_id: string };
  action?: { entity_type: string; entity_id: string };
  resource?: { entity_type: string; entity_id: string };
  context?: any;
}

/**
 * Hook para obtener la lista de escenarios guardados
 */
export const useSavedScenarios = (policyStoreId?: string) => {
  return useQuery<SavedScenario[]>({
    queryKey: ['saved-scenarios', policyStoreId],
    queryFn: async () => {
      const queryParams = policyStoreId 
        ? `?policy_store_id=${policyStoreId}`
        : '';
      const response = await fetch(`/api/scenarios${queryParams}`);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch scenarios: ${response.statusText}`);
      }
      
      return response.json();
    },
    staleTime: 5 * 60 * 1000, // 5 minutos
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
};

/**
 * Hook para obtener un escenario específico por ID
 */
export const useSavedScenario = (scenarioId: string) => {
  return useQuery<SavedScenario>({
    queryKey: ['saved-scenarios', 'single', scenarioId],
    queryFn: async () => {
      const response = await fetch(`/api/scenarios/${scenarioId}`);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch scenario: ${response.statusText}`);
      }
      
      return response.json();
    },
    enabled: !!scenarioId,
    staleTime: 5 * 60 * 1000,
    retry: 3,
  });
};

/**
 * Hook para guardar un nuevo escenario
 */
export const useSaveScenario = () => {
  const queryClient = useQueryClient();
  
  return useMutation<SavedScenario, Error, CreateScenarioData>({
    mutationFn: async (scenarioData) => {
      const response = await fetch('/api/scenarios', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(scenarioData),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to save scenario');
      }

      return response.json();
    },
    onSuccess: (savedScenario) => {
      // Invalidar y refetch la lista de escenarios
      queryClient.invalidateQueries({ queryKey: ['saved-scenarios'] });
      
      // También guardar en localStorage como backup
      try {
        const existingScenarios = JSON.parse(
          localStorage.getItem('hodei-scenarios') || '[]'
        );
        existingScenarios.push(savedScenario);
        localStorage.setItem('hodei-scenarios', JSON.stringify(existingScenarios));
      } catch (error) {
        console.warn('Failed to save to localStorage:', error);
      }
    },
  });
};

/**
 * Hook para actualizar un escenario existente
 */
export const useUpdateScenario = () => {
  const queryClient = useQueryClient();
  
  return useMutation<SavedScenario, Error, { id: string; data: UpdateScenarioData }>({
    mutationFn: async ({ id, data }) => {
      const response = await fetch(`/api/scenarios/${id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update scenario');
      }

      return response.json();
    },
    onSuccess: (updatedScenario) => {
      queryClient.invalidateQueries({ queryKey: ['saved-scenarios'] });
      queryClient.setQueryData(
        ['saved-scenarios', 'single', updatedScenario.id],
        updatedScenario
      );
      
      // Actualizar localStorage
      try {
        const existingScenarios = JSON.parse(
          localStorage.getItem('hodei-scenarios') || '[]'
        );
        const index = existingScenarios.findIndex((s: SavedScenario) => s.id === updatedScenario.id);
        if (index !== -1) {
          existingScenarios[index] = updatedScenario;
          localStorage.setItem('hodei-scenarios', JSON.stringify(existingScenarios));
        }
      } catch (error) {
        console.warn('Failed to update localStorage:', error);
      }
    },
  });
};

/**
 * Hook para eliminar un escenario
 */
export const useDeleteScenario = () => {
  const queryClient = useQueryClient();
  
  return useMutation<void, Error, string>({
    mutationFn: async (scenarioId: string) => {
      const response = await fetch(`/api/scenarios/${scenarioId}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to delete scenario');
      }
    },
    onSuccess: (_, scenarioId) => {
      queryClient.invalidateQueries({ queryKey: ['saved-scenarios'] });
      queryClient.removeQueries({ queryKey: ['saved-scenarios', 'single', scenarioId] });
      
      // Eliminar de localStorage
      try {
        const existingScenarios = JSON.parse(
          localStorage.getItem('hodei-scenarios') || '[]'
        );
        const filteredScenarios = existingScenarios.filter((s: SavedScenario) => s.id !== scenarioId);
        localStorage.setItem('hodei-scenarios', JSON.stringify(filteredScenarios));
      } catch (error) {
        console.warn('Failed to update localStorage:', error);
      }
    },
  });
};

/**
 * Hook para cargar escenarios desde localStorage como fallback
 */
export const useLocalScenarios = () => {
  return useQuery<SavedScenario[]>({
    queryKey: ['local-scenarios'],
    queryFn: () => {
      try {
        const scenarios = localStorage.getItem('hodei-scenarios');
        return scenarios ? JSON.parse(scenarios) : [];
      } catch (error) {
        console.warn('Failed to load from localStorage:', error);
        return [];
      }
    },
    staleTime: Infinity, // LocalStorage no cambia automáticamente
  });
};

/**
 * Hook combinado que carga desde backend primero, luego localStorage
 */
export const useAllScenarios = (policyStoreId?: string) => {
  const { data: backendScenarios, isLoading: backendLoading } = useSavedScenarios(policyStoreId);
  const { data: localScenarios } = useLocalScenarios();
  
  // Combinar y deduplicar por ID
  const allScenarios = React.useMemo(() => {
    if (!backendScenarios && !localScenarios) return [];
    
    const combined = [
      ...(backendScenarios || []),
      ...(localScenarios?.filter(ls => 
        !backendScenarios?.find(bs => bs.id === ls.id)
      ) || [])
    ];
    
    // Ordenar por updated_at descendente
    return combined.sort((a, b) => 
      new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
    );
  }, [backendScenarios, localScenarios]);
  
  return {
    data: allScenarios,
    isLoading: backendLoading,
    hasBackendData: !!backendScenarios?.length,
    hasLocalData: !!localScenarios?.length,
  };
};

// Import React for useMemo (needed for useAllScenarios)
import React from 'react';
