/**
 * AuthorizationResult Component - Display authorization test results
 */

import React from 'react';
import { AuthorizationResponse } from '../../../types';
import { Card, CardContent, CardHeader, CardTitle, Alert } from '../../../components';
import { CheckCircle, XCircle } from 'lucide-react';

export interface AuthorizationResultProps {
  result?: AuthorizationResponse;
  isLoading?: boolean;
}

export const AuthorizationResult: React.FC<AuthorizationResultProps> = ({
  result,
  isLoading,
}) => {
  if (isLoading) {
    return (
      <Card>
        <CardContent className="py-8 text-center">
          <p className="text-gray-500">Evaluating...</p>
        </CardContent>
      </Card>
    );
  }

  if (!result) {
    return null;
  }

  const isAllow = result.decision === 'ALLOW';

  return (
    <div className="space-y-4">
      {/* Decision */}
      <Card className={isAllow ? 'border-green-200 bg-green-50' : 'border-red-200 bg-red-50'}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            {isAllow ? (
              <>
                <CheckCircle className="h-6 w-6 text-green-600" />
                <span className="text-green-900">ALLOW</span>
              </>
            ) : (
              <>
                <XCircle className="h-6 w-6 text-red-600" />
                <span className="text-red-900">DENY</span>
              </>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className={isAllow ? 'text-green-800' : 'text-red-800'}>
            {isAllow
              ? 'The authorization request was approved.'
              : 'The authorization request was denied.'}
          </p>
        </CardContent>
      </Card>

      {/* Determining Policies */}
      {result.determiningPolicies.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Determining Policies</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {result.determiningPolicies.map((policy, idx) => (
                <div
                  key={idx}
                  className="p-3 bg-gray-50 rounded-md border border-gray-200 font-mono text-sm"
                >
                  {policy}
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Errors */}
      {result.errors.length > 0 && (
        <div className="space-y-2">
          {result.errors.map((error, idx) => (
            <Alert
              key={idx}
              type="error"
              message={error}
              closeable={false}
            />
          ))}
        </div>
      )}
    </div>
  );
};
