/**
 * PolicyStoresPage - HU 14.1 & 14.2
 * Ver lista de Policy Stores y crear nuevos
 */

import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus } from 'lucide-react';
import {
  usePolicyStores,
  useCreatePolicyStore,
  useDeletePolicyStore,
} from '../../../api';
import { Button, Alert } from '../../../components';
import { PolicyStoresList, CreatePolicyStoreForm } from '../components';
import { useUIStore } from '../../../store';

export const PolicyStoresPage: React.FC = () => {
  const navigate = useNavigate();
  const { addNotification } = useUIStore();
  const [showCreateForm, setShowCreateForm] = useState(false);

  // Fetch policy stores
  const { data, isLoading, error } = usePolicyStores();
  const createMutation = useCreatePolicyStore();
  const deleteMutation = useDeletePolicyStore();

  const handleCreateStore = async (description: string) => {
    try {
      await createMutation.mutateAsync(description);
      addNotification('Policy Store created successfully', 'success');
      setShowCreateForm(false);
    } catch (err) {
      addNotification(
        err instanceof Error ? err.message : 'Failed to create policy store',
        'error'
      );
    }
  };

  const handleDeleteStore = async (id: string) => {
    if (confirm('Are you sure you want to delete this policy store?')) {
      try {
        await deleteMutation.mutateAsync(id);
        addNotification('Policy Store deleted successfully', 'success');
      } catch (err) {
        addNotification(
          err instanceof Error ? err.message : 'Failed to delete policy store',
          'error'
        );
      }
    }
  };

  const handleSelectStore = (storeId: string) => {
    navigate(`/policy-stores/${storeId}`);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Policy Stores</h1>
          <p className="text-gray-600 mt-1">
            Manage authorization policy stores for your applications
          </p>
        </div>
        <Button
          variant="primary"
          onClick={() => setShowCreateForm(!showCreateForm)}
        >
          <Plus className="w-4 h-4 mr-2" />
          New Policy Store
        </Button>
      </div>

      {/* Create Form */}
      {showCreateForm && (
        <CreatePolicyStoreForm
          onSubmit={handleCreateStore}
          isLoading={createMutation.isPending}
          error={createMutation.error?.message}
        />
      )}

      {/* Policy Stores List */}
      <PolicyStoresList
        stores={data?.policyStores || []}
        isLoading={isLoading}
        error={error?.message}
        onSelectStore={(store) => handleSelectStore(store.policyStoreId)}
        onDeleteStore={handleDeleteStore}
      />
    </div>
  );
};

export default PolicyStoresPage;
