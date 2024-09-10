using Data;
using UnityEngine;

public class ControlRods : MonoBehaviour
{
    private float travel = 25f;
    public float maxHeight = 0.5353f;
    public float SetFineAdjustmentRodHeightCheck;
    public GameObject StarterRods;
    public GameObject FineAdjustmentRod;
    public GameObject CourseAdjustmentRod;

    public DataManager Data;

    public float UpdateInterval = 0.5f;

    private void Start()
    {
        StarterRods = GameObject.Find("Starter Rods");
        FineAdjustmentRod = GameObject.Find("Fine Adjustment Rod");
        CourseAdjustmentRod = GameObject.Find("Course Adjustment Rod");

        Data = DataManager.GetInstance();

        InvokeRepeating("SetStarterRodsHeight", 1, UpdateInterval);
        InvokeRepeating("SetFineAdjustmentRodHeight", 1, UpdateInterval);
        InvokeRepeating("SetCourseAdjustmentRodHeight", 1, UpdateInterval);
    }

    private void Update()
    {
    }

    public void SetStarterRodsHeight()
    {
        double height = Data.rodState.Sr1_Engaged;

        if (height != 0.00)
        {
            StarterRods.transform.localPosition = new Vector3(StarterRods.transform.localPosition.x, maxHeight, StarterRods.transform.localPosition.z);
        }
        else
        {
            StarterRods.transform.localPosition = new Vector3(StarterRods.transform.localPosition.x, 0.0f, StarterRods.transform.localPosition.z);
        }
    }

    public void SetFineAdjustmentRodHeight()
    {
        float height = Data.predictedStates[6].Reported;
        float setHeight = (float)(height / travel) * maxHeight;
        FineAdjustmentRod.transform.localPosition = new Vector3(FineAdjustmentRod.transform.localPosition.x, setHeight, FineAdjustmentRod.transform.localPosition.z);
    }

    public void SetCourseAdjustmentRodHeight()
    {
        
        
        float height = Data.predictedStates[5].Reported;
        float setHeight = (float)((height / travel) * maxHeight);
        CourseAdjustmentRod.transform.localPosition = new Vector3(CourseAdjustmentRod.transform.localPosition.x, setHeight, CourseAdjustmentRod.transform.localPosition.z);
    }
}
