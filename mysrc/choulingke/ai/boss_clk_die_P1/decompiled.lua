local AINodes = require("hexm.common.AI.nodes.ai_nodes").AINodes
local _M = {}
_M.TREE_NAME = "boss_clk_die_P1"
_M.TREE_NODES_NUM = 6
_M.NODE_DATA_CACHE = {}

function _M.n0_CBTAlwaysSequenceNode()
  return {
    node_name = "n0_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    cls_name = "CBTAlwaysSequenceNode",
    child_num = 4,
    child_name_list = {
      "n00_GetEntityAIData",
      "n01_ApplySkillAction",
      "n02_NPassNode",
      "n03_DestroySelf"
    }
  }
end

function _M.n00_GetEntityAIData()
  return {
    node_name = "n00_GetEntityAIData",
    cls = AINodes.GetEntityAIData,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "GetEntityAIData",
    fast_node = "n01_ApplySkillAction",
    idx_in_parent = 0,
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

function _M.n01_ApplySkillAction()
  return {
    node_name = "n01_ApplySkillAction",
    cls = AINodes.ApplySkillAction,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "ApplySkillAction",
    fast_node = "n02_NPassNode",
    idx_in_parent = 1,
    skill_type = 0,
    skill_slot = 90021659,
    skill_slot_bb_key = "",
    check_target = false,
    important_skill_log = false,
    spec_target = false,
    target_bb_key = "",
    target_bone_name = "",
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

function _M.n02_NPassNode()
  return {
    node_name = "n02_NPassNode",
    node_type = 2,
    cls = AINodes.NPassNode,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "NPassNode",
    fast_node = "n03_DestroySelf",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n020_SyncDeadAction"
    },
    record_type = 1,
    total_pass_count = 1
  }
end

function _M.n020_SyncDeadAction()
  return {
    node_name = "n020_SyncDeadAction",
    cls = AINodes.SyncDeadAction,
    parent_node_name = "n02_NPassNode",
    cls_name = "SyncDeadAction",
    idx_in_parent = 0,
    dead_vat = "normal",
    vat_time = 1.0,
    anim_choose = 3,
    direct_dead = true
  }
end

function _M.n03_DestroySelf()
  return {
    node_name = "n03_DestroySelf",
    cls = AINodes.DestroySelf,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "DestroySelf",
    idx_in_parent = 3,
    destroy_effect = true,
    delay = 0.5
  }
end

_M.entry = "n0_CBTAlwaysSequenceNode"
return _M
