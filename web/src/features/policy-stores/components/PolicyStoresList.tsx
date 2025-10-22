/**
 * PolicyStoresList Component - Display list of policy stores
 */

import React from 'react';
import { PolicyStore } from '../../../types';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  LoadingSpinner,
  Alert,
} from '../../../components';
import { formatRelativeTime } from '../../../utils';
import { ChevronRight, Trash2 } from 'lucide-react';

export interface PolicyStoresListProps {
  stores: PolicyStore[];
  isLoading?: boolean;
  error?: string;
  onSelectStore?: (store: PolicyStore) => void;
  onDeleteStore?: (id: string) => void;
}

export const PolicyStoresList: React.FC<PolicyStoresListProps> = ({
  stores,
  isLoading,
  error,
  onSelectStore,
  onDeleteStore,
}) => {
  if (isLoading) {
    return (
      <div className="flex justify-center py-12">
        <LoadingSpinner message="Loading policy stores..." />
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        type="error"
        title="Error loading policy stores"
        message={error}
      />
    );
  }

  if (stores.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-gray-500">No policy stores found</p>
          <p className="text-sm text-gray-400 mt-2">
            Create your first policy store to get started
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-3">
      {stores.map((store) => (
        <Card
          key={store.policyStoreId}
          className="hover:shadow-md transition-shadow cursor-pointer"
          onClick={() => onSelectStore?.(store)}
        >
          <CardContent className="p-4 flex items-center justify-between">
            <div className="flex-1">
              <h3 className="font-semibold text-gray-900">
                {store.description || 'Untitled Policy Store'}
              </h3>
              <p className="text-sm text-gray-500 mt-1">
                ID: {store.policyStoreId}
              </p>
              <p className="text-xs text-gray-400 mt-1">
                Created {formatRelativeTime(store.createdAt)}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onDeleteStore?.(store.policyStoreId);
                }}
                className="p-2 hover:bg-red-50 rounded-md text-red-600 hover:text-red-700"
                title="Delete policy store"
              >
                <Trash2 className="h-4 w-4" />
              </button>
              <ChevronRight className="h-5 w-5 text-gray-400" />
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
};
