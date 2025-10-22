/**
 * PolicyForm Component - Form to create/edit a policy
 */

import React, { useState } from 'react';
import { Button, Input, Card, CardContent, CardHeader, CardTitle } from '../../../components';
import { CedarEditor } from '../../../components/editors';

export interface PolicyFormProps {
  policyId?: string;
  statement?: string;
  description?: string;
  onSubmit?: (data: { policyId: string; statement: string; description?: string }) => void;
  isLoading?: boolean;
  error?: string;
  isEditing?: boolean;
}

export const PolicyForm: React.FC<PolicyFormProps> = ({
  policyId: initialPolicyId = '',
  statement: initialStatement = '',
  description: initialDescription = '',
  onSubmit,
  isLoading,
  error,
  isEditing = false,
}) => {
  const [policyId, setPolicyId] = useState(initialPolicyId);
  const [statement, setStatement] = useState(initialStatement);
  const [description, setDescription] = useState(initialDescription);
  const [isValid, setIsValid] = useState(!!initialStatement);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (policyId.trim() && statement.trim() && isValid) {
      onSubmit?.({
        policyId,
        statement,
        description,
      });
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>
          {isEditing ? 'Edit Policy' : 'Create New Policy'}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <Input
            label="Policy ID"
            placeholder="e.g., allow-read-documents"
            value={policyId}
            onChange={(e) => setPolicyId(e.target.value)}
            disabled={isEditing}
            required
          />

          <Input
            label="Description"
            placeholder="e.g., Allow users to read documents"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Cedar Policy Statement *
            </label>
            <CedarEditor
              value={statement}
              onChange={setStatement}
              onValidationChange={setIsValid}
              height="300px"
            />
          </div>

          {error && (
            <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="ghost"
              onClick={() => {
                setPolicyId(initialPolicyId);
                setStatement(initialStatement);
                setDescription(initialDescription);
              }}
              disabled={isLoading}
            >
              Reset
            </Button>
            <Button
              type="submit"
              variant="primary"
              loading={isLoading}
              disabled={!policyId.trim() || !statement.trim() || !isValid || isLoading}
            >
              {isEditing ? 'Update Policy' : 'Create Policy'}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
};
