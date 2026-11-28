using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.UI;

[InitializeOnLoad]
public static class WorkshopReplayBench
{
    const string Folder="Assets/Editor/ReplayBench/";
    internal const string LevelPath=Folder+"PushAndBounce.asset";
    const string Menu="纸坊/推砖弹射基准关/";
    const string Key="WorkshopReplayBench.";
    const string LastTape="Library/WorkshopReplay/last.json";
    // Brick centre in the authored straight channel, measured in world metres.
    const float BrickStartZ=2.25f, BrickStopZ=5.25f;
    enum Mode { Idle, Record, Replay, Example, VariantOriginal, SearchWall, SearchBase, ReplayWall, ReplayBase, WallControl, Witness, RecordVariant, ReplayVariant, Generation }

    [Serializable]
    internal struct TimedInput
    {
        public int tick;
        public WorkshopInput input;
    }

    [Serializable]
    internal struct AimedClick
    {
        public Vector3 point;
        public int tool;
        public bool secondary;
    }

    [Serializable]
    internal sealed class Tape
    {
        public int version=1, endTick;
        public string levelHash;
        public float fixedDelta;
        public bool usedBrickAtStop;
        public bool wallSpring,updraft;
        public Vector3 wallPoint,springPoint;
        public List<TimedInput> commands=new List<TimedInput>();
        public int viewProfile;
        public List<AimedClick> aimedClicks=new List<AimedClick>();
    }

    static Mode mode;
    static FerryPuzzle puzzle;
    static Canvas sessionCanvas;
    static Tape playback,recording;
    static readonly List<WorkshopInput> pending=new List<WorkshopInput>();
    static int tick,cursor;
    static Rigidbody pushedBrick;
    static int pushTick,bounceTick;
    static WorkshopLevel level;
    static WorkshopShortcutSearch search;
    static int wallTick,wallKick,stationaryTick,liftTick;
    static bool fanPushedBrick,retried;
    static Vector3 actualWall,actualSpring;
    static bool Searching=>mode==Mode.SearchWall || mode==Mode.SearchBase;
    static bool ShortcutReplay=>mode==Mode.ReplayWall || mode==Mode.ReplayBase || mode==Mode.Witness;
    static bool Variant=>mode==Mode.VariantOriginal || mode==Mode.SearchWall || mode==Mode.ReplayWall || mode==Mode.Witness ||
        mode==Mode.RecordVariant || mode==Mode.ReplayVariant;
    static bool PlayerRecording=>mode==Mode.Record || mode==Mode.RecordVariant;
    static bool HasRecording=>PlayerRecording || Searching;
    static bool Verifying=>SessionState.GetBool(Key+"verify",false);
    const string VariantPlayerTape="Library/WorkshopReplay/side-wall-player.json";
    static string TapePath=>Verifying?SessionState.GetString(Key+"testTape",""):Variant?VariantPlayerTape:LastTape;
    static string ShortcutPath=>"Library/WorkshopReplay/"+(Variant || mode==Mode.WallControl?"wall":"base")+".json";

    static WorkshopReplayBench()=>EditorApplication.playModeStateChanged+=OnPlayMode;

    [MenuItem(Menu+"录制通关")]
    static void Record()=>Launch(Mode.Record);
    [MenuItem(Menu+"回放上次通关")]
    static void Replay()=>Launch(Mode.Replay);
    [MenuItem(Menu+"播放示例解法")]
    static void Example()=>Launch(Mode.Example);
    [MenuItem(Menu+"对照实验/搜索侧墙捷径")]
    static void SearchWall()=>StartSearch(Mode.VariantOriginal);
    [MenuItem(Menu+"对照实验/搜索基准图捷径")]
    static void SearchBase()=>StartSearch(Mode.SearchBase);
    [MenuItem(Menu+"对照实验/回放侧墙捷径")]
    static void ReplayWall()=>Launch(Mode.ReplayWall);
    [MenuItem(Menu+"对照实验/播放侧墙捷径示例")]
    static void Witness()=>Launch(Mode.Witness);
    [MenuItem(Menu+"对照实验/试玩并录制对照图")]
    static void RecordVariant()=>Launch(Mode.RecordVariant);
    [MenuItem(Menu+"对照实验/回放对照图试玩")]
    static void ReplayVariant()=>Launch(Mode.ReplayVariant);
    [MenuItem(Menu+"对照实验/回放对照图试玩",true)]
    static bool CanReplayVariant()=>CanLaunch() && File.Exists(VariantPlayerTape);
    [MenuItem(Menu+"对照实验/回放基准图捷径")]
    static void ReplayBase()=>Launch(Mode.ReplayBase);
    [MenuItem(Menu+"对照实验/播放对照图原解")]
    static void VariantOriginal()=>Launch(Mode.VariantOriginal);
    [MenuItem(Menu+"对照实验/回放侧墙捷径",true)]
    static bool CanReplayWall()=>CanLaunch() && File.Exists("Library/WorkshopReplay/wall.json");
    [MenuItem(Menu+"对照实验/回放基准图捷径",true)]
    static bool CanReplayBase()=>CanLaunch() && File.Exists("Library/WorkshopReplay/base.json");

