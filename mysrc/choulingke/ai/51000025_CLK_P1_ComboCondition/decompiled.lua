local AINodes = require("hexm.common.AI.nodes.ai_nodes").AINodes
local _M = {}
_M.TREE_NAME = "51000025_CLK_P1_ComboCondition"
_M.TREE_NODES_NUM = 122
_M.NODE_DATA_CACHE = {}

function _M.n0_CBTAlwaysSequenceNode()
  return {
    node_name = "n0_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    cls_name = "CBTAlwaysSequenceNode",
    child_num = 1,
    child_name_list = {
      "n00_CBTIfElseNode"
    }
  }
end

function _M.n00_CBTIfElseNode()
  return {
    node_name = "n00_CBTIfElseNode",
    cls = AINodes.CBTIfElseNode,
    parent_node_name = "n0_CBTAlwaysSequenceNode",
    cls_name = "CBTIfElseNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n000_CBTSwitchCaseNode",
      "n001_CBTAlwaysSequenceNode",
      "n002_CBTAlwaysSequenceNode"
    }
  }
end

function _M.n000_CBTSwitchCaseNode()
  return {
    node_name = "n000_CBTSwitchCaseNode",
    cls = AINodes.CBTSwitchCaseNode,
    parent_node_name = "n00_CBTIfElseNode",
    cls_name = "CBTSwitchCaseNode",
    succ_fast_node = "n001_CBTAlwaysSequenceNode",
    fail_fast_node = "n002_CBTAlwaysSequenceNode",
    idx_in_parent = 0,
    child_num = 28,
    child_name_list = {
      "n0000_CBTSequenceNode",
      "n0001_CBTSequenceNode",
      "n0002_CBTSelectNode",
      "n0003_CBTSelectNode",
      "n0004_CBTSequenceNode",
      "n0005_CBTSequenceNode",
      "n0006_CBTSequenceNode",
      "n0007_CBTSequenceNode",
      "n0008_CBTSequenceNode",
      "n0009_CBTSequenceNode",
      "n000A_CBTSequenceNode",
      "n000B_CBTSelectNode",
      "n000C_CBTSelectNode",
      "n000D_CBTSequenceNode",
      "n000E_CBTSelectNode",
      "n000F_CBTSequenceNode",
      "n000G_CBTSequenceNode",
      "n000H_CBTSequenceNode",
      "n000I_CBTSequenceNode",
      "n000J_CBTSelectNode",
      "n000K_CBTSelectNode",
      "n000L_CBTSelectNode",
      "n000M_CBTSelectNode",
      "n000N_CBTSelectNode",
      "n000O_CBTSelectNode",
      "n000P_CBTSequenceNode",
      "n000Q_CBTSequenceNode",
      "n000R_CBTSequenceNode"
    },
    key_type = 1,
    key = "AIComboTagID",
    case_type = 4,
    cases_dict = {
      ["1"] = 0,
      behit_gold = 1,
      lock_behit = 2,
      dodgetandao = 3,
      ["593"] = 4,
      ["594"] = 5,
      ["595"] = 6,
      ["596"] = 7,
      ["599"] = 8,
      ["601"] = 9,
      ["602"] = 10,
      ["626"] = 11,
      ["627"] = 12,
      ["628"] = 13,
      ["629"] = 14,
      ["635"] = 15,
      ["6351"] = 16,
      ["636"] = 17,
      ["6361"] = 18,
      ["637"] = 19,
      ["638"] = 20,
      ["6381"] = 21,
      ["639"] = 22,
      ["640"] = 23,
      ["641"] = 24,
      ["80"] = 25,
      ["81"] = 26,
      ["99"] = 27
    }
  }
end

function _M.n0000_CBTSequenceNode()
  return {
    node_name = "n0000_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n00000_CBTSequenceNode"
    }
  }
end

function _M.n00000_CBTSequenceNode()
  return {
    node_name = "n00000_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0000_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000000_CheckTargetDis",
      "n000001_SetBlackboard"
    }
  }
end

