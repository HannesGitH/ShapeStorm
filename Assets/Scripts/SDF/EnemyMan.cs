using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class EnemyMan : MonoBehaviour
{


    public GameObject breb;
    public float speed = 5f;

    public float despawnZ = -35f;

    private List<GameObject> enemies = new List<GameObject>();
    void Start() {
        for (int y=0; y<2; ++y)
       {
           for (int x=0; x<2; ++x)
           {
               enemies.Add(Instantiate(breb, new Vector3(5*x,5*y,50), Quaternion.identity));
           }
       }
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
        }
        // remove out of bounds enemies from List
        foreach (int i in indices_to_remove)enemies.RemoveAt(i);
    }

    // void OnDestroy() {
    //     DestroyImmediate(breb);
    // }

    public float timefactor = 1;

}
