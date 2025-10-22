/**
 * usePolicyStores - React Query hooks for Policy Stores API
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ListPolicyStoresResponse, PolicyStore } from '../../types';

// Mock API client - will be replaced with gRPC-Web client
const mockApiClient = {
  listPolicyStores: async (): Promise<ListPolicyStoresResponse> => {
    // Simulate API delay
    await new Promise((resolve) => setTimeout(resolve, 500));
    return {
      policyStores: [
        {
          policyStoreId: '550e8400-e29b-41d4-a716-446655440000',
          description: 'Production Policy Store',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      ],
      nextToken: undefined,
    };
  },

  createPolicyStore: async (description?: string): Promise<PolicyStore> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return {
      policyStoreId: `550e8400-e29b-41d4-a716-${Math.random().toString().slice(2, 12)}`,
      description,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  },

  getPolicyStore: async (id: string): Promise<PolicyStore> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return {
      policyStoreId: id,
      description: 'Test Policy Store',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  },

  deletePolicyStore: async (id: string): Promise<void> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
  },
};

/**
 * Hook to fetch all policy stores
 */
export const usePolicyStores = () => {
  return useQuery({
    queryKey: ['policyStores'],
    queryFn: () => mockApiClient.listPolicyStores(),
    staleTime: 5 * 60 * 1000, // 5 minutes
    retry: 2,
  });
};

/**
 * Hook to fetch a single policy store
 */
export const usePolicyStore = (id: string) => {
  return useQuery({
    queryKey: ['policyStore', id],
    queryFn: () => mockApiClient.getPolicyStore(id),
    enabled: !!id,
    staleTime: 5 * 60 * 1000,
    retry: 2,
  });
};

/**
 * Hook to create a policy store
 */
export const useCreatePolicyStore = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (description?: string) =>
      mockApiClient.createPolicyStore(description),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['policyStores'] });
    },
  });
};

/**
 * Hook to delete a policy store
 */
export const useDeletePolicyStore = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => mockApiClient.deletePolicyStore(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['policyStores'] });
    },
  });
};
