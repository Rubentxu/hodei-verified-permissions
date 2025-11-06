"use client";

import React, { useState } from "react";
import {
  usePolicyStore,
  usePolicyStoreSnapshots,
  usePolicyStoreTags,
  useUpdatePolicyStore,
} from "@/hooks/usePolicyStores";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Shield,
  Eye,
  FileText,
  Layers,
  Clock,
  Calendar,
  Tag,
  History,
  Edit,
  Save,
  X,
  User,
} from "lucide-react";
import TagManager from "./TagManager";

// Tags Panel Component
interface TagsPanelProps {
  policyStoreId: string;
}

const TagsPanel: React.FC<TagsPanelProps> = ({ policyStoreId }) => {
  const {
    data: tagsData,
    isLoading,
    error,
  } = usePolicyStoreTags(policyStoreId);

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[...Array(3)].map((_, i) => (
          <div
            key={i}
            className="animate-pulse h-8 bg-gray-200 rounded w-full"
          ></div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-red-600">Error loading tags: {error.message}</div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-3">Manage Tags</h3>
        <TagManager policyStoreId={policyStoreId} />
      </div>

      {tagsData?.tags && tagsData.tags.length > 0 && (
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-2">
            Current Tags ({tagsData.tags.length})
          </h4>
          <div className="flex flex-wrap gap-2">
            {tagsData.tags.map((tag: string, index: number) => (
              <span
                key={index}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-blue-50 text-blue-700 rounded-full text-sm font-medium"
              >
                <Tag className="w-3 h-3" />
                {tag}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

// Version History Panel Component
interface VersionHistoryPanelProps {
  policyStoreId: string;
}

const VersionHistoryPanel: React.FC<VersionHistoryPanelProps> = ({
  policyStoreId,
}) => {
  const {
    data: snapshots,
    isLoading,
    error,
  } = usePolicyStoreSnapshots(policyStoreId);

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="animate-pulse h-20 bg-gray-200 rounded"></div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-red-600">
        Error loading version history: {error.message}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {snapshots?.snapshots?.length === 0 ? (
        <p className="text-gray-500 text-center py-8">
          No version history found
        </p>
      ) : (
        snapshots?.snapshots?.map((snapshot: any, index: number) => (
          <div key={index} className="border rounded-lg p-4 hover:bg-gray-50">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center space-x-2 mb-2">
                  <Badge variant="outline">
                    v{snapshot.version || index + 1}
                  </Badge>
                  <span className="text-sm text-gray-500">
                    {new Date(snapshot.created_at).toLocaleString()}
                  </span>
                </div>
                {snapshot.description && (
                  <p className="text-sm text-gray-700 mb-2">
                    {snapshot.description}
                  </p>
                )}
                <div className="flex items-center space-x-4 text-xs text-gray-500">
                  <div className="flex items-center space-x-1">
                    <User className="w-3 h-3" />
                    <span>{snapshot.created_by || "Unknown"}</span>
                  </div>
                  <div className="flex items-center space-x-1">
                    <Calendar className="w-3 h-3" />
                    <span>
                      {new Date(snapshot.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              </div>
              <div className="flex space-x-2">
                <Button variant="ghost" size="sm" title="View snapshot">
                  <Eye className="w-4 h-4" />
                </Button>
                <Button variant="ghost" size="sm" title="Download snapshot">
                  <FileText className="w-4 h-4" />
                </Button>
              </div>
            </div>
          </div>
        ))
      )}
    </div>
  );
};

interface PolicyStoreDetailsPanelProps {
  policyStoreId: string;
}

const PolicyStoreDetailsPanel: React.FC<PolicyStoreDetailsPanelProps> = ({
  policyStoreId,
}) => {
  const { data: policyStore, isLoading, error } = usePolicyStore(policyStoreId);
  const [activeTab, setActiveTab] = useState<"overview" | "tags" | "versions">(
    "overview",
  );
  const [isEditing, setIsEditing] = useState(false);
  const [descriptionText, setDescriptionText] = useState("");
  const updateMutation = useUpdatePolicyStore();

  const handleEdit = () => {
    setIsEditing(true);
    setDescriptionText(policyStore?.description || "");
  };

  const handleSave = async () => {
    try {
      await updateMutation.mutateAsync({
        policyStoreId,
        description: descriptionText,
      });
      setIsEditing(false);
    } catch (error) {
      console.error("Failed to update policy store:", error);
      alert("Failed to update policy store: " + (error as Error).message);
    }
  };

  const handleCancel = () => {
    setIsEditing(false);
    setDescriptionText(policyStore?.description || "");
  };

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
      <div className="flex space-x-1 mb-6 border-b">
        <button
          onClick={() => setActiveTab("overview")}
          className={`px-4 py-2 flex items-center space-x-2 ${
            activeTab === "overview"
              ? "border-b-2 border-blue-600 text-blue-600"
              : "text-gray-500 hover:text-gray-700"
          }`}
        >
          <FileText className="w-4 h-4" />
          <span>Overview</span>
        </button>
        <button
          onClick={() => setActiveTab("tags")}
          className={`px-4 py-2 flex items-center space-x-2 ${
            activeTab === "tags"
              ? "border-b-2 border-blue-600 text-blue-600"
              : "text-gray-500 hover:text-gray-700"
          }`}
        >
          <Tag className="w-4 h-4" />
          <span>Tags</span>
        </button>
        <button
          onClick={() => setActiveTab("versions")}
          className={`px-4 py-2 flex items-center space-x-2 ${
            activeTab === "versions"
              ? "border-b-2 border-blue-600 text-blue-600"
              : "text-gray-500 hover:text-gray-700"
          }`}
        >
          <History className="w-4 h-4" />
          <span>Version History</span>
        </button>
      </div>

      {activeTab === "overview" && (
        <>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            {/* ... (Metrics Grid implementation remains the same) */}
          </div>

          <div className="mb-6">
            <div className="flex items-center justify-between mb-2">
              <h4 className="text-lg font-semibold">Description</h4>
              {!isEditing && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleEdit}
                  className="flex items-center space-x-1"
                >
                  <Edit className="w-4 h-4" />
                  <span>Edit</span>
                </Button>
              )}
            </div>
            {isEditing ? (
              <div className="space-y-3">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Description
                  </label>
                  <textarea
                    value={descriptionText}
                    onChange={(e) => setDescriptionText(e.target.value)}
                    className="w-full min-h-[100px] p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    placeholder="Enter policy store description..."
                  />
                </div>
                <div className="flex space-x-2">
                  <Button
                    onClick={handleSave}
                    disabled={updateMutation.isPending}
                    className="flex items-center space-x-1"
                  >
                    <Save className="w-4 h-4" />
                    <span>
                      {updateMutation.isPending ? "Saving..." : "Save"}
                    </span>
                  </Button>
                  <Button
                    variant="outline"
                    onClick={handleCancel}
                    disabled={updateMutation.isPending}
                    className="flex items-center space-x-1"
                  >
                    <X className="w-4 h-4" />
                    <span>Cancel</span>
                  </Button>
                </div>
              </div>
            ) : (
              <div>
                <span className="text-gray-700">
                  {policyStore.description || "No description provided"}
                </span>
              </div>
            )}
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
            {/* ... (Metadata implementation remains the same) */}
          </div>
        </>
      )}

      {activeTab === "tags" && (
        <TagsPanel policyStoreId={policyStore.policy_store_id} />
      )}
      {activeTab === "versions" && (
        <VersionHistoryPanel policyStoreId={policyStore.policy_store_id} />
      )}
    </div>
  );
};

export default PolicyStoreDetailsPanel;
