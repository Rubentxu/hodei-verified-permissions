/**
 * PoliciesList Component - Display list of policies
 */

import React from 'react';
import { Policy } from '../../../types';
import {
  Card,
  CardContent,
  LoadingSpinner,
  Alert,
} from '../../../components';
import { formatRelativeTime, truncate } from '../../../utils';
import { Edit2, Trash2, ChevronRight } from 'lucide-react';

export interface PoliciesListProps {
  policies: Policy[];
  isLoading?: boolean;
  error?: string;
  onSelectPolicy?: (policy: Policy) => void;
  onDeletePolicy?: (id: string) => void;
  searchTerm?: string;
  filterEffect?: 'permit' | 'forbid' | undefined;
}

export const PoliciesList: React.FC<PoliciesListProps> = ({
  policies,
  isLoading,
  error,
  onSelectPolicy,
  onDeletePolicy,
  searchTerm = '',
  filterEffect,
}) => {
  // Filter policies
  const filteredPolicies = policies.filter((policy) => {
    const matchesSearch =
      policy.policyId.toLowerCase().includes(searchTerm.toLowerCase()) ||
      policy.description?.toLowerCase().includes(searchTerm.toLowerCase());

    const matchesEffect = !filterEffect || policy.statement.startsWith(filterEffect);

    return matchesSearch && matchesEffect;
  });

  if (isLoading) {
    return (
      <div className="flex justify-center py-12">
        <LoadingSpinner message="Loading policies..." />
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        type="error"
        title="Error loading policies"
        message={error}
      />
    );
  }

  if (filteredPolicies.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <p className="text-gray-500">No policies found</p>
          <p className="text-sm text-gray-400 mt-2">
            {searchTerm || filterEffect ? 'Try adjusting your filters' : 'Create your first policy to get started'}
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-3">
      {filteredPolicies.map((policy) => {
        const isPermit = policy.statement.startsWith('permit');
        const effect = isPermit ? 'permit' : 'forbid';

        return (
          <Card
            key={policy.policyId}
            className="Card hover:shadow-md transition-shadow cursor-pointer"
            onClick={() => onSelectPolicy?.(policy)}
            role="button"
            tabIndex={0}
          >
            <CardContent className="p-4 flex items-center justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <h3 className="font-semibold text-gray-900">
                    {policy.policyId}
                  </h3>
                  <span
                    className={`text-xs px-2 py-1 rounded-full font-semibold ${
                      isPermit
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}
                  >
                    {effect}
                  </span>
                </div>
                {policy.description && (
                  <p className="text-sm text-gray-600 mt-1">
                    {truncate(policy.description, 100)}
                  </p>
                )}
                <p className="text-xs text-gray-400 mt-1">
                  {truncate(policy.statement, 80)}
                </p>
                <p className="text-xs text-gray-400 mt-1">
                  Updated {formatRelativeTime(policy.updatedAt)}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelectPolicy?.(policy);
                  }}
                  className="p-2 hover:bg-blue-50 rounded-md text-blue-600 hover:text-blue-700"
                  title="Edit policy"
                >
                  <Edit2 className="h-4 w-4" />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeletePolicy?.(policy.policyId);
                  }}
                  className="p-2 hover:bg-red-50 rounded-md text-red-600 hover:text-red-700"
                  title="Delete policy"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
                <ChevronRight className="h-5 w-5 text-gray-400" />
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
};