function _M.n000000_CheckTargetDis()
  return {
    node_name = "n000000_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00000_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000001_SetBlackboard",
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

function _M.n000001_SetBlackboard()
  return {
    node_name = "n000001_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00000_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n0001_CBTSequenceNode()
  return {
    node_name = "n0001_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n00010_SetBlackboard"
    }
  }
end

function _M.n00010_SetBlackboard()
  return {
    node_name = "n00010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0001_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "5",
    reason = ""
  }
end

function _M.n0002_CBTSelectNode()
  return {
    node_name = "n0002_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 2,
    child_num = 2,
    child_name_list = {
      "n00020_CBTSequenceNode",
      "n00021_CBTSequenceNode"
    }
  }
end

function _M.n00020_CBTSequenceNode()
  return {
    node_name = "n00020_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0002_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n00021_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n000200_NotNode",
      "n000201_CheckTargetDis",
      "n000202_CheckTimerDecorator"
    }
  }
end

function _M.n000200_NotNode()
  return {
    node_name = "n000200_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n00020_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n000201_CheckTargetDis",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0002000_CheckBuff"
    }
  }
end

function _M.n0002000_CheckBuff()
  return {
    node_name = "n0002000_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n000200_NotNode",
    cls_name = "CheckBuff",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450079",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n000201_CheckTargetDis()
  return {
    node_name = "n000201_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00020_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000202_CheckTimerDecorator",
    idx_in_parent = 1,
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
    distance = "5",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n000202_CheckTimerDecorator()
  return {
    node_name = "n000202_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n00020_CBTSequenceNode",
    cls_name = "CheckTimerDecorator",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n0002020_SetBlackboard"
    },
    decorator_result = "",
    timer_key = "ceshan_timer",
    duration = 55.0,
    first_no_exec = false
  }
end

function _M.n0002020_SetBlackboard()
  return {
    node_name = "n0002020_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000202_CheckTimerDecorator",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "10",
    reason = ""
  }
end

function _M.n00021_CBTSequenceNode()
  return {
    node_name = "n00021_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0002_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n000210_NotNode",
      "n000211_Probability"
    }
  }
end

function _M.n000210_NotNode()
  return {
    node_name = "n000210_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n00021_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n000211_Probability",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n0002100_CheckBuff"
    }
  }
end

function _M.n0002100_CheckBuff()
  return {
    node_name = "n0002100_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n000210_NotNode",
    cls_name = "CheckBuff",
    idx_in_parent = 0,
    target_type = 1,
    target_bb_key = "_T1",
    buff_id_from_blackboard = false,
    buff_id_bb_key = "buff_id_bb_key",
    buff_id = "450079",
    check_level = false,
    level_value_type = 3,
    level_value = "",
    level_operation = ">"
  }
end

function _M.n000211_Probability()
  return {
    node_name = "n000211_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n00021_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0002110_SetBlackboard",
      "n0002111_SetBlackboard"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = true
  }
end

function _M.n0002110_SetBlackboard()
  return {
    node_name = "n0002110_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000211_Probability",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "8",
    reason = ""
  }
end

function _M.n0002111_SetBlackboard()
  return {
    node_name = "n0002111_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000211_Probability",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "9",
    reason = ""
  }
end

function _M.n0003_CBTSelectNode()
  return {
    node_name = "n0003_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 3,
    child_num = 1,
    child_name_list = {
      "n00030_SetBlackboard"
    }
  }
end

function _M.n00030_SetBlackboard()
  return {
    node_name = "n00030_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0003_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "9",
    reason = ""
  }
end

function _M.n0004_CBTSequenceNode()
  return {
    node_name = "n0004_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 4,
    child_num = 1,
    child_name_list = {
      "n00040_CBTSequenceNode"
    }
  }
end

function _M.n00040_CBTSequenceNode()
  return {
    node_name = "n00040_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0004_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000400_CheckTargetDis",
      "n000401_SetBlackboard"
    }
  }
end

function _M.n000400_CheckTargetDis()
  return {
    node_name = "n000400_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00040_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000401_SetBlackboard",
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

function _M.n000401_SetBlackboard()
  return {
    node_name = "n000401_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00040_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "21",
    reason = ""
  }
end

function _M.n0005_CBTSequenceNode()
  return {
    node_name = "n0005_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 5,
    child_num = 1,
    child_name_list = {
      "n00050_CBTSequenceNode"
    }
  }
end

function _M.n00050_CBTSequenceNode()
  return {
    node_name = "n00050_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0005_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000500_CheckTargetDis",
      "n000501_SetBlackboard"
    }
  }
end

