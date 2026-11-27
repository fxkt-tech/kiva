using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEngine;
using Attempt = WorkshopGenerationReport.Attempt;
using Layout = WorkshopGenerationReport.Layout;
using Measurement = WorkshopGenerationReport.Measurement;
using Tape = WorkshopReplayBench.Tape;
using TimedInput = WorkshopReplayBench.TimedInput;

// Trials share one Play Mode session, with fresh game objects per trial.
public static class WorkshopLevelGenerator
{
    const string Menu="纸坊/关卡生成器/";
    const string State="WorkshopGenerator.job";
    const string Result="WorkshopGenerator.lastReport";
    const string Output="Assets/GeneratedLevels";
    const int MaxCandidates=8, MaxSteps=480;
    const float MinMargin=.20f, MinRunUp=.50f;
    enum Phase { Solution, ReplaySolution, Search, ReplayCounter, Perturb, Done, Watch, WatchCounter, Play, ComposeView, VerifyView, WatchOriginal, SkillSample }

    [Serializable] sealed class Job
    {
        public string id,report,templateHash,rulesHash;
        public int seed,index,sample,accepted=-1,viewPass;
        public Phase phase;
        public WorkshopGenerationRequest request;
        public List<Attempt> attempts=new List<Attempt>();
    }

    static Job job;
    static WorkshopLevel map;
    static FerryPuzzle puzzle;
    static Rigidbody brick;
    static WorkshopMotor motor;
    static WorkshopShortcutSearch search;
    static int regularSearchCount,searchCount;
    static WorkshopCourseTrial course;
    static List<WorkshopTool> shellTools;
    static bool Configured=>job.request!=null && job.request.version>0;
    static bool Modular=>Configured && !job.request.Classic;
    static Measurement events;
    static Tape inputTape,recording;
    static Tape preparedView;
    static WorkshopPlayerView viewDriver;
    static WorkshopViewAssessment assessment;
    static Vector3 springRunway;
    static bool Spatial=>Configured && job.request.version>=2;
    static bool Atrium=>Configured && job.request.IsAtrium;
    static int actionTick;
    static Vector3 mountPoint;
    static int tick,cursor,runStart,brakeStep,installStep;
    static bool retried,airborne,pushed,ended,springClicked,hopped;
    static bool generating;
    static int progressId=-1;
    static Vector3 runFrom;
    static BrickRegion platform;
    static Attempt Current=>job.attempts[job.index];
    static bool Preview=>job.phase==Phase.Watch || job.phase==Phase.WatchOriginal || job.phase==Phase.WatchCounter || job.phase==Phase.Play;
    static bool SavedMap=>!string.IsNullOrEmpty(job.report);
    static bool PreparingView=>job.phase==Phase.ComposeView || job.phase==Phase.VerifyView;

    [MenuItem(Menu+"按指定解法生成")]
    public static void GenerateNew()=>WorkshopGeneratorWindow.Open();
    const string QuickMenu=Menu+"按精彩程度生成/";
    [MenuItem(QuickMenu+"标准",false,100)] static void Standard()=>Quick(WorkshopGrade.Standard);
    [MenuItem(QuickMenu+"进阶",false,101)] static void Advanced()=>Quick(WorkshopGrade.Advanced);
    [MenuItem(QuickMenu+"挑战",false,102)] static void Challenge()=>Quick(WorkshopGrade.Challenge);
    static void Quick(WorkshopGrade grade)=>Generate(Guid.NewGuid().GetHashCode(),WorkshopGenerationRequest.Atrium(grade,WorkshopGeneratorWindow.SpacePreference));
    [MenuItem(QuickMenu+"标准",true),MenuItem(QuickMenu+"进阶",true),MenuItem(QuickMenu+"挑战",true)]
    [MenuItem(Menu+"按指定解法生成",true)]
    static bool CanLaunch()=>!EditorApplication.isPlayingOrWillChangePlaymode;
    [MenuItem(Menu+"取消生成",true)]
    static bool CanCancel()=>generating && EditorApplication.isPlaying && EditorApplication.isPlayingOrWillChangePlaymode;
    [MenuItem(Menu+"取消生成")]
    static void Cancel()
    {
        if(!CanCancel())return;
        StopTrials();
        Debug.Log("关卡生成已取消。");
        EditorApplication.ExitPlaymode();
    }
    [MenuItem(Menu+"查看上次生成报告")]
    static void ShowReport()=>SelectReport(LastReportPath());
    [MenuItem(Menu+"查看上次生成报告",true)]
    static bool HasReport()=>CanLaunch() && !string.IsNullOrEmpty(LastReportPath());

    [MenuItem(Menu+"播放最近生成关正解")]
    public static void ReplayLatest()
    {
        var report=AssetDatabase.LoadAssetAtPath<WorkshopGenerationReport>(LastReportPath());
        if(report && report.accepted>=0)Open(report,report.accepted,false);
        else throw new InvalidOperationException("请先生成一张通过验收的地图。");
    }
    [MenuItem(Menu+"播放最近生成关正解",true)]
    static bool CanReplayLatest(){
        if(!HasReport())return false;
        var report=AssetDatabase.LoadAssetAtPath<WorkshopGenerationReport>(LastReportPath());
        return report && report.accepted>=0;
    }

    public static void VerifySavedCounter()
    {
        var report=AssetDatabase.LoadAssetAtPath<WorkshopGenerationReport>(LastReportPath());
        Open(report,report.attempts.FindIndex(a=>a.counterConfirmed),true);
    }

