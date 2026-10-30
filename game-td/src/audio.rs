//! 音频层（教程 20 `20-audio`）：背景音乐 + 一次性音效 + 火箭推进器循环音。
//!
//! 教程（0.18 形态）是行为规格、非实现来源；本模块 API 按 bevy 0.20.0 本地源码
//! 重写（对照底稿 `docs/evidence/t044-td-spike/w2/api-drift.md` Part 20，全摩擦级 0）：
//! - `AudioPlayer`（`bevy_audio-0.20.0/src/audio.rs:251`；`#[require(PlaybackSettings)]`
//!   自动补组件）；
//! - `PlaybackSettings::LOOP`（`:100`）/ `::DESPAWN`（`:106`）/ `with_volume`（`:130`）；
//! - `Volume::Linear`（`bevy_audio-0.20.0/src/volume.rs:36`；`Volume` 不在 prelude，
//!   经 `bevy::audio::Volume` 引入——同 0.20 prelude 实况）；
//! - 音频资产 = `AssetServer::load("audio/*.ogg")`（`bevy_asset-0.20.0/src/server/mod.rs:364`）；
//!   ogg/vorbis 解码随 bevy 默认特性 `audio`（`bevy-0.20.0/Cargo.toml:2765-2768`，
//!   `audio = ["bevy_audio", "vorbis"]`）——**未新增依赖、未动 Cargo**。
//!
//! 设计（教程 20 Walkthrough）：
//! - **持久音频** = 实体化 + `LOOP`：音乐实体挂 [`BackgroundMusic`] 标记，
//!   `stop_background_music`（`OnExit(InGame)`）按标记回收；
//! - **一次性音效** = 消息解耦：玩法系统写 [`PlaySound`]，[`play_sound_effects`]
//!   消费并 spawn `PlaybackSettings::DESPAWN` 的短命实体（播完自动回收）；
//! - **火箭推进器** = 组件随实体：音效组件直接挂到弹体上（`tower::launch_rockets`），
//!   弹体 despawn 即声音停止——不需要「停止音效」消息或弹体→音效对照表（教程 20
//!   「Looping rocket thruster on the projectile」）。
//!
//! 音频资产说明（许可）：`assets/audio/*.ogg` 为**本工程合成**的占位音（ffmpeg
//! `sine`/`anoisesrc` 生成，无第三方权利），见 `game-td/CREDITS.md`；教程源码仓
//! 未随附音频文件，故行为规格按教程 Step 与文件名对齐、波形内容非规格面。
//!
//! 验证面（本工程扩展层，非教程）：[`AudioLog`] 计音效触发次数（同
//! `AmmoState::fired` 口径），`td.audio` 报告实体/播放态结构面。

use bevy::audio::Volume;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::GameState;

/// 预加载的音频句柄（教程 20 `AudioAssets`）：Startup 一次装载，系统间共享。
#[derive(Resource)]
pub struct AudioAssets {
    pub laser_fire: Handle<AudioSource>,
    pub rocket_explosion: Handle<AudioSource>,
    pub rocket_launch: Handle<AudioSource>,
    pub background_music: Handle<AudioSource>,
}

/// 背景音乐实体标记（教程 20）：`stop_background_music` 按它定位回收。
#[derive(Component)]
pub struct BackgroundMusic;

/// 一次性音效种类（教程 20 `SoundType`）：枚举命名声音，结构体携带单次音量。
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundType {
    LaserFire,
    RocketExplosion,
}

/// 播放一次性音效的消息（教程 20 `PlaySound`）：生产者（玩法）决定响度，
/// 消费者（本模块）只负责播放。
#[derive(Message, Clone, Copy, Debug)]
pub struct PlaySound {
    pub sound: SoundType,
    pub volume: f32,
}

