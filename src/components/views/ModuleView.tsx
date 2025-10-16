import { Table as TableIcon, Download, Filter } from 'lucide-react';
import { EntityTableView } from './EntityTableView';

interface ModuleViewProps {
  module: 'project' | 'requirements' | 'risks' | 'design' | 'verification' | 'manufacturing';
}

export function ModuleView({ module }: ModuleViewProps) {
  // Map modules to entity types
  const entityTypeMap: Record<string, 'tasks' | 'requirements' | 'risks'> = {
    project: 'tasks',
    requirements: 'requirements',
    risks: 'risks',
  };

  const entityType = entityTypeMap[module];

  return (
    <div className="h-full flex flex-col">
      {/* View Controls */}
      <div className="bg-white border-b border-slate-200 px-6 py-3 flex items-center justify-between shadow-sm">
        <div className="flex items-center gap-2">
          <button className="px-3 py-1.5 rounded-lg text-sm font-medium flex items-center gap-2 bg-blue-100 text-blue-700 transition-colors duration-200">
            <TableIcon size={16} />
            Table
          </button>
        </div>

        <div className="flex items-center gap-2">
          <button className="btn btn-secondary text-sm">
            <Filter size={16} />
            Filter
          </button>
          <button className="btn btn-secondary text-sm">
            <Download size={16} />
            Export
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6 bg-slate-50 scrollbar-thin">
        {entityType ? (
          <EntityTableView entityType={entityType} />
        ) : (
          <div className="bg-white rounded-lg border border-slate-200 shadow-sm p-12 text-center">
            <div className="text-slate-500 mb-2">This module is coming soon</div>
            <div className="text-sm text-slate-400">Stay tuned for updates!</div>
          </div>
        )}
      </div>
    </div>
  );
}
