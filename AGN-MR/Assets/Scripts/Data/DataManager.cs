using SQLite4Unity3d;
using UnityEngine;
using System;
using System.IO;
using System.Reflection;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using TMPro;

namespace Data
{
    public class DataManager : MonoBehaviour
    {
        public static float dataLoadIntervalControlRods = 2f;
        public static float dataLoadIntervalPredicted = 1f;

        public float Fcr_Up;
        public float Fcr_Engaged;
        // public float reactorWattsPredicted;
        // public float reactorWattsReported;
        public List<float> predictionsTimeCheck;

        public GameObject ActualPredictedPanelListitems;

        public List<string> predictionTableNames;
        public List<PredictedState> predictedStates = new List<PredictedState>();


        public ReactorState reactorState;
        public RodState rodState;

        // Singleton
        public static DataManager Instance { get; set; }
        public static DataManager GetInstance()
        {
            return Instance;
        }

        void Awake()
        {
            if (Instance == null)
            {
                DontDestroyOnLoad(gameObject);
                Instance = this;
            }
            else
            {
                Destroy(gameObject);
            }

            foreach (Transform child in ActualPredictedPanelListitems.transform)
            {
                predictionTableNames.Add(child.name);
            }
        }


        void Start()
        {
            string startTimestamp = "2023-02-13 13:55:00";
            string endTimestamp = "2023-02-13 13:55:00";
            string startTimeIndex = "0";
            string endTimeIndex = "50";

            LoadPredictionsHistorical(startTimestamp, endTimestamp, startTimeIndex, endTimeIndex);
            LoadOldDataForPrettyControlRods();
            //TestRustDB();

            // StartCoroutine(LoadPredictionsPseudoRealTime());
        }

        void LoadOldDataForPrettyControlRods()
        {
          rodState = new RodState();

          string dbName = "isu_filtered.db";
          string timestamp = "2023-02-08 16:06:00";
          string[] tableNames = {"digitals"};
          string[][] columnNames = new string[tableNames.Length][];
          columnNames[0] = new string[] {"time", "fcr_up", "fcr_engaged", "fcr_down", "ccr_up", "ccr_engaged", "ccr_down", "sr1_up", "sr1_engaged", "sr1_down", "sr2_up", "sr2_engaged", "sr2_down"};
          List<float>[][] allColumnValues = QueryMultipleTables(dbName, timestamp, tableNames, columnNames);

          // start loading
          WaitForSeconds wait = new WaitForSeconds(dataLoadIntervalControlRods);
          bool repeat_loading = true;
          StartCoroutine(LoadControlRodData(allColumnValues[0], wait, repeat_loading));
        }

        void LoadPredictionsHistorical(string startTimestamp, string endTimestamp, string startTimeIndex, string endTimeIndex)
        {
            // stuff for querying
            string dbName = "isu_predictions.db";
            // string[] tableNamesPredicted = new string[] {"FCR_cm", "Temp", "Ch2_Watts", "Ch1_CPS", "Ch3_Watts", "CCR_cm", "inv_period"};
            string[] tableNamesPredicted = predictionTableNames.ToArray();
            string[] columnNamesPredicted = new string[] { "time", "predicted", "reported", "delta" };

            // open db connection
            SQLiteConnection db = OpenConnection(dbName, SQLiteOpenFlags.ReadOnly);
            // loop over table names, make table specific query string, query one table with QueryPredictionTable
            List<float>[][] allTablesColumnValues = new List<float>[tableNamesPredicted.Length][];
            for (int i = 0; i < tableNamesPredicted.Length; i++)
            {
                string sqlQuery = $"SELECT {string.Join(",", columnNamesPredicted)} FROM {tableNamesPredicted[i]} WHERE date_time>='{startTimestamp}' AND date_time<='{endTimestamp}' AND time>='{startTimeIndex}' AND time<='{endTimeIndex}'";
                List<float>[] oneTableColumnValues = QueryPredictionTable(db, sqlQuery, tableNamesPredicted[i], columnNamesPredicted);
                allTablesColumnValues[i] = oneTableColumnValues;
            }
            // close db connection
            db.Dispose();

            // create data class instances that data will be loaded to
            foreach (string childName in predictionTableNames)
            {
                Debug.Log(childName);
                predictedStates.Add(new PredictedState(childName));
            }

            // set current query chunk data to PredictedState's PredictedQueryChunk and ReportedQueryChunk
            SetPredictionStatesQueryChunks(allTablesColumnValues);

            // start loading
            WaitForSeconds wait = new WaitForSeconds(dataLoadIntervalPredicted);
            bool repeat_loading = true;
            StartCoroutine(LoadPredictionsData(allTablesColumnValues, wait, repeat_loading));

            // preview some data
            predictionsTimeCheck = allTablesColumnValues[0][0];
            // predictedStates[0]
        }