/// 音效记账（**验证面**资源，非教程；同 `SpawnSchedule::spawned` / `AmmoState::fired`
/// 口径——只服务 `td.audio` 快照与断言）。
#[derive(Resource, Default)]
pub struct AudioLog {
    pub laser_fire: u32,
    pub rocket_explosion: u32,
    /// 投产的短命音效实体累计数（= 消息消费数，两种音效合计）。
    pub sfx_spawned: u32,
    /// 最近一次音效的请求音量（证明载荷穿越消息）。
    pub last_volume: Option<f32>,
}

/// 音频资产路径（教程 20：`assets/audio/*.ogg`；相对资产根）。
pub const LASER_FIRE_FILE: &str = "audio/laser_fire.ogg";
pub const ROCKET_EXPLOSION_FILE: &str = "audio/rocket_explosion.ogg";
pub const ROCKET_LAUNCH_FILE: &str = "audio/rocket_launch.ogg";
pub const BACKGROUND_MUSIC_FILE: &str = "audio/background_music.ogg";

/// 背景音乐音量（教程 20 `start_background_music`：`Volume::Linear(0.3)`）。
pub const MUSIC_VOLUME: f32 = 0.3;

/// 装载音频句柄（教程 20 `load_audio_assets`）：Startup 链中执行。
pub fn load_audio_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AudioAssets {
        laser_fire: asset_server.load(LASER_FIRE_FILE),
        rocket_explosion: asset_server.load(ROCKET_EXPLOSION_FILE),
        rocket_launch: asset_server.load(ROCKET_LAUNCH_FILE),
        background_music: asset_server.load(BACKGROUND_MUSIC_FILE),
    });
}

/// 开始背景音乐（教程 20 `start_background_music`）：`OnEnter(InGame)` 生成
/// `LOOP` 音乐实体（挂 [`BackgroundMusic`]）。
pub fn start_background_music(mut commands: Commands, audio_assets: Res<AudioAssets>) {
    commands.spawn((
        AudioPlayer::new(audio_assets.background_music.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(MUSIC_VOLUME)),
        BackgroundMusic,
    ));
}

/// 停止背景音乐（教程 20 `stop_background_music`）：`OnExit(InGame)` 按标记回收。
pub fn stop_background_music(mut commands: Commands, music: Query<Entity, With<BackgroundMusic>>) {
    for entity in music.iter() {
        commands.entity(entity).despawn();
    }
}

/// 播放一次性音效（教程 20 `play_sound_effects`）：消费 [`PlaySound`] → spawn
/// `DESPAWN` 短命音频实体（播完由 Bevy 自动回收，无需手动清理）。
pub fn play_sound_effects(
    mut messages: MessageReader<PlaySound>,
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    mut log: ResMut<AudioLog>,
) {
    for message in messages.read() {
        let handle = match message.sound {
            SoundType::LaserFire => audio_assets.laser_fire.clone(),
            SoundType::RocketExplosion => audio_assets.rocket_explosion.clone(),
        };
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(message.volume)),
        ));
        match message.sound {
            SoundType::LaserFire => log.laser_fire += 1,
            SoundType::RocketExplosion => log.rocket_explosion += 1,
        }
        log.sfx_spawned += 1;
        log.last_volume = Some(message.volume);
    }
}

/// 音频插件（教程 21 Step 1 `AudioPlugin`）：把 P20 散在 `main.rs` 的六处注册
/// （消息 / 记账资源 / Startup 装载 / OnEnter 起播 / OnExit 停播 / Update 消费）
/// 收拢为一次 `.add_plugins(AudioPlugin)`。
///
/// 纯重构（教程 21「The game plays identically」）：注册面与 P20 接线逐一对应，
/// 行为零变化。注意：`play_sound_effects` **不带**暂停门控——教程 21「Audio does
/// not pause」：音乐与推进器在暂停时继续播（Bevy 音频独立于 ECS 调度）。
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySound>()
            .init_resource::<AudioLog>()
            .add_systems(Startup, load_audio_assets)
            .add_systems(OnEnter(GameState::InGame), start_background_music)
            .add_systems(OnExit(GameState::InGame), stop_background_music)
            .add_systems(
                Update,
                play_sound_effects.run_if(in_state(GameState::InGame)),
            );
    }
}
