<script lang="ts">
  import GlobalDialog from "$components/GlobalDialog.svelte";
  import Router from "$components/Router.svelte";
  import Loading from "$pages/Loading.svelte";
  import { socketReady } from "$lib/socket";
</script>

{#if $socketReady}
  <Router />
{:else}
  <Loading />
{/if}

<GlobalDialog />
