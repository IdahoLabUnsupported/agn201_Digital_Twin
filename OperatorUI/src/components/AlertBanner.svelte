<Banner bind:open on:SMUIBanner:closed={handleDisplayBanner}>
  <Label slot="label">
    {#if $message !== ''}
      {$message}
    {:else if $errorMessage !== ''}
      ERROR: {$errorMessage}
    {/if}
  </Label>
  <svelte:fragment slot="actions">
    {#if $errorMessage !== ''}
      <Button on:click={() => location.reload()} class="custom-button" variant="raised" color="secondary">Acknowledge and Reload</Button>
    {:else if $message !== ''}
      <Button on:click={() => handleUpdateMessage('')} class="custom-button" variant="raised" color="secondary">Acknowledge</Button>
    {/if}
  </svelte:fragment>
</Banner>

<script lang="ts">
  import Button from '@smui/button';
  import Banner, { Label } from '@smui/banner';
  import { message, errorMessage } from '../store/index.js';

  let open = false;

  const handleDisplayBanner = () => {
    const messageValue = $message;
    const errorMessageValue = $errorMessage;
    open = messageValue !== '' || errorMessageValue !== '';
  }

  const handleUpdateMessage = (newMessage: string) => {
    message.set(newMessage);
    open = false;
  }

  $: {
    if ($message !== '' || $errorMessage !== '') {
      handleDisplayBanner();
    }
  }
</script>

<style>
  :global(.mdc-banner__fixed) {
    height: 64px !important;
  }

  :global(.mdc-banner__content)  {
    display: flex;
    flex-direction: row;
    width: 100%;
    height: 100%;
    max-width: unset !important;
  }

  :global(.mdc-banner__graphic-text-wrapper) {
    display: flex;
    flex: 1;
    margin-left: 0px!important;

  }

  :global(.mdc-banner__text) {
    margin: 0 !important;
    padding: 24px;
  }

  :global(.mdc-banner__actions) {
    padding: 8px 20px !important;
  }
</style>