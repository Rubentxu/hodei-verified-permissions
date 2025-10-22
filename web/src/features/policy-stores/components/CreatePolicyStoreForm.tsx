/**
 * CreatePolicyStoreForm Component - Form to create a new policy store
 */

import React, { useState } from 'react';
import { Button, Input, Card, CardContent, CardHeader, CardTitle } from '../../../components';

export interface CreatePolicyStoreFormProps {
  onSubmit?: (description: string) => void;
  isLoading?: boolean;
  error?: string;
}

export const CreatePolicyStoreForm: React.FC<CreatePolicyStoreFormProps> = ({
  onSubmit,
  isLoading,
  error,
}) => {
  const [description, setDescription] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (description.trim()) {
      onSubmit?.(description);
      setDescription('');
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Create New Policy Store</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            label="Description"
            placeholder="e.g., Production Authorization Store"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            error={error}
            required
          />
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="ghost"
              onClick={() => setDescription('')}
              disabled={isLoading}
            >
              Clear
            </Button>
            <Button
              type="submit"
              variant="primary"
              loading={isLoading}
              disabled={!description.trim() || isLoading}
            >
              Create Policy Store
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
};
