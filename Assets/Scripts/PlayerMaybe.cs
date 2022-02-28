using System.Collections;
using System.Collections.Generic;
using UnityEngine;

// A behaviour

public class PlayerMaybe : MonoBehaviour
{

    [Range(1,50)] public float speed = 20f; 
    [Range(1,50)] public float turnSpeed = 20f; 

    // public Vector3 pos;

    private void Start()
    {
        // ahead = new GameObject("ahead");
    }

    float smoothV;
    void Update() 
    { 
        //speed = Mathf.SmoothDamp(speed,speed+35*Input.GetAxis("Vertical"),ref smoothV,0.80f);
        transform.RotateAround(Vector3.zero, transform.right, speed  * Input.GetAxis("Vertical")* Time.deltaTime);
        transform.RotateAround(transform.position, transform.forward, -turnSpeed * Time.deltaTime * Input.GetAxis("Horizontal"));
        transform.position += transform.forward * -speed * 0.005f * Input.GetAxis("Zoom");
        //// _renderer.enabled = (currentDistance > hideDistance); w
    }

}
