/**
 * CodeEditor Component - Monaco Editor wrapper
 */

import React, { useCallback } from 'react';
import Editor, { type OnChange } from '@monaco-editor/react';
import { cn } from '../../utils/cn';

export interface CodeEditorProps {
  value: string;
  onChange?: (value: string) => void;
  language?: 'json' | 'cedar' | 'javascript' | 'typescript' | 'yaml';
  readOnly?: boolean;
  height?: string;
  className?: string;
  theme?: 'light' | 'dark';
  options?: Record<string, unknown>;
}

// Cedar language configuration
const cedarLanguageConfig = {
  id: 'cedar',
  extensions: ['.cedar'],
  aliases: ['Cedar'],
  mimetypes: ['text/x-cedar'],
};

export const CodeEditor = React.forwardRef<HTMLDivElement, CodeEditorProps>(
  (
    {
      value,
      onChange,
      language = 'json',
      readOnly = false,
      height = '400px',
      className,
      theme = 'light',
      options = {},
    },
    ref
  ) => {
    const handleChange: OnChange = useCallback(
      (newValue) => {
        if (newValue && onChange) {
          onChange(newValue);
        }
      },
      [onChange]
    );

    const editorOptions = {
      minimap: { enabled: false },
      fontSize: 13,
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      automaticLayout: true,
      readOnly,
      ...options,
    };

    return (
      <div
        ref={ref}
        className={cn('rounded-md border border-gray-300 overflow-hidden', className)}
      >
        <Editor
          height={height}
          language={language === 'cedar' ? 'javascript' : language}
          value={value}
          onChange={handleChange}
          theme={theme === 'dark' ? 'vs-dark' : 'vs-light'}
          options={editorOptions}
          loading={<div className="p-4 text-gray-500">Loading editor...</div>}
        />
      </div>
    );
  }
);

CodeEditor.displayName = 'CodeEditor';
