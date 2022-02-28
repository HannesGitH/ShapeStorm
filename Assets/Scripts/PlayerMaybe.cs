using System.Collections;
using System.Collections.Generic;
using UnityEngine;

// A behaviour

public class PlayerMaybe : MonoBehaviour
{

    [Range(1,50)] public float speed = 20f; 

    // public Vector3 pos;

    private void Start()
    {
        // ahead = new GameObject("ahead");
    }

    void Update() 
    { 
        transform.RotateAround(Vector3.zero, transform.right, speed * Time.deltaTime * Input.GetAxis("Vertical"));
        transform.RotateAround(transform.position, transform.forward, -speed * Time.deltaTime * Input.GetAxis("Horizontal"));
        //pos += speed*new Vector3(Input.GetAxis("Horizontal"),0,Input.GetAxis("Vertical"));
        //transform.position = pos;
        // transform.LookAt(new Vector3(0,0,0)); 
        transform.position += transform.forward * -speed * Input.GetAxis("Mouse ScrollWheel");
        //// _renderer.enabled = (currentDistance > hideDistance); w
    }

}
