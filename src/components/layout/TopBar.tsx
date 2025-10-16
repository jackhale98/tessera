import { Search, Bell, Star, Plus } from 'lucide-react';
import { useUIStore } from '@/stores/useUIStore';

interface TopBarProps {
  title: string;
  onNewEntity?: () => void;
}

export function TopBar({ title, onNewEntity }: TopBarProps) {
  const { searchQuery, setSearchQuery } = useUIStore();

  return (
    <header className="bg-white border-b border-slate-200 px-6 py-3 flex items-center justify-between shadow-sm">
      <div className="flex items-center gap-4">
        <h2 className="text-lg font-semibold text-slate-800">{title}</h2>

        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
          <input
            type="text"
            placeholder="Search entities..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="input pl-9 pr-4 py-1.5 w-64"
          />
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button className="p-2 hover:bg-slate-100 rounded-lg relative transition-colors duration-200">
          <Bell size={18} className="text-slate-600" />
          <span className="absolute top-1 right-1 w-2 h-2 bg-red-500 rounded-full ring-2 ring-white"></span>
        </button>

        <button className="p-2 hover:bg-slate-100 rounded-lg transition-colors duration-200">
          <Star size={18} className="text-slate-600" />
        </button>

        {onNewEntity && (
          <button
            onClick={onNewEntity}
            className="btn btn-primary ml-2"
          >
            <Plus size={16} />
            New Entity
          </button>
        )}
      </div>
    </header>
  );
}
