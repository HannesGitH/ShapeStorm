using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class EnemyMan : MonoBehaviour
{

    public double SpawnLength = 5;
    public float SpawnRadius = 15;
    public float speed = 5f;
    public int? randomSeed = null;

    public float despawnZ = -35f;

    private List<GameObject> enemies = new List<GameObject>();
    void Start() {
        Random.seed = randomSeed ?? (int)(Time.realtimeSinceStartupAsDouble%1)*int.MaxValue;
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
        foreach (GameObject enemy in enemies)
        {
            Destroy(enemy);
        }
        enemies.Clear();
    }

    void Update() {
        List<int> indices_to_remove = new List<int>();
        for (int i = 0; i < enemies.Count; i++)
        {
            GameObject enemy = enemies[i];
            if(enemy.transform.position.z <= despawnZ){
                Destroy(enemy);
                indices_to_remove.Add(i);
                break;
            }
            enemy.transform.Translate(new Vector3(0,0,1f)*Time.deltaTime*-speed*timefactor);
            // enemy.transform.RotateAround();
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
        if (Random.value*timeSinceLastSpawn>SpawnLength)
        {
            timeSinceLastSpawn = 0;
            SpawnRandomShape();
        }
    }
    private void SpawnRandomShape(){
        GameObject nextShapeObj = new GameObject("shape");
        nextShapeObj.AddComponent<Shape>();
        Shape nextShape = nextShapeObj.GetComponent<Shape>();
        nextShape.operation = Shape.Operation.None;
        nextShape.shapeType = (Shape.ShapeType)Random.Range(0, System.Enum.GetValues(typeof(Shape.ShapeType)).Length);
        Vector2 icr = Random.insideUnitCircle*SpawnRadius;
        nextShape.transform.position = new Vector3(icr.x,icr.y,50);
        nextShape.transform.rotation = Random.rotation;
        enemies.Add(nextShapeObj);
    }
    private double timeSinceLastSpawn = 0;

}

struct Enemy
{
    
}
