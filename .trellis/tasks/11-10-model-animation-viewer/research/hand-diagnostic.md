# Hand diagnostic: default Li Huowang pose

Date: 2026-11-11. The diagnostic sections below record the pre-fix baseline. The user confirmed the screenshot is the default paused state (Li Huowang + Fulangmao, game:idle, frame 0), then explicitly authorized confirmed shared-runtime fixes. The final section records implemented behavior and targeted verification.

## Reproduction and what the active input actually is

The ignored dump probe was removed from production tests after diagnosis. A reusable focused patch and source block are retained here; apply the patch before running the explicit diagnostic, then reverse that same patch to remove only the probe. Current implementation deliberately produces different post-fix metrics.

Explicit diagnostic invocation:

```sh
cargo test --locked hand_diagnostic_default_state -- --ignored --nocapture
```

The probe records the real bundled assets, `RigPose::new`, shared avatar driver computation and per-part skin palettes. Output: `idle`, CC0 `Sword_Idle`, `source=None`. The default pose therefore does not run the extracted Yanyun source animation layer. `poses::idle_with_trace` samples CC0 and applies `hold_guard`; the melee path solves only the right arm in the CC0 rig before transferring motion to the imported body.

## Measured reference mismatch

Li's imported current reference has angled/downward arms. CC0 structural rest has nearly horizontal arms. The own-rest segment vectors differ from CC0 rest by:

| Segment | Right | Left |
|---|---:|---:|
| Upper arm → elbow | 48.794° | 48.794° |
| Elbow → wrist | 56.854° | 56.854° |

The current `driver_pose` applies the CC0 global rest-relative rotation to Li's own anchor offsets. Length is retained, but this rest direction difference is retained too. Default right skin wrist is approximately `(-0.2194,1.0612,-0.0986)` while the CC0 wrist is `(0.0385,1.0510,-0.0792)`. Left skin wrist is `(-0.0289,0.7299,0.0001)` versus CC0 `(-0.4061,0.9336,-0.3772)`. Different body lengths alone do not explain these large direction changes.

The target arm segment-length invariant passes. This rules out literal bone-length growth in the structural chain; it does not prove the pose is anatomically correct or that a cuff cannot self-intersect.

A proposed regression for the required retarget behavior is to preserve the sampled anatomical arm directions while preserving each target segment's own length. It must be agreed as the shared-game retarget contract before altering runtime behavior; it is stronger than the existing same-helper/palette consistency tests.

## Finger motion is present in source, lost in Naraka transfer

Li has 30 distinct finger bones (15 each hand); every one is assigned the left/right wrist driver (10/14). Non-Choulingke `SourceBone.source` is None, so Yanyun finger channels do not directly drive Li's fingers. Both CC0 and source retargeting currently keep the original imported finger arrangement rigid relative to the hand.

CPU skinning of 868 actual vertices exclusively weighted to the right-hand driver confirms the same hand-local geometry in asset reference, raw CC0, default game idle and `skill_ylk_1` at 1.35s. Maximum deviation from a single rigid hand transformation is under `4.86e-7m`. Approximate local extent remains `9.65 × 10.05 × 10.42 cm` in all cases.

The imported fingers are already curled, not simply open: measured right middle-joint bends are thumb18.9°, index70.9°, middle74.9°, ring62.5°, little50.3°. Therefore frozen finger motion is a real limitation but cannot alone explain every default wrist/palm orientation problem.

The actual decoded `skill_ylk_1` has substantial finger rotation. Sampling 0/.25/.5/1/1.35/1.8/2.5s gives these maximum wrist-local rotations from source rest:

| Finger | Right | Left |
|---|---:|---:|
| Thumb | 25.3° | 50.7° |
| Index | 36.7° | 33.8° |
| Middle | 66.5° | 41.8° |
| Ring | 86.6° | 52.5° |
| Little | 91.6° | 60.3° |

Wrist-local base translations remain under `7e-7m`. This establishes a meaningful source-finger motion input rather than blaming absent/invalid decoded source channels.