        //void TestRustDB()
        //{
        //    SQLiteConnection db = OpenConnection("isu_db_1.db", SQLiteOpenFlags.ReadOnly);

        //    string sqlQuery = $"SELECT * FROM ch3_engineering_data ORDER BY date_time DESC, time DESC LIMIT 1";

        //    var rows = db.Query<ch3_engineering_data>(sqlQuery);

        //    GameObject.Find("TEST").GetComponent<TextMeshPro>().text = rows[0].ch3_watts.ToString();
        //    //            SELECT*
        //    //FROM your_table_name
        //    //ORDER BY timestamp DESC, time_index DESC
        //    //LIMIT 1;

        //}

        // wip
        // private IEnumerator LoadPredictionsPseudoRealTime()
        // {
        //     Debug.Log("Start LoadPredictionsPseudoRealTime");
        //     // maybe we'll do dynamic load intervals some day..
        //     // float queryIntervalSeconds = 0.1f;
        //     float queryIntervalSeconds = 10f;
        //     float displayIntervalSeconds = 0.1f;
        //     WaitForSeconds waitQuery = new WaitForSeconds(queryIntervalSeconds);
        //     WaitForSeconds waitLoad = new WaitForSeconds(displayIntervalSeconds);
        //
        //     string dbName = "isu_filtered.db";
        //     string initialTimestamp = "2023-02-08 16:06:00";
        //     string timestamp = initialTimestamp;
        //     float startTime = 0f;
        //     float endTime = startTime + queryIntervalSeconds;
        //
        //     // just loading ch3_engineering_data for now to keep it more simpler
        //     string tableName = "ch3_engineering_data";
        //     string[] columnNames = new string[] {"time", "ch3_watts", "ch3_reactivity", "ch3_inv_period"};
        //     string columnList = string.Join(",", columnNames);
        //
        //     // Define a list of SQL queries to execute
        //     List<string> sqlQueries = new List<string>() {$"SELECT {string.Join(",", columnNames)} FROM {tableName} WHERE date_time='{timestamp}' AND time BETWEEN {startTime.ToString()} AND {endTime.ToString()} ORDER BY time"};
        //
        //     // Loop over each SQL query and grow the list dynamically
        //     bool loadingStarted = false;
        //     // List<float>[] endlessLoadList = new List<float>[columnNames.Length];
        //     List<float>[] endlessLoadList = null;
        //     // foreach (string sqlQuery in sqlQueries)
        //     for (int i = 0; i < sqlQueries.Count; i++)
        //     {
        //         // Debug.Log(endNumber);
        //         // Debug.Log(endNumber.ToString());
        //         Debug.Log(sqlQueries[i]);
        //         try
        //         {
        //             // open new sqlite connection for each chunk query
        //             SQLiteConnection db = OpenConnection(dbName, SQLiteOpenFlags.ReadOnly);
        //             // todo: loop over tableNames list and add to List<float>[][] queryChunkResults
        //             List<float>[] queryChunkResult = QueryPredictionTable(db, sqlQueries[i], tableName, columnNames);
        //             // dispose of connection
        //             db.Dispose();
        //
        //             // reactorTime = queryChunkResult[0];
        //
        //             // do stuff with result chunk
        //
        //             // start loading data
        //             if (loadingStarted == false)
        //             {
        //                 endlessLoadList = queryChunkResult;
        //                 // todo: either loop over tableNames or create new method LoadTablesChunks
        //                 // todo: talk to Kolton about what data where
        //                 StartCoroutine(LoadPredictionsChunks(endlessLoadList, waitLoad));
        //                 loadingStarted = true;
        //             }
        //             // continue loading data
        //             else
        //             {
        //                 // add to endless load list array
        //                 for (int j = 0; j < endlessLoadList.Length; j++)
        //                 {
        //                     endlessLoadList[j].AddRange(queryChunkResult[j]);
        //                 }
        //             }
        //
        //             // Add to sqlQueries list
        //             startTime = startTime + queryIntervalSeconds;
        //             endTime = endTime + queryIntervalSeconds;
        //             sqlQueries.Add($"SELECT {string.Join(",", columnNames)} FROM {tableName} WHERE date_time='{timestamp}' AND time BETWEEN {startTime.ToString()} AND {endTime.ToString()} ORDER BY time");
        //
        //             // todo: add a way to check if we're at the end of the time index for a given timestamp and adjust query accordingly
        //         }
        //         catch (Exception e)
        //         {
        //             Debug.Log(e.ToString());
        //         }
        //         yield return waitQuery;
        //     }
        //
        // }

