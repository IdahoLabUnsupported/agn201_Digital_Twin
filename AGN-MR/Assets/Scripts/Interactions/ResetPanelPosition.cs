using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class ResetPanelPosition : MonoBehaviour
{
    public void ResetAndFacePlayer()
    {
        Transform cameraPosition = Camera.main.transform;
        //transform.position = cameraPosition.position + cameraPosition.forward * 2;
        transform.LookAt(cameraPosition);
    }
}
