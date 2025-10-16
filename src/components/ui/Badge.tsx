import type { EntityStatus } from '@/types';

interface BadgeProps {
  status: EntityStatus | string;
  variant?: 'status' | 'type' | 'custom';
}

const statusStyles: Record<string, string> = {
  Draft: 'bg-slate-100 text-slate-700',
  PendingApproval: 'bg-amber-100 text-amber-700',
  Approved: 'bg-green-100 text-green-700',
  Released: 'bg-blue-100 text-blue-700',
};

const typeStyles: Record<string, string> = {
  User: 'bg-purple-100 text-purple-700',
  System: 'bg-blue-100 text-blue-700',
  Design: 'bg-green-100 text-green-700',
  Software: 'bg-cyan-100 text-cyan-700',
  Safety: 'bg-red-100 text-red-700',
};

export function Badge({ status, variant = 'status' }: BadgeProps) {
  const styles = variant === 'type' ? typeStyles : statusStyles;
  const className = styles[status] || 'bg-slate-100 text-slate-700';

  return (
    <span className={`px-2 py-1 rounded-full text-xs font-medium ${className}`}>
      {status}
    </span>
  );
}
