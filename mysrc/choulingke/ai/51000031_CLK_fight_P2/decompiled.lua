local AINodes = require("hexm.common.AI.nodes.ai_nodes").AINodes
local _M = {}
_M.TREE_NAME = "51000031_CLK_fight_P2"
_M.TREE_NODES_NUM = 195
_M.NODE_DATA_CACHE = {}

function _M.n0_CBTAlwaysSequenceNode()
  return {
    node_name = "n0_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    cls_name = "CBTAlwaysSequenceNode",
    child_num = 3,
    child_name_list = {
      "n00_PassNode",
      "n01_ChooseTargetAction",
      "n02_CBTParallelNode"
    }
  }
end

function _M.n00_PassNode()
  return {
    node_name = "n00_PassNode",
    node_type = 2,
    cls = AINodes.PassNode,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "PassNode",
    fast_node = "n01_ChooseTargetAction",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000_CBTAlwaysSequenceNode"
    },
    record_type = 1,
    total_pass_count = 1,
    pass_name = "",
    pass_group = "default"
  }
end

function _M.n000_CBTAlwaysSequenceNode()
  return {
    node_name = "n000_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n00_PassNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0000_CBTAlwaysSequenceNode",
      "n0001_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n0000_CBTAlwaysSequenceNode()
  return {
    node_name = "n0000_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n000_CBTAlwaysSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    fast_node = "n0001_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n00000_StopWanfaAction",
      "n00001_SetBlackboard"
    }
  }
end

function _M.n00000_StopWanfaAction()
  return {
    node_name = "n00000_StopWanfaAction",
    cls = AINodes.StopWanfaAction,
    parent_node_name = "n0000_CBTAlwaysSequenceNode",
    cls_name = "StopWanfaAction",
    fast_node = "n00001_SetBlackboard",
    idx_in_parent = 0
  }
end

function _M.n00001_SetBlackboard()
  return {
    node_name = "n00001_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0000_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "_fight_stage_",
    value_type = 3,
    value = "2",
    reason = ""
  }
end

function _M.n0001_CBTAlwaysSequenceNode()
  return {
    node_name = "n0001_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n000_CBTAlwaysSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 11,
    child_name_list = {
      "n00010_EnterCombatAction",
      "n00011_GetEntityAIData",
      "n00012_ApplyWanfaAction",
      "n00013_SetBlackboard",
      "n00014_SetBlackboard",
      "n00015_RegisterActionBtree",
      "n00016_SetVariableAction",
      "n00017_ApplyBuff",
      "n00018_SetBlackboard",
      "n00019_ApplyBuff",
      "n0001A_CBTSequenceNode"
    }
  }
end

function _M.n00010_EnterCombatAction()
  return {
    node_name = "n00010_EnterCombatAction",
    cls = AINodes.EnterCombatAction,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "EnterCombatAction",
    fast_node = "n00011_GetEntityAIData",
    idx_in_parent = 0
  }
end

function _M.n00011_GetEntityAIData()
  return {
    node_name = "n00011_GetEntityAIData",
    cls = AINodes.GetEntityAIData,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "GetEntityAIData",
    fast_node = "n00012_ApplyWanfaAction",
    idx_in_parent = 1,
    entity_bb_key = "",
    config_no_bb_key = "",
    data_from = "ai_blackboard_data",
    is_all = true,
    data_key_type = 4,
    data_key = "",
    default_value_type = 4,
    default_value = "",
    save_bb_key = ""
  }
end

function _M.n00012_ApplyWanfaAction()
  return {
    node_name = "n00012_ApplyWanfaAction",
    cls = AINodes.ApplyWanfaAction,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "ApplyWanfaAction",
    fast_node = "n00013_SetBlackboard",
    idx_in_parent = 2,
    wanfa_no = 51000025,
    sub_wanfa_ids = string.split("1,2,3", ","),
    leave_battle_end = true
  }
end