    static string LastReportPath()
    {
        var path=EditorPrefs.GetString(Result+Application.dataPath,"");
        if(path.StartsWith(Output+"/",StringComparison.Ordinal) && File.Exists(path))return path;
        if(!Directory.Exists(Output))return "";
        var reports=Directory.GetFiles(Output,"Report.asset",SearchOption.AllDirectories);
        Array.Sort(reports,(a,b)=>File.GetLastWriteTimeUtc(b).CompareTo(File.GetLastWriteTimeUtc(a)));
        return reports.Length>0?reports[0].Replace('\\','/'):"";
    }

    public static void VerifyGeneration()
    {
        var args=Environment.GetCommandLineArgs();
        int option=Array.IndexOf(args,"-generationSeed");
        int seed=option<0?17041:int.Parse(args[option+1],System.Globalization.CultureInfo.InvariantCulture);
        var request=WorkshopGenerationRequest.Resolve(WorkshopModule.All,WorkshopGear.Fan|WorkshopGear.Mushroom,
            "风扇推活动砖 → 从该砖顶弹起",seed);
        Generate(seed,request);
    }
    internal static void Generate(int seed,WorkshopGenerationRequest request=null)
    {
        if(!CanLaunch())throw new InvalidOperationException("请先结束当前播放。");
        job=new Job{id=Guid.NewGuid().ToString("N"),seed=seed,request=request,rulesHash=RulesHash(),
            templateHash=request==null?AssetDatabase.GetAssetDependencyHash(WorkshopReplayBench.LevelPath).ToString():""};
        // Begin with an open landing platform, then use witnessed shortcuts to trim it.
        Add(new Layout{start=1,stop=4,nearX=3,nearZ=5});
        StartJob();
    }

    internal static void Open(WorkshopGenerationReport report,int index,bool counter,bool manual=false,bool original=false)
    {
        if(!CanLaunch())return;
        if(!report || index<0 || index>=report.attempts.Count)throw new InvalidOperationException("没有符合条件的回放。");
        job=new Job{report=AssetDatabase.GetAssetPath(report),index=index,
            phase=manual?Phase.Play:counter?Phase.WatchCounter:original?Phase.WatchOriginal:
                report.attempts[index].HasPlayerView?Phase.Watch:Phase.ComposeView};
        StartJob();
    }

    static void StartJob()
    {
        SessionState.EraseString(State+".verifiedView");
        SessionState.EraseString(State+".viewAssessment");
        SessionState.EraseBool(State+".invalidView");
        SessionState.SetString(State,JsonUtility.ToJson(job));
        WorkshopReplayBench.LaunchGeneration();
    }

    static void Add(Layout layout)
    {
        job.index=job.attempts.Count;
        job.attempts.Add(new Attempt{layout=layout});
        job.phase=Phase.Solution;job.sample=0;
        job.viewPass=0;job.accepted=-1;
    }

    static WorkshopLevel BuildMap(int index){
        string id=$"generated/{(Configured?"course":"push-bounce")}/{job.id}/{index+1}";
        var a=job.attempts[index];
        if(!Configured)return a.layout.Build(id);
        if(job.request.IsAtrium)return WorkshopAtrium.Build(job.request,id,job.seed,index,out a.legs);
        if(!Modular)return WorkshopCourse.PushMap(job.request,id,a.layout);
        var bypasses=new List<int>();
        for(int i=0;i<index;i++)if(job.attempts[i].counterConfirmed)bypasses.Add(job.attempts[i].counterLeg);
        return WorkshopCourse.Build(job.request,id,job.seed,index,bypasses.ToArray(),out a.legs);
    }

    static string RulesHash()
    {
        var parts=new System.Text.StringBuilder(Application.unityVersion);
        foreach(var script in new[]{"FerryPuzzle","WorkshopMotor","WorkshopGeometry","PaperTerrain","PaperDevice","WorkshopPlacement","WorkshopLayout","WorkshopBlock","BlockFaces","WorkshopFace"})
            parts.Append(AssetDatabase.GetAssetDependencyHash("Assets/Scripts/"+script+".cs"));
        foreach(var prefab in new[]{"WorkshopRig","LayoutTemplate"})
            parts.Append(AssetDatabase.GetAssetDependencyHash("Assets/Prefabs/"+prefab+".prefab"));
        foreach(var settings in new[]{"DynamicsManager","TimeManager"})
            parts.Append(File.ReadAllText("ProjectSettings/"+settings+".asset").Replace("\r\n","\n"));
        return Hash128.Compute(parts.ToString()).ToString();
    }

    internal static void BeginSession()
    {
        job=JsonUtility.FromJson<Job>(SessionState.GetString(State,""));
        preparedView=null;
        SessionState.SetFloat(State+".speed",Time.timeScale);
        if(!Preview){
            generating=true;
            SessionState.SetBool(State+".running",true);
            AssemblyReloadEvents.beforeAssemblyReload+=StopTrials;
            Time.timeScale=8;
            string title=SavedMap?"准备玩家视角回放":Atrium?$"按精彩程度生成 · {job.request.GradeText}":"按指定解法生成";
            progressId=Progress.Start(title);
            SessionState.SetInt(State+".progress",progressId);
            Progress.RegisterCancelCallback(progressId,()=>{Cancel();return true;});
        }
        BeginTrial();
    }

    // Script recompilation loses the physics and editor callbacks of an in-flight trial.
    [InitializeOnLoadMethod]
    static void CheckInterruptedSession()
    {
        if(SessionState.GetBool(State+".running",false))
            EditorApplication.delayCall+=()=>{
                if(!generating && SessionState.GetBool(State+".running",false) && EditorApplication.isPlaying){
                    Debug.Log("脚本已重新编译，当前关卡生成已停止。");
                    EditorApplication.ExitPlaymode();
                }
            };
    }