    [MenuItem(Menu+"录制通关",true)]
    [MenuItem(Menu+"播放示例解法",true)]
    [MenuItem(Menu+"对照实验/搜索侧墙捷径",true)]
    [MenuItem(Menu+"对照实验/搜索基准图捷径",true)]
    [MenuItem(Menu+"对照实验/播放对照图原解",true)]
    [MenuItem(Menu+"对照实验/播放侧墙捷径示例",true)]
    [MenuItem(Menu+"对照实验/试玩并录制对照图",true)]
    static bool CanLaunch()=>!EditorApplication.isPlayingOrWillChangePlaymode;
    [MenuItem(Menu+"回放上次通关",true)]
    static bool CanReplay()=>CanLaunch() && File.Exists(LastTape);

    public static void VerifySideWall()=>StartSearch(Mode.VariantOriginal);
    public static void VerifyBaseSearch()=>StartSearch(Mode.SearchBase);
    public static void VerifyWitness()=>StartSearch(Mode.Witness);
    static void StartSearch(Mode start)
    {
        SessionState.SetBool(Key+"experiment",true);
        SessionState.SetBool(Key+"result",false);
        SessionState.SetInt(Key+"trial",0);
        Launch(start);
    }

    // Unity -batchmode -projectPath ... -executeMethod WorkshopReplayBench.VerifyExample
    public static void VerifyExample()
    {
        if(!Application.isBatchMode)throw new InvalidOperationException("请在批处理模式下运行回归检查。");
        SessionState.SetBool(Key+"verify",true);
        SessionState.SetString(Key+"testTape",Path.Combine(Path.GetTempPath(),"kiva-replay-"+Guid.NewGuid().ToString("N")+".json"));
        Launch(Mode.Record);
    }

    static void Launch(Mode requested)
    {
        if(!CanLaunch())throw new InvalidOperationException("请先结束当前播放。");
        var scene=AssetDatabase.LoadAssetAtPath<SceneAsset>(Folder+"Bench.unity");
        if(!scene)throw new FileNotFoundException("找不到基准关测试场景。");
        SessionState.SetString(Key+"previousScene",AssetDatabase.GetAssetPath(EditorSceneManager.playModeStartScene));
        SessionState.SetInt(Key+"mode",(int)requested);
        SessionState.SetBool(Key+"result",false);
        EditorSceneManager.playModeStartScene=scene;
        EditorApplication.EnterPlaymode();
    }

    internal static void LaunchGeneration()=>Launch(Mode.Generation);
    internal static void EndGeneration(bool passed,string message)=>End(passed,message);

