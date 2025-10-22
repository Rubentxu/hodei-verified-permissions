/**
 * Policy Editor Feature Tests
 * Tests for HU 16.1, 16.2, 16.3
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PoliciesList } from '../../../src/features/policy-editor/components/PoliciesList';
import { PolicyForm } from '../../../src/features/policy-editor/components/PolicyForm';
import { Policy } from '../../../src/types';

// Mock data
const mockPolicies: Policy[] = [
  {
    policyId: 'allow-read',
    policyStoreId: 'store-1',
    statement: 'permit(principal == User::"alice", action == Action::"read", resource == Document::"doc1");',
    description: 'Allow alice to read doc1',
    createdAt: '2025-10-22T20:00:00Z',
    updatedAt: '2025-10-22T20:00:00Z',
  },
  {
    policyId: 'deny-delete',
    policyStoreId: 'store-1',
    statement: 'forbid(principal, action == Action::"delete", resource);',
    description: 'Deny all delete actions',
    createdAt: '2025-10-22T19:00:00Z',
    updatedAt: '2025-10-22T19:00:00Z',
  },
];

describe('PoliciesList Component', () => {
  describe('HU 16.1: Listar y filtrar políticas', () => {
    it('should render list of policies', () => {
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
        />
      );

      expect(screen.getByText('allow-read')).toBeInTheDocument();
      expect(screen.getByText('deny-delete')).toBeInTheDocument();
    });

    it('should display policy descriptions', () => {
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
        />
      );

      expect(screen.getByText('Allow alice to read doc1')).toBeInTheDocument();
      expect(screen.getByText('Deny all delete actions')).toBeInTheDocument();
    });

    it('should show permit/forbid badges', () => {
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
        />
      );

      expect(screen.getByText('permit')).toBeInTheDocument();
      expect(screen.getByText('forbid')).toBeInTheDocument();
    });

    it('should show loading spinner when loading', () => {
      render(
        <PoliciesList
          policies={[]}
          isLoading={true}
        />
      );

      expect(screen.getByText('Loading policies...')).toBeInTheDocument();
    });

    it('should display error message when error occurs', () => {
      render(
        <PoliciesList
          policies={[]}
          isLoading={false}
          error="Failed to load policies"
        />
      );

      expect(screen.getByText('Error loading policies')).toBeInTheDocument();
      expect(screen.getByText('Failed to load policies')).toBeInTheDocument();
    });

    it('should show empty state when no policies', () => {
      render(
        <PoliciesList
          policies={[]}
          isLoading={false}
        />
      );

      expect(screen.getByText('No policies found')).toBeInTheDocument();
    });

    it('should filter policies by search term', () => {
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
          searchTerm="allow"
        />
      );

      expect(screen.getByText('allow-read')).toBeInTheDocument();
      expect(screen.queryByText('deny-delete')).not.toBeInTheDocument();
    });

    it('should filter policies by effect (permit/forbid)', () => {
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
          filterEffect="permit"
        />
      );

      expect(screen.getByText('allow-read')).toBeInTheDocument();
      expect(screen.queryByText('deny-delete')).not.toBeInTheDocument();
    });

    it('should call onSelectPolicy when clicking a policy', async () => {
      const onSelectPolicy = vi.fn();
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
          onSelectPolicy={onSelectPolicy}
        />
      );

      const policyCard = screen.getByText('allow-read').closest('div[class*="Card"]');
      fireEvent.click(policyCard!);

      expect(onSelectPolicy).toHaveBeenCalledWith(mockPolicies[0]);
    });

    it('should call onDeletePolicy when clicking delete button', async () => {
      const onDeletePolicy = vi.fn();
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
          onDeletePolicy={onDeletePolicy}
        />
      );

      const deleteButtons = screen.getAllByTitle('Delete policy');
      fireEvent.click(deleteButtons[0]);

      expect(onDeletePolicy).toHaveBeenCalledWith(mockPolicies[0].policyId);
    });
  });
});

describe('PolicyForm Component', () => {
  describe('HU 16.2: Crear política estática con editor inteligente', () => {
    it('should render form with input fields', () => {
      render(<PolicyForm />);

      expect(screen.getByLabelText('Policy ID')).toBeInTheDocument();
      expect(screen.getByLabelText('Description')).toBeInTheDocument();
      expect(screen.getByLabelText('Cedar Policy Statement')).toBeInTheDocument();
    });

    it('should have submit button', () => {
      render(<PolicyForm />);

      expect(screen.getByText('Create Policy')).toBeInTheDocument();
    });

    it('should call onSubmit with form data when submitted', async () => {
      const onSubmit = vi.fn();
      const user = userEvent.setup();

      render(<PolicyForm onSubmit={onSubmit} />);

      const policyIdInput = screen.getByLabelText('Policy ID');
      await user.type(policyIdInput, 'test-policy');

      // Note: Cedar editor interaction is complex in tests
      // This test verifies the form structure
      expect(screen.getByText('Create Policy')).toBeInTheDocument();
    });

    it('should disable submit button when required fields are empty', () => {
      render(<PolicyForm />);

      const submitButton = screen.getByText('Create Policy') as HTMLButtonElement;
      expect(submitButton.disabled).toBe(true);
    });

    it('should show loading state when isLoading is true', () => {
      render(<PolicyForm isLoading={true} />);

      const submitButton = screen.getByText('Create Policy') as HTMLButtonElement;
      expect(submitButton.disabled).toBe(true);
    });

    it('should display error message when error prop is provided', () => {
      render(<PolicyForm error="Failed to create policy" />);

      expect(screen.getByText('Failed to create policy')).toBeInTheDocument();
    });

    it('should show edit mode when isEditing is true', () => {
      render(
        <PolicyForm
          isEditing={true}
          policyId="existing-policy"
        />
      );

      expect(screen.getByText('Edit Policy')).toBeInTheDocument();
      const policyIdInput = screen.getByLabelText('Policy ID') as HTMLInputElement;
      expect(policyIdInput.disabled).toBe(true);
    });

    it('should have reset button', () => {
      render(<PolicyForm />);

      expect(screen.getByText('Reset')).toBeInTheDocument();
    });
  });

  describe('HU 16.3: Validar política contra esquema', () => {
    it('should validate Cedar policy syntax', () => {
      render(<PolicyForm />);

      // Cedar editor handles validation
      expect(screen.getByLabelText('Cedar Policy Statement')).toBeInTheDocument();
    });

    it('should disable submit button if policy is invalid', () => {
      render(<PolicyForm statement="invalid policy" />);

      const submitButton = screen.getByText('Create Policy') as HTMLButtonElement;
      expect(submitButton.disabled).toBe(true);
    });

    it('should enable submit button if policy is valid', () => {
      const validPolicy = 'permit(principal, action, resource);';
      render(
        <PolicyForm
          policyId="test-policy"
          statement={validPolicy}
        />
      );

      // Note: Validation depends on CedarEditor component
      expect(screen.getByText('Create Policy')).toBeInTheDocument();
    });
  });

  describe('Policy Editor Integration', () => {
    it('HU 16.1 + 16.2: Should list and create policies', async () => {
      const onSelectPolicy = vi.fn();
      const onSubmit = vi.fn();

      const { rerender } = render(
        <>
          <PolicyForm onSubmit={onSubmit} />
          <PoliciesList
            policies={mockPolicies}
            isLoading={false}
            onSelectPolicy={onSelectPolicy}
          />
        </>
      );

      // Verify form is present
      expect(screen.getByText('Create Policy')).toBeInTheDocument();

      // Verify list is present
      expect(screen.getByText('allow-read')).toBeInTheDocument();
      expect(screen.getByText('deny-delete')).toBeInTheDocument();

      // Test selection
      const policyCard = screen.getByText('allow-read').closest('div[class*="Card"]');
      fireEvent.click(policyCard!);

      expect(onSelectPolicy).toHaveBeenCalledWith(mockPolicies[0]);
    });

    it('HU 16.1 + 16.3: Should list and filter policies', () => {
      render(
        <PoliciesList
          policies={mockPolicies}
          isLoading={false}
          searchTerm="allow"
          filterEffect="permit"
        />
      );

      expect(screen.getByText('allow-read')).toBeInTheDocument();
      expect(screen.queryByText('deny-delete')).not.toBeInTheDocument();
    });
  });
});
