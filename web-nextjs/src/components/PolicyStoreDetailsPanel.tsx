"use client";

import React from "react";
import { usePolicyStore } from "@/hooks/usePolicyStores";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Shield, FileText, Tag, User } from "lucide-react";

interface PolicyStoreDetailsPanelProps {
  policyStoreId: string;
}

const PolicyStoreDetailsPanel: React.FC<PolicyStoreDetailsPanelProps> = ({
  policyStoreId,
}) => {
  const { data: policyStore, isLoading, error } = usePolicyStore(policyStoreId);

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (error) {
    return <div>Error: {error.message}</div>;
  }

  if (!policyStore) {
    return <div>Policy store not found.</div>;
  }

  return (
    <div>
      <h2 className="text-2xl font-bold text-gray-900 mb-6">
        Policy Store Overview
      </h2>

      <div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
          <Card>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">Status</p>
                  <p className="text-2xl font-bold text-gray-900 mt-1">
                    <Badge
                      variant={
                        policyStore.status === "active"
                          ? "default"
                          : "secondary"
                      }
                      className="text-sm"
                    >
                      {policyStore.status}
                    </Badge>
                  </p>
                </div>
                <Shield className="w-8 h-8 text-blue-600" />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">Version</p>
                  <p className="text-2xl font-bold text-gray-900 mt-1">
                    v{policyStore.version}
                  </p>
                </div>
                <FileText className="w-8 h-8 text-green-600" />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">Author</p>
                  <p className="text-2xl font-bold text-gray-900 mt-1">
                    {policyStore.author}
                  </p>
                </div>
                <User className="w-8 h-8 text-purple-600" />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">Tags</p>
                  <p className="text-2xl font-bold text-gray-900 mt-1">
                    {policyStore.tags?.length || 0}
                  </p>
                </div>
                <Tag className="w-8 h-8 text-orange-600" />
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="mb-6">
          <h4 className="text-lg font-semibold mb-2">Description</h4>
          <div>
            <span className="text-gray-700">
              {policyStore.description || "No description provided"}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          <Card>
            <CardContent className="pt-6">
              <h4 className="text-sm font-semibold text-gray-600 mb-3">
                Metadata
              </h4>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">Store ID:</span>
                  <code className="text-sm bg-gray-100 px-2 py-1 rounded">
                    {policyStore.policy_store_id}
                  </code>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">Created:</span>
                  <span className="text-sm text-gray-900">
                    {new Date(policyStore.created_at).toLocaleString()}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">Last Updated:</span>
                  <span className="text-sm text-gray-900">
                    {new Date(policyStore.updated_at).toLocaleString()}
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="pt-6">
              <h4 className="text-sm font-semibold text-gray-600 mb-3">Tags</h4>
              {policyStore.tags && policyStore.tags.length > 0 ? (
                <div className="flex flex-wrap gap-2">
                  {policyStore.tags.map((tag: string, index: number) => (
                    <Badge key={index} variant="outline" className="text-xs">
                      {tag}
                    </Badge>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-gray-500">No tags assigned</p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default PolicyStoreDetailsPanel;
