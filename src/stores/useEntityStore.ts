import { create } from 'zustand';
import type { Entity, EntityType, Task, Requirement, Risk, Component, Assembly } from '@/types';
import { api } from '@/lib/api';

interface EntityState {
  // Data
  tasks: Task[];
  requirements: Requirement[];
  risks: Risk[];
  components: Component[];
  assemblies: Assembly[];

  // UI State
  loading: boolean;
  error: string | null;
  selectedEntityId: string | null;

  // Actions - Tasks
  loadTasks: () => Promise<void>;
  createTask: (data: Partial<Task>) => Promise<Task>;
  updateTask: (id: string, data: Partial<Task>) => Promise<Task>;
  deleteTask: (id: string) => Promise<void>;

  // Actions - Requirements
  loadRequirements: () => Promise<void>;
  createRequirement: (data: Partial<Requirement>) => Promise<Requirement>;
  updateRequirement: (id: string, data: Partial<Requirement>) => Promise<Requirement>;
  deleteRequirement: (id: string) => Promise<void>;

  // Actions - Risks
  loadRisks: () => Promise<void>;
  createRisk: (data: Partial<Risk>) => Promise<Risk>;
  updateRisk: (id: string, data: Partial<Risk>) => Promise<Risk>;
  deleteRisk: (id: string) => Promise<void>;

  // Actions - Components
  loadComponents: () => Promise<void>;
  createComponent: (data: Partial<Component>) => Promise<Component>;
  updateComponent: (id: string, data: Partial<Component>) => Promise<Component>;
  deleteComponent: (id: string) => Promise<void>;

  // Actions - Assemblies
  loadAssemblies: () => Promise<void>;
  createAssembly: (data: Partial<Assembly>) => Promise<Assembly>;
  updateAssembly: (id: string, data: Partial<Assembly>) => Promise<Assembly>;
  deleteAssembly: (id: string) => Promise<void>;

  // UI Actions
  selectEntity: (id: string | null) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useEntityStore = create<EntityState>((set, get) => ({
  // Initial state
  tasks: [],
  requirements: [],
  risks: [],
  components: [],
  assemblies: [],
  loading: false,
  error: null,
  selectedEntityId: null,

  // Task actions
  loadTasks: async () => {
    set({ loading: true, error: null });
    try {
      const tasks = await api.task.list();
      set({ tasks, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  createTask: async (data) => {
    set({ loading: true, error: null });
    try {
      const task = await api.task.create(data);
      set(state => ({
        tasks: [...state.tasks, task],
        loading: false,
      }));
      return task;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  updateTask: async (id, data) => {
    set({ loading: true, error: null });
    try {
      const task = await api.task.update(id, data);
      set(state => ({
        tasks: state.tasks.map(t => t.metadata.id === id ? task : t),
        loading: false,
      }));
      return task;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  deleteTask: async (id) => {
    set({ loading: true, error: null });
    try {
      await api.task.delete(id);
      set(state => ({
        tasks: state.tasks.filter(t => t.metadata.id !== id),
        loading: false,
        selectedEntityId: state.selectedEntityId === id ? null : state.selectedEntityId,
      }));
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  // Requirement actions
  loadRequirements: async () => {
    set({ loading: true, error: null });
    try {
      const requirements = await api.requirement.list();
      set({ requirements, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  createRequirement: async (data) => {
    set({ loading: true, error: null });
    try {
      const requirement = await api.requirement.create(data);
      set(state => ({
        requirements: [...state.requirements, requirement],
        loading: false,
      }));
      return requirement;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  updateRequirement: async (id, data) => {
    set({ loading: true, error: null });
    try {
      const requirement = await api.requirement.update(id, data);
      set(state => ({
        requirements: state.requirements.map(r => r.metadata.id === id ? requirement : r),
        loading: false,
      }));
      return requirement;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  deleteRequirement: async (id) => {
    set({ loading: true, error: null });
    try {
      await api.requirement.delete(id);
      set(state => ({
        requirements: state.requirements.filter(r => r.metadata.id !== id),
        loading: false,
        selectedEntityId: state.selectedEntityId === id ? null : state.selectedEntityId,
      }));
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  // Risk actions
  loadRisks: async () => {
    set({ loading: true, error: null });
    try {
      const risks = await api.risk.list();
      set({ risks, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  createRisk: async (data) => {
    set({ loading: true, error: null });
    try {
      const risk = await api.risk.create(data);
      set(state => ({
        risks: [...state.risks, risk],
        loading: false,
      }));
      return risk;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  updateRisk: async (id, data) => {
    set({ loading: true, error: null });
    try {
      const risk = await api.risk.update(id, data);
      set(state => ({
        risks: state.risks.map(r => r.metadata.id === id ? risk : r),
        loading: false,
      }));
      return risk;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  deleteRisk: async (id) => {
    set({ loading: true, error: null });
    try {
      await api.risk.delete(id);
      set(state => ({
        risks: state.risks.filter(r => r.metadata.id !== id),
        loading: false,
        selectedEntityId: state.selectedEntityId === id ? null : state.selectedEntityId,
      }));
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  // Component actions
  loadComponents: async () => {
    set({ loading: true, error: null });
    try {
      const components = await api.component.list();
      set({ components, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  createComponent: async (data) => {
    set({ loading: true, error: null });
    try {
      const component = await api.component.create(data);
      set(state => ({
        components: [...state.components, component],
        loading: false,
      }));
      return component;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  updateComponent: async (id, data) => {
    set({ loading: true, error: null });
    try {
      const component = await api.component.update(id, data);
      set(state => ({
        components: state.components.map(c => c.metadata.id === id ? component : c),
        loading: false,
      }));
      return component;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  deleteComponent: async (id) => {
    set({ loading: true, error: null });
    try {
      await api.component.delete(id);
      set(state => ({
        components: state.components.filter(c => c.metadata.id !== id),
        loading: false,
        selectedEntityId: state.selectedEntityId === id ? null : state.selectedEntityId,
      }));
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  // Assembly actions
  loadAssemblies: async () => {
    set({ loading: true, error: null });
    try {
      const assemblies = await api.assembly.list();
      set({ assemblies, loading: false });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  createAssembly: async (data) => {
    set({ loading: true, error: null });
    try {
      const assembly = await api.assembly.create(data);
      set(state => ({
        assemblies: [...state.assemblies, assembly],
        loading: false,
      }));
      return assembly;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  updateAssembly: async (id, data) => {
    set({ loading: true, error: null });
    try {
      const assembly = await api.assembly.update(id, data);
      set(state => ({
        assemblies: state.assemblies.map(a => a.metadata.id === id ? assembly : a),
        loading: false,
      }));
      return assembly;
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  deleteAssembly: async (id) => {
    set({ loading: true, error: null });
    try {
      await api.assembly.delete(id);
      set(state => ({
        assemblies: state.assemblies.filter(a => a.metadata.id !== id),
        loading: false,
        selectedEntityId: state.selectedEntityId === id ? null : state.selectedEntityId,
      }));
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
      throw error;
    }
  },

  // UI actions
  selectEntity: (id) => set({ selectedEntityId: id }),
  setError: (error) => set({ error }),
  clearError: () => set({ error: null }),
}));
