'use client';

import React, { useState, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Layers,
  Plus,
  Save,
  FileText,
  Settings,
  Eye,
  Code,
  Users,
  Shield,
  Lock,
  FolderOpen,
  Search,
  Filter,
  Copy,
  Trash2,
  AlertCircle,
  CheckCircle,
} from 'lucide-react';

interface Template {
  policy_template_id: string;
  name: string;
  description?: string;
  created_at: string;
}

const TEMPLATE_CATEGORIES = [
  {
    id: 'access-control',
    name: 'Access Control',
    description: 'Basic and advanced access control patterns',
    icon: <Shield className="w-5 h-5" />,
    color: 'blue',
  },
  {
    id: 'rbac',
    name: 'Role-Based',
    description: 'Role-based access control templates',
    icon: <Users className="w-5 h-5" />,
    color: 'green',
  },
  {
    id: 'abac',
    name: 'Attribute-Based',
    description: 'Attribute-based access control patterns',
    icon: <Eye className="w-5 h-5" />,
    color: 'purple',
  },
  {
    id: 'resource-specific',
    name: 'Resource-Specific',
    description: 'Templates for specific resource types',
    icon: <FileText className="w-5 h-5" />,
    color: 'orange',
  },
  {
    id: 'security',
    name: 'Security',
    description: 'Security-focused policy templates',
    icon: <Lock className="w-5 h-5" />,
    color: 'red',
  },
  {
    id: 'custom',
    name: 'Custom',
    description: 'User-defined templates',
    icon: <Code className="w-5 h-5" />,
    color: 'gray',
  },
];

