<!-- Input for deciding the amount of time something should take -->

<script lang="ts">
  // The time in milliseconds for the input
  export let value: number;

  export let min: number;
  export let max: number;

  const enum Unit {
    Seconds,
    Milliseconds
  }

  let unit = Unit.Seconds;
  let actualValue = convertTo(value);

  function convertFrom(value: number) {
    if (unit == Unit.Seconds) {
      return value * 1000;
    } else {
      return value;
    }
  }

  function convertTo(value: number) {
    if (unit == Unit.Seconds) {
      return value / 1000;
    } else {
      return value;
    }
  }

  function onChange() {
    let converted = convertFrom(actualValue);
    if (converted > max) {
      converted = max;
      actualValue = convertTo(max);
    } else if (converted < min) {
      converted = min;
      actualValue = convertTo(min);
    }
    value = converted;
  }
</script>

<div>
  <input
    type="number"
    min={convertTo(min)}
    max={convertTo(max)}
    bind:value={actualValue}
    on:change={onChange}
  />
  <select bind:value={unit}>
    <option value={Unit.Seconds}>Seconds</option>
    <option value={Unit.Milliseconds}>Milliseconds</option>
  </select>
</div>