function _M.n00013_SetBlackboard()
  return {
    node_name = "n00013_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n00014_SetBlackboard",
    idx_in_parent = 3,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n00014_SetBlackboard()
  return {
    node_name = "n00014_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n00015_RegisterActionBtree",
    idx_in_parent = 4,
    is_space = false,
    key = "_ACTION_FLOW_",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n00015_RegisterActionBtree()
  return {
    node_name = "n00015_RegisterActionBtree",
    cls = AINodes.RegisterActionBtree,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "RegisterActionBtree",
    fast_node = "n00016_SetVariableAction",
    idx_in_parent = 5,
    btree_name = "51000025_CLK_fight_Action"
  }
end

function _M.n00016_SetVariableAction()
  return {
    node_name = "n00016_SetVariableAction",
    cls = AINodes.SetVariableAction,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetVariableAction",
    fast_node = "n00017_ApplyBuff",
    idx_in_parent = 6,
    set_type = "single",
    key_type = 2,
    key = "lock_type",
    value_type = 3,
    value = "1"
  }
end

function _M.n00017_ApplyBuff()
  return {
    node_name = "n00017_ApplyBuff",
    cls = AINodes.ApplyBuff,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "ApplyBuff",
    fast_node = "n00018_SetBlackboard",
    idx_in_parent = 7,
    op = "add",
    target_type = 2,
    target_list = "bb_main_player",
    buff_id = "450109",
    buff_target = "",
    use_buff_level = false,
    buff_level_type = 1,
    buff_level_list = ""
  }
end

function _M.n00018_SetBlackboard()
  return {
    node_name = "n00018_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n00019_ApplyBuff",
    idx_in_parent = 8,
    is_space = false,
    key = "attack_count",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n00019_ApplyBuff()
  return {
    node_name = "n00019_ApplyBuff",
    cls = AINodes.ApplyBuff,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "ApplyBuff",
    fast_node = "n0001A_CBTSequenceNode",
    idx_in_parent = 9,
    op = "add",
    target_type = 1,
    target_list = "bb_main_player",
    buff_id = "450107",
    buff_target = "",
    use_buff_level = false,
    buff_level_type = 1,
    buff_level_list = ""
  }
end

function _M.n0001A_CBTSequenceNode()
  return {
    node_name = "n0001A_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 10,
    child_num = 4,
    child_name_list = {
      "n0001A0_CheckTargetAchievementDone",
      "n0001A1_NotNode",
      "n0001A2_ApplyBuff",
      "n0001A3_PlaySound"
    }
  }
end

function _M.n0001A0_CheckTargetAchievementDone()
  return {
    node_name = "n0001A0_CheckTargetAchievementDone",
    cls = AINodes.CheckTargetAchievementDone,
    parent_node_name = "n0001A_CBTSequenceNode",
    cls_name = "CheckTargetAchievementDone",
    succ_fast_node = "n0001A1_NotNode",
    idx_in_parent = 0,
    target_bb_key = "_T1",
    id_type = 1,
    check_id = 3109
  }
end

function _M.n0001A1_NotNode()
  return {
    node_name = "n0001A1_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0001A_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n0001A2_ApplyBuff",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n0001A10_CheckTargetAchievementDone"
    }
  }
end

function _M.n0001A10_CheckTargetAchievementDone()
  return {
    node_name = "n0001A10_CheckTargetAchievementDone",
    cls = AINodes.CheckTargetAchievementDone,
    parent_node_name = "n0001A1_NotNode",
    cls_name = "CheckTargetAchievementDone",
    idx_in_parent = 0,
    target_bb_key = "_T1",
    id_type = 1,
    check_id = 3108
  }
end

function _M.n0001A2_ApplyBuff()
  return {
    node_name = "n0001A2_ApplyBuff",
    cls = AINodes.ApplyBuff,
    parent_node_name = "n0001A_CBTSequenceNode",
    cls_name = "ApplyBuff",
    succ_fast_node = "n0001A3_PlaySound",
    idx_in_parent = 2,
    op = "add",
    target_type = 2,
    target_list = "_T1",
    buff_id = "450100",
    buff_target = "",
    use_buff_level = false,
    buff_level_type = 1,
    buff_level_list = ""
  }
end

function _M.n0001A3_PlaySound()
  return {
    node_name = "n0001A3_PlaySound",
    cls = AINodes.PlaySound,
    parent_node_name = "n0001A_CBTSequenceNode",
    cls_name = "PlaySound",
    idx_in_parent = 3,
    sound_no = 6302990,
    sound_no_bb_key = "sound_no_bb_key",
    entity_bb_key = "",
    play_and_wait = false,
    is_3d = false
  }
end

function _M.n01_ChooseTargetAction()
  return {
    node_name = "n01_ChooseTargetAction",
    cls = AINodes.ChooseTargetAction,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "ChooseTargetAction",
    fast_node = "n02_CBTParallelNode",
    idx_in_parent = 1,
    target_source = 0,
    specify_npc_type = 3,
    specify_npc_no = "0",
    search_range_type = 3,
    search_range = "10",
    target_wx_list = "0",
    distance_filter_mode = 0,
    value_filter_mode = 1,
    pick_attr = "",
    pick_attr_mode = 0,
    target_bb_key = "_T1",
    filter_entity_mode = 0,
    entity_filter_key_type = 1,
    entity_filter_key = "",
    add_to_ray_check = false,
    ray_check_target_bone = "Bip001 Spine2",
    check_target_valid = false
  }
end

function _M.n02_CBTParallelNode()
  return {
    node_name = "n02_CBTParallelNode",
    node_type = 3,
    cls = AINodes.CBTParallelNode,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "CBTParallelNode",
    idx_in_parent = 2,
    child_num = 3,
    child_name_list = {
      "n020_CBTWhileNode",
      "n021_ApplyWatch",
      "n022_ConditionPriorityNode"
    }
  }
end

function _M.n020_CBTWhileNode()
  return {
    node_name = "n020_CBTWhileNode",
    cls = AINodes.CBTWhileNode,
    parent_node_name = "n02_CBTParallelNode",
    cls_name = "CBTWhileNode",
    child_fast_idx = 1,
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0200_Nop",
      "n0201_ReceiveEventDecorator"
    }
  }
end

function _M.n0200_Nop()
  return {
    node_name = "n0200_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n020_CBTWhileNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n0201_ReceiveEventDecorator()
  return {
    node_name = "n0201_ReceiveEventDecorator",
    node_type = 2,
    cls = AINodes.ReceiveEventDecorator,
    parent_node_name = "n020_CBTWhileNode",
    cls_name = "ReceiveEventDecorator",
    fast_node = "n0201_ReceiveEventDecorator",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n02010_CBTAlwaysSequenceNode"
    },
    decorator_result = "SUCCEED",
    filter_type = "block",
    event_from = 0,
    eid_bb_key = "_T1",
    event_str_list = string.split("e_max_aggro_changed", ","),
    save_data_key_list = string.split("", ","),
    compare_key_list = string.split("", ","),
    enable_reason = false,
    trigger_reason = "",
    callback_handle_func = "",
    trigger_event_name_bb_key = ""
  }
end

function _M.n02010_CBTAlwaysSequenceNode()
  return {
    node_name = "n02010_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0201_ReceiveEventDecorator",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n020100_ChooseTargetAction"
    }
  }
end

function _M.n020100_ChooseTargetAction()
  return {
    node_name = "n020100_ChooseTargetAction",
    cls = AINodes.ChooseTargetAction,
    parent_node_name = "n02010_CBTAlwaysSequenceNode",
    cls_name = "ChooseTargetAction",
    idx_in_parent = 0,
    target_source = 0,
    specify_npc_type = 3,
    specify_npc_no = "0",
    search_range_type = 3,
    search_range = "10",
    target_wx_list = "0",
    distance_filter_mode = 0,
    value_filter_mode = 1,
    pick_attr = "",
    pick_attr_mode = 0,
    target_bb_key = "_T1",
    filter_entity_mode = 0,
    entity_filter_key_type = 1,
    entity_filter_key = "",
    add_to_ray_check = false,
    ray_check_target_bone = "Bip001 Spine2",
    check_target_valid = false
  }
end

function _M.n021_ApplyWatch()
  return {
    node_name = "n021_ApplyWatch",
    cls = AINodes.ApplyWatch,
    parent_node_name = "n02_CBTParallelNode",
    cls_name = "ApplyWatch",
    idx_in_parent = 1,
    is_enabled = true,
    target_bb_key = "_T1",
    watch_type = 0,
    is_blocked = true,
    watch_time_type = 1,
    watch_time = "10.0",
    cancel_over_angle = false,
    cancel_over_angle_value = 90.0,
    over_angle_finish = true,
    over_angle_watch_interval = 2.0,
    lookat_node_type_2 = false,
    lookat_cancel = true,
    lookat_max_yaw = 60.0,
    lookat_max_pitch = 30.0
  }
end

function _M.n022_ConditionPriorityNode()
  return {
    node_name = "n022_ConditionPriorityNode",
    node_type = 2,
    cls = AINodes.ConditionPriorityNode,
    parent_node_name = "n02_CBTParallelNode",
    cls_name = "ConditionPriorityNode",
    idx_in_parent = 2,
    child_num = 5,
    child_name_list = {
      "n0220_CBTAlwaysSequenceNode",
      "n0221_CBTAlwaysSequenceNode",
      "n0222_CBTSequenceNode",
      "n0223_CBTSequenceNode",
      "n0224_CBTAlwaysSequenceNode"
    },
    conditions = listlib.filter(string.split("0#AIComboCondition#51000031_CLK_P2_ComboCondition#CanComboCancel,0#CheckBuffCondition#self#with;450079,0#GedangCondition#ALL,0#BBCheckCondition#attack_count;<;int;5,-1", ","), function(e)
      return "" ~= e
    end),
    tick_rate = 1.0
  }
end

function _M.n0220_CBTAlwaysSequenceNode()
  return {
    node_name = "n0220_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n022_ConditionPriorityNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n02200_CBTSwitchCaseNode"
    }
  }
