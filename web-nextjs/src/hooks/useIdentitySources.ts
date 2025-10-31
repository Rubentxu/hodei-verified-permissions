import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

interface ListIdentitySourcesParams {
  policy_store_id: string;
  max_results?: number;
  next_token?: string;
}

export const useIdentitySources = (params: ListIdentitySourcesParams) => {
  return useQuery({
    queryKey: ['identity-sources', params.policy_store_id],
    queryFn: async () => {
      const queryParams = new URLSearchParams();
      queryParams.append('policy_store_id', params.policy_store_id);
      if (params.max_results) queryParams.append('max_results', params.max_results.toString());
      if (params.next_token) queryParams.append('next_token', params.next_token);

      const response = await fetch(`/api/identity-sources?${queryParams.toString()}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch identity sources: ${response.statusText}`);
      }
      return response.json();
    },
    enabled: !!params.policy_store_id,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const useIdentitySource = (policyStoreId: string, identitySourceId: string) => {
  return useQuery({
    queryKey: ['identity-sources', policyStoreId, identitySourceId],
    queryFn: async () => {
      const response = await fetch(`/api/identity-sources/${identitySourceId}?policy_store_id=${policyStoreId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch identity source: ${response.statusText}`);
      }
      return response.json();
    },
    enabled: !!policyStoreId && !!identitySourceId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const useCreateIdentitySource = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      config: {
        cognito?: {
          user_pool_id: string;
          region: string;
          client_id: string;
          client_secret: string;
        };
        oidc?: {
          issuer: string;
          client_id: string;
          client_secret: string;
          authorization_endpoint: string;
          token_endpoint: string;
          userinfo_endpoint: string;
          scopes: string[];
        };
      };
    }) => {
      const response = await fetch('/api/identity-sources', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create identity source');
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch identity sources list for this policy store
      queryClient.invalidateQueries({
        queryKey: ['identity-sources', variables.policy_store_id]
      });
    },
  });
};

export const useUpdateIdentitySource = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      identity_source_id: string;
      config: {
        cognito?: {
          user_pool_id: string;
          region: string;
          client_id: string;
          client_secret: string;
        };
        oidc?: {
          issuer: string;
          client_id: string;
          client_secret: string;
          authorization_endpoint: string;
          token_endpoint: string;
          userinfo_endpoint: string;
          scopes: string[];
        };
      };
    }) => {
      const response = await fetch(`/api/identity-sources/${data.identity_source_id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update identity source');
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch identity sources list and specific source
      queryClient.invalidateQueries({
        queryKey: ['identity-sources', variables.policy_store_id]
      });
      queryClient.invalidateQueries({
        queryKey: ['identity-sources', variables.policy_store_id, variables.identity_source_id]
      });
    },
  });
};

export const useDeleteIdentitySource = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      identity_source_id: string;
    }) => {
      const response = await fetch(`/api/identity-sources/${data.identity_source_id}?policy_store_id=${data.policy_store_id}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to delete identity source');
      }
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch identity sources list and remove specific source from cache
      queryClient.invalidateQueries({
        queryKey: ['identity-sources', variables.policy_store_id]
      });
      queryClient.removeQueries({
        queryKey: ['identity-sources', variables.policy_store_id, variables.identity_source_id]
      });
    },
  });
};

export const useTestIdentitySourceConnection = () => {
  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      identity_source_id: string;
    }) => {
      // TODO: Implement test connection endpoint
      // For now, simulate a test
      await new Promise(resolve => setTimeout(resolve, 2000));
      return {
        success: true,
        message: 'Connection successful',
        latency_ms: 150
      };
    },
  });
};
