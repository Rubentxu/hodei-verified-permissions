import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

interface SchemaData {
  policy_store_id: string;
  schema: string;
  created_at?: string;
  updated_at?: string;
}

export const useSchema = (policyStoreId: string) => {
  return useQuery<SchemaData>({
    queryKey: ['schemas', policyStoreId],
    queryFn: async () => {
      const response = await fetch(`/api/schemas/${policyStoreId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch schema: ${response.statusText}`);
      }
      return response.json();
    },
    enabled: !!policyStoreId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const useUpdateSchema = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: {
      policy_store_id: string;
      schema: string;
    }) => {
      const response = await fetch(`/api/schemas/${data.policy_store_id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to update schema');
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch schema
      queryClient.invalidateQueries({
        queryKey: ['schemas', variables.policy_store_id]
      });
    },
  });
};

export const useValidateSchema = (schema: string) => {
  return useQuery({
    queryKey: ['validate-schema', schema],
    queryFn: async () => {
      try {
        // Validate JSON format
        const parsed = JSON.parse(schema);
        return { valid: true, data: parsed };
      } catch (error) {
        return {
          valid: false,
          error: error instanceof Error ? error.message : 'Invalid JSON'
        };
      }
    },
    enabled: !!schema,
    staleTime: 0, // Always revalidate
  });
};
