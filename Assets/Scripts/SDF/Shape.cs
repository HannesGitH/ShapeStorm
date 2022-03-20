using System.Collections;
using System.Collections.Generic;
using UnityEngine;
//TODO: repetition

public class Shape : MonoBehaviour
{

    public enum ShapeType {Sphere,Cube,Torus, Tube, BoxFrame, Octahedron, Link};
    public enum Operation {None, Blend, Cut,Mask}

    public ShapeType shapeType;
    public Operation operation;
    public Color colour = Color.white;
    public float lightness = 0.2f;
    [Range(0,1)]
    public float blendStrength;
    [HideInInspector]
    public int numChildren;

    public Vector3 Position {
        get {
            return transform.position;
        }
    }
    public Vector4 Rotation {
        get {
            Quaternion q = transform.rotation;
            return new Vector4(q.x,q.y,q.z,q.w);
        }
    }

    public Vector3 Scale {
        get {
            Vector3 parentScale = Vector3.one;
            if (transform.parent != null && transform.parent.GetComponent<Shape>() != null) {
                parentScale = transform.parent.GetComponent<Shape>().Scale;
            }
            return Vector3.Scale(transform.localScale, parentScale);
        }
    }
}
