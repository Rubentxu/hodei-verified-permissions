/**
 * SchemaEditorPage - HU 15.1 & 15.2
 * View and edit schema with real-time validation
 */

import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { Button, LoadingSpinner, Alert, Card, CardContent, CardHeader, CardTitle } from '../../../components';
import { useSchema, useUpdateSchema } from '../../../api';
import { useUIStore } from '../../../store';
import { SchemaEditor } from '../components/SchemaEditor';
import { Save, X } from 'lucide-react';

export const SchemaEditorPage: React.FC = () => {
  const { policyStoreId } = useParams<{ policyStoreId: string }>();
  const { addNotification } = useUIStore();
  const [schemaContent, setSchemaContent] = useState('');
  const [isValid, setIsValid] = useState(false);
  const [isDirty, setIsDirty] = useState(false);

  // Fetch schema
  const { data: schema, isLoading, error } = useSchema(policyStoreId || '');
  const updateMutation = useUpdateSchema();

  // Initialize schema content
  useEffect(() => {
    if (schema?.schema) {
      setSchemaContent(schema.schema);
      setIsDirty(false);
    }
  }, [schema]);

  const handleSchemaChange = (newValue: string) => {
    setSchemaContent(newValue);
    setIsDirty(true);
  };

  const handleSave = async () => {
    if (!policyStoreId) return;

    try {
      await updateMutation.mutateAsync({
        policyStoreId,
        schema: schemaContent,
      });
      addNotification('Schema saved successfully', 'success');
      setIsDirty(false);
    } catch (err) {
      addNotification(
        err instanceof Error ? err.message : 'Failed to save schema',
        'error'
      );
    }
  };

  const handleDiscard = () => {
    if (schema?.schema) {
      setSchemaContent(schema.schema);
      setIsDirty(false);
    }
  };

  if (isLoading) {
    return (
      <div className="flex justify-center py-12">
        <LoadingSpinner message="Loading schema..." />
      </div>
    );
  }

  if (error) {
    return (
      <Alert
        type="error"
        title="Error loading schema"
        message={error.message}
      />
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Schema Editor</h1>
          <p className="text-gray-600 mt-1">
            Define the Cedar schema for {policyStoreId}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            onClick={handleDiscard}
            disabled={!isDirty || updateMutation.isPending}
          >
            <X className="w-4 h-4 mr-2" />
            Discard
          </Button>
          <Button
            variant="primary"
            onClick={handleSave}
            disabled={!isDirty || !isValid || updateMutation.isPending}
            loading={updateMutation.isPending}
          >
            <Save className="w-4 h-4 mr-2" />
            Save Schema
          </Button>
        </div>
      </div>

      {/* Schema Editor */}
      <SchemaEditor
        value={schemaContent}
        onChange={handleSchemaChange}
        onValidationChange={setIsValid}
        error={updateMutation.error?.message}
      />

      {/* Info Card */}
      <Card>
        <CardHeader>
          <CardTitle>Schema Information</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <p>
            <span className="font-semibold">Policy Store ID:</span> {policyStoreId}
          </p>
          <p>
            <span className="font-semibold">Last Updated:</span>{' '}
            {schema?.updatedAt ? new Date(schema.updatedAt).toLocaleString() : 'Never'}
          </p>
          <p>
            <span className="font-semibold">Status:</span>{' '}
            <span className={isValid ? 'text-green-600' : 'text-red-600'}>
              {isValid ? '✓ Valid' : '✗ Invalid'}
            </span>
          </p>
        </CardContent>
      </Card>
    </div>
  );
};

export default SchemaEditorPage;
