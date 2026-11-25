using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;

[Flags] internal enum WorkshopModule { Fixed=1, Loose=2, White=4, Bamboo=8, All=Fixed|Loose|White|Bamboo }
internal enum WorkshopAction { Walk, Jump, Push, Spring, Glide, Wind, Portal, LooseSpring }
internal enum WorkshopSpace { Enclosed, Open }

[Serializable]
internal sealed class WorkshopGenerationRequest
{
    public int version;
    public WorkshopSpace space;
    public bool Enclosed=>version>=2 && space==WorkshopSpace.Enclosed;
    public string SpaceText=>Enclosed?"封闭":"开放";
    public WorkshopModule modules;
    public WorkshopGear gear;
    public string sequence;
    public WorkshopAction[] actions;
    public static readonly string[] Names={"步行","跳过断口","风扇推活动砖","踩弹弹菇起跳","展开纸翼滑翔","借风扇跨越断口","穿过传送门","从活动砖顶弹起"};
    static readonly string[] Patterns={
        "(?:步行|行走|走路|走到(?:目标|鸟旁))",
        "(?:起跳|跳跃|跳过(?:断口|缺口|沟)|助跑跳跃)",
        "(?:(?:安装|放置|用|使用)?风扇)?(?:把活动砖推|推(?:动)?(?:活动|纸)?砖)(?:到(?:止挡|断口|尽头))?",
        "(?:(?:在|从|借|踩上?)?(?:同一块|该|这块|原位|活动)?砖顶(?:的|上)?(?:装|安装)?(?:弹弹菇|菇)?(?:并)?(?:助跑)?(?:弹起|弹射|起跳)|(?:踩|借|使用|用)?(?:弹弹菇|蘑菇)(?:弹射|起跳|弹起)?|弹射)",
        "(?:(?:展开|用|使用)?纸翼(?:滑翔|转向)?(?:到|至)?(?:目标台|目标平台|高台)?|滑翔)",
        "(?:借风扇跨越断口|风扇吹玩家|借风跨越|借风越过断口)",
        "(?:穿过传送门|穿门|传送|用时空枪穿门)"
    };
    public bool Has(WorkshopModule m)=>(modules&m)==m;
    public bool Classic=>actions!=null && actions.SequenceEqual(new[]{WorkshopAction.Push,WorkshopAction.Spring});
    public string Description=>string.Join(" → ",actions.Select((a,i)=>
        a==WorkshopAction.Spring && i>0 && actions[i-1]==WorkshopAction.Push?"从同一块砖顶弹射":Names[(int)a]))+" → 收鸟并落地";
    public string ModulesText=>string.Join("、",new[]{WorkshopModule.Fixed,WorkshopModule.Loose,WorkshopModule.White,WorkshopModule.Bamboo}
        .Where(Has).Select(m=>m switch {WorkshopModule.Fixed=>"固定砖",WorkshopModule.Loose=>"活动砖",WorkshopModule.White=>"白面",_=>"竹面"}));
    public string GearText=>gear==0?"无":string.Join("、",new[]{WorkshopGear.Gun,WorkshopGear.Mushroom,WorkshopGear.Fan,WorkshopGear.Glider}
        .Where(g=>(gear&g)!=0).Select(g=>g switch {WorkshopGear.Gun=>"时空枪",WorkshopGear.Mushroom=>"弹弹菇",WorkshopGear.Fan=>"风扇",_=>"纸翼"}));

