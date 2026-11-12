# 资源来源记录

`assets/` 里每个运行时文件的来源说明、转换报告和来源清单都放在这里，目录结构和 `assets/` 一一对应。例如 `assets/characters/lihuowang.bin` 的记录在 `characters/lihuowang/`。

这个目录会提交到仓库，但不会随网页发布。

记录保持生成时的原样：里面写到的路径（例如 `assets/lihuowang/character.bin`、`assets/prepared/naraka/...`）和命令，是文件转换当时的位置，不是现在的位置。资源目录已经在 2026-11-12 重新整理，新旧位置的对照：

| 现在 | 原来 |
|---|---|
| `assets/characters/{choulingke,lihuowang,bailingmiao}.bin` | `assets/<角色>/character.bin` |
| `assets/weapons/melee/{fulangmao,zhexingcuishan}.bin` | `assets/<武器>/weapon.bin` |
| `assets/weapons/ranged/{yinbingluo,yizhiying}.bin` | `assets/<武器>/weapon.bin` |
| `assets/animations/yanyun.bin` | `assets/choulingke/animations.bin` |
| `assets/animations/naraka.bin` | `assets/naraka/animations.bin` |
| `assets/environment/bamboo/*.png` | `assets/choulingke/environment/*.png` |
| `assets/audio/effects/`、`music/`、`naraka/` | `assets/audio/` |
| `assets/audio/naraka/cues.json` | `assets/audio/naraka-cues.json` |
| `src/client/embedded/stickman.anim`、`shaders/` | `assets/animations/`、`assets/shaders/` |
| 仓库外 `../kiva-asset-research/prepared/naraka/` | `assets/prepared/naraka/` |

`scripts/` 里把燕云和永劫源文件转换成运行时资源的导入脚本，默认输入输出仍是原来的位置，没有随这次整理更新。