function _M.n000500_CheckTargetDis()
  return {
    node_name = "n000500_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00050_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000501_SetBlackboard",
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

function _M.n000501_SetBlackboard()
  return {
    node_name = "n000501_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00050_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "22",
    reason = ""
  }
end

function _M.n0006_CBTSequenceNode()
  return {
    node_name = "n0006_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 6,
    child_num = 1,
    child_name_list = {
      "n00060_CBTSequenceNode"
    }
  }
end

function _M.n00060_CBTSequenceNode()
  return {
    node_name = "n00060_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0006_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000600_CheckTargetDis",
      "n000601_SetBlackboard"
    }
  }
end

function _M.n000600_CheckTargetDis()
  return {
    node_name = "n000600_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00060_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000601_SetBlackboard",
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

function _M.n000601_SetBlackboard()
  return {
    node_name = "n000601_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00060_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "24",
    reason = ""
  }
end

function _M.n0007_CBTSequenceNode()
  return {
    node_name = "n0007_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 7,
    child_num = 1,
    child_name_list = {
      "n00070_CBTSequenceNode"
    }
  }
end

function _M.n00070_CBTSequenceNode()
  return {
    node_name = "n00070_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0007_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000700_SetBlackboard"
    }
  }
end

function _M.n000700_SetBlackboard()
  return {
    node_name = "n000700_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00070_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "24",
    reason = ""
  }
end

function _M.n0008_CBTSequenceNode()
  return {
    node_name = "n0008_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 8,
    child_num = 1,
    child_name_list = {
      "n00080_CBTSequenceNode"
    }
  }
end

function _M.n00080_CBTSequenceNode()
  return {
    node_name = "n00080_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0008_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000800_CheckTargetDis",
      "n000801_SetBlackboard"
    }
  }
end

function _M.n000800_CheckTargetDis()
  return {
    node_name = "n000800_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00080_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000801_SetBlackboard",
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

function _M.n000801_SetBlackboard()
  return {
    node_name = "n000801_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00080_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "23",
    reason = ""
  }
end

function _M.n0009_CBTSequenceNode()
  return {
    node_name = "n0009_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 9,
    child_num = 1,
    child_name_list = {
      "n00090_NotNode"
    }
  }
end

function _M.n00090_NotNode()
  return {
    node_name = "n00090_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0009_CBTSequenceNode",
    cls_name = "NotNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000900_Nop"
    }
  }
end

function _M.n000900_Nop()
  return {
    node_name = "n000900_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n00090_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n000A_CBTSequenceNode()
  return {
    node_name = "n000A_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 10,
    child_num = 1,
    child_name_list = {
      "n000A0_CBTSequenceNode"
    }
  }
end

function _M.n000A0_CBTSequenceNode()
  return {
    node_name = "n000A0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000A_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000A00_CheckTargetDis",
      "n000A01_SetBlackboard"
    }
  }
end

function _M.n000A00_CheckTargetDis()
  return {
    node_name = "n000A00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000A0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000A01_SetBlackboard",
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
    distance = "5",
    skill_id_value_type = 1,
    skill_id = "",
    operation = ">="
  }
end

function _M.n000A01_SetBlackboard()
  return {
    node_name = "n000A01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000A0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "25",
    reason = ""
  }
end

function _M.n000B_CBTSelectNode()
  return {
    node_name = "n000B_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 11,
    child_num = 2,
    child_name_list = {
      "n000B0_CBTSequenceNode",
      "n000B1_SetBlackboard"
    }
  }
end

function _M.n000B0_CBTSequenceNode()
  return {
    node_name = "n000B0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000B_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n000B1_SetBlackboard",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000B00_CheckTargetDis",
      "n000B01_SetBlackboard"
    }
  }
end

function _M.n000B00_CheckTargetDis()
  return {
    node_name = "n000B00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000B0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000B01_SetBlackboard",
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

function _M.n000B01_SetBlackboard()
  return {
    node_name = "n000B01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000B0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "11",
    reason = ""
  }
end

function _M.n000B1_SetBlackboard()
  return {
    node_name = "n000B1_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000B_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "2",
    reason = ""
  }
end

function _M.n000C_CBTSelectNode()
  return {
    node_name = "n000C_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 12,
    child_num = 2,
    child_name_list = {
      "n000C0_CBTSequenceNode",
      "n000C1_SetBlackboard"
    }
  }
end

function _M.n000C0_CBTSequenceNode()
  return {
    node_name = "n000C0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000C_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n000C1_SetBlackboard",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000C00_CheckTargetDis",
      "n000C01_SetBlackboard"
    }
  }