    static void OnPlayMode(PlayModeStateChange change)
    {
        mode=(Mode)SessionState.GetInt(Key+"mode",0);
        if(mode==Mode.Idle)return;
        if(change==PlayModeStateChange.EnteredPlayMode){
            try{
                if(mode==Mode.Generation)WorkshopLevelGenerator.BeginSession();
                else StartRun();
            }
            catch(Exception e){End(false,e.ToString());}
        }
        if(change==PlayModeStateChange.ExitingPlayMode && mode==Mode.Generation)WorkshopLevelGenerator.LeavePlayMode();
        if(change==PlayModeStateChange.ExitingPlayMode && Searching)Time.timeScale=SessionState.GetFloat(Key+"speed",1);
        if(change==PlayModeStateChange.EnteredEditMode){
            var path=SessionState.GetString(Key+"previousScene","");
            EditorSceneManager.playModeStartScene=string.IsNullOrEmpty(path)?null:AssetDatabase.LoadAssetAtPath<SceneAsset>(path);
            SessionState.EraseInt(Key+"mode");
            var next=(Mode)SessionState.GetInt(Key+"next",0);
            SessionState.EraseInt(Key+"next");
            if(mode==Mode.Generation){
                WorkshopLevelGenerator.EndSession(SessionState.GetBool(Key+"result",false));
                return;
            }
            if(next!=Mode.Idle){
                EditorApplication.delayCall+=()=>Launch(next);
                return;
            }
            if(SessionState.GetBool(Key+"experiment",false)){
                SessionState.EraseBool(Key+"experiment");
                if(Application.isBatchMode)EditorApplication.Exit(SessionState.GetBool(Key+"result",false)?0:1);
            }
            if(Verifying){
                bool passed=SessionState.GetBool(Key+"passed",false);
                SessionState.EraseBool(Key+"passed");
                if(passed && mode==Mode.Record){
                    EditorApplication.delayCall+=()=>Launch(Mode.Replay);
                }else{
                    File.Delete(TapePath);
                    SessionState.EraseString(Key+"testTape");
                    SessionState.EraseBool(Key+"verify");
                    EditorApplication.Exit(passed?0:1);
                }
            }
        }
    }

    static void StartRun()
    {
        tick=cursor=0;
        pending.Clear();
        var path=Variant?Folder+"SideWall.asset":LevelPath;
        level=AssetDatabase.LoadAssetAtPath<WorkshopLevel>(path);
        var hash=AssetDatabase.GetAssetDependencyHash(path).ToString();
        recording=HasRecording?new Tape{levelHash=hash,fixedDelta=Time.fixedDeltaTime}:null;
        playback=null;
        if(mode==Mode.Example || mode==Mode.VariantOriginal || (Verifying && HasRecording))playback=ExampleInputs();
        if(mode==Mode.Replay || mode==Mode.ReplayVariant || ShortcutReplay || mode==Mode.WallControl){
            var file=ShortcutReplay || mode==Mode.WallControl?ShortcutPath:TapePath;
            if((ShortcutReplay || mode==Mode.WallControl) && SessionState.GetBool(Key+"experiment",false))file+=".candidate";
            if(mode==Mode.Witness)file=Folder+"SideWallWitness.json";
            playback=JsonUtility.FromJson<Tape>(File.ReadAllText(file));
            var recordedHash=mode==Mode.WallControl?AssetDatabase.GetAssetDependencyHash(Folder+"SideWall.asset").ToString():hash;
            if(playback==null || playback.version!=1 || playback.levelHash!=recordedHash || playback.fixedDelta!=Time.fixedDeltaTime)
                throw new InvalidDataException("回放版本、关卡内容或物理步长已改变，请重新录制。");
        }

        puzzle=BuildSession(level,Finish);
        puzzle.CollectInput=input=>{if(PlayerRecording && playback==null)pending.Add(input);};
        puzzle.BeforePhysics=BeforePhysics;
        retried=false;
        puzzle.Restarted+=()=>{
            pushedBrick=null;pushTick=bounceTick=0;
            if(tick>0)retried=true;
        };
        var player=puzzle.GetComponentInChildren<WorkshopMotor>().Body;
        var brick=puzzle.GetComponentInChildren<WorkshopBrick>().GetComponent<Rigidbody>();
        var origin=((Vector3)level.physicalBricks[0].cell+Vector3.one*.5f)*PaperTerrain.Cell;
        wallTick=stationaryTick=wallKick=liftTick=0;fanPushedBrick=false;
        foreach(var device in puzzle.GetComponentsInChildren<PaperDevice>(true)){
            device.Applied+=(body,host)=>{
                if(device.Kind==WorkshopTool.Fan && body==brick)fanPushedBrick=true;
                if(device.Kind==WorkshopTool.Fan && body==player && host==brick && wallKick>0 && liftTick==0 &&
                    Vector3.Distance(brick.position,origin)<.08f && device.transform.forward.y>.95f){
                    liftTick=tick;actualSpring=device.transform.position;
                }
                if(device.Kind==WorkshopTool.Fan && body==player && !host && stationaryTick==0){
                    wallTick=tick;actualWall=device.transform.position;
                }
                if(device.Kind==WorkshopTool.Mushroom && body==player && !host && wallKick==0 && Mathf.Abs(device.transform.forward.y)<.01f){
                    wallKick=tick;actualWall=device.transform.position;
                }
                if(device.Kind==WorkshopTool.Mushroom && body==player && host==brick && stationaryTick==0 &&
                    Vector3.Distance(brick.position,origin)<.08f && device.transform.forward.y>.95f){
                    stationaryTick=tick;actualSpring=device.transform.position;
                }
                if(device.Kind==WorkshopTool.Fan && body!=player && !pushedBrick && Mathf.Abs(body.position.z-BrickStartZ)<.12f){
                    pushedBrick=body;
                    pushTick=tick;
                }
                if(device.Kind==WorkshopTool.Mushroom && bounceTick==0 && body==player && host && host==pushedBrick &&
                    Mathf.Abs(host.position.z-BrickStopZ)<.12f && Vector3.Dot(device.transform.forward,Vector3.up)>.95f)
                    bounceTick=tick;
            };
        }
        puzzle.Begin();
        if(Searching){
            SessionState.SetFloat(Key+"speed",Time.timeScale);
            Time.timeScale=8;
            search=new WorkshopShortcutSearch(puzzle,level,Variant);
            int trial=SessionState.GetInt(Key+"trial",0);
            for(int i=0;i<=trial;i++)if(!search.Next()){End(false,"采样范围内没有更多候选，唯一性未验证。");return;}
            Debug.Log($"捷径搜索 {search.Number}/{search.Count} · 白面 {search.Wall:F2} · 砖顶 {search.Spring:F2}");
        }
    }

