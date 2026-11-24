using UnityEditor;
using UnityEngine;

[CustomEditor(typeof(WorkshopLevel))]
public sealed class WorkshopLevelEditor : Editor
{
    void OnEnable(){SceneView.duringSceneGui+=Draw;}
    void OnDisable(){SceneView.duringSceneGui-=Draw;}
    void Draw(SceneView view){
        var stage=(WorkshopLevel)target;
        foreach(var perch in stage.perches){
            Handles.color=new Color(.2f,.65f,.77f);
            var chest=perch.feet+Vector3.up*.25f;
            Handles.DrawWireDisc(perch.feet,Vector3.up,.35f);
            Handles.DrawWireDisc(chest,view.camera.transform.forward,.45f);
            Handles.Label(chest+Vector3.up*.48f,perch.species?perch.species.birdName:perch.id);
        }
    }
}