end

function _M.n000C00_CheckTargetDis()
  return {
    node_name = "n000C00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000C0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000C01_SetBlackboard",
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

function _M.n000C01_SetBlackboard()
  return {
    node_name = "n000C01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000C0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "12",
    reason = ""
  }
end

function _M.n000C1_SetBlackboard()
  return {
    node_name = "n000C1_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000C_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "4",
    reason = ""
  }
end

function _M.n000D_CBTSequenceNode()
  return {
    node_name = "n000D_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 13,
    child_num = 1,
    child_name_list = {
      "n000D0_NotNode"
    }
  }
end

function _M.n000D0_NotNode()
  return {
    node_name = "n000D0_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000D_CBTSequenceNode",
    cls_name = "NotNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000D00_Nop"
    }
  }
end

function _M.n000D00_Nop()
  return {
    node_name = "n000D00_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n000D0_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n000E_CBTSelectNode()
  return {
    node_name = "n000E_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 14,
    child_num = 1,
    child_name_list = {
      "n000E0_SetBlackboard"
    }
  }
end

function _M.n000E0_SetBlackboard()
  return {
    node_name = "n000E0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000E_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "20",
    reason = ""
  }
end

function _M.n000F_CBTSequenceNode()
  return {
    node_name = "n000F_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 15,
    child_num = 1,
    child_name_list = {
      "n000F0_CBTSequenceNode"
    }
  }
end

function _M.n000F0_CBTSequenceNode()
  return {
    node_name = "n000F0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000F_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000F00_CheckTargetDis",
      "n000F01_SetBlackboard"
    }
  }
end

function _M.n000F00_CheckTargetDis()
  return {
    node_name = "n000F00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000F0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000F01_SetBlackboard",
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
    distance = "9",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n000F01_SetBlackboard()
  return {
    node_name = "n000F01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000F0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "17",
    reason = ""
  }
end

function _M.n000G_CBTSequenceNode()
  return {
    node_name = "n000G_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 16,
    child_num = 1,
    child_name_list = {
      "n000G0_NotNode"
    }
  }
end

function _M.n000G0_NotNode()
  return {
    node_name = "n000G0_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000G_CBTSequenceNode",
    cls_name = "NotNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000G00_Nop"
    }
  }
end

function _M.n000G00_Nop()
  return {
    node_name = "n000G00_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n000G0_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n000H_CBTSequenceNode()
  return {
    node_name = "n000H_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 17,
    child_num = 1,
    child_name_list = {
      "n000H0_CBTSequenceNode"
    }
  }
end

function _M.n000H0_CBTSequenceNode()
  return {
    node_name = "n000H0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000H_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000H00_CheckTargetDis",
      "n000H01_SetBlackboard"
    }
  }
end

function _M.n000H00_CheckTargetDis()
  return {
    node_name = "n000H00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000H0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000H01_SetBlackboard",
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

function _M.n000H01_SetBlackboard()
  return {
    node_name = "n000H01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000H0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "3",
    reason = ""
  }
end

function _M.n000I_CBTSequenceNode()
  return {
    node_name = "n000I_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 18,
    child_num = 1,
    child_name_list = {
      "n000I0_CBTSequenceNode"
    }
  }
end

function _M.n000I0_CBTSequenceNode()
  return {
    node_name = "n000I0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000I_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000I00_CheckTargetDis",
      "n000I01_SetBlackboard"
    }
  }
end

function _M.n000I00_CheckTargetDis()
  return {
    node_name = "n000I00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000I0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000I01_SetBlackboard",
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

function _M.n000I01_SetBlackboard()
  return {
    node_name = "n000I01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000I0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "18",
    reason = ""
  }
end

function _M.n000J_CBTSelectNode()
  return {
    node_name = "n000J_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 19,
    child_num = 1,
    child_name_list = {
      "n000J0_CBTSequenceNode"
    }
  }
end

function _M.n000J0_CBTSequenceNode()
  return {
    node_name = "n000J0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000J_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000J00_CheckTargetDis",
      "n000J01_SetBlackboard"
    }
  }
end