    internal static FerryPuzzle BuildSession(WorkshopLevel stage, Action finish)
    {
        sessionCanvas=new GameObject("关卡 HUD",typeof(RectTransform),typeof(Canvas)).GetComponent<Canvas>();
        sessionCanvas.renderMode=RenderMode.ScreenSpaceOverlay;
        T Instance<T>(string name, Transform parent=null) where T:Component
        {
            var prefab=AssetDatabase.LoadAssetAtPath<GameObject>("Assets/Prefabs/"+name+".prefab");
            return UnityEngine.Object.Instantiate(prefab,parent).GetComponent<T>();
        }
        var hud=Instance<WorkshopHud>("GameplayHUD",sessionCanvas.transform);
        puzzle=Instance<FerryPuzzle>("WorkshopRig");
        RenderSettings.sun=Instance<Light>("StageLight",puzzle.transform);
        RenderSettings.ambientMode=UnityEngine.Rendering.AmbientMode.Flat;
        RenderSettings.ambientLight=new Color(.61f,.64f,.69f);
        var layout=Instance<WorkshopLayout>("LayoutTemplate",puzzle.transform);
        layout.Populate(stage,(prefab,parent)=>UnityEngine.Object.Instantiate(prefab,parent));
        puzzle.Build(hud,layout,finish,null);
        if (stage.skybox)
        {
            RenderSettings.skybox = stage.skybox;
            puzzle.GetComponentInChildren<WorkshopMotor>().Eyes.clearFlags = CameraClearFlags.Skybox;
        }
        return puzzle;
    }

    internal static void ReleaseTrial()
    {
        puzzle.enabled=false;
        puzzle.gameObject.SetActive(false);
        UnityEngine.Object.Destroy(puzzle.gameObject);
        sessionCanvas.gameObject.SetActive(false);
        UnityEngine.Object.Destroy(sessionCanvas.gameObject);
    }

