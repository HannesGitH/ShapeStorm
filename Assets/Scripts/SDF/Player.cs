using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class Player : MonoBehaviour
{
    // Start is called before the first frame update
    void Start()
    {
        
    }

    // Update is called once per frame
    void Update()
    {
        if(Input.touchCount>0){
            Vector2 movement = Input.touches[0].deltaPosition/50;
            transform.position += new Vector3(movement.x,-movement.y,0);

        }
    }
}
