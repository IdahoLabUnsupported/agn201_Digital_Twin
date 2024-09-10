using System.Collections.Generic;

namespace DeepLynx
{
    public class Sensor
    {
        public Sensor(int sensorID)
        {
            SensorID = sensorID;
            SensorData = new SensorData();
        }
        public int SensorID { get; set; }
        public SensorData SensorData { get; set; }

    }

    public class SensorData
    {
        public SensorData()
        {
            Time_index = 0;
            Value = 0;
        }

        public int Time_index { get; set; }
        public float Value { get; set; }
    }

    public class Equipment
    {
        public Equipment(string nodeID)
        {
            NodeID = nodeID;
            SensorDictionary = new Dictionary<string, Sensor>();
        }
        public string NodeID { get; set; }
        public Dictionary<string, Sensor> SensorDictionary { get; set; }
    }
}

