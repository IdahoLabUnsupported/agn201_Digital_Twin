using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class MultiChildColorChange : MonoBehaviour
{
    GameObject[] equipment;

    public Material opaque;
    public Material transparent;

    public bool isTransparent = false;
    public bool changed = false;

    private void Awake()
    {
        equipment = GameObject.FindGameObjectsWithTag("Transparent");
    }

    // Start is called before the first frame update
    void Start()
    {
       
    }

    public void ToggleTransparent()
    {
        isTransparent = !isTransparent;
        changed = true;
    }

    private void Update()
    {
        if (changed)
        {
            if (isTransparent)
            {
                ChangeMaterialTransparentMultipleObjects();
            }
            else
            {
                ChangeMaterialOpaqueMultipleObjects();
            }
        }
    }

    public void ChangeMaterialTransparentMultipleObjects()
    {
        foreach (GameObject go in equipment)
        {
            AssignNewColor(go.transform, transparent);
        }
        changed = false;
    }

    public void ChangeMaterialOpaqueMultipleObjects()
    {
        foreach (GameObject go in equipment)
        {
            AssignNewColor(go.transform, opaque);
        }
        changed = false;
    }

    public void AssignNewColor(Transform parent, Material newMaterial)
    {
        foreach (Transform child in parent)
        {
            if (child.GetComponent<Renderer>()){

                Renderer meshRenderer = child.GetComponent<Renderer>();

                meshRenderer.material = newMaterial; 
            }
            
            AssignNewColor(child, newMaterial);
            
        }
    }

    
}
