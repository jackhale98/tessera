import { create } from 'zustand';
import type { EntityType } from '@/types';

type ViewMode = 'table' | 'graph' | 'matrix' | 'chart';

interface UIState {
  // Navigation
  activeModule: string;
  viewMode: ViewMode;

  // Sidebar
  sidebarCollapsed: boolean;

  // Search
  searchQuery: string;

  // Filters
  activeFilters: Record<string, any>;

  // Actions
  setActiveModule: (module: string) => void;
  setViewMode: (mode: ViewMode) => void;
  toggleSidebar: () => void;
  setSearchQuery: (query: string) => void;
  setFilter: (key: string, value: any) => void;
  clearFilters: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  // Initial state
  activeModule: 'dashboard',
  viewMode: 'table',
  sidebarCollapsed: false,
  searchQuery: '',
  activeFilters: {},

  // Actions
  setActiveModule: (module) => set({ activeModule: module }),
  setViewMode: (mode) => set({ viewMode: mode }),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setFilter: (key, value) => set((state) => ({
    activeFilters: { ...state.activeFilters, [key]: value },
  })),
  clearFilters: () => set({ activeFilters: {} }),
}));
