# CREDITS

## 教程（行为规格来源）

- [JerryAZR/bevy-tower-defense](https://github.com/JerryAZR/bevy-tower-defense) 26 部教程（MIT License）——本工程的外部路线图与**行为规格**（非实现来源：行为等价实现独立完成，见 `docs/m6-game-selection.md`；API 漂移对照底稿见 `docs/evidence/t044-td-spike/w2/`）。

## 美术资产

- `assets/Tilesheet/towerDefense_tilesheet.png`——"Tower Defense (1.0)" by **Kenney**（www.kenney.nl），**Creative Commons Zero (CC0)** <http://creativecommons.org/publicdomain/zero/1.0/>；许可证原文随附于 `assets/Tilesheet/License.txt`。

## 音频资产（T050 · P20）

- `assets/audio/laser_fire.ogg`、`rocket_explosion.ogg`、`rocket_launch.ogg`、
  `background_music.ogg`——**本工程合成**占位音：ffmpeg（`sine` / `anoisesrc` 滤波器）
  离线生成的正弦 / 噪声波形，无第三方素材、无外部权利主张；随本仓库按
  MIT OR Apache-2.0 双许可发布（与仓库根许可一致）。
- 许可与规格注记：教程（JerryAZR/bevy-tower-defense，MIT）正文引用
  `audio/laser_fire.ogg` 等文件名与播放行为，但其**音频文件未随教程源码提供**
  （recon 树仅含 Markdown）；故本工程按教程 Step 的文件名与行为（音乐循环 /
  一次性音效 / 弹体循环推进器）对齐，波形内容不在行为规格面内。
