using System.Collections.Generic;
using UnityEngine;

public class TimeSeriesLineGraph : MonoBehaviour
{
    // Variables for controlling the appearance of the graph
    public Color lineColor = Color.green;
    public float lineWidth = 0.1f;
    public int maxDataPoints = 5;

    // Variables for controlling the data updates
    public float dataUpdateInterval = 1.0f;
    private float timeSinceLastUpdate = 0.0f;
    private List<Vector3> dataPoints = new List<Vector3>();

    // Components for rendering the graph
    private LineRenderer lineRenderer;

    // Start is called before the first frame update
    void Start()
    {
        // Get the LineRenderer component from the Graph GameObject
        lineRenderer = GetComponent<LineRenderer>();

        // Set the LineRenderer's initial parameters
        lineRenderer.startWidth = lineWidth;
        lineRenderer.endWidth = lineWidth;
        lineRenderer.material.color = lineColor;

        // render lines in space relative to canvas parent
        lineRenderer.useWorldSpace = false;

        // Add an initial data point
        AddDataPoint(0.0f, 0.0f);

        // Start the data update timer
        InvokeRepeating("UpdateData", dataUpdateInterval, dataUpdateInterval);
    }

    // Update is called once per frame
    void Update()
    {
        // Update the graph by setting the LineRenderer's positions
        lineRenderer.positionCount = dataPoints.Count;
        lineRenderer.SetPositions(dataPoints.ToArray());
    }

    // Add a new data point to the graph
    void AddDataPoint(float time, float value)
    {
        // Remove the oldest data point if we've reached the maximum number of data points
        if (dataPoints.Count >= maxDataPoints)
        {
            dataPoints.RemoveAt(0);
        }

        // Add the new data point
        Vector3 point = new Vector3(time, value, 0.0f);
        dataPoints.Add(point);
    }

    // Update the data at regular intervals
    void UpdateData()
    {
        // Add a new random data point
        float time = Time.time;
        float value = Random.Range(0.0f, 1.0f);
        AddDataPoint(time, value);
    }
}
