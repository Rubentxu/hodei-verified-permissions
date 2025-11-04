import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { format } from 'date-fns';

interface AuditLogEntry {
  id: number;
  policy_store_id: string;
  action: string;
  user_id: string;
  changes: string | null;
  ip_address: string | null;
  timestamp: string;
}

interface AuditLogProps {
  policyStoreId: string;
}

export default function AuditLog({ policyStoreId }: AuditLogProps) {
  const [eventTypes, setEventTypes] = useState<string[]>([]);
  const [maxResults, setMaxResults] = useState(100);

  // Fetch audit log data
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ['auditLog', policyStoreId, eventTypes, maxResults],
    queryFn: async () => {
      const params = new URLSearchParams();
      if (eventTypes.length > 0) {
        params.append('event_types', eventTypes.join(','));
      }
      if (maxResults > 0) {
        params.append('max_results', maxResults.toString());
      }

      const response = await fetch(`/api/policy-stores/${policyStoreId}/audit?${params}`);
      if (!response.ok) {
        throw new Error('Failed to fetch audit log');
      }
      return response.json();
    },
    enabled: !!policyStoreId,
  });

  const handleExport = async () => {
    const response = await fetch(`/api/policy-stores/${policyStoreId}/audit/export`, {
      method: 'POST',
    });

    if (response.ok) {
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `audit-log-${policyStoreId}-${Date.now()}.json`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    }
  };

  if (!policyStoreId) {
    return (
      <div className="flex items-center justify-center h-64 bg-gray-50 rounded-lg">
        <p className="text-gray-500">Select a policy store to view audit log</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-gray-900">Audit Log</h2>
        <div className="flex space-x-2">
          <button
            onClick={refetch}
            className="px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            Refresh
          </button>
          <button
            onClick={handleExport}
            className="px-3 py-1 text-sm bg-green-500 text-white rounded hover:bg-green-600"
          >
            Export
          </button>
        </div>
      </div>

      <div className="bg-white p-4 rounded-lg shadow">
        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Event Types
          </label>
          <select
            multiple
            value={eventTypes}
            onChange={(e) => {
              const values = Array.from(e.target.selectedOptions, option => option.value);
              setEventTypes(values);
            }}
            className="w-full border border-gray-300 rounded-md px-3 py-2"
          >
            <option value="ApiCalled">API Called</option>
            <option value="ApiCompleted">API Completed</option>
            <option value="PolicyStoreCreated">Policy Store Created</option>
            <option value="PolicyStoreUpdated">Policy Store Updated</option>
            <option value="PolicyStoreTagsUpdated">Policy Store Tags Updated</option>
            <option value="PolicyStoreDeleted">Policy Store Deleted</option>
            <option value="PolicyStoreAccessed">Policy Store Accessed</option>
            <option value="AuthorizationPerformed">Authorization Performed</option>
          </select>
        </div>

        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Max Results
          </label>
          <input
            type="number"
            value={maxResults}
            onChange={(e) => setMaxResults(parseInt(e.target.value))}
            className="w-full border border-gray-300 rounded-md px-3 py-2"
            min="1"
            max="1000"
          />
        </div>
      </div>

      <div className="bg-white rounded-lg shadow overflow-hidden">
        {isLoading ? (
          <div className="p-8 text-center text-gray-500">Loading audit log...</div>
        ) : error ? (
          <div className="p-8 text-center text-red-500">
            Error loading audit log: {error instanceof Error ? error.message : 'Unknown error'}
          </div>
        ) : !data || data.log_entries.length === 0 ? (
          <div className="p-8 text-center text-gray-500">No audit entries found</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Timestamp
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Event Type
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    User
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Details
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {data.log_entries.map((entry: AuditLogEntry, index: number) => (
                  <tr key={index} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {format(new Date(entry.timestamp), 'yyyy-MM-dd HH:mm:ss')}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`px-2 py-1 text-xs font-semibold rounded-full ${
                        entry.action.includes('Created') ? 'bg-green-100 text-green-800' :
                        entry.action.includes('Updated') ? 'bg-blue-100 text-blue-800' :
                        entry.action.includes('Deleted') ? 'bg-red-100 text-red-800' :
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {entry.action}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {entry.user_id}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-500 max-w-md truncate">
                      {entry.changes || '-'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
