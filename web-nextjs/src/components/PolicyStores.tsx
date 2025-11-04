"use client";

import React, { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Shield,
  Plus,
  Search,
  Filter,
  Edit,
  Copy,
  Trash2,
  AlertCircle,
  Eye,
} from "lucide-react";
import { usePolicyStores, useDeletePolicyStore } from "@/hooks/usePolicyStores";
import { usePolicyStorePanelStore } from "@/lib/stores/policy-store-panel-store";

const PolicyStores = () => {
  const { data, isLoading, error, refetch } = usePolicyStores();
  const deleteMutation = useDeletePolicyStore();
  const { openPanel, isOpen, selectedStoreId } = usePolicyStorePanelStore();

  const handleDelete = async (policyStoreId: string) => {
    if (window.confirm("Are you sure you want to delete this policy store?")) {
      try {
        await deleteMutation.mutateAsync(policyStoreId);
      } catch (error) {
        console.error("Failed to delete policy store:", error);
        alert("Failed to delete policy store: " + (error as Error).message);
      }
    }
  };

  const handleViewDetails = (storeId: string) => {
    if (isOpen && selectedStoreId === storeId) {
      usePolicyStorePanelStore.getState().closePanel();
    } else {
      openPanel("details", storeId);
    }
  };

  const [searchTerm, setSearchTerm] = useState("");

  const filteredStores = (data?.policy_stores || []).filter(
    (store) =>
      store.policy_store_id.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (store.name &&
        store.name.toLowerCase().includes(searchTerm.toLowerCase())) ||
      (store.description &&
        store.description.toLowerCase().includes(searchTerm.toLowerCase())),
  );

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (error) {
    return <div>Error: {error.message}</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-end">
        <Button
          onClick={() => openPanel("create")}
          className="flex items-center space-x-2"
        >
          <Plus className="w-4 h-4" />
          <span>Create Policy Store</span>
        </Button>
      </div>

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
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredStores.map((store) => (
          <Card
            key={store.policy_store_id}
            className={`hover:shadow-md transition-shadow ${isOpen && selectedStoreId === store.policy_store_id ? "border-blue-500" : ""}`}
          >
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <Shield className="w-5 h-5 text-blue-600" />
                  <Badge variant="outline">{store.policy_store_id}</Badge>
                  <Badge
                    variant={
                      store.status === "active" ? "default" : "secondary"
                    }
                  >
                    {store.status}
                  </Badge>
                </div>
                <div className="flex items-center space-x-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => openPanel("edit", store.policy_store_id)}
                    title="Edit policy store"
                  >
                    <Edit className="w-4 h-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDelete(store.policy_store_id)}
                    disabled={deleteMutation.isPending}
                    title="Delete policy store"
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
              <CardTitle className="text-lg">
                {store.name || "No name"}
              </CardTitle>
              <CardDescription>
                {store.description || "No description"}
              </CardDescription>
              <div className="flex items-center space-x-2 text-sm text-gray-500">
                <span>v{store.version}</span>
                <span>•</span>
                <span>by {store.author}</span>
                <span>•</span>
                <span>
                  Created: {new Date(store.created_at).toLocaleDateString()}
                </span>
              </div>
              {store.tags && store.tags.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-2">
                  {store.tags.map((tag, index) => (
                    <Badge key={index} variant="outline" className="text-xs">
                      {tag}
                    </Badge>
                  ))}
                </div>
              )}
            </CardHeader>
            <CardContent>
              <div className="pt-3">
                <Button
                  variant={
                    isOpen && selectedStoreId === store.policy_store_id
                      ? "default"
                      : "outline"
                  }
                  className="w-full"
                  onClick={() => handleViewDetails(store.policy_store_id)}
                >
                  <Eye className="w-4 h-4 mr-2" />
                  {isOpen && selectedStoreId === store.policy_store_id
                    ? "Hide Details"
                    : "View Details"}
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
};

export default PolicyStores;