function _M.n000J00_CheckTargetDis()
  return {
    node_name = "n000J00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000J0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000J01_SetBlackboard",
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

function _M.n000J01_SetBlackboard()
  return {
    node_name = "n000J01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000J0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n000K_CBTSelectNode()
  return {
    node_name = "n000K_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 20,
    child_num = 1,
    child_name_list = {
      "n000K0_CBTSequenceNode"
    }
  }
end

function _M.n000K0_CBTSequenceNode()
  return {
    node_name = "n000K0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000K_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000K00_CheckTargetDis",
      "n000K01_SetBlackboard"
    }
  }
end

function _M.n000K00_CheckTargetDis()
  return {
    node_name = "n000K00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000K0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000K01_SetBlackboard",
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

function _M.n000K01_SetBlackboard()
  return {
    node_name = "n000K01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000K0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "26",
    reason = ""
  }
end

function _M.n000L_CBTSelectNode()
  return {
    node_name = "n000L_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 21,
    child_num = 1,
    child_name_list = {
      "n000L0_CBTSequenceNode"
    }
  }
end

function _M.n000L0_CBTSequenceNode()
  return {
    node_name = "n000L0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000L_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000L00_CheckTargetDis",
      "n000L01_SetBlackboard"
    }
  }
end

function _M.n000L00_CheckTargetDis()
  return {
    node_name = "n000L00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000L0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000L01_SetBlackboard",
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
    distance = "9",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n000L01_SetBlackboard()
  return {
    node_name = "n000L01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000L0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "29",
    reason = ""
  }
end

function _M.n000M_CBTSelectNode()
  return {
    node_name = "n000M_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 22,
    child_num = 1,
    child_name_list = {
      "n000M0_CBTSequenceNode"
    }
  }
end

function _M.n000M0_CBTSequenceNode()
  return {
    node_name = "n000M0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000M_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000M00_CheckTargetDis",
      "n000M01_SetBlackboard"
    }
  }
end

function _M.n000M00_CheckTargetDis()
  return {
    node_name = "n000M00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000M0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000M01_SetBlackboard",
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
    distance = "9",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n000M01_SetBlackboard()
  return {
    node_name = "n000M01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000M0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "28",
    reason = ""
  }
end

function _M.n000N_CBTSelectNode()
  return {
    node_name = "n000N_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 23,
    child_num = 2,
    child_name_list = {
      "n000N0_CBTSequenceNode",
      "n000N1_NotNode"
    }
  }
end

function _M.n000N0_CBTSequenceNode()
  return {
    node_name = "n000N0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000N_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n000N1_NotNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000N00_CheckTargetDis",
      "n000N01_CheckTimerDecorator"
    }
  }
end

function _M.n000N00_CheckTargetDis()
  return {
    node_name = "n000N00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000N0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000N01_CheckTimerDecorator",
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
    distance = "12",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n000N01_CheckTimerDecorator()
  return {
    node_name = "n000N01_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n000N0_CBTSequenceNode",
    cls_name = "CheckTimerDecorator",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n000N010_SetBlackboard"
    },
    decorator_result = "",
    timer_key = "touji1_timer",
    duration = 90.0,
    first_no_exec = false
  }
end

function _M.n000N010_SetBlackboard()
  return {
    node_name = "n000N010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000N01_CheckTimerDecorator",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "15",
    reason = ""
  }
end

function _M.n000N1_NotNode()
  return {
    node_name = "n000N1_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000N_CBTSelectNode",
    cls_name = "NotNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n000N10_Nop"
    }
  }
end

function _M.n000N10_Nop()
  return {
    node_name = "n000N10_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n000N1_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n000O_CBTSelectNode()
  return {
    node_name = "n000O_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 24,
    child_num = 2,
    child_name_list = {
      "n000O0_CBTSequenceNode",
      "n000O1_NotNode"
    }
  }
end

function _M.n000O0_CBTSequenceNode()
  return {
    node_name = "n000O0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000O_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n000O1_NotNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000O00_CheckTargetDis",
      "n000O01_SetBlackboard"
    }
  }
end

function _M.n000O00_CheckTargetDis()
  return {
    node_name = "n000O00_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000O0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000O01_SetBlackboard",
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
    distance = "8",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n000O01_SetBlackboard()
  return {
    node_name = "n000O01_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000O0_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "14",
    reason = ""
  }
end

function _M.n000O1_NotNode()
  return {
    node_name = "n000O1_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000O_CBTSelectNode",
    cls_name = "NotNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n000O10_Nop"
    }
  }
end

