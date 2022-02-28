using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class EnemyMan : MonoBehaviour
{


    public GameObject breb;
    public float speed = 5f;

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
        foreach (GameObject enemy in enemies)
        {
            enemy.transform.Translate(new Vector3(0,0,1f)*Time.deltaTime*-speed);
        }
    }

    // void OnDestroy() {
    //     DestroyImmediate(breb);
    // }

}