        private static List<float>[] QueryPredictionTable(SQLiteConnection db, string sqlQuery, string tableName, string[] columnNames)
        {
            List<float>[] allColumnValues = new List<float>[columnNames.Length];

            try
            {
                for (int i = 0; i < columnNames.Length; i++)
                {
                    allColumnValues[i] = new List<float>();
                }

                // Jank incoming
                // need data interface so this method can be applied to more tables
                var rows = db.Query<predictions>(sqlQuery);
                foreach (var row in rows)
                {
                    allColumnValues[0].Add(row.time);
                    allColumnValues[1].Add(row.predicted);
                    allColumnValues[2].Add(row.reported);
                    allColumnValues[3].Add(row.delta);
                }
            }
            catch (Exception e)
            {
                Debug.Log(e.ToString());
            }

            return allColumnValues;
        }

        private IEnumerator LoadPredictionsData(List<float>[][] allColumnValues, WaitForSeconds waitLoad, bool repeat_loading)
        {
            // always load the first one
            bool load = true;
            while (load)
            {
                for (int i = 0; i < allColumnValues[0][0].Count; i++)
                {
                    // Debug.Log(i);
                    try
                    {
                        int index = 0;
                        foreach (PredictedState predictedState in predictedStates)
                        {
                          predictedState.Time = allColumnValues[index][0][i];
                          predictedState.Predicted = allColumnValues[index][1][i];
                          predictedState.Reported = allColumnValues[index][2][i];
                          predictedState.Delta = allColumnValues[index][3][i];
                          index++;
                        }

                        // // Preview some dataz
                        // reactorWattsPredicted = predictedStateCh3_Watts.Predicted;
                        // reactorWattsReported = predictedStateCh3_Watts.Reported;
                    }
                    catch (Exception e)
                    {
                        Debug.Log(e.ToString());
                    }

                    yield return waitLoad;
                }
                // repeat loading if param set true
                load = repeat_loading;
            }
        }

        private void SetPredictionStatesQueryChunks(List<float>[][] allColumnValues)
        {
            try
            {
                int index = 0;
                foreach (PredictedState predictedState in predictedStates)
                {
                  predictedState.TimeQueryChunk = allColumnValues[index][0];
                  predictedState.PredictedQueryChunk = allColumnValues[index][1];
                  predictedState.ReportedQueryChunk = allColumnValues[index][2];
                  index++;
                }
            }
            catch (Exception e)
            {
                Debug.Log(e.ToString());
            }
        }

        private static SQLiteConnection OpenConnection(string dbName, SQLiteOpenFlags flags)
        {
            SQLiteConnection conn = null;
            try
            {
                // the fact that this is in streamingAssetsPath means it can only be read only, so this method is dumb
                string dbPath = Path.Combine(Application.streamingAssetsPath, dbName);
                conn = new SQLiteConnection(dbPath, flags);
            }
            catch (Exception e)
            {
                Debug.Log(e.ToString());
            }
            return conn;
        }

