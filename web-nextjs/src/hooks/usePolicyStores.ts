import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

// Local type definitions to avoid importing gRPC module in client code
interface PolicyStoreItem {
  policy_store_id: string;
  name: string;
  description?: string;
  status: string;
  version: string;
  author: string;
  tags: string[];
  identity_source_ids: string[];
  default_identity_source_id?: string;
  created_at: string;
  updated_at: string;
}

interface ListPolicyStoresResponse {
  policy_stores: PolicyStoreItem[];
  next_token?: string;
}

interface CreatePolicyStoreResponse {
  policy_store_id: string;
  created_at: string;
}

interface ListPolicyStoresParams {
  max_results?: number;
  next_token?: string;
}

interface ListPoliciesParams {
  policy_store_id: string;
  max_results?: number;
  next_token?: string;
}

export const usePolicyStores = (params?: ListPolicyStoresParams) => {
  return useQuery<ListPolicyStoresResponse>({
    queryKey: ["policy-stores", params],
    queryFn: async () => {
      const queryParams = new URLSearchParams();
      if (params?.max_results)
        queryParams.append("max_results", params.max_results.toString());
      if (params?.next_token)
        queryParams.append("next_token", params.next_token);

      const response = await fetch(
        `/api/policy-stores?${queryParams.toString()}`,
      );
      if (!response.ok) {
        throw new Error(
          `Failed to fetch policy stores: ${response.statusText}`,
        );
      }
      return response.json();
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const usePolicyStore = (policyStoreId: string) => {
  return useQuery({
    queryKey: ["policy-stores", policyStoreId],
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

  return useMutation<
    CreatePolicyStoreResponse,
    Error,
    { name: string; description?: string; tags?: string[]; user: string }
  >({
    mutationFn: async (data) => {
      const response = await fetch("/api/policy-stores", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to create policy store");
      }

      return response.json();
    },
    onSuccess: () => {
      // Invalidate and refetch policy stores list
      queryClient.invalidateQueries({ queryKey: ["policy-stores"] });
    },
  });
};

export const useDeletePolicyStore = () => {
  const queryClient = useQueryClient();

  return useMutation<void, Error, string>({
    mutationFn: async (policyStoreId: string) => {
      const response = await fetch(`/api/policy-stores/${policyStoreId}`, {
        method: "DELETE",
      });

      if (!response.ok) {
        try {
          // Try to parse JSON error response
          const error = await response.json();
          throw new Error(error.error || "Failed to delete policy store");
        } catch {
          // If JSON parsing fails, use status text
          throw new Error(
            `Failed to delete policy store: ${response.status} ${response.statusText}`,
          );
        }
      }
    },
    onSuccess: (_, policyStoreId) => {
      // Invalidate and refetch policy stores list
      queryClient.invalidateQueries({ queryKey: ["policy-stores"] });
      // Remove the specific policy store from cache
      queryClient.removeQueries({ queryKey: ["policy-stores", policyStoreId] });
    },
  });
};

export const useUpdatePolicyStore = () => {
  const queryClient = useQueryClient();

  return useMutation<
    void,
    Error,
    { policyStoreId: string; description: string; tags?: string[] }
  >({
    mutationFn: async ({ policyStoreId, description, tags }) => {
      const response = await fetch(`/api/policy-stores/${policyStoreId}`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ description, tags }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to update policy store");
      }

      return response.json();
    },
    onSuccess: (_, { policyStoreId }) => {
      // Invalidate and refetch policy stores list
      queryClient.invalidateQueries({ queryKey: ["policy-stores"] });
      // Update the specific policy store in cache
      queryClient.invalidateQueries({
        queryKey: ["policy-stores", policyStoreId],
      });
      // Invalidate tags cache to ensure consistency
      queryClient.invalidateQueries({
        queryKey: ["policy-store-tags", policyStoreId],
      });
    },
  });
};

// Hook to get policy count for a specific policy store
export const usePolicyCount = (policyStoreId: string) => {
  return useQuery({
    queryKey: ["policy-count", policyStoreId],
    queryFn: async () => {
      try {
        const response = await fetch(`/api/policy-stores/${policyStoreId}`);
        if (!response.ok) {
          return 0;
        }
        const data = await response.json();
        return data.metrics?.policies || 0;
      } catch {
        return 0;
      }
    },
    enabled: !!policyStoreId,
    staleTime: 30000, // 30 seconds
  });
};

// Hook to get complete metrics for a specific policy store
export const usePolicyStoreMetrics = (policyStoreId: string) => {
  return useQuery({
    queryKey: ["policy-store-metrics", policyStoreId],
    queryFn: async () => {
      if (!policyStoreId) return null;

      try {
        const [detailsResponse] = await Promise.all([
          fetch(`/api/policy-stores/${policyStoreId}`),
        ]);

        if (!detailsResponse.ok) {
          return {
            policies: 0,
            schemas: 0,
            loading: false,
          };
        }

        const details = await detailsResponse.json();
        return {
          policies: details.metrics?.policies || 0,
          schemas: details.metrics?.schemas || 0,
          loading: false,
        };
      } catch (error) {
        console.error("Failed to fetch metrics:", error);
        return {
          policies: 0,
          schemas: 0,
          loading: false,
        };
      }
    },
    enabled: !!policyStoreId,
    staleTime: 30000,
    refetchInterval: 30000, // Auto-refresh every 30 seconds
  });
};

// Hook to get all existing tags for autocomplete
export const useAllTags = () => {
  return useQuery({
    queryKey: ["all-tags"],
    queryFn: async () => {
      try {
        const response = await fetch("/api/policy-stores/tags");
        if (!response.ok) {
          return [];
        }
        const data = await response.json();
        return data.tags || [];
      } catch (error) {
        console.error("Failed to fetch tags:", error);
        return [];
      }
    },
    staleTime: 60000, // Cache for 1 minute
  });
};

// Hook to manage tags for a specific policy store
export const usePolicyStoreTags = (policyStoreId: string) => {
  const queryClient = useQueryClient();

  // Get current tags
  const { data: tags = [], isLoading } = useQuery({
    queryKey: ["policy-store-tags", policyStoreId],
    queryFn: async () => {
      if (!policyStoreId) return [];

      try {
        const response = await fetch(
          `/api/policy-stores/${policyStoreId}/tags`,
        );
        if (!response.ok) {
          return [];
        }
        const data = await response.json();
        return data.tags || [];
      } catch (error) {
        console.error("Failed to fetch tags:", error);
        return [];
      }
    },
    enabled: !!policyStoreId,
    staleTime: 30000,
  });

  // Add tag mutation
  const addTagMutation = useMutation({
    mutationFn: async (tag: string) => {
      const response = await fetch(`/api/policy-stores/${policyStoreId}/tags`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tag }),
      });
      if (!response.ok) {
        throw new Error("Failed to add tag");
      }
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["policy-store-tags", policyStoreId],
      });
      queryClient.invalidateQueries({ queryKey: ["all-tags"] });
    },
  });

  // Remove tag mutation
  const removeTagMutation = useMutation({
    mutationFn: async (tag: string) => {
      const response = await fetch(`/api/policy-stores/${policyStoreId}/tags`, {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tag }),
      });
      if (!response.ok) {
        throw new Error("Failed to remove tag");
      }
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["policy-store-tags", policyStoreId],
      });
      queryClient.invalidateQueries({ queryKey: ["all-tags"] });
    },
  });

  // Update tags mutation
  const updateTagsMutation = useMutation({
    mutationFn: async (newTags: string[]) => {
      const response = await fetch(`/api/policy-stores/${policyStoreId}/tags`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tags: newTags }),
      });
      if (!response.ok) {
        throw new Error("Failed to update tags");
      }
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["policy-store-tags", policyStoreId],
      });
      queryClient.invalidateQueries({ queryKey: ["all-tags"] });
    },
  });

  return {
    tags,
    isLoading,
    addTag: addTagMutation.mutateAsync,
    removeTag: removeTagMutation.mutateAsync,
    updateTags: updateTagsMutation.mutateAsync,
    isAddingTag: addTagMutation.isPending,
    isRemovingTag: removeTagMutation.isPending,
    isUpdatingTags: updateTagsMutation.isPending,
  };
};