    internal static void ShowViewPreparation(){
        var panel=new GameObject("回放准备",typeof(RectTransform),typeof(Image)).GetComponent<Image>();
        panel.rectTransform.SetParent(sessionCanvas.transform,false);
        panel.rectTransform.anchorMin=Vector2.zero;panel.rectTransform.anchorMax=Vector2.one;
        panel.rectTransform.offsetMin=panel.rectTransform.offsetMax=Vector2.zero;
        panel.color=new Color(.12f,.13f,.14f);
        var label=new GameObject("进度",typeof(RectTransform),typeof(Text)).GetComponent<Text>();
        label.rectTransform.SetParent(panel.transform,false);
        label.rectTransform.anchorMin=Vector2.zero;label.rectTransform.anchorMax=Vector2.one;
        label.rectTransform.offsetMin=label.rectTransform.offsetMax=Vector2.zero;
        label.font=NightUi.Body;label.fontSize=20;label.alignment=TextAnchor.MiddleCenter;
        label.text="正在准备并验证玩家视角回放…";
    }

    static void BeforePhysics()
    {
        tick++;
        if(Searching){
            if(retried || tick>480){RejectTrial(retried?"离开场地后重试":"超出单条路线的 480 步预算");return;}
            pending.Add(search.Command(tick,wallKick));
        }
        if(playback!=null){
            if(tick>playback.endTick){
                if(mode==Mode.WallControl)FinishControl(false);
                else ReplayFailed("回放结束仍未触发落地结算。");
                return;
            }
            while(cursor<playback.commands.Count && playback.commands[cursor].tick==tick)
                pending.Add(playback.commands[cursor++].input);
        }
        foreach(var input in pending){
            recording?.commands.Add(new TimedInput{tick=tick,input=input});
            puzzle.ApplyInput(input);
        }
        pending.Clear();
    }

    static void Finish()
    {
        if(mode==Mode.WallControl){FinishControl(true);return;}
        bool usedBrick=pushTick>0 && bounceTick>=pushTick && tick>=bounceTick;
        if(Searching || ShortcutReplay){
            var wall=Searching?search.Wall:playback.wallPoint;
            var spring=Searching?search.Spring:playback.springPoint;
            bool wallSpring=Searching?search.WallSpring:playback.wallSpring;
            bool updraft=Searching?search.Updraft:playback.updraft;
            int launchTick=updraft?liftTick:stationaryTick;
            int impulse=wallSpring?wallKick:wallTick;
            bool wallMatched=!Variant || (impulse>0 && impulse<=launchTick && launchTick-impulse<=12 && Vector3.Distance(wall,actualWall)<.1f);
            if(retried || fanPushedBrick || launchTick<=0 || tick<launchTick || !wallMatched || Vector3.Distance(spring,actualSpring)>.1f){
                if(Searching)RejectTrial($"已结算，墙面风力步 {wallTick}、原位砖顶弹射步 {stationaryTick}、风推砖 {fanPushedBrick}");
                else ReplayFailed("反例回放未复现相同的作用面和跨越方式。");
                return;
            }
            if(Searching){recording.wallPoint=wall;recording.springPoint=spring;recording.wallSpring=wallSpring;recording.updraft=updraft;}
        }
        if(playback!=null && usedBrick!=playback.usedBrickAtStop){
            ReplayFailed("触发了结算，但实际经过的推砖弹射步骤与录制不一致。");
            return;
        }
        if(HasRecording){
            try{
                recording.endTick=tick+2;
                recording.usedBrickAtStop=usedBrick;
                var file=Searching?ShortcutPath+".candidate":TapePath;
                Directory.CreateDirectory(Path.GetDirectoryName(file));
                var temp=file+".tmp";
                File.WriteAllText(temp,JsonUtility.ToJson(recording));
                if(File.Exists(file))File.Replace(temp,file,null);
                else File.Move(temp,file);
            }catch(Exception e){End(false,e.Message);return;}
        }
        if(SessionState.GetBool(Key+"experiment",false)){
            if(mode==Mode.VariantOriginal)SessionState.SetInt(Key+"next",(int)Mode.SearchWall);
            if(Searching)SessionState.SetInt(Key+"next",(int)(Variant?Mode.ReplayWall:Mode.ReplayBase));
            if(mode==Mode.ReplayWall)SessionState.SetInt(Key+"next",(int)Mode.WallControl);
            if(mode==Mode.ReplayBase && !PublishShortcut())return;
        }
        if(Searching || ShortcutReplay){
            string assist=Variant?$"侧墙 {actualWall:F2} 在第 {Mathf.Max(wallTick,wallKick)} 步通过{(wallKick>0?"弹弹菇":"风扇")}推动角色，":"";
            string launch=liftTick>0?$"第 {liftTick} 步进入原位砖顶上升风场":$"第 {stationaryTick} 步从原位砖顶弹起";
            End(true,(Searching?"发现候选":"反例重放成功")+" · "+assist+launch+$"，第 {tick} 步抵达出口，已经落地结算，全程未用风推砖。");
            return;
        }
        End(true,usedBrick?
            $"风推砖（第 {pushTick} 步）→ 止挡处砖顶弹射（第 {bounceTick} 步）→ 抵达出口（第 {tick} 步）→ 落地结算。":
            "已通关，未观察到完整的推砖弹射链。");
    }

