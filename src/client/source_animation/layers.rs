//! Verified composition metadata from the native bake report.
//! Keep in sync with assets/naraka/animations-manifest.json when adding clips.
//! These are Kiva missing-chain supplements, not original controller weights.
pub(super) fn finger_layers(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "female_unarmed_finger_default_left_01"
        | "female_unarmed_finger_default_right_01"
        | "male_bow_finger_default_left_01"
        | "male_bow_finger_default_right_01"
        | "male_hengdao_attack_light_01"
        | "male_hengdao_attack_light_02"
        | "male_hengdao_attack_light_03"
        | "male_hengdao_attack_shock_light_01"
        | "male_hengdao_dodge_back_01"
        | "male_hengdao_dodge_front_01"
        | "male_hengdao_dodge_left_01"
        | "male_hengdao_dodge_right_01"
        | "male_hengdao_finger_left_01"
        | "male_hengdao_finger_right_01"
        | "male_hengdao_jump_down_to_land_01"
        | "male_hengdao_jump_inplace_up_01"
        | "male_hengdao_jump_inplace_up_02"
        | "male_hengdao_promenade_01"
        | "male_hengdao_run_01"
        | "male_hengdao_shock_01"
        | "male_hengdao_shock_04"
        | "male_unarmed_finger_default_left_01"
        | "male_unarmed_finger_default_right_01" => ("", ""),
        "male_hurt_stand_light_front_01" => ("", "male_hengdao_finger_right_01"),
        "female_sniper_attack_01" => ("female_unarmed_finger_default_left_01", ""),
        "female_sniper_ads_attack_01"
        | "female_sniper_ads_idle_01"
        | "female_sniper_attack_01_recover"
        | "female_sniper_idle_01"
        | "female_sniper_reload_01" => (
            "female_unarmed_finger_default_left_01",
            "female_unarmed_finger_default_right_01",
        ),
        "male_bow_attack_01" => ("male_bow_finger_default_left_01", ""),
        "male_bow_hold_idle_01" => (
            "male_bow_finger_default_left_01",
            "male_bow_finger_default_right_01",
        ),
        "male_hengdao_crouch_idle_01" | "male_hengdao_idle_01" => {
            ("male_hengdao_finger_left_01", "")
        }
        "male_die_hurt_stand_light_front_01"
        | "male_hengdao_crouch_promenade_01"
        | "male_hengdao_jump_down_loop_01" => (
            "male_hengdao_finger_left_01",
            "male_hengdao_finger_right_01",
        ),
        "male_sniper_attack_01" => ("male_unarmed_finger_default_left_01", ""),
        "male_fk_attack_01"
        | "male_fk_attack_01_pre_aim"
        | "male_fk_attack_01_recover"
        | "male_fk_hold_idle_01"
        | "male_fk_idle_01"
        | "male_sniper_ads_idle_01" => (
            "male_unarmed_finger_default_left_01",
            "male_unarmed_finger_default_right_01",
        ),
        _ => return None,
    })
}
