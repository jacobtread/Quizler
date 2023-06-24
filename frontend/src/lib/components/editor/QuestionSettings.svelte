<script lang="ts">
  import { type Question } from "$api/models";
  import * as constants from "$lib/constants";

  import TimeInput from "../TimeInput.svelte";
  import FloatingModal from "../FloatingModal.svelte";

  export let question: Question;
  export let visible: boolean;
</script>

<FloatingModal bind:visible>
  <div class="section">
    <h2 class="section__title">Timing</h2>
    <p class="section__desc">
      Below you can configuring the timing for different events
    </p>
    <div class="field-group">
      <div class="field">
        <span class="field__name">Answer Time</span>
        <p class="field__desc">Time the players have to answer the question</p>
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

  <div class="section">
    <h2 class="section__title">Scoring</h2>
    <p class="section__desc">
      Score is awarded to players based on how quickly the player answers the
      question. You can configure the minimum and maximum values for this below
    </p>
    <div class=" field-group">
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
</FloatingModal>

<style lang="scss">
  @import "../../../assets/scheme.scss";

  .field-group {
    display: flex;
    gap: 1rem;
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

  @media screen and (max-width: 48rem) {
    .field-group {
      flex-flow: column;
    }
  }
</style>
