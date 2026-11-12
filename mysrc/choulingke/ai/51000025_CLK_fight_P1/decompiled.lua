local AINodes = require("hexm.common.AI.nodes.ai_nodes").AINodes
local _M = {}
_M.TREE_NAME = "51000025_CLK_fight_P1"
_M.TREE_NODES_NUM = 192
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
    value = "1",
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
    child_num = 10,
    child_name_list = {
      "n00010_EnterCombatAction",
      "n00011_GetEntityAIData",
      "n00012_SetBlackboard",
      "n00013_RegisterActionBtree",
      "n00014_SetBlackboard",
      "n00015_SetVariableAction",
      "n00016_ApplyWanfaAction",
      "n00017_SetBlackboard",
      "n00018_SetBlackboard",
      "n00019_SetBlackboard"
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
    fast_node = "n00012_SetBlackboard",
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

function _M.n00012_SetBlackboard()
  return {
    node_name = "n00012_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n00013_RegisterActionBtree",
    idx_in_parent = 2,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n00013_RegisterActionBtree()
  return {
    node_name = "n00013_RegisterActionBtree",
    cls = AINodes.RegisterActionBtree,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "RegisterActionBtree",
    fast_node = "n00014_SetBlackboard",
    idx_in_parent = 3,
    btree_name = "51000025_CLK_fight_Action"
  }
end

function _M.n00014_SetBlackboard()
  return {
    node_name = "n00014_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n00015_SetVariableAction",
    idx_in_parent = 4,
    is_space = false,
    key = "_ACTION_FLOW_",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n00015_SetVariableAction()
  return {
    node_name = "n00015_SetVariableAction",
    cls = AINodes.SetVariableAction,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetVariableAction",
    fast_node = "n00016_ApplyWanfaAction",
    idx_in_parent = 5,
    set_type = "single",
    key_type = 2,
    key = "lock_type",
    value_type = 3,
    value = "1"
  }
end

function _M.n00016_ApplyWanfaAction()
  return {
    node_name = "n00016_ApplyWanfaAction",
    cls = AINodes.ApplyWanfaAction,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "ApplyWanfaAction",
    fast_node = "n00017_SetBlackboard",
    idx_in_parent = 6,
    wanfa_no = 51000025,
    sub_wanfa_ids = string.split("1", ","),
    leave_battle_end = true
  }
end

function _M.n00017_SetBlackboard()
  return {
    node_name = "n00017_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n00018_SetBlackboard",
    idx_in_parent = 7,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "1",
    reason = ""
  }
end

function _M.n00018_SetBlackboard()
  return {
    node_name = "n00018_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n00019_SetBlackboard",
    idx_in_parent = 8,
    is_space = false,
    key = "kengki_count",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n00019_SetBlackboard()
  return {
    node_name = "n00019_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 9,
    is_space = false,
    key = "attack_count",
    value_type = 3,
    value = "0",
    reason = ""
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
    child_num = 4,
    child_name_list = {
      "n020_CBTWhileNode",
      "n021_ApplyWatch",
      "n022_CBTWhileNode",
      "n023_ConditionPriorityNode"
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

function _M.n022_CBTWhileNode()
  return {
    node_name = "n022_CBTWhileNode",
    cls = AINodes.CBTWhileNode,
    parent_node_name = "n02_CBTParallelNode",
    cls_name = "CBTWhileNode",
    idx_in_parent = 2,
    child_num = 2,
    child_name_list = {
      "n0220_Wait",
      "n0221_CBTSequenceNode"
    }
  }
end

function _M.n0220_Wait()
  return {
    node_name = "n0220_Wait",
    cls = AINodes.Wait,
    parent_node_name = "n022_CBTWhileNode",
    cls_name = "Wait",
    succ_fast_node = "n0221_CBTSequenceNode",
    idx_in_parent = 0,
    duration = 0.06,
    value_type = 1,
    value = "",
    only_server = false
  }
end

function _M.n0221_CBTSequenceNode()
  return {
    node_name = "n0221_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n022_CBTWhileNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0220_Wait",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n02210_CompareWithPercent",
      "n02211_NotNode",
      "n02212_ApplyBuff"
    }
  }
end

function _M.n02210_CompareWithPercent()
  return {
    node_name = "n02210_CompareWithPercent",
    cls = AINodes.CompareWithPercent,
    parent_node_name = "n0221_CBTSequenceNode",
    cls_name = "CompareWithPercent",
    succ_fast_node = "n02211_NotNode",
    idx_in_parent = 0,
    left_value_type = 2,
    left_value = "HP",
    right_value_type = 2,
    right_value = "HP_MAX",
    percent_type = 3,
    percent = "65",
    operation = "<="
  }
end

function _M.n02211_NotNode()
  return {
    node_name = "n02211_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0221_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n02212_ApplyBuff",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n022110_CheckBuff"
    }
  }
end

function _M.n022110_CheckBuff()
  return {
    node_name = "n022110_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02211_NotNode",
    cls_name = "CheckBuff",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02212_ApplyBuff()
  return {
    node_name = "n02212_ApplyBuff",
    cls = AINodes.ApplyBuff,
    parent_node_name = "n0221_CBTSequenceNode",
    cls_name = "ApplyBuff",
    idx_in_parent = 2,
    op = "add",
    target_type = 1,
    target_list = "_T1",
    buff_id = "450097",
    buff_target = "",
    use_buff_level = false,
    buff_level_type = 1,
    buff_level_list = ""
  }
end

function _M.n023_ConditionPriorityNode()
  return {
    node_name = "n023_ConditionPriorityNode",
    node_type = 2,
    cls = AINodes.ConditionPriorityNode,
    parent_node_name = "n02_CBTParallelNode",
    cls_name = "ConditionPriorityNode",
    idx_in_parent = 3,
    child_num = 5,
    child_name_list = {
      "n0230_CBTAlwaysSequenceNode",
      "n0231_CBTAlwaysSequenceNode",
      "n0232_CBTSequenceNode",
      "n0233_CBTSequenceNode",
      "n0234_CBTAlwaysSequenceNode"
    },
    conditions = listlib.filter(string.split("0#AIComboCondition#51000025_CLK_P1_ComboCondition#CanComboCancel,0#CheckBuffCondition#self#with;450079,0#GedangCondition#ALL,0#BBCheckCondition#attack_count;<;int;3,-1", ","), function(e)
      return "" ~= e
    end),
    tick_rate = 1.0
  }
end

function _M.n0230_CBTAlwaysSequenceNode()
  return {
    node_name = "n0230_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n023_ConditionPriorityNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n02300_CBTSwitchCaseNode"
    }
  }
end

function _M.n02300_CBTSwitchCaseNode()
  return {
    node_name = "n02300_CBTSwitchCaseNode",
    cls = AINodes.CBTSwitchCaseNode,
    parent_node_name = "n0230_CBTAlwaysSequenceNode",
    cls_name = "CBTSwitchCaseNode",
    idx_in_parent = 0,
    child_num = 30,
    child_name_list = {
      "n023000_CBTSelectNode",
      "n023001_CBTSequenceNode",
      "n023002_CBTSequenceNode",
      "n023003_CBTSequenceNode",
      "n023004_CBTSequenceNode",
      "n023005_CBTSequenceNode",
      "n023006_CBTSequenceNode",
      "n023007_CBTSequenceNode",
      "n023008_CBTSequenceNode",
      "n023009_CBTSequenceNode",
      "n02300A_CBTSelectNode",
      "n02300B_CBTSelectNode",
      "n02300C_CBTSelectNode",
      "n02300D_CBTSelectNode",
      "n02300E_CBTSelectNode",
      "n02300F_CBTSequenceNode",
      "n02300G_CBTSelectNode",
      "n02300H_CBTSequenceNode",
      "n02300I_CBTSequenceNode",
      "n02300J_CBTSelectNode",
      "n02300K_CBTSequenceNode",
      "n02300L_CBTSelectNode",
      "n02300M_CBTSelectNode",
      "n02300N_CBTSelectNode",
      "n02300O_CBTSequenceNode",
      "n02300P_CBTSequenceNode",
      "n02300Q_CBTSequenceNode",
      "n02300R_CBTSequenceNode",
      "n02300S_CBTSequenceNode",
      "n02300T_CBTSelectNode"
    },
    key_type = 1,
    key = "clk_Combo",
    case_type = 3,
    cases_dict = {
      [1] = 0,
      [2] = 1,
      [3] = 2,
      [4] = 3,
      [5] = 4,
      [6] = 5,
      [7] = 6,
      [8] = 7,
      [9] = 8,
      [10] = 9,
      [11] = 10,
      [12] = 11,
      [13] = 12,
      [14] = 13,
      [15] = 14,
      [16] = 15,
      [17] = 16,
      [18] = 17,
      [19] = 18,
      [20] = 19,
      [21] = 20,
      [22] = 21,
      [23] = 22,
      [24] = 23,
      [25] = 24,
      [26] = 25,
      [27] = 26,
      [28] = 27,
      [29] = 28,
      [30] = 29
    }
  }
end

function _M.n023000_CBTSelectNode()
  return {
    node_name = "n023000_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n0230000_CBTSequenceNode",
      "n0230001_CBTSequenceNode",
      "n0230002_ApplyActionByCondition"
    }
  }
end

function _M.n0230000_CBTSequenceNode()
  return {
    node_name = "n0230000_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n023000_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n0230001_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n02300000_CheckTargetDis",
      "n02300001_CheckBuff",
      "n02300002_ApplyActionByCondition"
    }
  }
end

function _M.n02300000_CheckTargetDis()
  return {
    node_name = "n02300000_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n0230000_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02300001_CheckBuff",
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
    distance = "7",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02300001_CheckBuff()
  return {
    node_name = "n02300001_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n0230000_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n02300002_ApplyActionByCondition",
    idx_in_parent = 1,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02300002_ApplyActionByCondition()
  return {
    node_name = "n02300002_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0230000_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 2,
    random_list = listlib.filter(string.split("46,44,40,41", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("90,15,15,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n0230001_CBTSequenceNode()
  return {
    node_name = "n0230001_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n023000_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n0230002_ApplyActionByCondition",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02300010_CheckTargetDis",
      "n02300011_ApplyActionByCondition"
    }
  }
end

function _M.n02300010_CheckTargetDis()
  return {
    node_name = "n02300010_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n0230001_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02300011_ApplyActionByCondition",
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
    distance = "7",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02300011_ApplyActionByCondition()
  return {
    node_name = "n02300011_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0230001_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("41,44,40", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("70,15,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n0230002_ApplyActionByCondition()
  return {
    node_name = "n0230002_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n023000_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 2,
    random_list = listlib.filter(string.split("16,45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n023001_CBTSequenceNode()
  return {
    node_name = "n023001_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n0230010_ApplyActionByCondition"
    }
  }
end

function _M.n0230010_ApplyActionByCondition()
  return {
    node_name = "n0230010_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n023001_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("45,3,16,14", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,10,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n023002_CBTSequenceNode()
  return {
    node_name = "n023002_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n0230020_ApplyActionByCondition"
    }
  }
end

function _M.n0230020_ApplyActionByCondition()
  return {
    node_name = "n0230020_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n023002_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("17", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n023003_CBTSequenceNode()
  return {
    node_name = "n023003_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 3,
    child_num = 1,
    child_name_list = {
      "n0230030_ApplyActionByCondition"
    }
  }
end

function _M.n0230030_ApplyActionByCondition()
  return {
    node_name = "n0230030_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n023003_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("45,3,14,16", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,10,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n023004_CBTSequenceNode()
  return {
    node_name = "n023004_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 4,
    child_num = 1,
    child_name_list = {
      "n0230040_ApplyActionDirect"
    }
  }
end

function _M.n0230040_ApplyActionDirect()
  return {
    node_name = "n0230040_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n023004_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 34
  }
end

function _M.n023005_CBTSequenceNode()
  return {
    node_name = "n023005_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 5,
    child_num = 1,
    child_name_list = {
      "n0230050_ApplyActionDirect"
    }
  }
end

function _M.n0230050_ApplyActionDirect()
  return {
    node_name = "n0230050_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n023005_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 9
  }
end

function _M.n023006_CBTSequenceNode()
  return {
    node_name = "n023006_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 6,
    child_num = 1,
    child_name_list = {
      "n0230060_ApplyActionDirect"
    }
  }
end

function _M.n0230060_ApplyActionDirect()
  return {
    node_name = "n0230060_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n023006_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 8
  }
end

function _M.n023007_CBTSequenceNode()
  return {
    node_name = "n023007_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 7,
    child_num = 1,
    child_name_list = {
      "n0230070_ApplyActionDirect"
    }
  }
end

function _M.n0230070_ApplyActionDirect()
  return {
    node_name = "n0230070_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n023007_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 7
  }
end

function _M.n023008_CBTSequenceNode()
  return {
    node_name = "n023008_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 8,
    child_num = 1,
    child_name_list = {
      "n0230080_ApplyActionDirect"
    }
  }
end

function _M.n0230080_ApplyActionDirect()
  return {
    node_name = "n0230080_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n023008_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 6
  }
end

function _M.n023009_CBTSequenceNode()
  return {
    node_name = "n023009_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 9,
    child_num = 1,
    child_name_list = {
      "n0230090_ApplyActionDirect"
    }
  }
end

function _M.n0230090_ApplyActionDirect()
  return {
    node_name = "n0230090_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n023009_CBTSequenceNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 4
  }
end

function _M.n02300A_CBTSelectNode()
  return {
    node_name = "n02300A_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 10,
    child_num = 2,
    child_name_list = {
      "n02300A0_CBTSequenceNode",
      "n02300A1_ApplyActionByCondition"
    }
  }
end

function _M.n02300A0_CBTSequenceNode()
  return {
    node_name = "n02300A0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300A_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300A1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300A00_CheckBuff",
      "n02300A01_ApplyActionByCondition"
    }
  }
end

function _M.n02300A00_CheckBuff()
  return {
    node_name = "n02300A00_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02300A0_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n02300A01_ApplyActionByCondition",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02300A01_ApplyActionByCondition()
  return {
    node_name = "n02300A01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300A0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("38,39,41,44", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,25,25", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300A1_ApplyActionByCondition()
  return {
    node_name = "n02300A1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300A_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("38,39,19,18", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,25,25", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300B_CBTSelectNode()
  return {
    node_name = "n02300B_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 11,
    child_num = 2,
    child_name_list = {
      "n02300B0_CBTSequenceNode",
      "n02300B1_ApplyActionByCondition"
    }
  }
end

function _M.n02300B0_CBTSequenceNode()
  return {
    node_name = "n02300B0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300B_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300B1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300B00_CheckBuff",
      "n02300B01_ApplyActionByCondition"
    }
  }
end

function _M.n02300B00_CheckBuff()
  return {
    node_name = "n02300B00_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02300B0_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n02300B01_ApplyActionByCondition",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02300B01_ApplyActionByCondition()
  return {
    node_name = "n02300B01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300B0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("13,37,41,44", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,25,25", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300B1_ApplyActionByCondition()
  return {
    node_name = "n02300B1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300B_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("13,37,19,17", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50,25,25", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300C_CBTSelectNode()
  return {
    node_name = "n02300C_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 12,
    child_num = 1,
    child_name_list = {
      "n02300C0_ApplyActionByCondition"
    }
  }
end

function _M.n02300C0_ApplyActionByCondition()
  return {
    node_name = "n02300C0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300C_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("20", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300D_CBTSelectNode()
  return {
    node_name = "n02300D_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 13,
    child_num = 2,
    child_name_list = {
      "n02300D0_CBTSequenceNode",
      "n02300D1_ApplyActionDirect"
    }
  }
end

function _M.n02300D0_CBTSequenceNode()
  return {
    node_name = "n02300D0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300D_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300D1_ApplyActionDirect",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300D00_CheckTargetDis",
      "n02300D01_ApplyActionByCondition"
    }
  }
end

function _M.n02300D00_CheckTargetDis()
  return {
    node_name = "n02300D00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n02300D0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02300D01_ApplyActionByCondition",
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

function _M.n02300D01_ApplyActionByCondition()
  return {
    node_name = "n02300D01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300D0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("43", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300D1_ApplyActionDirect()
  return {
    node_name = "n02300D1_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02300D_CBTSelectNode",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 1,
    action_value = 16
  }
end

function _M.n02300E_CBTSelectNode()
  return {
    node_name = "n02300E_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 14,
    child_num = 1,
    child_name_list = {
      "n02300E0_ApplyActionByCondition"
    }
  }
end

function _M.n02300E0_ApplyActionByCondition()
  return {
    node_name = "n02300E0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300E_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("44", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300F_CBTSequenceNode()
  return {
    node_name = "n02300F_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 15,
    child_num = 1,
    child_name_list = {
      "n02300F0_ApplyActionByCondition"
    }
  }
end

function _M.n02300F0_ApplyActionByCondition()
  return {
    node_name = "n02300F0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300F_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("17", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300G_CBTSelectNode()
  return {
    node_name = "n02300G_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 16,
    child_num = 3,
    child_name_list = {
      "n02300G0_CBTSequenceNode",
      "n02300G1_CBTSequenceNode",
      "n02300G2_ApplyActionByCondition"
    }
  }
end

function _M.n02300G0_CBTSequenceNode()
  return {
    node_name = "n02300G0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300G_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300G1_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n02300G00_CheckTargetDis",
      "n02300G01_CheckBuff",
      "n02300G02_ApplyActionByCondition"
    }
  }
end

function _M.n02300G00_CheckTargetDis()
  return {
    node_name = "n02300G00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n02300G0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02300G01_CheckBuff",
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
    distance = "7",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02300G01_CheckBuff()
  return {
    node_name = "n02300G01_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02300G0_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n02300G02_ApplyActionByCondition",
    idx_in_parent = 1,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02300G02_ApplyActionByCondition()
  return {
    node_name = "n02300G02_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300G0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 2,
    random_list = listlib.filter(string.split("46,44,41", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("90,15,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300G1_CBTSequenceNode()
  return {
    node_name = "n02300G1_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300G_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300G2_ApplyActionByCondition",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02300G10_CheckTargetDis",
      "n02300G11_ApplyActionByCondition"
    }
  }
end

function _M.n02300G10_CheckTargetDis()
  return {
    node_name = "n02300G10_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n02300G1_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02300G11_ApplyActionByCondition",
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
    distance = "7",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n02300G11_ApplyActionByCondition()
  return {
    node_name = "n02300G11_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300G1_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("44,41", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300G2_ApplyActionByCondition()
  return {
    node_name = "n02300G2_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300G_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 2,
    random_list = listlib.filter(string.split("16,45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300H_CBTSequenceNode()
  return {
    node_name = "n02300H_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 17,
    child_num = 1,
    child_name_list = {
      "n02300H0_ApplyActionByCondition"
    }
  }
end

function _M.n02300H0_ApplyActionByCondition()
  return {
    node_name = "n02300H0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300H_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("45,18", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("30,70", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300I_CBTSequenceNode()
  return {
    node_name = "n02300I_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 18,
    child_num = 1,
    child_name_list = {
      "n02300I0_ApplyActionByCondition"
    }
  }
end

function _M.n02300I0_ApplyActionByCondition()
  return {
    node_name = "n02300I0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300I_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("13", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300J_CBTSelectNode()
  return {
    node_name = "n02300J_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 19,
    child_num = 2,
    child_name_list = {
      "n02300J0_CBTSequenceNode",
      "n02300J1_ApplyActionByCondition"
    }
  }
end

function _M.n02300J0_CBTSequenceNode()
  return {
    node_name = "n02300J0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300J_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300J1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300J00_CheckTargetDis",
      "n02300J01_ApplyActionByCondition"
    }
  }
end

function _M.n02300J00_CheckTargetDis()
  return {
    node_name = "n02300J00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n02300J0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02300J01_ApplyActionByCondition",
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
    operation = "<"
  }
end

function _M.n02300J01_ApplyActionByCondition()
  return {
    node_name = "n02300J01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300J0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("15", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300J1_ApplyActionByCondition()
  return {
    node_name = "n02300J1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300J_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("16,45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300K_CBTSequenceNode()
  return {
    node_name = "n02300K_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 20,
    child_num = 1,
    child_name_list = {
      "n02300K0_ApplyActionByCondition"
    }
  }
end

function _M.n02300K0_ApplyActionByCondition()
  return {
    node_name = "n02300K0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300K_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("20,18", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300L_CBTSelectNode()
  return {
    node_name = "n02300L_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 21,
    child_num = 2,
    child_name_list = {
      "n02300L0_CBTSequenceNode",
      "n02300L1_ApplyActionByCondition"
    }
  }
end

function _M.n02300L0_CBTSequenceNode()
  return {
    node_name = "n02300L0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300L_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300L1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300L00_CheckBuff",
      "n02300L01_ApplyActionByCondition"
    }
  }
end

function _M.n02300L00_CheckBuff()
  return {
    node_name = "n02300L00_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02300L0_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n02300L01_ApplyActionByCondition",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02300L01_ApplyActionByCondition()
  return {
    node_name = "n02300L01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300L0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("46,17,41", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("90,15,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300L1_ApplyActionByCondition()
  return {
    node_name = "n02300L1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300L_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("41,17", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("70,30", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300M_CBTSelectNode()
  return {
    node_name = "n02300M_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 22,
    child_num = 2,
    child_name_list = {
      "n02300M0_CBTSequenceNode",
      "n02300M1_ApplyActionByCondition"
    }
  }
end

function _M.n02300M0_CBTSequenceNode()
  return {
    node_name = "n02300M0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300M_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300M1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300M00_CheckBuff",
      "n02300M01_ApplyActionByCondition"
    }
  }
end

function _M.n02300M00_CheckBuff()
  return {
    node_name = "n02300M00_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02300M0_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n02300M01_ApplyActionByCondition",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02300M01_ApplyActionByCondition()
  return {
    node_name = "n02300M01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300M0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("46,40,41", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("90,15,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300M1_ApplyActionByCondition()
  return {
    node_name = "n02300M1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300M_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("44,41", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,25", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300N_CBTSelectNode()
  return {
    node_name = "n02300N_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 23,
    child_num = 2,
    child_name_list = {
      "n02300N0_CBTSequenceNode",
      "n02300N1_ApplyActionByCondition"
    }
  }
end

function _M.n02300N0_CBTSequenceNode()
  return {
    node_name = "n02300N0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300N_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300N1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300N00_CheckBuff",
      "n02300N01_ApplyActionByCondition"
    }
  }
end

function _M.n02300N00_CheckBuff()
  return {
    node_name = "n02300N00_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n02300N0_CBTSequenceNode",
    cls_name = "CheckBuff",
    succ_fast_node = "n02300N01_ApplyActionByCondition",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450097",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n02300N01_ApplyActionByCondition()
  return {
    node_name = "n02300N01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300N0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("46,40,41", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("90,15,15", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300N1_ApplyActionByCondition()
  return {
    node_name = "n02300N1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300N_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("41,40", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300O_CBTSequenceNode()
  return {
    node_name = "n02300O_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 24,
    child_num = 1,
    child_name_list = {
      "n02300O0_ApplyActionByCondition"
    }
  }
end

function _M.n02300O0_ApplyActionByCondition()
  return {
    node_name = "n02300O0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300O_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("16,45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300P_CBTSequenceNode()
  return {
    node_name = "n02300P_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 25,
    child_num = 1,
    child_name_list = {
      "n02300P0_ApplyActionByCondition"
    }
  }
end

function _M.n02300P0_ApplyActionByCondition()
  return {
    node_name = "n02300P0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300P_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("20,39", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300Q_CBTSequenceNode()
  return {
    node_name = "n02300Q_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 26,
    child_num = 1,
    child_name_list = {
      "n02300Q0_ApplyActionByCondition"
    }
  }
end

function _M.n02300Q0_ApplyActionByCondition()
  return {
    node_name = "n02300Q0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300Q_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("39,20", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300R_CBTSequenceNode()
  return {
    node_name = "n02300R_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 27,
    child_num = 1,
    child_name_list = {
      "n02300R0_Probability"
    }
  }
end

function _M.n02300R0_Probability()
  return {
    node_name = "n02300R0_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n02300R_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300R00_ApplyActionDirect",
      "n02300R01_ApplyActionByCondition"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = true
  }
end

function _M.n02300R00_ApplyActionDirect()
  return {
    node_name = "n02300R00_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02300R0_Probability",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 20
  }
end

function _M.n02300R01_ApplyActionByCondition()
  return {
    node_name = "n02300R01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300R0_Probability",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("18,45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300S_CBTSequenceNode()
  return {
    node_name = "n02300S_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 28,
    child_num = 1,
    child_name_list = {
      "n02300S0_ApplyActionByCondition"
    }
  }
end

function _M.n02300S0_ApplyActionByCondition()
  return {
    node_name = "n02300S0_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300S_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("20,42", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300T_CBTSelectNode()
  return {
    node_name = "n02300T_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n02300_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 29,
    child_num = 2,
    child_name_list = {
      "n02300T0_CBTSequenceNode",
      "n02300T1_ApplyActionByCondition"
    }
  }
end

function _M.n02300T0_CBTSequenceNode()
  return {
    node_name = "n02300T0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02300T_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02300T1_ApplyActionByCondition",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02300T00_CheckTargetDis",
      "n02300T01_ApplyActionByCondition"
    }
  }
end

function _M.n02300T00_CheckTargetDis()
  return {
    node_name = "n02300T00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n02300T0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n02300T01_ApplyActionByCondition",
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

function _M.n02300T01_ApplyActionByCondition()
  return {
    node_name = "n02300T01_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300T0_CBTSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("20,17", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n02300T1_ApplyActionByCondition()
  return {
    node_name = "n02300T1_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02300T_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("16,45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n0231_CBTAlwaysSequenceNode()
  return {
    node_name = "n0231_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n023_ConditionPriorityNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n02310_ApplyActionByCondition"
    }
  }
end

function _M.n02310_ApplyActionByCondition()
  return {
    node_name = "n02310_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0231_CBTAlwaysSequenceNode",
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

function _M.n0232_CBTSequenceNode()
  return {
    node_name = "n0232_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n023_ConditionPriorityNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 2,
    child_num = 2,
    child_name_list = {
      "n02320_CheckBlackboard",
      "n02321_Probability"
    }
  }
end

function _M.n02320_CheckBlackboard()
  return {
    node_name = "n02320_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0232_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n02321_Probability",
    idx_in_parent = 0,
    key_from = 1,
    key = "_GD_SKILL_ID",
    operator = "==",
    value_type = 3,
    value = "20901006"
  }
end

function _M.n02321_Probability()
  return {
    node_name = "n02321_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n0232_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n023210_ApplyActionDirect",
      "n023211_ApplyActionDirect"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = true
  }
end

function _M.n023210_ApplyActionDirect()
  return {
    node_name = "n023210_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02321_Probability",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 0,
    action_value = 6
  }
end

function _M.n023211_ApplyActionDirect()
  return {
    node_name = "n023211_ApplyActionDirect",
    cls = AINodes.ApplyActionDirect,
    parent_node_name = "n02321_Probability",
    cls_name = "ApplyActionDirect",
    idx_in_parent = 1,
    action_value = 7
  }
end

function _M.n0233_CBTSequenceNode()
  return {
    node_name = "n0233_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n023_ConditionPriorityNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 3,
    child_num = 2,
    child_name_list = {
      "n02330_ChooseTargetAction",
      "n02331_CBTSelectNode"
    }
  }
end

function _M.n02330_ChooseTargetAction()
  return {
    node_name = "n02330_ChooseTargetAction",
    cls = AINodes.ChooseTargetAction,
    parent_node_name = "n0233_CBTSequenceNode",
    cls_name = "ChooseTargetAction",
    succ_fast_node = "n02331_CBTSelectNode",
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

function _M.n02331_CBTSelectNode()
  return {
    node_name = "n02331_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0233_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 4,
    child_name_list = {
      "n023310_PassNode",
      "n023311_CheckDisAndAngleDecorator",
      "n023312_CheckDisAndAngleDecorator",
      "n023313_ApplyActionByCondition"
    }
  }
end

function _M.n023310_PassNode()
  return {
    node_name = "n023310_PassNode",
    node_type = 2,
    cls = AINodes.PassNode,
    parent_node_name = "n02331_CBTSelectNode",
    cls_name = "PassNode",
    fail_fast_node = "n023311_CheckDisAndAngleDecorator",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0233100_CBTAlwaysSequenceNode"
    },
    record_type = 1,
    total_pass_count = 1,
    pass_name = "",
    pass_group = "default"
  }
end

function _M.n0233100_CBTAlwaysSequenceNode()
  return {
    node_name = "n0233100_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n023310_PassNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02331000_LockAnimLRAction",
      "n02331001_ApplyActionByCondition"
    }
  }
end

function _M.n02331000_LockAnimLRAction()
  return {
    node_name = "n02331000_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n0233100_CBTAlwaysSequenceNode",
    cls_name = "LockAnimLRAction",
    fast_node = "n02331001_ApplyActionByCondition",
    idx_in_parent = 0,
    entity_bb_key = "_T1",
    move_type = 1,
    min_duration = "2",
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

function _M.n02331001_ApplyActionByCondition()
  return {
    node_name = "n02331001_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n0233100_CBTAlwaysSequenceNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 1,
    random_list = listlib.filter(string.split("45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("100", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n023311_CheckDisAndAngleDecorator()
  return {
    node_name = "n023311_CheckDisAndAngleDecorator",
    node_type = 2,
    cls = AINodes.CheckDisAndAngleDecorator,
    parent_node_name = "n02331_CBTSelectNode",
    cls_name = "CheckDisAndAngleDecorator",
    fail_fast_node = "n023312_CheckDisAndAngleDecorator",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n0233110_ApplyActionByCondition"
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

function _M.n0233110_ApplyActionByCondition()
  return {
    node_name = "n0233110_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n023311_CheckDisAndAngleDecorator",
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

function _M.n023312_CheckDisAndAngleDecorator()
  return {
    node_name = "n023312_CheckDisAndAngleDecorator",
    node_type = 2,
    cls = AINodes.CheckDisAndAngleDecorator,
    parent_node_name = "n02331_CBTSelectNode",
    cls_name = "CheckDisAndAngleDecorator",
    fail_fast_node = "n023313_ApplyActionByCondition",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n0233120_ApplyActionByCondition"
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

function _M.n0233120_ApplyActionByCondition()
  return {
    node_name = "n0233120_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n023312_CheckDisAndAngleDecorator",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 0,
    random_list = listlib.filter(string.split("3,14,16,45", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("10,50,50,50", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n023313_ApplyActionByCondition()
  return {
    node_name = "n023313_ApplyActionByCondition",
    cls = AINodes.ApplyActionByCondition,
    parent_node_name = "n02331_CBTSelectNode",
    cls_name = "ApplyActionByCondition",
    idx_in_parent = 3,
    random_list = listlib.filter(string.split("1,10,12,11,19", ","), function(x)
      return "" ~= x
    end),
    random_weights = listlib.filter(string.split("1,50,50,50,1", ","), function(x)
      return "" ~= x
    end)
  }
end

function _M.n0234_CBTAlwaysSequenceNode()
  return {
    node_name = "n0234_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n023_ConditionPriorityNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 4,
    child_num = 1,
    child_name_list = {
      "n02340_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n02340_CBTAlwaysSequenceNode()
  return {
    node_name = "n02340_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0234_CBTAlwaysSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n023400_CBTAlwaysSequenceNode",
      "n023401_SetBlackboard"
    }
  }
end

function _M.n023400_CBTAlwaysSequenceNode()
  return {
    node_name = "n023400_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n02340_CBTAlwaysSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    fast_node = "n023401_SetBlackboard",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0234000_CBTSequenceNode",
      "n0234001_CBTSelectNode"
    }
  }
end

function _M.n0234000_CBTSequenceNode()
  return {
    node_name = "n0234000_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n023400_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0234001_CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02340000_JudgeTargetInRangeAction",
      "n02340001_TurningAction"
    }
  }
end

function _M.n02340000_JudgeTargetInRangeAction()
  return {
    node_name = "n02340000_JudgeTargetInRangeAction",
    cls = AINodes.JudgeTargetInRangeAction,
    parent_node_name = "n0234000_CBTSequenceNode",
    cls_name = "JudgeTargetInRangeAction",
    succ_fast_node = "n02340001_TurningAction",
    idx_in_parent = 0,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "135.0",
    right_v = 225.0
  }
end

function _M.n02340001_TurningAction()
  return {
    node_name = "n02340001_TurningAction",
    cls = AINodes.TurningAction,
    parent_node_name = "n0234000_CBTSequenceNode",
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

function _M.n0234001_CBTSelectNode()
  return {
    node_name = "n0234001_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n023400_CBTAlwaysSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02340010_CBTSequenceNode",
      "n02340011_CBTSelectNode"
    }
  }
end

function _M.n02340010_CBTSequenceNode()
  return {
    node_name = "n02340010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0234001_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02340011_CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n023400100_CheckTimer",
      "n023400101_Probability"
    }
  }
end

function _M.n023400100_CheckTimer()
  return {
    node_name = "n023400100_CheckTimer",
    cls = AINodes.CheckTimer,
    parent_node_name = "n02340010_CBTSequenceNode",
    cls_name = "CheckTimer",
    succ_fast_node = "n023400101_Probability",
    idx_in_parent = 0,
    timer_key_type = 4,
    timer_key = "LockTimer2",
    is_pass_none_timer = true
  }
end

function _M.n023400101_Probability()
  return {
    node_name = "n023400101_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n02340010_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0234001010_CBTSequenceNode",
      "n0234001011_CBTSequenceNode"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = false
  }
end

function _M.n0234001010_CBTSequenceNode()
  return {
    node_name = "n0234001010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n023400101_Probability",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02340010100_CheckBlackboard",
      "n02340010101_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n02340010100_CheckBlackboard()
  return {
    node_name = "n02340010100_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0234001010_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n02340010101_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "1"
  }
end

function _M.n02340010101_CBTAlwaysSequenceNode()
  return {
    node_name = "n02340010101_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0234001010_CBTSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n023400101010_SetBlackboard",
      "n023400101011_SetTimer",
      "n023400101012_LockAnimLRAction"
    }
  }
end

function _M.n023400101010_SetBlackboard()
  return {
    node_name = "n023400101010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n02340010101_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n023400101011_SetTimer",
    idx_in_parent = 0,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n023400101011_SetTimer()
  return {
    node_name = "n023400101011_SetTimer",
    cls = AINodes.SetTimer,
    parent_node_name = "n02340010101_CBTAlwaysSequenceNode",
    cls_name = "SetTimer",
    fast_node = "n023400101012_LockAnimLRAction",
    idx_in_parent = 1,
    timer_key = "LockTimer2",
    duration_type = "random",
    duration = 1.0,
    duration_max = 2.0,
    duration_bb = ""
  }
end

function _M.n023400101012_LockAnimLRAction()
  return {
    node_name = "n023400101012_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02340010101_CBTAlwaysSequenceNode",
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

function _M.n0234001011_CBTSequenceNode()
  return {
    node_name = "n0234001011_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n023400101_Probability",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02340010110_CheckBlackboard",
      "n02340010111_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n02340010110_CheckBlackboard()
  return {
    node_name = "n02340010110_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0234001011_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n02340010111_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "0"
  }
end

function _M.n02340010111_CBTAlwaysSequenceNode()
  return {
    node_name = "n02340010111_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0234001011_CBTSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n023400101110_SetBlackboard",
      "n023400101111_SetTimer",
      "n023400101112_LockAnimLRAction"
    }
  }
end

function _M.n023400101110_SetBlackboard()
  return {
    node_name = "n023400101110_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n02340010111_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n023400101111_SetTimer",
    idx_in_parent = 0,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n023400101111_SetTimer()
  return {
    node_name = "n023400101111_SetTimer",
    cls = AINodes.SetTimer,
    parent_node_name = "n02340010111_CBTAlwaysSequenceNode",
    cls_name = "SetTimer",
    fast_node = "n023400101112_LockAnimLRAction",
    idx_in_parent = 1,
    timer_key = "LockTimer2",
    duration_type = "random",
    duration = 1.0,
    duration_max = 2.0,
    duration_bb = ""
  }
end

function _M.n023400101112_LockAnimLRAction()
  return {
    node_name = "n023400101112_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02340010111_CBTAlwaysSequenceNode",
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

function _M.n02340011_CBTSelectNode()
  return {
    node_name = "n02340011_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0234001_CBTSelectNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n023400110_CBTSequenceNode",
      "n023400111_CBTSequenceNode"
    }
  }
end

function _M.n023400110_CBTSequenceNode()
  return {
    node_name = "n023400110_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02340011_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n023400111_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0234001100_CheckBlackboard",
      "n0234001101_CBTSelectNode"
    }
  }
end

function _M.n0234001100_CheckBlackboard()
  return {
    node_name = "n0234001100_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n023400110_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n0234001101_CBTSelectNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "0"
  }
end

function _M.n0234001101_CBTSelectNode()
  return {
    node_name = "n0234001101_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n023400110_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02340011010_CBTSequenceNode",
      "n02340011011_LockAnimLRAction"
    }
  }
end

function _M.n02340011010_CBTSequenceNode()
  return {
    node_name = "n02340011010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0234001101_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02340011011_LockAnimLRAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n023400110100_CheckObstacle",
      "n023400110101_LockAnimLRAction"
    }
  }
end

function _M.n023400110100_CheckObstacle()
  return {
    node_name = "n023400110100_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n02340011010_CBTSequenceNode",
    cls_name = "CheckObstacle",
    succ_fast_node = "n023400110101_LockAnimLRAction",
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

function _M.n023400110101_LockAnimLRAction()
  return {
    node_name = "n023400110101_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02340011010_CBTSequenceNode",
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

function _M.n02340011011_LockAnimLRAction()
  return {
    node_name = "n02340011011_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n0234001101_CBTSelectNode",
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

function _M.n023400111_CBTSequenceNode()
  return {
    node_name = "n023400111_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n02340011_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0234001110_CheckBlackboard",
      "n0234001111_CBTSelectNode"
    }
  }
end

function _M.n0234001110_CheckBlackboard()
  return {
    node_name = "n0234001110_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n023400111_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n0234001111_CBTSelectNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "1"
  }
end

function _M.n0234001111_CBTSelectNode()
  return {
    node_name = "n0234001111_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n023400111_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n02340011110_CBTSequenceNode",
      "n02340011111_LockAnimLRAction"
    }
  }
end

function _M.n02340011110_CBTSequenceNode()
  return {
    node_name = "n02340011110_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0234001111_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n02340011111_LockAnimLRAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n023400111100_CheckObstacle",
      "n023400111101_LockAnimLRAction"
    }
  }
end

function _M.n023400111100_CheckObstacle()
  return {
    node_name = "n023400111100_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n02340011110_CBTSequenceNode",
    cls_name = "CheckObstacle",
    succ_fast_node = "n023400111101_LockAnimLRAction",
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

function _M.n023400111101_LockAnimLRAction()
  return {
    node_name = "n023400111101_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n02340011110_CBTSequenceNode",
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

function _M.n02340011111_LockAnimLRAction()
  return {
    node_name = "n02340011111_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n0234001111_CBTSelectNode",
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

function _M.n023401_SetBlackboard()
  return {
    node_name = "n023401_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n02340_CBTAlwaysSequenceNode",
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
