<script lang="ts">
  import { routeState } from "$lib/router";
</script>

<svelte:component this={$routeState.component} {...$routeState.props} />
