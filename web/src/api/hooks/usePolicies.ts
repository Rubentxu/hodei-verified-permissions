/**
 * usePolicies - React Query hooks for Policies API
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Policy } from '../../types';

// Mock API client
const mockApiClient = {
  listPolicies: async (policyStoreId: string) => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return {
      policies: [
        {
          policyId: 'policy-1',
          policyStoreId,
          statement: 'permit(principal == User::"alice", action == Action::"read", resource == Document::"doc1");',
          description: 'Allow alice to read doc1',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      ],
      nextToken: undefined,
    };
  },

  createPolicy: async (
    policyStoreId: string,
    policyId: string,
    statement: string,
    description?: string
  ): Promise<Policy> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return {
      policyId,
      policyStoreId,
      statement,
      description,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  },

  getPolicy: async (policyStoreId: string, policyId: string): Promise<Policy> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    return {
      policyId,
      policyStoreId,
      statement: 'permit(principal, action, resource);',
      description: 'Test policy',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  },

  updatePolicy: async (
    policyStoreId: string,
    policyId: string,
    statement: string,
    description?: string
  ): Promise<Policy> => {
    await new Promise((resolve) => setTimeout(resolve, 500));
    return {
      policyId,
      policyStoreId,
      statement,
      description,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  },

  deletePolicy: async (policyStoreId: string, policyId: string): Promise<void> => {
    await new Promise((resolve) => setTimeout(resolve, 300));
  },
};

/**
 * Hook to list policies in a policy store
 */
export const usePolicies = (policyStoreId: string) => {
  return useQuery({
    queryKey: ['policies', policyStoreId],
    queryFn: () => mockApiClient.listPolicies(policyStoreId),
    enabled: !!policyStoreId,
    staleTime: 5 * 60 * 1000,
    retry: 2,
  });
};

/**
 * Hook to fetch a single policy
 */
export const usePolicy = (policyStoreId: string, policyId: string) => {
  return useQuery({
    queryKey: ['policy', policyStoreId, policyId],
    queryFn: () => mockApiClient.getPolicy(policyStoreId, policyId),
    enabled: !!policyStoreId && !!policyId,
    staleTime: 5 * 60 * 1000,
    retry: 2,
  });
};

/**
 * Hook to create a policy
 */
export const useCreatePolicy = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      policyStoreId,
      policyId,
      statement,
      description,
    }: {
      policyStoreId: string;
      policyId: string;
      statement: string;
      description?: string;
    }) => mockApiClient.createPolicy(policyStoreId, policyId, statement, description),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['policies', data.policyStoreId] });
    },
  });
};

/**
 * Hook to update a policy
 */
export const useUpdatePolicy = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      policyStoreId,
      policyId,
      statement,
      description,
    }: {
      policyStoreId: string;
      policyId: string;
      statement: string;
      description?: string;
    }) => mockApiClient.updatePolicy(policyStoreId, policyId, statement, description),
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['policies', data.policyStoreId] });
      queryClient.invalidateQueries({ queryKey: ['policy', data.policyStoreId, data.policyId] });
    },
  });
};

/**
 * Hook to delete a policy
 */
export const useDeletePolicy = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ policyStoreId, policyId }: { policyStoreId: string; policyId: string }) =>
      mockApiClient.deletePolicy(policyStoreId, policyId),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['policies', variables.policyStoreId] });
    },
  });
};
