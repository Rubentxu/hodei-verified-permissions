'use client';

import React, { useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Plus, X } from 'lucide-react';
import { usePolicyStoreTags, useAllTags } from '@/hooks/usePolicyStores';

interface TagManagerProps {
  policyStoreId: string;
  readonly?: boolean;
}

const TagManager: React.FC<TagManagerProps> = ({ policyStoreId, readonly = false }) => {
  const { tags, isLoading, addTag, removeTag } = usePolicyStoreTags(policyStoreId);
  const { data: allTags = [] } = useAllTags();
  const [newTag, setNewTag] = useState('');
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  const handleAddTag = async (tag: string) => {
    const trimmedTag = tag.trim();
    if (!trimmedTag) return;

    try {
      await addTag(trimmedTag);
      setNewTag('');
      setSuggestions([]);
      setShowSuggestions(false);
    } catch (error) {
      console.error('Failed to add tag:', error);
      alert('Failed to add tag: ' + (error as Error).message);
    }
  };

  const handleRemoveTag = async (tag: string) => {
    try {
      await removeTag(tag);
    } catch (error) {
      console.error('Failed to remove tag:', error);
      alert('Failed to remove tag: ' + (error as Error).message);
    }
  };

  const handleInputChange = (value: string) => {
    setNewTag(value);
    if (value.trim()) {
      const filtered = allTags.filter(tag =>
        tag.toLowerCase().includes(value.toLowerCase()) &&
        !tags.includes(tag)
      );
      setSuggestions(filtered.slice(0, 5)); // Limit to 5 suggestions
      setShowSuggestions(true);
    } else {
      setSuggestions([]);
      setShowSuggestions(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleAddTag(newTag);
    } else if (e.key === 'Escape') {
      setShowSuggestions(false);
    }
  };

  if (isLoading) {
    return <div className="animate-pulse h-8 bg-gray-200 rounded w-full"></div>;
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-2">
        {tags.length > 0 ? (
          tags.map((tag, index) => (
            <Badge key={index} variant="outline" className="px-3 py-1">
              {tag}
              {!readonly && (
                <button
                  onClick={() => handleRemoveTag(tag)}
                  className="ml-2 hover:text-red-600"
                  title="Remove tag"
                >
                  <X className="w-3 h-3" />
                </button>
              )}
            </Badge>
          ))
        ) : (
          <span className="text-sm text-gray-500">No tags</span>
        )}
      </div>

      {!readonly && (
        <div className="relative">
          <div className="flex space-x-2">
            <Input
              value={newTag}
              onChange={(e) => handleInputChange(e.target.value)}
              onKeyDown={handleKeyPress}
              placeholder="Add a tag..."
              className="flex-1"
            />
            <Button
              onClick={() => handleAddTag(newTag)}
              size="sm"
              disabled={!newTag.trim()}
            >
              <Plus className="w-4 h-4 mr-1" />
              Add
            </Button>
          </div>

          {showSuggestions && suggestions.length > 0 && (
            <div className="absolute z-10 mt-1 w-full bg-white border border-gray-200 rounded-md shadow-lg">
              {suggestions.map((suggestion, index) => (
                <button
                  key={index}
                  onClick={() => handleAddTag(suggestion)}
                  className="block w-full text-left px-3 py-2 hover:bg-gray-100 text-sm"
                >
                  {suggestion}
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default TagManager;
