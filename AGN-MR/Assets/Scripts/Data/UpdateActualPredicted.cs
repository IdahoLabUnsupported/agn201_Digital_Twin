using Data;
using TMPro;
using UnityEngine;

public class UpdateActualPredicted : MonoBehaviour
{
    private GameObject ActualPredictedPanel;

    private TextMeshPro Ch2_Watts_Name;
    private TextMeshPro Ch3_Watts_Name;
    private TextMeshPro Temp_Name;
    private TextMeshPro Ch1_CPS_Name;
    private TextMeshPro Inv_Period_Name;
    private TextMeshPro CCR_cm_Name;
    private TextMeshPro FCR_cm_Name;

    private TextMeshPro Ch2_Watts;
    private TextMeshPro Ch3_Watts;
    private TextMeshPro Temp;
    private TextMeshPro Ch1_CPS;
    private TextMeshPro Inv_Period;
    private TextMeshPro CCR_cm;
    private TextMeshPro FCR_cm;

    private TextMeshPro Ch2_Watts_P;
    private TextMeshPro Ch3_Watts_P;
    private TextMeshPro Temp_P;
    private TextMeshPro Ch1_CPS_P;
    private TextMeshPro Inv_Period_P;
    private TextMeshPro CCR_cm_P;
    private TextMeshPro FCR_cm_P;

    public DataManager Data;

    private void Awake()
    {
        ActualPredictedPanel = GameObject.Find("ActualPredictedPanel");

        // todo..
        Ch2_Watts_Name = GameObject.Find("Ch2_Watts/CompressableButtonVisuals/IconAndText/SensorName").GetComponent<TextMeshPro>();
        Ch3_Watts_Name = GameObject.Find("Ch3_Watts/CompressableButtonVisuals/IconAndText/SensorName").GetComponent<TextMeshPro>();
        Temp_Name = GameObject.Find("Temp/CompressableButtonVisuals/IconAndText/SensorName").GetComponent<TextMeshPro>();
        Ch1_CPS_Name = GameObject.Find("Ch1_CPS/CompressableButtonVisuals/IconAndText/SensorName").GetComponent<TextMeshPro>();
        Inv_Period_Name = GameObject.Find("Inv_Period/CompressableButtonVisuals/IconAndText/SensorName").GetComponent<TextMeshPro>();
        CCR_cm_Name = GameObject.Find("CCR_cm/CompressableButtonVisuals/IconAndText/SensorName").GetComponent<TextMeshPro>();
        FCR_cm_Name = GameObject.Find("FCR_cm/CompressableButtonVisuals/IconAndText/SensorName").GetComponent<TextMeshPro>();

        Ch2_Watts = GameObject.Find("Ch2_Watts/CompressableButtonVisuals/IconAndText/ActualValue").GetComponent<TextMeshPro>();
        Ch3_Watts = GameObject.Find("Ch3_Watts/CompressableButtonVisuals/IconAndText/ActualValue").GetComponent<TextMeshPro>();
        Temp = GameObject.Find("Temp/CompressableButtonVisuals/IconAndText/ActualValue").GetComponent<TextMeshPro>();
        Ch1_CPS = GameObject.Find("Ch1_CPS/CompressableButtonVisuals/IconAndText/ActualValue").GetComponent<TextMeshPro>();
        Inv_Period = GameObject.Find("Inv_Period/CompressableButtonVisuals/IconAndText/ActualValue").GetComponent<TextMeshPro>();
        CCR_cm = GameObject.Find("CCR_cm/CompressableButtonVisuals/IconAndText/ActualValue").GetComponent<TextMeshPro>();
        FCR_cm = GameObject.Find("FCR_cm/CompressableButtonVisuals/IconAndText/ActualValue").GetComponent<TextMeshPro>();

        Ch2_Watts_P = GameObject.Find("Ch2_Watts/CompressableButtonVisuals/IconAndText/PredictedValue").GetComponent<TextMeshPro>();
        Ch3_Watts_P = GameObject.Find("Ch3_Watts/CompressableButtonVisuals/IconAndText/PredictedValue").GetComponent<TextMeshPro>();
        Temp_P = GameObject.Find("Temp/CompressableButtonVisuals/IconAndText/PredictedValue").GetComponent<TextMeshPro>();
        Ch1_CPS_P = GameObject.Find("Ch1_CPS/CompressableButtonVisuals/IconAndText/PredictedValue").GetComponent<TextMeshPro>();
        Inv_Period_P = GameObject.Find("Inv_Period/CompressableButtonVisuals/IconAndText/PredictedValue").GetComponent<TextMeshPro>();
        CCR_cm_P = GameObject.Find("CCR_cm/CompressableButtonVisuals/IconAndText/PredictedValue").GetComponent<TextMeshPro>();
        FCR_cm_P = GameObject.Find("FCR_cm/CompressableButtonVisuals/IconAndText/PredictedValue").GetComponent<TextMeshPro>();
    }
    // Start is called before the first frame update
    void Start()
    {
        Data = DataManager.GetInstance();
    }

    // Update is called once per frame
    void Update()
    {
        if (ActualPredictedPanel.activeSelf)
        {
            if (!IsInvoking("UpdateValues"))
            {
                InvokeRepeating("UpdateValues", 0, 1.5f);
            }
        }
        else
        {
            CancelInvoke("UpdateValues");
        }
    }

    void UpdateValues()
    {
        // todo..
        Ch2_Watts_Name.text = "Ch2_Watts";
        Ch3_Watts_Name.text = "Ch3_Watts";
        Temp_Name.text = "Temp";
        Ch1_CPS_Name.text = "Ch1_CPS";
        Inv_Period_Name.text = "Inv_Period";
        CCR_cm_Name.text = "CCR_cm";
        FCR_cm_Name.text = "FCR_cm";

        Ch2_Watts.text = Data.predictedStates[0].Predicted.ToString();
        Ch3_Watts.text = Data.predictedStates[1].Predicted.ToString();
        Temp.text = Data.predictedStates[2].Predicted.ToString();
        Ch1_CPS.text = Data.predictedStates[3].Predicted.ToString();
        Inv_Period.text = Data.predictedStates[4].Predicted.ToString();
        CCR_cm.text = Data.predictedStates[5].Predicted.ToString();
        FCR_cm.text = Data.predictedStates[6].Predicted.ToString();

        Ch2_Watts_P.text = Data.predictedStates[0].Reported.ToString();
        Ch3_Watts_P.text = Data.predictedStates[1].Reported.ToString();
        Temp_P.text = Data.predictedStates[2].Reported.ToString();
        Ch1_CPS_P.text = Data.predictedStates[3].Reported.ToString();
        Inv_Period_P.text = Data.predictedStates[4].Reported.ToString();
        CCR_cm_P.text = Data.predictedStates[5].Reported.ToString();
        FCR_cm_P.text = Data.predictedStates[6].Reported.ToString();

    }
}
