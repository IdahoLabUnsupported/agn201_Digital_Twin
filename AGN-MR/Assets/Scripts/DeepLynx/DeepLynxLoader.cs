using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;

namespace DeepLynx
{
    public class DeepLynxLoader: MonoBehaviour
    {
        [DllImport("deeplynx_loader", CharSet = CharSet.Ansi)]
        public static extern IntPtr load(string config_path);

        private IntPtr result;

        //void OnEnable()
        //{
        //    RegisterDebugCallback(OnDebugCallback);
        //}
        private void Start()
        {
            Debug.Log("Entering DeepLynxLoader");
            try
            {
                // then call like any other static method
                result = load("Assets/StreamingAssets/config.yaml");

                string s = Marshal.PtrToStringUTF8(result);
                Debug.Log("Rust: " + s);
                //Marshal.FreeHGlobal(result);


            }
            catch (System.Exception e)
            {
                Debug.Log("Error in DeepLynxLoader: " + e);
                Debug.Log(result);
            }


            InvokeRepeating("QueryTheDuck", 0f, 2);
        }

        private void QueryTheDuck()
        {

        }
    }
}