        private IEnumerator LoadControlRodData(List<float>[] allColumnValues, WaitForSeconds waitLoad, bool repeat_loading)
        {
            // always load the first one
            bool load = true;
            while (load)
            {
                for (int i = 0; i < allColumnValues[0].Count; i++)
                {
                    // Debug.Log(i);
                    try
                    {
                        UpdateRodState(rodState, allColumnValues, i);
                        // Preview some dataz
                        Fcr_Up = rodState.Fcr_Up;
                        Fcr_Engaged = rodState.Fcr_Engaged;
                    }
                    catch (Exception e)
                    {
                        Debug.Log(e.ToString());
                    }

                    yield return waitLoad;
                }
                // repeat loading if param set true
                load = repeat_loading;
            }
        }

        // private IEnumerator LoadData(List<float>[][] allColumnValues)
        // {
        //     int longestListLength = GetLongestListLength(allColumnValues);
        //     WaitForSeconds wait = new WaitForSeconds(dataLoadInterval);
        //     float dataLoadingTime = 0.0;
        //     int rodIndex;
        //     int reactorIndex;
        //
        //     while (true)
        //     {
        //         for (int i = 0; i < longestListLength; i++)
        //         {
        //             // Debug.Log(dataLoadingTime);
        //             // todo: probably not the most efficient way..
        //             rodIndex = allColumnValues[0][0].FindIndex(x => x == dataLoadingTime);
        //             if (rodIndex != -1)
        //             {
        //                 // Debug.Log("rodIndex: " + rodIndex);
        //                 UpdateRodState(rodState, allColumnValues[0], rodIndex);
        //             }
        //
        //             reactorIndex = allColumnValues[1][0].FindIndex(x => x == dataLoadingTime);
        //             if (reactorIndex != -1)
        //             {
        //                 reactorState.Watts = allColumnValues[1][1][reactorIndex];
        //                 reactorState.Reactivity = allColumnValues[1][2][reactorIndex];
        //                 reactorState.Inv_Period = allColumnValues[1][3][reactorIndex];
        //             }
        //
        //             // dataLoadingTime = dataLoadingTime + dataLoadInterval;
        //             // todo: this could be better
        //             dataLoadingTime = allColumnValues[1][0][i];
        //
        //             // Preview some dataz
        //             Fcr_Up = rodState.Fcr_Up;
        //             Fcr_Engaged = rodState.Fcr_Engaged;
        //
        //             yield return wait;
        //         }
        //     }
        // }

        private static List<float>[][] QueryMultipleTables(string dbName, string timestamp, string[] tableNames, string[][] columnNames)
        {
            List<float>[][] allColumnValues = new List<float>[tableNames.Length][];
            for (int i = 0; i < tableNames.Length; i++)
            {
                string[] columns = columnNames[i];
                allColumnValues[i] = new List<float>[columns.Length];
                for (int j = 0; j < columns.Length; j++)
                {
                    allColumnValues[i][j] = new List<float>();
                }
            }

            // string dbPath = Path.Combine(Application.streamingAssetsPath, dbName);
            string dbPath = Application.streamingAssetsPath + "/" + dbName;
            var db = new SQLiteConnection(dbPath);

            for (int i = 0; i < tableNames.Length; i++)
            {
                try
                {
                    string tableName = tableNames[i];
                    string[] columns = columnNames[i];


                    // Prepare SQL query
                    string columnList = string.Join(",", columns);
                    string sqlQuery = $"SELECT {columnList} FROM {tableName} WHERE date_time='" + timestamp + "' ORDER BY time";

                    if (tableName == "ch3_engineering_data")
                    {
                        // Execute the query and get the results as a list of arrays
                        var rows = db.Query<ch3_engineering_data>(sqlQuery);

                        // Loop through the rows and set by names
                        // todo: either need generic data class that can handle many tables, or a better way to get values by column index
                        foreach (var row in rows)
                        {
                            allColumnValues[i][0].Add(row.time);

                            allColumnValues[i][1].Add(row.ch3_watts);
                            allColumnValues[i][2].Add(row.ch3_reactivity);
                            allColumnValues[i][3].Add(row.ch3_inv_period);
                        }
                    }
                    else if (tableName == "digitals")
                    {
                        var rows = db.Query<digitals>(sqlQuery);

                        foreach (var row in rows)
                        {
                            allColumnValues[i][0].Add(row.time);

                            allColumnValues[i][1].Add(row.fcr_up);
                            allColumnValues[i][2].Add(row.fcr_engaged);
                            allColumnValues[i][3].Add(row.fcr_down);

                            allColumnValues[i][4].Add(row.ccr_up);
                            allColumnValues[i][5].Add(row.ccr_engaged);
                            allColumnValues[i][6].Add(row.ccr_down);

                            allColumnValues[i][7].Add(row.sr1_up);
                            allColumnValues[i][8].Add(row.sr1_engaged);
                            allColumnValues[i][9].Add(row.sr1_down);

                            allColumnValues[i][10].Add(row.sr2_up);
                            allColumnValues[i][11].Add(row.sr2_engaged);
                            allColumnValues[i][12].Add(row.sr2_down);
                        }
                    }
                }
                catch (Exception e)
                {
                    Debug.Log(e.ToString());
                }
            }

            return allColumnValues;
        }

