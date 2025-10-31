'use client';

import React, { useState, useCallback } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  FileText,
  Save,
  Play,
  CheckCircle,
  XCircle,
  AlertCircle,
  RefreshCw,
  Upload,
  Download,
} from 'lucide-react';
import Editor from '@monaco-editor/react';
import { useSchema, useUpdateSchema, useValidateSchema } from '@/hooks/useSchemas';
import { usePolicyStores } from '@/hooks/usePolicyStores';

const DEFAULT_SCHEMA = `{
  "entity_type": "User",
  "attributes": {
    "id": {
      "type": "string",
      "required": true,
      "description": "Unique identifier for the user"
    },
    "email": {
      "type": "string",
      "required": true,
      "format": "email",
      "description": "User email address"
    },
    "role": {
      "type": "string",
      "enum": ["admin", "user", "viewer"],
      "default": "user",
      "description": "User role in the system"
    },
    "department": {
      "type": "string",
      "required": false,
      "description": "Department the user belongs to"
    }
  }
}`;

const Schemas = () => {
  const [selectedPolicyStoreId, setSelectedPolicyStoreId] = useState<string>('');
  const [editorContent, setEditorContent] = useState(DEFAULT_SCHEMA);
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);

  // Fetch policy stores for selection
  const {
    data: policyStoresData,
    isLoading: isLoadingStores,
    error: storesError,
  } = usePolicyStores();

  // Fetch schema for selected policy store
  const {
    data: schemaData,
    isLoading: isLoadingSchema,
    error: schemaError,
    refetch: refetchSchema,
  } = useSchema(selectedPolicyStoreId);

  // Update schema mutation
  const updateSchemaMutation = useUpdateSchema();

  // Validate schema (local validation)
  const {
    data: validationData,
    isFetching: isValidating,
    refetch: refetchValidation,
  } = useValidateSchema(editorContent);

  const handlePolicyStoreChange = useCallback((policyStoreId: string) => {
    setSelectedPolicyStoreId(policyStoreId);
    setEditorContent(DEFAULT_SCHEMA);
    setHasUnsavedChanges(false);
  }, []);

  const handleEditorChange = useCallback((value: string | undefined) => {
    setEditorContent(value || '');
    setHasUnsavedChanges(true);
  }, []);

  const handleSave = useCallback(async () => {
    if (!selectedPolicyStoreId || !validationData?.valid) return;

    try {
      await updateSchemaMutation.mutateAsync({
        policy_store_id: selectedPolicyStoreId,
        schema: editorContent,
      });
      setHasUnsavedChanges(false);
    } catch (error) {
      console.error('Failed to save schema:', error);
    }
  }, [selectedPolicyStoreId, editorContent, validationData, updateSchemaMutation]);

  const handleValidate = useCallback(() => {
    refetchValidation();
  }, [refetchValidation]);

  const handleReset = useCallback(() => {
    if (schemaData?.schema) {
      setEditorContent(schemaData.schema);
      setHasUnsavedChanges(false);
    } else {
      setEditorContent(DEFAULT_SCHEMA);
      setHasUnsavedChanges(false);
    }
  }, [schemaData]);

  // Update editor content when schema data changes
  React.useEffect(() => {
    if (schemaData?.schema) {
      setEditorContent(schemaData.schema);
      setHasUnsavedChanges(false);
    }
  }, [schemaData]);

  const policyStores = policyStoresData?.policy_stores || [];
  const isLoading = isLoadingStores || isLoadingSchema;

  // Loading state
  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="h-8 w-48 bg-gray-200 rounded animate-pulse mb-2"></div>
            <div className="h-4 w-96 bg-gray-200 rounded animate-pulse"></div>
          </div>
          <div className="h-10 w-44 bg-gray-200 rounded animate-pulse"></div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <Card>
            <CardHeader>
              <div className="h-6 w-32 bg-gray-200 rounded animate-pulse"></div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="h-16 bg-gray-200 rounded animate-pulse"></div>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card className="lg:col-span-2">
            <CardHeader>
              <div className="h-6 w-64 bg-gray-200 rounded animate-pulse"></div>
            </CardHeader>
            <CardContent>
              <div className="h-96 bg-gray-200 rounded animate-pulse"></div>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  // Error state for stores
  if (storesError) {
    return (
      <div className="flex items-center justify-center h-64">
        <Card className="max-w-md">
          <CardContent className="pt-6">
            <div className="text-center">
              <AlertCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                Failed to load policy stores
              </h3>
              <p className="text-gray-600 mb-4">
                {storesError.message || 'An error occurred while fetching policy stores'}
              </p>
              <Button onClick={() => window.location.reload()}>
                Retry
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Error state for schema
  if (schemaError && selectedPolicyStoreId) {
    return (
      <div className="flex items-center justify-center h-64">
        <Card className="max-w-md">
          <CardContent className="pt-6">
            <div className="text-center">
              <AlertCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                Failed to load schema
              </h3>
              <p className="text-gray-600 mb-4">
                {schemaError.message || 'An error occurred while fetching schema'}
              </p>
              <Button onClick={() => refetchSchema()}>
                Retry
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Schemas</h2>
          <p className="text-gray-600">Define entity types and their attributes for policy stores</p>
        </div>
        <Button
          onClick={handleSave}
          disabled={!selectedPolicyStoreId || !hasUnsavedChanges || !validationData?.valid || updateSchemaMutation.isPending}
          className="flex items-center space-x-2"
        >
          <Save className="w-4 h-4" />
          <span>{updateSchemaMutation.isPending ? 'Saving...' : 'Save Schema'}</span>
        </Button>
      </div>

      {/* Policy Store Selector */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <FileText className="w-5 h-5" />
            <span>Select Policy Store</span>
          </CardTitle>
          <CardDescription>
            Choose a policy store to view and edit its schema
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {policyStores.map((store) => (
              <div
                key={store.policy_store_id}
                className={`p-4 border rounded-lg cursor-pointer transition-all ${
                  selectedPolicyStoreId === store.policy_store_id
                    ? 'border-blue-500 bg-blue-50 ring-2 ring-blue-200'
                    : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                }`}
                onClick={() => handlePolicyStoreChange(store.policy_store_id)}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <h4 className="font-medium text-gray-900 mb-1">
                      {store.policy_store_id}
                    </h4>
                    <p className="text-sm text-gray-600 mb-2">
                      {store.description || 'No description'}
                    </p>
                    <div className="flex items-center space-x-2">
                      <Badge variant="outline" className="text-xs">
                        Created: {new Date(store.created_at).toLocaleDateString()}
                      </Badge>
                    </div>
                  </div>
                  {selectedPolicyStoreId === store.policy_store_id && (
                    <CheckCircle className="w-5 h-5 text-blue-600 flex-shrink-0" />
                  )}
                </div>
              </div>
            ))}
          </div>

          {!selectedPolicyStoreId && (
            <div className="mt-4 p-4 bg-yellow-50 border border-yellow-200 rounded-md">
              <div className="flex items-center space-x-2">
                <AlertCircle className="w-5 h-5 text-yellow-600" />
                <p className="text-sm text-yellow-800">
                  Please select a policy store to view and edit its schema
                </p>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Schema Editor */}
      {selectedPolicyStoreId && (
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center space-x-2">
                  <FileText className="w-5 h-5" />
                  <span>Schema Editor</span>
                  {schemaData && (
                    <Badge variant="outline" className="text-xs">
                      Last updated: {new Date(schemaData.updated_at || schemaData.created_at).toLocaleString()}
                    </Badge>
                  )}
                </CardTitle>
                <CardDescription>
                  Define your entity schema in JSON format for policy store: {selectedPolicyStoreId}
                </CardDescription>
              </div>
              <div className="flex items-center space-x-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleValidate}
                  disabled={isValidating}
                  className="flex items-center space-x-2"
                >
                  <Play className="w-4 h-4" />
                  <span>{isValidating ? 'Validating...' : 'Validate'}</span>
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleReset}
                  disabled={!schemaData?.schema || hasUnsavedChanges}
                  className="flex items-center space-x-2"
                >
                  <RefreshCw className="w-4 h-4" />
                  <span>Reset</span>
                </Button>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            {/* Validation Status */}
            {validationData && (
              <div className={`mb-4 p-3 rounded-md ${
                validationData.valid
                  ? 'bg-green-50 border border-green-200'
                  : 'bg-red-50 border border-red-200'
              }`}>
                <div className="flex items-center space-x-2">
                  {validationData.valid ? (
                    <>
                      <CheckCircle className="w-5 h-5 text-green-600" />
                      <span className="font-medium text-green-800">
                        Schema is valid
                      </span>
                    </>
                  ) : (
                    <>
                      <XCircle className="w-5 h-5 text-red-600" />
                      <span className="font-medium text-red-800">
                        Schema has errors
                      </span>
                    </>
                  )}
                </div>
                {!validationData.valid && validationData.error && (
                  <ul className="mt-2 text-sm text-red-700 space-y-1">
                    <li className="flex items-start space-x-2">
                      <span className="text-red-500 mt-0.5">•</span>
                      <span>{validationData.error}</span>
                    </li>
                  </ul>
                )}
              </div>
            )}

            {/* Unsaved Changes Warning */}
            {hasUnsavedChanges && (
              <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-md">
                <div className="flex items-center space-x-2">
                  <AlertCircle className="w-5 h-5 text-yellow-600" />
                  <span className="text-sm text-yellow-800">
                    You have unsaved changes
                  </span>
                </div>
              </div>
            )}

            {/* Monaco Editor */}
            <div className="border border-gray-200 rounded-md overflow-hidden">
              <Editor
                height="500px"
                defaultLanguage="json"
                value={editorContent}
                onChange={handleEditorChange}
                theme="vs-dark"
                options={{
                  minimap: { enabled: false },
                  fontSize: 14,
                  lineNumbers: 'on',
                  roundedSelection: false,
                  scrollBeyondLastLine: false,
                  automaticLayout: true,
                  suggestOnTriggerCharacters: true,
                  quickSuggestions: true,
                }}
              />
            </div>

            {/* Schema Structure Guide */}
            <div className="mt-4 p-4 bg-gray-50 rounded-md">
              <h4 className="font-medium text-gray-900 mb-2">Schema Structure Guide</h4>
              <div className="text-sm text-gray-600 space-y-1">
                <p><code className="bg-gray-200 px-1 rounded">entity_type</code>: Name of the entity type (required)</p>
                <p><code className="bg-gray-200 px-1 rounded">attributes</code>: Object defining entity attributes (required)</p>
                <p>Each attribute must have:</p>
                <ul className="ml-4 space-y-1">
                  <li>• <code className="bg-gray-200 px-1 rounded">type</code>: string, number, boolean, array, or object</li>
                  <li>• <code className="bg-gray-200 px-1 rounded">required</code>: Whether the attribute is required (optional)</li>
                  <li>• <code className="bg-gray-200 px-1 rounded">description</code>: Human-readable description (optional)</li>
                  <li>• <code className="bg-gray-200 px-1 rounded">enum</code>: Array of allowed values (optional)</li>
                  <li>• <code className="bg-gray-200 px-1 rounded">default</code>: Default value (optional)</li>
                </ul>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default Schemas;
