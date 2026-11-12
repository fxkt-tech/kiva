local AINodes = require("hexm.common.AI.nodes.ai_nodes").AINodes
local _M = {}
_M.TREE_NAME = "51000031_CLK_P2_ComboCondition"
_M.TREE_NODES_NUM = 107
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
    child_num = 25,
    child_name_list = {
      "n0000_CBTSequenceNode",
      "n0001_CBTSequenceNode",
      "n0002_CBTSequenceNode",
      "n0003_CBTSequenceNode",
      "n0004_CBTSequenceNode",
      "n0005_CBTSequenceNode",
      "n0006_CBTSelectNode",
      "n0007_CBTSelectNode",
      "n0008_CBTSelectNode",
      "n0009_CBTSequenceNode",
      "n000A_CBTSequenceNode",
      "n000B_CBTSelectNode",
      "n000C_CBTSelectNode",
      "n000D_CBTSequenceNode",
      "n000E_CBTSequenceNode",
      "n000F_CBTSequenceNode",
      "n000G_CBTSequenceNode",
      "n000H_CBTSequenceNode",
      "n000I_CBTSequenceNode",
      "n000J_CBTSequenceNode",
      "n000K_CBTSelectNode",
      "n000L_CBTSelectNode",
      "n000M_CBTSequenceNode",
      "n000N_CBTSequenceNode",
      "n000O_CBTSequenceNode"
    },
    key_type = 1,
    key = "AIComboTagID",
    case_type = 4,
    cases_dict = {
      ["604"] = 0,
      ["605"] = 1,
      ["607"] = 2,
      ["608"] = 3,
      ["615"] = 4,
      ["616"] = 5,
      ["626"] = 6,
      ["627"] = 7,
      ["629"] = 8,
      ["647"] = 9,
      ["648"] = 10,
      ["650"] = 11,
      ["652"] = 12,
      ["653"] = 13,
      ["655"] = 14,
      ["656"] = 15,
      ["657"] = 16,
      ["658"] = 17,
      behit_gold = 18,
      break_success = 19,
      lock_behit = 20,
      dodgetandao = 21,
      ["80"] = 22,
      ["81"] = 23,
      ["99"] = 24
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
    distance = "15",
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
    value_type = 4,
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
      "n00010_CBTSequenceNode"
    }
  }
end

function _M.n00010_CBTSequenceNode()
  return {
    node_name = "n00010_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0001_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n000100_CheckBlackboard",
      "n000101_SetBlackboard",
      "n000102_SetBlackboard"
    }
  }
end

function _M.n000100_CheckBlackboard()
  return {
    node_name = "n000100_CheckBlackboard",
    cls = AINodes.CheckBlackboard,
    parent_node_name = "n00010_CBTSequenceNode",
    cls_name = "CheckBlackboard",
    succ_fast_node = "n000101_SetBlackboard",
    idx_in_parent = 0,
    key_from = 1,
    key = "is_CA",
    operator = "==",
    value_type = 3,
    value = "1"
  }
end

function _M.n000101_SetBlackboard()
  return {
    node_name = "n000101_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00010_CBTSequenceNode",
    cls_name = "SetBlackboard",
    succ_fast_node = "n000102_SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "is_CA",
    value_type = 3,
    value = "0",
    reason = ""
  }
end

function _M.n000102_SetBlackboard()
  return {
    node_name = "n000102_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00010_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 2,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "21",
    reason = ""
  }
end

function _M.n0002_CBTSequenceNode()
  return {
    node_name = "n0002_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n00020_CBTSequenceNode"
    }
  }
end

function _M.n00020_CBTSequenceNode()
  return {
    node_name = "n00020_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0002_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000200_CheckTargetDis",
      "n000201_SetBlackboard"
    }
  }
end

function _M.n000200_CheckTargetDis()
  return {
    node_name = "n000200_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00020_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000201_SetBlackboard",
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

function _M.n000201_SetBlackboard()
  return {
    node_name = "n000201_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00020_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "20",
    reason = ""
  }
end

function _M.n0003_CBTSequenceNode()
  return {
    node_name = "n0003_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 3,
    child_num = 1,
    child_name_list = {
      "n00030_CBTSequenceNode"
    }
  }
end

function _M.n00030_CBTSequenceNode()
  return {
    node_name = "n00030_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0003_CBTSequenceNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000300_CheckTargetDis",
      "n000301_SetBlackboard"
    }
  }
end

