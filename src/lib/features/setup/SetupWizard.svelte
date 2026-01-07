<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { homeDir } from '@tauri-apps/api/path';
  import Button from '../../ui/Button.svelte';
  import { toast } from '../../stores/toast.svelte';
  import { saveConfig, testSshDirect, getConfigPath } from '../../api';
  import type { AppConfig } from '../../types';

  interface Props {
    onComplete: () => void;
  }

  const { onComplete }: Props = $props();

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
      // Ensure path has trailing slash
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
      // Create a temporary config for testing
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

      // Save temporarily to test
      await saveConfig(tempConfig);
      // Pass passphrase if provided (for encrypted keys)
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

  const canTest = $derived(
    sshHost.trim().length > 0 && sshUser.trim().length > 0 && sshKeyPath.trim().length > 0,
  );
  const canSave = $derived(
    connectionSuccess && remoteBase.trim().length > 0 && uvPath.trim().length > 0,
  );
</script>

<div class="min-h-screen flex items-center justify-center p-8">
  <div class="glass-panel max-w-xl w-full p-8 space-y-6">
    <!-- Header -->
    <div class="text-center space-y-2">
      <h1 class="text-2xl font-bold text-white">Bienvenue dans SolverPilot</h1>
      <p class="text-slate-400">Configurez votre connexion au serveur de calcul</p>
    </div>

    <!-- SSH Section -->
    <section class="space-y-4">
      <h2
        class="text-sm font-medium text-slate-300 uppercase tracking-wider flex items-center gap-2"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
          />
        </svg>
        Connexion SSH
      </h2>

      <div class="grid grid-cols-3 gap-4">
        <div class="space-y-1 col-span-2">
          <label for="ssh-host" class="text-sm text-slate-400">Serveur (host)</label>
          <input
            id="ssh-host"
            type="text"
            bind:value={sshHost}
            placeholder="serveur.exemple.com"
            class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <div class="space-y-1">
          <label for="ssh-port" class="text-sm text-slate-400">Port</label>
          <input
            id="ssh-port"
            type="number"
            bind:value={sshPort}
            min="1"
            max="65535"
            placeholder="22"
            class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
      </div>

      <div class="space-y-1">
        <label for="ssh-user" class="text-sm text-slate-400">Utilisateur</label>
        <input
          id="ssh-user"
          type="text"
          bind:value={sshUser}
          placeholder="username"
          class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      <div class="space-y-1">
        <label for="ssh-key" class="text-sm text-slate-400">Cle SSH</label>
        <div class="flex gap-2">
          <input
            id="ssh-key"
            type="text"
            bind:value={sshKeyPath}
            placeholder="~/.ssh/id_ed25519"
            class="flex-1 px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <Button variant="secondary" onclick={pickSshKey}>
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
              />
            </svg>
          </Button>
        </div>
        <p class="text-xs text-slate-500">Chemin vers votre cle privee SSH</p>
      </div>

      <!-- SSH Passphrase (optional) -->
      <div class="space-y-1">
        <label for="ssh-passphrase" class="text-sm text-slate-400">
          Passphrase <span class="text-slate-600">(optionnel)</span>
        </label>
        <input
          id="ssh-passphrase"
          type="password"
          bind:value={sshPassphrase}
          placeholder="Laissez vide si la cle n'est pas chiffree"
          class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <p class="text-xs text-slate-500">
          Passphrase de dechiffrement de la cle SSH (si chiffree)
        </p>
      </div>

      <!-- SSH Help -->
      <div class="bg-slate-800/30 rounded-lg p-3 text-xs text-slate-400 space-y-1">
        <p class="font-medium text-slate-300">Pas de cle SSH ?</p>
        <code class="block text-blue-400">ssh-keygen -t ed25519</code>
        <code class="block text-blue-400"
          >ssh-copy-id {sshUser || 'user'}@{sshHost || 'server'}</code
        >
      </div>

      <!-- Test Connection -->
      <div class="flex items-center gap-3">
        <Button
          variant="secondary"
          onclick={handleTestConnection}
          disabled={!canTest}
          loading={testingConnection}
        >
          {#if !testingConnection}
            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
              />
            </svg>
          {/if}
          Tester la connexion
        </Button>

        {#if connectionTested}
          {#if connectionSuccess}
            <span class="flex items-center gap-1 text-green-400 text-sm">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 13l4 4L19 7"
                />
              </svg>
              Connexion reussie
            </span>
          {:else}
            <span class="flex items-center gap-1 text-red-400 text-sm">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
              Echec
            </span>
          {/if}
        {/if}
      </div>

      {#if connectionError}
        <p class="text-sm text-red-400 bg-red-900/20 p-2 rounded">{connectionError}</p>
      {/if}
    </section>

    <!-- Remote Section -->
    <section class="space-y-4">
      <h2
        class="text-sm font-medium text-slate-300 uppercase tracking-wider flex items-center gap-2"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"
          />
        </svg>
        Dossier distant
      </h2>

      <div class="space-y-1">
        <label for="remote-base" class="text-sm text-slate-400">Chemin de base</label>
        <input
          id="remote-base"
          type="text"
          bind:value={remoteBase}
          placeholder="~/benchmarks"
          class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <p class="text-xs text-slate-500">Dossier sur le serveur pour les projets et jobs</p>
      </div>
    </section>

    <!-- Tools Section -->
    <section class="space-y-4">
      <h2
        class="text-sm font-medium text-slate-300 uppercase tracking-wider flex items-center gap-2"
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
        Outils
      </h2>

      <div class="space-y-1">
        <label for="uv-path" class="text-sm text-slate-400">Chemin uv</label>
        <input
          id="uv-path"
          type="text"
          bind:value={uvPath}
          placeholder="~/.local/bin/uv"
          class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <p class="text-xs text-slate-500">
          Gestionnaire de paquets Python (utilisez "uv" si dans le PATH)
        </p>
      </div>
    </section>

    <!-- Gurobi Section (optional) -->
    <section class="space-y-4">
      <label class="flex items-center gap-3 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={showGurobi}
          class="w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-500 focus:ring-blue-500 focus:ring-offset-slate-900"
        />
        <span class="text-sm font-medium text-slate-300 uppercase tracking-wider"
          >Configurer Gurobi (optionnel)</span
        >
      </label>

      {#if showGurobi}
        <div class="space-y-3 pl-7">
          <div class="space-y-1">
            <label for="gurobi-home" class="text-sm text-slate-400">GUROBI_HOME</label>
            <input
              id="gurobi-home"
              type="text"
              bind:value={gurobiHome}
              placeholder="~/gurobi1200/linux64"
              class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          <div class="space-y-1">
            <label for="gurobi-license" class="text-sm text-slate-400">GRB_LICENSE_FILE</label>
            <input
              id="gurobi-license"
              type="text"
              bind:value={gurobiLicense}
              placeholder="~/gurobi1200/gurobi.lic"
              class="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
        </div>
      {/if}
    </section>

    <!-- Save Button -->
    <div class="pt-4 border-t border-slate-700/50">
      <Button
        variant="primary"
        size="lg"
        class="w-full"
        onclick={handleSave}
        disabled={!canSave}
        loading={saving}
      >
        {#if !saving}
          <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            />
          </svg>
        {/if}
        Enregistrer et demarrer
      </Button>

      {#if !connectionSuccess}
        <p class="text-xs text-center text-slate-500 mt-2">
          Testez la connexion SSH avant de pouvoir sauvegarder
        </p>
      {/if}
    </div>

    <!-- Config path info -->
    <p class="text-xs text-center text-slate-600">
      Configuration: {configPath}
    </p>
  </div>
</div>
