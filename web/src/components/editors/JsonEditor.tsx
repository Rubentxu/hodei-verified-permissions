/**
 * JsonEditor Component - Specialized JSON editor
 */

import React, { useCallback } from 'react';
import { CodeEditor, type CodeEditorProps } from './CodeEditor';
import { isValidJSON } from '../../utils/validators';
import { Alert } from '../common/Alert';

export interface JsonEditorProps extends Omit<CodeEditorProps, 'language'> {
  onValidationChange?: (isValid: boolean) => void;
}

export const JsonEditor = React.forwardRef<HTMLDivElement, JsonEditorProps>(
  ({ value, onChange, onValidationChange, ...props }, ref) => {
    const [error, setError] = React.useState<string | null>(null);

    const handleChange = useCallback(
      (newValue: string) => {
        const isValid = isValidJSON(newValue);

        if (!isValid && newValue.trim()) {
          setError('Invalid JSON format');
          onValidationChange?.(false);
        } else {
          setError(null);
          onValidationChange?.(true);
        }

        onChange?.(newValue);
      },
      [onChange, onValidationChange]
    );

    return (
      <div ref={ref}>
        <CodeEditor
          {...props}
          value={value}
          onChange={handleChange}
          language="json"
        />
        {error && (
          <Alert
            type="error"
            message={error}
            className="mt-2"
            closeable={false}
          />
        )}
      </div>
    );
  }
);

JsonEditor.displayName = 'JsonEditor';
