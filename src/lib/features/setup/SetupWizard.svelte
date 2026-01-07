<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { homeDir } from '@tauri-apps/api/path';
  import Button from '../../ui/Button.svelte';
  import { toast } from '../../stores/toast.svelte';
  import { saveConfig, testSshDirect, getConfigPath } from '../../api';
  import type { AppConfig } from '../../types';
  import { fade } from 'svelte/transition';

  interface Props {
    onComplete: () => void;
  }

  const { onComplete }: Props = $props();

  // Wizard state
  let currentStep = $state(1);
  const totalSteps = 4;

  // Form state
  let sshHost = $state('');
  let sshUser = $state('');
  let sshPort = $state(22);
  let sshKeyPath = $state('');
  let sshPassphrase = $state('');
  let remoteBase = $state('~/benchmarks');
  let uvPath = $state('~/.local/bin/uv');

  // Gurobi (optional)
  let showGurobi = $state(false);
  let gurobiHome = $state('');
  let gurobiLicense = $state('');

  // UI state
  let testingConnection = $state(false);
  let connectionTested = $state(false);
  let connectionSuccess = $state(false);
  let connectionError = $state('');
  let saving = $state(false);
  let configPath = $state('');

  // Initialize default key path
  $effect(() => {
    void initDefaults();
  });

  async function initDefaults() {
    try {
      const home = await homeDir();
      const homePath = home.endsWith('/') || home.endsWith('\\') ? home : `${home}/`;
      sshKeyPath = `${homePath}.ssh/id_ed25519`;
      configPath = await getConfigPath();
    } catch {
      sshKeyPath = '~/.ssh/id_ed25519';
    }
  }

  async function pickSshKey() {
    try {
      const home = await homeDir();
      const homePath = home.endsWith('/') || home.endsWith('\\') ? home : `${home}/`;
      const selected = await open({
        title: 'Selectionner la cle SSH',
        defaultPath: `${homePath}.ssh`,
        multiple: false,
      });
      if (selected !== null) {
        sshKeyPath = selected;
        // Reset connection status when key changes
        connectionTested = false;
        connectionSuccess = false;
      }
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      toast.error(`Erreur selection fichier: ${errorMessage}`);
    }
  }

  async function handleTestConnection() {
    if (!sshHost || !sshUser || !sshKeyPath) {
      toast.warning('Remplissez tous les champs SSH avant de tester');
      return;
    }

    testingConnection = true;
    connectionTested = false;
    connectionError = '';

    try {
      const tempConfig: AppConfig = {
        ssh: {
          host: sshHost,
          user: sshUser,
          port: sshPort,
          key_path: sshKeyPath,
        },
        remote: { remote_base: remoteBase },
        polling: { interval_seconds: 2 },
        gurobi: { home: '', license_file: '' },
        tools: { uv_path: uvPath },
      };

      await saveConfig(tempConfig);
      await testSshDirect(sshPassphrase || undefined);

      connectionTested = true;
      connectionSuccess = true;
      toast.success('Connexion SSH reussie !');
    } catch (e) {
      connectionTested = true;
      connectionSuccess = false;
      const errorMessage = e instanceof Error ? e.message : String(e);
      connectionError = errorMessage;
      toast.error(errorMessage);
    } finally {
      testingConnection = false;
    }
  }

  async function handleSave() {
    if (!connectionSuccess) {
      toast.warning('Testez la connexion avant de sauvegarder');
      return;
    }

    saving = true;

    try {
      const config: AppConfig = {
        ssh: {
          host: sshHost,
          user: sshUser,
          port: sshPort,
          key_path: sshKeyPath,
        },
        remote: { remote_base: remoteBase },
        polling: { interval_seconds: 2 },
        gurobi: {
          home: showGurobi ? gurobiHome : '',
          license_file: showGurobi ? gurobiLicense : '',
        },
        tools: { uv_path: uvPath },
      };

      await saveConfig(config);
      toast.success('Configuration sauvegardee !');
      onComplete();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      toast.error(`Erreur sauvegarde: ${errorMessage}`);
    } finally {
      saving = false;
    }
  }

  function nextStep() {
    if (currentStep < totalSteps) {
      currentStep++;
    }
  }

  function prevStep() {
    if (currentStep > 1) {
      currentStep--;
    }
  }

  // Validation for steps
  const step1Valid = $derived(sshHost.trim().length > 0 && sshUser.trim().length > 0);
  const step2Valid = $derived(sshKeyPath.trim().length > 0 && connectionSuccess);
  const step3Valid = $derived(remoteBase.trim().length > 0 && uvPath.trim().length > 0);

  const canProceed = $derived.by(() => {
    switch (currentStep) {
      case 1:
        return step1Valid;
      case 2:
        return step2Valid;
      case 3:
        return step3Valid;
      case 4:
        return true; // Can always try to save if we got here
      default:
        return false;
    }
  });

  const steps = [
    { num: 1, label: 'Serveur' },
    { num: 2, label: 'Auth' },
    { num: 3, label: 'Distant' },
    { num: 4, label: 'Options' },
  ];
