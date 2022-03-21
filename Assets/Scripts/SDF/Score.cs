using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

public class Score : MonoBehaviour
{

    public Text scoreText;
    private float _score;
    public float score
    {
        get { return _score; }
        set { _score = value; updateDisplay(); }
    }
    private void updateDisplay()
    {
        scoreText.text = score.ToString("0");
    }
    private void Start() {
        score = 0f;
    }
}
