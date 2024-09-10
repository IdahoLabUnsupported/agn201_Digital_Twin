using System.Linq;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using Data;

public class ParticleGraph : MonoBehaviour
{
    public ParticleSystem ps;
    public float particleSize = 0.008f;
    public Gradient grady;
    private float max = 0.18f;

    // time
    private float xMin = 0f;
    private float xMax = 0.8f;
    // data
    private float yMin = -0.05f;
    private float yMax = 0.35f;
    // private Vector2[] currentDataPoints;

    // public DataManager Data;

    public List<float> Times;
    public List<float> Data;

    public List<Vector2> Points;
    public List<Vector3> Posis;

    public float dataMin;
    public float dataMax;


    private void Start()
    {
    }

    // private void Update()
    // {
    //     // Check if it's time to update the graph
    //     if (Time.time > nextUpdate)
    //     {
    //         // Update the graph with new data points
    //         UpdateGraph(newDataPoints);
    //
    //         // Schedule the next update
    //         nextUpdate = Time.time + updateInterval;
    //     }
    // }

    public void EmitParticles(List<float> times, List<float> data)
    {
        Data = data;
        Times = times;
        // dataMin = data.Min();
        dataMin = yMin;
        dataMax = data.Max();
        float timeMin = times.Min();
        float timeMax = times.Max();
        // Clear any existing particles
        ps.Clear();

        // Loop through the data points and emit particles at each point
        for (int i = 0; i < times.Count; i++)
        {
            // filtering outlier data so graph data scales well and looks good
            if (data[i] >= dataMin)
            {
                float time = Normalize(times[i], timeMin, timeMax, xMin, xMax);
                float datum = Normalize(data[i], dataMin, dataMax, yMin, yMax);
                Vector2 point = new Vector2 (time, datum);
                Points.Add(point);
                // Vector3 posi = new Vector3(point.x, point.y, .0001f);
                Vector3 posi = new Vector3(point.x, .0001f, point.y);
                Posis.Add(posi);
                var main = ps.main;
                var gradientValue = point.y / max;
                main.startColor = grady.Evaluate(gradientValue);
                ps.Emit(new ParticleSystem.EmitParams()
                {
                    position = posi,
                    startSize3D = new Vector3(particleSize * 3, particleSize * 8, particleSize)
                }, 1);
            }
        }

        // Store the data points for future updates
        // currentDataPoints = dataPoints;
    }

    // public void UpdateGraph(Vector2[] newDataPoints)
    // {
    //     // Update the data points
    //     Vector2[] dataPoints = newDataPoints;
    //
    //     // Get the particles from the particle system
    //     ParticleSystem.Particle[] particles = new ParticleSystem.Particle[ps.particleCount];
    //     ps.GetParticles(particles, particles.Length);
    //
    //     // Update the positions of the particles based on the new data points
    //     for (int i = 0; i < particles.Length; i++)
    //     {
    //         Vector3 newPos = new Vector3(dataPoints[i].x, dataPoints[i].y, particles[i].position.z);
    //         particles[i].position = newPos;
    //     }
    //
    //     // Apply the updated particles to the particle system
    //     ps.SetParticles(particles, particles.Length);
    // }

    // normalize to range from valmin to valmax
    private float Normalize(float val, float valmin, float valmax, float min=0f, float max=1f)
    {
        return (((val - valmin) / (valmax - valmin)) * (max - min)) + min;
    }




}
