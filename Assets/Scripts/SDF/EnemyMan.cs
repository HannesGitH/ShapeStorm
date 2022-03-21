using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class EnemyMan : MonoBehaviour
{

    public double SpawnLength = 5;
    public float SpawnRadius = 15;
    public float MaxSize = 15;
    public float MinSize = 15;
    public float speed = 5f;
    public bool randomSeed = true;
    public int fixedSeed;

    public float despawnZ = -35f;

    private List<Enemy> enemies = new List<Enemy>();    
    private Score score;
    private void Start()
    {
        score = FindObjectOfType<Score>();
        Random.seed = randomSeed ? (int)(Time.realtimeSinceStartupAsDouble%1)*int.MaxValue : fixedSeed;
    //     for (int y=0; y<2; ++y)
    //    {
    //        for (int x=0; x<2; ++x)
    //        {
    //            enemies.Add(Instantiate(breb, new Vector3(5*x,5*y,50), Quaternion.identity));
    //        }
    //    }
    }

    private void Reset() {
        DestroyAllEnemies();
    }

    private void OnDestroy() {
        Reset();
    }

    private void DestroyAllEnemies(){
        foreach (Enemy enemy in enemies)
        {
            Destroy(enemy.em);
        }
        enemies.Clear();
    }

    void Update() {
        List<int> indices_to_remove = new List<int>();
        for (int i = 0; i < enemies.Count; i++)
        {
            GameObject enemy = enemies[i].em;
            if(enemy.transform.position.z <= despawnZ){
                Debug.Log("destroyed "+enemy.name);
                Destroy(enemy);
                indices_to_remove.Add(i);
                break;
            }
            enemy.transform.Translate(new Vector3(0,0,1f)*Time.deltaTime*-speed*timefactor);
            //TODO: rotation?
            // Vector3 curPos = enemy.transform.position;
            // enemy.transform.position = new Vector3();
            // enemy.transform.Rotate(enemies[i].rot.eulerAngles, Time.deltaTime*speed*2);
            // enemy.transform.position = curPos;
        }
        // remove out of bounds enemies from List
        foreach (int i in indices_to_remove)enemies.RemoveAt(i);

        randomSpawn();
    }

    // void OnDestroy() {
    //     DestroyImmediate(breb);
    // }

    public float timefactor = 1;

    private void randomSpawn(){
        timeSinceLastSpawn += Time.deltaTime;
        if (Random.value*timeSinceLastSpawn*score.score>SpawnLength)
        {
            timeSinceLastSpawn = 0;
            SpawnRandomShape();
        }
    }
    private void SpawnRandomShape(){
        GameObject nextShapeObj = new GameObject("shape-"+Random.value.ToString());
        nextShapeObj.AddComponent<Shape>();
        Shape nextShape = nextShapeObj.GetComponent<Shape>();
        nextShape.operation = Shape.Operation.None;
        nextShape.shapeType = (Shape.ShapeType)Random.Range(0, System.Enum.GetValues(typeof(Shape.ShapeType)).Length);
        Vector2 icr = Random.insideUnitCircle*SpawnRadius;
        nextShapeObj.transform.position = new Vector3(icr.x,icr.y,30);
        //todo:roatation along its own centre
        // nextShapeObj.transform.rotation = Random.rotation;
        nextShapeObj.transform.localScale = Random.insideUnitSphere*(MaxSize-MinSize)+new Vector3(1,1,1)*MinSize;
        enemies.Add(new Enemy(){em=nextShapeObj,rot=Random.rotation});
        Debug.Log("spawned an enemy, list has objs: "+enemies.Count);
    }
    private double timeSinceLastSpawn = 0;

}

struct Enemy
{
    public GameObject em;
    public Quaternion rot;
}
