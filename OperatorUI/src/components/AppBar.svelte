<TopAppBar
  variant="static"
  {prominent}
  {dense}
  color={secondaryColor ? 'secondary' : 'primary'}
>
  <Row class="appbar-flex-container">
    <Section>
      <Title>ISU AGN-201</Title>
    </Section>
    <div class="led-box">
      <div class={`led-circle led-circle-green ${!live ? 'off' : ''}`}></div>
      <p>Live</p>
    </div>
  </Row>
</TopAppBar>

<script lang="ts">
  import TopAppBar, { Row, Section, Title } from '@smui/top-app-bar';

  let prominent = false;
  let dense = false;
  let secondaryColor = false;

  export let live: any;
</script>

<style lang="scss">
  :global(.mdc-top-app-bar__title) {
    padding: 0px!important;
  }

  :global(.appbar-flex-container) {
    padding: 12px!important;
    display: flex;
    justify-content: space-between;
  }

  :global(.led-box) {
    margin-right: 8px;
    padding: 5px 10px 3px;
    display: flex;
    align-items: center;
    background-color: rgb(33, 33, 37);
    border-radius: 4px;
  }

  :global(.led-circle) {
    margin: 0 10px 1.5px 0;
    width: 14px;
    height: 14px;
    border-radius: 50%;
  }

  :global(.led-circle-green) {
    background-color: #ABFF00;
    box-shadow: rgba(0, 0, 0, 0.2) 0 -1px 7px 1px, inset #304701 0 -1px 9px, #89FF00 0 1px 7px;

    &.off {
      background-color: #365100;
      box-shadow: rgba(0, 0, 0, 0.4) 0 -1px 7px 1px, inset rgb(33, 33, 33) 0 -1px 9px;
    }
  }

  :global(.led-circle-yellow) {
    background-color: #ABFF00;
    box-shadow: rgba(0, 0, 0, 0.2) 0 -1px 7px 1px, inset #304701 0 -1px 9px, #89FF00 0 1px 7px;
  }

  :global(.led-circle-red) {
    background-color: #ABFF00;
    box-shadow: rgba(0, 0, 0, 0.2) 0 -1px 7px 1px, inset #304701 0 -1px 9px, #89FF00 0 1px 7px;
  }
</style>