        public class ch3_engineering_data
        {
            public float time { get; set; }

            public float ch3_watts { get; set; }
            public float ch3_reactivity { get; set; }
            public float ch3_inv_period { get; set; }
        }

        public class digitals
        {
            public float time { get; set; }

            public float fcr_up { get; set; }
            public float fcr_engaged { get; set; }
            public float fcr_down { get; set; }

            public float ccr_up { get; set; }
            public float ccr_engaged { get; set; }
            public float ccr_down { get; set; }

            public float sr1_up { get; set; }
            public float sr1_engaged { get; set; }
            public float sr1_down { get; set; }

            public float sr2_up { get; set; }
            public float sr2_engaged { get; set; }
            public float sr2_down { get; set; }
        }

        public class predictions
        {
            public float time { get; set; }
            public float predicted { get; set; }
            public float reported { get; set; }
            public float delta { get; set; }
        }

        // todo: would be nice to have a generic class that we can easily dump queried table data into
        // public class TableData<T>
        // {
        //     public T[] Columns { get; set; }
        // }

        // public class TableData<T>
        // {
        //     private List<T> columns = new List<T>();
        //
        //     public void AddColumn(T value)
        //     {
        //         columns.Add(value);
        //     }
        //
        //     public T GetColumn(int index)
        //     {
        //         return columns[index];
        //     }
        //
        //     public int Count()
        //     {
        //         return columns.Count;
        //     }
        //
        //     public T this[int index]
        //     {
        //         get { return GetColumn(index); }
        //         set { columns[index] = value; }
        //     }
        // }


        // private void UpdateReactorState(ReactorState type, List<float>[] data, int time_index)
        // {
        //     // Get all the public properties of the ReactorState class
        //     PropertyInfo[] properties = typeof(ReactorState).GetProperties();
        //
        //     // Iterate through each property and set its value to a new value
        //     // skip time at column_index = 0
        //     int column_index = 1;
        //     foreach (PropertyInfo property in properties)
        //     {
        //         // Debug.Log(property.Name);
        //         if (property.PropertyType == typeof(float))
        //         {
        //             // column_index+1 to skip time data
        //             property.SetValue(type, data[column_index][time_index]);
        //             // Debug.Log("updating state?");
        //         }
        //         column_index++;
        //     }
        // }

        private void UpdateRodState(RodState type, List<float>[] data, int time_index)
        {
            // Get all the public properties of the RodState class
            PropertyInfo[] properties = typeof(RodState).GetProperties();

            // Iterate through each property and set its value to a new value
            int column_index = 1;
            foreach (PropertyInfo property in properties)
            {
                // Debug.Log(property.Name);
                if (property.PropertyType == typeof(float))
                {
                    property.SetValue(type, data[column_index][time_index]);
                }
                column_index++;
            }
        }

        private static int GetLongestListLength(List<float>[][] allColumnValues)
        {
            int maxListLength = 0;

            foreach (List<float>[] tableValues in allColumnValues)
            {
                foreach (List<float> columnValues in tableValues)
                {
                    if (columnValues.Count > maxListLength)
                    {
                        maxListLength = columnValues.Count;
                    }
                }
            }

            return maxListLength;
        }

    }
}
