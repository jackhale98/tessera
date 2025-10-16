import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useUIStore } from '../useUIStore';

describe('useUIStore', () => {
  beforeEach(() => {
    // Reset the store before each test
    const { result } = renderHook(() => useUIStore());
    act(() => {
      result.current.setActiveModule('dashboard');
      result.current.setViewMode('table');
      result.current.setSearchQuery('');
      result.current.clearFilters();
    });
  });

  it('has correct initial state', () => {
    const { result } = renderHook(() => useUIStore());

    expect(result.current.activeModule).toBe('dashboard');
    expect(result.current.viewMode).toBe('table');
    expect(result.current.sidebarCollapsed).toBe(false);
    expect(result.current.searchQuery).toBe('');
    expect(result.current.activeFilters).toEqual({});
  });

  it('changes active module', () => {
    const { result } = renderHook(() => useUIStore());

    act(() => {
      result.current.setActiveModule('project');
    });

    expect(result.current.activeModule).toBe('project');
  });

  it('changes view mode', () => {
    const { result } = renderHook(() => useUIStore());

    act(() => {
      result.current.setViewMode('graph');
    });

    expect(result.current.viewMode).toBe('graph');
  });

  it('toggles sidebar', () => {
    const { result } = renderHook(() => useUIStore());

    expect(result.current.sidebarCollapsed).toBe(false);

    act(() => {
      result.current.toggleSidebar();
    });

    expect(result.current.sidebarCollapsed).toBe(true);

    act(() => {
      result.current.toggleSidebar();
    });

    expect(result.current.sidebarCollapsed).toBe(false);
  });

  it('updates search query', () => {
    const { result } = renderHook(() => useUIStore());

    act(() => {
      result.current.setSearchQuery('test query');
    });

    expect(result.current.searchQuery).toBe('test query');
  });

  it('sets and clears filters', () => {
    const { result } = renderHook(() => useUIStore());

    act(() => {
      result.current.setFilter('status', 'Draft');
    });

    expect(result.current.activeFilters).toEqual({ status: 'Draft' });

    act(() => {
      result.current.setFilter('type', 'Task');
    });

    expect(result.current.activeFilters).toEqual({ status: 'Draft', type: 'Task' });

    act(() => {
      result.current.clearFilters();
    });

    expect(result.current.activeFilters).toEqual({});
  });
});
