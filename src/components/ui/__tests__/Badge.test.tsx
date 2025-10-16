import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Badge } from '../Badge';

describe('Badge', () => {
  it('renders status badge correctly', () => {
    render(<Badge status="Draft" />);
    expect(screen.getByText('Draft')).toBeInTheDocument();
  });

  it('applies correct styles for Draft status', () => {
    render(<Badge status="Draft" />);
    const badge = screen.getByText('Draft');
    expect(badge).toHaveClass('bg-slate-100', 'text-slate-700');
  });

  it('applies correct styles for PendingApproval status', () => {
    render(<Badge status="PendingApproval" />);
    const badge = screen.getByText('PendingApproval');
    expect(badge).toHaveClass('bg-amber-100', 'text-amber-700');
  });

  it('applies correct styles for Approved status', () => {
    render(<Badge status="Approved" />);
    const badge = screen.getByText('Approved');
    expect(badge).toHaveClass('bg-green-100', 'text-green-700');
  });

  it('applies correct styles for Released status', () => {
    render(<Badge status="Released" />);
    const badge = screen.getByText('Released');
    expect(badge).toHaveClass('bg-blue-100', 'text-blue-700');
  });

  it('renders type badges correctly', () => {
    render(<Badge status="User" variant="type" />);
    const badge = screen.getByText('User');
    expect(badge).toHaveClass('bg-purple-100', 'text-purple-700');
  });

  it('renders unknown status with default styles', () => {
    render(<Badge status="Unknown" />);
    const badge = screen.getByText('Unknown');
    expect(badge).toHaveClass('bg-slate-100', 'text-slate-700');
  });
});
