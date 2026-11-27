using UnityEngine;
using UnityEngine.UI;

public sealed class WorkshopHud : MonoBehaviour
{
    [SerializeField] Text hint, toolName;
    [SerializeField] WorkshopGlyph sight, wing;
    [SerializeField] WorkshopGlyph[] slots;
    [SerializeField] Image[] marks;
    float hintUntil, labelUntil;

    public void Hint(string value,float seconds=3){hint.text=value;hintUntil=Time.unscaledTime+seconds;}
    public void Selected(WorkshopTool tool){toolName.text=WorkshopToolSpec.For(tool).name;labelUntil=Time.unscaledTime+1.2f;}

    public void Paint(WorkshopTool tool,WorkshopGear gear,bool valid,bool first,bool second,bool gliding)
    {
        int index=(int)tool;
        var options=GameSettings.Current;
        sight.enabled=options.crosshair && WorkshopToolSpec.Owned(gear,index);
        sight.Show(index,true,!valid,first,second);
        for(int i=0;i<slots.Length;i++){
            bool owned=WorkshopToolSpec.Owned(gear,i);
            slots[i].Show(owned?i:-1);
            slots[i].color=new Color(1,1,1,i==index?1:owned?.7f:.32f);
            marks[i].enabled=owned && i==index;
        }
        wing.enabled=(gear & WorkshopGear.Glider)!=0;wing.Show(3,flying:gliding);
        toolName.enabled=Time.unscaledTime<labelUntil;
        if(Time.unscaledTime>hintUntil || !options.hints)hint.text="";
    }
}
