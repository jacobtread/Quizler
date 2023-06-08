<script lang="ts">
  export let value: string;
  export let group: string;
</script>

<label class="label">
  <input class="input" type="radio" on:change bind:group {value} />

  <div class="wrapper">
    <span class="mark" />
  </div>

  <slot />
</label>

<style lang="scss">
  @import "../../assets/scheme.scss";

  .label {
    display: inline-block;
    color: #cccccc;
  }

  .wrapper {
    display: inline-block;
    background-color: $surfaceLight;
    margin-right: 0.5rem;
    line-height: 0;
    padding: 0.25rem;

    border-radius: 0.5rem;
    transition: transform 0.15s ease;

    &:active {
      transform: scale(1.1);
    }
  }

  .input {
    display: none;

    &:checked ~ .wrapper .mark {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .mark {
    display: inline-block;
    width: 1rem;
    height: 1rem;
    background-color: #ffffff;
    border-radius: 0.25rem;

    transition: opacity 0.15s ease, transform 0.15s ease;
    opacity: 0;
    transform: translateY(15px);
  }
</style>
