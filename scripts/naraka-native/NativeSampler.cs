// Standalone, manually evaluated Unity 2019.4 animation sampler.
// Runs only in a task-owned fixture. Never loads user game assemblies/scripts.
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Reflection;
using System.Text;
using UnityEngine;
using UnityEngine.Animations;
using UnityEngine.Playables;

namespace KivaOracle {
[Serializable] public class Config {
 public string rig_bundle; public string rig_path; public string avatar_sha256;
 public string rig_json; public string hand_mesh_json; public string output; public string report;
 public ClipInput[] clips; public int fps = 60; public bool adaptive; public float time_offset;
}
[Serializable] public class ClipInput {
 public string name; public string path; public string bundle; public string source_sha256;
 public float duration; public bool[] authored_muscle_mask; public string default_left; public string default_right;
}
[Serializable] public class RigInput { public string rig_id; public int motion_root; public NodeInput[] nodes; }
[Serializable] public class NodeInput { public string name; public int parent; public float[] rest; }
[Serializable] public class MeshInput { public MeshPrimitive[] primitives; }
[Serializable] public class MeshPrimitive {
 public string[] bone_names; public float[] positions; public int[] joints; public float[] weights;
 public float[] bindposes_column_major; public int[] triangles;
}
[Serializable] public class ClipCheck {
 public string name; public string source_sha256; public int frames; public int left_missing; public int right_missing;
 public float native_human_roundtrip_degrees; public float native_human_roundtrip_position_m;
 public float authored_body_and_hand_drift_degrees; public float authored_muscle_drift; public float real_vertex_lbs_error_m;
 public float main_reference_local_degrees; public float main_reference_local_position_m; public float default_reference_local_degrees; public float default_reference_local_position_m; public float repeat_degrees; public float repeat_position_m;
 public float[] initial_pelvis; public float[] middle_pelvis; public float[] final_pelvis;
 public string composition; public string left_layer; public string right_layer;
 public int adaptive_evaluations; public int precision_boundary_intervals;
 public float interpolation_midpoint_degrees; public float interpolation_midpoint_position_m;
 public float interpolation_midpoint_world_joint_m;
 public float adaptive_replay_degrees; public float adaptive_replay_position_m;
 public List<CutCheck> precision_cuts = new List<CutCheck>();
}
[Serializable] public class CutBone {public string bone;public float[] left_local;public float[] right_local;}
[Serializable] public class CutCheck {public float left_time;public float right_time;public string reason="Adjacent f32 times; both sides retained as original native samples";public List<CutBone> changed_bones=new List<CutBone>();}
[Serializable] public class Report {
 public string format = "KIVA_NATIVE_SAMPLER_REPORT_1";
 public string unity_version; public string source_unity_version = "2019.4.41f2";
 public int capture_fps;
 public bool adaptive;
 public float time_offset;
 public string adaptive_rule = "Native quarter/mid/three-quarter refinement: all local rotations <=0.075 degrees, local translations <=0.25mm, all source joint world positions <=0.25mm. Stop unresolved cuts only at adjacent f32 times and retain both original native values. Measured samples are not a continuous mathematical error bound.";
 public string avatar_source_sha256; public string original_root_name; public string avatar_name;
 public bool avatar_is_human; public bool foot_ik = false;
 public string root_motion = "Animator.applyRootMotion=false; raw evaluated local transforms retained";
 public string layer_rule = "Kiva adaptation: only complete unauthored four-DoF finger chains receive three local TRS from a separately native-evaluated original default clip; main body/authored finger chains remain untouched; default time follows main elapsed modulo default length. No original controller weight/order claim. HumanPose Get/Set is diagnostic only.";
 public ClipCheck[] clips; public bool passed; public string failure; public bool real_skin_checked; public int real_skin_vertices;
}

sealed class Actor : IDisposable {
 public GameObject go; public Animator animator; public Transform[] bones; public RigInput rig;
 public PlayableGraph graph; public AnimationClipPlayable playable;
 public HumanPoseHandler human; public HumanPose pose; Avatar avatar;
 public Actor(GameObject source, Avatar avatar, RigInput input, AnimationClip clip) {
  rig=input; this.avatar=avatar; go=UnityEngine.Object.Instantiate(source); go.name="KivaNativeActor";
  go.transform.position=Vector3.zero; go.transform.rotation=Quaternion.identity; go.transform.localScale=Vector3.one;
  animator=go.GetComponent<Animator>(); if(animator==null)animator=go.AddComponent<Animator>();
  animator.runtimeAnimatorController=null; animator.avatar=avatar;
  animator.applyRootMotion=false; animator.cullingMode=AnimatorCullingMode.AlwaysAnimate;
  animator.enabled=true;
  Dictionary<string,Transform> lookup=new Dictionary<string,Transform>();
  foreach(Transform t in go.GetComponentsInChildren<Transform>(true)) {
   if(t==go.transform)continue;
   if(lookup.ContainsKey(t.name))throw new Exception("Ambiguous original bone "+t.name);
   lookup.Add(t.name,t);
  }
  bones=new Transform[input.nodes.Length]; bones[0]=go.transform;
  for(int i=1;i<bones.Length;i++) {
   if(!lookup.TryGetValue(input.nodes[i].name,out bones[i]))throw new Exception("Missing original bone "+input.nodes[i].name);
   if(bones[i].parent!=bones[input.nodes[i].parent])throw new Exception("Original hierarchy mismatch "+input.nodes[i].name);
  }
  Reset();
  graph=PlayableGraph.Create("KivaNativeManual"); graph.SetTimeUpdateMode(DirectorUpdateMode.Manual);
  playable=AnimationClipPlayable.Create(graph,clip); playable.SetApplyFootIK(false); playable.SetApplyPlayableIK(false);
  // Retain an explicitly seekable final frame even for source looping clips.
  object boxed=playable;
  MethodInfo loop=typeof(AnimationClipPlayable).GetMethod("SetOverrideLoopTime",BindingFlags.Instance|BindingFlags.NonPublic);
  MethodInfo value=typeof(AnimationClipPlayable).GetMethod("SetLoopTime",BindingFlags.Instance|BindingFlags.NonPublic);
  if(loop==null||value==null)throw new Exception("Native loop override API unavailable");
  loop.Invoke(boxed,new object[]{true}); value.Invoke(boxed,new object[]{false});
  playable=(AnimationClipPlayable)boxed;
  playable.SetSpeed(0);
  AnimationPlayableOutput output=AnimationPlayableOutput.Create(graph,"KivaNativePose",animator);
  output.SetSourcePlayable(playable); graph.Play();
 }
 public void Reset() {
  for(int i=0;i<bones.Length;i++) {
   float[] r=rig.nodes[i].rest;
   bones[i].localPosition=new Vector3(r[0],r[1],r[2]);
   bones[i].localRotation=new Quaternion(r[3],r[4],r[5],r[6]);
   bones[i].localScale=new Vector3(r[7],r[8],r[9]);
  }
 }
 public void Sample(float time) {
  Reset(); playable.SetTime((double)time); graph.Evaluate(0);
  foreach(Transform t in bones) Bootstrap.CheckFinite(t.localPosition,t.localRotation,t.localScale);
 }
 public HumanPose Human() { if(human==null)human=new HumanPoseHandler(avatar,go.transform); human.GetHumanPose(ref pose); return pose; }
 public void SetHuman(ref HumanPose p) { human.SetHumanPose(ref p); }
 public void Dispose() {
  if(graph.IsValid())graph.Destroy();
  if(human!=null)human.Dispose();
  if(go!=null)UnityEngine.Object.DestroyImmediate(go);
 }
}

sealed class NativeFrame {
 public float time; public Vector3[] p,s; public Quaternion[] q; public Matrix4x4[] world;
 public NativeFrame(float t,Actor actor){
  time=t;int n=actor.bones.Length;p=new Vector3[n];s=new Vector3[n];q=new Quaternion[n];world=new Matrix4x4[n];
  for(int i=0;i<n;i++){Transform b=actor.bones[i];p[i]=b.localPosition;s[i]=b.localScale;q[i]=b.localRotation;world[i]=b.localToWorldMatrix;}
 }
 public void Write(BinaryWriter writer){
  writer.Write(time);for(int i=0;i<p.Length;i++){
   writer.Write(p[i].x);writer.Write(p[i].y);writer.Write(p[i].z);
   writer.Write(q[i].x);writer.Write(q[i].y);writer.Write(q[i].z);writer.Write(q[i].w);
   writer.Write(s[i].x);writer.Write(s[i].y);writer.Write(s[i].z);
   for(int k=0;k<16;k++)writer.Write(world[i][k]);
  }
 }
}

sealed class NativeTimeline {
 Actor actor,left,right;AnimationClip leftClip,rightClip;bool[] mask;bool fill;
 ClipCheck check;RigInput rig;Dictionary<float,NativeFrame> cache=new Dictionary<float,NativeFrame>();
 public NativeTimeline(Actor a,Actor l,Actor r,AnimationClip lc,AnimationClip rc,bool[] m,bool f,ClipCheck c){actor=a;left=l;right=r;leftClip=lc;rightClip=rc;mask=m;fill=f;check=c;rig=a.rig;}
 public NativeFrame Sample(float time){
  NativeFrame frame;if(cache.TryGetValue(time,out frame))return frame;
  if(cache.Count>=36001)throw new Exception("Adaptive evaluation budget exceeded "+check.name);
  actor.Sample(time);if(fill){left.Sample(time%leftClip.length);right.Sample(time%rightClip.length);Bootstrap.FillMissingChains(actor,left,right,mask,check);}
  frame=new NativeFrame(time,actor);cache.Add(time,frame);return frame;
 }
 internal static double Angle(Quaternion a,Quaternion b){
  double dot=(double)a.x*b.x+(double)a.y*b.y+(double)a.z*b.z+(double)a.w*b.w;
  double aa=(double)a.x*a.x+(double)a.y*a.y+(double)a.z*a.z+(double)a.w*a.w;
  double bb=(double)b.x*b.x+(double)b.y*b.y+(double)b.z*b.z+(double)b.w*b.w;
  return 2*Math.Acos(Math.Min(1,Math.Abs(dot/Math.Sqrt(aa*bb))))*180/Math.PI;
 }
 public void Refine(NativeFrame a,NativeFrame b,List<NativeFrame> output,int depth){
  float time=(float)(((double)a.time+b.time)*.5);
  if(time<=a.time||time>=b.time){
   CutCheck cut=new CutCheck();cut.left_time=a.time;cut.right_time=b.time;
   for(int i=0;i<rig.nodes.Length;i++)if(Angle(a.q[i],b.q[i])>.075||(a.p[i]-b.p[i]).magnitude>.00025f){
    CutBone bone=new CutBone();bone.bone=rig.nodes[i].name;bone.left_local=Values(a,i);bone.right_local=Values(b,i);cut.changed_bones.Add(bone);
   }
   check.precision_cuts.Add(cut);check.precision_boundary_intervals++;output.Add(b);return;
  }
  NativeFrame middle=Sample(time);float angular=0,position=0,worldError=0;
  foreach(double fraction in new double[]{.25,.5,.75}){
   float probe=(float)((double)a.time+((double)b.time-a.time)*fraction);
   if(probe<=a.time||probe>=b.time)continue;
   NativeFrame reference=Sample(probe);float u=(probe-a.time)/(b.time-a.time);Matrix4x4[] world=new Matrix4x4[rig.nodes.Length];
   for(int i=0;i<world.Length;i++){
    Vector3 p=Vector3.LerpUnclamped(a.p[i],b.p[i],u),s=Vector3.LerpUnclamped(a.s[i],b.s[i],u);
    Quaternion q=Quaternion.SlerpUnclamped(a.q[i],b.q[i],u);
    angular=Mathf.Max(angular,(float)Angle(q,reference.q[i]));position=Mathf.Max(position,(p-reference.p[i]).magnitude);
    Matrix4x4 local=Matrix4x4.TRS(p,q,s);int parent=rig.nodes[i].parent;world[i]=parent<0?local:world[parent]*local;
    Vector3 delta=(Vector3)world[i].GetColumn(3)-(Vector3)reference.world[i].GetColumn(3);worldError=Mathf.Max(worldError,delta.magnitude);
   }
  }
  if(angular<=.075f&&position<=.00025f&&worldError<=.00025f){
   check.interpolation_midpoint_degrees=Mathf.Max(check.interpolation_midpoint_degrees,angular);
   check.interpolation_midpoint_position_m=Mathf.Max(check.interpolation_midpoint_position_m,position);
   check.interpolation_midpoint_world_joint_m=Mathf.Max(check.interpolation_midpoint_world_joint_m,worldError);
   output.Add(b);return;
  }
  if(depth>=30)throw new Exception("Adaptive depth exhausted before adjacent f32 times "+check.name);
  Refine(a,middle,output,depth+1);Refine(middle,b,output,depth+1);
 }
 static float[] Values(NativeFrame f,int i){return new float[]{f.p[i].x,f.p[i].y,f.p[i].z,f.q[i].x,f.q[i].y,f.q[i].z,f.q[i].w,f.s[i].x,f.s[i].y,f.s[i].z};}
 public List<NativeFrame> Build(List<float> times,bool adaptive){
  List<NativeFrame> frames=new List<NativeFrame>();frames.Add(Sample(times[0]));
  for(int i=1;i<times.Count;i++){
   NativeFrame end=Sample(times[i]);if(adaptive)Refine(frames[frames.Count-1],end,frames,0);else frames.Add(end);
  }
  check.adaptive_evaluations=cache.Count;
  return frames;
 }
}

sealed class SkinCheck : IDisposable {
 GameObject go; SkinnedMeshRenderer renderer; Mesh source,baked; MeshPrimitive data;
 Transform[] bones; Matrix4x4[] inverse;
 public SkinCheck(MeshPrimitive p, Actor actor) {
  data=p; go=new GameObject("KivaRealSourceHandLbs"); renderer=go.AddComponent<SkinnedMeshRenderer>();
  source=new Mesh(); source.name="Actual source body hand subset";
  Vector3[] vertices=new Vector3[p.positions.Length/3]; BoneWeight[] weights=new BoneWeight[vertices.Length];
  for(int i=0;i<vertices.Length;i++) {
   vertices[i]=new Vector3(p.positions[3*i],p.positions[3*i+1],p.positions[3*i+2]);
   BoneWeight w=new BoneWeight(); w.boneIndex0=p.joints[4*i];w.boneIndex1=p.joints[4*i+1];w.boneIndex2=p.joints[4*i+2];w.boneIndex3=p.joints[4*i+3];
   w.weight0=p.weights[4*i];w.weight1=p.weights[4*i+1];w.weight2=p.weights[4*i+2];w.weight3=p.weights[4*i+3];weights[i]=w;
  }
  bones=new Transform[p.bone_names.Length]; inverse=new Matrix4x4[bones.Length];
  for(int i=0;i<bones.Length;i++) {
   for(int j=0;j<actor.rig.nodes.Length;j++)if(actor.rig.nodes[j].name==p.bone_names[i]){bones[i]=actor.bones[j];break;}
   if(bones[i]==null)throw new Exception("Real source skin bone missing "+p.bone_names[i]);
   Matrix4x4 m=new Matrix4x4();for(int k=0;k<16;k++)m[k]=p.bindposes_column_major[16*i+k];inverse[i]=m;
  }
  source.vertices=vertices;source.boneWeights=weights;source.bindposes=inverse;source.triangles=p.triangles;
  renderer.sharedMesh=source;renderer.bones=bones;renderer.updateWhenOffscreen=true;renderer.quality=SkinQuality.Bone4;
  baked=new Mesh();
 }
 public float Measure() {
  renderer.BakeMesh(baked);Vector3[] actual=baked.vertices;float maximum=0;
  if(actual.Length!=data.positions.Length/3)throw new Exception("Native BakeMesh vertex count changed");
  for(int i=0;i<actual.Length;i++) {
   Vector3 point=new Vector3(data.positions[3*i],data.positions[3*i+1],data.positions[3*i+2]);Vector3 expected=Vector3.zero;
   for(int k=0;k<4;k++){
    int j=data.joints[4*i+k];float w=data.weights[4*i+k];
    expected+=(bones[j].localToWorldMatrix*inverse[j]).MultiplyPoint3x4(point)*w;
   }
   maximum=Mathf.Max(maximum,(actual[i]-expected).magnitude);
  }
  return maximum;
 }
 public void Dispose(){UnityEngine.Object.DestroyImmediate(go);UnityEngine.Object.DestroyImmediate(source);UnityEngine.Object.DestroyImmediate(baked);}
}

public static class Bootstrap {
 static bool running;
 static string status;
 static void Note(string value){Debug.Log("KIVA_NATIVE "+value);File.AppendAllText(status,value+"\n");}
 public static void CheckFinite(Vector3 t,Quaternion q,Vector3 s) {
  float[] a={t.x,t.y,t.z,q.x,q.y,q.z,q.w,s.x,s.y,s.z};foreach(float x in a)if(float.IsNaN(x)||float.IsInfinity(x))throw new Exception("Native pose nonfinite");
 }
 static void Text(BinaryWriter w,string value){byte[] b=Encoding.UTF8.GetBytes(value);w.Write(b.Length);w.Write(b);}
 static void Pose(BinaryWriter w,Transform t){Vector3 p=t.localPosition;Quaternion q=t.localRotation;Vector3 s=t.localScale;w.Write(p.x);w.Write(p.y);w.Write(p.z);w.Write(q.x);w.Write(q.y);w.Write(q.z);w.Write(q.w);w.Write(s.x);w.Write(s.y);w.Write(s.z);}
 static float[] Pos(Transform t){Vector3 p=t.position;return new float[]{p.x,p.y,p.z};}
 static bool Finger(string name){return name.IndexOf(" Finger",StringComparison.Ordinal)>=0;}
 static float[] Local(Transform t){Vector3 p=t.localPosition,s=t.localScale;Quaternion q=t.localRotation;return new float[]{p.x,p.y,p.z,q.x,q.y,q.z,q.w,s.x,s.y,s.z};}
 static void Compare(Transform a,Transform b,ref float angle,ref float position){angle=Mathf.Max(angle,Quaternion.Angle(a.localRotation,b.localRotation));position=Mathf.Max(position,(a.localPosition-b.localPosition).magnitude);}
 internal static HashSet<int> FillMissingChains(Actor main,Actor left,Actor right,bool[] mask,ClipCheck check){
  HashSet<int> copied=new HashSet<int>();
  for(int side=0;side<2;side++)for(int finger=0;finger<5;finger++){
   int start=55+side*20+finger*4,authored=0;for(int d=0;d<4;d++)if(mask[start+d])authored++;
   if(authored==4)continue;if(authored!=0)throw new Exception("Partially authored finger DoF cannot be guessed");
   Actor source=side==0?left:right;
   for(int segment=0;segment<3;segment++){
    string name="gMan "+(side==0?"L":"R")+" Finger"+finger+(segment==0?"":segment.ToString(CultureInfo.InvariantCulture));
    int index=-1;for(int i=0;i<main.rig.nodes.Length;i++)if(main.rig.nodes[i].name==name){index=i;break;}
    if(index<0)throw new Exception("Original default finger bone missing "+name);
    main.bones[index].localPosition=source.bones[index].localPosition;main.bones[index].localRotation=source.bones[index].localRotation;main.bones[index].localScale=source.bones[index].localScale;
    Compare(main.bones[index],source.bones[index],ref check.default_reference_local_degrees,ref check.default_reference_local_position_m);copied.Add(index);
   }
  }
  return copied;
 }
 static void RoundTrip(Actor actor,ClipCheck check,bool[] authored,bool onlyUnchanged) {
  Quaternion[] q=new Quaternion[actor.bones.Length];Vector3[] p=new Vector3[q.Length],local=new Vector3[q.Length],scale=new Vector3[q.Length];
  for(int i=0;i<q.Length;i++){q[i]=actor.bones[i].localRotation;p[i]=actor.bones[i].position;local[i]=actor.bones[i].localPosition;scale[i]=actor.bones[i].localScale;}
  HumanPose pose=actor.Human();actor.SetHuman(ref pose);
  for(int i=0;i<q.Length;i++) {
   float angle=Quaternion.Angle(q[i],actor.bones[i].localRotation);
   check.native_human_roundtrip_degrees=Mathf.Max(check.native_human_roundtrip_degrees,angle);
   check.native_human_roundtrip_position_m=Mathf.Max(check.native_human_roundtrip_position_m,(p[i]-actor.bones[i].position).magnitude);
  }
  for(int i=0;i<q.Length;i++){actor.bones[i].localPosition=local[i];actor.bones[i].localRotation=q[i];actor.bones[i].localScale=scale[i];}
 }
 static AnimationClip LoadClip(ClipInput input,Dictionary<string,AssetBundle> bundles) {
  AssetBundle b;if(!bundles.TryGetValue(input.bundle,out b)){b=AssetBundle.LoadFromFile(input.bundle);if(b==null)throw new Exception("Native clip bundle load failed "+input.bundle);bundles.Add(input.bundle,b);}
  AnimationClip clip=b.LoadAsset<AnimationClip>(input.path);
  if(clip==null||clip.name!=input.name)throw new Exception("Native original clip unavailable "+input.name);
  if(Mathf.Abs(clip.length-input.duration)>.001f)throw new Exception("Native clip duration mismatch "+input.name+" "+clip.length+" != "+input.duration);
  return clip;
 }
 [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.BeforeSceneLoad)]
 public static void Run() {
  if(running)return;running=true;status=Path.Combine(Application.dataPath,"probe-status.txt");File.WriteAllText(status,"");
  Report report=new Report();report.unity_version=Application.unityVersion;
  string reportPath=Path.Combine(Application.dataPath,"native-report.json");
  try {
   Config cfg=JsonUtility.FromJson<Config>(File.ReadAllText(Path.Combine(Application.dataPath,"KivaOracle.input.json")));
   reportPath=cfg.report;report.avatar_source_sha256=cfg.avatar_sha256;
   if(cfg.fps!=60&&cfg.fps!=120&&cfg.fps!=137)throw new Exception("Unsupported capture rate");
   if(float.IsNaN(cfg.time_offset)||float.IsInfinity(cfg.time_offset)||cfg.time_offset<0||cfg.time_offset>=1f/cfg.fps)throw new Exception("Invalid reference time offset");
   report.capture_fps=cfg.fps;
   report.adaptive=cfg.adaptive;
   report.time_offset=cfg.time_offset;
   RigInput rig=JsonUtility.FromJson<RigInput>(File.ReadAllText(cfg.rig_json));
   QualitySettings.skinWeights=SkinWeights.FourBones;
   Note("start "+Application.unityVersion+" clips="+cfg.clips.Length);
   AssetBundle rigBundle=AssetBundle.LoadFromFile(cfg.rig_bundle);if(rigBundle==null)throw new Exception("Native rig bundle load failed");
   UnityEngine.Object[] rigAssets=rigBundle.LoadAssetWithSubAssets(cfg.rig_path);
   GameObject model=null;Avatar avatar=null;
   foreach(UnityEngine.Object obj in rigAssets){Note("rig subasset "+obj.GetType().Name+" "+obj.name);if(obj is GameObject)model=(GameObject)obj;if(obj is Avatar)avatar=(Avatar)obj;}
   if(model==null)model=rigBundle.LoadAsset<GameObject>(cfg.rig_path);
   if(avatar==null)avatar=rigBundle.LoadAsset<Avatar>(cfg.rig_path);
   if(model==null||avatar==null)throw new Exception("Original rig GameObject/Avatar missing from selected FBX subassets");
   report.original_root_name=model.name;report.avatar_name=avatar.name;report.avatar_is_human=avatar.isHuman;
   if(!avatar.isValid||!avatar.isHuman)throw new Exception("Original Avatar invalid/nonhuman");
   Note("rig valid "+model.name+" human="+avatar.isHuman);
   Dictionary<string,AssetBundle> bundles=new Dictionary<string,AssetBundle>();
   Dictionary<string,ClipInput> clipInputs=new Dictionary<string,ClipInput>();foreach(ClipInput x in cfg.clips)clipInputs.Add(x.name,x);
   MeshInput mesh=String.IsNullOrEmpty(cfg.hand_mesh_json)?null:JsonUtility.FromJson<MeshInput>(File.ReadAllText(cfg.hand_mesh_json));
   report.real_skin_checked=mesh!=null&&mesh.primitives.Length>0;
   report.real_skin_vertices=report.real_skin_checked?mesh.primitives[0].positions.Length/3:0;
   List<ClipCheck> checks=new List<ClipCheck>();
   using(BinaryWriter writer=new BinaryWriter(File.Open(cfg.output,FileMode.Create))) {
    writer.Write(Encoding.ASCII.GetBytes("KIVANAT2"));Text(writer,Application.unityVersion);Text(writer,rig.rig_id);writer.Write(rig.nodes.Length);writer.Write(rig.motion_root);
    foreach(NodeInput node in rig.nodes){Text(writer,node.name);writer.Write(node.parent);foreach(float x in node.rest)writer.Write(x);}
    writer.Write(cfg.clips.Length);
    foreach(ClipInput input in cfg.clips) {
     Note("sample "+input.name);
     AnimationClip clip=LoadClip(input,bundles);ClipCheck check=new ClipCheck();check.name=input.name;check.source_sha256=input.source_sha256;
     bool[] mask=input.authored_muscle_mask;
     for(int j=55;j<75;j++)if(!mask[j])check.left_missing++;
     for(int j=75;j<95;j++)if(!mask[j])check.right_missing++;
     bool fill=check.left_missing+check.right_missing>0 && input.name.IndexOf("finger",StringComparison.Ordinal)<0;
     check.composition=fill?report.layer_rule:"Original main clip native evaluation; no hand overlay";
     AnimationClip leftClip=null,rightClip=null;
     if(fill){
      ClipInput li,ri;if(!clipInputs.TryGetValue(input.default_left,out li)||!clipInputs.TryGetValue(input.default_right,out ri))throw new Exception("Missing declared original default hand assets "+input.name);
      leftClip=LoadClip(li,bundles);rightClip=LoadClip(ri,bundles);check.left_layer=check.left_missing>0?li.name:null;check.right_layer=check.right_missing>0?ri.name:null;
     }
     using(Actor actor=new Actor(model,avatar,rig,clip))
     using(Actor left=fill?new Actor(model,avatar,rig,leftClip):null)
     using(Actor right=fill?new Actor(model,avatar,rig,rightClip):null) {
      List<float> times=new List<float>();times.Add(0);int n=(int)Math.Ceiling(input.duration*cfg.fps);
      for(int k=0;k<n;k++){float t=(float)((double)k/cfg.fps+cfg.time_offset);if(t<input.duration&&t>times[times.Count-1])times.Add(t);}
      if(times.Count==0||input.duration>times[times.Count-1])times.Add(input.duration);
      NativeTimeline timeline=new NativeTimeline(actor,left,right,leftClip,rightClip,mask,fill,check);
      List<NativeFrame> frames=timeline.Build(times,cfg.adaptive);
      check.frames=frames.Count;Text(writer,input.name);Text(writer,input.source_sha256);writer.Write(input.duration);writer.Write(frames.Count);
      SkinCheck skin=mesh==null||mesh.primitives.Length==0?null:new SkinCheck(mesh.primitives[0],actor);
      try {
       for(int f=0;f<frames.Count;f++) {
        float t=frames[f].time;bool inspect=f==0||f==frames.Count/2||f==frames.Count-1;
        // Replay each emitted native instant independently of the adaptive
        // traversal before writing it. Never export the interpolated estimate.
        actor.Sample(t);
        HashSet<int> copied=new HashSet<int>();
        if(fill){left.Sample(t%leftClip.length);right.Sample(t%rightClip.length);copied=FillMissingChains(actor,left,right,mask,check);}
        for(int j=0;j<actor.bones.Length;j++){
         check.adaptive_replay_degrees=Mathf.Max(check.adaptive_replay_degrees,(float)NativeTimeline.Angle(frames[f].q[j],actor.bones[j].localRotation));
         check.adaptive_replay_position_m=Mathf.Max(check.adaptive_replay_position_m,(frames[f].p[j]-actor.bones[j].localPosition).magnitude);
        }
        if(inspect){
         // Fresh native original graph is the reference. No inverse HumanPose
         // call ever touches the production actor or its Animator cache.
         using(Actor reference=new Actor(model,avatar,rig,clip)){
          reference.Sample(t);
          for(int j=0;j<actor.bones.Length;j++)if(!copied.Contains(j))Compare(actor.bones[j],reference.bones[j],ref check.main_reference_local_degrees,ref check.main_reference_local_position_m);
         }
         using(Actor diagnostic=new Actor(model,avatar,rig,clip)){
          diagnostic.Sample(t);RoundTrip(diagnostic,check,mask,false);
         }
         using(Actor repeat=new Actor(model,avatar,rig,clip)){
          repeat.Sample(t);
          if(fill)FillMissingChains(repeat,left,right,mask,new ClipCheck());
          for(int j=0;j<actor.bones.Length;j++)Compare(actor.bones[j],repeat.bones[j],ref check.repeat_degrees,ref check.repeat_position_m);
         }
        }
        if(inspect&&skin!=null)check.real_vertex_lbs_error_m=Mathf.Max(check.real_vertex_lbs_error_m,skin.Measure());
        if(f==0)check.initial_pelvis=Pos(actor.bones[5]);if(f==frames.Count/2)check.middle_pelvis=Pos(actor.bones[5]);if(f==frames.Count-1)check.final_pelvis=Pos(actor.bones[5]);
        writer.Write(t);foreach(Transform bone in actor.bones){Pose(writer,bone);Matrix4x4 matrix=bone.localToWorldMatrix;for(int k=0;k<16;k++)writer.Write(matrix[k]);}
       }
      } finally {if(skin!=null)skin.Dispose();}
     }
     checks.Add(check);Note("done "+input.name+" frames="+check.frames+" roundtrip="+check.native_human_roundtrip_degrees+" lbs="+check.real_vertex_lbs_error_m);
    }
   }
   report.clips=checks.ToArray();report.passed=true;
   foreach(ClipCheck c in report.clips)if(c.main_reference_local_degrees>.1f||c.main_reference_local_position_m>.001f||c.default_reference_local_degrees>.1f||c.default_reference_local_position_m>.001f||c.repeat_degrees>.1f||c.repeat_position_m>.001f||c.real_vertex_lbs_error_m>.001f||c.adaptive_replay_degrees>.1f||c.adaptive_replay_position_m>.001f)report.passed=false;
   File.WriteAllText(reportPath,JsonUtility.ToJson(report,true));Note("finished passed="+report.passed);
   Environment.Exit(report.passed?0:2);
  }catch(Exception ex){report.passed=false;report.failure=ex.ToString();File.WriteAllText(reportPath,JsonUtility.ToJson(report,true));Note("FAILED "+ex);Environment.Exit(2);}
 }
}
}