const Templates = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);

  // Mock data for templates - in real implementation, fetch from API
  const templates: Template[] = [
    {
      policy_template_id: 'tmpl-001',
      name: 'Basic User Access',
      description: 'Allow users to access their own resources',
      created_at: '2024-01-15T10:30:00Z',
    },
    {
      policy_template_id: 'tmpl-002',
      name: 'Admin Full Access',
      description: 'Grant full access to admin users',
      created_at: '2024-01-14T09:15:00Z',
    },
    {
      policy_template_id: 'tmpl-003',
      name: 'Document Owner Access',
      description: 'Allow document owners to read and edit their documents',
      created_at: '2024-01-13T14:22:00Z',
    },
    {
      policy_template_id: 'tmpl-004',
      name: 'Role-Based Department Access',
      description: 'Access based on user role and department',
      created_at: '2024-01-12T16:45:00Z',
    },
    {
      policy_template_id: 'tmpl-005',
      name: 'Time-Based Access',
      description: 'Access allowed only during business hours',
      created_at: '2024-01-11T11:20:00Z',
    },
  ];

  // Filter templates based on search and category
  const filteredTemplates = templates.filter((template) => {
    const matchesSearch =
      template.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (template.description || '').toLowerCase().includes(searchTerm.toLowerCase()) ||
      template.policy_template_id.toLowerCase().includes(searchTerm.toLowerCase());

    return matchesSearch;
  });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Policy Templates</h2>
          <p className="text-gray-600">
            Reusable policy templates for common authorization patterns
          </p>
        </div>
        <Button onClick={() => setIsCreateModalOpen(true)} className="flex items-center space-x-2">
          <Plus className="w-4 h-4" />
          <span>Create Template</span>
        </Button>
      </div>

      {/* Search and Filter */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col md:flex-row md:items-center md:justify-between space-y-4 md:space-y-0 md:space-x-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-4 h-4" />
              <input
                type="text"
                placeholder="Search templates..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <Button variant="outline" className="flex items-center space-x-2">
              <Filter className="w-4 h-4" />
              <span>Filter</span>
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Categories */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Layers className="w-5 h-5" />
            <span>Categories</span>
          </CardTitle>
          <CardDescription>Browse templates by category</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
            <button
              onClick={() => setSelectedCategory('all')}
              className={`p-4 border rounded-lg transition-colors ${
                selectedCategory === 'all'
                  ? 'border-blue-500 bg-blue-50 ring-2 ring-blue-200'
                  : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
              }`}
            >
              <FolderOpen className="w-6 h-6 mx-auto mb-2 text-gray-600" />
              <p className="text-sm font-medium text-gray-900">All</p>
              <p className="text-xs text-gray-600">{templates.length} templates</p>
            </button>

            {TEMPLATE_CATEGORIES.map((category) => (
              <button
                key={category.id}
                onClick={() => setSelectedCategory(category.id)}
                className={`p-4 border rounded-lg transition-colors ${
                  selectedCategory === category.id
                    ? 'border-blue-500 bg-blue-50 ring-2 ring-blue-200'
                    : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                }`}
              >
                <div className={`text-${category.color}-600 mb-2 flex justify-center`}>
                  {category.icon}
                </div>
                <p className="text-sm font-medium text-gray-900">{category.name}</p>
                <p className="text-xs text-gray-600">{category.description}</p>
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Templates Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredTemplates.map((template) => (
          <Card key={template.policy_template_id} className="hover:shadow-md transition-shadow">
            <CardHeader>
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <FileText className="w-5 h-5 text-blue-600" />
                    <Badge variant="outline">{template.policy_template_id}</Badge>
                  </div>
                  <CardTitle className="text-lg">{template.name}</CardTitle>
                  <CardDescription>{template.description}</CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-gray-600">Created</span>
                  <span className="text-sm text-gray-900">
                    {new Date(template.created_at).toLocaleDateString()}
                  </span>
                </div>
                <div className="flex items-center space-x-1 pt-2">
                  <Button variant="outline" size="sm" className="flex-1">
                    <Eye className="w-4 h-4 mr-2" />
                    View
                  </Button>
                  <Button variant="outline" size="sm" className="flex-1">
                    <Copy className="w-4 h-4 mr-2" />
                    Use
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Empty State */}
      {filteredTemplates.length === 0 && (
        <Card>
          <CardContent className="pt-6">
            <div className="text-center py-8">
              <Layers className="w-12 h-12 text-gray-400 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                {searchTerm ? 'No templates found' : 'No templates yet'}
              </h3>
              <p className="text-gray-600 mb-4">
                {searchTerm
                  ? 'Try adjusting your search terms or filters'
                  : 'Create your first policy template to get started'}
              </p>
              {!searchTerm && (
                <Button onClick={() => setIsCreateModalOpen(true)}>
                  Create Template
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Create Template Modal */}
      {isCreateModalOpen && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
            <h3 className="text-lg font-semibold mb-4">Create Policy Template</h3>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Template Name *
                </label>
                <input
                  type="text"
                  className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="e.g., Document Owner Access"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Description
                </label>
                <textarea
                  className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  rows={3}
                  placeholder="Describe what this template does..."
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Category *
                </label>
                <select className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
                  <option value="">Select a category</option>
                  {TEMPLATE_CATEGORIES.map((category) => (
                    <option key={category.id} value={category.id}>
                      {category.name}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Template Content (Cedar Policy)
                </label>
                <textarea
                  className="w-full p-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
                  rows={10}
                  placeholder="Enter Cedar policy template..."
                />
                <p className="text-xs text-gray-600 mt-1">
                  Use placeholders like {'{principal_id}'} for parameterized policies
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Parameters
                </label>
                <div className="space-y-2">
                  <div className="p-3 bg-gray-50 border border-gray-200 rounded-md">
                    <p className="text-sm text-gray-600">
                      No parameters defined. You can add parameters to make your template reusable.
                    </p>
                  </div>
                  <Button variant="outline" size="sm" className="w-full">
                    <Plus className="w-4 h-4 mr-2" />
                    Add Parameter
                  </Button>
                </div>
              </div>
            </div>

            <div className="flex justify-end space-x-2 mt-6">
              <Button
                variant="outline"
                onClick={() => setIsCreateModalOpen(false)}
                disabled={false}
              >
                Cancel
              </Button>
              <Button disabled={false} className="flex items-center space-x-2">
                <Save className="w-4 h-4" />
                <span>Create Template</span>
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Templates;
