<script lang="ts">
    import {onMount} from 'svelte';
    import Plotly from 'plotly.js-dist-min'
    import Button from '@smui/button';
    import {Label} from "@smui/button";
    import LayoutGrid, {Cell} from '@smui/layout-grid';
    import {invoke} from "@tauri-apps/api/tauri";
    import Select, {Option} from '@smui/select';
    import {listen} from '@tauri-apps/api/event';
    import Checkbox from '@smui/checkbox';
    import FormField from '@smui/form-field';
    import { message, errorMessage } from './store/index.js';
    import Switch from '@smui/switch';

  
    // Custom Components
    import AppBar from './components/AppBar.svelte';
    import AlertBanner from './components/AlertBanner.svelte';
    import ContentCard from './components/ContentCard.svelte';
    import PlotlyGraph from './components/PlotlyGraph.svelte';
  
    let data: FinalizedData | null = null
    let dates: any = []
    let selectedDate: string | null = null
    let selectedVar: string | null = null
    let live = false
    let limit: number | null = null
    let limits = [
      {value: 1000, label: '1,000'},
      {value: 5000, label: '5,000'},
      {value: 10000, label: '10,000'},
      {value: 100000, label: '100,000'}
    ]
    let graphElement: HTMLDivElement | null = null;
  
    type FinalizedData = {
      ccr_cm: Data[],
      ch1_cps: Data[],
      ch2_watts: Data[],
      ch3_watts: Data[],
      fcr_cm: Data[],
      inv_period: Data[],
      temp: Data
    }
  
    type Data = {
      date_time: string,
      delta: number,
      predicted: number,
      reported: number,
      time: number
    }
  
    function drawGraph(variable: string) {
      if (!variable || !graphElement) return;
  
      let selected: Data[] = data[variable];
  
      const predicted = {
        x: selected.map(d => d.time),
        y: selected.map(d => d.predicted),
        mode: 'lines+markers',
        name: "Predicted"
      }
  
  
      const actual = {
        x: selected.map(d => d.time),
        y: selected.map(d => d.reported),
        mode: 'lines+markers',
        name: "Actual"
      }
  
      const toRender = [predicted, actual]
      const layout = {
        autosize: true,
        plot_bgcolor: 'transparent',
        paper_bgcolor: 'transparent',
        font: {
          // family: 'Source Sans Pro, sans-serif',
          color: '#ffffff',
        },
        responsive: true,
        margin: {
          t: 20,
          l: 80,
          b: 90,
          r: 40,
        },
        xaxis: {
          title: {
            text: `x Axis`,
          },
          autorange: true,
    //       linecolor: '#ffffff',
    //       zerolinecolor: '#969696',
    // zerolinewidth: 10,
          // ticks: 'outside',
          // rangemode: 'tozero',
        },
        yaxis: {
          title: {
            text: `y Axis`,
          },
          autorange: true,
          // linecolor: '#ffffff',
          // ticks: 'outside',
          // rangemode: 'tozero',
        },
      }
  
      Plotly.react(graphElement, toRender, layout)
    }

    onMount(() => {
      listen<string>('tauri://resize', (event) => {
        location.reload()
      });

      graphElement = document.getElementById('graph') as HTMLDivElement;
  
      invoke("fetch_run_dates")
        .then((d) => {
          dates = d
        })
        .catch((e) => {
          errorMessage.set(e); // Set errorMessage in the store
        })

  
      let timer = setTimeout(function load() {
        reload()
          .finally(() => timer = setTimeout(load, 5000))
      }, 5000)
  

    })
  
    function reload(): Promise<void> {
      return new Promise(resolve => {
        invoke("manual_load")
          .catch(e => {
            errorMessage.set(e); // Set errorMessage in the store
          })
          .finally(() => resolve());
      });
    }
  
    function manualLoad() {
      invoke("manual_load")
        .then(() => {
          message.set("Successfully loaded latest data from DeepLynx"); // Set message in the store
  
          fetchData(selectedDate, limit)
            .then(() => {
              if (selectedVar) drawGraph(selectedVar)
            })
        })
        .catch((e) => {
          errorMessage.set(e); // Set errorMessage in the store
        })
    }
  
    function fetchData(dateTime: string, limit: number): Promise<void> {
      if (!dateTime) return;
  
      return new Promise(resolve => {
        invoke("fetch_data", {lastDatetime: dateTime, lastTime: 0, limit})
          .then((d) => {
            data = d[dateTime] as FinalizedData
            resolve()
          })
          .catch((e) => {
            errorMessage.set(e); // Set errorMessage in the store
          })
      })
    }
  
    $: 
      {
        if (limit !== null && limit !== undefined) {
          fetchData(selectedDate, limit)
        }
      }
    $: {
      if (selectedVar && graphElement) {
        drawGraph(selectedVar);
      }
    }
  </script>
  
  <main class="container app-body">
    <!-- <link
      rel="stylesheet"
      href="/svelte-dark.min.css"
      media="screen and (prefers-color-scheme: dark)"
    /> -->
    <!-- Material Icons -->
    <link
      rel="stylesheet"
      href="https://fonts.googleapis.com/icon?family=Material+Icons"
    />
    <!-- Roboto -->
    <link
      rel="stylesheet"
      href="https://fonts.googleapis.com/css?family=Roboto:300,400,500,600,700"
    />
    <!-- Roboto Mono -->
    <link
      rel="stylesheet"
      href="https://fonts.googleapis.com/css?family=Roboto+Mono"
    />
      <div id="header">
        <AppBar {live} />
        <AlertBanner />
      </div>

      <div id="content">
        <ContentCard title={'Data Analysis and Monitoring'}>
          <div style="position: absolute; right: 0px; top: 0px; padding: 4px 20px;">
            <FormField>
              <Switch bind:checked={live} />
              <span slot="label">Live Data Monitoring</span>
            </FormField>
          </div>

          <LayoutGrid>
            <Cell align="middle" span={12}>
              <div class="cell">
                <LayoutGrid>      
                  <Cell span={4}>
                    <div class="cell">
                      <Select variant="outlined" bind:value={selectedDate} disabled={dates.length <= 0} label="Run Date/Time">
                        {#each dates as date}
                          <Option value={date}>{date}</Option>
                        {/each}
                      </Select>
                    </div>
                  </Cell>
                  <Cell span={4}>
                    <div class="cell">
                      <Select
                        variant="outlined"
                        key={(limit) => `${limit ? limit.value : ''}`}
                        bind:value={limit}
                        disabled={!selectedDate}
                        label="# of Data Points"
                      >
                        {#each limits as limit (limit.label)}
                          <Option value={limit.value}>{limit.label}</Option>
                        {/each}
                      </Select>
                    </div>
                  </Cell>
                  <Cell span={4}>
                    <div class="cell">
                      <Select
                        variant="outlined"
                        bind:value={selectedVar}
                        disabled={!data || limit === null}
                        label="Monitored Variable"
                      >
                        {#if data}
                          {#each Object.keys(data) as variable}
                            <Option value={variable}>{variable}</Option>
                          {/each}
                        {/if}
                      </Select>
                    </div>
                  </Cell>

                  {#if selectedVar && !live}
                    <Cell span={12}>
                      <div class="cell">
                        <Button on:click={() => manualLoad()} variant="raised" color="secondary">
                          <Label>Reload From DeepLynx</Label>
                        </Button>
                      </div>
                    </Cell>
                  {/if}
                </LayoutGrid>
              </div>
            </Cell>
          </LayoutGrid>
        </ContentCard>
        <ContentCard title={'Variable Trend Visualization'}>
          {#if !selectedVar}
          <div style="background-color: rgb(45, 45, 45); display: flex; align-content: center; justify-content: center; padding: 20px; margin: 20px">
            No data to display. Please select data to analyze.
          </div>
          {/if}
          
          <PlotlyGraph />
        </ContentCard>
      </div>

    
  </main>
  
  <style lang="scss">
    :global(#header) {
      position: sticky;
      top: 0;
      z-index: 2;
    }

    :global(.cell .mdc-select) {
      width: 100%;
    }
  
    // code,
    // pre {
    //   font-family: 'Roboto Mono', monospace;
    // }
  
    // small {
    //   font-size: 0.9em;
    // }
  
    // big {
    //   font-size: 1.1em;
    // }
  
    // b,
    // strong {
    //   font-weight: bold;
    // }
  </style>