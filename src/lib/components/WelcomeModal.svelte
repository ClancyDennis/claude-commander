<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { createEventDispatcher } from "svelte";
  import logoIcon from "$lib/assets/claude-commander-icon.png";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import {
    Sparkles,
    Code,
    FileText,
    MessageSquare,
    Play,
    Settings,
    FolderOpen,
    Check,
    ChevronRight,
    Keyboard,
    ArrowRight,
    Rocket,
    Star
  } from "lucide-svelte";

  interface Props {
    show: boolean;
    onClose: () => void;
  }

  let { show = $bindable(), onClose }: Props = $props();

  const dispatch = createEventDispatcher<{ startTutorial: void }>();

  // Wizard state
  let currentStep = $state(1);
  let selectedPath = $state<"example" | "custom" | null>(null);
  let apiKey = $state("");
  let selectedFolder = $state("");
  let projectType = $state<"web" | "cli" | "library" | "other" | null>(null);
  let animationKey = $state(0);

  const TOTAL_STEPS = 4;
  const STORAGE_KEY = "claude-commander-onboarding-complete";

  // Derived state
  let isApiConfigured = $derived(apiKey.length > 0);
  let canProceedFromStep3 = $derived(selectedFolder.length > 0);

  // Example use cases for Step 1
  const useCases = [
    {
      icon: Code,
      title: "Build Features",
      description: "Describe what you want and let AI agents write the code"
    },
    {
      icon: FileText,
      title: "Refactor Code",
      description: "Modernize legacy code or improve architecture automatically"
    },
    {
      icon: MessageSquare,
      title: "Debug Issues",
      description: "Explain bugs in plain language and get fixes generated"
    }
  ];

  // Keyboard shortcuts for Step 4
  const shortcuts = [
    { keys: ["Ctrl", "K"], description: "Open command palette" },
    { keys: ["Ctrl", "Enter"], description: "Send message to agent" },
    { keys: ["Ctrl", "Shift", "N"], description: "New agent" },
    { keys: ["Esc"], description: "Cancel current operation" }
  ];

  // Project types for Step 3
  const projectTypes = [
    { id: "web", label: "Web App" },
    { id: "cli", label: "CLI Tool" },
    { id: "library", label: "Library" },
    { id: "other", label: "Other" }
  ] as const;

  function goToStep(step: number) {
    animationKey++;
    currentStep = step;
  }

  function handleGetStarted() {
    goToStep(2);
  }

  function handleChoosePath(path: "example" | "custom") {
    selectedPath = path;
    if (path === "example") {
      goToStep(4);
    } else {
      goToStep(3);
    }
  }

  async function handleSelectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Project Folder"
      });
      if (selected && typeof selected === "string") {
        selectedFolder = selected;
      }
    } catch (e) {
      console.error("Failed to open folder picker:", e);
    }
  }

  function handleContinueFromSetup() {
    goToStep(4);
  }

  function handleStartTutorial() {
    completeOnboarding();
    dispatch("startTutorial");
    onClose();
  }

  function handleSkip() {
    completeOnboarding();
    onClose();
  }

  function completeOnboarding() {
    try {
      localStorage.setItem(STORAGE_KEY, "true");
    } catch (e) {
      console.error("Failed to save onboarding status:", e);
    }
  }

  async function handleOpenConfigFolder() {
    try {
      await invoke("open_config_directory");
    } catch (e) {
      console.error("Failed to open config directory:", e);
    }
  }

  function handleBack() {
    if (currentStep === 3) {
      goToStep(2);
    } else if (currentStep === 4) {
      if (selectedPath === "custom") {
        goToStep(3);
      } else {
        goToStep(2);
      }
    } else if (currentStep === 2) {
      goToStep(1);
    }
  }
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="max-w-2xl p-0 overflow-hidden">
    <!-- Progress Indicator -->
    <div class="flex items-center justify-center gap-2 pt-6 pb-2">
      {#each Array(TOTAL_STEPS) as _, i}
        <div
          class="h-2 rounded-full transition-all duration-300 {i + 1 === currentStep
            ? 'w-8 bg-primary'
            : i + 1 < currentStep
              ? 'w-2 bg-primary/60'
              : 'w-2 bg-muted'}"
        ></div>
      {/each}
    </div>

    <!-- Step Content -->
    <div class="p-6 pt-2" key={animationKey}>
      <!-- Step 1: Welcome -->
      {#if currentStep === 1}
        <div class="animate-fade-in">
          <div class="flex flex-col items-center text-center mb-6">
            <div class="mb-4">
              <img src={logoIcon} alt="Claude Commander" class="w-16 h-16 rounded-xl" />
            </div>
            <Dialog.Header class="items-center">
              <Dialog.Title class="text-2xl">Welcome to Claude Commander</Dialog.Title>
              <Dialog.Description class="text-base mt-2">
                Claude Commander helps you get things done using AI assistants.
                Describe what you want in plain language, and watch intelligent agents
                handle the heavy lifting.
              </Dialog.Description>
            </Dialog.Header>
          </div>

          <div class="grid gap-3 mb-6">
            {#each useCases as useCase}
              <div class="flex items-start gap-4 p-4 rounded-lg bg-muted/50 border border-border">
                <div class="flex-shrink-0 w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
                  <svelte:component this={useCase.icon} class="w-5 h-5 text-primary" />
                </div>
                <div class="flex-1">
                  <h4 class="font-medium text-foreground">{useCase.title}</h4>
                  <p class="text-sm text-muted-foreground">{useCase.description}</p>
                </div>
              </div>
            {/each}
          </div>

          <Dialog.Footer class="justify-center">
            <Button onclick={handleGetStarted} size="lg" class="gap-2">
              Get Started
              <ArrowRight class="w-4 h-4" />
            </Button>
          </Dialog.Footer>
        </div>
      {/if}

      <!-- Step 2: Choose Path -->
      {#if currentStep === 2}
        <div class="animate-slide-in">
          <Dialog.Header class="text-center mb-6">
            <Dialog.Title class="text-xl">How would you like to start?</Dialog.Title>
            <Dialog.Description>
              Choose your preferred onboarding path
            </Dialog.Description>
          </Dialog.Header>

          <div class="grid gap-4 mb-6">
            <!-- Try an Example Card -->
            <button
              onclick={() => handleChoosePath("example")}
              class="text-left w-full"
            >
              <Card.Root class="transition-all duration-200 hover:border-primary hover:shadow-md cursor-pointer {selectedPath === 'example' ? 'border-primary ring-2 ring-primary/20' : ''}">
                <Card.Header class="pb-2">
                  <div class="flex items-center gap-3">
                    <div class="w-12 h-12 rounded-lg bg-success/10 flex items-center justify-center">
                      <Play class="w-6 h-6 text-success" />
                    </div>
                    <div class="flex-1">
                      <div class="flex items-center gap-2">
                        <Card.Title class="text-lg">Try an Example</Card.Title>
                        <span class="px-2 py-0.5 text-xs font-medium bg-success/10 text-success rounded-full">
                          Recommended
                        </span>
                      </div>
                      <Card.Description>
                        Jump right in with a pre-configured demo project
                      </Card.Description>
                    </div>
                    <ChevronRight class="w-5 h-5 text-muted-foreground" />
                  </div>
                </Card.Header>
              </Card.Root>
            </button>

            <!-- Set Up My Project Card -->
            <button
              onclick={() => handleChoosePath("custom")}
              class="text-left w-full"
            >
              <Card.Root class="transition-all duration-200 hover:border-primary hover:shadow-md cursor-pointer {selectedPath === 'custom' ? 'border-primary ring-2 ring-primary/20' : ''}">
                <Card.Header class="pb-2">
                  <div class="flex items-center gap-3">
                    <div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center">
                      <Settings class="w-6 h-6 text-primary" />
                    </div>
                    <div class="flex-1">
                      <Card.Title class="text-lg">Set Up My Project</Card.Title>
                      <Card.Description>
                        Configure Claude Commander for your own codebase
                      </Card.Description>
                    </div>
                    <ChevronRight class="w-5 h-5 text-muted-foreground" />
                  </div>
                </Card.Header>
              </Card.Root>
            </button>
          </div>

          <Dialog.Footer class="justify-between">
            <Button variant="ghost" onclick={handleBack}>
              Back
            </Button>
            <Button variant="ghost" onclick={handleSkip}>
              Skip for now
            </Button>
          </Dialog.Footer>
        </div>
      {/if}

      <!-- Step 3: Quick Setup -->
      {#if currentStep === 3}
        <div class="animate-slide-in">
          <Dialog.Header class="text-center mb-6">
            <Dialog.Title class="text-xl">Quick Setup</Dialog.Title>
            <Dialog.Description>
              Configure the essentials to get started
            </Dialog.Description>
          </Dialog.Header>

          <div class="space-y-4 mb-6">
            <!-- API Key Input -->
            <div class="space-y-2">
              <label for="api-key" class="text-sm font-medium flex items-center gap-2">
                API Key
                <span class="text-xs text-muted-foreground">(optional - can be set later)</span>
              </label>
              <div class="flex gap-2">
                <input
                  id="api-key"
                  type="password"
                  bind:value={apiKey}
                  placeholder="sk-..."
                  class="flex-1 h-10 px-3 rounded-md border border-input bg-background text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                />
                <Button variant="outline" onclick={handleOpenConfigFolder}>
                  <FolderOpen class="w-4 h-4" />
                </Button>
              </div>
              <p class="text-xs text-muted-foreground">
                Or set ANTHROPIC_API_KEY in your .env file
              </p>
            </div>

            <!-- Folder Picker -->
            <div class="space-y-2">
              <label for="folder" class="text-sm font-medium">
                Project Folder
              </label>
              <div class="flex gap-2">
                <input
                  id="folder"
                  type="text"
                  bind:value={selectedFolder}
                  placeholder="Select a folder..."
                  readonly
                  class="flex-1 h-10 px-3 rounded-md border border-input bg-background text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 cursor-pointer"
                  onclick={handleSelectFolder}
                />
                <Button variant="outline" onclick={handleSelectFolder}>
                  <FolderOpen class="w-4 h-4" />
                </Button>
              </div>
            </div>

            <!-- Project Type Selection -->
            <div class="space-y-2">
              <label class="text-sm font-medium flex items-center gap-2">
                Project Type
                <span class="text-xs text-muted-foreground">(optional)</span>
              </label>
              <div class="flex flex-wrap gap-2">
                {#each projectTypes as type}
                  <button
                    onclick={() => (projectType = type.id)}
                    class="px-3 py-1.5 text-sm rounded-md border transition-colors {projectType === type.id
                      ? 'bg-primary text-primary-foreground border-primary'
                      : 'bg-background border-input hover:bg-accent hover:text-accent-foreground'}"
                  >
                    {type.label}
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <Dialog.Footer class="justify-between">
            <Button variant="ghost" onclick={handleBack}>
              Back
            </Button>
            <Button onclick={handleContinueFromSetup} class="gap-2">
              Continue
              <ArrowRight class="w-4 h-4" />
            </Button>
          </Dialog.Footer>
        </div>
      {/if}

      <!-- Step 4: Ready -->
      {#if currentStep === 4}
        <div class="animate-slide-in">
          <div class="flex flex-col items-center text-center mb-6">
            <div class="w-16 h-16 rounded-full bg-success/10 flex items-center justify-center mb-4">
              <Check class="w-8 h-8 text-success" />
            </div>
            <Dialog.Header class="items-center">
              <Dialog.Title class="text-2xl">You're all set!</Dialog.Title>
              <Dialog.Description class="text-base mt-2">
                Claude Commander is ready to help you build amazing things.
              </Dialog.Description>
            </Dialog.Header>
          </div>

          <!-- Keyboard Shortcuts -->
          <div class="mb-6">
            <h4 class="text-sm font-medium mb-3 flex items-center gap-2">
              <Keyboard class="w-4 h-4" />
              Keyboard Shortcuts
            </h4>
            <div class="grid grid-cols-2 gap-2">
              {#each shortcuts as shortcut}
                <div class="flex items-center justify-between p-3 rounded-lg bg-muted/50 border border-border">
                  <span class="text-sm text-muted-foreground">{shortcut.description}</span>
                  <div class="flex gap-1">
                    {#each shortcut.keys as key}
                      <kbd class="px-2 py-1 text-xs font-medium bg-background border border-border rounded shadow-sm">
                        {key}
                      </kbd>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>

          <Dialog.Footer class="flex-col gap-2">
            <Button onclick={handleStartTutorial} size="lg" class="w-full gap-2">
              <Rocket class="w-4 h-4" />
              Take Interactive Tour
            </Button>
            <Button variant="ghost" onclick={handleSkip} class="w-full">
              Skip for now
            </Button>
          </Dialog.Footer>
        </div>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>

<style>
  /* Animation classes */
  :global(.animate-fade-in) {
    animation: fadeIn 0.3s ease-out;
  }

  :global(.animate-slide-in) {
    animation: slideIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  /* Input styling */
  input {
    font-family: inherit;
  }

  input:read-only {
    cursor: pointer;
  }

  /* Keyboard shortcut styling */
  kbd {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
  }
</style>