function _M.n000O10_Nop()
  return {
    node_name = "n000O10_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n000O1_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n000P_CBTSequenceNode()
  return {
    node_name = "n000P_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 25,
    child_num = 1,
    child_name_list = {
      "n000P0_SetBlackboard"
    }
  }
end

function _M.n000P0_SetBlackboard()
  return {
    node_name = "n000P0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000P_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "6",
    reason = ""
  }
end

function _M.n000Q_CBTSequenceNode()
  return {
    node_name = "n000Q_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 26,
    child_num = 1,
    child_name_list = {
      "n000Q0_SetBlackboard"
    }
  }
end

function _M.n000Q0_SetBlackboard()
  return {
    node_name = "n000Q0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000Q_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 3,
    value = "7",
    reason = ""
  }
end

function _M.n000R_CBTSequenceNode()
  return {
    node_name = "n000R_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 27,
    child_num = 1,
    child_name_list = {
      "n000R0_NotNode"
    }
  }
end

function _M.n000R0_NotNode()
  return {
    node_name = "n000R0_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000R_CBTSequenceNode",
    cls_name = "NotNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000R00_Nop"
    }
  }
end

function _M.n000R00_Nop()
  return {
    node_name = "n000R00_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n000R0_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n001_CBTAlwaysSequenceNode()
  return {
    node_name = "n001_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n00_CBTIfElseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n0010_SetBlackboard",
      "n0011_SendEventAction"
    }
  }
end

function _M.n0010_SetBlackboard()
  return {
    node_name = "n0010_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n001_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n0011_SendEventAction",
    idx_in_parent = 0,
    is_space = false,
    key = "CanComboCancel",
    value_type = 3,
    value = "1",
    reason = ""
  }
end

function _M.n0011_SendEventAction()
  return {
    node_name = "n0011_SendEventAction",
    cls = AINodes.SendEventAction,
    parent_node_name = "n001_CBTAlwaysSequenceNode",
    cls_name = "SendEventAction",
    idx_in_parent = 1,
    send_type = "self",
    search_range_type = 3,
    search_range = "5.0",
    entity_no_list = listlib.filter(string.split("list()", ","), function(x)
      return "" ~= x
    end),
    entity_same_faction = false,
    serial_no_list = listlib.filter(string.split("list()", ","), function(x)
      return "" ~= x
    end),
    serial_no_type = "abs",
    event_name = "E_AI_COMBO_CHECK_END",
    event_data_from = 0,
    blackboard_key = "",
    event_key = "list()",
    event_value = "list()",
    entity_bb_key = "_target"
  }
end

function _M.n002_CBTAlwaysSequenceNode()
  return {
    node_name = "n002_CBTAlwaysSequenceNode",
    cls = AINodes.CBTAlwaysSequenceNode,
    parent_node_name = "n00_CBTIfElseNode",
    cls_name = "CBTAlwaysSequenceNode",
    idx_in_parent = 2,
    child_num = 2,
    child_name_list = {
      "n0020_SetBlackboard",
      "n0021_SendEventAction"
    }
  }
end

function _M.n0020_SetBlackboard()
  return {
    node_name = "n0020_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n002_CBTAlwaysSequenceNode",
    cls_name = "SetBlackboard",
    fast_node = "n0021_SendEventAction",
    idx_in_parent = 0,
    is_space = false,
    key = "CanComboCancel",
    value_type = 3,
    value = "-1",
    reason = ""
  }
end

function _M.n0021_SendEventAction()
  return {
    node_name = "n0021_SendEventAction",
    cls = AINodes.SendEventAction,
    parent_node_name = "n002_CBTAlwaysSequenceNode",
    cls_name = "SendEventAction",
    idx_in_parent = 1,
    send_type = "self",
    search_range_type = 3,
    search_range = "5.0",
    entity_no_list = listlib.filter(string.split("list()", ","), function(x)
      return "" ~= x
    end),
    entity_same_faction = false,
    serial_no_list = listlib.filter(string.split("list()", ","), function(x)
      return "" ~= x
    end),
    serial_no_type = "abs",
    event_name = "E_AI_COMBO_CHECK_END",
    event_data_from = 0,
    blackboard_key = "",
    event_key = "list()",
    event_value = "list()",
    entity_bb_key = "_target"
  }
end

_M.entry = "n0_CBTAlwaysSequenceNode"
return _M
