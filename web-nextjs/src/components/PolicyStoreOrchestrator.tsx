"use client";

import React from "react";
import { usePolicyStorePanelStore } from "@/lib/stores/policy-store-panel-store";
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

  const handleCreate = async (name: string, description: string) => {
    try {
      await createMutation.mutateAsync({ name, description });
      closePanel();
    } catch (error) {
      console.error("Failed to create policy store:", error);
      alert("Failed to create policy store: " + (error as Error).message);
    }
  };

  const handleUpdate = async (name: string, description: string) => {
    if (!selectedStoreId) return;
    try {
      await updateMutation.mutateAsync({
        policyStoreId: selectedStoreId,
        name,
        description,
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
      title={
        content === "details"
          ? `Policy Store Details: ${selectedStoreId}`
          : content === "create"
            ? "Create Policy Store"
            : "Edit Policy Store"
      }
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
          />
        )}
        {content === "edit" && selectedStoreId && (
          <PolicyStoreFormPanel
            isLoading={updateMutation.isPending}
            onSubmit={handleUpdate}
            initialName=""
            initialDescription=""
          />
        )}
      </motion.div>
    </BottomSheet>
  );
};

export default PolicyStoreOrchestrator;
