using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class Player : MonoBehaviour
{

    public InputManI inputMan =  new InputMan(); //TODO: wanna drag n drop

    // Start is called before the first frame update
    void Start()
    {
        
    }

    // Update is called once per frame
    void Update()
    {
        Vector3 delta = inputMan.getDelta();
        transform.position += delta;
    }
}