end

function _M.n02200_CBTSwitchCaseNode()
  return {
    node_name = "n02200_CBTSwitchCaseNode",
    cls = AINodes.CBTSwitchCaseNode,
    parent_node_name = "n0220_CBTAlwaysSequenceNode",
    cls_name = "CBTSwitchCaseNode",
    idx_in_parent = 0,
    child_num = 29,
    child_name_list = {
      "n022000_CBTSequenceNode",
      "n022001_CBTSequenceNode",
      "n022002_CBTSelectNode",
      "n022003_CBTSequenceNode",
      "n022004_CBTSequenceNode",
      "n022005_CBTSelectNode",
      "n022006_CBTSequenceNode",
      "n022007_CBTSequenceNode",
      "n022008_CBTSequenceNode",
      "n022009_CBTSequenceNode",
      "n02200A_CBTSequenceNode",
      "n02200B_CBTSequenceNode",
      "n02200C_CBTSelectNode",
      "n02200D_CBTSequenceNode",
      "n02200E_CBTSequenceNode",
      "n02200F_CBTSequenceNode",
      "n02200G_CBTSequenceNode",
      "n02200H_CBTSequenceNode",
      "n02200I_CBTSelectNode",
      "n02200J_CBTSequenceNode",
      "n02200K_CBTSequenceNode",
      "n02200L_CBTSelectNode",
      "n02200M_CBTSequenceNode",
      "n02200N_CBTSequenceNode",
      "n02200O_CBTSequenceNode",
      "n02200P_CBTSequenceNode",
      "n02200Q_CBTSequenceNode",
      "n02200R_CBTSequenceNode",
      "n02200S_CBTSequenceNode"
    },
    key_type = 1,
    key = "clk_Combo",
    case_type = 4,
    cases_dict = {
      ["1"] = 0,
      ["2"] = 1,
      ["3"] = 2,
      ["4"] = 3,
      ["5"] = 4,
      ["6"] = 5,
      ["7"] = 6,
      ["8"] = 7,
      ["9"] = 8,
      ["10"] = 9,
      ["11"] = 10,
      ["12"] = 11,
      ["13"] = 12,
      ["14"] = 13,
      ["15"] = 14,
      ["16"] = 15,
      ["17"] = 16,
      ["18"] = 17,
      ["19"] = 18,
      ["20"] = 19,
      ["21"] = 20,
      ["22"] = 21,
      behit_gold = 22,
      tandaor = 23,
      tandaol = 24,
      ceshan = 25,
      tandaoR = 26,
      tandaoL = 27,
      breaksuccess = 28
    }
  }
end

function _M.n022000_CBTSequenceNode()
  return {
    node_name = "n022000_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0220000_ApplyActionDirect"
    }
  }
end

function _M.n0220000_ApplyActionDirect()
  return {
    node_name = "n0220000_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n022000_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 22
  }
end

function _M.n022001_CBTSequenceNode()
  return {
    node_name = "n022001_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n0220010_ApplyActionDirect"
    }
  }
end

function _M.n0220010_ApplyActionDirect()
  return {
    node_name = "n0220010_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n022001_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 33
  }
end

function _M.n022002_CBTSelectNode()
  return {
    node_name = "n022002_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 2,
    child_num = 2,
    child_name_list = {
      "n0220020_PassNode",
      "n0220021_ApplyActionByCondition"
    }
  }
end

function _M.n0220020_PassNode()
  return {
    node_name = "n0220020_PassNode",
    node_type = 2,
    cls = AINodes.PassNode,
    parent_node_name = "n022002_CBTSelectNode",
    cls_name = "PassNode",
    fail_fast_node = "n0220021_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n02200200_ApplyActionDirect"
    },
    record_type = 1,
    total_pass_count = 1,
    pass_name = "start",
    pass_group = "default"
  }
end

function _M.n02200200_ApplyActionDirect()
  return {
    node_name = "n02200200_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n0220020_PassNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 28
  }
end

