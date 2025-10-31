'use client';

import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Users,
  Plus,
  Settings as SettingsIcon,
  CheckCircle,
  XCircle,
  AlertCircle,
  RefreshCw,
  Trash2,
  Edit,
  Eye,
  EyeOff,
  Key,
  Shield,
} from 'lucide-react';
import { useIdentitySources, useCreateIdentitySource, useTestIdentitySourceConnection, useDeleteIdentitySource } from '@/hooks/useIdentitySources';
import { usePolicyStores } from '@/hooks/usePolicyStores';

interface CreateCognitoModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: {
    policy_store_id: string;
    config: {
      cognito: {
        user_pool_id: string;
        region: string;
        client_id: string;
        client_secret: string;
      };
    };
  }) => void;
  policyStoreId: string;
  isLoading: boolean;
}

const CreateCognitoModal: React.FC<CreateCognitoModalProps> = ({
  isOpen,
  onClose,
  onSubmit,
  policyStoreId,
  isLoading,
}) => {
  const [formData, setFormData] = useState({
    user_pool_id: '',
    region: '',
    client_id: '',
    client_secret: '',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      policy_store_id: policyStoreId,
      config: { cognito: formData },
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <h3 className="text-lg font-semibold mb-4">Configure Amazon Cognito</h3>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                User Pool ID *
              </label>
              <input
                type="text"
                value={formData.user_pool_id}
                onChange={(e) => setFormData({ ...formData, user_pool_id: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="us-east-1_XXXXXXX"
                required
              />
              <p className="text-xs text-gray-600 mt-1">Example: us-east-1_123456789</p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Region *
              </label>
              <input
                type="text"
                value={formData.region}
                onChange={(e) => setFormData({ ...formData, region: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="us-east-1"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                App Client ID *
              </label>
              <input
                type="text"
                value={formData.client_id}
                onChange={(e) => setFormData({ ...formData, client_id: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="xxxxxxxxxxxxxxxxxx"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                App Client Secret *
              </label>
              <input
                type="password"
                value={formData.client_secret}
                onChange={(e) => setFormData({ ...formData, client_secret: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                required
              />
            </div>
          </div>

          <div className="flex justify-end space-x-2 mt-6">
            <Button type="button" variant="outline" onClick={onClose} disabled={isLoading}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? 'Creating...' : 'Create Identity Source'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

interface CreateOIDCIDModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: {
    policy_store_id: string;
    config: {
      oidc: {
        issuer: string;
        client_id: string;
        client_secret: string;
        authorization_endpoint: string;
        token_endpoint: string;
        userinfo_endpoint: string;
        scopes: string[];
      };
    };
  }) => void;
  policyStoreId: string;
  isLoading: boolean;
}

const CreateOIDCIDModal: React.FC<CreateOIDCIDModalProps> = ({
  isOpen,
  onClose,
  onSubmit,
  policyStoreId,
  isLoading,
}) => {
  const [formData, setFormData] = useState({
    issuer: '',
    client_id: '',
    client_secret: '',
    authorization_endpoint: '',
    token_endpoint: '',
    userinfo_endpoint: '',
    scopes: 'openid,profile,email',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({
      policy_store_id: policyStoreId,
      config: {
        oidc: {
          ...formData,
          scopes: formData.scopes.split(',').map(s => s.trim()),
        },
      },
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <h3 className="text-lg font-semibold mb-4">Configure OIDC Provider</h3>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Issuer URL *
              </label>
              <input
                type="text"
                value={formData.issuer}
                onChange={(e) => setFormData({ ...formData, issuer: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="https://your-provider.com"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Client ID *
              </label>
              <input
                type="text"
                value={formData.client_id}
                onChange={(e) => setFormData({ ...formData, client_id: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Client Secret *
              </label>
              <input
                type="password"
                value={formData.client_secret}
                onChange={(e) => setFormData({ ...formData, client_secret: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Authorization Endpoint *
              </label>
              <input
                type="text"
                value={formData.authorization_endpoint}
                onChange={(e) => setFormData({ ...formData, authorization_endpoint: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="https://provider.com/oauth/authorize"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Token Endpoint *
              </label>
              <input
                type="text"
                value={formData.token_endpoint}
                onChange={(e) => setFormData({ ...formData, token_endpoint: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="https://provider.com/oauth/token"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                UserInfo Endpoint *
              </label>
              <input
                type="text"
                value={formData.userinfo_endpoint}
                onChange={(e) => setFormData({ ...formData, userinfo_endpoint: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="https://provider.com/oauth/userinfo"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Scopes
              </label>
              <input
                type="text"
                value={formData.scopes}
                onChange={(e) => setFormData({ ...formData, scopes: e.target.value })}
                className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="openid,profile,email"
              />
              <p className="text-xs text-gray-600 mt-1">Comma-separated list of scopes</p>
            </div>
          </div>

          <div className="flex justify-end space-x-2 mt-6">
            <Button type="button" variant="outline" onClick={onClose} disabled={isLoading}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? 'Creating...' : 'Create Identity Source'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};

const IdentitySources = () => {
  const [selectedPolicyStoreId, setSelectedPolicyStoreId] = useState<string>('');
  const [isCognitoModalOpen, setIsCognitoModalOpen] = useState(false);
  const [isOIDCIDModalOpen, setIsOIDCIDModalOpen] = useState(false);

  // Fetch policy stores
  const { data: policyStoresData } = usePolicyStores();

  // Fetch identity sources for selected store
  const { data: identitySourcesData, isLoading: isLoadingSources, error: sourcesError, refetch: refetchSources } = useIdentitySources({
    policy_store_id: selectedPolicyStoreId,
  });

  // Mutations
  const createIdentitySourceMutation = useCreateIdentitySource();
  const testConnectionMutation = useTestIdentitySourceConnection();
  const deleteIdentitySourceMutation = useDeleteIdentitySource();

  const identitySources = identitySourcesData?.identity_sources || [];

  const handleCreateCognito = async (data: any) => {
    try {
      await createIdentitySourceMutation.mutateAsync(data);
      setIsCognitoModalOpen(false);
    } catch (error) {
      console.error('Failed to create Cognito identity source:', error);
    }
  };

  const handleCreateOIDCID = async (data: any) => {
    try {
      await createIdentitySourceMutation.mutateAsync(data);
      setIsOIDCIDModalOpen(false);
    } catch (error) {
      console.error('Failed to create OIDC identity source:', error);
    }
  };

  const handleTestConnection = async (identitySourceId: string) => {
    try {
      await testConnectionMutation.mutateAsync({
        policy_store_id: selectedPolicyStoreId,
        identity_source_id: identitySourceId,
      });
      alert('Connection test successful!');
    } catch (error) {
      console.error('Connection test failed:', error);
      alert('Connection test failed. Please check your configuration.');
    }
  };

  const handleDeleteIdentitySource = async (identitySourceId: string) => {
    if (window.confirm('Are you sure you want to delete this identity source?')) {
      try {
        await deleteIdentitySourceMutation.mutateAsync({
          policy_store_id: selectedPolicyStoreId,
          identity_source_id: identitySourceId,
        });
      } catch (error) {
        console.error('Failed to delete identity source:', error);
      }
    }
  };

  // Loading state
  if (isLoadingSources) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <div className="h-8 w-48 bg-gray-200 rounded animate-pulse mb-2"></div>
            <div className="h-4 w-96 bg-gray-200 rounded animate-pulse"></div>
          </div>
          <div className="h-10 w-44 bg-gray-200 rounded animate-pulse"></div>
        </div>

        <Card>
          <CardHeader>
            <div className="h-6 w-64 bg-gray-200 rounded animate-pulse"></div>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {[...Array(3)].map((_, i) => (
                <div key={i} className="h-20 bg-gray-200 rounded animate-pulse"></div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Error state
  if (sourcesError && selectedPolicyStoreId) {
    return (
      <div className="flex items-center justify-center h-64">
        <Card className="max-w-md">
          <CardContent className="pt-6">
            <div className="text-center">
              <AlertCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                Failed to load identity sources
              </h3>
              <p className="text-gray-600 mb-4">
                {sourcesError.message || 'An error occurred while fetching identity sources'}
              </p>
              <Button onClick={() => refetchSources()}>
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
          <h2 className="text-2xl font-bold text-gray-900">Identity Sources</h2>
          <p className="text-gray-600">
            Configure identity providers for JWT token validation
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            onClick={() => setIsCognitoModalOpen(true)}
            disabled={!selectedPolicyStoreId}
            className="flex items-center space-x-2"
          >
            <Shield className="w-4 h-4" />
            <span>Add Cognito</span>
          </Button>
          <Button
            onClick={() => setIsOIDCIDModalOpen(true)}
            disabled={!selectedPolicyStoreId}
            variant="outline"
            className="flex items-center space-x-2"
          >
            <Key className="w-4 h-4" />
            <span>Add OIDC</span>
          </Button>
        </div>
      </div>

      {/* Policy Store Selector */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Users className="w-5 h-5" />
            <span>Select Policy Store</span>
          </CardTitle>
          <CardDescription>Choose a policy store to configure identity sources</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {policyStoresData?.policy_stores.map((store) => (
              <div
                key={store.policy_store_id}
                className={`p-4 border rounded-lg cursor-pointer transition-all ${
                  selectedPolicyStoreId === store.policy_store_id
                    ? 'border-blue-500 bg-blue-50 ring-2 ring-blue-200'
                    : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                }`}
                onClick={() => setSelectedPolicyStoreId(store.policy_store_id)}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <h4 className="font-medium text-gray-900 mb-1">
                      {store.policy_store_id}
                    </h4>
                    <p className="text-sm text-gray-600 mb-2">
                      {store.description || 'No description'}
                    </p>
                    <Badge variant="outline" className="text-xs">
                      Created: {new Date(store.created_at).toLocaleDateString()}
                    </Badge>
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
                  Please select a policy store to configure identity sources
                </p>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Identity Sources List */}
      {selectedPolicyStoreId && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <SettingsIcon className="w-5 h-5" />
              <span>Identity Sources for {selectedPolicyStoreId}</span>
            </CardTitle>
            <CardDescription>Manage identity providers for this policy store</CardDescription>
          </CardHeader>
          <CardContent>
            {identitySources.length === 0 ? (
              <div className="text-center py-8">
                <Users className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">
                  No identity sources configured
                </h3>
                <p className="text-gray-600 mb-4">
                  Add a Cognito user pool or OIDC provider to get started
                </p>
                <div className="flex justify-center space-x-2">
                  <Button
                    onClick={() => setIsCognitoModalOpen(true)}
                    size="sm"
                    className="flex items-center space-x-2"
                  >
                    <Shield className="w-4 h-4" />
                    <span>Add Cognito</span>
                  </Button>
                  <Button
                    onClick={() => setIsOIDCIDModalOpen(true)}
                    size="sm"
                    variant="outline"
                    className="flex items-center space-x-2"
                  >
                    <Key className="w-4 h-4" />
                    <span>Add OIDC</span>
                  </Button>
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {identitySources.map((source: any) => (
                  <div
                    key={source.identity_source_id}
                    className="p-4 border border-gray-200 rounded-md hover:border-gray-300 hover:bg-gray-50 transition-colors"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <div className="flex items-center space-x-2 mb-1">
                          <h4 className="font-medium text-gray-900">
                            {source.identity_source_id}
                          </h4>
                          <Badge variant="outline">
                            {source.config.cognito ? 'Cognito' : 'OIDC'}
                          </Badge>
                          <Badge
                            variant={source.status === 'active' ? 'default' : 'secondary'}
                          >
                            {source.status}
                          </Badge>
                        </div>
                        <p className="text-sm text-gray-600">
                          Created: {new Date(source.created_at).toLocaleDateString()}
                        </p>
                      </div>
                      <div className="flex items-center space-x-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleTestConnection(source.identity_source_id)}
                          disabled={testConnectionMutation.isPending}
                        >
                          <RefreshCw className="w-4 h-4" />
                        </Button>
                        <Button variant="ghost" size="sm">
                          <Edit className="w-4 h-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDeleteIdentitySource(source.identity_source_id)}
                          disabled={deleteIdentitySourceMutation.isPending}
                        >
                          <Trash2 className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Modals */}
      <CreateCognitoModal
        isOpen={isCognitoModalOpen}
        onClose={() => setIsCognitoModalOpen(false)}
        onSubmit={handleCreateCognito}
        policyStoreId={selectedPolicyStoreId}
        isLoading={createIdentitySourceMutation.isPending}
      />

      <CreateOIDCIDModal
        isOpen={isOIDCIDModalOpen}
        onClose={() => setIsOIDCIDModalOpen(false)}
        onSubmit={handleCreateOIDCID}
        policyStoreId={selectedPolicyStoreId}
        isLoading={createIdentitySourceMutation.isPending}
      />
    </div>
  );
};

export default IdentitySources;
