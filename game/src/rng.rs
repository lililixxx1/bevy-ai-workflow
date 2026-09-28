//! 确定性伪随机源：SplitMix64（公开算法，无外部依赖）。
//!
//! 选型理由：不引入 `rand` 依赖；种子 → 输出序列是纯数学映射，
//! 跨平台/跨进程可复现（模拟确定性口径的基础，见 `sim.rs`）。
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// 下一个 u64（标准 SplitMix64 输出函数）。
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// [0, 1) 均匀 f32（取高 24 位，f32 尾数宽度）。
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// 当前内部状态（M4 战斗回合机：RNG 进度存进 `BattleState` 资源，
    /// 跨 BRP 请求续跑——同种子 + 同操作序列 → 同抽取序列的基础）。
    pub fn state(&self) -> u64 {
        self.0
    }

    /// 从已保存状态恢复（`from_state(s).state() == s`；与 `new` 无行为差异，
    /// 语义上是「续跑」而非「重新播种」）。
    pub fn from_state(state: u64) -> Self {
        Self(state)
    }

    /// [-half_width, half_width) 均匀 f32。
    pub fn next_range_f32(&mut self, half_width: f32) -> f32 {
        (self.next_f32() * 2.0 - 1.0) * half_width
    }
}

/// 种子×实体索引 → 相位扰动（哈希混合，同种子同索引恒等输出）。
/// 用于把「每实体相位」绑定到 (seed, index) 而非 PRNG 抽取顺序。
pub fn phase_hash(seed: u64, index: u32) -> f32 {
    let mut r = SplitMix64::new(seed ^ (0x5DEE_CE66_u64 << 32) ^ index as u64);
    r.next_f32() * std::f32::consts::TAU
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_reference_vectors() {
        // 参考向量由独立实现（node BigInt，同算法常数）计算，防手误；
        // 初版凭记忆写的向量有两处错误，正是「禁凭记忆」纪律的实证。
        let mut r = SplitMix64::new(0);
        assert_eq!(r.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(r.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(r.next_u64(), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn next_f32_in_unit_interval() {
        let mut r = SplitMix64::new(20260926);
        for _ in 0..1000 {
            let v = r.next_f32();
            assert!((0.0..1.0).contains(&v), "v={v}");
        }
    }

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn from_state_resumes_exact_sequence() {
        // 跑 3 步 → 存状态 → 两个分叉（原实例 / from_state 恢复实例）序列一致。
        let mut a = SplitMix64::new(7);
        for _ in 0..3 {
            a.next_u64();
        }
        let mut b = SplitMix64::from_state(a.state());
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
