import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

interface ListPoliciesParams {
  policy_store_id: string;
  max_results?: number;
  next_token?: string;
}

export const usePolicies = (params: ListPoliciesParams) => {
  return useQuery({
    queryKey: ['policies', params.policy_store_id],
    queryFn: async () => {
      const queryParams = new URLSearchParams();
      queryParams.append('policy_store_id', params.policy_store_id);
      if (params.max_results) queryParams.append('max_results', params.max_results.toString());
      if (params.next_token) queryParams.append('next_token', params.next_token);

      const response = await fetch(`/api/policies?${queryParams.toString()}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch policies: ${response.statusText}`);
      }
      return response.json();
    },
    enabled: !!params.policy_store_id,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const usePolicy = (policyStoreId: string, policyId: string) => {
  return useQuery({
    queryKey: ['policies', policyStoreId, policyId],
    queryFn: async () => {
      const response = await fetch(`/api/policies/${policyId}?policy_store_id=${policyStoreId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch policy: ${response.statusText}`);
      }
      return response.json();
    },
    enabled: !!policyStoreId && !!policyId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const useCreatePolicy = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      definition: any;
    }) => {
      const response = await fetch('/api/policies', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create policy');
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch policies list for this policy store
      queryClient.invalidateQueries({
        queryKey: ['policies', variables.policy_store_id]
      });
    },
  });
};

export const useUpdatePolicy = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      policy_id: string;
      definition: any;
    }) => {
      const response = await fetch(`/api/policies/${data.policy_id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update policy');
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch policies list and specific policy
      queryClient.invalidateQueries({
        queryKey: ['policies', variables.policy_store_id]
      });
      queryClient.invalidateQueries({
        queryKey: ['policies', variables.policy_store_id, variables.policy_id]
      });
    },
  });
};

export const useDeletePolicy = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      policy_id: string;
    }) => {
      const response = await fetch(`/api/policies/${data.policy_id}?policy_store_id=${data.policy_store_id}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to delete policy');
      }
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch policies list and remove specific policy from cache
      queryClient.invalidateQueries({
        queryKey: ['policies', variables.policy_store_id]
      });
      queryClient.removeQueries({
        queryKey: ['policies', variables.policy_store_id, variables.policy_id]
      });
    },
  });
};
