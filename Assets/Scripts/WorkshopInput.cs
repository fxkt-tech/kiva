using System;
using UnityEngine;
using UnityEngine.InputSystem;

// Commands after key mapping and mouse sensitivity, before movement or aim validation.
[Serializable]
public struct WorkshopInput
{
    public Vector2 movement, turn;
    public bool spacePressed, spaceHeld, toggleGlide;
    public bool primary, secondary, retry, restart;
    public int selectMask, cycle;

    public static bool Read(out WorkshopInput input)
    {
        input=default;
        var keys=Keyboard.current;
        var mouse=Mouse.current;
        if(keys==null || mouse==null)return false;
        var options=GameSettings.Current;
        input.turn=mouse.delta.ReadValue()*(.22f*options.sensitivity);
        if(!options.invertY)input.turn.y=-input.turn.y;
        static float Axis(bool negative,bool positive)=>(positive?1:0)-(negative?1:0);
        input.movement=Vector2.ClampMagnitude(new Vector2(
            Axis(keys.aKey.isPressed || keys.leftArrowKey.isPressed,keys.dKey.isPressed || keys.rightArrowKey.isPressed),
            Axis(keys.sKey.isPressed || keys.downArrowKey.isPressed,keys.wKey.isPressed || keys.upArrowKey.isPressed)),1);
        input.spacePressed=keys.spaceKey.wasPressedThisFrame;
        input.spaceHeld=keys.spaceKey.isPressed;
        input.toggleGlide=options.glideToggle;
        for(int i=0;i<3;i++)if(keys[Key.Digit1+i].wasPressedThisFrame)input.selectMask|=1<<i;
        float scroll=mouse.scroll.ReadValue().y;
        input.cycle=scroll==0?0:scroll>0?-1:1;
        input.primary=mouse.leftButton.wasPressedThisFrame;
        input.secondary=mouse.rightButton.wasPressedThisFrame;
        input.retry=keys.rKey.wasPressedThisFrame;
        input.restart=keys.tKey.wasPressedThisFrame;
        return true;
    }
}
