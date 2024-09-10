
<!-- Plotly Graph Attaches to this div -->
<div id="graph"></div>



<script lang="ts">
  import {onMount} from 'svelte';
  import Plotly from 'plotly.js-dist-min'
  import {invoke} from "@tauri-apps/api/tauri";
  import {listen} from '@tauri-apps/api/event';

  import { message, errorMessage } from '../store/index.js';

  let data: FinalizedData | null = null
  let dates: any = []
  let selectedDate: string | null = null
  let selectedVar: string | null = null
  let live = false
  let limit = 1000

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

  let graphElement: HTMLDivElement | null = null;

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

    // const layout = {
    //   autosize: true,
    //   plot_bgcolor: 'transparent',
    //   paper_bgcolor: 'transparent',
    //   font: {
    //     // family: 'Source Sans Pro, sans-serif',
    //     color: '#ffffff',
    //   },
    //   responsive: true,
    //   margin: {
    //     t: 20,
    //     l: 80,
    //     b: 90,
    //     r: 40,
    //   },
    //   xaxis: {
    //     title: {
    //       text: `x Axis`,
    //     },
    //     showgrid: true,
    //     zeroline: true,
    //     showline: true,
    //     gridcolor: '#3b3b3b',
    //     gridwidth: 2,
    //     zerolinecolor: '#ffffff',
    //     zerolinewidth: 1,
    //     linecolor: '#3b3b3b',
    //   },
    //   yaxis: {
    //     title: {
    //       text: `y Axis`,
    //     },
    //     showgrid: true,
    //     zeroline: true,
    //     showline: true,
    //     gridcolor: '#3b3b3b',
    //     gridwidth: 2,
    //     zerolinecolor: '#ffffff',
    //     zerolinewidth: 1,
    //     linecolor: '#3b3b3b',
    //   },
    // }

    // Plotly.newPlot('graph', [], layout)

    let timer = setTimeout(function load() {
      reload()
        .finally(() => timer = setTimeout(load, 5000))
    }, 5000)

    setInterval(() => {
      if(!live || !selectedDate) return;

      fetchData(selectedDate, limit)
        .then(() => {
          if (selectedVar) drawGraph(selectedVar)
        })
    }, 5000)

    // setInterval(() => {
    //   if (!live || !selectedDate) return;

    //   fetchData(selectedDate, limit)
    //     .then(() => {
    //       if (selectedVar !== null) {
    //         drawGraph(selectedVar);
    //       }
    //     });
    // }, 5000);
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

  $: fetchData(selectedDate, limit)
  $: {
    if (selectedVar && graphElement) {
      drawGraph(selectedVar);
    }
  }
</script>


<style lang="scss">

</style>