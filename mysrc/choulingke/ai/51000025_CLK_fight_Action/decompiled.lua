local AINodes = require("hexm.common.AI.nodes.ai_nodes").AINodes
local _M = {}
_M.TREE_NAME = "51000025_CLK_fight_Action"
_M.TREE_NODES_NUM = 192
_M.NODE_DATA_CACHE = {}

function _M.n0_CBTSwitchCaseNode()
  return {
    node_name = "n0_CBTSwitchCaseNode",
    cls = AINodes.CBTSwitchCaseNode,
    cls_name = "CBTSwitchCaseNode",
    child_num = 60,
    child_name_list = {
      "n00_CBTSelectNode",
      "n01_CBTSelectNode",
      "n02_CBTSelectNode",
      "n03_CBTAlwaysSequenceNode",
      "n04_CBTAlwaysSequenceNode",
      "n05_CBTSequenceNode",
      "n06_CBTAlwaysSequenceNode",
      "n07_CBTSequenceNode",
      "n08_CBTAlwaysSequenceNode",
      "n09_CBTAlwaysSequenceNode",
      "n0A_CBTAlwaysSequenceNode",
      "n0B_CBTAlwaysSequenceNode",
      "n0C_CBTAlwaysSequenceNode",
      "n0D_CBTAlwaysSequenceNode",
      "n0E_CBTAlwaysSequenceNode",
      "n0F_CBTAlwaysSequenceNode",
      "n0G_CBTAlwaysSequenceNode",
      "n0H_CBTAlwaysSequenceNode",
      "n0I_CBTAlwaysSequenceNode",
      "n0J_CBTAlwaysSequenceNode",
      "n0K_CBTAlwaysSequenceNode",
      "n0L_CBTAlwaysSequenceNode",
      "n0M_CBTAlwaysSequenceNode",
      "n0N_CBTAlwaysSequenceNode",
      "n0O_CBTAlwaysSequenceNode",
      "n0P_CBTAlwaysSequenceNode",
      "n0Q_CBTAlwaysSequenceNode",
      "n0R_CBTAlwaysSequenceNode",
      "n0S_CBTAlwaysSequenceNode",
      "n0T_CBTAlwaysSequenceNode",
      "n0U_CBTAlwaysSequenceNode",
      "n0V_CBTAlwaysSequenceNode",
      "n0W_CBTAlwaysSequenceNode",
      "n0X_CBTAlwaysSequenceNode",
      "n0Y_CBTAlwaysSequenceNode",
      "n0Z_CBTAlwaysSequenceNode",
      "n0a_CBTAlwaysSequenceNode",
      "n0b_CBTAlwaysSequenceNode",
      "n0c_CBTAlwaysSequenceNode",
      "n0d_CBTAlwaysSequenceNode",
      "n0e_CBTAlwaysSequenceNode",
      "n0f_CBTAlwaysSequenceNode",
      "n0g_CBTAlwaysSequenceNode",
      "n0h_CBTAlwaysSequenceNode",
      "n0i_CBTAlwaysSequenceNode",
      "n0j_CBTAlwaysSequenceNode",
      "n0k_CBTAlwaysSequenceNode",
      "n0l_CBTAlwaysSequenceNode",
      "n0m_CBTAlwaysSequenceNode",
      "n0n_CBTAlwaysSequenceNode",
      "n0o_CBTAlwaysSequenceNode",
      "n0p_CBTAlwaysSequenceNode",
      "n0q_CBTAlwaysSequenceNode",
      "n0r_CBTAlwaysSequenceNode",
      "n0s_CBTAlwaysSequenceNode",
      "n0t_CBTAlwaysSequenceNode",
      "n0u_CBTAlwaysSequenceNode",
      "n0v_CBTAlwaysSequenceNode",
      "n0w_CBTAlwaysSequenceNode",
      "n0x_CBTAlwaysSequenceNode"
    },
    key_type = 1,
    key = "_ACTION_FLOW_",
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
      [30] = 29,
      [31] = 30,
      [32] = 31,
      [33] = 32,
      [34] = 33,
      [35] = 34,
      [36] = 35,
      [37] = 36,
      [38] = 37,
      [39] = 38,
      [40] = 39,
      [41] = 40,
      [42] = 41,
      [43] = 42,
      [44] = 43,
      [45] = 44,
      [46] = 45,
      [47] = 46,
      [48] = 47,
      [49] = 48,
      [50] = 49,
      [51] = 50,
      [52] = 51,
      [53] = 52,
      [54] = 53,
      [55] = 54,
      [56] = 55,
      [57] = 56,
      [58] = 57,
      [59] = 58,
      [60] = 59
    }
  }
end

function _M.n00_CBTSelectNode()
  return {
    node_name = "n00_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n000_CBTAlwaysSequenceNode()
  return {
    node_name = "n000_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n00_CBTSelectNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0000_CBTSequenceNode",
      "n0001_CBTSelectNode"
    }
  }
end

function _M.n0000_CBTSequenceNode()
  return {
    node_name = "n0000_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0001_CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n00000_JudgeTargetInRangeAction",
      "n00001_TurningAction"
    }
  }
end

function _M.n00000_JudgeTargetInRangeAction()
  return {
    node_name = "n00000_JudgeTargetInRangeAction",
    cls = AINodes.JudgeTargetInRangeAction,
    parent_node_name = "n0000_CBTSequenceNode",
    cls_name = "JudgeTargetInRangeAction",
    succ_fast_node = "n00001_TurningAction",
    idx_in_parent = 0,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "135.0",
    right_v = 225.0
  }
end

function _M.n00001_TurningAction()
  return {
    node_name = "n00001_TurningAction",
    cls = AINodes.TurningAction,
    parent_node_name = "n0000_CBTSequenceNode",
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

function _M.n0001_CBTSelectNode()
  return {
    node_name = "n0001_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTAlwaysSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n00010_CBTSequenceNode",
      "n00011_CBTSelectNode"
    }
  }
