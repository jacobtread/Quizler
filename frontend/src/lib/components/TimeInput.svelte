<!-- Input for deciding the amount of time something should take -->

<script lang="ts">
  // The time in milliseconds for the input
  export let value: number;

  export let min: number;
  export let max: number;

  const enum Unit {
    Seconds,
    Minutes
  }

  let unit = Unit.Seconds;
  let actualValue = convertTo(value);

  function convertFrom(value: number) {
    if (unit == Unit.Seconds) {
      return value * 1000;
    } else if (unit == Unit.Minutes) {
      return value * 1000 * 60;
    } else {
      return value;
    }
  }

  function convertTo(value: number) {
    if (unit == Unit.Seconds) {
      return value / 1000;
    } else if (unit == Unit.Minutes) {
      return value / 60 / 1000;
    } else {
      return value;
    }
  }

  /**
   * Handles updating the produced value to ensure it
   * stays within the desired range
   */
  function updateValue() {
    if (actualValue < 1) {
      actualValue = 1;
    } else {
      let converted = convertFrom(actualValue);
      if (converted > max) {
        converted = max;
      } else if (converted < min) {
        converted = min;
      }
      actualValue = convertTo(converted);
      value = converted;
    }
  }
</script>

<div class="wrapper">
  <input
    class="input"
    type="number"
    min={convertTo(min)}
    max={convertTo(max)}
    bind:value={actualValue}
    on:change={updateValue}
  />
  <select class="select" bind:value={unit} on:change={updateValue}>
    <option value={Unit.Minutes}>Minutes</option>
    <option value={Unit.Seconds}>Seconds</option>
  </select>
</div>

<style lang="scss">
  @import "../assets/scheme.scss";

  .wrapper {
    display: flex;
    gap: 0.5rem;
  }

  .select,
  .input {
    padding: 0.5rem;
    border-radius: 0.25rem;
    font-size: 1rem;
    background-color: $surfaceLight;
    border: none;
  }

  .select {
    cursor: pointer;
  }
</style>
