import {
  Home, FolderKanban, FileText, AlertTriangle, Box, CheckSquare, Factory,
  Settings, GitBranch
} from 'lucide-react';
import { useUIStore } from '@/stores/useUIStore';

interface Module {
  id: string;
  name: string;
  icon: React.ElementType;
}

const modules: Module[] = [
  { id: 'dashboard', name: 'Dashboard', icon: Home },
  { id: 'project', name: 'Project Management', icon: FolderKanban },
  { id: 'requirements', name: 'Requirements', icon: FileText },
  { id: 'risks', name: 'Risk Management', icon: AlertTriangle },
  { id: 'design', name: 'Design & BOM', icon: Box },
  { id: 'verification', name: 'V&V', icon: CheckSquare },
  { id: 'manufacturing', name: 'Manufacturing', icon: Factory },
];

export function Sidebar() {
  const { activeModule, setActiveModule } = useUIStore();

  return (
    <div className="w-64 bg-slate-900 text-white flex flex-col shadow-xl">
      {/* Header */}
      <div className="p-4 border-b border-slate-700">
        <h1 className="text-xl font-bold text-white tracking-tight">Tessera</h1>
        <p className="text-xs text-slate-400 mt-0.5">Product Lifecycle Management</p>
      </div>

      {/* Module Navigation */}
      <nav className="flex-1 py-4 overflow-y-auto scrollbar-thin">
        {modules.map(module => {
          const Icon = module.icon;
          const isActive = activeModule === module.id;

          return (
            <button
              key={module.id}
              onClick={() => setActiveModule(module.id)}
              className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-all duration-200 ${
                isActive
                  ? 'bg-blue-600 text-white shadow-lg'
                  : 'text-slate-300 hover:bg-slate-800 hover:text-white'
              }`}
            >
              <Icon size={18} className={isActive ? 'text-white' : 'text-slate-400'} />
              <span className="font-medium">{module.name}</span>
            </button>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-slate-700 space-y-2">
        <button className="w-full flex items-center gap-2 px-3 py-2 text-sm text-slate-300 hover:bg-slate-800 hover:text-white rounded transition-all duration-200">
          <GitBranch size={16} />
          <span>main</span>
        </button>
        <button className="w-full flex items-center gap-2 px-3 py-2 text-sm text-slate-300 hover:bg-slate-800 hover:text-white rounded transition-all duration-200">
          <Settings size={16} />
          <span>Settings</span>
        </button>
      </div>
    </div>
  );
}