    static void BeginTrial()
    {
        if(SavedMap){
            var report=AssetDatabase.LoadAssetAtPath<WorkshopGenerationReport>(job.report);
            job.attempts=report.attempts;
            job.request=report.request;
            map=Current.map;
            if(!map)throw new InvalidDataException("找不到候选地图。");
            if(job.phase!=Phase.Play && (report.rulesHash!=RulesHash() ||
                Current.hash!=AssetDatabase.GetAssetDependencyHash(AssetDatabase.GetAssetPath(map)).ToString()))
                throw new InvalidDataException("地图、运动规则或物理设置已改变，请重新生成验证报告。");
        }else{
            if((!Configured && AssetDatabase.GetAssetDependencyHash(WorkshopReplayBench.LevelPath).ToString()!=job.templateHash) || RulesHash()!=job.rulesHash)
                throw new InvalidDataException("生成期间模板或物理规则已改变，请重新生成。");
            map=BuildMap(job.index);
        }
        if(!Modular){
            platform=Array.Find(map.fixedBricks,r=>r.label=="目标高台");
            if(platform.size==Vector3Int.zero)throw new InvalidDataException("模板缺少目标高台。");
        }
        retried=airborne=pushed=ended=springClicked=hopped=false;
        tick=cursor=runStart=brakeStep=installStep=actionTick=0;
        events=new Measurement();
        if(job.phase==Phase.Perturb){
            // Test all four corners; a change of heading can amplify or cancel the start offset.
            int corner=Current.tolerance.Count;
            events.approachOffset=corner<2?-.12f:.12f;
            events.headingOffset=corner%2==0?-2:2;
        }
        inputTape=null;
        if(job.phase==Phase.ReplaySolution || job.phase==Phase.WatchOriginal)inputTape=Current.solution;
        if(job.phase==Phase.VerifyView || job.phase==Phase.Watch)inputTape=preparedView??Current.playerView;
        if(job.phase==Phase.ReplayCounter || job.phase==Phase.WatchCounter)inputTape=Current.counter;
        if(inputTape!=null && inputTape.fixedDelta!=Time.fixedDeltaTime)throw new InvalidDataException("回放物理步长不匹配。");
        recording=new Tape{fixedDelta=Time.fixedDeltaTime};
        puzzle=WorkshopReplayBench.BuildSession(map,Finish);
        motor=puzzle.GetComponentInChildren<WorkshopMotor>();
        bool compose=job.phase==Phase.ComposeView;
        if(compose)recording.version=WorkshopPlayerView.TapeVersion;
        recording.viewProfile=job.viewPass;
        viewDriver=compose || job.phase==Phase.VerifyView || job.phase==Phase.Watch
            ?new WorkshopPlayerView(motor,puzzle.GetComponentInChildren<PlacementPreview>(true),compose?recording:inputTape,compose):null;
        if(SavedMap && PreparingView && !Application.isBatchMode)WorkshopReplayBench.ShowViewPreparation();
        if(Modular){
            shellTools=WorkshopShellBypass.Tools(job.request);
            int count=Current.legs.Count;
            int omit=job.phase==Phase.Search?job.sample%count:
                job.phase==Phase.WatchCounter || job.phase==Phase.ReplayCounter?Current.counterLeg:-1;
            int probe=job.phase==Phase.Search?job.sample/count:omit>=0?Current.counterProbe:0;
            var shell=probe>0?new WorkshopShellBypass(map,shellTools[probe-1]):null;
            var skill=job.phase==Phase.SkillSample?SkillVariation(job.sample):null;
            course=new WorkshopCourseTrial(puzzle,map,Current.legs,events,omit,viewDriver!=null,shell,skill?.delay??0,skill?.angle??0);
        }else{
          brick=puzzle.GetComponentInChildren<WorkshopBrick>().GetComponent<Rigidbody>();
          foreach(var device in puzzle.GetComponentsInChildren<PaperDevice>(true)){
            device.Applied+=(body,host)=>{
                if(device.Kind==WorkshopTool.Fan && body==brick){
                    if(!pushed && Mathf.Abs(brick.position.z-Current.layout.StartZ)<.12f)events.push=tick;
                    pushed=true;
                }
                if(device.Kind==WorkshopTool.Mushroom && body==motor.Body && host==brick && events.bounce==0 && device.transform.forward.y>.95f){
                    float at=pushed?Current.layout.StopZ:Current.layout.StartZ;
                    if(Mathf.Abs(brick.position.z-at)<.12f){
                        events.bounce=tick;
                        events.runUp=runStart>0?Vector3.Distance(motor.Body.position,runFrom):0;
                    }
                }
            };
          }
        }
        puzzle.Restarted+=()=>{if(tick>0)retried=true;};
        if(job.phase!=Phase.Play)puzzle.CollectInput=_=>{};
        puzzle.BeforePhysics=GuardedStep;
        puzzle.Begin();
        if(Atrium && job.phase==Phase.Solution)WorkshopAtrium.Inspect(map,Current.legs[0],motor,Current);
        assessment=Spatial && job.phase==Phase.VerifyView?new WorkshopViewAssessment(motor,Atrium):null;
        if(job.phase==Phase.Search && !Modular){
            var edge=(Vector3)platform.start*PaperTerrain.Cell+new Vector3(.4f,0,.4f);
            search=new WorkshopShortcutSearch(puzzle,map,false,edge);
            int sample=job.sample;
            regularSearchCount=searchCount=search.Count;
            if(Spatial && job.request.Enclosed && !job.request.HasSolidFinish){
                var walls=new WorkshopShortcutSearch(puzzle,map,true,roomWallsOnly:true);
                searchCount+=Math.Min(2,walls.Count);
                if(sample>=regularSearchCount){search=walls;sample-=regularSearchCount;}
            }
            for(int i=0;i<=sample;i++)if(!search.Next())throw new InvalidDataException("缺少所需的捷径样本。");
        }
        if(job.phase==Phase.Watch)Time.timeScale=1;
        if(generating && !Preview){
            Cursor.lockState=CursorLockMode.None;Cursor.visible=true;
            string stage=job.phase switch {
                Phase.Solution=>"尝试正解", Phase.ReplaySolution=>"重放正解",
                Phase.Search=>Modular?$"尝试绕过第 {job.sample%Current.legs.Count+1} 段 {course.ShellName}":$"搜索捷径 {job.sample+1}/{searchCount}", Phase.ReplayCounter=>"重放捷径",
                Phase.Perturb=>$"验证助跑偏差 {Current.tolerance.Count+1}/4", Phase.ComposeView=>"录制玩家视角操作",
                Phase.VerifyView=>"独立重放玩家视角操作",Phase.SkillSample=>$"实测操作容错 {job.sample+1}/4", _=>throw new InvalidOperationException()
            };
            Progress.Report(progressId,(float)job.index/MaxCandidates,$"候选 {job.index+1}/{MaxCandidates} · {stage}");
        }
    }

