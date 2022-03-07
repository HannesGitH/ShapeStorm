using UnityEngine;

public interface InputManI {
    Vector3 getDelta();
}

public class InputMan : InputManI {
    public Vector3 getDelta(){
        if(Input.touchCount>0){
            Vector2 movement = -Input.touches[0].deltaPosition/50;
            return new Vector3(movement.x,movement.y,0);
        }else
        {
            return new Vector3();
        }
    }
}