'use client';

import React, { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Shield, Plus, Search, Filter, Edit, Copy, Trash2, AlertCircle, Eye, FileText, Layers, Clock, User, Calendar, Tag, History } from 'lucide-react';
import { usePolicyStores, useCreatePolicyStore, useDeletePolicyStore, useUpdatePolicyStore, usePolicyStore, usePolicyCount, usePolicyStoreMetrics, usePolicyStoreAuditLog, useAllTags, usePolicyStoreSnapshots, useCreateSnapshot, useRollbackToSnapshot, useDeleteSnapshot } from '@/hooks/usePolicyStores';
import TagManager from './TagManager';

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

  const handleClose = () => {
    setDescription('');
    onClose();
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
            <Button type="button" variant="outline" onClick={handleClose} disabled={isLoading}>
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

interface EditModalProps {
  isOpen: boolean;
  onClose: () => void;
  onUpdate: (policyStoreId: string, description: string) => void;
  isLoading: boolean;
  initialDescription: string;
  policyStoreId: string;
}

const EditPolicyStoreModal: React.FC<EditModalProps> = ({
  isOpen,
  onClose,
  onUpdate,
  isLoading,
  initialDescription,
  policyStoreId
}) => {
  const [description, setDescription] = useState(initialDescription);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onUpdate(policyStoreId, description);
  };

  const handleClose = () => {
    setDescription(initialDescription);
    onClose();
  };

  React.useEffect(() => {
    if (isOpen) {
      setDescription(initialDescription);
    }
  }, [isOpen, initialDescription]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-md">
        <h3 className="text-lg font-semibold mb-4">Edit Policy Store</h3>
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
            <Button type="button" variant="outline" onClick={handleClose} disabled={isLoading}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading || !description}>
              {isLoading ? 'Updating...' : 'Update'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

interface PolicyStoreDetails {
  policy_store_id: string;
  name?: string;
  description?: string;
  created_at: string;
  updated_at: string;
  metrics: {
    policies: number;
    schemas: number;
    lastModified: string;
    status: string;
    version: string;
    author: string;
    tags: string[];
  };
}

// Badge component to display policy count with loading state
const PolicyCountBadge: React.FC<{ policyStoreId: string }> = ({ policyStoreId }) => {
  const { data, isLoading } = usePolicyStoreMetrics(policyStoreId);

  if (isLoading) {
    return <span className="animate-pulse">...</span>;
  }

  return <span>{data?.policies || 0}</span>;
};

// Badge component to display schema count with loading state
const SchemaCountBadge: React.FC<{ policyStoreId: string }> = ({ policyStoreId }) => {
  const { data, isLoading } = usePolicyStoreMetrics(policyStoreId);

  if (isLoading) {
    return <span className="animate-pulse">...</span>;
  }

  return <span>{data?.schemas || 0}</span>;
};

// Audit Log Panel Component
interface AuditLogPanelProps {
  policyStoreId: string;
}

const AuditLogPanel: React.FC<AuditLogPanelProps> = ({ policyStoreId }) => {
  const { data: auditLogs, isLoading } = usePolicyStoreAuditLog(policyStoreId);

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[1, 2, 3].map((i) => (
          <Card key={i}>
            <CardContent className="pt-6">
              <div className="animate-pulse space-y-2">
                <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                <div className="h-3 bg-gray-200 rounded w-1/2"></div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-4">
        <h4 className="text-lg font-semibold">Activity History</h4>
        <Badge variant="outline">
          {auditLogs?.length || 0} events
        </Badge>
      </div>

      {auditLogs && auditLogs.length > 0 ? (
        <div className="space-y-3">
          {auditLogs.map((log) => (
            <Card key={log.id}>
              <CardContent className="pt-6">
                <div className="flex items-start justify-between">
                  <div className="flex items-start space-x-3">
                    <div className={`mt-1 p-2 rounded-full ${
                      log.action === 'CREATE' ? 'bg-green-100' :
                      log.action === 'UPDATE' ? 'bg-blue-100' :
                      'bg-red-100'
                    }`}>
                      <History className={`w-4 h-4 ${
                        log.action === 'CREATE' ? 'text-green-600' :
                        log.action === 'UPDATE' ? 'text-blue-600' :
                        'text-red-600'
                      }`} />
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center space-x-2">
                        <Badge variant={
                          log.action === 'CREATE' ? 'default' :
                          log.action === 'UPDATE' ? 'secondary' :
                          'destructive'
                        }>
                          {log.action}
                        </Badge>
                        <span className="text-sm text-gray-500">by</span>
                        <span className="text-sm font-medium">{log.user_id}</span>
                      </div>
                      {log.changes && (
                        <p className="text-sm text-gray-600 mt-1">
                          Changes: {log.changes}
                        </p>
                      )}
                      <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                        <span className="flex items-center space-x-1">
                          <Clock className="w-3 h-3" />
                          <span>{new Date(log.timestamp).toLocaleString()}</span>
                        </span>
                        {log.ip_address && (
                          <span>IP: {log.ip_address}</span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center py-8">
              <History className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No audit history</h3>
              <p className="text-gray-600">
                Activity for this policy store will appear here
              </p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

// Tags Panel Component
interface TagsPanelProps {
  policyStoreId: string;
}

const TagsPanel: React.FC<TagsPanelProps> = ({ policyStoreId }) => {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-4">
        <h4 className="text-lg font-semibold">Manage Tags</h4>
        <Badge variant="outline">
          Tag Management
        </Badge>
      </div>

      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h5 className="font-medium text-blue-900 mb-2">📝 Tag Management</h5>
        <p className="text-sm text-blue-700 mb-4">
          Organize your policy stores with custom tags. Add tags to categorize and filter your stores easily.
        </p>
        <TagManager policyStoreId={policyStoreId} />
      </div>

      <div className="mt-6">
        <h5 className="text-sm font-medium text-gray-700 mb-2">💡 Tips</h5>
        <ul className="text-sm text-gray-600 space-y-1 list-disc list-inside">
          <li>Use descriptive tags like "production", "testing", or "frontend"</li>
          <li>Click on existing tags to remove them</li>
          <li>Start typing to see autocomplete suggestions</li>
          <li>Tags are case-sensitive but can be searched case-insensitively</li>
        </ul>
      </div>
    </div>
  );
};

// Version History Panel Component
interface VersionHistoryPanelProps {
  policyStoreId: string;
}

const VersionHistoryPanel: React.FC<VersionHistoryPanelProps> = ({ policyStoreId }) => {
  const { data: snapshotsData, isLoading, refetch } = usePolicyStoreSnapshots(policyStoreId);
  const createSnapshotMutation = useCreateSnapshot();
  const rollbackMutation = useRollbackToSnapshot();
  const deleteSnapshotMutation = useDeleteSnapshot();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newSnapshotDescription, setNewSnapshotDescription] = useState('');

  const handleCreateSnapshot = async () => {
    try {
      await createSnapshotMutation.mutateAsync({
        policyStoreId,
        description: newSnapshotDescription || undefined,
      });
      setShowCreateModal(false);
      setNewSnapshotDescription('');
    } catch (error) {
      console.error('Failed to create snapshot:', error);
      alert('Failed to create snapshot: ' + (error as Error).message);
    }
  };

  const handleRollback = async (snapshotId: string) => {
    if (window.confirm('Are you sure you want to rollback to this snapshot? This will replace all current policies and schema.')) {
      try {
        await rollbackMutation.mutateAsync({
          policyStoreId,
          snapshotId,
          description: `Rollback to snapshot ${snapshotId}`,
        });
        alert('Successfully rolled back to snapshot');
      } catch (error) {
        console.error('Failed to rollback:', error);
        alert('Failed to rollback: ' + (error as Error).message);
      }
    }
  };

  const handleDeleteSnapshot = async (snapshotId: string) => {
    if (window.confirm('Are you sure you want to delete this snapshot?')) {
      try {
        await deleteSnapshotMutation.mutateAsync({
          policyStoreId,
          snapshotId,
        });
      } catch (error) {
        console.error('Failed to delete snapshot:', error);
        alert('Failed to delete snapshot: ' + (error as Error).message);
      }
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-4">
        <h4 className="text-lg font-semibold">Version History</h4>
        <Button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center space-x-2"
        >
          <History className="w-4 h-4 mr-1" />
          <span>Create Snapshot</span>
        </Button>
      </div>

      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4">
        <h5 className="font-medium text-blue-900 mb-2">📸 Snapshot Management</h5>
        <p className="text-sm text-blue-700">
          Create point-in-time snapshots of your policy store. You can rollback to any snapshot to restore previous state.
        </p>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardContent className="pt-6">
                <div className="animate-pulse space-y-2">
                  <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                  <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : snapshotsData?.snapshots && snapshotsData.snapshots.length > 0 ? (
        <div className="space-y-3">
          {snapshotsData.snapshots.map((snapshot: any) => (
            <Card key={snapshot.snapshot_id}>
              <CardContent className="pt-6">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-2">
                      <Badge variant="outline">{snapshot.snapshot_id}</Badge>
                      {snapshot.description && (
                        <span className="text-sm text-gray-600">{snapshot.description}</span>
                      )}
                    </div>
                    <div className="grid grid-cols-3 gap-4 text-sm text-gray-600 mb-3">
                      <div>
                        <span className="font-medium">Policies:</span> {snapshot.policy_count}
                      </div>
                      <div>
                        <span className="font-medium">Schema:</span> {snapshot.has_schema ? 'Yes' : 'No'}
                      </div>
                      <div>
                        <span className="font-medium">Size:</span> {(snapshot.size_bytes / 1024).toFixed(2)} KB
                      </div>
                    </div>
                    <div className="text-xs text-gray-500">
                      Created: {new Date(snapshot.created_at).toLocaleString()}
                    </div>
                  </div>
                  <div className="flex items-center space-x-2 ml-4">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleRollback(snapshot.snapshot_id)}
                      disabled={rollbackMutation.isPending}
                      className="text-orange-600 hover:text-orange-700"
                    >
                      <History className="w-4 h-4 mr-1" />
                      Rollback
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDeleteSnapshot(snapshot.snapshot_id)}
                      disabled={deleteSnapshotMutation.isPending}
                      className="text-red-600 hover:text-red-700"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center py-8">
              <History className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">No snapshots yet</h3>
              <p className="text-gray-600 mb-4">
                Create your first snapshot to start tracking version history
              </p>
              <Button onClick={() => setShowCreateModal(true)}>
                Create Snapshot
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Create Snapshot Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold mb-4">Create Snapshot</h3>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                handleCreateSnapshot();
              }}
            >
              <div className="mb-4">
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Description (optional)
                </label>
                <input
                  type="text"
                  value={newSnapshotDescription}
                  onChange={(e) => setNewSnapshotDescription(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="e.g., Before migration, Pre-production setup"
                />
              </div>
              <div className="flex justify-end space-x-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setShowCreateModal(false);
                    setNewSnapshotDescription('');
                  }}
                  disabled={createSnapshotMutation.isPending}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={createSnapshotMutation.isPending}
                >
                  {createSnapshotMutation.isPending ? 'Creating...' : 'Create Snapshot'}
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

const PolicyStores = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isDetailsModalOpen, setIsDetailsModalOpen] = useState(false);
  const [editingStore, setEditingStore] = useState<{ id: string; description: string } | null>(null);
  const [detailsStore, setDetailsStore] = useState<PolicyStoreDetails | null>(null);
  const [detailsModalTab, setDetailsModalTab] = useState<'overview' | 'audit' | 'tags' | 'versions'>('overview');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [showFilters, setShowFilters] = useState(false);
  const [statusFilter, setStatusFilter] = useState<string>('all');

  // Fetch policy stores using React Query
  const { data, isLoading, error, refetch } = usePolicyStores();

  // Fetch all available tags for filtering
  const { data: allAvailableTags = [] } = useAllTags();

  // Create policy store mutation
  const createMutation = useCreatePolicyStore();
  const deleteMutation = useDeletePolicyStore();
  const updateMutation = useUpdatePolicyStore();

  const handleCreatePolicyStore = async (description: string) => {
    try {
      await createMutation.mutateAsync({ description });
      setIsCreateModalOpen(false);
    } catch (error) {
      console.error('Failed to create policy store:', error);
      alert('Failed to create policy store: ' + (error as Error).message);
    }
  };

  const handleDeletePolicyStore = async (policyStoreId: string) => {
    if (window.confirm('Are you sure you want to delete this policy store?')) {
      try {
        await deleteMutation.mutateAsync(policyStoreId);
      } catch (error) {
        console.error('Failed to delete policy store:', error);
        const errorMessage = (error as Error).message;
        if (errorMessage.includes('not yet implemented')) {
          alert('Delete functionality is not yet implemented in the backend.\n\n' + errorMessage);
        } else {
          alert('Failed to delete policy store: ' + errorMessage);
        }
      }
    }
  };

  const handleEditPolicyStore = (store: { policy_store_id: string; description?: string }) => {
    setEditingStore({
      id: store.policy_store_id,
      description: store.description || ''
    });
    setIsEditModalOpen(true);
  };

  const handleUpdatePolicyStore = async (policyStoreId: string, description: string) => {
    try {
      await updateMutation.mutateAsync({ policyStoreId, description });
      setIsEditModalOpen(false);
      setEditingStore(null);
    } catch (error) {
      console.error('Failed to update policy store:', error);
      const errorMessage = (error as Error).message;
      if (errorMessage.includes('not yet implemented')) {
        alert('Update functionality is not yet implemented in the backend.\n\n' + errorMessage);
      } else {
        alert('Failed to update policy store: ' + errorMessage);
      }
    }
  };

  const handleViewDetails = async (store: { policy_store_id: string }) => {
    try {
      const response = await fetch(`/api/policy-stores/${store.policy_store_id}`);
      if (!response.ok) {
        throw new Error('Failed to fetch policy store details');
      }
      const details = await response.json();
      setDetailsStore(details);
      setDetailsModalTab('overview'); // Reset to overview tab
      setIsDetailsModalOpen(true);
    } catch (error) {
      console.error('Failed to fetch policy store details:', error);
      alert('Failed to load policy store details: ' + (error as Error).message);
    }
  };

  // Enhanced filtering logic
  const filteredStores = (data?.policy_stores || []).filter((store) => {
    // Search term filter (search in description or ID)
    const matchesSearch =
      (store.description || '').toLowerCase().includes(searchTerm.toLowerCase()) ||
      store.policy_store_id.toLowerCase().includes(searchTerm.toLowerCase());

    // TODO: Add status filtering when status is available in API
    // For now, all stores are considered 'active'
    const matchesStatus = statusFilter === 'all' || statusFilter === 'active';

    // TODO: Add tag filtering when tags are available in API
    // This will be implemented once the backend provides tag data in list response
    const matchesTags = selectedTags.length === 0 || true; // Placeholder until backend support

    return matchesSearch && matchesStatus && matchesTags;
  });

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
            <Button
              variant="outline"
              className="flex items-center space-x-2"
              onClick={() => setShowFilters(!showFilters)}
            >
              <Filter className="w-4 h-4" />
              <span>Filter</span>
              {(selectedTags.length > 0 || statusFilter !== 'all') && (
                <Badge variant="secondary" className="ml-1">
                  {[selectedTags.length, statusFilter !== 'all' ? 1 : 0].reduce((a, b) => a + b, 0)}
                </Badge>
              )}
            </Button>
          </div>

          {/* Advanced Filters Panel */}
          {showFilters && (
            <div className="mt-4 pt-4 border-t space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Status Filter */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">Status</label>
                  <select
                    value={statusFilter}
                    onChange={(e) => setStatusFilter(e.target.value)}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  >
                    <option value="all">All Statuses</option>
                    <option value="active">Active</option>
                    <option value="inactive">Inactive</option>
                  </select>
                </div>

                {/* Tags Filter */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">Tags</label>
                  <div className="flex flex-wrap gap-2 max-h-32 overflow-y-auto p-2 border border-gray-200 rounded-md">
                    {allAvailableTags.map((tag, index) => (
                      <button
                        key={index}
                        onClick={() => {
                          if (selectedTags.includes(tag)) {
                            setSelectedTags(selectedTags.filter(t => t !== tag));
                          } else {
                            setSelectedTags([...selectedTags, tag]);
                          }
                        }}
                        className={`px-3 py-1 rounded-full text-sm ${
                          selectedTags.includes(tag)
                            ? 'bg-blue-100 text-blue-800 border border-blue-300'
                            : 'bg-gray-100 text-gray-700 border border-gray-300 hover:bg-gray-200'
                        }`}
                      >
                        {tag}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              {/* Clear Filters */}
              {(selectedTags.length > 0 || statusFilter !== 'all') && (
                <div className="flex justify-end">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      setSelectedTags([]);
                      setStatusFilter('all');
                    }}
                  >
                    Clear Filters
                  </Button>
                </div>
              )}
            </div>
          )}
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
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleEditPolicyStore(store)}
                    title="Edit policy store"
                  >
                    <Edit className="w-4 h-4" />
                  </Button>
                  <Button variant="ghost" size="sm" title="Copy policy store ID">
                    <Copy className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDeletePolicyStore(store.policy_store_id)}
                    disabled={deleteMutation.isPending}
                    title="Delete policy store"
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
                  <Badge variant="secondary">
                    <FileText className="w-3 h-3 mr-1" />
                    {store.policy_store_id ? (
                      <PolicyCountBadge policyStoreId={store.policy_store_id} />
                    ) : (
                      '0'
                    )}
                  </Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600">Schemas</span>
                  <Badge variant="secondary">
                    <Layers className="w-3 h-3 mr-1" />
                    {store.policy_store_id ? (
                      <SchemaCountBadge policyStoreId={store.policy_store_id} />
                    ) : (
                      '0'
                    )}
                  </Badge>
                </div>
                <div className="pt-3">
                  <Button
                    variant="outline"
                    className="w-full"
                    onClick={() => handleViewDetails(store)}
                  >
                    <Eye className="w-4 h-4 mr-2" />
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

      {/* Edit Modal */}
      {editingStore && (
        <EditPolicyStoreModal
          isOpen={isEditModalOpen}
          onClose={() => {
            setIsEditModalOpen(false);
            setEditingStore(null);
          }}
          onUpdate={handleUpdatePolicyStore}
          isLoading={updateMutation.isPending}
          initialDescription={editingStore.description}
          policyStoreId={editingStore.id}
        />
      )}

      {/* View Details Modal */}
      {detailsStore && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-4xl max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center space-x-3">
                <Shield className="w-6 h-6 text-blue-600" />
                <div>
                  <h3 className="text-xl font-semibold">Policy Store Details</h3>
                  <p className="text-sm text-gray-500">{detailsStore.policy_store_id}</p>
                </div>
              </div>
              <Button variant="ghost" onClick={() => setIsDetailsModalOpen(false)}>
                ✕
              </Button>
            </div>

            {/* Tabs */}
            <div className="flex space-x-1 mb-6 border-b">
              <button
                onClick={() => setDetailsModalTab('overview')}
                className={`px-4 py-2 flex items-center space-x-2 ${
                  detailsModalTab === 'overview'
                    ? 'border-b-2 border-blue-600 text-blue-600'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                <FileText className="w-4 h-4" />
                <span>Overview</span>
              </button>
              <button
                onClick={() => setDetailsModalTab('audit')}
                className={`px-4 py-2 flex items-center space-x-2 ${
                  detailsModalTab === 'audit'
                    ? 'border-b-2 border-blue-600 text-blue-600'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                <History className="w-4 h-4" />
                <span>Audit Log</span>
                <Badge variant="secondary" className="ml-2">
                  <Eye className="w-3 h-3 mr-1" />
                  Live
                </Badge>
              </button>
              <button
                onClick={() => setDetailsModalTab('tags')}
                className={`px-4 py-2 flex items-center space-x-2 ${
                  detailsModalTab === 'tags'
                    ? 'border-b-2 border-blue-600 text-blue-600'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                <Tag className="w-4 h-4" />
                <span>Tags</span>
              </button>
              <button
                onClick={() => setDetailsModalTab('versions')}
                className={`px-4 py-2 flex items-center space-x-2 ${
                  detailsModalTab === 'versions'
                    ? 'border-b-2 border-blue-600 text-blue-600'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                <History className="w-4 h-4" />
                <span>Version History</span>
                <Badge variant="secondary" className="ml-2">
                  Snapshots
                </Badge>
              </button>
            </div>

            {/* Tab Content */}
            {detailsModalTab === 'overview' ? (
              <>
                {/* Metrics Grid */}
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
                  <Card>
                    <CardContent className="pt-6">
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm text-gray-600">Policies</p>
                          <p className="text-2xl font-bold">{detailsStore.metrics.policies}</p>
                        </div>
                        <FileText className="w-8 h-8 text-blue-500" />
                      </div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardContent className="pt-6">
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm text-gray-600">Schemas</p>
                          <p className="text-2xl font-bold">{detailsStore.metrics.schemas}</p>
                        </div>
                        <Layers className="w-8 h-8 text-green-500" />
                      </div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardContent className="pt-6">
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm text-gray-600">Status</p>
                          <Badge variant={detailsStore.metrics.status === 'active' ? 'default' : 'secondary'}>
                            {detailsStore.metrics.status}
                          </Badge>
                        </div>
                        <Shield className="w-8 h-8 text-purple-500" />
                      </div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardContent className="pt-6">
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm text-gray-600">Version</p>
                          <p className="text-2xl font-bold">{detailsStore.metrics.version}</p>
                        </div>
                        <Tag className="w-8 h-8 text-orange-500" />
                      </div>
                    </CardContent>
                  </Card>
                </div>

                {/* Description */}
                <div className="mb-6">
                  <h4 className="text-lg font-semibold mb-2">Description</h4>
                  <p className="text-gray-700 bg-gray-50 p-4 rounded-lg">
                    {detailsStore.description || 'No description provided'}
                  </p>
                </div>

                {/* Metadata */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
                  <div>
                    <h4 className="text-lg font-semibold mb-3">Metadata</h4>
                    <div className="space-y-3">
                      <div className="flex items-center space-x-2">
                        <User className="w-4 h-4 text-gray-500" />
                        <span className="text-sm text-gray-600">Author:</span>
                        <span className="text-sm font-medium">{detailsStore.metrics.author}</span>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Calendar className="w-4 h-4 text-gray-500" />
                        <span className="text-sm text-gray-600">Created:</span>
                        <span className="text-sm font-medium">
                          {new Date(detailsStore.created_at).toLocaleString()}
                        </span>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Clock className="w-4 h-4 text-gray-500" />
                        <span className="text-sm text-gray-600">Last Modified:</span>
                        <span className="text-sm font-medium">
                          {new Date(detailsStore.metrics.lastModified).toLocaleString()}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h4 className="text-lg font-semibold mb-3">Tags</h4>
                    <div className="flex flex-wrap gap-2">
                      {detailsStore.metrics.tags.length > 0 ? (
                        detailsStore.metrics.tags.map((tag, index) => (
                          <Badge key={index} variant="outline">{tag}</Badge>
                        ))
                      ) : (
                        <span className="text-sm text-gray-500">No tags</span>
                      )}
                    </div>
                  </div>
                </div>
              </>
            ) : detailsModalTab === 'audit' ? (
              <AuditLogPanel policyStoreId={detailsStore.policy_store_id} />
            ) : detailsModalTab === 'tags' ? (
              <TagsPanel policyStoreId={detailsStore.policy_store_id} />
            ) : (
              <VersionHistoryPanel policyStoreId={detailsStore.policy_store_id} />
            )}

            {/* Actions */}
            <div className="flex justify-end space-x-2 pt-4 border-t">
              <Button
                variant="outline"
                onClick={() => {
                  handleEditPolicyStore({
                    policy_store_id: detailsStore.policy_store_id,
                    description: detailsStore.description
                  });
                  setIsDetailsModalOpen(false);
                }}
              >
                <Edit className="w-4 h-4 mr-2" />
                Edit
              </Button>
              <Button onClick={() => setIsDetailsModalOpen(false)}>
                Close
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default PolicyStores;