    static bool PublishShortcut()
    {
        try{
            var staged=ShortcutPath+".candidate";
            if(File.Exists(ShortcutPath))File.Replace(staged,ShortcutPath,null);
            else File.Move(staged,ShortcutPath);
            return true;
        }catch(IOException e){End(false,e.Message);return false;}
    }

    static void FinishControl(bool completed)
    {
        if((wallTick>0 || wallKick>0) && Vector3.Distance(actualWall,playback.wallPoint)<.1f){
            ReplayFailed("相同输入在基准图仍触发该墙面的装置，无法将差异归因到新增白面。");return;
        }
        if(PublishShortcut())End(true,"原图对照：相同输入未触发新增墙面的装置，"+(completed?"仍然完成结算。":"在录制时限内未结算。"));
    }

    static void ReplayFailed(string why)
    {
        if(SessionState.GetBool(Key+"experiment",false) &&
            (mode==Mode.ReplayBase || mode==Mode.ReplayWall || mode==Mode.WallControl)){
            SessionState.SetInt(Key+"trial",SessionState.GetInt(Key+"trial",0)+1);
            SessionState.SetInt(Key+"next",(int)(mode==Mode.ReplayBase?Mode.SearchBase:Mode.SearchWall));
            End(true,why+" 继续检查下一个候选。");
        }else End(false,why);
    }

    static void RejectTrial(string why)
    {
        if(search.Number<search.Count){
            SessionState.SetInt(Key+"trial",search.Number);
            Debug.Log($"候选 {search.Number}/{search.Count} · {why}");
            tick=cursor=wallTick=wallKick=stationaryTick=liftTick=0;
            fanPushedBrick=retried=false;
            recording.commands.Clear();
            pending.Clear();
            search.Next();
            puzzle.Begin();
        }else End(false,$"已采样 {search.Count} 条助跑路线，{why}。未确认此类捷径，唯一性未验证。");
    }

    static void End(bool success,string result)
    {
        if(puzzle)puzzle.enabled=false;
        string message=mode==Mode.Generation?"关卡生成器 · "+result:"推砖弹射基准关 · "+mode+" · 物理步 "+tick+" · "+result;
        if(success)Debug.Log(message);else Debug.LogError(message);
        SessionState.SetBool(Key+"result",success);
        if(Verifying)SessionState.SetBool(Key+"passed",success);
        EditorApplication.ExitPlaymode();
    }

    static Tape ExampleInputs()
    {
        // A hand-authored input tape. All clicks still aim through the player's camera.
        var tape=new Tape{endTick=310,usedBrickAtStop=true};
        void At(int step,WorkshopInput input)=>tape.commands.Add(new TimedInput{tick=step,input=input});
        At(8,new WorkshopInput{turn=new Vector2(0,39.5f),selectMask=4,primary=true});
        At(24,new WorkshopInput{turn=new Vector2(-137,-35.5f),movement=Vector2.up});
        At(64,default);
        At(110,new WorkshopInput{turn=new Vector2(90,25.2f),selectMask=2,primary=true});
        At(120,new WorkshopInput{movement=Vector2.up,turn=new Vector2(0,-25.2f)});
        At(145,new WorkshopInput{turn=new Vector2(-32,0)});
        At(204,default);
        return tape;
    }
}