    public static WorkshopGenerationRequest Resolve(WorkshopModule modules,WorkshopGear gear,string text,int seed,WorkshopSpace space=WorkshopSpace.Enclosed)
    {
        var r=new WorkshopGenerationRequest{version=2,modules=modules,gear=gear,sequence=text??"",space=space};
        if(!r.Has(WorkshopModule.Fixed))throw new ArgumentException("当前搭建器需要固定砖承载出生点和目标栖点。");
        if(!r.Has(WorkshopModule.White) && !r.Has(WorkshopModule.Bamboo))throw new ArgumentException("请至少允许一种表面：白面或竹面。");
        var plan=new List<WorkshopAction>();
        if(string.IsNullOrWhiteSpace(text))plan=r.Plan(seed);
        else{
            bool goal=false;
            foreach(string part in Regex.Split(text.Trim(),"\\s*(?:→|->|然后|随后|接着|(?<=[起跳挡口射门翔砖台])后|再(?=[用借从踩展穿跳滑收])|，|,|；|;|\\r?\\n)\\s*")){
                string token=Regex.Replace(part.Trim().TrimEnd('。'),"^(?:首先|先)","");
                if(token.Length==0)continue;
                if(Regex.IsMatch(token,"^(?:收鸟(?:并落地(?:结算)?)?|落地(?:结算)?|到达目标台)$")){goal=true;continue;}
                if(goal)throw new ArgumentException("收鸟结算必须放在所有操作之后。");
                int i=Array.FindIndex(Patterns,p=>Regex.IsMatch(token,"^(?:"+p+")$"));
                if(i<0)throw new ArgumentException("无法识别「"+token+"」。可点击下方动作补入顺序，每个动作以 → 分隔。");
                var action=(WorkshopAction)i;
                if(action==WorkshopAction.Spring && token.Contains("砖顶") && (plan.Count==0 || plan[plan.Count-1]!=WorkshopAction.Push))
                    action=WorkshopAction.LooseSpring;
                plan.Add(action);
            }
            if(plan.Count==0){
                if(!goal)throw new ArgumentException("请输入动作，或清空此栏以自动规划。");
                plan.Add(WorkshopAction.Walk);
            }
        }
        foreach(var a in plan){
            string missing=r.Missing(a);
            if(missing!=null)throw new ArgumentException(Names[(int)a]+"需要"+missing+"。请调整白名单或操作顺序。");
        }
        // A requested glide needs a takeoff. Insert the character's ordinary jump when unspecified.
        for(int i=0;i<plan.Count;i++)
            if(plan[i]==WorkshopAction.Glide && (i==0 || (plan[i-1]!=WorkshopAction.Spring && plan[i-1]!=WorkshopAction.LooseSpring && plan[i-1]!=WorkshopAction.Jump)))
                plan.Insert(i++,WorkshopAction.Jump);
        r.actions=plan.ToArray();
        return r;
    }

    string Missing(WorkshopAction a){
        var deps=new List<string>();
        if((a==WorkshopAction.Push || a==WorkshopAction.LooseSpring) && !Has(WorkshopModule.Loose))deps.Add("活动砖");
        var tool=a switch {WorkshopAction.Push or WorkshopAction.Wind=>WorkshopGear.Fan,WorkshopAction.Spring or WorkshopAction.LooseSpring=>WorkshopGear.Mushroom,
            WorkshopAction.Glide=>WorkshopGear.Glider,WorkshopAction.Portal=>WorkshopGear.Gun,_=>WorkshopGear.None};
        if(tool!=WorkshopGear.None && (gear&tool)==0)deps.Add(tool switch {WorkshopGear.Fan=>"风扇",WorkshopGear.Mushroom=>"弹弹菇",WorkshopGear.Glider=>"纸翼",_=>"时空枪"});
        if(tool!=WorkshopGear.None && tool!=WorkshopGear.Glider && !Has(WorkshopModule.White))deps.Add("可安装的白面");
        return deps.Count==0?null:string.Join("、",deps);
    }

    // Bounded forward search over grounded, pushed-brick and airborne states.
    // The seed breaks ties between plans with equally many usable mechanisms.
    List<WorkshopAction> Plan(int seed){
        var random=new System.Random(seed);
        var best=new List<WorkshopAction>();
        double bestScore=-1;
        void Expand(List<WorkshopAction> path){
            if(path.Count>0){
                int mechanisms=path.Count(a=>a!=WorkshopAction.Walk && a!=WorkshopAction.Jump);
                int links=0;
                for(int i=1;i<path.Count;i++)
                    if((path[i-1]==WorkshopAction.Push && path[i]==WorkshopAction.Spring) || path[i]==WorkshopAction.Glide)links++;
                double score=mechanisms+links*.65+random.NextDouble();
                if(score>bestScore){bestScore=score;best=new List<WorkshopAction>(path);}
            }
            if(path.Count==3)return;
            foreach(WorkshopAction a in Enum.GetValues(typeof(WorkshopAction))){
                if(path.Contains(a) || Missing(a)!=null)continue;
                var previous=path.Count==0?WorkshopAction.Walk:path[path.Count-1];
                if(a==WorkshopAction.Glide && previous!=WorkshopAction.Jump && previous!=WorkshopAction.Spring && previous!=WorkshopAction.LooseSpring)continue;
                if(previous==WorkshopAction.Push && a!=WorkshopAction.Spring)continue;
                if(a==WorkshopAction.Walk && path.Count>0)continue;
                path.Add(a);Expand(path);path.RemoveAt(path.Count-1);
            }
        }
        Expand(new List<WorkshopAction>());return best;
    }
}
