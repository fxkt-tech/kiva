using System;
using System.Collections.Generic;
using UnityEditor;
using UnityEngine;
using Tape = WorkshopReplayBench.Tape;

public sealed class WorkshopGenerationReport : ScriptableObject
{
    public const string Actions="风扇推砖到止挡 → 从同一块砖顶弹射 → 抵达出口";
    [Serializable] internal struct Layout
    {
        public int start,stop,nearX,nearZ;
        public float StartZ=>(start+.5f)*PaperTerrain.Cell;
        public float StopZ=>(stop-.5f)*PaperTerrain.Cell;
        public Vector3 NearCorner=>new Vector3(nearX,0,nearZ)*PaperTerrain.Cell;

        internal WorkshopLevel Build(string id)
        {
            var source=AssetDatabase.LoadAssetAtPath<WorkshopLevel>(WorkshopReplayBench.LevelPath);
            var map=Instantiate(source);
            map.stageId=id;
            map.title="推砖渡槽";
            map.physicalBricks[0].cell.z=start;
            for(int i=0;i<map.fixedBricks.Length;i++){
                var region=map.fixedBricks[i];
                switch(region.label){
                    case "止挡": region.start.z=stop;break;
                    case "起步侧台": region.size.z=stop+1;break;
                    case "直槽底板": case "槽边": region.size.z=stop;break;
                    case "目标高台":
                        var end=region.start+region.size;
                        region.start.x=nearX;region.start.z=nearZ;
                        region.size=end-region.start;
                        break;
                }
                map.fixedBricks[i]=region;
            }
            return map;
        }
    }

    [Serializable] internal class Measurement
    {
        public int push,bounce,finish;
        public Vector3 landing;
        public bool landed;
        public float edgeMargin,runUp,approachOffset,headingOffset;
        public float airTurn,portalSpringSeconds;
        public List<string> trace=new List<string>();
    }
    [Serializable] internal class SkillSample
    {
        public int delay;
        public float angle;
        public bool passed;
    }

    [Serializable] internal class ViewAssessment
    {
        public bool verified;
        public int courseTurns,lookTurns,visible,checkedTargets;
        public float longestStraight;
        public List<string> details=new List<string>();
        internal float Score=>courseTurns*2+Math.Min(lookTurns,courseTurns+2)-longestStraight*.025f;
    }

    [Serializable] internal class Attempt
    {
        public Layout layout;
        public string conclusion;
        public Tape solution,counter,playerView;
        public string playerViewNote;
        public ViewAssessment viewAssessment;
        public bool connected,exitVisible,stationVisible;
        public List<SkillSample> skillSamples=new List<SkillSample>();
        public bool HasPlayerView=>playerView!=null && playerView.version==WorkshopPlayerView.TapeVersion && playerView.endTick>0 &&
            playerView.commands.Count>0 && playerView.levelHash==hash;
        public bool solutionConfirmed,counterConfirmed;
        public int launchedSamples;
        public Measurement nominal,shortcut;
        public List<Measurement> tolerance=new List<Measurement>();
        public List<string> samples=new List<string>();
        public WorkshopLevel map;
        public string hash;
        public int counterLeg;
        public int counterProbe;
        public List<WorkshopCourse.Leg> legs;
    }

    [SerializeField] internal int seed,accepted=-1;
    [SerializeField] internal string unityVersion,rulesHash;
    [SerializeField] internal WorkshopGenerationRequest request;
    [SerializeField] internal List<Attempt> attempts=new List<Attempt>();

