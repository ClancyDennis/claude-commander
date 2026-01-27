/**
 * Async Data Hook
 *
 * Provides reactive state for async data fetching operations.
 * Eliminates duplicate loading/error/data patterns across components.
 *
 * Usage:
 *   const users = useAsyncData(() => invoke<User[]>('get_users'));
 *   await users.fetch();
 *   // Access: users.data, users.loading, users.error
 *
 * With options:
 *   const config = useAsyncData(() => invoke<Config>('get_config'), {
 *     initialData: defaultConfig,
 *     fetchOnMount: true
 *   });
 */

export interface AsyncDataState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

export interface AsyncDataOptions<T> {
  /** Initial data before first fetch */
  initialData?: T;
  /** Automatically fetch on hook creation (use with onMount) */
  fetchOnMount?: boolean;
}

export function useAsyncData<T>(
  fetcher: () => Promise<T>,
  options?: AsyncDataOptions<T>
) {
  let state = $state<AsyncDataState<T>>({
    data: options?.initialData ?? null,
    loading: false,
    error: null,
  });

  /**
   * Fetch data from the async source
   */
  async function fetch(): Promise<T | null> {
    state.loading = true;
    state.error = null;

    try {
      const result = await fetcher();
      state.data = result;
      return result;
    } catch (e) {
      state.error = e instanceof Error ? e.message : String(e);
      return null;
    } finally {
      state.loading = false;
    }
  }

  /**
   * Reset state to initial values
   */
  function reset(): void {
    state.data = options?.initialData ?? null;
    state.loading = false;
    state.error = null;
  }

  /**
   * Set data manually (useful for optimistic updates)
   */
  function setData(data: T | null): void {
    state.data = data;
  }

  /**
   * Set error manually
   */
  function setError(error: string | null): void {
    state.error = error;
  }

  return {
    // Reactive state access
    get state() {
      return state;
    },
    get data() {
      return state.data;
    },
    get loading() {
      return state.loading;
    },
    get error() {
      return state.error;
    },

    // Actions
    fetch,
    reset,
    setData,
    setError,
  };
}
