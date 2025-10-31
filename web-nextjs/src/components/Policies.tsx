'use client';

import React, { useState, useCallback } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Code,
  Plus,
  Save,
  Play,
  CheckCircle,
  XCircle,
  AlertCircle,
  Trash2,
  Edit,
  Copy,
  FileText,
  Settings,
  Eye,
  ArrowLeft,
  ArrowRight,
} from 'lucide-react';
import Editor from '@monaco-editor/react';
import { usePolicies, useCreatePolicy } from '@/hooks/usePolicies';
import { usePolicyStores } from '@/hooks/usePolicyStores';

const DEFAULT_POLICY_TEMPLATE = `permit(
  principal: User,
  action: Action,
  resource: Document
) when {
  resource.owner == principal.id
};`;

interface WizardData {
  name: string;
  description: string;
  policy_store_id: string;
  template: 'basic' | 'rbac' | 'abac' | 'custom';
  entities: string[];
  content: string;
}

// Wizard Step 1: Basic Information
const Step1BasicInfo = ({
  data,
  onChange,
}: {
  data: WizardData;
  onChange: (data: Partial<WizardData>) => void;
}) => {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Policy Name *
        </label>
        <input
          type="text"
          value={data.name}
          onChange={(e) => onChange({ name: e.target.value })}
          className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          placeholder="e.g., Document Access Policy"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Description *
        </label>
        <textarea
          value={data.description}
          onChange={(e) => onChange({ description: e.target.value })}
          className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          rows={3}
          placeholder="Describe what this policy controls..."
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Policy Store *
        </label>
        <select
          value={data.policy_store_id}
          onChange={(e) => onChange({ policy_store_id: e.target.value })}
          className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="">Select a policy store</option>
          {/* Options will be populated by parent component */}
        </select>
      </div>
    </div>
  );
};

