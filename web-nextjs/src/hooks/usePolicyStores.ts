import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type {
  ListPolicyStoresResponse,
  CreatePolicyStoreResponse
} from '@/lib/grpc/node-client';

interface ListPolicyStoresParams {
  max_results?: number;
  next_token?: string;
}

export const usePolicyStores = (params?: ListPolicyStoresParams) => {
  return useQuery<ListPolicyStoresResponse>({
    queryKey: ['policy-stores', params],
    queryFn: async () => {
      const queryParams = new URLSearchParams();
      if (params?.max_results) queryParams.append('max_results', params.max_results.toString());
      if (params?.next_token) queryParams.append('next_token', params.next_token);

      const response = await fetch(`/api/policy-stores?${queryParams.toString()}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch policy stores: ${response.statusText}`);
      }
      return response.json();
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const usePolicyStore = (policyStoreId: string) => {
  return useQuery({
    queryKey: ['policy-stores', policyStoreId],
    queryFn: async () => {
      const response = await fetch(`/api/policy-stores/${policyStoreId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch policy store: ${response.statusText}`);
      }
      return response.json();
    },
    enabled: !!policyStoreId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const useCreatePolicyStore = () => {
  const queryClient = useQueryClient();

  return useMutation<CreatePolicyStoreResponse, Error, { description: string }>({
    mutationFn: async (data) => {
      const response = await fetch('/api/policy-stores', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create policy store');
      }

      return response.json();
    },
    onSuccess: () => {
      // Invalidate and refetch policy stores list
      queryClient.invalidateQueries({ queryKey: ['policy-stores'] });
    },
  });
};

export const useDeletePolicyStore = () => {
  const queryClient = useQueryClient();

  return useMutation<void, Error, string>({
    mutationFn: async (policyStoreId: string) => {
      const response = await fetch(`/api/policy-stores/${policyStoreId}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to delete policy store');
      }
    },
    onSuccess: (_, policyStoreId) => {
      // Invalidate and refetch policy stores list
      queryClient.invalidateQueries({ queryKey: ['policy-stores'] });
      // Remove the specific policy store from cache
      queryClient.removeQueries({ queryKey: ['policy-stores', policyStoreId] });
    },
  });
};
