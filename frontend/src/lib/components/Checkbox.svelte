<script lang="ts">
  export let value: boolean;
</script>

<label class="checkbox">
  <input class="checkbox__value" type="checkbox" bind:checked={value} />
  <svg
    width="16"
    height="16"
    viewBox="0 0 12 9"
    xmlns="http://www.w3.org/2000/svg"
    class="checkbox__mark"
  >
    <path
      d="M4.575 8.977s-.404-.007-.536-.165L.057 5.64l.829-1.227L4.47 7.268 10.921.086l.923 1.103-6.863 7.64c-.134-.003-.406.148-.406.148z"
      fill="#FFF"
      fill-rule="evenodd"
    />
  </svg>
</label>

<style lang="scss">
  @import "../../assets/scheme.scss";

  .checkbox {
    display: inline-block;

    background-color: $surfaceLight;

    width: 2rem;
    height: 2rem;

    border-radius: 0.25rem;
    transition: transform 0.15s ease;

    &:active {
      transform: scale(1.1);
    }

    &__value {
      display: none;

      &:checked ~ .checkbox__mark {
        opacity: 1;
        transform: translateY(0);
      }
    }

    &__mark {
      display: block;
      width: 1rem;
      height: 1rem;
      margin: 0.5rem;
      color: #ffffff;

      transition: opacity 0.15s ease, transform 0.15s ease;
      opacity: 0;
      transform: translateY(15px);
    }
  }
</style>