    static void GuardedStep()
    {
        if(ended)return;
        try{Step();}
        catch(Exception e){ended=true;WorkshopReplayBench.EndGeneration(false,e.ToString());}
    }

    static void Step()
    {
        tick++;
        if(job.phase==Phase.Play)return;
        assessment?.Travel();
        if(Modular){
            int before=course.Index;
            course.Observe(tick);
            if(course.Index!=before)assessment?.Arrive(before,Current.legs[before]);
            if(job.phase==Phase.Search && course.ShellUnavailable){Outcome(false,"起步处没有可见且能容纳该道具的围墙安装点");return;}
        }
        else if(events.bounce>0 && !motor.Grounded)airborne=true;
        if(!Modular && airborne && !events.landed && motor.Grounded){
            events.landed=true;events.landing=motor.Body.position;
            assessment?.Arrive(1,new WorkshopCourse.Leg{approach=springRunway,action=WorkshopAction.Spring});
            var min=(Vector3)platform.start*PaperTerrain.Cell;
            var max=(Vector3)(platform.start+platform.size)*PaperTerrain.Cell;
            var p=events.landing;
            events.edgeMargin=Mathf.Min(Mathf.Min(p.x-min.x,max.x-p.x),Mathf.Min(p.z-min.z,max.z-p.z))
                -motor.GetComponent<CapsuleCollider>().radius;
            if(Mathf.Abs(p.y-max.y-WorkshopMotor.HalfHeight)>.15f)events.edgeMargin=-1;
        }
        int budget=inputTape!=null?inputTape.endTick+(viewDriver!=null?150:0):Modular?MaxSteps*(Current.legs.Count+1):MaxSteps;
        if(retried || tick>budget){
            Outcome(false,retried?"跌落后重试":"耗尽物理步预算");
            return;
        }
        if(inputTape!=null && viewDriver==null){
            while(cursor<inputTape.commands.Count && inputTape.commands[cursor].tick==tick)
                Apply(inputTape.commands[cursor++].input);
            return;
        }
        WorkshopInput input;
        if(viewDriver!=null){
            try{input=ViewInput();}
            catch(InvalidOperationException e){Outcome(false,e.Message);return;}
        }else{
            actionTick=tick;
            input=Modular?course.Command(tick):job.phase==Phase.Search?search.Command(tick,0):PositiveInput();
        }
        if(inputTape!=null){Apply(viewDriver.ReplayInput(tick,input));return;}
        // Only movement persists in the generated commands. Keep every click and turn.
        var commands=recording.commands;
        var previous=commands.Count==0?Vector2.zero:commands[commands.Count-1].input.movement;
        bool wasHeld=commands.Count>0 && commands[commands.Count-1].input.spaceHeld;
        if(input.primary || input.secondary || input.spacePressed || wasHeld!=input.spaceHeld || input.selectMask!=0 || input.turn.x!=0 || input.turn.y!=0 || previous!=input.movement)
            commands.Add(new TimedInput{tick=tick,input=input});
        Apply(input);
    }

    static WorkshopInput ViewInput(){
        if(viewDriver.Aiming)return viewDriver.Aim();
        actionTick++;
        var intent=Modular?course.Command(actionTick):PositiveInput();
        if(intent.primary || intent.secondary){
            viewDriver.StartAim(intent,Modular?course.MountPoint:mountPoint);
            return viewDriver.Aim();
        }
        Vector3? gaze=Modular?course.LookAtPoint:null;
        if(!Modular && pushed && !events.landed){
            if(!springClicked)gaze=brick.position+Vector3.up*.75f;
            else if(motor.Body.position.x<map.exitFeet.x-1)gaze=map.exitFeet;
        }
        return viewDriver.Move(intent,gaze);
    }

    static void Apply(WorkshopInput input)
    {
        if(Modular)course.ObserveInput(input);
        if(input.primary && (input.selectMask & (1<<(int)WorkshopTool.Mushroom))!=0)springClicked=true;
        if(runStart==0 && springClicked && input.movement.sqrMagnitude>0){runStart=tick;runFrom=motor.Body.position;}
        puzzle.ApplyInput(input);
        if(assessment==null)return;
        bool click=input.primary || input.secondary;
        bool takeoff=input.spacePressed && motor.Grounded;
        if(!click && !takeoff)return;
        int i=Modular?course.Index:0;
        bool spring=click && (input.selectMask&(1<<(int)WorkshopTool.Mushroom))!=0;
        string action=!click?"起跳":spring?"安装弹弹菇":(input.selectMask&(1<<(int)WorkshopTool.Fan))!=0?"安装风扇":input.secondary?"放置出口门":"放置入口门";
        assessment.Look($"第 {i+1} 段{action}");
        if(Modular && !course.Complete && (takeoff || spring))
            assessment.CheckLanding(i,Current.legs[i].destination);
        if(!Modular && spring){
            springRunway=motor.Body.position;
            assessment.Arrive(0,new WorkshopCourse.Leg{approach=map.spawnFeet,action=WorkshopAction.Push});
            var size=(Vector3)platform.size*PaperTerrain.Cell;
            var top=(Vector3)platform.start*PaperTerrain.Cell+new Vector3(size.x/2,size.y,size.z/2);
            assessment.CheckLanding(0,new Bounds(top,new Vector3(size.x,.1f,size.z)));
        }
    }

