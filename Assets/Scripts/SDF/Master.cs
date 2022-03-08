using System.Collections.Generic;
using UnityEngine;


[ExecuteInEditMode, ImageEffectAllowedInSceneView]
public class Master : MonoBehaviour
{
    public ComputeShader raymarching;
    public ComputeShader liteMarcher;

    public int liteModeAggressor = 16;
    [Range(0.000001f,1f)]
    public float liteEps = 0.3f;

    public GameObject player;

    public bool usesLiteMode = true; //might want to make 2 shaders and use the faster one then

    RenderTexture target;
    Camera cam;
    Light lightSource;
    List<ComputeBuffer> buffersToDispose;

    void Init()
    {
        cam = Camera.current;
        lightSource = FindObjectOfType<Light>();
    }


    void OnRenderImage(RenderTexture source, RenderTexture destination)
    {
        Init();
        buffersToDispose = new List<ComputeBuffer>();

        InitRenderTexture();
        CreateScene();
        SetParameters();

        if (!usesLiteMode)
        {
            raymarching.SetTexture(0, "Source", source);
            raymarching.SetTexture(0, "Destination", target);

            int threadGroupsX = Mathf.CeilToInt(cam.pixelWidth / 16.0f);
            int threadGroupsY = Mathf.CeilToInt(cam.pixelHeight / 16.0f);

            raymarching.Dispatch(0, threadGroupsX, threadGroupsY, 1);
        }
        else
        {
            liteMarcher.SetTexture(0, "Source", source);
            liteMarcher.SetTexture(0, "Destination", target);

            int threadGroupsX = Mathf.CeilToInt(cam.pixelWidth / 16.0f / liteModeAggressor);
            int threadGroupsY = Mathf.CeilToInt(cam.pixelHeight / 16.0f / liteModeAggressor);

            liteMarcher.Dispatch(0, threadGroupsX, threadGroupsY, 1);
        }


        Graphics.Blit(target, destination);

        foreach (var buffer in buffersToDispose)
        {
            buffer.Dispose();
        }
    }

    private void Update()
    {
        // first person kinda
        transform.position = player.transform.position;
        transform.forward = player.transform.forward;
    }

    void CreateScene()
    {
        List<Shape> allShapes = new List<Shape>(FindObjectsOfType<Shape>());
        allShapes.Sort((a, b) => a.operation.CompareTo(b.operation));

        List<Shape> orderedShapes = new List<Shape>();

        for (int i = 0; i < allShapes.Count; i++)
        {
            // Add top-level shapes (those without a parent)
            if (allShapes[i].transform.parent == null)
            {

                Transform parentShape = allShapes[i].transform;
                orderedShapes.Add(allShapes[i]);
                allShapes[i].numChildren = parentShape.childCount;
                // Add all children of the shape (nested children not supported currently)
                for (int j = 0; j < parentShape.childCount; j++)
                {
                    if (parentShape.GetChild(j).GetComponent<Shape>() != null)
                    {
                        orderedShapes.Add(parentShape.GetChild(j).GetComponent<Shape>());
                        orderedShapes[orderedShapes.Count - 1].numChildren = 0;
                    }
                }
            }

        }

        ShapeData[] shapeData = new ShapeData[orderedShapes.Count];
        for (int i = 0; i < orderedShapes.Count; i++)
        {
            var s = orderedShapes[i];
            Vector3 col = new Vector3(s.colour.r, s.colour.g, s.colour.b);
            shapeData[i] = new ShapeData()
            {
                //TODO: rotation
                position = s.Position,
                rotation = s.Rotation,
                scale = s.Scale,
                lightness = s.lightness,
                colour = col,
                shapeType = (int)s.shapeType,
                operation = (int)s.operation,
                blendStrength = s.blendStrength * 3,
                numChildren = s.numChildren
            };
        }

        ComputeBuffer shapeBuffer = new ComputeBuffer(shapeData.Length, ShapeData.GetSize());
        shapeBuffer.SetData(shapeData);

        if (!usesLiteMode)
        {
            raymarching.SetBuffer(0, "shapes", shapeBuffer);
            raymarching.SetInt("numShapes", shapeData.Length);
        }
        else
        {
            liteMarcher.SetBuffer(0, "shapes", shapeBuffer);
            liteMarcher.SetInt("numShapes", shapeData.Length);
        }


        buffersToDispose.Add(shapeBuffer);
    }

    void SetParameters()
    {
        if (!usesLiteMode)
        {
            bool lightIsDirectional = lightSource.type == LightType.Directional;
            raymarching.SetMatrix("_CameraToWorld", cam.cameraToWorldMatrix);
            raymarching.SetMatrix("_CameraInverseProjection", cam.projectionMatrix.inverse);
            raymarching.SetVector("_Light", (lightIsDirectional) ? lightSource.transform.forward : lightSource.transform.position);
            raymarching.SetBool("positionLight", !lightIsDirectional);
            raymarching.SetFloat("crazyEffectStrength", 0f);
        }
        else
        {
            bool lightIsDirectional = lightSource.type == LightType.Directional;
            liteMarcher.SetMatrix("_CameraToWorld", cam.cameraToWorldMatrix);
            liteMarcher.SetMatrix("_CameraInverseProjection", cam.projectionMatrix.inverse);
            liteMarcher.SetVector("_Light", (lightIsDirectional) ? lightSource.transform.forward : lightSource.transform.position);
            liteMarcher.SetBool("positionLight", !lightIsDirectional);
            liteMarcher.SetFloat("crazyEffectStrength", 0f);
            liteMarcher.SetInt("liteModeAggressor", liteModeAggressor);
            liteMarcher.SetFloat("epsilon", Mathf.Clamp(liteEps,0.000001f,1f));
        }
    }

    void InitRenderTexture()
    {
        if (target == null || target.width != cam.pixelWidth || target.height != cam.pixelHeight)
        {
            if (target != null)
            {
                target.Release();
            }
            target = new RenderTexture(cam.pixelWidth, cam.pixelHeight, 0, RenderTextureFormat.ARGBFloat, RenderTextureReadWrite.Linear);
            target.enableRandomWrite = true;
            target.Create();
        }
    }



    struct ShapeData
    {
        public Vector3 position;
        public Vector3 rotation;
        public Vector3 scale;
        public Vector3 colour;
        public float lightness;
        public int shapeType;
        public int operation;
        public float blendStrength;
        public int numChildren;

        public static int GetSize()
        {
            return sizeof(float) * 14 + sizeof(int) * 3;
        }
    }
}
