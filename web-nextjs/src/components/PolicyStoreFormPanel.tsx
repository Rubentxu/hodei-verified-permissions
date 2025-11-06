"use client";

import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";

interface PolicyStoreFormPanelProps {
  isLoading: boolean;
  onSubmit: (
    name: string,
    description: string,
    tags: string[],
    user: string,
  ) => void;
  initialName?: string;
  initialDescription?: string;
  initialTags?: string[];
  initialUser?: string;
  currentUser?: string;
  showTagsAndUser?: boolean;
}

const PolicyStoreFormPanel: React.FC<PolicyStoreFormPanelProps> = ({
  isLoading,
  onSubmit,
  initialName = "",
  initialDescription = "",
  initialTags = [],
  initialUser = "",
  currentUser = "web_user", // Default fallback
  showTagsAndUser = false,
}) => {
  const [name, setName] = useState(initialName);
  const [description, setDescription] = useState(initialDescription);
  const [tags, setTags] = useState<string[]>(initialTags);
  const [tagInput, setTagInput] = useState("");
  const [user, setUser] = useState(initialUser || currentUser);
  const [errors, setErrors] = useState<{
    name?: string;
    user?: string;
  }>({});
  const isEditing = initialName.length > 0;

  useEffect(() => {
    setName(initialName);
    setDescription(initialDescription);
    setTags(initialTags);
    setUser(initialUser || currentUser);
    if (!isEditing && showTagsAndUser) {
      setUser(currentUser);
    }
    // Clear errors when fields change
    setErrors({});
  }, [
    initialName,
    initialDescription,
    initialTags,
    initialUser,
    currentUser,
    isEditing,
    showTagsAndUser,
  ]);

  const handleAddTag = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      const newTag = tagInput.trim();
      if (newTag && !tags.includes(newTag)) {
        setTags([...tags, newTag]);
        setTagInput("");
      }
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter((tag) => tag !== tagToRemove));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    // Validate required fields
    const newErrors: { name?: string; user?: string } = {};

    if (!name.trim()) {
      newErrors.name = "Name is required";
    }

    if (showTagsAndUser && !user.trim()) {
      newErrors.user = "User is required";
    }

    // If there are errors, set them and don't submit
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    // Clear errors and submit
    setErrors({});
    const userValue = showTagsAndUser ? user : "system";
    onSubmit(name.trim(), description, tags, userValue.trim());
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
          disabled={isEditing}
          className={`w-full px-3 py-2 border rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors ${
            errors.name ? "border-red-500 bg-red-50" : "border-gray-300"
          } ${isEditing ? "bg-gray-100 cursor-not-allowed" : ""}`}
          placeholder={isEditing ? "" : "Enter policy store name"}
        />
        {isEditing && (
          <p className="mt-1 text-xs text-gray-500">
            Name cannot be changed after creation
          </p>
        )}
        {errors.name && (
          <p className="mt-1 text-sm text-red-600 flex items-center gap-1">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                clipRule="evenodd"
              />
            </svg>
            {errors.name}
          </p>
        )}
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
      {showTagsAndUser && (
        <>
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Tags
            </label>
            <div className="space-y-2">
              {/* Tag chips display */}
              {tags.length > 0 && (
                <div className="flex flex-wrap gap-2">
                  {tags.map((tag) => (
                    <span
                      key={tag}
                      className="inline-flex items-center gap-1 px-3 py-1 bg-blue-100 text-blue-800 text-sm rounded-full"
                    >
                      {tag}
                      <button
                        type="button"
                        onClick={() => handleRemoveTag(tag)}
                        className="hover:text-blue-600 focus:outline-none"
                        aria-label={`Remove ${tag} tag`}
                      >
                        <svg
                          className="w-4 h-4"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M6 18L18 6M6 6l12 12"
                          />
                        </svg>
                      </button>
                    </span>
                  ))}
                </div>
              )}
              {/* Tag input */}
              <input
                type="text"
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyDown={handleAddTag}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="Type a tag and press Enter (e.g., production, frontend, api)"
              />
              <p className="text-xs text-gray-500">
                Press{" "}
                <kbd className="px-1.5 py-0.5 bg-gray-100 rounded text-xs">
                  Enter
                </kbd>{" "}
                or{" "}
                <kbd className="px-1.5 py-0.5 bg-gray-100 rounded text-xs">
                  ,
                </kbd>{" "}
                to add a tag. Tags help categorize and organize policy stores.
              </p>
            </div>
          </div>
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              User <span className="text-red-500">*</span>
            </label>
            <input
              type="text"
              value={user}
              onChange={(e) => setUser(e.target.value)}
              className={`w-full px-3 py-2 border rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors ${
                errors.user ? "border-red-500 bg-red-50" : "border-gray-300"
              }`}
              placeholder="Enter your username or email"
            />
            {errors.user && (
              <p className="mt-1 text-sm text-red-600 flex items-center gap-1">
                <svg
                  className="w-4 h-4"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fillRule="evenodd"
                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                    clipRule="evenodd"
                  />
                </svg>
                {errors.user}
              </p>
            )}
          </div>
        </>
      )}
      <div className="flex justify-center pt-4 border-t border-gray-200 mt-6">
        <button
          type="submit"
          disabled={
            isLoading || !name.trim() || (showTagsAndUser && !user.trim())
          }
          className={`
            px-8 py-3 font-medium rounded-lg transition-all duration-200
            ${
              initialName
                ? "bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white"
                : "bg-indigo-600 hover:bg-indigo-700 active:bg-indigo-800 text-white"
            }
            disabled:opacity-50 disabled:cursor-not-allowed
            shadow-sm hover:shadow-md
            flex items-center gap-2
          `}
        >
          {isLoading ? (
            <>
              <svg
                className="animate-spin h-4 w-4"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              {initialName ? "Updating..." : "Creating..."}
            </>
          ) : (
            <>
              {initialName ? (
                <>
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                    />
                  </svg>
                  Update
                </>
              ) : (
                <>
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 4v16m8-8H4"
                    />
                  </svg>
                  Create
                </>
              )}
            </>
          )}
        </button>
      </div>
    </form>
  );
};

export default PolicyStoreFormPanel;
