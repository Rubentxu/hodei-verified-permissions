/**
 * CedarEditor Component - Specialized Cedar policy editor
 */

import React, { useCallback } from 'react';
import { CodeEditor, type CodeEditorProps } from './CodeEditor';
import { isValidCedarPolicy } from '../../utils/validators';
import { Alert } from '../common/Alert';

export interface CedarEditorProps extends Omit<CodeEditorProps, 'language'> {
  onValidationChange?: (isValid: boolean) => void;
}

export const CedarEditor = React.forwardRef<HTMLDivElement, CedarEditorProps>(
  ({ value, onChange, onValidationChange, ...props }, ref) => {
    const [error, setError] = React.useState<string | null>(null);

    const handleChange = useCallback(
      (newValue: string) => {
        const isValid = isValidCedarPolicy(newValue);

        if (!isValid && newValue.trim()) {
          setError(
            'Invalid Cedar policy. Must start with "permit" or "forbid" and end with ";"'
          );
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
          language="cedar"
          options={{
            ...props.options,
            wordWrap: 'on',
            formatOnPaste: true,
          }}
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

CedarEditor.displayName = 'CedarEditor';
