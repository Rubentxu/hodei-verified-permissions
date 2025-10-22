/**
 * useSchemas - React Query hooks for Schema API
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Schema } from '../../types';

// Mock API client
const mockApiClient = {
  getSchema: async (policyStoreId: string): Promise<Schema> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return {
      policyStoreId,
      schema: JSON.stringify({
        App: {
          entityTypes: {
            User: {
              shape: {
                type: 'Record',
                attributes: {
                  department: { type: 'String' },
                },
              },
            },
            Document: {
              shape: {
                type: 'Record',
                attributes: {
                  owner: { type: 'String' },
                },
              },
            },
          },
          actions: {
            read: {
              appliesTo: {
                principalTypes: ['User'],
                resourceTypes: ['Document'],
              },
            },
            write: {
              appliesTo: {
                principalTypes: ['User'],
                resourceTypes: ['Document'],
              },
            },
          },
        },
      }, null, 2),
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  },

  putSchema: async (policyStoreId: string, schema: string): Promise<Schema> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return {
      policyStoreId,
      schema,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  },

  validateSchema: async (schema: string): Promise<{ isValid: boolean; errors: string[] }> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    try {
      JSON.parse(schema);
      return { isValid: true, errors: [] };
    } catch (error) {
      return {
        isValid: false,
        errors: [error instanceof Error ? error.message : 'Invalid JSON'],
      };
    }
  },
};

/**
 * Hook to fetch schema for a policy store
 */
export const useSchema = (policyStoreId: string) => {
  return useQuery({
    queryKey: ['schema', policyStoreId],
    queryFn: () => mockApiClient.getSchema(policyStoreId),
    enabled: !!policyStoreId,
    staleTime: 10 * 60 * 1000, // 10 minutes
    retry: 2,
  });
};

/**
 * Hook to update schema
 */
export const useUpdateSchema = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ policyStoreId, schema }: { policyStoreId: string; schema: string }) =>
      mockApiClient.putSchema(policyStoreId, schema),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['schema', data.policyStoreId] });
    },
  });
};

/**
 * Hook to validate schema
 */
export const useValidateSchema = () => {
  return useMutation({
    mutationFn: (schema: string) => mockApiClient.validateSchema(schema),
  });
};
