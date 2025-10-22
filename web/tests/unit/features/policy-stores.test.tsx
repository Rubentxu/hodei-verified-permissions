/**
 * Policy Stores Feature Tests
 * Tests for HU 14.1, 14.2, 14.3
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PolicyStoresList } from '../../../src/features/policy-stores/components/PolicyStoresList';
import { CreatePolicyStoreForm } from '../../../src/features/policy-stores/components/CreatePolicyStoreForm';
import { PolicyStore } from '../../../src/types';

// Mock data
const mockPolicyStores: PolicyStore[] = [
  {
    policyStoreId: '550e8400-e29b-41d4-a716-446655440000',
    description: 'Production Store',
    createdAt: '2025-10-22T20:00:00Z',
    updatedAt: '2025-10-22T20:00:00Z',
  },
  {
    policyStoreId: '550e8400-e29b-41d4-a716-446655440001',
    description: 'Development Store',
    createdAt: '2025-10-22T19:00:00Z',
    updatedAt: '2025-10-22T19:00:00Z',
  },
];

describe('PolicyStoresList Component', () => {
  describe('HU 14.1: Ver lista de Policy Stores', () => {
    it('should render list of policy stores', () => {
      render(
        <PolicyStoresList
          stores={mockPolicyStores}
          isLoading={false}
        />
      );

      expect(screen.getByText('Production Store')).toBeInTheDocument();
      expect(screen.getByText('Development Store')).toBeInTheDocument();
    });

    it('should display policy store IDs', () => {
      render(
        <PolicyStoresList
          stores={mockPolicyStores}
          isLoading={false}
        />
      );

      expect(screen.getByText(/550e8400-e29b-41d4-a716-446655440000/)).toBeInTheDocument();
    });

    it('should show loading spinner when loading', () => {
      render(
        <PolicyStoresList
          stores={[]}
          isLoading={true}
        />
      );

      expect(screen.getByText('Loading policy stores...')).toBeInTheDocument();
    });

    it('should display error message when error occurs', () => {
      render(
        <PolicyStoresList
          stores={[]}
          isLoading={false}
          error="Failed to load stores"
        />
      );

      expect(screen.getByText('Error loading policy stores')).toBeInTheDocument();
      expect(screen.getByText('Failed to load stores')).toBeInTheDocument();
    });

    it('should show empty state when no stores', () => {
      render(
        <PolicyStoresList
          stores={[]}
          isLoading={false}
        />
      );

      expect(screen.getByText('No policy stores found')).toBeInTheDocument();
    });

    it('should call onSelectStore when clicking a store', async () => {
      const onSelectStore = vi.fn();
      render(
        <PolicyStoresList
          stores={mockPolicyStores}
          isLoading={false}
          onSelectStore={onSelectStore}
        />
      );

      const storeCard = screen.getByText('Production Store').closest('div[class*="Card"]');
      fireEvent.click(storeCard!);

      expect(onSelectStore).toHaveBeenCalledWith(mockPolicyStores[0]);
    });

    it('should call onDeleteStore when clicking delete button', async () => {
      const onDeleteStore = vi.fn();
      render(
        <PolicyStoresList
          stores={mockPolicyStores}
          isLoading={false}
          onDeleteStore={onDeleteStore}
        />
      );

      const deleteButtons = screen.getAllByTitle('Delete policy store');
      fireEvent.click(deleteButtons[0]);

      expect(onDeleteStore).toHaveBeenCalledWith(mockPolicyStores[0].policyStoreId);
    });
  });
});

describe('CreatePolicyStoreForm Component', () => {
  describe('HU 14.2: Crear nuevo Policy Store', () => {
    it('should render form with input field', () => {
      render(<CreatePolicyStoreForm />);

      expect(screen.getByLabelText('Description')).toBeInTheDocument();
      expect(screen.getByPlaceholderText(/Production Authorization Store/)).toBeInTheDocument();
    });

    it('should have submit button', () => {
      render(<CreatePolicyStoreForm />);

      expect(screen.getByText('Create Policy Store')).toBeInTheDocument();
    });

    it('should call onSubmit with description when form is submitted', async () => {
      const onSubmit = vi.fn();
      const user = userEvent.setup();

      render(<CreatePolicyStoreForm onSubmit={onSubmit} />);

      const input = screen.getByLabelText('Description');
      await user.type(input, 'Test Store');

      const submitButton = screen.getByText('Create Policy Store');
      await user.click(submitButton);

      expect(onSubmit).toHaveBeenCalledWith('Test Store');
    });

    it('should disable submit button when input is empty', () => {
      render(<CreatePolicyStoreForm />);

      const submitButton = screen.getByText('Create Policy Store') as HTMLButtonElement;
      expect(submitButton.disabled).toBe(true);
    });

    it('should enable submit button when input has value', async () => {
      const user = userEvent.setup();
      render(<CreatePolicyStoreForm />);

      const input = screen.getByLabelText('Description');
      await user.type(input, 'Test Store');

      const submitButton = screen.getByText('Create Policy Store') as HTMLButtonElement;
      expect(submitButton.disabled).toBe(false);
    });

    it('should show loading state when isLoading is true', () => {
      render(<CreatePolicyStoreForm isLoading={true} />);

      const submitButton = screen.getByText('Create Policy Store') as HTMLButtonElement;
      expect(submitButton.disabled).toBe(true);
    });

    it('should display error message when error prop is provided', () => {
      render(<CreatePolicyStoreForm error="Failed to create store" />);

      expect(screen.getByText('Failed to create store')).toBeInTheDocument();
    });

    it('should clear input when Clear button is clicked', async () => {
      const user = userEvent.setup();
      render(<CreatePolicyStoreForm />);

      const input = screen.getByLabelText('Description') as HTMLInputElement;
      await user.type(input, 'Test Store');
      expect(input.value).toBe('Test Store');

      const clearButton = screen.getByText('Clear');
      await user.click(clearButton);

      expect(input.value).toBe('');
    });

    it('should clear input after successful submission', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn();

      render(<CreatePolicyStoreForm onSubmit={onSubmit} />);

      const input = screen.getByLabelText('Description') as HTMLInputElement;
      await user.type(input, 'Test Store');

      const submitButton = screen.getByText('Create Policy Store');
      await user.click(submitButton);

      await waitFor(() => {
        expect(input.value).toBe('');
      });
    });
  });
});

describe('Policy Stores Integration', () => {
  it('HU 14.1 + 14.2: Should list stores and allow creation', async () => {
    const onSelectStore = vi.fn();
    const onSubmit = vi.fn();
    const user = userEvent.setup();

    const { rerender } = render(
      <>
        <CreatePolicyStoreForm onSubmit={onSubmit} />
        <PolicyStoresList
          stores={mockPolicyStores}
          isLoading={false}
          onSelectStore={onSelectStore}
        />
      </>
    );

    // Test creation form
    const input = screen.getByLabelText('Description');
    await user.type(input, 'New Store');
    await user.click(screen.getByText('Create Policy Store'));

    expect(onSubmit).toHaveBeenCalledWith('New Store');

    // Test list display
    expect(screen.getByText('Production Store')).toBeInTheDocument();
    expect(screen.getByText('Development Store')).toBeInTheDocument();

    // Test selection
    const storeCard = screen.getByText('Production Store').closest('div[class*="Card"]');
    fireEvent.click(storeCard!);

    expect(onSelectStore).toHaveBeenCalledWith(mockPolicyStores[0]);
  });
});
