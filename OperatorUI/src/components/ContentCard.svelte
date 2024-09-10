<Card class="custom-card">
  <div class="card-title-container">
    <div class="card-title">
      <h2 class="mdc-typography--body1 card-title-heading">
        { title }
      </h2>
    </div>
  </div>
  <slot />
</Card>

<script lang="ts">
  import Card from '@smui/card';

  export let title: string;
</script>

<style lang="scss">
  :global(.custom-card) {
    margin: 20px;
    position: relative;

    .card-title-container {
      .card-title {
        display: inline-block;
        position: relative;

        .card-title-heading {
          margin: 0;
          background-color: var(--mdc-theme-primary);
          display: inline-block;
          padding: .6rem 1rem;
          border-top-left-radius: 4px;
        }

        &:before {
          content: '';
          position: absolute;
          top: 25px; right: 0;
          border-bottom: 18px solid rgb(33, 33, 37);
          border-left: 18px solid var(--mdc-theme-primary);
          width: 0;
        }
      }
    }
  }
</style>