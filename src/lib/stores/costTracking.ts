import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { CostSummary, DateRangeCostSummary } from '../types';

export interface CostTrackingState {
  summary: CostSummary | null;
  todayCost: number;
  monthCost: number;
  loading: boolean;
  error: string | null;
  lastUpdated: Date | null;
}

const initialState: CostTrackingState = {
  summary: null,
  todayCost: 0,
  monthCost: 0,
  loading: false,
  error: null,
  lastUpdated: null,
};

export const costTrackingStore = writable<CostTrackingState>(initialState);

// Derived stores for easy access
export const totalCost = derived(costTrackingStore, ($state) => $state.summary?.totalCostUsd ?? 0);
export const todayCost = derived(costTrackingStore, ($state) => $state.todayCost);
export const monthCost = derived(costTrackingStore, ($state) => $state.monthCost);
export const isLoading = derived(costTrackingStore, ($state) => $state.loading);

// Fetch cost summary from backend
export async function refreshCostSummary() {
  costTrackingStore.update((state) => ({
    ...state,
    loading: true,
    error: null,
  }));

  try {
    const [summary, today, month] = await Promise.all([
      invoke<CostSummary>('get_cost_summary'),
      invoke<number>('get_today_cost'),
      invoke<number>('get_current_month_cost'),
    ]);

    costTrackingStore.update((state) => ({
      ...state,
      summary,
      todayCost: today,
      monthCost: month,
      loading: false,
      lastUpdated: new Date(),
    }));
  } catch (error) {
    console.error('Failed to refresh cost summary:', error);
    costTrackingStore.update((state) => ({
      ...state,
      loading: false,
      error: error instanceof Error ? error.message : 'Failed to load cost data',
    }));
  }
}

// Get costs for a specific date range
export async function getCostsByDateRange(
  startDate?: string,
  endDate?: string
): Promise<DateRangeCostSummary | null> {
  try {
    const result = await invoke<DateRangeCostSummary>('get_cost_by_date_range', {
      startDate,
      endDate,
    });
    return result;
  } catch (error) {
    console.error('Failed to get costs by date range:', error);
    return null;
  }
}

// Clear all cost history
export async function clearCostHistory(): Promise<boolean> {
  try {
    await invoke('clear_cost_history');
    await refreshCostSummary();
    return true;
  } catch (error) {
    console.error('Failed to clear cost history:', error);
    return false;
  }
}

// Format cost for display
export function formatCost(cost: number): string {
  if (cost === 0) return '$0.00';
  if (cost < 0.0001) return '<$0.0001';
  if (cost < 1) return `$${cost.toFixed(4)}`;
  return `$${cost.toFixed(2)}`;
}

// Format large numbers with commas
export function formatNumber(num: number): string {
  return num.toLocaleString();
}

// Get cost by model as sorted array
export function getCostByModelArray(summary: CostSummary | null): Array<{ model: string; cost: number }> {
  if (!summary?.costByModel) return [];
  return Object.entries(summary.costByModel)
    .map(([model, cost]) => ({ model, cost }))
    .sort((a, b) => b.cost - a.cost);
}

// Get cost by working directory as sorted array
export function getCostByWorkingDirArray(summary: CostSummary | null): Array<{ dir: string; cost: number }> {
  if (!summary?.costByWorkingDir) return [];
  return Object.entries(summary.costByWorkingDir)
    .map(([dir, cost]) => ({ dir: dir.split('/').pop() || dir, cost, fullPath: dir }))
    .sort((a, b) => b.cost - a.cost);
}

// Auto-refresh cost data periodically
let refreshInterval: NodeJS.Timeout | null = null;

export function startAutoRefresh(intervalMs: number = 30000) {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }

  // Initial refresh
  refreshCostSummary();

  // Set up periodic refresh
  refreshInterval = setInterval(() => {
    refreshCostSummary();
  }, intervalMs);
}

export function stopAutoRefresh() {
  if (refreshInterval) {
    clearInterval(refreshInterval);
    refreshInterval = null;
  }
}

// Initialize on module load
if (typeof window !== 'undefined') {
  // Start auto-refresh when in browser environment
  startAutoRefresh();
}
