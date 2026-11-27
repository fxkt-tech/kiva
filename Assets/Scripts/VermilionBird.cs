using UnityEngine;

// A hollow-looking painted body with independent, two-sided cut-paper wings, crest and tail.
public sealed class VermilionBird : MonoBehaviour
{
    public const int ModelLayer=8; // VermilionBird in TagManager.
    [SerializeField] Transform plumage, tail;
    [SerializeField] Transform[] wings, feet, eyes;
    float stride;

    public void Pose(Vector3 velocity,float yaw,bool grounded,float dt)
    {
        float step=grounded?Mathf.Clamp01(new Vector2(velocity.x,velocity.z).magnitude/4.8f):0;
        stride+=dt*step*14;
        float gait=Mathf.Sin(stride)*step;
        Face(yaw);
        float breath=grounded?(1-step)*Mathf.Sin(Time.time*2.4f):0;
        plumage.localPosition=Vector3.up*(Mathf.Abs(gait)*.052f+breath*.011f);
        plumage.localRotation=Quaternion.Euler(grounded?breath*.8f:Mathf.Clamp(-velocity.y,-12,16),0,gait*4);
        float blink=Mathf.Clamp01(Mathf.Abs(Mathf.Repeat(Time.time+.8f,4.1f)-.08f)/.08f);
        for(int i=0;i<2;i++){
            float sign=i==0?-1:1;
            eyes[i].localScale=new Vector3(1f,Mathf.Lerp(.07f,1f,blink),1f);
            feet[i].localRotation=Quaternion.Euler(gait*sign*23,0,0);
            wings[i].localRotation=Quaternion.Euler(0,0,sign*(grounded?gait*4:16+Mathf.Min(14,Mathf.Abs(velocity.y))));
        }
        tail.localRotation=Quaternion.Euler(grounded?gait*5:Mathf.Clamp(-velocity.y*2,-16,22),0,0);
    }

    public void Face(float degrees){transform.localRotation=Quaternion.Euler(0,degrees,0);}
}
