import { Card } from '@/components/ui/Card';
import { TrendingUp, Calendar, AlertTriangle, FileText, AlertCircle, MessageSquare } from 'lucide-react';

interface MetricCardProps {
  title: string;
  value: string;
  icon: React.ElementType;
  trend: string;
  color: 'blue' | 'green' | 'red' | 'purple';
}

function MetricCard({ title, value, icon: Icon, trend, color }: MetricCardProps) {
  const colors = {
    blue: 'bg-blue-500',
    green: 'bg-green-500',
    red: 'bg-red-500',
    purple: 'bg-purple-500',
  };

  return (
    <div className="bg-white rounded-lg border border-slate-200 p-5 shadow-sm hover:shadow-md transition-shadow duration-200">
      <div className="flex items-center justify-between mb-3">
        <div className={`p-2.5 rounded-lg ${colors[color]} shadow-sm`}>
          <Icon size={20} className="text-white" />
        </div>
      </div>
      <div className="text-2xl font-bold text-slate-800 mb-1">{value}</div>
      <div className="text-sm text-slate-600 font-medium">{title}</div>
      <div className="text-xs text-slate-500 mt-2">{trend}</div>
    </div>
  );
}

interface WarningItemProps {
  severity: 'high' | 'medium' | 'low';
  text: string;
  module: string;
}

function WarningItem({ severity, text, module }: WarningItemProps) {
  const severityColor = {
    high: 'text-red-500',
    medium: 'text-amber-500',
    low: 'text-blue-500',
  };

  return (
    <div className="flex items-start gap-3 p-3 bg-slate-50 rounded-lg hover:bg-slate-100 transition-colors duration-150">
      <AlertCircle size={18} className={`${severityColor[severity]} flex-shrink-0 mt-0.5`} />
      <div className="flex-1 min-w-0">
        <div className="text-sm text-slate-800">{text}</div>
        <div className="text-xs text-slate-500 mt-1">{module}</div>
      </div>
    </div>
  );
}

export function Dashboard() {
  return (
    <div className="p-6 space-y-6 overflow-auto scrollbar-thin">
      {/* Metrics */}
      <div className="grid grid-cols-4 gap-4">
        <MetricCard
          title="Project Complete"
          value="0%"
          icon={TrendingUp}
          trend="Getting started"
          color="blue"
        />
        <MetricCard
          title="Est. Completion"
          value="TBD"
          icon={Calendar}
          trend="No tasks yet"
          color="green"
        />
        <MetricCard
          title="Open Risks"
          value="0"
          icon={AlertTriangle}
          trend="No risks identified"
          color="red"
        />
        <MetricCard
          title="Requirements"
          value="0"
          icon={FileText}
          trend="No requirements yet"
          color="purple"
        />
      </div>

      {/* Warnings & Activity */}
      <div className="grid grid-cols-2 gap-6">
        <Card title="Warnings & Issues" icon={<AlertCircle size={18} className="text-amber-500" />}>
          <div className="text-sm text-slate-500 text-center py-8">
            No warnings or issues to display
          </div>
        </Card>

        <Card title="Recent Activity" icon={<MessageSquare size={18} className="text-blue-500" />}>
          <div className="text-sm text-slate-500 text-center py-8">
            No recent activity
          </div>
        </Card>
      </div>

      {/* Welcome Message */}
      <Card>
        <div className="text-center py-12">
          <h2 className="text-2xl font-bold text-slate-800 mb-3">Welcome to Tessera</h2>
          <p className="text-slate-600 max-w-2xl mx-auto leading-relaxed">
            Your product lifecycle management system is ready. Start by creating your first entities in the
            Project Management, Requirements, Risk Management, or Design modules.
          </p>
        </div>
      </Card>
    </div>
  );
}