    static WorkshopInput Aim(Vector3 direction,bool forward=false)
    {
        var angles=Quaternion.LookRotation(direction).eulerAngles;
        return new WorkshopInput{movement=forward?Vector2.up:Vector2.zero,turn=new Vector2(
            Mathf.DeltaAngle(motor.Yaw,angles.y),Mathf.DeltaAngle(motor.Eyes.transform.eulerAngles.x,angles.x))};
    }

    static WorkshopInput Place(Vector3 point,WorkshopTool tool)
    {
        mountPoint=point;
        var eye=motor.Body.position+Vector3.up*(.90f*WorkshopMotor.BirdScale-WorkshopMotor.HalfHeight);
        var input=Aim(point-eye);input.selectMask=1<<(int)tool;input.primary=true;return input;
    }

    static WorkshopInput PositiveInput()
    {
        if(actionTick==8)return Place(new Vector3(.75f,.75f,0),WorkshopTool.Fan);
        if(actionTick<24)return default;
        if(brakeStep==0){
            float align=Current.layout.StopZ+events.approachOffset-(viewDriver!=null?.12f:0);
            if(motor.Body.position.z<align-.38f)return Aim(Vector3.forward,true);
            brakeStep=actionTick;
        }
        if(installStep==0){
            if(actionTick-brakeStep<(viewDriver!=null?28:14) || Mathf.Abs(brick.position.z-Current.layout.StopZ)>.08f)return default;
            installStep=actionTick;
            return Place(brick.position+Vector3.up*(PaperTerrain.Cell*.5f),WorkshopTool.Mushroom);
        }
        if(actionTick-installStep<6)return default;
        if(viewDriver!=null && viewDriver.Profile!=1 && !events.landed){
            // Step onto the spring from above, clearing the seam between the ledge and moving brick.
            if(!hopped){hopped=true;return new WorkshopInput{spacePressed=true};}
            if(events.bounce==0){
                if(motor.Body.linearVelocity.y>.65f)return default;
                return Aim(Vector3.right,true);
            }
            var landing=Current.layout.NearCorner+new Vector3(.95f,0,1.10f);
            var direction=landing-motor.Body.position;direction.y=0;
            return direction.sqrMagnitude>.09f?Aim(direction,true):default;
        }
        if(events.landed){
            var delta=map.exitFeet-motor.Body.position;delta.y=0;
            return delta.magnitude>.28f?Aim(delta,true):default;
        }
        // The fan contributes forward speed. On trimmed platforms, angle the run slightly
        // inwards so the range of launch directions still clears the new near edge.
        float inward=platform.start.z>=7?-4f:0f;
        return Aim(Quaternion.Euler(0,inward+events.headingOffset,0)*Vector3.right,true);
    }

    static void Finish()
    {
        if(ended)return;
        try{events.finish=tick;Outcome(true,"已抵达出口");}
        catch(Exception e){ended=true;WorkshopReplayBench.EndGeneration(false,e.ToString());}
    }