end

function _M.n00010_CBTSequenceNode()
  return {
    node_name = "n00010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0001_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n00011_CBTSelectNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000100_CheckTimer",
      "n000101_Probability"
    }
  }
end

function _M.n000100_CheckTimer()
  return {
    node_name = "n000100_CheckTimer",
    cls = AINodes.CheckTimer,
    parent_node_name = "n00010_CBTSequenceNode",
    cls_name = "CheckTimer",
    succ_fast_node = "n000101_Probability",
    idx_in_parent = 0,
    timer_key_type = 4,
    timer_key = "LockTimer",
    is_pass_none_timer = true
  }
end

function _M.n000101_Probability()
  return {
    node_name = "n000101_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n00010_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0001010_CBTSequenceNode",
      "n0001011_CBTSequenceNode"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = false
  }
end

function _M.n0001010_CBTSequenceNode()
  return {
    node_name = "n0001010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000101_Probability",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n00010100_CheckBlackboard",
      "n00010101_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n00010100_CheckBlackboard()
  return {
    node_name = "n00010100_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0001010_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n00010101_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "1"
  }
end

function _M.n00010101_CBTAlwaysSequenceNode()
  return {
    node_name = "n00010101_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0001010_CBTSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n000101010_SetBlackboard",
      "n000101011_SetTimer",
      "n000101012_LockAnimLRAction"
    }
  }
end

