using UnityEngine;
using UnityEngine.SceneManagement;
public class GameManager : MonoBehaviour
{
    public float restartDelay = 3f;
    public bool gameIsOver = false;
    public Canvas gameOverView;
    public void GameOver()
    {
        if (gameIsOver) return;
        Debug.Log("Game-Over");
        gameIsOver = true;
        gameOverView.enabled = true;
    }

    private Score score;
    private void Start()
    {
        score = FindObjectOfType<Score>();
        gameOverView.enabled = false;
    }
    public void Restart()
    {
        SceneManager.LoadScene(SceneManager.GetActiveScene().name);
    }
}
