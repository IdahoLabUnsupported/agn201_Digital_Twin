using TMPro;
using UnityEngine;

public class ToggleMenuItems : MonoBehaviour
{

    public GameObject ConfigPanel;
    public GameObject DataPanel;
    public GameObject TimeseriesGraph;

    public string Selection;
    // Start is called before the first frame update
    void Start()
    {
        TimeseriesGraph.SetActive(false);
    }

    // Update is called once per frame
    void Update()
    {
        switch (Selection)
        {
            case "Data Panel":
                ConfigPanel.SetActive(false);
                DataPanel.SetActive(true);
                break;
            case "Config Panel":
                TimeseriesGraph.SetActive(false);
                DataPanel.SetActive(false);
                ConfigPanel.SetActive(true);
                break;
            case "Close":
                SetSelection("");
                break;
            default:
                TimeseriesGraph.SetActive(false);
                DataPanel.SetActive(false);
                ConfigPanel.SetActive(false);
                break;
        }
    }



    public void SetSelection(string selection)
    {
        Selection = selection;
    }
}
