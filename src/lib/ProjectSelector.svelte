<script lang="ts">
  import type { Project } from './types';
  import * as api from './api';
  import { registerShortcut, unregisterShortcut } from './stores/shortcuts.svelte';

  // Portal action - moves element to body to escape backdrop-filter containing block
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  }

  // Props
  interface Props {
    activeProject: Project | null;
    onProjectChange: (project: Project | null) => void;
  }

  const { activeProject, onProjectChange }: Props = $props();

  // State
  let projects = $state<Project[]>([]);
  let showCreateModal = $state(false);
  let showSettingsModal = $state(false);
  let newProjectName = $state('');
  let newPythonVersion = $state('3.12');
  let availablePythonVersions = $state<string[]>([]);
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  // Load projects on mount
  $effect(() => {
    void loadProjects();
  });

  // Register shortcuts
  $effect(() => {
    // Register Ctrl+N
    registerShortcut({
      key: 'n',
      ctrl: true,
      action: () => void openCreateModal(),
      description: 'New Project',
    });

    // Register Escape for local modals
    registerShortcut({
      key: 'Escape',
      action: () => {
        if (showCreateModal || showSettingsModal) {
          closeModals();
        }
      },
      description: 'Close Project Modal',
    });

    return () => {
      unregisterShortcut('n');
      // We shouldn't unregister Escape globally if others use it...
      // But with our current API, unregisterShortcut('Escape') removes ALL escape handlers.
      // This is a flaw in the requested API for 'unregisterShortcut(key)'.
      // So we simply DO NOT unregister Escape here.
      // This causes a memory leak of the handler closure if ProjectSelector is destroyed and recreated often.
      // But ProjectSelector is in Header, which is permanent. So it's acceptable.

      // unregisterShortcut('Escape'); // DANGEROUS
    };
  });

  async function loadProjects() {
    try {
      projects = await api.listProjects();
    } catch (e) {
      error = `Erreur chargement projets: ${String(e)}`;
    }
  }

  async function loadPythonVersions() {
    try {
      availablePythonVersions = await api.listPythonVersions();
      // Sort versions descending (newest first)
      availablePythonVersions.sort((a, b) => {
        const aParts = a.split('.').map(Number);
        const bParts = b.split('.').map(Number);
        for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
          const aVal = aParts[i] || 0;
          const bVal = bParts[i] || 0;
          if (bVal !== aVal) return bVal - aVal;
        }
        return 0;
      });
    } catch (e) {
      error = `Erreur chargement versions Python: ${String(e)}`;
    }
  }

  async function handleSelectProject(project: Project) {
    try {
      const selected = await api.setActiveProject(project.id);
      onProjectChange(selected);
    } catch (e) {
      error = `Erreur sélection projet: ${String(e)}`;
    }
  }

  async function openCreateModal() {
    newProjectName = '';
    newPythonVersion = '3.12';
    error = null;
    await loadPythonVersions();
    showCreateModal = true;
  }

  async function createProject() {
    if (!newProjectName.trim()) {
      error = 'Le nom du projet est requis';
      return;
    }

    isLoading = true;
    error = null;

    try {
      const project = await api.createProject(newProjectName.trim(), newPythonVersion);
      projects = [...projects, project];
      showCreateModal = false;
      // Auto-select the new project
      await handleSelectProject(project);
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  function openSettingsModal() {
    if (!activeProject) return;
    error = null;
    void loadPythonVersions();
    showSettingsModal = true;
  }

  async function changePythonVersion(version: string) {
    if (!activeProject) return;

    isLoading = true;
    error = null;

    try {
      await api.setProjectPythonVersion(version);
      // Refresh active project
      const updated = await api.getActiveProject();
      if (updated) {
        onProjectChange(updated);
        // Update projects list
        projects = projects.map(p => (p.id === updated.id ? updated : p));
      }
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  async function deleteCurrentProject() {
    if (!activeProject) return;

    if (!confirm(`Supprimer le projet "${activeProject.name}" ? Cette action est irréversible.`)) {
      return;
    }

    isLoading = true;
    error = null;

    try {
      await api.deleteProject(activeProject.id);
      projects = projects.filter(p => p.id !== activeProject.id);
      onProjectChange(null);
      showSettingsModal = false;
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  function closeModals() {
    showCreateModal = false;
    showSettingsModal = false;
    error = null;
  }
</script>

<div class="project-selector">
  <div class="flex items-center gap-2">
    <label for="project-select" class="text-sm text-gray-400">Projet:</label>

    <div class="relative">
      <select
        id="project-select"
        class="bg-zinc-800 border border-zinc-700 rounded-sm px-3 py-1.5 pr-8 text-sm focus:outline-hidden focus:border-blue-500 appearance-none cursor-pointer min-w-[160px]"
        value={activeProject?.id ?? ''}
        onchange={e => {
          const id = Number(e.currentTarget.value);
          const project = projects.find(p => p.id === id);
          if (project) void handleSelectProject(project);
        }}
        disabled={isLoading}
      >
        <option value="" disabled>Sélectionner...</option>
        {#each projects as project (project.id)}
          <option value={project.id}>{project.name}</option>
        {/each}
      </select>
      <div class="absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none text-gray-400">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </div>
    </div>

    {#if activeProject}
      <span class="text-xs text-gray-500 bg-zinc-800 px-2 py-1 rounded-sm">
        Python {activeProject.python_version}
      </span>
      <button
        class="p-1.5 hover:bg-zinc-700 rounded-sm text-gray-400 hover:text-white transition-colors"
        title="Paramètres du projet"
        onclick={openSettingsModal}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
          />
        </svg>
      </button>
    {/if}

    <button
      class="p-1.5 hover:bg-zinc-700 rounded-sm text-green-400 hover:text-green-300 transition-colors"
      title="Nouveau projet (Ctrl+N)"
      onclick={openCreateModal}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  </div>
</div>

<!-- Create Project Modal -->
{#if showCreateModal}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    use:portal
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    onclick={closeModals}
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="bg-zinc-900 border border-zinc-700 rounded-lg p-6 w-96 max-w-[90vw]"
      onclick={e => {
        e.stopPropagation();
      }}
    >
      <h2 class="text-lg font-semibold mb-4">Nouveau projet</h2>

      {#if error}
        <div
          class="bg-red-900/30 border border-red-500 text-red-300 px-3 py-2 rounded-sm mb-4 text-sm"
        >
          {error}
        </div>
      {/if}

      <div class="space-y-4">
        <div>
          <label for="project-name" class="block text-sm text-gray-400 mb-1">Nom du projet</label>
          <input
            id="project-name"
            type="text"
            class="w-full bg-zinc-800 border border-zinc-700 rounded-sm px-3 py-2 focus:outline-hidden focus:border-blue-500"
            placeholder="mon-projet"
            bind:value={newProjectName}
            disabled={isLoading}
            autofocus
          />
        </div>

        <div>
          <label for="python-version" class="block text-sm text-gray-400 mb-1">Version Python</label
          >
          <select
            id="python-version"
            class="w-full bg-zinc-800 border border-zinc-700 rounded-sm px-3 py-2 focus:outline-hidden focus:border-blue-500"
            bind:value={newPythonVersion}
            disabled={isLoading}
          >
            {#each availablePythonVersions as version (version)}
              <option value={version}>{version}</option>
            {/each}
            {#if availablePythonVersions.length === 0}
              <option value="3.12">3.12 (défaut)</option>
            {/if}
          </select>
        </div>
      </div>

      <div class="flex justify-end gap-2 mt-6">
        <button
          class="px-4 py-2 text-gray-400 hover:text-white transition-colors"
          onclick={closeModals}
          disabled={isLoading}
        >
          Annuler
        </button>
        <button
          class="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-sm transition-colors disabled:opacity-50"
          onclick={createProject}
          disabled={isLoading || !newProjectName.trim()}
        >
          {isLoading ? 'Création...' : 'Créer'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Settings Modal -->
{#if showSettingsModal && activeProject}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    use:portal
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    onclick={closeModals}
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="bg-zinc-900 border border-zinc-700 rounded-lg p-6 w-96 max-w-[90vw]"
      onclick={e => {
        e.stopPropagation();
      }}
    >
      <h2 class="text-lg font-semibold mb-4">Paramètres: {activeProject.name}</h2>

      {#if error}
        <div
          class="bg-red-900/30 border border-red-500 text-red-300 px-3 py-2 rounded-sm mb-4 text-sm"
        >
          {error}
        </div>
      {/if}

      <div class="space-y-4">
        <div>
          <label for="settings-python-version" class="block text-sm text-gray-400 mb-1"
            >Version Python</label
          >
          <select
            id="settings-python-version"
            class="w-full bg-zinc-800 border border-zinc-700 rounded-sm px-3 py-2 focus:outline-hidden focus:border-blue-500"
            value={activeProject.python_version}
            onchange={e => void changePythonVersion(e.currentTarget.value)}
            disabled={isLoading}
          >
            {#each availablePythonVersions as version (version)}
              <option value={version}>{version}</option>
            {/each}
          </select>
        </div>

        <div class="pt-4 border-t border-zinc-700">
          <button
            class="w-full px-4 py-2 bg-red-600/20 text-red-400 hover:bg-red-600/30 rounded-sm transition-colors disabled:opacity-50"
            onclick={deleteCurrentProject}
            disabled={isLoading}
          >
            Supprimer le projet
          </button>
          <p class="text-xs text-gray-500 mt-2">
            Cette action supprimera le dossier du projet et toutes ses références.
          </p>
        </div>
      </div>

      <div class="flex justify-end mt-6">
        <button
          class="px-4 py-2 text-gray-400 hover:text-white transition-colors"
          onclick={closeModals}
          disabled={isLoading}
        >
          Fermer
        </button>
      </div>
    </div>
  </div>
{/if}
