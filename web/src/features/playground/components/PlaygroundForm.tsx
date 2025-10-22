/**
 * PlaygroundForm Component - Form to test authorization
 */

import React, { useState } from 'react';
import { Button, Input, Card, CardContent, CardHeader, CardTitle, Alert } from '../../../components';
import { JsonEditor } from '../../../components/editors';

export interface PlaygroundFormProps {
  onSubmit?: (data: {
    principal?: string;
    action?: string;
    resource?: string;
    context?: Record<string, unknown>;
    policies: string[];
    entities: unknown[];
  }) => void;
  isLoading?: boolean;
  error?: string;
}

export const PlaygroundForm: React.FC<PlaygroundFormProps> = ({
  onSubmit,
  isLoading,
  error,
}) => {
  const [principal, setPrincipal] = useState('');
  const [action, setAction] = useState('');
  const [resource, setResource] = useState('');
  const [context, setContext] = useState('{}');
  const [policies, setPolicies] = useState('permit(principal, action, resource);');
  const [entities, setEntities] = useState('[]');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    try {
      const contextObj = JSON.parse(context);
      const entitiesArr = JSON.parse(entities);

      onSubmit?.({
        principal,
        action,
        resource,
        context: contextObj,
        policies: [policies],
        entities: entitiesArr,
      });
    } catch (err) {
      console.error('Failed to parse JSON:', err);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Authorization Test</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* PARC Fields */}
          <div className="grid grid-cols-3 gap-4">
            <Input
              label="Principal"
              placeholder="e.g., User::alice"
              value={principal}
              onChange={(e) => setPrincipal(e.target.value)}
            />
            <Input
              label="Action"
              placeholder="e.g., Action::read"
              value={action}
              onChange={(e) => setAction(e.target.value)}
            />
            <Input
              label="Resource"
              placeholder="e.g., Document::doc1"
              value={resource}
              onChange={(e) => setResource(e.target.value)}
            />
          </div>

          {/* Context */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Context (JSON)
            </label>
            <JsonEditor
              value={context}
              onChange={setContext}
              height="150px"
              aria-label="Context (JSON)"
            />
          </div>

          {/* Policies */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Policies
            </label>
            <textarea
              value={policies}
              onChange={(e) => setPolicies(e.target.value)}
              className="w-full h-32 p-3 border border-gray-300 rounded-md font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="permit(principal, action, resource);"
            />
          </div>

          {/* Entities */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Entities (JSON)
            </label>
            <JsonEditor
              value={entities}
              onChange={setEntities}
              height="150px"
              aria-label="Entities (JSON)"
            />
          </div>

          {error && (
            <Alert
              type="error"
              message={error}
              closeable={false}
            />
          )}

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="ghost"
              onClick={() => {
                setPrincipal('');
                setAction('');
                setResource('');
                setContext('{}');
                setPolicies('permit(principal, action, resource);');
                setEntities('[]');
              }}
              disabled={isLoading}
            >
              Reset
            </Button>
            <Button
              type="submit"
              variant="primary"
              loading={isLoading}
              disabled={isLoading}
            >
              Evaluate
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
};
