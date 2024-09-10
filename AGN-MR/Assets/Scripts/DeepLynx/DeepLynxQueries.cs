using UnityEngine;
using LitJson;
using System.Threading.Tasks;
using UnityEngine.Networking;
using System;

namespace DeepLynx
{

    public class DeepLynxQueries : MonoBehaviour, IDeepLynxBaseQuery
    {
        public string baseURL { get ; set; }
        public int containerID { get; set; }
        public int dataSourceID { get; set; }
        public string bearerToken { get; set; }

        public DeepLynxQueries(string baseURL, int containerID = 0, int dataSourceID = 0, string bearerToken = "")
        {
            this.baseURL = baseURL;
            this.containerID = containerID;
            this.dataSourceID = dataSourceID;
            this.bearerToken = "";
        }


        public async Task<JsonData> TimeseriesByDataSource(string query_body, int dataSourceID)
        {
            string endPoint = "/containers/" + containerID + "/import/datasources/" + dataSourceID + "/data";
            UnityWebRequest request = new UnityWebRequest(baseURL + endPoint, "POST");

            request.SetRequestHeader("Content-Type", "application/json");
            request.SetRequestHeader("Authorization", "Bearer " + bearerToken);

            byte[] bodyRaw = new System.Text.UTF8Encoding().GetBytes(query_body);
            request.uploadHandler = (UploadHandler)new UploadHandlerRaw(bodyRaw);

            request.downloadHandler = (DownloadHandler)new DownloadHandlerBuffer();

            var operation = request.SendWebRequest();

            while (!operation.isDone)
                await Task.Yield();

            if (request.result != UnityWebRequest.Result.Success)
            {
                Debug.LogError($"Failed TimeseriesByDataSource Request: {request.error}");
            }
                
            
            JsonData res_json = JsonMapper.ToObject(request.downloadHandler.text);


            return res_json;
            
            
        }

        public async Task<JsonData> TimeseriesByNode(string nodeID, string requestType, string query_body = "")
        {
            try
            {
                string endPoint = "/containers/" + containerID + "/graphs/nodes/" + nodeID + "/timeseries";
                UnityWebRequest request = new UnityWebRequest(baseURL + endPoint, requestType);

                request.SetRequestHeader("Content-Type", "application/json");
                request.SetRequestHeader("Authorization", "Bearer " + bearerToken);

                
                if (requestType == "POST")
                {
                    Debug.Log("Timeseries by node query: " + query_body);
                    byte[] bodyRaw = new System.Text.UTF8Encoding().GetBytes(query_body);
                    request.uploadHandler = new UploadHandlerRaw(bodyRaw);
                }

                request.downloadHandler = new DownloadHandlerBuffer();

                var operation = request.SendWebRequest();

                while (!operation.isDone)

                    await Task.Yield();

                if (request.result != UnityWebRequest.Result.Success)
                {
                    Debug.LogError($"Failed TimeseriesByNode Request: {request.error}");
                    Debug.Log(request.result);
                }


                JsonData res_json = JsonMapper.ToObject(request.downloadHandler.text);


                return res_json;
            }
            catch (Exception e)
            {
                Debug.Log("TimeseriesByNode: " + e);
                return new JsonData();
            }
            


        }

        public async Task<JsonData> GraphQL(string query_body)
        {
            string endPoint = "/containers/" + containerID + "/data";
            UnityWebRequest request = new UnityWebRequest(baseURL + endPoint, "POST");

            request.SetRequestHeader("Content-Type", "application/json");
            request.SetRequestHeader("Authorization", "Bearer " + bearerToken);

            byte[] bodyRaw = new System.Text.UTF8Encoding().GetBytes(query_body);
            request.uploadHandler = (UploadHandler)new UploadHandlerRaw(bodyRaw);

            request.downloadHandler = (DownloadHandler)new DownloadHandlerBuffer();

            var operation = request.SendWebRequest();

            while (!operation.isDone)
                await Task.Yield();

            if (request.result != UnityWebRequest.Result.Success)
            {
                Debug.LogError($"GraphQL error: {request.error}");
                return new JsonData(new { request.error });
            }


            JsonData res_json = JsonMapper.ToObject(request.downloadHandler.text);


            return res_json;


        }


        public async Task<JsonData> Get(string url)
        {
            
            UnityWebRequest request = new UnityWebRequest(baseURL + url, "GET");

            request.certificateHandler = new BypassCertificate();

            request.SetRequestHeader("Content-Type", "application/json");
            request.SetRequestHeader("Authorization", "Bearer " + bearerToken);

            request.downloadHandler = (DownloadHandler)new DownloadHandlerBuffer();

            var operation = request.SendWebRequest();

            while (!operation.isDone)
                await Task.Yield();

            if (request.result != UnityWebRequest.Result.Success)
                Debug.LogError($"Failed: {request.error}");

            JsonData res_json = JsonMapper.ToObject(request.downloadHandler.text);


            return res_json;
        }

        public class BypassCertificate : CertificateHandler
        {
            protected override bool ValidateCertificate(byte[] certificateData)
            {
                //Simply return true no matter what
                return true;
            }
        }

    }
}