function _M.n000300_CheckTargetDis()
  return {
    node_name = "n000300_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00030_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000301_SetBlackboard",
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

function _M.n000301_SetBlackboard()
  return {
    node_name = "n000301_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00030_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "19",
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
      "n00040_NotNode"
    }
  }
end

function _M.n00040_NotNode()
  return {
    node_name = "n00040_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n0004_CBTSequenceNode",
    cls_name = "NotNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000400_Nop"
    }
  }
end

function _M.n000400_Nop()
  return {
    node_name = "n000400_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n00040_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
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
    value_type = 4,
    value = "6",
    reason = ""
  }
end

function _M.n0006_CBTSelectNode()
  return {
    node_name = "n0006_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 6,
    child_num = 2,
    child_name_list = {
      "n00060_CBTSequenceNode",
      "n00061_SetBlackboard"
    }
  }
end

function _M.n00060_CBTSequenceNode()
  return {
    node_name = "n00060_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0006_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n00061_SetBlackboard",
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
    distance = "6",
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
    value_type = 4,
    value = "3",
    reason = ""
  }
end

function _M.n00061_SetBlackboard()
  return {
    node_name = "n00061_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0006_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "4",
    reason = ""
  }
end

function _M.n0007_CBTSelectNode()
  return {
    node_name = "n0007_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 7,
    child_num = 2,
    child_name_list = {
      "n00070_CBTSequenceNode",
      "n00071_SetBlackboard"
    }
  }
end

function _M.n00070_CBTSequenceNode()
  return {
    node_name = "n00070_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0007_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n00071_SetBlackboard",
    idx_in_parent = 0,
    child_num = 2,
    child_name_list = {
      "n000700_CheckTargetDis",
      "n000701_SetBlackboard"
    }
  }
end

function _M.n000700_CheckTargetDis()
  return {
    node_name = "n000700_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n00070_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000701_SetBlackboard",
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

function _M.n000701_SetBlackboard()
  return {
    node_name = "n000701_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n00070_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "13",
    reason = ""
  }
end

function _M.n00071_SetBlackboard()
  return {
    node_name = "n00071_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0007_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "14",
    reason = ""
  }
end

function _M.n0008_CBTSelectNode()
  return {
    node_name = "n0008_CBTSelectNode",
    cls = AINodes.CBTSelectNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSelectNode",
    idx_in_parent = 8,
    child_num = 2,
    child_name_list = {
      "n00080_CBTSequenceNode",
      "n00081_SetBlackboard"
    }
  }
end

function _M.n00080_CBTSequenceNode()
  return {
    node_name = "n00080_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n0008_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n00081_SetBlackboard",
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
    distance = "6",
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
    value_type = 4,
    value = "2",
    reason = ""
  }
end

function _M.n00081_SetBlackboard()
  return {
    node_name = "n00081_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0008_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "5",
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
      "n00090_SetBlackboard"
    }
  }
end

function _M.n00090_SetBlackboard()
  return {
    node_name = "n00090_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n0009_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "9",
    reason = ""
  }
end

function _M.n000A_CBTSequenceNode()
  return {
    node_name = "n000A_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 10,
    child_num = 2,
    child_name_list = {
      "n000A0_CheckTargetDis",
      "n000A1_SetBlackboard"
    }
  }
end

function _M.n000A0_CheckTargetDis()
  return {
    node_name = "n000A0_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000A_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000A1_SetBlackboard",
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

function _M.n000A1_SetBlackboard()
  return {
    node_name = "n000A1_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000A_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "8",
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
    distance = "5",
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
    value_type = 4,
    value = "15",
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
    value_type = 4,
    value = "16",
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
    child_num = 3,
    child_name_list = {
      "n000C0_CheckTimerDecorator",
      "n000C1_CheckTimerDecorator",
      "n000C2_NotNode"
    }
  }
end

function _M.n000C0_CheckTimerDecorator()
  return {
    node_name = "n000C0_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n000C_CBTSelectNode",
    cls_name = "CheckTimerDecorator",
    fail_fast_node = "n000C1_CheckTimerDecorator",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000C00_SetBlackboard"
    },
    decorator_result = "",
    timer_key = "suohun_timer",
    duration = 65.0,
    first_no_exec = false
  }
end

function _M.n000C00_SetBlackboard()
  return {
    node_name = "n000C00_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000C0_CheckTimerDecorator",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "7",
    reason = ""
  }
end

function _M.n000C1_CheckTimerDecorator()
  return {
    node_name = "n000C1_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n000C_CBTSelectNode",
    cls_name = "CheckTimerDecorator",
    fail_fast_node = "n000C2_NotNode",
    idx_in_parent = 1,
    child_num = 1,
    child_name_list = {
      "n000C10_SetBlackboard"
    },
    decorator_result = "",
    timer_key = "CA_timer",
    duration = 70.0,
    first_no_exec = false
  }