function _M.n000101010_SetBlackboard()
  return {
    node_name = "n000101010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00010101_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n000101011_SetTimer",
    idx_in_parent = 0,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n000101011_SetTimer()
  return {
    node_name = "n000101011_SetTimer",
    cls = AINodes.SetTimer,
    parent_node_name = "n00010101_CBTAlwaysSequenceNode",
    cls_name = "SetTimer",
    fast_node = "n000101012_LockAnimLRAction",
    idx_in_parent = 1,
    timer_key = "LockTimer",
    duration_type = "random",
    duration = 4.0,
    duration_max = 5.0,
    duration_bb = ""
  }
end

function _M.n000101012_LockAnimLRAction()
  return {
    node_name = "n000101012_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n00010101_CBTAlwaysSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 2,
    entity_bb_key = "_T1",
    move_type = 1,
    min_duration = "3",
    max_duration = "3",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "3",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n0001011_CBTSequenceNode()
  return {
    node_name = "n0001011_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000101_Probability",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n00010110_CheckBlackboard",
      "n00010111_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n00010110_CheckBlackboard()
  return {
    node_name = "n00010110_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n0001011_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n00010111_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "0"
  }
end

function _M.n00010111_CBTAlwaysSequenceNode()
  return {
    node_name = "n00010111_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0001011_CBTSequenceNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 3,
    child_name_list = {
      "n000101110_SetBlackboard",
      "n000101111_SetTimer",
      "n000101112_LockAnimLRAction"
    }
  }
end

function _M.n000101110_SetBlackboard()
  return {
    node_name = "n000101110_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00010111_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n000101111_SetTimer",
    idx_in_parent = 0,
    is_space = false,
    key = "lastlockdirection",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n000101111_SetTimer()
  return {
    node_name = "n000101111_SetTimer",
    cls = AINodes.SetTimer,
    parent_node_name = "n00010111_CBTAlwaysSequenceNode",
    cls_name = "SetTimer",
    fast_node = "n000101112_LockAnimLRAction",
    idx_in_parent = 1,
    timer_key = "LockTimer",
    duration_type = "random",
    duration = 4.0,
    duration_max = 5.0,
    duration_bb = ""
  }
end

function _M.n000101112_LockAnimLRAction()
  return {
    node_name = "n000101112_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n00010111_CBTAlwaysSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 2,
    entity_bb_key = "_T1",
    move_type = 0,
    min_duration = "3",
    max_duration = "3",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "3",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n00011_CBTSelectNode()
  return {
    node_name = "n00011_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0001_CBTSelectNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n000110_CBTSequenceNode",
      "n000111_CBTSequenceNode"
    }
  }
end

function _M.n000110_CBTSequenceNode()
  return {
    node_name = "n000110_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n00011_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n000111_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0001100_CheckBlackboard",
      "n0001101_CBTSelectNode"
    }
  }
end

function _M.n0001100_CheckBlackboard()
  return {
    node_name = "n0001100_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n000110_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n0001101_CBTSelectNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "0"
  }
end

function _M.n0001101_CBTSelectNode()
  return {
    node_name = "n0001101_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000110_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n00011010_CBTSequenceNode",
      "n00011011_LockAnimLRAction"
    }
  }
end

function _M.n00011010_CBTSequenceNode()
  return {
    node_name = "n00011010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0001101_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n00011011_LockAnimLRAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000110100_CheckObstacle",
      "n000110101_LockAnimLRAction"
    }
  }
end

function _M.n000110100_CheckObstacle()
  return {
    node_name = "n000110100_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n00011010_CBTSequenceNode",
    cls_name = "CheckObstacle",
    succ_fast_node = "n000110101_LockAnimLRAction",
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

function _M.n000110101_LockAnimLRAction()
  return {
    node_name = "n000110101_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n00011010_CBTSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 0,
    min_duration = "3",
    max_duration = "3",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "3",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n00011011_LockAnimLRAction()
  return {
    node_name = "n00011011_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n0001101_CBTSelectNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 1,
    min_duration = "3",
    max_duration = "3",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "3",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n000111_CBTSequenceNode()
  return {
    node_name = "n000111_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n00011_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0001110_CheckBlackboard",
      "n0001111_CBTSelectNode"
    }
  }
end

function _M.n0001110_CheckBlackboard()
  return {
    node_name = "n0001110_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n000111_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n0001111_CBTSelectNode",
    idx_in_parent = 0,
    key_from = 1,
    key = "lastlockdirection",
    operator = "==",
    value_type = 3,
    value = "1"
  }
end

function _M.n0001111_CBTSelectNode()
  return {
    node_name = "n0001111_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000111_CBTSequenceNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n00011110_CBTSequenceNode",
      "n00011111_LockAnimLRAction"
    }
  }
end

function _M.n00011110_CBTSequenceNode()
  return {
    node_name = "n00011110_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0001111_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n00011111_LockAnimLRAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000111100_CheckObstacle",
      "n000111101_LockAnimLRAction"
    }
  }
end

function _M.n000111100_CheckObstacle()
  return {
    node_name = "n000111100_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n00011110_CBTSequenceNode",
    cls_name = "CheckObstacle",
    succ_fast_node = "n000111101_LockAnimLRAction",
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

function _M.n000111101_LockAnimLRAction()
  return {
    node_name = "n000111101_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n00011110_CBTSequenceNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 1,
    min_duration = "3",
    max_duration = "3",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "3",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n00011111_LockAnimLRAction()
  return {
    node_name = "n00011111_LockAnimLRAction",
    cls = AINodes.LockAnimLRAction,
    parent_node_name = "n0001111_CBTSelectNode",
    cls_name = "LockAnimLRAction",
    idx_in_parent = 1,
    entity_bb_key = "_T1",
    move_type = 0,
    min_duration = "3",
    max_duration = "3",
    lock_speed_type = 0,
    move_speed_rate = "0.7",
    move_speed_default = "1",
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 1.0,
    move_anim_y_not_move_up = true,
    is_spec_entity = false,
    timeout = "3",
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false
  }
end

function _M.n01_CBTSelectNode()
  return {
    node_name = "n01_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n010_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n010_CBTAlwaysSequenceNode()
  return {
    node_name = "n010_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n01_CBTSelectNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0100_CBTSequenceNode",
      "n0101_LockAnimDistanceAction"
    }
  }
end

function _M.n0100_CBTSequenceNode()
  return {
    node_name = "n0100_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n010_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0101_LockAnimDistanceAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n01000_JudgeTargetInRangeAction",
      "n01001_TurningAction"
    }
  }
end

function _M.n01000_JudgeTargetInRangeAction()
  return {
    node_name = "n01000_JudgeTargetInRangeAction",
    cls = AINodes.JudgeTargetInRangeAction,
    parent_node_name = "n0100_CBTSequenceNode",
    cls_name = "JudgeTargetInRangeAction",
    succ_fast_node = "n01001_TurningAction",
    idx_in_parent = 0,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "135.0",
    right_v = 225.0
  }
end

function _M.n01001_TurningAction()
  return {
    node_name = "n01001_TurningAction",
    cls = AINodes.TurningAction,
    parent_node_name = "n0100_CBTSequenceNode",
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

function _M.n0101_LockAnimDistanceAction()
  return {
    node_name = "n0101_LockAnimDistanceAction",
    cls = AINodes.LockAnimDistanceAction,
    parent_node_name = "n010_CBTAlwaysSequenceNode",
    cls_name = "LockAnimDistanceAction",
    idx_in_parent = 1,
    use_navigate = true,
    use_input_navigate = false,
    server_boss_allow_navigate = false,
    entity_bb_key = "_T1",
    distance_mode = 0,
    distance_value = 6.0,
    distance_bb_key = "",
    target_hater_check = true,
    target_hater_num = 3,
    target_point_mode = 0,
    enable_half_angle_bb = false,
    half_angle_bb = "",
    half_angle = 0.0,
    half_angle_lr = 1,
    float_distance = 0.0,
    float_angle = 0.0,
    keep_end_yaw = false,
    turn_speed = -1.0,
    forward_anim_type = 1,
    move_speed_rate = "1.0",
    move_speed_default = "1.0",
    lock_end_anim = true,
    lock_enter_transit_time = -1.0,
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 0.5,
    timeout = "5",
    timeout_flag = true,
    enable_ai_combat_controller = false,
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false,
    lock_angle_threshold = -1.0
  }
end

function _M.n02_CBTSelectNode()
  return {
    node_name = "n02_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n020_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n020_CBTAlwaysSequenceNode()
  return {
    node_name = "n020_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n02_CBTSelectNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0200_CBTSequenceNode",
      "n0201_LockAnimDistanceAction"
    }
  }
end

function _M.n0200_CBTSequenceNode()
  return {
    node_name = "n0200_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n020_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0201_LockAnimDistanceAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n02000_JudgeTargetInRangeAction",
      "n02001_TurningAction"
    }
  }
end

function _M.n02000_JudgeTargetInRangeAction()
  return {
    node_name = "n02000_JudgeTargetInRangeAction",
    cls = AINodes.JudgeTargetInRangeAction,
    parent_node_name = "n0200_CBTSequenceNode",
    cls_name = "JudgeTargetInRangeAction",
    succ_fast_node = "n02001_TurningAction",
    idx_in_parent = 0,
    from_type = 0,
    from_value = "",
    to_type = 1,
    to_value = "_T1",
    left_v = "135.0",
    right_v = 225.0
  }
end

function _M.n02001_TurningAction()
  return {
    node_name = "n02001_TurningAction",
    cls = AINodes.TurningAction,
    parent_node_name = "n0200_CBTSequenceNode",
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

function _M.n0201_LockAnimDistanceAction()
  return {
    node_name = "n0201_LockAnimDistanceAction",
    cls = AINodes.LockAnimDistanceAction,
    parent_node_name = "n020_CBTAlwaysSequenceNode",
    cls_name = "LockAnimDistanceAction",
    idx_in_parent = 1,
    use_navigate = true,
    use_input_navigate = false,
    server_boss_allow_navigate = false,
    entity_bb_key = "_T1",
    distance_mode = 0,
    distance_value = 5.0,
    distance_bb_key = "",
    target_hater_check = true,
    target_hater_num = 3,
    target_point_mode = 0,
    enable_half_angle_bb = false,
    half_angle_bb = "",
    half_angle = 0.0,
    half_angle_lr = 1,
    float_distance = 0.0,
    float_angle = 0.0,
    keep_end_yaw = false,
    turn_speed = -1.0,
    forward_anim_type = 0,
    move_speed_rate = "1.0",
    move_speed_default = "1.0",
    lock_end_anim = false,
    lock_enter_transit_time = -1.0,
    enable_detect_block = true,
    detect_block_interval = 1.0,
    detect_block_threshold = 0.5,
    timeout = "3",
    timeout_flag = true,
    enable_ai_combat_controller = false,
    ai_behit_state = true,
    tag_id = "lock_behit",
    only_one_behit = false,
    lock_angle_threshold = -1.0
  }
end

function _M.n03_CBTAlwaysSequenceNode()
  return {
    node_name = "n03_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 3,
    child_num = 1,
    child_name_list = {
      "n030_ApplySkillAction"
    }
  }
end

function _M.n030_ApplySkillAction()
  return {
    node_name = "n030_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n03_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021629,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n04_CBTAlwaysSequenceNode()
  return {
    node_name = "n04_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 4,
    child_num = 1,
    child_name_list = {
      "n040_ApplySkillAction"
    }
  }
end

function _M.n040_ApplySkillAction()
  return {
    node_name = "n040_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n04_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021435,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n05_CBTSequenceNode()
  return {
    node_name = "n05_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 5,
    child_num = 1,
    child_name_list = {
      "n050_ApplySkillAction"
    }
  }
end

function _M.n050_ApplySkillAction()
  return {
    node_name = "n050_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n05_CBTSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021632,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 27,
    action_flow_time = 25.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n06_CBTAlwaysSequenceNode()
  return {
    node_name = "n06_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 6,
    child_num = 1,
    child_name_list = {
      "n060_ApplySkillAction"
    }
  }
end

function _M.n060_ApplySkillAction()
  return {
    node_name = "n060_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n06_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021633,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n07_CBTSequenceNode()
  return {
    node_name = "n07_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 7,
    child_num = 1,
    child_name_list = {
      "n070_ApplySkillAction"
    }
  }
end

function _M.n070_ApplySkillAction()
  return {
    node_name = "n070_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n07_CBTSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021626,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "AIComboSkillClass",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 27,
    action_flow_time = 25.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n08_CBTAlwaysSequenceNode()
  return {
    node_name = "n08_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 8,
    child_num = 1,
    child_name_list = {
      "n080_ApplySkillAction"
    }
  }
end

function _M.n080_ApplySkillAction()
  return {
    node_name = "n080_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n08_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021627,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "AIComboSkillClass",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n09_CBTAlwaysSequenceNode()
  return {
    node_name = "n09_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 9,
    child_num = 2,
    child_name_list = {
      "n090_UpdateBlackboard",
      "n091_ApplySkillAction"
    }
  }
end

function _M.n090_UpdateBlackboard()
  return {
    node_name = "n090_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n09_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n091_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n091_ApplySkillAction()
  return {
    node_name = "n091_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n09_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021593,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 10,
    action_flow_time = 40.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0A_CBTAlwaysSequenceNode()
  return {
    node_name = "n0A_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 10,
    child_num = 2,
    child_name_list = {
      "n0A0_UpdateBlackboard",
      "n0A1_ApplySkillAction"
    }
  }
end

function _M.n0A0_UpdateBlackboard()
  return {
    node_name = "n0A0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0A_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0A1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0A1_ApplySkillAction()
  return {
    node_name = "n0A1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0A_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021594,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 11,
    action_flow_time = 55.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0B_CBTAlwaysSequenceNode()
  return {
    node_name = "n0B_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 11,
    child_num = 2,
    child_name_list = {
      "n0B0_CBTSequenceNode",
      "n0B1_ApplySkillAction"
    }
  }
end

function _M.n0B0_CBTSequenceNode()
  return {
    node_name = "n0B0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0B_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0B1_ApplySkillAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0B00_NotNode",
      "n0B01_UpdateBlackboard"
    }
  }
end

function _M.n0B00_NotNode()
  return {
    node_name = "n0B00_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0B0_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n0B01_UpdateBlackboard",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0B000_CheckBuff"
    }
  }
end

function _M.n0B000_CheckBuff()
  return {
    node_name = "n0B000_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n0B00_NotNode",
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

function _M.n0B01_UpdateBlackboard()
  return {
    node_name = "n0B01_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0B0_CBTSequenceNode",
    cls_name = "UpdateBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0B1_ApplySkillAction()
  return {
    node_name = "n0B1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0B_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021595,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 12,
    action_flow_time = 50.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0C_CBTAlwaysSequenceNode()
  return {
    node_name = "n0C_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 12,
    child_num = 2,
    child_name_list = {
      "n0C0_UpdateBlackboard",
      "n0C1_ApplySkillAction"
    }
  }
end

function _M.n0C0_UpdateBlackboard()
  return {
    node_name = "n0C0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0C_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0C1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "kengki_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0C1_ApplySkillAction()
  return {
    node_name = "n0C1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0C_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021596,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 13,
    action_flow_time = 65.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0D_CBTAlwaysSequenceNode()
  return {
    node_name = "n0D_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 13,
    child_num = 1,
    child_name_list = {
      "n0D0_ApplySkillAction"
    }
  }
end

function _M.n0D0_ApplySkillAction()
  return {
    node_name = "n0D0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0D_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021597,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 14,
    action_flow_time = 60.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0E_CBTAlwaysSequenceNode()
  return {
    node_name = "n0E_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 14,
    child_num = 1,
    child_name_list = {
      "n0E0_ApplySkillAction"
    }
  }
end

function _M.n0E0_ApplySkillAction()
  return {
    node_name = "n0E0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0E_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021598,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0F_CBTAlwaysSequenceNode()
  return {
    node_name = "n0F_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 15,
    child_num = 1,
    child_name_list = {
      "n0F0_ApplySkillAction"
    }
  }
end

function _M.n0F0_ApplySkillAction()
  return {
    node_name = "n0F0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0F_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021599,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 16,
    action_flow_time = 50.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0G_CBTAlwaysSequenceNode()
  return {
    node_name = "n0G_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 16,
    child_num = 1,
    child_name_list = {
      "n0G0_ApplySkillAction"
    }
  }
end

function _M.n0G0_ApplySkillAction()
  return {
    node_name = "n0G0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0G_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021600,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 17,
    action_flow_time = 60.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0H_CBTAlwaysSequenceNode()
  return {
    node_name = "n0H_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 17,
    child_num = 1,
    child_name_list = {
      "n0H0_ApplySkillAction"
    }
  }
end

function _M.n0H0_ApplySkillAction()
  return {
    node_name = "n0H0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0H_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021601,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 18,
    action_flow_time = 60.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0I_CBTAlwaysSequenceNode()
  return {
    node_name = "n0I_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 18,
    child_num = 2,
    child_name_list = {
      "n0I0_NotNode",
      "n0I1_ApplySkillAction"
    }
  }
end

function _M.n0I0_NotNode()
  return {
    node_name = "n0I0_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0I_CBTAlwaysSequenceNode",
    cls_name = "NotNode",
    fast_node = "n0I1_ApplySkillAction",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0I00_CheckObstacle"
    }
  }
end

function _M.n0I00_CheckObstacle()
  return {
    node_name = "n0I00_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n0I0_NotNode",
    cls_name = "CheckObstacle",
    idx_in_parent = 0,
    bb_target = "_T1",
    direction = 3,
    target_bone_name = "Bip001 Spine1",
    target_ent_bone_name = "Scene Root",
    distance = 5.0,
    collision_type = 39,
    ignore_self = true,
    draw_line_flag = false,
    use_sweep = false,
    use_target_ent = false
  }
end

function _M.n0I1_ApplySkillAction()
  return {
    node_name = "n0I1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0I_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021602,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 19,
    action_flow_time = 55.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0J_CBTAlwaysSequenceNode()
  return {
    node_name = "n0J_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 19,
    child_num = 1,
    child_name_list = {
      "n0J0_ApplySkillAction"
    }
  }
end

function _M.n0J0_ApplySkillAction()
  return {
    node_name = "n0J0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0J_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021603,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 20,
    action_flow_time = 70.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0K_CBTAlwaysSequenceNode()
  return {
    node_name = "n0K_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 20,
    child_num = 1,
    child_name_list = {
      "n0K0_ApplySkillAction"
    }
  }
end

function _M.n0K0_ApplySkillAction()
  return {
    node_name = "n0K0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0K_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021604,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 21,
    action_flow_time = 65.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0L_CBTAlwaysSequenceNode()
  return {
    node_name = "n0L_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 21,
    child_num = 1,
    child_name_list = {
      "n0L0_ApplySkillAction"
    }
  }
end

function _M.n0L0_ApplySkillAction()
  return {
    node_name = "n0L0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0L_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021605,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0M_CBTAlwaysSequenceNode()
  return {
    node_name = "n0M_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 22,
    child_num = 1,
    child_name_list = {
      "n0M0_ApplySkillAction"
    }
  }
end

function _M.n0M0_ApplySkillAction()
  return {
    node_name = "n0M0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0M_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021606,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 23,
    action_flow_time = 80.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0N_CBTAlwaysSequenceNode()
  return {
    node_name = "n0N_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 23,
    child_num = 1,
    child_name_list = {
      "n0N0_ApplySkillAction"
    }
  }
end

function _M.n0N0_ApplySkillAction()
  return {
    node_name = "n0N0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0N_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021607,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 24,
    action_flow_time = 65.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0O_CBTAlwaysSequenceNode()
  return {
    node_name = "n0O_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 24,
    child_num = 1,
    child_name_list = {
      "n0O0_ApplySkillAction"
    }
  }
end

function _M.n0O0_ApplySkillAction()
  return {
    node_name = "n0O0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0O_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021608,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 25,
    action_flow_time = 55.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0P_CBTAlwaysSequenceNode()
  return {
    node_name = "n0P_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 25,
    child_num = 1,
    child_name_list = {
      "n0P0_ApplySkillAction"
    }
  }
end

function _M.n0P0_ApplySkillAction()
  return {
    node_name = "n0P0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0P_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021609,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0Q_CBTAlwaysSequenceNode()
  return {
    node_name = "n0Q_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 26,
    child_num = 2,
    child_name_list = {
      "n0Q0_UpdateBlackboard",
      "n0Q1_ApplySkillAction"
    }
  }
end

function _M.n0Q0_UpdateBlackboard()
  return {
    node_name = "n0Q0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0Q_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0Q1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0Q1_ApplySkillAction()
  return {
    node_name = "n0Q1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0Q_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021610,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 27,
    action_flow_time = 75.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0R_CBTAlwaysSequenceNode()
  return {
    node_name = "n0R_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 27,
    child_num = 1,
    child_name_list = {
      "n0R0_ApplySkillAction"
    }
  }
end

function _M.n0R0_ApplySkillAction()
  return {
    node_name = "n0R0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0R_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021611,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0S_CBTAlwaysSequenceNode()
  return {
    node_name = "n0S_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 28,
    child_num = 2,
    child_name_list = {
      "n0S0_UpdateBlackboard",
      "n0S1_ApplySkillAction"
    }
  }
end

function _M.n0S0_UpdateBlackboard()
  return {
    node_name = "n0S0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0S_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0S1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0S1_ApplySkillAction()
  return {
    node_name = "n0S1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0S_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021612,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 29,
    action_flow_time = 75.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0T_CBTAlwaysSequenceNode()
  return {
    node_name = "n0T_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 29,
    child_num = 2,
    child_name_list = {
      "n0T0_UpdateBlackboard",
      "n0T1_ApplySkillAction"
    }
  }
end

function _M.n0T0_UpdateBlackboard()
  return {
    node_name = "n0T0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0T_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0T1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0T1_ApplySkillAction()
  return {
    node_name = "n0T1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0T_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021613,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 30,
    action_flow_time = 75.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0U_CBTAlwaysSequenceNode()
  return {
    node_name = "n0U_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 30,
    child_num = 1,
    child_name_list = {
      "n0U0_ApplySkillAction"
    }
  }
end

function _M.n0U0_ApplySkillAction()
  return {
    node_name = "n0U0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0U_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021614,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 31,
    action_flow_time = 55.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0V_CBTAlwaysSequenceNode()
  return {
    node_name = "n0V_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 31,
    child_num = 1,
    child_name_list = {
      "n0V0_ApplySkillAction"
    }
  }
end

function _M.n0V0_ApplySkillAction()
  return {
    node_name = "n0V0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0V_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021615,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 32,
    action_flow_time = 55.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0W_CBTAlwaysSequenceNode()
  return {
    node_name = "n0W_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 32,
    child_num = 1,
    child_name_list = {
      "n0W0_ApplySkillAction"
    }
  }
end

function _M.n0W0_ApplySkillAction()
  return {
    node_name = "n0W0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0W_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021616,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0X_CBTAlwaysSequenceNode()
  return {
    node_name = "n0X_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 33,
    child_num = 1,
    child_name_list = {
      "n0X0_ApplySkillAction"
    }
  }
end

function _M.n0X0_ApplySkillAction()
  return {
    node_name = "n0X0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0X_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021625,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 0,
    action_flow_time = 0.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0Y_CBTAlwaysSequenceNode()
  return {
    node_name = "n0Y_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 34,
    child_num = 1,
    child_name_list = {
      "n0Y0_ApplySkillAction"
    }
  }
end

function _M.n0Y0_ApplySkillAction()
  return {
    node_name = "n0Y0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0Y_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021631,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 35,
    action_flow_time = 35.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0Z_CBTAlwaysSequenceNode()
  return {
    node_name = "n0Z_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 35,
    child_num = 2,
    child_name_list = {
      "n0Z0_UpdateBlackboard",
      "n0Z1_ApplySkillAction"
    }
  }
end

function _M.n0Z0_UpdateBlackboard()
  return {
    node_name = "n0Z0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0Z_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0Z1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "kengki_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0Z1_ApplySkillAction()
  return {
    node_name = "n0Z1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0Z_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021634,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 36,
    action_flow_time = 45.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0a_CBTAlwaysSequenceNode()
  return {
    node_name = "n0a_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 36,
    child_num = 2,
    child_name_list = {
      "n0a0_UpdateBlackboard",
      "n0a1_ApplySkillAction"
    }
  }
end

function _M.n0a0_UpdateBlackboard()
  return {
    node_name = "n0a0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0a_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0a1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "kengki_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0a1_ApplySkillAction()
  return {
    node_name = "n0a1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0a_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021635,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 37,
    action_flow_time = 60.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0b_CBTAlwaysSequenceNode()
  return {
    node_name = "n0b_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 37,
    child_num = 2,
    child_name_list = {
      "n0b0_UpdateBlackboard",
      "n0b1_ApplySkillAction"
    }
  }
end

function _M.n0b0_UpdateBlackboard()
  return {
    node_name = "n0b0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0b_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0b1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "kengki_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0b1_ApplySkillAction()
  return {
    node_name = "n0b1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0b_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021636,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 38,
    action_flow_time = 65.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0c_CBTAlwaysSequenceNode()
  return {
    node_name = "n0c_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 38,
    child_num = 2,
    child_name_list = {
      "n0c0_UpdateBlackboard",
      "n0c1_ApplySkillAction"
    }
  }
end

function _M.n0c0_UpdateBlackboard()
  return {
    node_name = "n0c0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0c_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0c1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "kengki_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0c1_ApplySkillAction()
  return {
    node_name = "n0c1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0c_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021637,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 39,
    action_flow_time = 60.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0d_CBTAlwaysSequenceNode()
  return {
    node_name = "n0d_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 39,
    child_num = 2,
    child_name_list = {
      "n0d0_NotNode",
      "n0d1_ApplySkillAction"
    }
  }
end

function _M.n0d0_NotNode()
  return {
    node_name = "n0d0_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0d_CBTAlwaysSequenceNode",
    cls_name = "NotNode",
    fast_node = "n0d1_ApplySkillAction",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0d00_CheckObstacle"
    }
  }
end

function _M.n0d00_CheckObstacle()
  return {
    node_name = "n0d00_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n0d0_NotNode",
    cls_name = "CheckObstacle",
    idx_in_parent = 0,
    bb_target = "_T1",
    direction = 3,
    target_bone_name = "Bip001 Spine1",
    target_ent_bone_name = "Scene Root",
    distance = 5.0,
    collision_type = 39,
    ignore_self = true,
    draw_line_flag = false,
    use_sweep = false,
    use_target_ent = false
  }
end

function _M.n0d1_ApplySkillAction()
  return {
    node_name = "n0d1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0d_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021628,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 40,
    action_flow_time = 55.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0e_CBTAlwaysSequenceNode()
  return {
    node_name = "n0e_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 40,
    child_num = 1,
    child_name_list = {
      "n0e0_ApplySkillAction"
    }
  }
end

function _M.n0e0_ApplySkillAction()
  return {
    node_name = "n0e0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0e_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021639,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 41,
    action_flow_time = 60.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0f_CBTAlwaysSequenceNode()
  return {
    node_name = "n0f_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 41,
    child_num = 1,
    child_name_list = {
      "n0f0_ApplySkillAction"
    }
  }
end

function _M.n0f0_ApplySkillAction()
  return {
    node_name = "n0f0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0f_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021640,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 42,
    action_flow_time = 45.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0g_CBTAlwaysSequenceNode()
  return {
    node_name = "n0g_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 42,
    child_num = 2,
    child_name_list = {
      "n0g0_SetBlackboard",
      "n0g1_ApplySkillAction"
    }
  }
end

function _M.n0g0_SetBlackboard()
  return {
    node_name = "n0g0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0g_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n0g1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "kengki_count",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n0g1_ApplySkillAction()
  return {
    node_name = "n0g1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0g_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021638,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 43,
    action_flow_time = 55.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0h_CBTAlwaysSequenceNode()
  return {
    node_name = "n0h_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 43,
    child_num = 1,
    child_name_list = {
      "n0h0_ApplySkillAction"
    }
  }
end

function _M.n0h0_ApplySkillAction()
  return {
    node_name = "n0h0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0h_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021622,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 44,
    action_flow_time = 85.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0i_CBTAlwaysSequenceNode()
  return {
    node_name = "n0i_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 44,
    child_num = 2,
    child_name_list = {
      "n0i0_CBTSequenceNode",
      "n0i1_ApplySkillAction"
    }
  }
end

function _M.n0i0_CBTSequenceNode()
  return {
    node_name = "n0i0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0i_CBTAlwaysSequenceNode",
    cls_name = "CBTSequenceNode",
    fast_node = "n0i1_ApplySkillAction",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n0i00_NotNode",
      "n0i01_UpdateBlackboard"
    }
  }
end

function _M.n0i00_NotNode()
  return {
    node_name = "n0i00_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0i0_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n0i01_UpdateBlackboard",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0i000_CheckBuff"
    }
  }
end

function _M.n0i000_CheckBuff()
  return {
    node_name = "n0i000_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n0i00_NotNode",
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

function _M.n0i01_UpdateBlackboard()
  return {
    node_name = "n0i01_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0i0_CBTSequenceNode",
    cls_name = "UpdateBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0i1_ApplySkillAction()
  return {
    node_name = "n0i1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0i_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021641,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 45,
    action_flow_time = 45.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0j_CBTAlwaysSequenceNode()
  return {
    node_name = "n0j_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 45,
    child_num = 1,
    child_name_list = {
      "n0j0_ApplySkillAction"
    }
  }
end

function _M.n0j0_ApplySkillAction()
  return {
    node_name = "n0j0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0j_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021623,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 46,
    action_flow_time = 85.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0k_CBTAlwaysSequenceNode()
  return {
    node_name = "n0k_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 46,
    child_num = 3,
    child_name_list = {
      "n0k0_ApplyBuff",
      "n0k1_ApplyBuff",
      "n0k2_ApplySkillAction"
    }
  }
end

function _M.n0k0_ApplyBuff()
  return {
    node_name = "n0k0_ApplyBuff",
    cls = AINodes.ApplyBuff,
    parent_node_name = "n0k_CBTAlwaysSequenceNode",
    cls_name = "ApplyBuff",
    fast_node = "n0k1_ApplyBuff",
    idx_in_parent = 0,
    op = "add",
    target_type = 1,
    target_list = "_T1",
    buff_id = "450098",
    buff_target = "",
    use_buff_level = false,
    buff_level_type = 1,
    buff_level_list = ""
  }
end

function _M.n0k1_ApplyBuff()
  return {
    node_name = "n0k1_ApplyBuff",
    cls = AINodes.ApplyBuff,
    parent_node_name = "n0k_CBTAlwaysSequenceNode",
    cls_name = "ApplyBuff",
    fast_node = "n0k2_ApplySkillAction",
    idx_in_parent = 1,
    op = "remove",
    target_type = 2,
    target_list = "_T1",
    buff_id = "450100",
    buff_target = "",
    use_buff_level = false,
    buff_level_type = 1,
    buff_level_list = ""
  }
end

function _M.n0k2_ApplySkillAction()
  return {
    node_name = "n0k2_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0k_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 2,
    skill_type = 0,
    skill_slot = 90021646,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 46,
    action_flow_time = 85.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0l_CBTAlwaysSequenceNode()
  return {
    node_name = "n0l_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 47,
    child_num = 1,
    child_name_list = {
      "n0l0_ApplySkillAction"
    }
  }
end

function _M.n0l0_ApplySkillAction()
  return {
    node_name = "n0l0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0l_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021647,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 48,
    action_flow_time = 65.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0m_CBTAlwaysSequenceNode()
  return {
    node_name = "n0m_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 48,
    child_num = 1,
    child_name_list = {
      "n0m0_ApplySkillAction"
    }
  }
end

function _M.n0m0_ApplySkillAction()
  return {
    node_name = "n0m0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0m_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021648,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 49,
    action_flow_time = 50.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0n_CBTAlwaysSequenceNode()
  return {
    node_name = "n0n_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 49,
    child_num = 2,
    child_name_list = {
      "n0n0_UpdateBlackboard",
      "n0n1_ApplySkillAction"
    }
  }
end

function _M.n0n0_UpdateBlackboard()
  return {
    node_name = "n0n0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0n_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0n1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0n1_ApplySkillAction()
  return {
    node_name = "n0n1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0n_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021649,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 50,
    action_flow_time = 65.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0o_CBTAlwaysSequenceNode()
  return {
    node_name = "n0o_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 50,
    child_num = 2,
    child_name_list = {
      "n0o0_NotNode",
      "n0o1_ApplySkillAction"
    }
  }
end

function _M.n0o0_NotNode()
  return {
    node_name = "n0o0_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0o_CBTAlwaysSequenceNode",
    cls_name = "NotNode",
    fast_node = "n0o1_ApplySkillAction",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0o00_CheckObstacle"
    }
  }
end

function _M.n0o00_CheckObstacle()
  return {
    node_name = "n0o00_CheckObstacle",
    cls = AINodes.CheckObstacle,
    parent_node_name = "n0o0_NotNode",
    cls_name = "CheckObstacle",
    idx_in_parent = 0,
    bb_target = "_T1",
    direction = 3,
    target_bone_name = "Bip001 Spine1",
    target_ent_bone_name = "Scene Root",
    distance = 5.0,
    collision_type = 39,
    ignore_self = true,
    draw_line_flag = false,
    use_sweep = false,
    use_target_ent = false
  }
end

function _M.n0o1_ApplySkillAction()
  return {
    node_name = "n0o1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0o_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021650,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 51,
    action_flow_time = 65.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0p_CBTAlwaysSequenceNode()
  return {
    node_name = "n0p_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 51,
    child_num = 1,
    child_name_list = {
      "n0p0_ApplySkillAction"
    }
  }
end

function _M.n0p0_ApplySkillAction()
  return {
    node_name = "n0p0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0p_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021651,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 52,
    action_flow_time = 50.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0q_CBTAlwaysSequenceNode()
  return {
    node_name = "n0q_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 52,
    child_num = 1,
    child_name_list = {
      "n0q0_ApplySkillAction"
    }
  }
end

function _M.n0q0_ApplySkillAction()
  return {
    node_name = "n0q0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0q_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021652,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 53,
    action_flow_time = 55.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0r_CBTAlwaysSequenceNode()
  return {
    node_name = "n0r_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 53,
    child_num = 1,
    child_name_list = {
      "n0r0_ApplySkillAction"
    }
  }
end

function _M.n0r0_ApplySkillAction()
  return {
    node_name = "n0r0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0r_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021653,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 54,
    action_flow_time = 55.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0s_CBTAlwaysSequenceNode()
  return {
    node_name = "n0s_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 54,
    child_num = 1,
    child_name_list = {
      "n0s0_ApplySkillAction"
    }
  }
end

function _M.n0s0_ApplySkillAction()
  return {
    node_name = "n0s0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0s_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021654,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 55,
    action_flow_time = 60.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0t_CBTAlwaysSequenceNode()
  return {
    node_name = "n0t_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 55,
    child_num = 1,
    child_name_list = {
      "n0t0_ApplySkillAction"
    }
  }
end

function _M.n0t0_ApplySkillAction()
  return {
    node_name = "n0t0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0t_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021655,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 56,
    action_flow_time = 45.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0u_CBTAlwaysSequenceNode()
  return {
    node_name = "n0u_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 56,
    child_num = 1,
    child_name_list = {
      "n0u0_ApplySkillAction"
    }
  }
end

function _M.n0u0_ApplySkillAction()
  return {
    node_name = "n0u0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0u_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021656,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 57,
    action_flow_time = 50.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0v_CBTAlwaysSequenceNode()
  return {
    node_name = "n0v_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 57,
    child_num = 1,
    child_name_list = {
      "n0v0_ApplySkillAction"
    }
  }
end

function _M.n0v0_ApplySkillAction()
  return {
    node_name = "n0v0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0v_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021657,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 58,
    action_flow_time = 40.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0w_CBTAlwaysSequenceNode()
  return {
    node_name = "n0w_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 58,
    child_num = 2,
    child_name_list = {
      "n0w0_UpdateBlackboard",
      "n0w1_ApplySkillAction"
    }
  }
end

function _M.n0w0_UpdateBlackboard()
  return {
    node_name = "n0w0_UpdateBlackboard",
    cls = AINodes.UpdateBlackboard,
    parent_node_name = "n0w_CBTAlwaysSequenceNode",
    cls_name = "UpdateBlackboard",
    fast_node = "n0w1_ApplySkillAction",
    idx_in_parent = 0,
    is_space = false,
    key = "attack_count",
    operator = "+",
    value_type = 1,
    value = "1"
  }
end

function _M.n0w1_ApplySkillAction()
  return {
    node_name = "n0w1_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0w_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021658,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = true,
    action_flow_multi = false,
    action_flow_key = 59,
    action_flow_time = 55.0,
    action_flow_value = 1,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

function _M.n0x_CBTAlwaysSequenceNode()
  return {
    node_name = "n0x_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n0_CBTSwitchCaseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 59,
    child_num = 1,
    child_name_list = {
      "n0x0_ApplySkillAction"
    }
  }
end

function _M.n0x0_ApplySkillAction()
  return {
    node_name = "n0x0_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0x_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    idx_in_parent = 0,
    skill_type = 0,
    skill_slot = 90021645,
    skill_slot_bb_key = "",
    check_target = true,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "_T1",
    target_bone_name = "Bip001 Head",
    enable_physics = true,
    check_skill_cd = false,
    check_skill_resource = false,
    timer_key_type = 4,
    timer_key_list = "list()",
    timer_duration_list = "list()",
    blackboard_key_list = "list()",
    is_check_distance = false,
    check_distance = -1.0,
    can_gd = false,
    limit_can_gd_skill_state = 1,
    write_skill_id_to_bb = false,
    end_skill_on_cancel = true,
    action_flow_modify = false,
    action_flow_multi = false,
    action_flow_key = 46,
    action_flow_time = 85.0,
    action_flow_value = 0,
    action_flow_key_list = "list()",
    action_flow_time_list = "list()",
    action_flow_value_list = "list()",
    use_qishu_slot = false
  }
end

_M.entry = "n0_CBTSwitchCaseNode"
return _M
