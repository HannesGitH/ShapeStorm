using System.Collections;
using System.Collections.Generic;
using UnityEngine;

// A behaviour

public class PlayerMaybe : MonoBehaviour
{

    [Range(0,1)] public float speed = 0.2f; 

    public Vector3 pos;

    private void Start()
    {
        // ahead = new GameObject("ahead");
    }

    void Update() 
    { 
        transform.RotateAround(Vector3.zero, Vector3.up, -speed * Input.GetAxis("Horizontal"));
        //pos += speed*new Vector3(Input.GetAxis("Horizontal"),0,Input.GetAxis("Vertical"));
        //transform.position = pos;
        transform.LookAt(new Vector3(0,0,0)); 
        transform.Translate(Vector3.Normalize(transform.position)*speed*Input.GetAxis("Mouse ScrollWheel"));
        //// _renderer.enabled = (currentDistance > hideDistance); 
    }

}
