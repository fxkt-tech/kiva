using System;
using System.Collections.Generic;
using UnityEngine;

public sealed class StageBirds
{
    readonly WorkshopLevel stage;
    readonly BirdCollection collection;
    readonly WorkshopHud hud;
    readonly List<PerchedBird> birds=new List<PerchedBird>();
    readonly Action finish;
    Vector3 previous;
    float nextCall, settle, feedbackEnd, retryAt;
    bool completedHere;
    int count;

    public StageBirds(Transform root,WorkshopLevel stage,BirdCollection collection,WorkshopHud hud,Func<Color,Material> pigment,Action finish)
    {
        this.stage=stage;this.collection=collection;this.hud=hud;this.finish=finish;
        foreach(var perch in stage.perches){
            var anchor=new GameObject(perch.species.birdName);
            anchor.transform.SetParent(root,false);
            var bird=anchor.AddComponent<PerchedBird>();
            bird.Build(perch,Array.ConvertAll(perch.species.pigments,c=>pigment(c)));
            birds.Add(bird);
        }
    }
    public string Progress => birds.Count==1
        ? (count==0?"寻访":"已寻回")+birds[0].Perch.species.birdName
        : "已寻回 "+count+"/"+birds.Count;
    public void Enter()
    {
        count=collection.Count(stage);completedHere=false;settle=0;
        foreach(var bird in birds)bird.Refresh(collection.Has(stage.stageId,bird.Perch.id));
        hud.Collection(Progress);
    }
    public void ResetPosition(Vector3 body) { previous=body;settle=0;nextCall=Time.time+2;hud.HideDirection(); }
    public bool SavePending(Vector3 visitor)
    {
        if(!collection.NeedsSave)return true;
        if(!collection.Flush()){
            hud.Collection("收藏待保存 · 离开前将重试",true);retryAt=Time.unscaledTime+4;
            return false;
        }
        foreach(var bird in birds){
            if(!bird.Available || !collection.Has(stage.stageId,bird.Perch.id))continue;
            bird.ReturnToScroll(visitor);
            count++;
            hud.Collection(bird.Perch.species.birdName+"入卷　"+count+"/"+birds.Count);
        }
        if(count==birds.Count){
            completedHere=true;
            hud.Collection(birds.Count==1?birds[0].Perch.species.birdName+"归巢":"本关已集齐　"+count+"/"+birds.Count);
        }
        feedbackEnd=Time.time+.7f;
        return true;
    }
    public void Step(WorkshopMotor motor,Vector3 incoming)
    {
        var body=motor.Body.position;
        foreach(var bird in birds){
            if(!bird.Available || collection.IsPending(stage.stageId,bird.Perch.id))continue;
            bool touched=motor.CrossedThisStep
                ? Near(previous,motor.EntryCenter,bird.Chest) || Near(motor.ExitCenter,body,bird.Chest)
                : Near(previous,incoming,bird.Chest);
            if(touched)collection.Queue(stage.stageId,bird.Perch);
        }
        previous=body;
        if(collection.NeedsSave && Time.unscaledTime>=retryAt)SavePending(body);
        if(Time.time>=nextCall){
            PerchedBird nearest=null;float best=8*8;
            foreach(var bird in birds){
                float d=(bird.Chest-body).sqrMagnitude;
                if(bird.Available && d<best){nearest=bird;best=d;}
            }
            if(nearest){nearest.Call();hud.BirdCall(motor.Eyes,nearest.Chest);}
            nextCall=Time.time+(nearest?UnityEngine.Random.Range(6f,9f):1f);
        }
        if(!completedHere)return;
        settle=motor.Grounded && !motor.CrossedThisStep ? settle+Time.fixedDeltaTime : 0;
        if(settle>=.2f && Time.time>=feedbackEnd){completedHere=false;finish();}
    }

    // Closest point on a swept upright capsule. Its short central axis is vertical.
    static bool Near(Vector3 start,Vector3 end,Vector3 bird)
    {
        const float radius=.52f*WorkshopMotor.BirdScale;
        const float spine=WorkshopMotor.HalfHeight-radius;
        var step=end-start;
        var delta=bird-start;
        float best=float.PositiveInfinity;
        Vector3 chest=start;
        void Consider(float time){
            var center=start+Mathf.Clamp01(time)*step;
            var offset=bird-center;
            offset.y-=Mathf.Clamp(offset.y,-spine,spine);
            float distance=offset.sqrMagnitude;
            if(distance<best){best=distance;chest=center;}
        }
        // The minimum lies on a swept cap or in the strip between the two caps.
        float length=step.sqrMagnitude,flat=step.x*step.x+step.z*step.z;
        if(length<1e-10f)Consider(0);
        else{
            float dot=Vector3.Dot(delta,step);
            Consider((dot-spine*step.y)/length);
            Consider((dot+spine*step.y)/length);
            if(flat>1e-10f)Consider((delta.x*step.x+delta.z*step.z)/flat);
        }
        return best<=(radius+.45f)*(radius+.45f) &&
            !Physics.Linecast(chest,bird,Physics.DefaultRaycastLayers,QueryTriggerInteraction.Ignore);
    }
}
