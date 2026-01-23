const STORAGE_KEY = 'tutorial_completed';

export function createTutorialStore() {
  let currentStep = $state(0);
  let isActive = $state(false);
  let hasCompleted = $state(typeof localStorage !== 'undefined' && localStorage.getItem(STORAGE_KEY) === 'true');

  const steps = [
    {
      target: '[data-tutorial="agent-list"]',
      title: 'Your Helpers List',
      message: 'All your AI helpers appear here. Each one works in a specific folder.',
    },
    {
      target: '[data-tutorial="new-button"]',
      title: 'Create a Helper',
      message: 'Click here to create a new helper for any task.',
    },
    {
      target: '[data-tutorial="agent-view"]',
      title: 'Watch Progress',
      message: 'See what your helper is doing in real-time.',
    },
    {
      target: '[data-tutorial="chat-button"]',
      title: 'Chat Interface',
      message: 'Need to adjust your task? Chat here to guide your helpers.',
    },
    {
      target: '[data-tutorial="status-badge"]',
      title: 'Status Indicators',
      message: 'These show what stage your helper is at: Working, Waiting, or Completed.',
    },
  ];

  return {
    get currentStep() { return currentStep; },
    get isActive() { return isActive; },
    get hasCompleted() { return hasCompleted; },
    get steps() { return steps; },
    get currentStepData() { return steps[currentStep]; },

    start() { isActive = true; currentStep = 0; },
    next() {
      if (currentStep < steps.length - 1) currentStep++;
      else this.complete();
    },
    skip() { this.complete(); },
    complete() {
      isActive = false;
      hasCompleted = true;
      if (typeof localStorage !== 'undefined') localStorage.setItem(STORAGE_KEY, 'true');
    },
    reset() {
      if (typeof localStorage !== 'undefined') localStorage.removeItem(STORAGE_KEY);
      hasCompleted = false;
    },
  };
}

export const tutorialStore = createTutorialStore();