    internal void DrawResults()
    {
        bool configured=request!=null && request.version>0;
        bool modular=configured && !request.Classic;
        if(configured){
            if(request.IsAtrium)EditorGUILayout.LabelField("精彩程度",request.GradeText+" · 中庭与回廊");
            EditorGUILayout.LabelField("空间形态",request.SpaceText);
            EditorGUILayout.LabelField("允许的场景模块",request.ModulesText);
            EditorGUILayout.LabelField("玩家可用道具",request.GearText);
            EditorGUILayout.LabelField("输入顺序",request.IsAtrium?"按档位自动编排":string.IsNullOrWhiteSpace(request.sequence)?"留空 · 本地自动规划":request.sequence,EditorStyles.wordWrappedLabel);
        }
        EditorGUILayout.LabelField(configured?request.Description:Actions,EditorStyles.wordWrappedLabel);
        EditorGUILayout.LabelField("随机种子",seed.ToString());
        EditorGUILayout.HelpBox(accepted<0?"候选预算用尽，请查看各次验证记录。":modular?attempts[accepted].conclusion:"正解已重放。采样范围内未找到原位起跳捷径，唯一性未验证。",MessageType.Info);
        using(new EditorGUI.DisabledScope(EditorApplication.isPlayingOrWillChangePlaymode)){
            if(accepted>=0){
                if(GUILayout.Button("播放生成关正解（玩家视角）"))WorkshopLevelGenerator.Open(this,accepted,false);
                if(!string.IsNullOrEmpty(attempts[accepted].playerViewNote))
                    EditorGUILayout.LabelField(attempts[accepted].playerViewNote,EditorStyles.wordWrappedMiniLabel);
                if(GUILayout.Button("播放原始验证回放"))WorkshopLevelGenerator.Open(this,accepted,false,original:true);
                if(GUILayout.Button("试玩生成关"))WorkshopLevelGenerator.Open(this,accepted,false,true);
            }
            for(int i=0;i<attempts.Count;i++){
                var a=attempts[i];
                EditorGUILayout.Space();
                EditorGUILayout.LabelField($"候选 {i+1}",EditorStyles.boldLabel);
                EditorGUILayout.LabelField(a.conclusion,EditorStyles.wordWrappedLabel);
                if(configured && request.IsAtrium){
                    EditorGUILayout.LabelField($"静态结构连通 {(a.connected?"是":"否")}　入口可见出口 {(a.exitVisible?"是":"否")} / 工位 {(a.stationVisible?"是":"否")}",EditorStyles.wordWrappedMiniLabel);
                    if(a.solutionConfirmed && a.nominal.airTurn>0)EditorGUILayout.LabelField($"相对弹射工位朝向，空中最大转角 {a.nominal.airTurn:F0}°"+(a.nominal.portalSpringSeconds>0?$"，出门至弹起 {a.nominal.portalSpringSeconds:F2} 秒":""),EditorStyles.wordWrappedMiniLabel);
                    for(int j=0;j<a.skillSamples.Count;j++){
                        var s=a.skillSamples[j];
                        float seconds=s.delay*a.solution.fixedDelta;
                        string shift=request.grade==WorkshopGrade.Standard?$"起跳点沿助跑方向 {seconds*4.8f:+0.00;-0.00;0} 米":$"开翼延后 {seconds:F2} 秒";
                        EditorGUILayout.LabelField($"{(j<2?"轻微":"明显")}偏差 · {shift}、方向偏转 {s.angle:+0;-0;0}° · {(s.passed?"通关":"未完成指定路线")}",EditorStyles.wordWrappedMiniLabel);
                    }
                }
                if(a.viewAssessment!=null && a.viewAssessment.verified){
                    var v=a.viewAssessment;
                    EditorGUILayout.LabelField($"行进转向 {v.courseTurns} 次，操作转向 {v.lookTurns} 次，最长连续直行 {v.longestStraight:F1} 米",EditorStyles.wordWrappedLabel);
                    if(v.checkedTargets>0)EditorGUILayout.LabelField($"起跳前可见落台 {v.visible}/{v.checkedTargets}");
                    foreach(var note in v.details)EditorGUILayout.LabelField(note,EditorStyles.wordWrappedMiniLabel);
                }
                if(!modular)EditorGUILayout.LabelField($"砖起点 Z {a.layout.StartZ:F2} m，止挡处砖心 Z {a.layout.StopZ:F2} m，高台近角 X/Z {a.layout.NearCorner.x:F1}/{a.layout.NearCorner.z:F1} m",EditorStyles.wordWrappedLabel);
                if(a.solutionConfirmed){
                    var m=a.nominal;
                    if(modular){
                        foreach(var line in m.trace)EditorGUILayout.LabelField(line,EditorStyles.wordWrappedMiniLabel);
                        EditorGUILayout.LabelField($"抵达出口 {m.finish}（物理步）");
                    }else{
                        EditorGUILayout.LabelField($"推砖 {m.push} → 弹射 {m.bounce} → 抵达出口 {m.finish}（物理步）",EditorStyles.wordWrappedLabel);
                        EditorGUILayout.LabelField($"正解助跑 {m.runUp:F2} m，首次落台净余量 {m.edgeMargin:F2} m。",EditorStyles.wordWrappedLabel);
                    }
                    foreach(var t in a.tolerance)
                        EditorGUILayout.LabelField($"助跑起点 Z 偏移 {t.approachOffset:+0.00;-0.00;0} m，方向 {t.headingOffset:+0;-0;0}°，助跑 {t.runUp:F2} m，落台净余量 {t.edgeMargin:F2} m",EditorStyles.wordWrappedLabel);
                    if(i!=accepted && GUILayout.Button($"播放候选 {i+1} 的正解"))WorkshopLevelGenerator.Open(this,i,false);
                }
                if(!modular && a.samples.Count>0)EditorGUILayout.LabelField($"捷径采样 {a.samples.Count} 次，原位起跳样本实际弹起 {a.launchedSamples} 次。");
                foreach(var sample in a.samples)EditorGUILayout.LabelField(sample,EditorStyles.wordWrappedMiniLabel);
                if(a.counterConfirmed && GUILayout.Button($"回放候选 {i+1} 的捷径"))WorkshopLevelGenerator.Open(this,i,true);
            }
            if(GUILayout.Button("用相同输入和种子重新生成"))WorkshopLevelGenerator.Generate(seed,configured?request:null);
        }
    }
}

[CustomEditor(typeof(WorkshopGenerationReport))]
sealed class WorkshopGenerationReportEditor : Editor
{
    public override void OnInspectorGUI()=>((WorkshopGenerationReport)target).DrawResults();
}