// Wizard Step 2: Template Selection
const Step2Template = ({
  data,
  onChange,
}: {
  data: WizardData;
  onChange: (data: Partial<WizardData>) => void;
}) => {
  const templates = [
    {
      id: 'basic' as const,
      name: 'Basic Access Control',
      description: 'Simple permit/forbid rules for basic access control',
      icon: <FileText className="w-6 h-6" />,
    },
    {
      id: 'rbac' as const,
      name: 'Role-Based Access',
      description: 'Role-based access control with user roles and permissions',
      icon: <Settings className="w-6 h-6" />,
    },
    {
      id: 'abac' as const,
      name: 'Attribute-Based',
      description: 'Complex rules based on entity attributes and relationships',
      icon: <Eye className="w-6 h-6" />,
    },
    {
      id: 'custom' as const,
      name: 'Custom Policy',
      description: 'Start with a blank policy template',
      icon: <Code className="w-6 h-6" />,
    },
  ];

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {templates.map((template) => (
          <div
            key={template.id}
            className={`p-4 border rounded-md cursor-pointer transition-colors ${
              data.template === template.id
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
            }`}
            onClick={() => onChange({ template: template.id })}
          >
            <div className="flex items-center space-x-3">
              <div className="text-blue-600">{template.icon}</div>
              <div>
                <h4 className="font-medium text-gray-900">{template.name}</h4>
                <p className="text-sm text-gray-600">{template.description}</p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

// Wizard Step 3: Entity Configuration
const Step3Entities = ({
  data,
  onChange,
}: {
  data: WizardData;
  onChange: (data: Partial<WizardData>) => void;
}) => {
  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Entity Types
        </label>
        <div className="space-y-2">
          {['User', 'Action', 'Resource', 'Document'].map((entity) => (
            <label key={entity} className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={data.entities.includes(entity)}
                onChange={(e) => {
                  if (e.target.checked) {
                    onChange({ entities: [...data.entities, entity] });
                  } else {
                    onChange({ entities: data.entities.filter((e) => e !== entity) });
                  }
                }}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span className="text-sm text-gray-700">{entity}</span>
            </label>
          ))}
        </div>
      </div>
      <div className="p-4 bg-gray-50 rounded-md">
        <h4 className="font-medium text-gray-900 mb-2">Entity Types Guide</h4>
        <div className="text-sm text-gray-600 space-y-1">
          <p><strong>User:</strong> People or services that perform actions</p>
          <p><strong>Action:</strong> Operations that can be performed (view, edit, delete)</p>
          <p><strong>Resource:</strong> Objects that actions are performed on</p>
          <p><strong>Document:</strong> Specific type of resource with additional attributes</p>
        </div>
      </div>
    </div>
  );
};

// Wizard Step 4: Review
const Step4Review = ({ data }: { data: WizardData }) => {
  return (
    <div className="space-y-4">
      <div className="p-4 bg-gray-50 rounded-md">
        <h4 className="font-medium text-gray-900 mb-2">Policy Summary</h4>
        <dl className="space-y-2">
          <div>
            <dt className="text-sm font-medium text-gray-700">Name</dt>
            <dd className="text-sm text-gray-900">{data.name}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-700">Description</dt>
            <dd className="text-sm text-gray-900">{data.description}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-700">Policy Store</dt>
            <dd className="text-sm text-gray-900">{data.policy_store_id}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-700">Template</dt>
            <dd className="text-sm text-gray-900">{data.template}</dd>
          </div>
          <div>
            <dt className="text-sm font-medium text-gray-700">Entities</dt>
            <dd className="text-sm text-gray-900">{data.entities.join(', ') || 'None'}</dd>
          </div>
        </dl>
      </div>
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-md">
        <div className="flex items-center space-x-2">
          <CheckCircle className="w-5 h-5 text-blue-600" />
          <span className="text-sm font-medium text-blue-900">
            Ready to create policy
          </span>
        </div>
      </div>
    </div>
  );
};

const Policies = () => {
  const [selectedPolicyStoreId, setSelectedPolicyStoreId] = useState<string>('');
  const [editorContent, setEditorContent] = useState(DEFAULT_POLICY_TEMPLATE);
  const [isWizardOpen, setIsWizardOpen] = useState(false);
  const [currentStep, setCurrentStep] = useState(0);
  const [wizardData, setWizardData] = useState<WizardData>({
    name: '',
    description: '',
    policy_store_id: '',
    template: 'basic',
    entities: ['User', 'Action', 'Resource', 'Document'],
    content: DEFAULT_POLICY_TEMPLATE,
  });

  // Fetch policy stores
  const { data: policyStoresData } = usePolicyStores();

  // Fetch policies for selected store
  const { data: policiesData, isLoading: isLoadingPolicies, error: policiesError } = usePolicies({
    policy_store_id: selectedPolicyStoreId,
  });

  // Create policy mutation
  const createPolicyMutation = useCreatePolicy();

  const policies = policiesData?.policies || [];

  const wizardSteps = [
    { id: 'basic', title: 'Basic Information', component: Step1BasicInfo },
    { id: 'template', title: 'Choose Template', component: Step2Template },
    { id: 'entities', title: 'Entity Configuration', component: Step3Entities },
    { id: 'review', title: 'Review & Create', component: Step4Review },
  ];

  const handleWizardNext = () => {
    if (currentStep < wizardSteps.length - 1) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handleWizardBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleWizardSubmit = async () => {
    try {
      const policyDefinition = {
        static_policy: {
          description: wizardData.description,
          statement: wizardData.content,
          applies_to: wizardData.entities.map((entity) => ({
            resource_type: entity,
          })),
        },
      };

      await createPolicyMutation.mutateAsync({
        policy_store_id: wizardData.policy_store_id,
        definition: policyDefinition,
      });

      setIsWizardOpen(false);
      setCurrentStep(0);
      setWizardData({
        name: '',
        description: '',
        policy_store_id: '',
        template: 'basic',
        entities: ['User', 'Action', 'Resource', 'Document'],
        content: DEFAULT_POLICY_TEMPLATE,
      });
    } catch (error) {
      console.error('Failed to create policy:', error);
    }
  };

  const CurrentStepComponent = wizardSteps[currentStep].component;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Policies</h2>
          <p className="text-gray-600">Create and manage Cedar policies for your policy stores</p>
        </div>
        <Button
          onClick={() => setIsWizardOpen(true)}
          className="flex items-center space-x-2"
        >
          <Plus className="w-4 h-4" />
          <span>Create Policy</span>
        </Button>
      </div>

      {/* Policy Store Selector */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Code className="w-5 h-5" />
            <span>Select Policy Store</span>
          </CardTitle>
          <CardDescription>Choose a policy store to view and manage its policies</CardDescription>
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
        </CardContent>
      </Card>

      {/* Policies List */}
      {selectedPolicyStoreId && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <FileText className="w-5 h-5" />
              <span>Policies for {selectedPolicyStoreId}</span>
            </CardTitle>
            <CardDescription>Manage policies in this policy store</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoadingPolicies ? (
              <div className="space-y-2">
                {[...Array(5)].map((_, i) => (
                  <div key={i} className="h-16 bg-gray-200 rounded animate-pulse"></div>
                ))}
              </div>
            ) : policies.length === 0 ? (
              <div className="text-center py-8">
                <FileText className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">No policies yet</h3>
                <p className="text-gray-600 mb-4">
                  Create your first policy to get started
                </p>
                <Button onClick={() => setIsWizardOpen(true)} variant="outline" size="sm">
                  Create Policy
                </Button>
              </div>
            ) : (
              <div className="space-y-2">
                {policies.map((policy) => (
                  <div
                    key={policy.policy_id}
                    className="p-4 border border-gray-200 rounded-md hover:border-gray-300 hover:bg-gray-50 transition-colors"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <div className="flex items-center space-x-2 mb-1">
                          <h4 className="font-medium text-gray-900">
                            {policy.policy_id}
                          </h4>
                          <Badge variant="outline">Active</Badge>
                        </div>
                        <p className="text-sm text-gray-600">
                          Created: {new Date(policy.created_at).toLocaleDateString()}
                        </p>
                      </div>
                      <div className="flex items-center space-x-1">
                        <Button variant="ghost" size="sm">
                          <Eye className="w-4 h-4" />
                        </Button>
                        <Button variant="ghost" size="sm">
                          <Edit className="w-4 h-4" />
                        </Button>
                        <Button variant="ghost" size="sm">
                          <Copy className="w-4 h-4" />
                        </Button>
                        <Button variant="ghost" size="sm">
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

      {/* Wizard Modal */}
      {isWizardOpen && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-3xl max-h-[90vh] overflow-y-auto">
            <div className="mb-6">
              <h3 className="text-xl font-semibold mb-2">Create New Policy</h3>
              <div className="flex items-center space-x-2">
                {wizardSteps.map((step, index) => (
                  <React.Fragment key={step.id}>
                    <div
                      className={`flex items-center space-x-2 ${
                        index <= currentStep ? 'text-blue-600' : 'text-gray-400'
                      }`}
                    >
                      <div
                        className={`w-8 h-8 rounded-full flex items-center justify-center ${
                          index <= currentStep
                            ? 'bg-blue-600 text-white'
                            : 'bg-gray-200 text-gray-600'
                        }`}
                      >
                        {index + 1}
                      </div>
                      <span className="text-sm font-medium hidden md:inline">
                        {step.title}
                      </span>
                    </div>
                    {index < wizardSteps.length - 1 && (
                      <div className="flex-1 h-px bg-gray-200"></div>
                    )}
                  </React.Fragment>
                ))}
              </div>
            </div>

            <div className="mb-6">
              <h4 className="text-lg font-medium mb-2">{wizardSteps[currentStep].title}</h4>
              <p className="text-sm text-gray-600 mb-4">
                {currentStep === 0 && 'Configure basic policy information'}
                {currentStep === 1 && 'Choose a starting template for your policy'}
                {currentStep === 2 && 'Define the entities used in your policy'}
                {currentStep === 3 && 'Review your policy configuration'}
              </p>

              {/* Step 1 needs policy stores data */}
              {currentStep === 0 ? (
                <Step1BasicInfo
                  data={wizardData}
                  onChange={(data) => setWizardData({ ...wizardData, ...data })}
                />
              ) : currentStep === 1 ? (
                <Step2Template
                  data={wizardData}
                  onChange={(data) => setWizardData({ ...wizardData, ...data })}
                />
              ) : currentStep === 2 ? (
                <Step3Entities
                  data={wizardData}
                  onChange={(data) => setWizardData({ ...wizardData, ...data })}
                />
              ) : (
                <Step4Review data={wizardData} />
              )}
            </div>

            <div className="flex justify-between">
              <Button
                variant="outline"
                onClick={handleWizardBack}
                disabled={currentStep === 0}
                className="flex items-center space-x-2"
              >
                <ArrowLeft className="w-4 h-4" />
                <span>Back</span>
              </Button>

              {currentStep < wizardSteps.length - 1 ? (
                <Button
                  onClick={handleWizardNext}
                  className="flex items-center space-x-2"
                >
                  <span>Next</span>
                  <ArrowRight className="w-4 h-4" />
                </Button>
              ) : (
                <Button
                  onClick={handleWizardSubmit}
                  disabled={createPolicyMutation.isPending}
                  className="flex items-center space-x-2"
                >
                  <Save className="w-4 h-4" />
                  <span>{createPolicyMutation.isPending ? 'Creating...' : 'Create Policy'}</span>
                </Button>
              )}
            </div>

            <Button
              variant="ghost"
              onClick={() => setIsWizardOpen(false)}
              className="absolute top-4 right-4"
            >
              ✕
            </Button>
          </div>
        </div>
      )}
    </div>
  );
};

export default Policies;
