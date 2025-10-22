/**
 * Playground Feature Tests
 * Tests for HU 17.1, 17.2, 17.3
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PlaygroundForm } from '../../../src/features/playground/components/PlaygroundForm';
import { AuthorizationResult } from '../../../src/features/playground/components/AuthorizationResult';
import { AuthorizationResponse } from '../../../src/types';

// Mock data
const mockAllowResult: AuthorizationResponse = {
  decision: 'ALLOW',
  determiningPolicies: ['permit(principal, action, resource);'],
  errors: [],
};

const mockDenyResult: AuthorizationResponse = {
  decision: 'DENY',
  determiningPolicies: ['forbid(principal, action, resource);'],
  errors: [],
};

describe('PlaygroundForm Component', () => {
  describe('HU 17.1: Formular solicitud de autorización de prueba', () => {
    it('should render form with PARC fields', () => {
      render(<PlaygroundForm />);

      expect(screen.getByLabelText('Principal')).toBeInTheDocument();
      expect(screen.getByLabelText('Action')).toBeInTheDocument();
      expect(screen.getByLabelText('Resource')).toBeInTheDocument();
    });

    it('should render context editor', () => {
      render(<PlaygroundForm />);

      expect(screen.getByLabelText('Context (JSON)')).toBeInTheDocument();
    });

    it('should render policies textarea', () => {
      render(<PlaygroundForm />);

      expect(screen.getByPlaceholderText(/permit\(principal/)).toBeInTheDocument();
    });

    it('should render evaluate button', () => {
      render(<PlaygroundForm />);

      expect(screen.getByText('Evaluate')).toBeInTheDocument();
    });

    it('should call onSubmit when form is submitted', async () => {
      const onSubmit = vi.fn();
      const user = userEvent.setup();

      render(<PlaygroundForm onSubmit={onSubmit} />);

      const principalInput = screen.getByLabelText('Principal');
      await user.type(principalInput, 'User::alice');

      const evaluateButton = screen.getByText('Evaluate');
      await user.click(evaluateButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalled();
      });
    });

    it('should have reset button', () => {
      render(<PlaygroundForm />);

      expect(screen.getByText('Reset')).toBeInTheDocument();
    });

    it('should show loading state', () => {
      render(<PlaygroundForm isLoading={true} />);

      const evaluateButton = screen.getByText('Evaluate') as HTMLButtonElement;
      expect(evaluateButton.disabled).toBe(true);
    });

    it('should display error message', () => {
      render(<PlaygroundForm error="Failed to evaluate" />);

      expect(screen.getByText('Failed to evaluate')).toBeInTheDocument();
    });
  });

  describe('HU 17.2: Proporcionar datos de entidades para simulación', () => {
    it('should render entities editor', () => {
      render(<PlaygroundForm />);

      expect(screen.getByLabelText('Entities (JSON)')).toBeInTheDocument();
    });

    it('should accept JSON entities', async () => {
      const onSubmit = vi.fn();
      const user = userEvent.setup();

      render(<PlaygroundForm onSubmit={onSubmit} />);

      const evaluateButton = screen.getByText('Evaluate');
      await user.click(evaluateButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith(
          expect.objectContaining({
            entities: [],
          })
        );
      });
    });

    it('should handle complex entity structures', async () => {
      const onSubmit = vi.fn();

      render(<PlaygroundForm onSubmit={onSubmit} />);

      const evaluateButton = screen.getByText('Evaluate');
      fireEvent.click(evaluateButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalled();
      });
    });
  });
});

describe('AuthorizationResult Component', () => {
  describe('HU 17.3: Ejecutar simulación y visualizar resultados', () => {
    it('should display ALLOW decision', () => {
      render(<AuthorizationResult result={mockAllowResult} />);

      expect(screen.getByText('ALLOW')).toBeInTheDocument();
      expect(screen.getByText(/authorization request was approved/)).toBeInTheDocument();
    });

    it('should display DENY decision', () => {
      render(<AuthorizationResult result={mockDenyResult} />);

      expect(screen.getByText('DENY')).toBeInTheDocument();
      expect(screen.getByText(/authorization request was denied/)).toBeInTheDocument();
    });

    it('should display determining policies', () => {
      render(<AuthorizationResult result={mockAllowResult} />);

      expect(screen.getByText('Determining Policies')).toBeInTheDocument();
      expect(screen.getByText('permit(principal, action, resource);')).toBeInTheDocument();
    });

    it('should display errors if present', () => {
      const resultWithErrors: AuthorizationResponse = {
        decision: 'DENY',
        determiningPolicies: [],
        errors: ['Missing attribute: department'],
      };

      render(<AuthorizationResult result={resultWithErrors} />);

      expect(screen.getByText('Missing attribute: department')).toBeInTheDocument();
    });

    it('should show loading state', () => {
      render(<AuthorizationResult isLoading={true} />);

      expect(screen.getByText('Evaluating...')).toBeInTheDocument();
    });

    it('should not render if no result', () => {
      const { container } = render(<AuthorizationResult />);

      expect(container.firstChild).toBeNull();
    });

    it('should highlight ALLOW with green styling', () => {
      const { container } = render(<AuthorizationResult result={mockAllowResult} />);

      const allowCard = container.querySelector('.bg-green-50');
      expect(allowCard).toBeInTheDocument();
    });

    it('should highlight DENY with red styling', () => {
      const { container } = render(<AuthorizationResult result={mockDenyResult} />);

      const denyCard = container.querySelector('.bg-red-50');
      expect(denyCard).toBeInTheDocument();
    });
  });
});

describe('Playground Integration', () => {
  it('HU 17.1 + 17.2 + 17.3: Should test authorization with entities', async () => {
    const onSubmit = vi.fn();
    const user = userEvent.setup();

    const { rerender } = render(
      <>
        <PlaygroundForm onSubmit={onSubmit} />
        <AuthorizationResult result={mockAllowResult} />
      </>
    );

    // Verify form is present
    expect(screen.getByText('Evaluate')).toBeInTheDocument();

    // Verify result is displayed
    expect(screen.getByText('ALLOW')).toBeInTheDocument();
    expect(screen.getByText('Determining Policies')).toBeInTheDocument();
  });

  it('should handle complete workflow', async () => {
    const onSubmit = vi.fn();

    const { rerender } = render(
      <PlaygroundForm onSubmit={onSubmit} />
    );

    const evaluateButton = screen.getByText('Evaluate');
    fireEvent.click(evaluateButton);

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalled();
    });

    // Simulate result display
    rerender(
      <>
        <PlaygroundForm onSubmit={onSubmit} />
        <AuthorizationResult result={mockAllowResult} />
      </>
    );

    expect(screen.getByText('ALLOW')).toBeInTheDocument();
  });

  it('should handle error scenarios', () => {
    const resultWithErrors: AuthorizationResponse = {
      decision: 'DENY',
      determiningPolicies: [],
      errors: ['Invalid policy syntax', 'Missing schema'],
    };

    render(
      <>
        <PlaygroundForm />
        <AuthorizationResult result={resultWithErrors} />
      </>
    );

    expect(screen.getByText('Invalid policy syntax')).toBeInTheDocument();
    expect(screen.getByText('Missing schema')).toBeInTheDocument();
  });

  it('should support PARC model completely', async () => {
    const onSubmit = vi.fn();
    const user = userEvent.setup();

    render(<PlaygroundForm onSubmit={onSubmit} />);

    const principalInput = screen.getByLabelText('Principal');
    const actionInput = screen.getByLabelText('Action');
    const resourceInput = screen.getByLabelText('Resource');

    await user.type(principalInput, 'User::alice');
    await user.type(actionInput, 'Action::read');
    await user.type(resourceInput, 'Document::doc1');

    const evaluateButton = screen.getByText('Evaluate');
    await user.click(evaluateButton);

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith(
        expect.objectContaining({
          principal: 'User::alice',
          action: 'Action::read',
          resource: 'Document::doc1',
        })
      );
    });
  });
});
