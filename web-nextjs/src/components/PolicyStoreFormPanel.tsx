"use client";

import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";

interface PolicyStoreFormPanelProps {
  isLoading: boolean;
  onSubmit: (name: string, description: string) => void;
  initialName?: string;
  initialDescription?: string;
}

const PolicyStoreFormPanel: React.FC<PolicyStoreFormPanelProps> = ({
  isLoading,
  onSubmit,
  initialName = "",
  initialDescription = "",
}) => {
  const [name, setName] = useState(initialName);
  const [description, setDescription] = useState(initialDescription);

  useEffect(() => {
    setName(initialName);
    setDescription(initialDescription);
  }, [initialName, initialDescription]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(name, description);
  };

  return (
    <form onSubmit={handleSubmit}>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Name <span className="text-red-500">*</span>
        </label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          required
          placeholder="Enter policy store name"
        />
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Description
        </label>
        <textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          rows={3}
          placeholder="Enter policy store description (optional)"
        />
      </div>
      <div className="flex justify-end space-x-2">
        <Button type="submit" disabled={isLoading || !name}>
          {isLoading
            ? initialName
              ? "Updating..."
              : "Creating..."
            : initialName
              ? "Update"
              : "Create"}
        </Button>
      </div>
    </form>
  );
};

export default PolicyStoreFormPanel;
