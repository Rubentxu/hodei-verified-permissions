/**
 * SchemaEditor Component - Edit Cedar schema with validation
 */

import React, { useState, useCallback, useEffect } from 'react';
import { JsonEditor } from '../../../components/editors';
import { Card, CardContent, CardHeader, CardTitle, Alert } from '../../../components';
import { isValidSchema } from '../../../utils/validators';

export interface SchemaEditorProps {
  value: string;
  onChange?: (value: string) => void;
  onValidationChange?: (isValid: boolean) => void;
  readOnly?: boolean;
  height?: string;
  error?: string;
}

export const SchemaEditor = React.forwardRef<HTMLDivElement, SchemaEditorProps>(
  (
    {
      value,
      onChange,
      onValidationChange,
      readOnly = false,
      height = '500px',
      error,
    },
    ref
  ) => {
    const [validationError, setValidationError] = useState<string | null>(null);

    const handleChange = useCallback(
      (newValue: string) => {
        onChange?.(newValue);

        // Validate schema structure
        if (newValue.trim()) {
          const isValid = isValidSchema(newValue);
          if (!isValid) {
            setValidationError('Invalid Cedar schema structure');
            onValidationChange?.(false);
          } else {
            setValidationError(null);
            onValidationChange?.(true);
          }
        }
      },
      [onChange, onValidationChange]
    );

    // Validate on initial mount and when external value changes
    useEffect(() => {
      if (value !== undefined) {
        if (value.trim()) {
          const ok = isValidSchema(value);
          if (!ok) {
            setValidationError('Invalid Cedar schema structure');
            onValidationChange?.(false);
          } else {
            setValidationError(null);
            onValidationChange?.(true);
          }
        } else {
          // Empty value considered neutral; do not set error, but notify false
          onValidationChange?.(false);
        }
      }
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [value]);

    return (
      <div ref={ref} className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>Schema Definition</CardTitle>
          </CardHeader>
          <CardContent>
            <JsonEditor
              value={value}
              onChange={handleChange}
              readOnly={readOnly}
              height={height}
              onValidationChange={(isValid) => {
                if (isValid) {
                  setValidationError(null);
                  onValidationChange?.(true);
                }
              }}
            />
          </CardContent>
        </Card>

        {(error || validationError) && (
          <Alert
            type="error"
            title="Schema Error"
            message={error || validationError || 'Invalid schema'}
            closeable={false}
          />
        )}

        <div className="text-sm text-gray-600 bg-blue-50 p-4 rounded-lg">
          <p className="font-semibold mb-2">Schema Format:</p>
          <ul className="list-disc list-inside space-y-1">
            <li>Define entity types with their attributes</li>
            <li>Define actions and their applicability</li>
            <li>Use valid JSON format</li>
            <li>Cedar validates the schema structure</li>
          </ul>
        </div>
      </div>
    );
  }
);

SchemaEditor.displayName = 'SchemaEditor';
