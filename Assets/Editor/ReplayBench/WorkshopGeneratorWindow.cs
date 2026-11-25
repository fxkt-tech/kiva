using System;
using UnityEditor;
using UnityEngine;

internal sealed class WorkshopGeneratorWindow : EditorWindow
{
    [SerializeField] WorkshopModule modules=WorkshopModule.All;
    [SerializeField] WorkshopGear gear=WorkshopGear.Fan|WorkshopGear.Mushroom;
    [SerializeField] string sequence="";
    static string SpaceKey=>"WorkshopGenerator.quickSpace."+Application.dataPath;
    internal static WorkshopSpace SpacePreference{
        get=>(WorkshopSpace)EditorPrefs.GetInt(SpaceKey,(int)WorkshopSpace.Enclosed);
        set=>EditorPrefs.SetInt(SpaceKey,(int)value);
    }
    [SerializeField] int seed=Guid.NewGuid().GetHashCode();
    Vector2 scroll;
    (WorkshopModule,WorkshopGear,string,int,WorkshopSpace)? lastInput;
    WorkshopGenerationRequest request;
    string problem;
    internal static void Open()=>GetWindow<WorkshopGeneratorWindow>("关卡生成器").Show();
    void OnGUI(){
        scroll=EditorGUILayout.BeginScrollView(scroll);
        EditorGUILayout.LabelField("允许的场景模块",EditorStyles.boldLabel);
        foreach(var (flag,name) in new[]{(WorkshopModule.Fixed,"固定砖"),(WorkshopModule.Loose,"活动砖"),(WorkshopModule.White,"白面"),(WorkshopModule.Bamboo,"竹面")})
            modules=EditorGUILayout.ToggleLeft(name,(modules&flag)!=0)?modules|flag:modules&~flag;
        EditorGUILayout.Space();EditorGUILayout.LabelField("玩家可用道具",EditorStyles.boldLabel);
        foreach(var (flag,name) in new[]{(WorkshopGear.Gun,"时空枪"),(WorkshopGear.Mushroom,"弹弹菇"),(WorkshopGear.Fan,"风扇"),(WorkshopGear.Glider,"纸翼")})
            gear=EditorGUILayout.ToggleLeft(name,(gear&flag)!=0)?gear|flag:gear&~flag;
        EditorGUILayout.Space();EditorGUILayout.LabelField("过关操作顺序（可留空）",EditorStyles.boldLabel);
        sequence=EditorGUILayout.TextArea(sequence,GUILayout.MinHeight(44));
        for(int i=0;i<WorkshopGenerationRequest.Names.Length;i++){
            if(i%3==0)EditorGUILayout.BeginHorizontal();
            string name=WorkshopGenerationRequest.Names[i];
            if(GUILayout.Button(name))sequence=string.IsNullOrWhiteSpace(sequence)?name:sequence+" → "+name;
            if(i%3==2 || i==WorkshopGenerationRequest.Names.Length-1)EditorGUILayout.EndHorizontal();
        }
        EditorGUILayout.Space();
        var previous=SpacePreference;
        var space=(WorkshopSpace)EditorGUILayout.Popup("空间形态",(int)previous,new[]{"封闭","开放"});
        if(space!=previous)SpacePreference=space;
        using(new EditorGUILayout.HorizontalScope()){
            seed=EditorGUILayout.IntField("随机种子",seed);
            if(GUILayout.Button("换种子",GUILayout.Width(68)))seed=Guid.NewGuid().GetHashCode();
        }
        var inputs=(modules,gear,sequence,seed,space);
        if(!lastInput.HasValue || !lastInput.Value.Equals(inputs)){
            lastInput=inputs;request=null;problem=null;
            try{request=WorkshopGenerationRequest.Resolve(modules,gear,sequence,seed,space);}
            catch(ArgumentException e){problem=e.Message;}
        }
        if(problem!=null)EditorGUILayout.HelpBox(problem,MessageType.Warning);
        if(request!=null)EditorGUILayout.HelpBox((string.IsNullOrWhiteSpace(sequence)?"自动规划（本地动作搜索）　":"将验证　")+request.Description,MessageType.Info);
        using(new EditorGUI.DisabledScope(request==null || EditorApplication.isPlayingOrWillChangePlaymode))
            if(GUILayout.Button("按以上输入生成地图",GUILayout.Height(30)))WorkshopLevelGenerator.Generate(seed,request);
        using(new EditorGUI.DisabledScope(EditorApplication.isPlayingOrWillChangePlaymode))
            if(GUILayout.Button(new GUIContent("一键生成进阶关","使用当前空间形态和种子，自动配置道具与操作顺序。"),GUILayout.Height(30)))
                WorkshopLevelGenerator.Generate(seed,WorkshopGenerationRequest.Atrium(WorkshopGrade.Advanced,space));
        EditorGUILayout.EndScrollView();
    }
}
