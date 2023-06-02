<script lang="ts">
  import Close from "$components/icons/Delete.svelte";

  import { type Question } from "$api/models";
  import * as constants from "$lib/constants";

  import TimeInput from "../TimeInput.svelte";

  export let question: Question;
  export let visible: boolean;
</script>

<div class="floating-wrapper">
  <div class="dialog">
    <button
      on:click={() => (visible = false)}
      class="btn btn--icon btn--surface"
    >
      <Close />
      Close
    </button>

    <div class="group">
      <h2 class="group__title">Timing</h2>
      <p class="group__desc">
        Below you can configuring the timing for different events
      </p>
      <div class="group__value field-group">
        <div class="field">
          <span class="field__name">Answer Time</span>
          <p class="field__desc">
            Time the players have to answer the question
          </p>
          <TimeInput
            bind:value={question.answer_time}
            min={constants.MIN_ANSWER_TIME}
            max={constants.MAX_ANSWER_TIME}
          />
        </div>

        <div class="field">
          <span class="field__name">Bonus Score Time</span>
          <p class="field__desc">
            Time the players must answer within for bonus scores
          </p>
          <TimeInput
            bind:value={question.bonus_score_time}
            min={constants.MIN_BONUS_TIME}
            max={constants.MAX_BONUS_TIME}
          />
        </div>
      </div>
    </div>

    <div class="group">
      <h2 class="group__title">Scoring</h2>
      <p class="group__desc">
        Score is awarded to players based on how quickly the player answers the
        question. You can configure the minimum and maximum values for this
        below
      </p>
      <div class="group__value field-group">
        <label class="field">
          <span class="field__name">Min Score</span>
          <p class="field__desc">
            The minimum amount of score to award for this question
          </p>
          <input
            class="input"
            type="number"
            min={constants.MIN_SCORE}
            max={question.scoring.max_score}
            bind:value={question.scoring.min_score}
          />
        </label>
        <label class="field">
          <span class="field__name">Max Score</span>
          <p class="field__desc">
            The maximum amount of score to award for this question
          </p>
          <input
            class="input"
            type="number"
            min={question.scoring.min_score}
            max={constants.MAX_MAX_SCORE}
            bind:value={question.scoring.max_score}
          />
        </label>
        <label class="field">
          <span class="field__name">Bonus Score</span>
          <p class="field__desc">
            The amount of score to add for being within the bonus time
          </p>
          <input
            class="input"
            type="number"
            min={constants.MIN_SCORE}
            max={constants.MAX_BONUS_SCORE}
            bind:value={question.scoring.bonus_score}
          />
        </label>
      </div>
    </div>
  </div>
</div>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .group {
    &__title {
      color: #ffffff;
      margin-bottom: 0.25rem;
    }

    &__desc {
      margin-bottom: 0.5rem;
    }
  }

  .floating-wrapper {
    z-index: 1;
  }

  .dialog {
    background-color: $surface;

    border-radius: 0.5rem;

    width: 100%;
    max-width: 48rem;

    margin: 1rem;
    padding: 1rem;

    display: flex;
    flex-flow: column;
    gap: 1rem;
  }

  .field {
    display: block;
    background-color: $surface;
    border-radius: 0.55rem;

    &__name {
      font-weight: bold;
      color: #ffffff;
    }

    &__desc {
      color: #cccccc;
      margin-bottom: 0.25rem;
    }
  }

  .input {
    display: block;
    margin-top: 0.25rem;
    width: 100%;
    padding: 0.5rem;
    border: none;
    background-color: $surfaceLight;
    border-radius: 0.25rem;
    margin-top: 0.5rem;
    font-size: 1rem;
    line-height: 1.5;
  }
</style>
