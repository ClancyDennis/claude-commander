/**
 * Circular buffer for memory-efficient event storage.
 * Keeps a fixed number of recent events in memory while older
 * events are persisted to SQLite for historical queries.
 *
 * Implements FIFO eviction - when buffer is full, oldest items
 * are evicted to make room for new ones.
 */
export class CircularBuffer<T> {
  private buffer: (T | undefined)[];
  private head: number = 0;  // Points to next write position
  private size: number = 0;  // Current number of items
  private readonly capacity: number;

  constructor(capacity: number) {
    if (capacity <= 0) {
      throw new Error('CircularBuffer capacity must be positive');
    }
    this.capacity = capacity;
    this.buffer = new Array(capacity);
  }

  /**
   * Add an item to the buffer.
   * Returns the evicted item if buffer was full, undefined otherwise.
   */
  push(item: T): T | undefined {
    const evicted = this.size === this.capacity
      ? this.buffer[this.head]
      : undefined;

    this.buffer[this.head] = item;
    this.head = (this.head + 1) % this.capacity;

    if (this.size < this.capacity) {
      this.size++;
    }

    return evicted as T | undefined;
  }

  /**
   * Add multiple items at once.
   * Returns array of evicted items.
   */
  pushMany(items: T[]): T[] {
    const evicted: T[] = [];
    for (const item of items) {
      const ev = this.push(item);
      if (ev !== undefined) {
        evicted.push(ev);
      }
    }
    return evicted;
  }

  /**
   * Get all items in order from oldest to newest.
   */
  toArray(): T[] {
    if (this.size === 0) return [];

    const result: T[] = [];
    // Start from the oldest item
    const start = this.size === this.capacity
      ? this.head  // If full, head points to oldest
      : 0;         // If not full, oldest is at 0

    for (let i = 0; i < this.size; i++) {
      const idx = (start + i) % this.capacity;
      result.push(this.buffer[idx] as T);
    }

    return result;
  }

  /**
   * Get the most recent N items (newest first).
   */
  getRecent(count: number): T[] {
    const items = this.toArray();
    return items.slice(-count).reverse();
  }

  /**
   * Get item at index (0 = oldest, length-1 = newest).
   */
  get(index: number): T | undefined {
    if (index < 0 || index >= this.size) {
      return undefined;
    }

    const start = this.size === this.capacity ? this.head : 0;
    const actualIndex = (start + index) % this.capacity;
    return this.buffer[actualIndex] as T;
  }

  /**
   * Get the newest item (last added).
   */
  peek(): T | undefined {
    if (this.size === 0) return undefined;
    const lastIdx = (this.head - 1 + this.capacity) % this.capacity;
    return this.buffer[lastIdx] as T;
  }

  /**
   * Clear all items from the buffer.
   */
  clear(): void {
    this.buffer = new Array(this.capacity);
    this.head = 0;
    this.size = 0;
  }

  /**
   * Get current number of items in buffer.
   */
  get length(): number {
    return this.size;
  }

  /**
   * Check if buffer is empty.
   */
  get isEmpty(): boolean {
    return this.size === 0;
  }

  /**
   * Check if buffer is at capacity.
   */
  get isFull(): boolean {
    return this.size === this.capacity;
  }

  /**
   * Get the maximum capacity of the buffer.
   */
  get maxCapacity(): number {
    return this.capacity;
  }

  /**
   * Get number of available slots.
   */
  get availableSpace(): number {
    return this.capacity - this.size;
  }
}

// Pre-configured buffer sizes for different event types
export const BUFFER_SIZES = {
  toolCalls: 200,      // High frequency during execution
  stateChanges: 50,    // Low frequency
  decisions: 30,       // Very low frequency (1 per iteration)
  agentOutputs: 500,   // High frequency per agent
  toolEvents: 300,     // Medium frequency per agent
} as const;
