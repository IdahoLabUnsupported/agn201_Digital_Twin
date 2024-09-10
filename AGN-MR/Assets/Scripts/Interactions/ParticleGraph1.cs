using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using UnityEngine;

public class ParticleGraph1 : MonoBehaviour
{
    public ParticleSystem ps;
    public Vector2[] dataPoints;
    public float particleSize = 0.001f;

    private float max = 0.18f;

    public Gradient grady;

    //void Start()
    //{
    //    EmitParticles();
    //}
    public void EmitParticles()
    {

        // Open the CSV file
        string filePath = "Assets/Resources/Data/Sensor1_Inlet 1.csv";
        StreamReader reader = new StreamReader(filePath);

        // Read the first line, which contains the column headers
        string line = reader.ReadLine();
        string[] headers = line.Split('\t');

        // Create a 2D array to store the data
        float[,] data = new float[1, 800];

        // Something to do with floats from strings or something... it's used in the col loop
        CultureInfo ci = (CultureInfo)CultureInfo.CurrentCulture.Clone();
        ci.NumberFormat.CurrencyDecimalSeparator = ".";

        // Read the rest of the lines, which contain the data
        for (int row = 0; row < 1; row++)
        {
            line = reader.ReadLine();
            string[] values = line.Split(',');
            for (int col = 0; col < 800; col++)
            {
                try
                {
                    float value;
                    // now we don't want to multiply by 0, ya hear!!
                    if (!float.TryParse(values[col], NumberStyles.Any, ci, out value))
                    {
                        value = .0001f;
                    }

                    Vector3 posi = new Vector3((col + 1) * particleSize, (row + 1) * particleSize, value);
                    var main = ps.main;
                    var gradientValue = value / max;
                    main.startColor = grady.Evaluate(gradientValue);
                    ps.Emit(new ParticleSystem.EmitParams()
                    {
                        position = posi,
                        startSize3D = new Vector3(particleSize, particleSize * 12, particleSize)
                    }, 1);
                }
                catch (System.Exception e)
                {
                    Debug.Log("Error in col loop: " + e);
                }

            }
        }



        // Close the CSV file
        reader.Close();

    }
}