</script>

<div class="min-h-screen flex items-center justify-center p-4 bg-slate-950 text-slate-200">
  <div class="glass-panel max-w-2xl w-full flex flex-col max-h-[90vh]">
    <!-- Header -->
    <div class="p-6 border-b border-slate-700/50">
      <h1 class="text-xl font-bold text-white text-center mb-1">Configuration SolverPilot</h1>
      <p class="text-slate-400 text-sm text-center mb-6">
        Suivez les etapes pour configurer votre connexion
      </p>

      <!-- Stepper -->
      <div class="flex items-center justify-between px-4 relative">
        <div class="absolute left-0 top-1/2 -translate-y-1/2 w-full h-0.5 bg-slate-800 -z-10"></div>
        {#each steps as step (step.num)}
          <div class="flex flex-col items-center gap-2 bg-slate-900/80 px-2 rounded-full z-10">
            <div
              class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold border-2 transition-colors duration-200
              {currentStep >= step.num
                ? 'bg-blue-600 border-blue-600 text-white'
                : 'bg-slate-800 border-slate-600 text-slate-500'}"
            >
              {#if currentStep > step.num}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              {:else}
                {step.num}
              {/if}
            </div>
            <span
              class="text-xs font-medium {currentStep >= step.num
                ? 'text-blue-400'
                : 'text-slate-500'}"
            >
              {step.label}
            </span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6 relative min-h-[300px]">
      {#if currentStep === 1}
        <div in:fade={{ duration: 200 }} class="space-y-6">
          <div class="grid grid-cols-3 gap-4">
            <div class="space-y-1.5 col-span-2">
              <label for="ssh-host" class="text-sm font-medium text-slate-300">Serveur (host)</label
              >
              <input
                id="ssh-host"
                type="text"
                bind:value={sshHost}
                placeholder="serveur.exemple.com"
                class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                autofocus
              />
            </div>
            <div class="space-y-1.5">
              <label for="ssh-port" class="text-sm font-medium text-slate-300">Port</label>
              <input
                id="ssh-port"
                type="number"
                bind:value={sshPort}
                min="1"
                max="65535"
                class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
              />
            </div>
          </div>
          <div class="space-y-1.5">
            <label for="ssh-user" class="text-sm font-medium text-slate-300">Utilisateur</label>
            <input
              id="ssh-user"
              type="text"
              bind:value={sshUser}
              placeholder="username"
              class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            />
          </div>
          <div class="bg-blue-900/20 text-blue-200 text-sm p-4 rounded-lg flex gap-3">
            <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              ></path></svg
            >
            <p>Entrez l'adresse de votre serveur de calcul et votre nom d'utilisateur SSH.</p>
          </div>
        </div>
      {:else if currentStep === 2}
        <div in:fade={{ duration: 200 }} class="space-y-6">
          <div class="space-y-1.5">
            <label for="ssh-key" class="text-sm font-medium text-slate-300">Cle SSH</label>
            <div class="flex gap-2">
              <input
                id="ssh-key"
                type="text"
                bind:value={sshKeyPath}
                placeholder="~/.ssh/id_ed25519"
                class="flex-1 px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
              />
              <Button variant="secondary" onclick={pickSshKey} title="Parcourir">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"
                  />
                </svg>
              </Button>
            </div>
          </div>

          <div class="space-y-1.5">
            <label for="ssh-passphrase" class="text-sm font-medium text-slate-300">
              Passphrase <span class="text-slate-500 font-normal">(optionnel)</span>
            </label>
            <input
              id="ssh-passphrase"
              type="password"
              bind:value={sshPassphrase}
              placeholder="Si votre cle est chiffree"
              class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            />
          </div>

          <div class="pt-2">
            <Button
              variant={connectionSuccess ? 'ghost' : 'secondary'}
              class="w-full justify-between group {connectionSuccess
                ? 'bg-green-500/10 text-green-400 border-green-500/50'
                : ''}"
              onclick={handleTestConnection}
              loading={testingConnection}
            >
              <span class="flex items-center">
                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
                  />
                </svg>
                Tester la connexion
              </span>
              {#if connectionTested}
                {#if connectionSuccess}
                  <span class="flex items-center text-green-400 font-bold">
                    OK
                    <svg class="w-5 h-5 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                      ><path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M5 13l4 4L19 7"
                      /></svg
                    >
                  </span>
                {:else}
                  <span class="text-red-400 font-bold">Echec</span>
                {/if}
              {/if}
            </Button>
            {#if connectionError}
              <p
                class="text-sm text-red-400 bg-red-900/20 p-3 rounded-lg mt-3 border border-red-900/30"
              >
                {connectionError}
              </p>
            {/if}
          </div>

          {#if !connectionSuccess}
            <div class="bg-slate-800/30 rounded-lg p-3 text-xs text-slate-400">
              <p class="font-medium text-slate-300 mb-1">Besoin d'aide ?</p>
              <p>Generer une cle: <code class="text-blue-400">ssh-keygen -t ed25519</code></p>
              <p>
                Copier la cle: <code class="text-blue-400"
                  >ssh-copy-id {sshUser || 'user'}@{sshHost || 'server'}</code
                >
              </p>
            </div>
          {/if}
        </div>
      {:else if currentStep === 3}
        <div in:fade={{ duration: 200 }} class="space-y-6">
          <div class="space-y-1.5">
            <label for="remote-base" class="text-sm font-medium text-slate-300"
              >Chemin de base (distant)</label
            >
            <input
              id="remote-base"
              type="text"
              bind:value={remoteBase}
              placeholder="~/benchmarks"
              class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            />
            <p class="text-xs text-slate-500">
              Dossier sur le serveur ou seront stockes les projets.
            </p>
          </div>

          <div class="space-y-1.5">
            <label for="uv-path" class="text-sm font-medium text-slate-300"
              >Chemin de l'outil uv</label
            >
            <input
              id="uv-path"
              type="text"
              bind:value={uvPath}
              placeholder="~/.local/bin/uv"
              class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            />
            <p class="text-xs text-slate-500">Chemin vers l'executable 'uv' sur le serveur.</p>
          </div>
        </div>
      {:else if currentStep === 4}
        <div in:fade={{ duration: 200 }} class="space-y-6">
          <div class="bg-slate-800/30 p-4 rounded-xl border border-slate-700/30">
            <label class="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={showGurobi}
                class="mt-1 w-5 h-5 rounded border-slate-600 bg-slate-800 text-blue-500 focus:ring-blue-500 focus:ring-offset-slate-900"
              />
              <div>
                <span class="text-base font-medium text-slate-200 block">Configurer Gurobi</span>
                <span class="text-sm text-slate-500 block"
                  >Activez si vous utilisez le solveur Gurobi.</span
                >
              </div>
            </label>
          </div>

          {#if showGurobi}
            <div class="space-y-4 pl-2 border-l-2 border-slate-700/50 ml-2" transition:fade>
              <div class="space-y-1.5">
                <label for="gurobi-home" class="text-sm font-medium text-slate-300"
                  >GUROBI_HOME</label
                >
                <input
                  id="gurobi-home"
                  type="text"
                  bind:value={gurobiHome}
                  placeholder="~/gurobi1200/linux64"
                  class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                />
              </div>
              <div class="space-y-1.5">
                <label for="gurobi-license" class="text-sm font-medium text-slate-300"
                  >GRB_LICENSE_FILE</label
                >
                <input
                  id="gurobi-license"
                  type="text"
                  bind:value={gurobiLicense}
                  placeholder="~/gurobi1200/gurobi.lic"
                  class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                />
              </div>
            </div>
          {/if}

          <div class="text-sm text-slate-400 mt-4 bg-slate-900/50 p-4 rounded-lg">
            <p class="font-bold text-slate-300 mb-2">Resume :</p>
            <ul class="space-y-1 list-disc list-inside">
              <li>Serveur: <span class="text-white">{sshUser}@{sshHost}:{sshPort}</span></li>
              <li>Cle: <span class="text-white">{sshKeyPath.split(/[/\\]/).pop()}</span></li>
              <li>Remote: <span class="text-white">{remoteBase}</span></li>
            </ul>
          </div>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="p-6 border-t border-slate-700/50 flex justify-between items-center bg-slate-900/30">
      <div class="text-xs text-slate-600 truncate max-w-[200px]" title={configPath}>
        {configPath}
      </div>
      <div class="flex gap-3">
        {#if currentStep > 1}
          <Button variant="secondary" onclick={prevStep}>Precedent</Button>
        {/if}

        {#if currentStep < totalSteps}
          <Button variant="primary" onclick={nextStep} disabled={!canProceed}>Suivant</Button>
        {:else}
          <Button variant="primary" onclick={handleSave} loading={saving}>Terminer</Button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  /* Scrollbar customization for the content area */
  .overflow-y-auto::-webkit-scrollbar {
    width: 6px;
  }
  .overflow-y-auto::-webkit-scrollbar-track {
    background: transparent;
  }
  .overflow-y-auto::-webkit-scrollbar-thumb {
    background-color: rgba(148, 163, 184, 0.2);
    border-radius: 20px;
  }
</style>
