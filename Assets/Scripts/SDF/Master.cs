using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;


[ExecuteInEditMode]//, ImageEffectAllowedInSceneView]
public class Master : MonoBehaviour
{

    public EnemyMan enemyMan;
    public bool isInvincible = false;
    // public ComputeShader raymarching;
    public ComputeShader liteMarcher;

    public int liteModeAggressor = 16;
    [Range(0.00001f, 1f)]
    public float liteEps = 0.3f;
    public float renderDistance = 40f;

    public GameObject player;

    // public bool usesLiteMode = true; 

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

        liteMarcher.SetTexture(0, "Source", source);
        liteMarcher.SetTexture(0, "Destination", target);

        int threadGroupsX = Mathf.CeilToInt(cam.pixelWidth / 16.0f / liteModeAggressor) + 1;
        int threadGroupsY = Mathf.CeilToInt(cam.pixelHeight / 16.0f / liteModeAggressor) + 1;

        liteMarcher.Dispatch(0, threadGroupsX, threadGroupsY, 1);


        Graphics.Blit(target, destination);


        // print(pixels);
        didCrashArr = new int[1] { 0 };
        didCrashBuffer.GetData(didCrashArr);
        foreach (int crash in didCrashArr)
        {
            if (crash > 0)
            {
                onCrash();
            }
        }

        DestroyBuffers();
        // print(crashHeatMap.);

    }
    private bool weCrashed = false;
    private void onCrash()
    {
        shake = 1f;
        print("crashed");
        if (!isInvincible)
        {
            weCrashed = true;
            StartCoroutine(fail(1));
        }
    }

    IEnumerator<WaitForSeconds> fail(int secs)
    {
        yield return new WaitForSeconds(secs);
        FindObjectOfType<GameManager>().GameOver();
    }
    private void OnPostRender()
    {

        // crashHeatMapImg.texture = crashHeatMap;
        // Graphics.DrawTexture(new Rect(),CrashHeatMap,null,-1);
    }
    private ComputeBuffer didCrashBuffer;
    private int[] didCrashArr;
    float shake = 0;
    public float shakeAmount = 0.7f;
    public float decreaseFactor = 1.0f;
    private void Update()
    {
        // first person kinda
        transform.position = player.transform.position;
        transform.forward = player.transform.forward;
        // crashHeatMapImg.texture = crashHeatMap;

        if (shake > 0)
        {
            transform.localPosition += Random.insideUnitSphere * shakeAmount;
            shake -= Time.deltaTime * decreaseFactor;
            enemyMan.timefactor = 1f - 5f * shake;
        }
        else
        {
            shake = 0.0f;
        }


    }
    private void FixedUpdate()
    {
        if(gameMan.gameIsOver)return;
        if (shake <= 0)
        {
            score.score += .05f;
            score.score *= 1.0001f;
        }
        else
        {
            score.score /= 1.0001f;
            score.score -= .1f;
        }
    }

    private Score score;
    private GameManager gameMan;
    private void Start()
    {
        score = FindObjectOfType<Score>();
        gameMan = FindObjectOfType<GameManager>();
    }
    private void OnDestroy() {
        DestroyBuffers();
    }

    private void DestroyBuffers(){
        foreach (var buffer in buffersToDispose)
        {
            buffer.Dispose();
        }
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


        liteMarcher.SetBuffer(0, "shapes", shapeBuffer);
        liteMarcher.SetInt("numShapes", shapeData.Length);


        buffersToDispose.Add(shapeBuffer);
    }

    void SetParameters()
    {
        liteMarcher.SetMatrix("_CameraToWorld", cam.cameraToWorldMatrix);
        liteMarcher.SetMatrix("_CameraInverseProjection", cam.projectionMatrix.inverse);
        // liteMarcher.SetFloat("crazyEffectStrength", 0f);
        SetFinalParameters();//TODO: do on startup
    }
    void SetFinalParameters()
    {
        didCrashBuffer = new ComputeBuffer(1, sizeof(int));
        didCrashBuffer.SetData(new int[1] { 0 });
        liteMarcher.SetBuffer(0, "CrashCheck", didCrashBuffer);
        buffersToDispose.Add(didCrashBuffer);
        bool lightIsDirectional = lightSource.type == LightType.Directional;
        liteMarcher.SetVector("_Light", (lightIsDirectional) ? lightSource.transform.forward : lightSource.transform.position);
        liteMarcher.SetBool("positionLight", !lightIsDirectional);
        liteMarcher.SetFloat("maxDst", renderDistance);
        liteMarcher.SetInt("liteModeAggressor", liteModeAggressor);
        liteMarcher.SetFloat("epsilon", Mathf.Clamp(liteEps, 0.00001f, 1f));
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
    //TODO: repetition
    {
        public Vector3 position;
        public Vector4 rotation;
        public Vector3 scale;
        public Vector3 colour;
        public float lightness;
        public int shapeType;
        public int operation;
        public float blendStrength;
        public int numChildren;

        public static int GetSize()
        {
            return sizeof(float) * 15 + sizeof(int) * 3;
        }
    }
}
