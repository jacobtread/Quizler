<div class="spin">
  <!--  -->
</div>

<style lang="scss">
  @use "../../assets/scheme.scss";

  .spin {
    width: 4rem;
    height: 4rem;
    display: block;
    border: 5px solid scheme.$spinnerFill;
    border-radius: 2rem;
    border-top-color: transparent;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    0% {
      transform: rotate(0);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
