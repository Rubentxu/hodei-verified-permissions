"use client";

import React from "react";
import { usePolicyStorePanelStore } from "@/lib/stores/policy-store-panel-store";
import { usePolicyStore } from "@/hooks/usePolicyStores";
import BottomSheet from "@/components/ui/BottomSheet";
import PolicyStoreDetailsPanel from "@/components/PolicyStoreDetailsPanel";
import PolicyStoreFormPanel from "@/components/PolicyStoreFormPanel";
import {
  useCreatePolicyStore,
  useUpdatePolicyStore,
} from "@/hooks/usePolicyStores";
import { motion } from "framer-motion";

const PolicyStoreOrchestrator = () => {
  const { isOpen, content, selectedStoreId, closePanel, isTransitioning } =
    usePolicyStorePanelStore();
  const createMutation = useCreatePolicyStore();
  const updateMutation = useUpdatePolicyStore();
  const { data: policyStore } = usePolicyStore(selectedStoreId || "");

  // Helper function to get display name
  const getDisplayName = (content: string, storeId?: string, store?: any) => {
    if (content === "create") return "Create Policy Store";
    if (content === "edit")
      return `Edit Policy Store${store?.name ? `: ${store.name}` : ""}`;
    if (content === "details")
      return `Policy Store Details${store?.name ? `: ${store.name}` : ""}`;
    return "";
  };

  const handleCreate = async (
    name: string,
    description: string,
    tags: string[],
    user: string,
  ) => {
    try {
      await createMutation.mutateAsync({ name, description, tags, user });
      closePanel();
    } catch (error) {
      console.error("Failed to create policy store:", error);
      alert("Failed to create policy store: " + (error as Error).message);
    }
  };

  const handleUpdate = async (
    name: string,
    description: string,
    tags: string[] = [],
    user: string = "",
    status: string = "active",
  ) => {
    if (!selectedStoreId) return;
    try {
      await updateMutation.mutateAsync({
        policyStoreId: selectedStoreId,
        name,
        description,
        tags,
        status,
      });
      closePanel();
    } catch (error) {
      console.error("Failed to update policy store:", error);
      alert("Failed to update policy store: " + (error as Error).message);
    }
  };

  return (
    <BottomSheet
      isOpen={isOpen}
      onClose={closePanel}
      title={getDisplayName(content, selectedStoreId, policyStore)}
      marginLeft={256} // Adjust this value to match the width of the side navigation
    >
      <motion.div
        key={selectedStoreId}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
      >
        {content === "details" && selectedStoreId && (
          <PolicyStoreDetailsPanel policyStoreId={selectedStoreId} />
        )}
        {content === "create" && (
          <PolicyStoreFormPanel
            isLoading={createMutation.isPending}
            onSubmit={handleCreate}
            showTagsAndUser={true}
          />
        )}
        {content === "edit" && selectedStoreId && (
          <PolicyStoreFormPanel
            isLoading={updateMutation.isPending}
            onSubmit={handleUpdate}
            initialName={policyStore?.name || ""}
            initialDescription={policyStore?.description || ""}
            initialTags={policyStore?.tags || []}
            initialUser={policyStore?.author || ""}
            initialStatus={policyStore?.status || "active"}
            showTagsAndUser={true}
          />
        )}
      </motion.div>
    </BottomSheet>
  );
};

export default PolicyStoreOrchestrator;