    static void Outcome(bool settled,string reason)
    {
        bool positive=settled && !retried && events.push>0 && events.push<events.bounce &&
            events.bounce<=events.finish && events.landed && events.edgeMargin>=MinMargin && events.runUp>=MinRunUp;
        bool alternative=settled && !retried && !pushed && events.finish>0;
        if(Modular){
            positive=settled && !retried && course.Complete && course.Matched && course.CompletionTick<=events.finish;
            alternative=settled && !retried && events.finish>0 && !positive;
            if(!positive)reason+=" · "+course.Status;
        }
        if(viewDriver!=null && !viewDriver.AllClicksPlayed){positive=false;reason+=" · 安装动作未全部复现";}
        if(positive && Atrium && job.phase==Phase.VerifyView){
            float minimumTurn=job.request.grade switch {WorkshopGrade.Advanced=>12,WorkshopGrade.Challenge=>40,_=>0};
            if(events.airTurn<minimumTurn || assessment.result.visible<assessment.result.checkedTargets){
                positive=false;reason+=" · 起手时落台受遮挡或空中转向不足";
            }
        }
        if(Preview){
            bool ok=job.phase==Phase.Play?settled:job.phase!=Phase.WatchCounter?positive:
                alternative && (events.bounce>0)==(Current.shortcut.bounce>0);
            if(!ok && job.phase==Phase.Watch){
                SessionState.EraseString(State+".verifiedView");
                SessionState.SetBool(State+".invalidView",true);
            }
            ended=true;WorkshopReplayBench.EndGeneration(ok,ok?reason:"未复现记录的路线："+reason);return;
        }
        // The final landing may move a few ticks between clean physics sessions.
        recording.endTick=tick+(Modular || PreparingView?Mathf.CeilToInt(.4f/Time.fixedDeltaTime):3);
        recording.usedBrickAtStop=positive;
        bool viewTrial=PreparingView;
        switch(job.phase){
        case Phase.ComposeView:
        case Phase.VerifyView:
            if(!positive){
                preparedView=null;
                Debug.LogWarning($"玩家视角第 {job.viewPass+1} 次{(job.phase==Phase.ComposeView?"录制":"复验")}未通过：{reason}");
                if(++job.viewPass<2){job.phase=Phase.ComposeView;break;}
                string note="玩家视角操作尚未通过复验，可在报告中播放原始验证回放。";
                if(SavedMap){ended=true;WorkshopReplayBench.EndGeneration(false,note);return;}
                if(Atrium)Current.conclusion=note;
                Current.playerViewNote=note;job.phase=Phase.Done;break;
            }
            if(job.phase==Phase.ComposeView){
                preparedView=recording;
                Debug.Log($"已录制玩家视角 · {tick} 物理步 · {viewDriver.Clicks} 次瞄准后点击 · 最高转速 {viewDriver.PeakTurnSpeed:F1}°/秒");
                job.phase=Phase.VerifyView;break;
            }
            Debug.Log($"玩家视角独立回放已通过 · 抵达出口 {events.finish}");
            if(assessment!=null){
                assessment.Finish();
                var v=assessment.result;
                if(SavedMap)SessionState.SetString(State+".viewAssessment",JsonUtility.ToJson(v));
                else Current.viewAssessment=v;
                Debug.Log($"视角验收 · 行进转向 {v.courseTurns} / 操作转向 {v.lookTurns} / 最长直行 {v.longestStraight:F1} 米 / 落台可见 {v.visible}/{v.checkedTargets}");
            }
            if(SavedMap){
                preparedView.levelHash=Current.hash;
                SessionState.SetString(State+".verifiedView",JsonUtility.ToJson(preparedView));
                job.phase=Phase.Watch;
            }else{
                Current.playerView=preparedView;
                Current.playerViewNote="玩家视角操作已独立回放通关。";job.phase=Phase.Done;
            }
            break;
        case Phase.Solution:
            if(Atrium && (!Current.connected || !Current.stationVisible || !Current.exitVisible)){
                Reject($"中庭未通过入口及结构检查（连通 {Current.connected}、可见出口 {Current.exitVisible}、可见工位 {Current.stationVisible}）");break;
            }
            if(!positive){Reject("正解未满足验收："+reason);break;}
            Current.solution=recording;Current.nominal=events;job.phase=Phase.ReplaySolution;break;
        case Phase.ReplaySolution:
            if(!positive){Reject("独立回放未复现正解："+reason);break;}
            Current.solutionConfirmed=true;Current.nominal=events;job.phase=Phase.Search;
            if(Modular)NextCourseProbe();
            break;
        case Phase.Search:
            if(Modular){
                int leg=job.sample%Current.legs.Count;
                Current.samples.Add($"尝试绕过第 {leg+1} 段 · {course.ShellName??"常规移动"} · {reason}");
                if(alternative){Current.counterLeg=leg;Current.counterProbe=job.sample/Current.legs.Count;Current.shortcut=events;Current.counter=recording;job.phase=Phase.ReplayCounter;}
                else{job.sample++;NextCourseProbe();}
                break;
            }
            if(events.bounce>0 && job.sample<regularSearchCount)Current.launchedSamples++;
            var angles=WorkshopShortcutSearch.RunUpAngles;
            string probeLabel=job.sample>=regularSearchCount?"借用围墙":$"{(job.sample<angles.Length?"朝出口":"朝高台近角")}，偏转 {angles[job.sample%angles.Length]}°";
            Current.samples.Add($"样本 {job.sample+1}（{probeLabel}）· "+
                (events.bounce>0?$"第 {events.bounce} 步弹起":"未触发原位弹射")+"，"+reason+$"，{tick} 步");
            if(settled){
                Current.shortcut=events;Current.counter=recording;job.phase=Phase.ReplayCounter;
            }else if(++job.sample==searchCount){
                if(Current.launchedSamples==0)Reject("所有助跑样本均未实际触发弹射，覆盖不足。");
                else job.phase=Phase.Perturb;
            }
            break;
        case Phase.ReplayCounter:
            Current.counterConfirmed=alternative && (events.bounce>0)==(Current.shortcut.bounce>0);
            if(Current.counterConfirmed)Current.shortcut=events;
            Reject(Current.counterConfirmed?(Modular?"发现可重放的绕过指定动作路线":"发现可重放的免推砖捷径"):"疑似捷径未能重放，候选未通过");
            break;
        case Phase.Perturb:
            Current.tolerance.Add(events);
            if(!positive){Reject("扰动后未能稳定落台："+reason);break;}
            if(Current.tolerance.Count<4)break;
            Current.conclusion="正解已重放，四种起点与方向扰动组合均通过，采样范围内未找到原位起跳捷径。";
            job.accepted=job.index;job.phase=Phase.Done;break;
        case Phase.SkillSample:
            var skill=SkillVariation(job.sample);skill.passed=positive;Current.skillSamples.Add(skill);
            if(job.sample<2 && !positive){Reject("轻微操作偏差未能稳定通关");break;}
            if(++job.sample<4)break;
            if(job.request.grade!=WorkshopGrade.Standard && Current.skillSamples[2].passed && Current.skillSamples[3].passed){
                Reject("两次明显操作偏差仍能完成动作链，重选布局");break;
            }
            Current.conclusion=$"{job.request.GradeText}中庭已通过通关、入口视线与操作容错验证。";
            job.accepted=job.index;job.phase=Phase.Done;break;
        }
        ended=true;
        Debug.Log(Modular?$"生成候选 · {reason} · 抵达出口 {events.finish} · 抵达平台净余量 {events.edgeMargin:F2} m":
            $"生成候选 · {reason} · 推砖 {events.push} / 弹射 {events.bounce} / 抵达出口 {events.finish} · 落台净余量 {events.edgeMargin:F2} m");
        if(!SavedMap && Spatial && Modular && job.phase==Phase.Done){
            int rated=0,best=-1;
            for(int i=0;i<job.attempts.Count;i++){
                var a=job.attempts[i];
                if(a.viewAssessment==null || !a.viewAssessment.verified)continue;
                rated++;
                if(best<0 || BetterView(a,job.attempts[best]))best=i;
            }
            // A preset stops at the first fully qualified courtyard; a custom route compares two.
            if(Atrium && viewTrial && Current.viewAssessment?.verified!=true)job.accepted=-1;
            int required=Atrium?1:2;
            if(viewTrial && Current.legs.Count>1 && rated<required && job.attempts.Count<MaxCandidates)Add(default);
            else if(best>=0 && (viewTrial || job.accepted<0 || job.attempts[job.accepted].counterConfirmed))job.accepted=best;
        }
        if(job.phase==Phase.Done && job.accepted>=0 && !viewTrial && job.attempts[job.accepted].viewAssessment?.verified!=true){
            job.index=job.accepted;job.phase=Phase.ComposeView;
        }else if(job.phase==Phase.Done){
            generating=false;
            SessionState.SetString(State,JsonUtility.ToJson(job));
            WorkshopReplayBench.EndGeneration(job.accepted>=0,job.accepted>=0?job.attempts[job.accepted].conclusion:Current.conclusion);
            return;
        }
        WorkshopReplayBench.ReleaseTrial();
        if(!SavedMap)UnityEngine.Object.Destroy(map);
        EditorApplication.update+=ContinueAfterTrial;
    }

