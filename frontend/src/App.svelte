<script lang="ts">
  import { socketReady } from "$lib/socket";
  import Loading from "$lib/pages/Loading.svelte";
  import GlobalDialog from "$lib/components/GlobalDialog.svelte";
  import Router from "$lib/components/Router.svelte";
</script>

{#if $socketReady}
  <Router />
{:else}
  <Loading />
{/if}

<GlobalDialog />
