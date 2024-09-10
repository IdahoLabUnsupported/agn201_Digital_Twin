using System.Collections;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;
using Data;
using TMPro;
using System;

public class ToggleTimeseriesGraph : MonoBehaviour
{

    public GameObject[] TimeseriesGraph;
    private bool panelIsActive = false;

    private DataManager Data;
    public List<float> times;
    public List<float> data;

    public PredictedState[] predictedStates;

    private void Awake()
    {
        TimeseriesGraph = GameObject.FindGameObjectsWithTag("TimeseriesGraph");
    }
    // Start is called before the first frame update
    void Start()
    {
        Data = DataManager.GetInstance();
    }

    // Update is called once per frame
    void Update()
    {

    }

    [ContextMenu("ToggleActiveTimeseriesGraph")]
    public void ToggleActiveTimeseriesGraph()
    {
        // Debug.Log(transform.name);
        PredictedState predictedState = Data.predictedStates.FirstOrDefault(x => x.Var == transform.name);
        times = predictedState.TimeQueryChunk;

        TimeseriesGraph[0].transform.parent.gameObject.SetActive(true);
        TimeseriesGraph[0].transform.parent.GetChild(2).GetChild(1).gameObject.GetComponent<TextMeshPro>().text = gameObject.name;

        foreach (GameObject timeseriesgraph in TimeseriesGraph)
        {
            double max = 0;
            double min = 0;

            if (timeseriesgraph.name == "TimeSeriesGraph")
            {
                data = predictedState.ReportedQueryChunk;
                max = data.Max();
                min = data.Min();
            }
            else if (timeseriesgraph.name == "TimeSeriesGraph (1)")
            {
                data = predictedState.PredictedQueryChunk;
                max = data.Max();
                min = data.Min();
            }

            
            timeseriesgraph.SetActive(true);
            timeseriesgraph.transform.GetChild(2).GetComponent<ParticleGraph>().EmitParticles(times, data);

            Transform quad = timeseriesgraph.transform.GetChild(3);
            TextMeshPro label = quad.GetChild(0).GetComponent<TextMeshPro>();
            TextMeshPro yMax = quad.GetChild(3).GetComponent<TextMeshPro>();
            TextMeshPro yMin = quad.GetChild(2).GetComponent<TextMeshPro>();

            max = Math.Round(max, 2);
            min = Math.Round(min, 2);

            yMax.text = max.ToString();
            yMin.text = min.ToString();

            switch (gameObject.name)
            {
                case "Ch2_Watts":
                    label.text = "Watts (W)";
                    yMax.text = "5";
                    yMin.text = "0";
                    break;
                case "Ch3_Watts":
                    label.text = "Watts (W)";
                    yMax.text = "5";
                    yMin.text = "0";
                    break;
                case "Temp":
                    label.text = "Temperature (C)";
                    break;
                case "Ch1_CPS":
                    label.text = "Counts/Sec (CPS)";
                    break;
                case "Inv_Period":
                    label.text = "Inv_Period (1/time)";
                    break;
                case "CCR_cm":
                    label.text = "Rod Height (cm)";
                    break;
                case "FCR_cm":
                    label.text = "Rod Height (cm)";
                    break;
                default:
                    break;
            }
        }

    }

    void timeseries()
    {

    }
}