    static bool BetterView(Attempt candidate,Attempt incumbent){
        if(candidate.counterConfirmed!=incumbent.counterConfirmed)return !candidate.counterConfirmed;
        var a=candidate.viewAssessment;var b=incumbent.viewAssessment;
        int hidden=a.checkedTargets-a.visible,otherHidden=b.checkedTargets-b.visible;
        return hidden!=otherHidden?hidden<otherHidden:a.Score>b.Score;
    }

    static void ContinueAfterTrial()
    {
        // Destroy is deferred until the end of the frame. Never run two physics worlds on top of each other.
        if(puzzle)return;
        EditorApplication.update-=ContinueAfterTrial;
        if(!CanCancel())return;
        try{
            if(job.phase==Phase.Watch){
                generating=false;
                if(progressId>=0 && Progress.Exists(progressId))Progress.Remove(progressId);
            }
            BeginTrial();
        }
        catch(Exception e){WorkshopReplayBench.EndGeneration(false,e.ToString());}
    }

    static void NextCourseProbe(){
        int legs=Current.legs.Count,total=legs*(1+shellTools.Count);
        while(job.sample<total && Current.legs[job.sample%legs].action==WorkshopAction.Walk)job.sample++;
        if(job.sample<total)return;
        if(Atrium){job.phase=Phase.SkillSample;job.sample=0;return;}
        Current.conclusion="已重放指定路线并完成出口结算。唯一性未验证。";
        job.accepted=job.index;job.phase=Phase.Done;
    }

    static void Reject(string why)
    {
        Current.conclusion=why;
        if(job.attempts.Count>=MaxCandidates){
            if(Modular && !Atrium){
                job.accepted=job.attempts.FindIndex(a=>a.solutionConfirmed && a.counterConfirmed);
                if(job.accepted>=0)job.attempts[job.accepted].conclusion="存在第二解：指定路线和绕过指定动作的路线均已重放，可对比试玩。";
            }
            job.phase=Phase.Done;return;
        }
        if(Modular){Add(default);return;}
        var choices=new List<Layout>();
        // Keep the exit on the platform. Move its near edges beyond observed shortcut landings.
        var goalCell=map.exitFeet/PaperTerrain.Cell;
        for(int start=1;start<=2;start++)for(int stop=4;stop<=6;stop++)
        for(int x=3;x<Mathf.FloorToInt(goalCell.x);x++)for(int z=5;z<=Mathf.FloorToInt(goalCell.z);z++){
            var layout=new Layout{start=start,stop=stop,nearX=x,nearZ=z};
            if(job.attempts.Exists(a=>a.layout.Equals(layout)))continue;
            if(job.attempts.Exists(a=>a.counterConfirmed && a.layout.start==start && a.shortcut.landed &&
                a.shortcut.landing.x>=layout.NearCorner.x-.35f && a.shortcut.landing.z>=layout.NearCorner.z-.35f))continue;
            choices.Add(layout);
        }
        if(choices.Count==0){job.phase=Phase.Done;return;}
        var random=new System.Random(unchecked(job.seed+job.attempts.Count*7919));
        // Bias toward longer stationary jumps, retaining seed-dependent choices among them.
        choices.Sort((a,b)=>StationaryGap(b).CompareTo(StationaryGap(a)));
        Add(choices[random.Next(Math.Min(5,choices.Count))]);
    }

    static WorkshopGenerationReport.SkillSample SkillVariation(int i){
        bool jump=job.request.grade==WorkshopGrade.Standard;
        int delay=i switch {0=>jump?-2:2,1=>jump?1:4,_=>jump?9:65};
        float angle=i switch {0=>-2,1=>2,2=>-24,_=>24};
        return new WorkshopGenerationReport.SkillSample{delay=delay,angle=angle};
    }

    static float StationaryGap(Layout l)=>Vector2.Distance(new Vector2(.75f,l.StartZ),new Vector2(l.NearCorner.x,l.NearCorner.z));

    static void StopTrials()
    {
        generating=false;
        ended=true;
        EditorApplication.update-=ContinueAfterTrial;
        if(puzzle)puzzle.gameObject.SetActive(false);
    }

