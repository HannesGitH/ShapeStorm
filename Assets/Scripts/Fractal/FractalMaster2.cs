using UnityEngine;

[ExecuteInEditMode, ImageEffectAllowedInSceneView]
public class FractalMaster2 : MonoBehaviour {

    public ComputeShader fractalShader;

    [Range (0, 20)]
    public float fractalPower = 1;
    public float darkness = 70;

    [Header ("Colour mixing")]

    [Range (0, 1)] public float blackAndWhite;
    public Color colorA;
    public Color colorB;

    RenderTexture target;
    Camera cam;
    Light directionalLight;

    [Header ("Animation Settings")]
    public float powerIncreaseSpeed = 0.2f;

    void Start() {
        Application.targetFrameRate = 60;
    }
    
    void Init () {
        cam = Camera.current;
        directionalLight = FindObjectOfType<Light> ();
    }

    // Animate properties
    void Update () {
        if (Application.isPlaying) {
            // fractalPower += powerIncreaseSpeed * Time.deltaTime;
            
        }
        // if(Input.GetKeyDown(KeyCode.Q))fractalPower+=powerIncreaseSpeed * Time.deltaTime;
        // if(Input.GetKeyDown(KeyCode.E))fractalPower-=powerIncreaseSpeed * Time.deltaTime;
        fractalPower += Input.GetAxis("Horizontal2") * powerIncreaseSpeed * Time.deltaTime;
    }

    void OnRenderImage (RenderTexture source, RenderTexture destination) {
        Init ();
        InitRenderTexture ();
        SetParameters ();

        int threadGroupsX = Mathf.CeilToInt (cam.pixelWidth / 8.0f);
        int threadGroupsY = Mathf.CeilToInt (cam.pixelHeight / 8.0f);
        fractalShader.Dispatch (0, threadGroupsX, threadGroupsY, 1);

        Graphics.Blit (target, destination);
    }

    void SetParameters () {
        fractalShader.SetTexture (0, "Destination", target);
        fractalShader.SetFloat ("power", Mathf.Max (fractalPower, 1.01f));
        fractalShader.SetFloat ("darkness", darkness);
        fractalShader.SetFloat ("blackAndWhite", blackAndWhite);
        fractalShader.SetVector ("colorAMix", colorA);
        fractalShader.SetVector ("colorBMix", colorB);

        fractalShader.SetMatrix ("_CameraToWorld", cam.cameraToWorldMatrix);
        fractalShader.SetMatrix ("_CameraInverseProjection", cam.projectionMatrix.inverse);
        fractalShader.SetVector ("_LightDirection", directionalLight.transform.forward);

    }

    void InitRenderTexture () {
        if (target == null || target.width != cam.pixelWidth || target.height != cam.pixelHeight) {
            if (target != null) {
                target.Release ();
            }
            target = new RenderTexture (cam.pixelWidth, cam.pixelHeight, 0, RenderTextureFormat.ARGBFloat, RenderTextureReadWrite.Linear);
            target.enableRandomWrite = true;
            target.Create ();
        }
    }
}