// Hook to get snapshots for a policy store
export const usePolicyStoreSnapshots = (policyStoreId: string) => {
  return useQuery({
    queryKey: ["policy-store-snapshots", policyStoreId],
    queryFn: async () => {
      if (!policyStoreId) return { snapshots: [] };

      try {
        const response = await fetch(
          `/api/policy-stores/${policyStoreId}/snapshots`,
        );
        if (!response.ok) {
          return { snapshots: [] };
        }
        return response.json();
      } catch (error) {
        console.error("Failed to fetch snapshots:", error);
        return { snapshots: [] };
      }
    },
    enabled: !!policyStoreId,
    staleTime: 30000, // 30 seconds
  });
};

// Hook to get a specific snapshot
export const usePolicyStoreSnapshot = (
  policyStoreId: string,
  snapshotId: string,
) => {
  return useQuery({
    queryKey: ["policy-store-snapshot", policyStoreId, snapshotId],
    queryFn: async () => {
      if (!policyStoreId || !snapshotId) return null;

      try {
        const response = await fetch(
          `/api/policy-stores/${policyStoreId}/snapshots/${snapshotId}`,
        );
        if (!response.ok) {
          throw new Error("Failed to fetch snapshot");
        }
        return response.json();
      } catch (error) {
        console.error("Failed to fetch snapshot:", error);
        return null;
      }
    },
    enabled: !!policyStoreId && !!snapshotId,
    staleTime: 30000,
  });
};

// Hook to create a snapshot
export const useCreateSnapshot = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      policyStoreId,
      description,
    }: {
      policyStoreId: string;
      description?: string;
    }) => {
      const response = await fetch(
        `/api/policy-stores/${policyStoreId}/snapshots`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ description }),
        },
      );

      if (!response.ok) {
        throw new Error("Failed to create snapshot");
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["policy-store-snapshots", variables.policyStoreId],
      });
    },
  });
};

// Hook to delete a snapshot
export const useDeleteSnapshot = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      policyStoreId,
      snapshotId,
    }: {
      policyStoreId: string;
      snapshotId: string;
    }) => {
      const response = await fetch(
        `/api/policy-stores/${policyStoreId}/snapshots/${snapshotId}`,
        {
          method: "DELETE",
        },
      );

      if (!response.ok) {
        throw new Error("Failed to delete snapshot");
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["policy-store-snapshots", variables.policyStoreId],
      });
    },
  });
};

// Hook to rollback to a snapshot
export const useRollbackToSnapshot = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      policyStoreId,
      snapshotId,
      description,
    }: {
      policyStoreId: string;
      snapshotId: string;
      description?: string;
    }) => {
      const response = await fetch(
        `/api/policy-stores/${policyStoreId}/snapshots/${snapshotId}`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ description }),
        },
      );

      if (!response.ok) {
        throw new Error("Failed to rollback to snapshot");
      }

      return response.json();
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: ["policy-store-snapshots", variables.policyStoreId],
      });
      queryClient.invalidateQueries({ queryKey: ["policy-stores"] });
      queryClient.invalidateQueries({
        queryKey: ["policy-store", variables.policyStoreId],
      });
      queryClient.invalidateQueries({
        queryKey: ["policy-store-metrics", variables.policyStoreId],
      });
    },
  });
};
