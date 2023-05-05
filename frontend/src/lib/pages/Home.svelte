<script lang="ts">
  import { setConnect, setCreate } from "$lib/stores/state";
</script>

<h1>Quizler</h1>
<button on:click={setCreate}> Create </button>
<button on:click={setConnect}> Join </button>