function _M.n0220021_ApplyActionByCondition()
  return {
    node_name = "n0220021_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022002_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("56,53,51", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022003_CBTSequenceNode()
  return {
    node_name = "n022003_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 3,
    child_num = 1,
    child_name_list = {
      "n0220030_ApplyActionByCondition"
    }
  }
end

function _M.n0220030_ApplyActionByCondition()
  return {
    node_name = "n0220030_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022003_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("48,49,24", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("25,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022004_CBTSequenceNode()
  return {
    node_name = "n022004_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 4,
    child_num = 1,
    child_name_list = {
      "n0220040_ApplyActionByCondition"
    }
  }
end

function _M.n0220040_ApplyActionByCondition()
  return {
    node_name = "n0220040_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022004_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("48,49,24", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022005_CBTSelectNode()
  return {
    node_name = "n022005_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 5,
    child_num = 2,
    child_name_list = {
      "n0220050_CBTSequenceNode",
      "n0220051_ApplyActionByCondition"
    }
  }
end

function _M.n0220050_CBTSequenceNode()
  return {
    node_name = "n0220050_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022005_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n0220051_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02200500_CheckTargetDis",
      "n02200501_ApplyActionByCondition"
    }
  }
end

function _M.n02200500_CheckTargetDis()
  return {
    node_name = "n02200500_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n0220050_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02200501_ApplyActionByCondition",
    idx_in_parent = 0,
    base_type = 0,
    base_bb_key = "",
    target_type = 1,
    entity_bb_key = "_T1",
    target_abs_pos = {
      0,
      0,
      0
    },
    distance_type = 1,
    distance_value_type = 1,
    distance = "6",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02200501_ApplyActionByCondition()
  return {
    node_name = "n02200501_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0220050_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("56,58", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n0220051_ApplyActionByCondition()
  return {
    node_name = "n0220051_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022005_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("48,49,24", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("25,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022006_CBTSequenceNode()
  return {
    node_name = "n022006_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 6,
    child_num = 1,
    child_name_list = {
      "n0220060_ApplyActionDirect"
    }
  }
end

function _M.n0220060_ApplyActionDirect()
  return {
    node_name = "n0220060_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n022006_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 28
  }
end

function _M.n022007_CBTSequenceNode()
  return {
    node_name = "n022007_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 7,
    child_num = 1,
    child_name_list = {
      "n0220070_ApplyActionDirect"
    }
  }
end

function _M.n0220070_ApplyActionDirect()
  return {
    node_name = "n0220070_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n022007_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 55
  }
end

function _M.n022008_CBTSequenceNode()
  return {
    node_name = "n022008_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 8,
    child_num = 1,
    child_name_list = {
      "n0220080_ApplyActionByCondition"
    }
  }
end

function _M.n0220080_ApplyActionByCondition()
  return {
    node_name = "n0220080_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022008_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("23,54", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("99,1", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022009_CBTSequenceNode()
  return {
    node_name = "n022009_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 9,
    child_num = 1,
    child_name_list = {
      "n0220090_ApplyActionByCondition"
    }
  }
end

function _M.n0220090_ApplyActionByCondition()
  return {
    node_name = "n0220090_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022009_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("57,54", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200A_CBTSequenceNode()
  return {
    node_name = "n02200A_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 10,
    child_num = 1,
    child_name_list = {
      "n02200A0_ApplyActionByCondition"
    }
  }
end

function _M.n02200A0_ApplyActionByCondition()
  return {
    node_name = "n02200A0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200A_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("57,54", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200B_CBTSequenceNode()
  return {
    node_name = "n02200B_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 11,
    child_num = 1,
    child_name_list = {
      "n02200B0_ApplyActionByCondition"
    }
  }
end

function _M.n02200B0_ApplyActionByCondition()
  return {
    node_name = "n02200B0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200B_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("52,31", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200C_CBTSelectNode()
  return {
    node_name = "n02200C_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 12,
    child_num = 2,
    child_name_list = {
      "n02200C0_PassNode",
      "n02200C1_ApplyActionByCondition"
    }
  }
end

function _M.n02200C0_PassNode()
  return {
    node_name = "n02200C0_PassNode",
    node_type = 2,
    cls = AINodes.PassNode,
    parent_node_name = "n02200C_CBTSelectNode",
    cls_name = "PassNode",
    fail_fast_node = "n02200C1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n02200C00_ApplyActionDirect"
    },
    record_type = 1,
    total_pass_count = 1,
    pass_name = "start",
    pass_group = "default"
  }
end

function _M.n02200C00_ApplyActionDirect()
  return {
    node_name = "n02200C00_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200C0_PassNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 28
  }
end

function _M.n02200C1_ApplyActionByCondition()
  return {
    node_name = "n02200C1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200C_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("53,58,51", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200D_CBTSequenceNode()
  return {
    node_name = "n02200D_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 13,
    child_num = 1,
    child_name_list = {
      "n02200D0_ApplyActionByCondition"
    }
  }
end

function _M.n02200D0_ApplyActionByCondition()
  return {
    node_name = "n02200D0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200D_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("48,49,24", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("25,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200E_CBTSequenceNode()
  return {
    node_name = "n02200E_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 14,
    child_num = 1,
    child_name_list = {
      "n02200E0_ApplyActionDirect"
    }
  }
end

function _M.n02200E0_ApplyActionDirect()
  return {
    node_name = "n02200E0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200E_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 16
  }
end

function _M.n02200F_CBTSequenceNode()
  return {
    node_name = "n02200F_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 15,
    child_num = 1,
    child_name_list = {
      "n02200F0_ApplyActionByCondition"
    }
  }
end

function _M.n02200F0_ApplyActionByCondition()
  return {
    node_name = "n02200F0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200F_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("48,49,24", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200G_CBTSequenceNode()
  return {
    node_name = "n02200G_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 16,
    child_num = 1,
    child_name_list = {
      "n02200G0_ApplyActionByCondition"
    }
  }
end

function _M.n02200G0_ApplyActionByCondition()
  return {
    node_name = "n02200G0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200G_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("59,48", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200H_CBTSequenceNode()
  return {
    node_name = "n02200H_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 17,
    child_num = 1,
    child_name_list = {
      "n02200H0_ApplyActionByCondition"
    }
  }
end

function _M.n02200H0_ApplyActionByCondition()
  return {
    node_name = "n02200H0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200H_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("55,31", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200I_CBTSelectNode()
  return {
    node_name = "n02200I_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 18,
    child_num = 2,
    child_name_list = {
      "n02200I0_PassNode",
      "n02200I1_ApplyActionDirect"
    }
  }
end

function _M.n02200I0_PassNode()
  return {
    node_name = "n02200I0_PassNode",
    node_type = 2,
    cls = AINodes.PassNode,
    parent_node_name = "n02200I_CBTSelectNode",
    cls_name = "PassNode",
    fail_fast_node = "n02200I1_ApplyActionDirect",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n02200I00_ApplyActionDirect"
    },
    record_type = 1,
    total_pass_count = 1,
    pass_name = "start",
    pass_group = "default"
  }
end

function _M.n02200I00_ApplyActionDirect()
  return {
    node_name = "n02200I00_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200I0_PassNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 28
  }
end

function _M.n02200I1_ApplyActionDirect()
  return {
    node_name = "n02200I1_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200I_CBTSelectNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 1,
    action_value = 59
  }
end

function _M.n02200J_CBTSequenceNode()
  return {
    node_name = "n02200J_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 19,
    child_num = 1,
    child_name_list = {
      "n02200J0_ApplyActionByCondition"
    }
  }
end

function _M.n02200J0_ApplyActionByCondition()
  return {
    node_name = "n02200J0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200J_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("57", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200K_CBTSequenceNode()
  return {
    node_name = "n02200K_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 20,
    child_num = 1,
    child_name_list = {
      "n02200K0_ApplyActionDirect"
    }
  }
end

function _M.n02200K0_ApplyActionDirect()
  return {
    node_name = "n02200K0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200K_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 60
  }
end

function _M.n02200L_CBTSelectNode()
  return {
    node_name = "n02200L_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 21,
    child_num = 2,
    child_name_list = {
      "n02200L0_CBTSequenceNode",
      "n02200L1_ApplyActionDirect"
    }
  }
end

function _M.n02200L0_CBTSequenceNode()
  return {
    node_name = "n02200L0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200L_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02200L1_ApplyActionDirect",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02200L00_CheckTargetDis",
      "n02200L01_ApplyActionByCondition"
    }
  }
end

function _M.n02200L00_CheckTargetDis()
  return {
    node_name = "n02200L00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n02200L0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02200L01_ApplyActionByCondition",
    idx_in_parent = 0,
    base_type = 0,
    base_bb_key = "",
    target_type = 1,
    entity_bb_key = "_T1",
    target_abs_pos = {
      0,
      0,
      0
    },
    distance_type = 1,
    distance_value_type = 1,
    distance = "6",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02200L01_ApplyActionByCondition()
  return {
    node_name = "n02200L01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02200L0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("57", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02200L1_ApplyActionDirect()
  return {
    node_name = "n02200L1_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200L_CBTSelectNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 1,
    action_value = 48
  }
end

function _M.n02200M_CBTSequenceNode()
  return {
    node_name = "n02200M_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 22,
    child_num = 1,
    child_name_list = {
      "n02200M0_ApplyActionDirect"
    }
  }
end

function _M.n02200M0_ApplyActionDirect()
  return {
    node_name = "n02200M0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200M_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 34
  }
end

function _M.n02200N_CBTSequenceNode()
  return {
    node_name = "n02200N_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 23,
    child_num = 1,
    child_name_list = {
      "n02200N0_ApplyActionDirect"
    }
  }
end

function _M.n02200N0_ApplyActionDirect()
  return {
    node_name = "n02200N0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200N_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 7
  }
end

function _M.n02200O_CBTSequenceNode()
  return {
    node_name = "n02200O_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 24,
    child_num = 1,
    child_name_list = {
      "n02200O0_ApplyActionDirect"
    }
  }
end

function _M.n02200O0_ApplyActionDirect()
  return {
    node_name = "n02200O0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200O_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 6
  }
end

function _M.n02200P_CBTSequenceNode()
  return {
    node_name = "n02200P_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 25,
    child_num = 1,
    child_name_list = {
      "n02200P0_ApplyActionDirect"
    }
  }
end

function _M.n02200P0_ApplyActionDirect()
  return {
    node_name = "n02200P0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200P_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 4
  }
end

function _M.n02200Q_CBTSequenceNode()
  return {
    node_name = "n02200Q_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 26,
    child_num = 1,
    child_name_list = {
      "n02200Q0_ApplyActionDirect"
    }
  }
end

function _M.n02200Q0_ApplyActionDirect()
  return {
    node_name = "n02200Q0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200Q_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 9
  }
end

function _M.n02200R_CBTSequenceNode()
  return {
    node_name = "n02200R_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 27,
    child_num = 1,
    child_name_list = {
      "n02200R0_ApplyActionDirect"
    }
  }
end

function _M.n02200R0_ApplyActionDirect()
  return {
    node_name = "n02200R0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200R_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 8
  }
end

function _M.n02200S_CBTSequenceNode()
  return {
    node_name = "n02200S_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02200_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 28,
    child_num = 1,
    child_name_list = {
      "n02200S0_ApplyActionDirect"
    }
  }
end

function _M.n02200S0_ApplyActionDirect()
  return {
    node_name = "n02200S0_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02200S_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 47
  }
end

function _M.n0221_CBTAlwaysSequenceNode()
  return {
    node_name = "n0221_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n022_ConditionPriorityNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n02210_ApplyActionByCondition"
    }
  }
end

function _M.n02210_ApplyActionByCondition()
  return {
    node_name = "n02210_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0221_CBTAlwaysSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("1", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n0222_CBTSequenceNode()
  return {
    node_name = "n0222_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022_ConditionPriorityNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 2,
    child_num = 2,
    child_name_list = {
      "n02220_CheckBlackboard",
      "n02221_Probability"
    }
  }
end

function _M.n02220_CheckBlackboard()
  return {
    node_name = "n02220_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0222_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n02221_Probability",
    idx_in_parent = 0,
    key_from = 1,
    key = "_GD_SKILL_ID",
    operator = "==",
    value_type = 3,
    value = "20901006"
  }
end

function _M.n02221_Probability()
  return {
    node_name = "n02221_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n0222_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n022210_ApplyActionDirect",
      "n022211_ApplyActionDirect"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = true
  }
end

function _M.n022210_ApplyActionDirect()
  return {
    node_name = "n022210_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02221_Probability",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 6
  }
end

function _M.n022211_ApplyActionDirect()
  return {
    node_name = "n022211_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02221_Probability",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 1,
    action_value = 7
  }
end

function _M.n0223_CBTSequenceNode()
  return {
    node_name = "n0223_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022_ConditionPriorityNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 3,
    child_num = 2,
    child_name_list = {
      "n02230_ChooseTargetAction",
      "n02231_CBTSelectNode"
    }
  }
end

function _M.n02230_ChooseTargetAction()
  return {
    node_name = "n02230_ChooseTargetAction",
    cls = AINodes.ChooseTargetAction,
    parent_node_name = "n0223_CBTSequenceNode",
    cls_name = "ChooseTargetAction",
    succ_fast_node = "n02231_CBTSelectNode",
    idx_in_parent = 0,
    target_source = 0,
    specify_npc_type = 3,
    specify_npc_no = "0",
    search_range_type = 3,
    search_range = "10",
    target_wx_list = "0",
    distance_filter_mode = 0,
    value_filter_mode = 1,
    pick_attr = "",
    pick_attr_mode = 0,
    target_bb_key = "_T1",
    filter_entity_mode = 0,
    entity_filter_key_type = 1,
    entity_filter_key = "",
    add_to_ray_check = false,
    ray_check_target_bone = "Bip001 Spine2",
    check_target_valid = true
  }
end

function _M.n02231_CBTSelectNode()
  return {
    node_name = "n02231_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0223_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 6,
    child_name_list = {
      "n022310_PassNode",
      "n022311_CBTIfElseNode",
      "n022312_CheckDisAndAngleDecorator",
      "n022313_CheckDisAndAngleDecorator",
      "n022314_CheckDisAndAngleDecorator",
      "n022315_ApplyActionByCondition"
    }
  }
end

function _M.n022310_PassNode()
  return {
    node_name = "n022310_PassNode",
    node_type = 2,
    cls = AINodes.PassNode,
    parent_node_name = "n02231_CBTSelectNode",
    cls_name = "PassNode",
    fail_fast_node = "n022311_CBTIfElseNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0223100_ApplyActionByCondition"
    },
    record_type = 1,
    total_pass_count = 1,
    pass_name = "",
    pass_group = "default"
  }
end

function _M.n0223100_ApplyActionByCondition()
  return {
    node_name = "n0223100_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022310_PassNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("21", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022311_CBTIfElseNode()
  return {
    node_name = "n022311_CBTIfElseNode",
    cls = AINodes.CBTIfElseNode,
    parent_node_name = "n02231_CBTSelectNode",
    cls_name = "CBTIfElseNode",
    fail_fast_node = "n022312_CheckDisAndAngleDecorator",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n0223110_CompareWithPercent",
      "n0223111_CBTIfElseNode",
      "n0223112_NotNode"
    }
  }
end

function _M.n0223110_CompareWithPercent()
  return {
    node_name = "n0223110_CompareWithPercent",
    cls = AINodes.CompareWithPercent,
    parent_node_name = "n022311_CBTIfElseNode",
    cls_name = "CompareWithPercent",
    succ_fast_node = "n0223111_CBTIfElseNode",
    fail_fast_node = "n0223112_NotNode",
    idx_in_parent = 0,
    left_value_type = 2,
    left_value = "HP",
    right_value_type = 2,
    right_value = "HP_MAX",
    percent_type = 3,
    percent = "75",
    operation = "<="
  }
end

function _M.n0223111_CBTIfElseNode()
  return {
    node_name = "n0223111_CBTIfElseNode",
    cls = AINodes.CBTIfElseNode,
    parent_node_name = "n022311_CBTIfElseNode",
    cls_name = "CBTIfElseNode",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n02231110_CompareWithPercent",
      "n02231111_CheckTimerDecorator",
      "n02231112_CheckTimerDecorator"
    }
  }
end

function _M.n02231110_CompareWithPercent()
  return {
    node_name = "n02231110_CompareWithPercent",
    cls = AINodes.CompareWithPercent,
    parent_node_name = "n0223111_CBTIfElseNode",
    cls_name = "CompareWithPercent",
    succ_fast_node = "n02231111_CheckTimerDecorator",
    fail_fast_node = "n02231112_CheckTimerDecorator",
    idx_in_parent = 0,
    left_value_type = 2,
    left_value = "HP",
    right_value_type = 2,
    right_value = "HP_MAX",
    percent_type = 3,
    percent = "30",
    operation = "<="
  }
end

function _M.n02231111_CheckTimerDecorator()
  return {
    node_name = "n02231111_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n0223111_CBTIfElseNode",
    cls_name = "CheckTimerDecorator",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n022311110_CBTSelectNode"
    },
    decorator_result = "",
    timer_key = "CA_timer",
    duration = 70.0,
    first_no_exec = false
  }
end

function _M.n022311110_CBTSelectNode()
  return {
    node_name = "n022311110_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02231111_CheckTimerDecorator",
    cls_name = "CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0223111100_CBTSequenceNode",
      "n0223111101_CBTSequenceNode"
    }
  }
end

function _M.n0223111100_CBTSequenceNode()
  return {
    node_name = "n0223111100_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022311110_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n0223111101_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02231111000_CheckTargetDis",
      "n02231111001_ApplyActionDirect"
    }
  }
end

function _M.n02231111000_CheckTargetDis()
  return {
    node_name = "n02231111000_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n0223111100_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02231111001_ApplyActionDirect",
    idx_in_parent = 0,
    base_type = 0,
    base_bb_key = "",
    target_type = 1,
    entity_bb_key = "_T1",
    target_abs_pos = {
      0,
      0,
      0
    },
    distance_type = 1,
    distance_value_type = 1,
    distance = "6",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02231111001_ApplyActionDirect()
  return {
    node_name = "n02231111001_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n0223111100_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 1,
    action_value = 60
  }
end

function _M.n0223111101_CBTSequenceNode()
  return {
    node_name = "n0223111101_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022311110_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n02231111010_SetBlackboard"
    }
  }
end

function _M.n02231111010_SetBlackboard()
  return {
    node_name = "n02231111010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0223111101_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "is_CA",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n02231112_CheckTimerDecorator()
  return {
    node_name = "n02231112_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n0223111_CBTIfElseNode",
    cls_name = "CheckTimerDecorator",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n022311120_CBTSelectNode"
    },
    decorator_result = "",
    timer_key = "CA_timer",
    duration = 70.0,
    first_no_exec = false
  }
end

function _M.n022311120_CBTSelectNode()
  return {
    node_name = "n022311120_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02231112_CheckTimerDecorator",
    cls_name = "CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0223111200_CBTSequenceNode",
      "n0223111201_CBTSequenceNode"
    }
  }
end

function _M.n0223111200_CBTSequenceNode()
  return {
    node_name = "n0223111200_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022311120_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n0223111201_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02231112000_CheckTargetDis",
      "n02231112001_ApplyActionDirect"
    }
  }
end

function _M.n02231112000_CheckTargetDis()
  return {
    node_name = "n02231112000_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n0223111200_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02231112001_ApplyActionDirect",
    idx_in_parent = 0,
    base_type = 0,
    base_bb_key = "",
    target_type = 1,
    entity_bb_key = "_T1",
    target_abs_pos = {
      0,
      0,
      0
    },
    distance_type = 1,
    distance_value_type = 1,
    distance = "6",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02231112001_ApplyActionDirect()
  return {
    node_name = "n02231112001_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n0223111200_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 1,
    action_value = 60
  }
end

function _M.n0223111201_CBTSequenceNode()
  return {
    node_name = "n0223111201_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022311120_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n02231112010_SetBlackboard"
    }
  }
end

function _M.n02231112010_SetBlackboard()
  return {
    node_name = "n02231112010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0223111201_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "is_CA",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n0223112_NotNode()
  return {
    node_name = "n0223112_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n022311_CBTIfElseNode",
    cls_name = "NotNode",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n02231120_Nop"
    }
  }
end

function _M.n02231120_Nop()
  return {
    node_name = "n02231120_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n0223112_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n022312_CheckDisAndAngleDecorator()
  return {
    node_name = "n022312_CheckDisAndAngleDecorator",
    node_type = 2,
    cls = AINodes.CheckDisAndAngleDecorator,
    parent_node_name = "n02231_CBTSelectNode",
    cls_name = "CheckDisAndAngleDecorator",
    fail_fast_node = "n022313_CheckDisAndAngleDecorator",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n0223120_ApplyActionByCondition"
    },
    decorator_result = "",
    check_dis = true,
    base_type = 0,
    base_bb_key = "",
    target_type = 1,
    entity_bb_key = "_T1",
    target_abs_pos = {
      0,
      0,
      0
    },
    distance_type = 1,
    distance_value_type = 1,
    distance_interval = {0, 0},
    distance = "12",
    operation = ">",
    check_angle = false,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "-60.0",
    right_v = 60.0
  }
end

function _M.n0223120_ApplyActionByCondition()
  return {
    node_name = "n0223120_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022312_CheckDisAndAngleDecorator",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("3", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022313_CheckDisAndAngleDecorator()
  return {
    node_name = "n022313_CheckDisAndAngleDecorator",
    node_type = 2,
    cls = AINodes.CheckDisAndAngleDecorator,
    parent_node_name = "n02231_CBTSelectNode",
    cls_name = "CheckDisAndAngleDecorator",
    fail_fast_node = "n022314_CheckDisAndAngleDecorator",
    idx_in_parent = 3,
    child_num = 1,
    child_name_list = {
      "n0223130_CBTSelectNode"
    },
    decorator_result = "",
    check_dis = true,
    base_type = 0,
    base_bb_key = "",
    target_type = 1,
    entity_bb_key = "_T1",
    target_abs_pos = {
      0,
      0,
      0
    },
    distance_type = 1,
    distance_value_type = 1,
    distance_interval = {0, 0},
    distance = "6",
    operation = ">",
    check_angle = false,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "-60.0",
    right_v = 60.0
  }
end

function _M.n0223130_CBTSelectNode()
  return {
    node_name = "n0223130_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n022313_CheckDisAndAngleDecorator",
    cls_name = "CBTSelectNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n02231300_CBTSequenceNode",
      "n02231301_CheckTimerDecorator",
      "n02231302_ApplyActionByCondition"
    }
  }
end

function _M.n02231300_CBTSequenceNode()
  return {
    node_name = "n02231300_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0223130_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02231301_CheckTimerDecorator",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n022313000_CheckBuff",
      "n022313001_CheckTimerDecorator"
    }
  }
end

function _M.n022313000_CheckBuff()
  return {
    node_name = "n022313000_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02231300_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n022313001_CheckTimerDecorator",
    idx_in_parent = 0,
    target_type = 2,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450100",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n022313001_CheckTimerDecorator()
  return {
    node_name = "n022313001_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n02231300_CBTSequenceNode",
    cls_name = "CheckTimerDecorator",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n0223130010_ApplyActionByCondition"
    },
    decorator_result = "",
    timer_key = "xihun_timer",
    duration = 40.0,
    first_no_exec = false
  }
end

function _M.n0223130010_ApplyActionByCondition()
  return {
    node_name = "n0223130010_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022313001_CheckTimerDecorator",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("29,30,27", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,900", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02231301_CheckTimerDecorator()
  return {
    node_name = "n02231301_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n0223130_CBTSelectNode",
    cls_name = "CheckTimerDecorator",
    fail_fast_node = "n02231302_ApplyActionByCondition",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n022313010_ApplyActionByCondition"
    },
    decorator_result = "",
    timer_key = "xihun_timer",
    duration = 40.0,
    first_no_exec = false
  }
end

function _M.n022313010_ApplyActionByCondition()
  return {
    node_name = "n022313010_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02231301_CheckTimerDecorator",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("29,30", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02231302_ApplyActionByCondition()
  return {
    node_name = "n02231302_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0223130_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 2,
    random_list = listlib.filter(string.split("48,49,24,3,21", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,50,10,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022314_CheckDisAndAngleDecorator()
  return {
    node_name = "n022314_CheckDisAndAngleDecorator",
    node_type = 2,
    cls = AINodes.CheckDisAndAngleDecorator,
    parent_node_name = "n02231_CBTSelectNode",
    cls_name = "CheckDisAndAngleDecorator",
    fail_fast_node = "n022315_ApplyActionByCondition",
    idx_in_parent = 4,
    child_num = 1,
    child_name_list = {
      "n0223140_CBTSelectNode"
    },
    decorator_result = "",
    check_dis = true,
    base_type = 0,
    base_bb_key = "",
    target_type = 1,
    entity_bb_key = "_T1",
    target_abs_pos = {
      0,
      0,
      0
    },
    distance_type = 1,
    distance_value_type = 1,
    distance_interval = {0, 0},
    distance = "6",
    operation = "<=",
    check_angle = false,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "-60.0",
    right_v = 60.0
  }
end

function _M.n0223140_CBTSelectNode()
  return {
    node_name = "n0223140_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n022314_CheckDisAndAngleDecorator",
    cls_name = "CBTSelectNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n02231400_CBTSequenceNode",
      "n02231401_CheckTimerDecorator",
      "n02231402_ApplyActionByCondition"
    }
  }
end

function _M.n02231400_CBTSequenceNode()
  return {
    node_name = "n02231400_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0223140_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02231401_CheckTimerDecorator",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n022314000_CheckBuff",
      "n022314001_CheckTimerDecorator"
    }
  }
end

function _M.n022314000_CheckBuff()
  return {
    node_name = "n022314000_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02231400_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n022314001_CheckTimerDecorator",
    idx_in_parent = 0,
    target_type = 2,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450100",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n022314001_CheckTimerDecorator()
  return {
    node_name = "n022314001_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n02231400_CBTSequenceNode",
    cls_name = "CheckTimerDecorator",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n0223140010_ApplyActionByCondition"
    },
    decorator_result = "",
    timer_key = "xihun_timer",
    duration = 40.0,
    first_no_exec = false
  }
end

function _M.n0223140010_ApplyActionByCondition()
  return {
    node_name = "n0223140010_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n022314001_CheckTimerDecorator",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("29,30,27", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,900", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02231401_CheckTimerDecorator()
  return {
    node_name = "n02231401_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n0223140_CBTSelectNode",
    cls_name = "CheckTimerDecorator",
    fail_fast_node = "n02231402_ApplyActionByCondition",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n022314010_ApplyActionByCondition"
    },
    decorator_result = "",
    timer_key = "xihun_timer",
    duration = 40.0,
    first_no_exec = false
  }
end

function _M.n022314010_ApplyActionByCondition()
  return {
    node_name = "n022314010_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02231401_CheckTimerDecorator",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("29,30", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02231402_ApplyActionByCondition()
  return {
    node_name = "n02231402_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0223140_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 2,
    random_list = listlib.filter(string.split("50,25,53", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n022315_ApplyActionByCondition()
  return {
    node_name = "n022315_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02231_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 5,
    random_list = listlib.filter(string.split("1", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n0224_CBTAlwaysSequenceNode()
  return {
    node_name = "n0224_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n022_ConditionPriorityNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 4,
    child_num = 1,
    child_name_list = {
      "n02240_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n02240_CBTAlwaysSequenceNode()
  return {
    node_name = "n02240_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0224_CBTAlwaysSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n022400_CBTAlwaysSequenceNode",
      "n022401_SetBlackboard"
    }
  }
end

function _M.n022400_CBTAlwaysSequenceNode()
  return {
    node_name = "n022400_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n02240_CBTAlwaysSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    fast_node = "n022401_SetBlackboard",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0224000_CBTSequenceNode",
      "n0224001_CBTSelectNode"
    }
  }
end

function _M.n0224000_CBTSequenceNode()
  return {
    node_name = "n0224000_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022400_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0224001_CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02240000_JudgeTargetInRangeAction",
      "n02240001_TurningAction"
    }
  }
end

function _M.n02240000_JudgeTargetInRangeAction()
  return {
    node_name = "n02240000_JudgeTargetInRangeAction",
    cls = AINodes.JudgeTargetInRangeAction,
    parent_node_name = "n0224000_CBTSequenceNode",
    cls_name = "JudgeTargetInRangeAction",
    succ_fast_node = "n02240001_TurningAction",
    idx_in_parent = 0,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "135.0",
    right_v = 225.0
  }
end

function _M.n02240001_TurningAction()
  return {
    node_name = "n02240001_TurningAction",
    cls = AINodes.TurningAction,
    parent_node_name = "n0224000_CBTSequenceNode",
    cls_name = "TurningAction",
    idx_in_parent = 1,
    target_type = 1,
    anim_trans_time = -1.0,
    entity_bb_key = "_T1",
    duration = 3.0,
    bias_yaw = 1.05,
    diff_yaw = 0.0,
    turn_speed = 1.0,
    yaw_speed = -1.0,
    end_diff = 0.0,
    enable_physics = true,
    once_turning = false,
    scale_yaw = false,
    interrupt_watch = true,
    turn_on_tach = false,
    disable_yaw_controller = false
  }
end

function _M.n0224001_CBTSelectNode()
  return {
    node_name = "n0224001_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n022400_CBTAlwaysSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02240010_CBTSequenceNode",
      "n02240011_CBTSelectNode"
    }
  }
end

function _M.n02240010_CBTSequenceNode()
  return {
    node_name = "n02240010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0224001_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02240011_CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n022400100_CheckTimer",
      "n022400101_Probability"
    }
  }
end

function _M.n022400100_CheckTimer()
  return {
    node_name = "n022400100_CheckTimer",
    cls = AINodes.CheckTimer,
    parent_node_name = "n02240010_CBTSequenceNode",
    cls_name = "CheckTimer",
    succ_fast_node = "n022400101_Probability",
    idx_in_parent = 0,
    timer_key_type = 4,
    timer_key = "LockTimer2",
    is_pass_none_timer = true
  }
end

function _M.n022400101_Probability()
  return {
    node_name = "n022400101_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n02240010_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0224001010_CBTSequenceNode",
      "n0224001011_CBTSequenceNode"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = false
  }
end

function _M.n0224001010_CBTSequenceNode()
  return {
    node_name = "n0224001010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022400101_Probability",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02240010100_CheckBlackboard",
      "n02240010101_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n02240010100_CheckBlackboard()
  return {
    node_name = "n02240010100_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0224001010_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n02240010101_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "1"
  }
end

function _M.n02240010101_CBTAlwaysSequenceNode()
  return {
    node_name = "n02240010101_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0224001010_CBTSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n022400101010_SetBlackboard",
      "n022400101011_SetTimer",
      "n022400101012_LockAnimLRAction"
    }
  }
end

function _M.n022400101010_SetBlackboard()
  return {
    node_name = "n022400101010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n02240010101_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n022400101011_SetTimer",
    idx_in_parent = 0,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n022400101011_SetTimer()
  return {
    node_name = "n022400101011_SetTimer",
    cls = AINodes.SetTimer,
    parent_node_name = "n02240010101_CBTAlwaysSequenceNode",
    cls_name = "SetTimer",
    fast_node = "n022400101012_LockAnimLRAction",
    idx_in_parent = 1,
    timer_key = "LockTimer2",
    duration_type = "random",
    duration = 1.0,
    duration_max = 2.0,
    duration_bb = ""
  }
end

function _M.n022400101012_LockAnimLRAction()
  return {
    node_name = "n022400101012_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02240010101_CBTAlwaysSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 2,
    entity_bb_key = "_T1",
    move_type = 1,
    min_duration = "1",
    max_duration = "2",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "2",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n0224001011_CBTSequenceNode()
  return {
    node_name = "n0224001011_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022400101_Probability",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02240010110_CheckBlackboard",
      "n02240010111_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n02240010110_CheckBlackboard()
  return {
    node_name = "n02240010110_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0224001011_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n02240010111_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "0"
  }
end

function _M.n02240010111_CBTAlwaysSequenceNode()
  return {
    node_name = "n02240010111_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0224001011_CBTSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n022400101110_SetBlackboard",
      "n022400101111_SetTimer",
      "n022400101112_LockAnimLRAction"
    }
  }
end

function _M.n022400101110_SetBlackboard()
  return {
    node_name = "n022400101110_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n02240010111_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n022400101111_SetTimer",
    idx_in_parent = 0,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n022400101111_SetTimer()
  return {
    node_name = "n022400101111_SetTimer",
    cls = AINodes.SetTimer,
    parent_node_name = "n02240010111_CBTAlwaysSequenceNode",
    cls_name = "SetTimer",
    fast_node = "n022400101112_LockAnimLRAction",
    idx_in_parent = 1,
    timer_key = "LockTimer2",
    duration_type = "random",
    duration = 1.0,
    duration_max = 2.0,
    duration_bb = ""
  }
end

function _M.n022400101112_LockAnimLRAction()
  return {
    node_name = "n022400101112_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02240010111_CBTAlwaysSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 2,
    entity_bb_key = "_T1",
    move_type = 0,
    min_duration = "1",
    max_duration = "2",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "2",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n02240011_CBTSelectNode()
  return {
    node_name = "n02240011_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0224001_CBTSelectNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n022400110_CBTSequenceNode",
      "n022400111_CBTSequenceNode"
    }
  }
end

function _M.n022400110_CBTSequenceNode()
  return {
    node_name = "n022400110_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02240011_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n022400111_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0224001100_CheckBlackboard",
      "n0224001101_CBTSelectNode"
    }
  }
end

function _M.n0224001100_CheckBlackboard()
  return {
    node_name = "n0224001100_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n022400110_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n0224001101_CBTSelectNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "0"
  }
end

function _M.n0224001101_CBTSelectNode()
  return {
    node_name = "n0224001101_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n022400110_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02240011010_CBTSequenceNode",
      "n02240011011_LockAnimLRAction"
    }
  }
end

function _M.n02240011010_CBTSequenceNode()
  return {
    node_name = "n02240011010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0224001101_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02240011011_LockAnimLRAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n022400110100_CheckObstacle",
      "n022400110101_LockAnimLRAction"
    }
  }
end

function _M.n022400110100_CheckObstacle()
  return {
    node_name = "n022400110100_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n02240011010_CBTSequenceNode",
    cls_name = "CheckObstacle",
    succ_fast_node = "n022400110101_LockAnimLRAction",
    idx_in_parent = 0,
    bb_target = "_T1",
    direction = 4,
    target_bone_name = "Bip001 Spine1",
    target_ent_bone_name = "Scene Root",
    distance = 3.0,
    collision_type = 39,
    ignore_self = false,
    draw_line_flag = false,
    use_sweep = false,
    use_target_ent = false
  }
end

function _M.n022400110101_LockAnimLRAction()
  return {
    node_name = "n022400110101_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02240011010_CBTSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 0,
    min_duration = "1",
    max_duration = "2",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "2",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n02240011011_LockAnimLRAction()
  return {
    node_name = "n02240011011_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n0224001101_CBTSelectNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 1,
    min_duration = "1",
    max_duration = "2",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "2",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n022400111_CBTSequenceNode()
  return {
    node_name = "n022400111_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02240011_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0224001110_CheckBlackboard",
      "n0224001111_CBTSelectNode"
    }
  }
end

function _M.n0224001110_CheckBlackboard()
  return {
    node_name = "n0224001110_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n022400111_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n0224001111_CBTSelectNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "1"
  }
end

function _M.n0224001111_CBTSelectNode()
  return {
    node_name = "n0224001111_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n022400111_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02240011110_CBTSequenceNode",
      "n02240011111_LockAnimLRAction"
    }
  }
end

function _M.n02240011110_CBTSequenceNode()
  return {
    node_name = "n02240011110_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0224001111_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02240011111_LockAnimLRAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n022400111100_CheckObstacle",
      "n022400111101_LockAnimLRAction"
    }
  }
end

function _M.n022400111100_CheckObstacle()
  return {
    node_name = "n022400111100_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n02240011110_CBTSequenceNode",
    cls_name = "CheckObstacle",
    succ_fast_node = "n022400111101_LockAnimLRAction",
    idx_in_parent = 0,
    bb_target = "_T1",
    direction = 2,
    target_bone_name = "Bip001 Spine1",
    target_ent_bone_name = "Scene Root",
    distance = 3.0,
    collision_type = 39,
    ignore_self = false,
    draw_line_flag = false,
    use_sweep = false,
    use_target_ent = false
  }
end

function _M.n022400111101_LockAnimLRAction()
  return {
    node_name = "n022400111101_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02240011110_CBTSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 1,
    min_duration = "1",
    max_duration = "2",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "2",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n02240011111_LockAnimLRAction()
  return {
    node_name = "n02240011111_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n0224001111_CBTSelectNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 0,
    min_duration = "1",
    max_duration = "2",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "2",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n022401_SetBlackboard()
  return {
    node_name = "n022401_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n02240_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "attack_count",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

_M.entry = "n0_CBTAlwaysSequenceNode"
return _M