    internal static void LeavePlayMode()
    {
        StopTrials();
        AssemblyReloadEvents.beforeAssemblyReload-=StopTrials;
        SessionState.EraseBool(State+".running");
        if(map && !EditorUtility.IsPersistent(map))UnityEngine.Object.Destroy(map);
        Time.timeScale=SessionState.GetFloat(State+".speed",1);
        SessionState.EraseFloat(State+".speed");
    }

    internal static void EndSession(bool passed)
    {
        try{
            job=JsonUtility.FromJson<Job>(SessionState.GetString(State,""));
            if(job.phase==Phase.Done)Export();
            else if(SavedMap)SaveVerifiedView();
        }catch(Exception e){passed=false;Debug.LogException(e);}
        finally{
            progressId=SessionState.GetInt(State+".progress",-1);
            if(progressId>=0 && Progress.Exists(progressId))Progress.Remove(progressId);
            progressId=-1;
            SessionState.EraseInt(State+".progress");
            SessionState.EraseString(State);
            SessionState.EraseString(State+".verifiedView");
            SessionState.EraseString(State+".viewAssessment");
            SessionState.EraseBool(State+".invalidView");
        }
        if(Application.isBatchMode)EditorApplication.Exit(passed?0:1);
    }

    static void SaveVerifiedView(){
        string json=SessionState.GetString(State+".verifiedView","");
        bool invalid=SessionState.GetBool(State+".invalidView",false);
        if(!invalid && string.IsNullOrEmpty(json))return;
        var report=AssetDatabase.LoadAssetAtPath<WorkshopGenerationReport>(job.report);
        if(!report || job.index>=report.attempts.Count)return;
        var a=report.attempts[job.index];
        if(invalid){
            a.playerView=null;a.viewAssessment=null;a.playerViewNote="玩家视角未能复现，下次播放将重新准备。";
            EditorUtility.SetDirty(report);AssetDatabase.SaveAssetIfDirty(report);return;
        }
        var tape=JsonUtility.FromJson<Tape>(json);
        if(!a.map || a.hash!=tape.levelHash || report.rulesHash!=RulesHash() ||
            AssetDatabase.GetAssetDependencyHash(AssetDatabase.GetAssetPath(a.map)).ToString()!=a.hash)
            throw new InvalidDataException("地图或规则已改变，本次玩家视角回放未写入报告。");
        a.playerView=tape;a.playerViewNote="玩家视角操作已独立回放通关。";
        var metrics=SessionState.GetString(State+".viewAssessment","");
        if(!string.IsNullOrEmpty(metrics))a.viewAssessment=JsonUtility.FromJson<WorkshopGenerationReport.ViewAssessment>(metrics);
        EditorUtility.SetDirty(report);AssetDatabase.SaveAssetIfDirty(report);
    }

    static void Export()
    {
        if(!AssetDatabase.IsValidFolder(Output))AssetDatabase.CreateFolder("Assets","GeneratedLevels");
        string name=$"{(Configured?"Course":"PushBounce")}-{job.seed}-{job.id.Substring(0,8)}";
        string folder=AssetDatabase.GUIDToAssetPath(AssetDatabase.CreateFolder(Output,name));
        if(string.IsNullOrEmpty(folder))throw new IOException("无法创建生成结果目录。");
        try{
            string evidence=AssetDatabase.GUIDToAssetPath(AssetDatabase.CreateFolder(folder,"Editor"));
            if(string.IsNullOrEmpty(evidence))throw new IOException("无法创建报告目录。");
            for(int i=0;i<job.attempts.Count;i++){
                var a=job.attempts[i];
                if(i!=job.accepted && !a.counterConfirmed && !a.solutionConfirmed)continue;
                a.map=BuildMap(i);
                string path=i==job.accepted?folder+"/Level.asset":evidence+$"/Rejected-{i+1}.asset";
                AssetDatabase.CreateAsset(a.map,path);
                if(AssetDatabase.GetAssetPath(a.map)!=path)throw new IOException("未能保存地图 "+path);
                a.hash=AssetDatabase.GetAssetDependencyHash(path).ToString();
                if(a.solutionConfirmed)a.solution.levelHash=a.hash;
                if(a.playerView!=null && a.playerView.endTick>0)a.playerView.levelHash=a.hash;
                if(a.counterConfirmed)a.counter.levelHash=a.hash;
            }
            var report=ScriptableObject.CreateInstance<WorkshopGenerationReport>();
            report.seed=job.seed;report.accepted=job.accepted;report.attempts=job.attempts;
            report.request=job.request;
            report.unityVersion=Application.unityVersion;report.rulesHash=job.rulesHash;
            string reportPath=evidence+"/Report.asset";
            AssetDatabase.CreateAsset(report,reportPath);AssetDatabase.SaveAssets();
            if(AssetDatabase.GetAssetPath(report)!=reportPath)throw new IOException("未能保存验证报告。");
            EditorPrefs.SetString(Result+Application.dataPath,reportPath);
            string status=job.accepted<0?"未生成":job.attempts[job.accepted].counterConfirmed?$"可预览候选 {job.accepted+1}（存在第二解）":$"通过 {job.accepted+1}";
            Debug.Log($"关卡生成报告 {reportPath} · 候选 {job.attempts.Count} · {status}",report);
            SelectReport(reportPath);
        }catch{
            AssetDatabase.DeleteAsset(folder);throw;
        }
    }

    static void SelectReport(string path)
    {
        var report=AssetDatabase.LoadAssetAtPath<WorkshopGenerationReport>(path);
        if(report){Selection.activeObject=report;EditorGUIUtility.PingObject(report);}
    }
}
