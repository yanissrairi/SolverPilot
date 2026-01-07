<script lang="ts">
  import Modal from '../../ui/Modal.svelte';
  import Button from '../../ui/Button.svelte';

  let {
    open = $bindable(false),
    onconfirm,
    oncancel,
  }: {
    open?: boolean;
    onconfirm: (passphrase: string) => void;
    oncancel: () => void;
  } = $props();

  let passphrase = $state('');
  let showPassword = $state(false);

  function handleConfirm() {
    onconfirm(passphrase);
    passphrase = '';
    open = false;
  }

  function handleCancel() {
    passphrase = '';
    oncancel();
    open = false;
  }
</script>

<Modal bind:open title="SSH Authentification" closable={false} size="sm">
  <div class="space-y-4">
    <p class="text-sm text-slate-300">
      Veuillez entrer la passphrase pour déverrouiller votre clé SSH.
    </p>

    <div class="relative">
      <input
        type={showPassword ? 'text' : 'password'}
        bind:value={passphrase}
        class="w-full bg-slate-950/50 border border-slate-700 rounded-lg py-2.5 pl-3 pr-10 text-sm text-slate-200 placeholder:text-slate-500 focus:outline-hidden focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all"
        placeholder="Passphrase"
        onkeydown={e => e.key === 'Enter' && passphrase && handleConfirm()}
      />
      <button
        type="button"
        class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 p-1.5 rounded-sm focus:outline-hidden focus:text-white"
        onclick={() => (showPassword = !showPassword)}
        aria-label={showPassword ? 'Masquer le mot de passe' : 'Afficher le mot de passe'}
      >
        {#if showPassword}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            class="w-4 h-4"
          >
            <path
              fill-rule="evenodd"
              d="M3.28 2.22a.75.75 0 00-1.06 1.06l14.5 14.5a.75.75 0 101.06-1.06l-1.745-1.745A10.551 10.551 0 012.25 10.5a10.499 10.499 0 019.565-5.541l-1.55 1.55c-.56-.606-1.328-1.01-2.265-1.01a3.5 3.5 0 00-3.5 3.5c0 .937.404 1.705 1.01 2.265L3.28 2.22zM6.75 6.75A.75.75 0 005.25 5.25v1.5zm.975 6.32l1.69 1.69c-.612.046-1.226.046-1.84 0l.15-.15z"
              clip-rule="evenodd"
            />
          </svg>
        {:else}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            class="w-4 h-4"
          >
            <path d="M10 12.5a2.5 2.5 0 100-5 2.5 2.5 0 000 5z" />
            <path
              fill-rule="evenodd"
              d="M.664 10.59a1.651 1.651 0 010-1.186A10.004 10.004 0 0110 3c4.257 0 7.893 2.66 9.336 6.41.147.381.146.804 0 1.186A10.004 10.004 0 0110 17c-4.257 0-7.893-2.66-9.336-6.41zM14 10a4 4 0 11-8 0 4 4 0 018 0z"
              clip-rule="evenodd"
            />
          </svg>
        {/if}
      </button>
    </div>
  </div>

  {#snippet footer()}
    <Button variant="secondary" onclick={handleCancel}>Annuler</Button>
    <Button variant="primary" onclick={handleConfirm} disabled={!passphrase}>Confirmer</Button>
  {/snippet}
</Modal>