Concrete red regression invocation:

```sh
cargo test --locked naraka_source_finger_rotation_reaches_actual_finger_joint -- --nocapture
```

It chooses the real clip sample with source right little-finger rotation >1rad, feeds that SourcePose through Li's actual joint palette, and requires a nonzero corresponding target finger rotation relative to its wrist. The pre-fix implementation rotates the whole hand but not the finger relative to it. This tests the newly required finger transfer behavior, not a vague visual quality score.

## Limits and artifact handling

Finite matrices, preserved bone lengths and equality between game/viewer are not correctness oracles for anatomical pose. The visual comparison remains the user's actual symptom and the root session's headed-browser captures. Raw temporary probe logs are under `/tmp/kiva-hand-diagnostic.log` and `/tmp/kiva-hand-finger-red.log`; test-only patches are to be saved here if runtime changes are not authorized. That paragraph describes the initial diagnostic stage; the shared runtime was subsequently fixed after explicit authorization as follows.


## Implemented shared fixes and targeted verification

The default-pose fault had three independently measured causes:

1. Rest direction transfer retained ~49°/57° A/T-pose differences in the actual arm chain. A new regression failed at 48.794° before the fix. Shared CC0/source retargeting now applies the minimum swing needed to aim each own-length segment along the actual sampled anatomical direction; explicit companion grip IK still runs afterward.
2. Thirty real Naraka finger joints were reduced to wrist drivers, dropping decoded source curl. A real `skill_ylk_1` right little-finger input rotating91.63° produced exactly0° target-relative response before the fix. Explicit semantic chains now retain all three weighted joints; source Nub supplies the terminal direction only. Current source anatomical frames are expressed relative to the current palm and converted through each target native rest-axis calibration. Local rotations blend before own-length FK, preventing source fade from shortening fingers. Li's partial thumb palette uses compatible named rest data from the actual other part, not invented joints or another character's palette indices.
3. The actual default palm length and previous attachment offset differed by63.2° (dot0.4513), and the shaft by30.1° from the finger bank. A geometry regression failed independently of old wrist-helper equality. The actual target hand is now calibrated to the declared CC0 grip frame or the current source palm. Shared attachment frames follow actual palette wrist/MCP geometry, +Y goes into the palm, +Z/head points toward index/thumb, scale is exactlyONE; the existing0.055m offset remains. Default weapon root changed less than1e-6m while the hand geometry was corrected into that grip. This fixes the hand and attachment together rather than moving a viewer-only weapon.

Original Choulingke full-source joint matrices and all asset rest/IBM/geometry remain unchanged. The existing Choulingke global-joint source crossfade is not a fully rigid wrist transform at intermediate weights; current wrist/MCP palette geometry therefore defines the semantic grip during that fade. This corrects attachment tracking, not the underlying original whole-body blend algorithm.

Targeted final verification:19 avatar tests and3 companion tests pass, zero ignored. The new checks include actual palette arm direction and own lengths for default/rawCC0Walk/realSource; all fingers/both hands/both Naraka bodies at three real source times, including thumb and terminal native+X direction in the current palm; every actual weighted palette finger binding including partial thumb parents; weights0/.25/.5/.75/1 own lengths and weight-zero fallback; actual LBS vertex shape response; exact imported reference preservation; signed palm/head axes through all three appearances/source weights; and original Choulingke named full-source matrices. Full quality gates and browser comparison are owned by the main session/checker, not claimed by these targeted results.

The temporary ignored dump was removed. Meaningful arm/finger/grip regressions remain in `avatar.rs` and `avatar/hand_tests.rs`; runtime support is private `avatar/hand.rs`. No source animations, packed assets, inverse binds, importers, physics or viewer-only offsets were changed.

Independent target twist-roll distribution remains unrestored. Actual35 source clips contain no independent named twist-category tracks; this task does not invent an original controller or add a guessed all-quaternion twist slerp. Parent-following behavior is retained; further distribution work requires a demonstrated remaining skin defect.
