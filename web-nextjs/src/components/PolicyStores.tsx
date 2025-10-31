'use client';

import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Shield, Plus, Search, Filter, Edit, Copy, Trash2, AlertCircle } from 'lucide-react';
import { usePolicyStores, useCreatePolicyStore, useDeletePolicyStore } from '@/hooks/usePolicyStores';

interface CreateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (description: string) => void;
  isLoading: boolean;
}

const CreatePolicyStoreModal: React.FC<CreateModalProps> = ({ isOpen, onClose, onCreate, isLoading }) => {
  const [description, setDescription] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onCreate(description);
    setDescription('');
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md">
        <h3 className="text-lg font-semibold mb-4">Create Policy Store</h3>
        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              rows={3}
              required
              placeholder="Enter policy store description"
            />
          </div>
          <div className="flex justify-end space-x-2">
            <Button type="button" variant="outline" onClick={onClose} disabled={isLoading}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading || !description}>
              {isLoading ? 'Creating...' : 'Create'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

const PolicyStores = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

  // Fetch policy stores using React Query
  const { data, isLoading, error, refetch } = usePolicyStores();

  // Create policy store mutation
  const createMutation = useCreatePolicyStore();
  const deleteMutation = useDeletePolicyStore();

  const handleCreatePolicyStore = async (description: string) => {
    try {
      await createMutation.mutateAsync({ description });
      setIsCreateModalOpen(false);
    } catch (error) {
      console.error('Failed to create policy store:', error);
    }
  };

  const handleDeletePolicyStore = async (policyStoreId: string) => {
    if (window.confirm('Are you sure you want to delete this policy store?')) {
      try {
        await deleteMutation.mutateAsync(policyStoreId);
      } catch (error) {
        console.error('Failed to delete policy store:', error);
      }
    }
  };

  // Filter stores based on search term
  const filteredStores = (data?.policy_stores || []).filter((store) =>
    (store.description || '').toLowerCase().includes(searchTerm.toLowerCase()) ||
    store.policy_store_id.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Loading skeleton
  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="h-8 w-48 bg-gray-200 rounded animate-pulse mb-2"></div>
            <div className="h-4 w-96 bg-gray-200 rounded animate-pulse"></div>
          </div>
          <div className="h-10 w-44 bg-gray-200 rounded animate-pulse"></div>
        </div>

        <Card>
          <CardContent className="pt-6">
            <div className="h-10 w-full bg-gray-200 rounded animate-pulse"></div>
          </CardContent>
        </Card>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {[...Array(6)].map((_, i) => (
            <Card key={i}>
              <CardHeader>
                <div className="h-6 w-32 bg-gray-200 rounded animate-pulse mb-2"></div>
                <div className="h-4 w-48 bg-gray-200 rounded animate-pulse"></div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="h-4 w-full bg-gray-200 rounded animate-pulse"></div>
                  <div className="h-4 w-full bg-gray-200 rounded animate-pulse"></div>
                  <div className="h-10 w-full bg-gray-200 rounded animate-pulse"></div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <Card className="max-w-md">
          <CardContent className="pt-6">
            <div className="text-center">
              <AlertCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                Failed to load policy stores
              </h3>
              <p className="text-gray-600 mb-4">
                {error.message || 'An error occurred while fetching policy stores'}
              </p>
              <Button onClick={() => refetch()}>
                Retry
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Policy Stores</h2>
          <p className="text-gray-600">Manage your policy stores and their configurations</p>
        </div>
        <Button onClick={() => setIsCreateModalOpen(true)} className="flex items-center space-x-2">
          <Plus className="w-4 h-4" />
          <span>Create Policy Store</span>
        </Button>
      </div>

      {/* Search and Filter */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex items-center space-x-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
              <input
                type="text"
                placeholder="Search policy stores by ID or description..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <Button variant="outline" className="flex items-center space-x-2">
              <Filter className="w-4 h-4" />
              <span>Filter</span>
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Policy Stores Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredStores.map((store) => (
          <Card key={store.policy_store_id} className="hover:shadow-md transition-shadow">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <Shield className="w-5 h-5 text-blue-600" />
                  <Badge variant="outline">{store.policy_store_id}</Badge>
                </div>
                <div className="flex items-center space-x-1">
                  <Button variant="ghost" size="sm">
                    <Edit className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm">
                    <Copy className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDeletePolicyStore(store.policy_store_id)}
                    disabled={deleteMutation.isPending}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
              <CardTitle className="text-lg">{store.description || 'No description'}</CardTitle>
              <CardDescription>
                Created: {new Date(store.created_at).toLocaleDateString()}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600">Policies</span>
                  <Badge variant="secondary">-</Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600">Schemas</span>
                  <Badge variant="secondary">-</Badge>
                </div>
                <div className="pt-3">
                  <Button variant="outline" className="w-full">
                    View Details
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Empty State */}
      {filteredStores.length === 0 && (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center py-8">
              <Shield className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                {data?.policy_stores?.length === 0 ? 'No policy stores found' : 'No results'}
              </h3>
              <p className="text-gray-600 mb-4">
                {searchTerm
                  ? 'Try adjusting your search terms'
                  : 'Get started by creating your first policy store'}
              </p>
              {!searchTerm && (
                <Button onClick={() => setIsCreateModalOpen(true)}>
                  Create Policy Store
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Create Modal */}
      <CreatePolicyStoreModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onCreate={handleCreatePolicyStore}
        isLoading={createMutation.isPending}
      />
    </div>
  );
};

export default PolicyStores;