end

function _M.n000C10_SetBlackboard()
  return {
    node_name = "n000C10_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000C1_CheckTimerDecorator",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "21",
    reason = ""
  }
end

function _M.n000C2_NotNode()
  return {
    node_name = "n000C2_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000C_CBTSelectNode",
    cls_name = "NotNode",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n000C20_Nop"
    }
  }
end

function _M.n000C20_Nop()
  return {
    node_name = "n000C20_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n000C2_NotNode",
    cls_name = "Nop",
    idx_in_parent = 0
  }
end

function _M.n000D_CBTSequenceNode()
  return {
    node_name = "n000D_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 13,
    child_num = 3,
    child_name_list = {
      "n000D0_NotNode",
      "n000D1_CheckTargetDis",
      "n000D2_SetBlackboard"
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
    succ_fast_node = "n000D1_CheckTargetDis",
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

function _M.n000D1_CheckTargetDis()
  return {
    node_name = "n000D1_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000D_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000D2_SetBlackboard",
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
    distance = "7",
    skill_id_value_type = 1,
    skill_id = "",
    operation = "<="
  }
end

function _M.n000D2_SetBlackboard()
  return {
    node_name = "n000D2_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000D_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 2,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "11",
    reason = ""
  }
end

function _M.n000E_CBTSequenceNode()
  return {
    node_name = "n000E_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 14,
    child_num = 2,
    child_name_list = {
      "n000E0_CheckTargetDis",
      "n000E1_SetBlackboard"
    }
  }
end

function _M.n000E0_CheckTargetDis()
  return {
    node_name = "n000E0_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000E_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000E1_SetBlackboard",
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

function _M.n000E1_SetBlackboard()
  return {
    node_name = "n000E1_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000E_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "10",
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
    child_num = 2,
    child_name_list = {
      "n000F0_CheckTargetDis",
      "n000F1_SetBlackboard"
    }
  }
end

function _M.n000F0_CheckTargetDis()
  return {
    node_name = "n000F0_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000F_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000F1_SetBlackboard",
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

function _M.n000F1_SetBlackboard()
  return {
    node_name = "n000F1_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000F_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "12",
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
    child_num = 2,
    child_name_list = {
      "n000G0_CheckTargetDis",
      "n000G1_SetBlackboard"
    }
  }
end

function _M.n000G0_CheckTargetDis()
  return {
    node_name = "n000G0_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000G_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000G1_SetBlackboard",
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

function _M.n000G1_SetBlackboard()
  return {
    node_name = "n000G1_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000G_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "18",
    reason = ""
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
      "n000H0_SetBlackboard"
    }
  }
end

function _M.n000H0_SetBlackboard()
  return {
    node_name = "n000H0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000H_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "22",
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
      "n000I0_SetBlackboard"
    }
  }
end

function _M.n000I0_SetBlackboard()
  return {
    node_name = "n000I0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000I_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "behit_gold",
    reason = ""
  }
end

function _M.n000J_CBTSequenceNode()
  return {
    node_name = "n000J_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 19,
    child_num = 1,
    child_name_list = {
      "n000J0_SetBlackboard"
    }
  }
end

function _M.n000J0_SetBlackboard()
  return {
    node_name = "n000J0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000J_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "break_success",
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
    child_num = 2,
    child_name_list = {
      "n000K0_CBTSequenceNode",
      "n000K1_CBTSequenceNode"
    }
  }
end

function _M.n000K0_CBTSequenceNode()
  return {
    node_name = "n000K0_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000K_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    fail_fast_node = "n000K1_CBTSequenceNode",
    idx_in_parent = 0,
    child_num = 3,
    child_name_list = {
      "n000K00_NotNode",
      "n000K01_CheckTargetDis",
      "n000K02_CheckTimerDecorator"
    }
  }
end

function _M.n000K00_NotNode()
  return {
    node_name = "n000K00_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000K0_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n000K01_CheckTargetDis",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000K000_CheckBuff"
    }
  }
end

function _M.n000K000_CheckBuff()
  return {
    node_name = "n000K000_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n000K00_NotNode",
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

function _M.n000K01_CheckTargetDis()
  return {
    node_name = "n000K01_CheckTargetDis",
    cls = AINodes.CheckTargetDis,
    parent_node_name = "n000K0_CBTSequenceNode",
    cls_name = "CheckTargetDis",
    succ_fast_node = "n000K02_CheckTimerDecorator",
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

function _M.n000K02_CheckTimerDecorator()
  return {
    node_name = "n000K02_CheckTimerDecorator",
    node_type = 2,
    cls = AINodes.CheckTimerDecorator,
    parent_node_name = "n000K0_CBTSequenceNode",
    cls_name = "CheckTimerDecorator",
    idx_in_parent = 2,
    child_num = 1,
    child_name_list = {
      "n000K020_SetBlackboard"
    },
    decorator_result = "",
    timer_key = "ceshan_timer",
    duration = 55.0,
    first_no_exec = false
  }
end

function _M.n000K020_SetBlackboard()
  return {
    node_name = "n000K020_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000K02_CheckTimerDecorator",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "ceshan",
    reason = ""
  }
end

function _M.n000K1_CBTSequenceNode()
  return {
    node_name = "n000K1_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000K_CBTSelectNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n000K10_NotNode",
      "n000K11_Probability"
    }
  }
end

function _M.n000K10_NotNode()
  return {
    node_name = "n000K10_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000K1_CBTSequenceNode",
    cls_name = "NotNode",
    succ_fast_node = "n000K11_Probability",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000K100_CheckBuff"
    }
  }
end

function _M.n000K100_CheckBuff()
  return {
    node_name = "n000K100_CheckBuff",
    cls = AINodes.CheckBuff,
    parent_node_name = "n000K10_NotNode",
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

function _M.n000K11_Probability()
  return {
    node_name = "n000K11_Probability",
    cls = AINodes.CBTProbabilityNode,
    parent_node_name = "n000K1_CBTSequenceNode",
    cls_name = "CBTProbabilityNode",
    idx_in_parent = 1,
    child_num = 2,
    child_name_list = {
      "n000K110_SetBlackboard",
      "n000K111_SetBlackboard"
    },
    child_weights = {1, 1},
    acc_child_weights = {1, 2},
    total_weight = 2,
    different_with_last = true
  }
end

function _M.n000K110_SetBlackboard()
  return {
    node_name = "n000K110_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000K11_Probability",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "tandaor",
    reason = ""
  }
end

function _M.n000K111_SetBlackboard()
  return {
    node_name = "n000K111_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000K11_Probability",
    cls_name = "SetBlackboard",
    idx_in_parent = 1,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "tandaol",
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
      "n000L0_SetBlackboard"
    }
  }
end

function _M.n000L0_SetBlackboard()
  return {
    node_name = "n000L0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000L_CBTSelectNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "tandaol",
    reason = ""
  }
end

function _M.n000M_CBTSequenceNode()
  return {
    node_name = "n000M_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 22,
    child_num = 1,
    child_name_list = {
      "n000M0_SetBlackboard"
    }
  }
end

function _M.n000M0_SetBlackboard()
  return {
    node_name = "n000M0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000M_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "tandaoR",
    reason = ""
  }
end

function _M.n000N_CBTSequenceNode()
  return {
    node_name = "n000N_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 23,
    child_num = 1,
    child_name_list = {
      "n000N0_SetBlackboard"
    }
  }
end

function _M.n000N0_SetBlackboard()
  return {
    node_name = "n000N0_SetBlackboard",
    cls = AINodes.SetBlackboard,
    parent_node_name = "n000N_CBTSequenceNode",
    cls_name = "SetBlackboard",
    idx_in_parent = 0,
    is_space = false,
    key = "clk_Combo",
    value_type = 4,
    value = "tandaoL",
    reason = ""
  }
end

function _M.n000O_CBTSequenceNode()
  return {
    node_name = "n000O_CBTSequenceNode",
    cls = AINodes.CBTSequenceNode,
    parent_node_name = "n000_CBTSwitchCaseNode",
    cls_name = "CBTSequenceNode",
    idx_in_parent = 24,
    child_num = 1,
    child_name_list = {
      "n000O0_NotNode"
    }
  }
end

function _M.n000O0_NotNode()
  return {
    node_name = "n000O0_NotNode",
    node_type = 2,
    cls = AINodes.NotNode,
    parent_node_name = "n000O_CBTSequenceNode",
    cls_name = "NotNode",
    idx_in_parent = 0,
    child_num = 1,
    child_name_list = {
      "n000O00_Nop"
    }
  }
end

function _M.n000O00_Nop()
  return {
    node_name = "n000O00_Nop",
    cls = AINodes.Nop,
    parent_node_name = "n000O0_NotNode",
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
