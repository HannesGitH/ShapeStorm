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
    private Vector3 lookingAt = new Vector3(0,0,50);
    void Update()
    {
        if(FindObjectOfType<GameManager>().gameIsOver)return;

        Vector3 delta = inputMan.getDelta();
        transform.position += delta;
        // lookingAt+=delta*2;
        // transform.LookAt(lookingAt);
        // transform.Translate(new Vector3(lookingAt.x,lookingAt.y,0)/15);
    }
}
