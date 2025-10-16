import { useEffect } from 'react';
import { Table, TableHeader, TableBody, TableRow, TableHead, TableCell } from '@/components/ui/Table';
import { Badge } from '@/components/ui/Badge';
import { useEntityStore } from '@/stores/useEntityStore';
import type { Task, Requirement, Risk } from '@/types';
import { format } from 'date-fns';
import { Loader2 } from 'lucide-react';

interface EntityTableViewProps {
  entityType: 'tasks' | 'requirements' | 'risks';
}

export function EntityTableView({ entityType }: EntityTableViewProps) {
  const store = useEntityStore();
  const entities = store[entityType];
  const loading = store.loading;

  useEffect(() => {
    // Load entities when component mounts
    if (entityType === 'tasks') {
      store.loadTasks();
    } else if (entityType === 'requirements') {
      store.loadRequirements();
    } else if (entityType === 'risks') {
      store.loadRisks();
    }
  }, [entityType]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="w-8 h-8 text-blue-500 animate-spin" />
          <div className="text-slate-500 text-sm">Loading {entityType}...</div>
        </div>
      </div>
    );
  }

  if (entities.length === 0) {
    return (
      <div className="bg-white rounded-lg border border-slate-200 shadow-sm p-12 text-center">
        <div className="text-slate-500 mb-2">No {entityType} found</div>
        <div className="text-sm text-slate-400">Click "New Entity" to create one</div>
      </div>
    );
  }

  // Render table based on entity type
  if (entityType === 'tasks') {
    return (
      <Table>
        <TableHeader>
          <tr>
            <TableHead>Name</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Deadline</TableHead>
            <TableHead>Progress</TableHead>
            <TableHead>Critical Path</TableHead>
          </tr>
        </TableHeader>
        <TableBody>
          {(entities as Task[]).map((task) => (
            <TableRow key={task.metadata.id}>
              <TableCell className="font-medium text-blue-600 hover:text-blue-700 cursor-pointer">
                {task.name}
              </TableCell>
              <TableCell>
                <Badge status={task.metadata.status} />
              </TableCell>
              <TableCell className="text-slate-600">
                {format(new Date(task.deadline), 'MMM dd, yyyy')}
              </TableCell>
              <TableCell>
                <div className="flex items-center gap-2">
                  <div className="flex-1 h-2 bg-slate-100 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-blue-500 transition-all duration-300"
                      style={{ width: `${task.percent_complete * 100}%` }}
                    />
                  </div>
                  <span className="text-xs text-slate-600 w-12 tabular-nums">
                    {Math.round(task.percent_complete * 100)}%
                  </span>
                </div>
              </TableCell>
              <TableCell>
                {task.is_critical_path ? (
                  <span className="inline-flex items-center gap-1.5 text-red-600 font-medium text-xs">
                    <span className="w-1.5 h-1.5 bg-red-600 rounded-full"></span>
                    Yes
                  </span>
                ) : (
                  <span className="text-slate-400 text-xs">No</span>
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    );
  }

  if (entityType === 'requirements') {
    return (
      <Table>
        <TableHeader>
          <tr>
            <TableHead>Name</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Description</TableHead>
          </tr>
        </TableHeader>
        <TableBody>
          {(entities as Requirement[]).map((req) => (
            <TableRow key={req.metadata.id}>
              <TableCell className="font-medium text-blue-600 hover:text-blue-700 cursor-pointer">
                {req.name}
              </TableCell>
              <TableCell>
                <Badge status={req.requirement_type} variant="type" />
              </TableCell>
              <TableCell>
                <Badge status={req.metadata.status} />
              </TableCell>
              <TableCell className="text-slate-600 truncate max-w-md">
                {req.description}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    );
  }

  if (entityType === 'risks') {
    return (
      <Table>
        <TableHeader>
          <tr>
            <TableHead>Name</TableHead>
            <TableHead className="text-center">Probability</TableHead>
            <TableHead className="text-center">Severity</TableHead>
            <TableHead className="text-center">Score</TableHead>
            <TableHead>Status</TableHead>
          </tr>
        </TableHeader>
        <TableBody>
          {(entities as Risk[]).map((risk) => (
            <TableRow key={risk.metadata.id}>
              <TableCell className="font-medium text-blue-600 hover:text-blue-700 cursor-pointer">
                {risk.name}
              </TableCell>
              <TableCell className="text-center">
                <span className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-amber-100 text-amber-800 text-sm font-semibold">
                  {risk.probability}
                </span>
              </TableCell>
              <TableCell className="text-center">
                <span className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-red-100 text-red-800 text-sm font-semibold">
                  {risk.severity}
                </span>
              </TableCell>
              <TableCell className="text-center">
                <RiskScoreBadge score={risk.risk_score} />
              </TableCell>
              <TableCell>
                <Badge status={risk.metadata.status} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    );
  }

  return null;
}

function RiskScoreBadge({ score }: { score: number }) {
  const getColor = () => {
    if (score <= 4) return 'bg-green-500';
    if (score <= 9) return 'bg-amber-500';
    if (score <= 15) return 'bg-orange-500';
    return 'bg-red-500';
  };

  return (
    <span className={`inline-flex items-center justify-center w-10 h-10 rounded-full ${getColor()} text-white text-sm font-bold shadow-sm`}>
      {score}
    </span>
  );
}
