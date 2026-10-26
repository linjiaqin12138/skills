//! 跌倒预测与落地检测。预测器只看新鲜、已收敛且允许反射的样本；
//! 任何不合格拍都清去抖，避免把几段独立倾斜拼成一次跌倒。

use crate::io::ImuData;

const TILT_Z: f64 = -0.90;
const PREDICT_Z: f64 = -0.5;
const LOOKAHEAD_SECONDS: f64 = 0.3;
const PREDICT_TICKS: u64 = 3; // 60ms / 20ms，见上游 limp_fall 默认配置。
const STILL_RATE: f64 = 1.0;
const STILL_TICKS: u64 = 10; // 200ms / 20ms。
pub const LIMP_MAX_TICKS: u64 = 75; // 1500ms / 20ms。

/// 投影重力 z 的解析导数：`-ω × g` 的 z 分量。
pub fn gravity_z_rate(gyro: &[f64; 3], gravity: &[f64; 3]) -> f64 {
    -(gyro[0] * gravity[1] - gyro[1] * gravity[0])
}

#[derive(Debug, Default)]
pub struct FallPredictor {
    falling_ticks: u64,
    fired: bool,
}

impl FallPredictor {
    pub fn reset(&mut self) {
        self.falling_ticks = 0;
        self.fired = false;
    }

    /// 返回仅在触发沿为 true；持续跌倒不会重复触发。条件断开后才重新武装。
    pub fn update(&mut self, imu: &ImuData, eligible: bool) -> bool {
        if !eligible {
            self.reset();
            return false;
        }

        let rate = gravity_z_rate(&imu.gyro, &imu.gravity);
        let predicted = imu.gravity[2] + rate * LOOKAHEAD_SECONDS;
        let falling = imu.gravity[2] > TILT_Z && rate > 0.0 && predicted > PREDICT_Z;
        if !falling {
            self.reset();
            return false;
        }
        if self.fired {
            return false;
        }

        self.falling_ticks += 1;
        if self.falling_ticks == PREDICT_TICKS {
            self.fired = true;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct LandingDetector {
    still_ticks: u64,
}

impl LandingDetector {
    pub fn reset(&mut self) {
        self.still_ticks = 0;
    }

    /// `fresh=false` 是滑行样本，`gyro=None` 是真瞎；两者都不能证明静止。
    pub fn update(&mut self, gyro: Option<&[f64; 3]>, fresh: bool) -> bool {
        let still = fresh
            && gyro.is_some_and(|g| g.iter().map(|v| v * v).sum::<f64>().sqrt() < STILL_RATE);
        if !still {
            self.reset();
            return false;
        }
        self.still_ticks += 1;
        self.still_ticks >= STILL_TICKS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn falling() -> ImuData {
        ImuData {
            gyro: [0.0, 4.0, 0.0],
            gravity: [0.4, 0.0, -0.8],
            ..ImuData::default()
        }
    }

    fn fire_on_third_tick(predictor: &mut FallPredictor) {
        assert!(!predictor.update(&falling(), true));
        assert!(!predictor.update(&falling(), true));
        assert!(predictor.update(&falling(), true));
        assert!(!predictor.update(&falling(), true), "持续跌倒只触发一次");
    }

    #[test]
    fn analytic_gravity_z_derivative() {
        let gyro = [0.3, -0.4, 0.5];
        let gravity = [0.2, -0.7, -0.6];
        assert!((gravity_z_rate(&gyro, &gravity) - 0.13).abs() < 1e-12);
    }

    fn eight_degree_lean() -> ImuData {
        let angle = 8.0f64.to_radians();
        ImuData {
            gravity: [angle.sin(), 0.0, -angle.cos()],
            ..ImuData::default()
        }
    }

    #[test]
    fn footfall_impulse_at_eight_degrees_never_fires() {
        let mut predictor = FallPredictor::default();
        let footfall = ImuData {
            gyro: [0.0, 3.0, 0.0],
            ..eight_degree_lean()
        };
        let rate = gravity_z_rate(&footfall.gyro, &footfall.gravity);
        assert!(footfall.gravity[2] + rate * LOOKAHEAD_SECONDS < PREDICT_Z);
        for _ in 0..10 {
            assert!(!predictor.update(&footfall, true));
        }
    }

    #[test]
    fn static_tilt_does_not_fire() {
        let mut predictor = FallPredictor::default();
        for _ in 0..20 {
            assert!(!predictor.update(&eight_degree_lean(), true));
        }
    }

    #[test]
    fn slow_lean_prediction_stays_below_threshold() {
        let mut predictor = FallPredictor::default();
        let slow_lean = ImuData {
            gyro: [0.0, 0.3, 0.0],
            ..eight_degree_lean()
        };
        let rate = gravity_z_rate(&slow_lean.gyro, &slow_lean.gravity);
        assert!(slow_lean.gravity[2] + rate * LOOKAHEAD_SECONDS < PREDICT_Z);
        for _ in 0..20 {
            assert!(!predictor.update(&slow_lean, true));
        }
    }

    #[test]
    fn fast_fall_fires_once_then_rearms_on_break_or_reset() {
        let mut predictor = FallPredictor::default();
        fire_on_third_tick(&mut predictor);

        assert!(!predictor.update(&ImuData::default(), true));
        fire_on_third_tick(&mut predictor);

        predictor.reset();
        fire_on_third_tick(&mut predictor);
    }

    #[test]
    fn ineligible_tick_resets_prediction_debounce() {
        let mut predictor = FallPredictor::default();
        assert!(!predictor.update(&falling(), true));
        assert!(!predictor.update(&falling(), true));
        assert!(!predictor.update(&falling(), false));
        assert!(!predictor.update(&falling(), true));
        assert!(!predictor.update(&falling(), true));
        assert!(predictor.update(&falling(), true));
    }

    #[test]
    fn landing_requires_ten_fresh_still_samples() {
        let mut landing = LandingDetector::default();
        let still = [0.2, -0.2, 0.1];
        for _ in 0..9 {
            assert!(!landing.update(Some(&still), true));
        }
        assert!(landing.update(Some(&still), true));
    }

    #[test]
    fn moving_coasted_and_blind_samples_reset_stillness() {
        let mut landing = LandingDetector::default();
        let still = [0.0; 3];
        for reset_sample in [
            (Some([2.0, 0.0, 0.0]), true),
            (Some([0.0, 0.0, 0.0]), false),
        ] {
            for _ in 0..9 {
                assert!(!landing.update(Some(&still), true));
            }
            assert!(!landing.update(reset_sample.0.as_ref(), reset_sample.1));
            for _ in 0..9 {
                assert!(!landing.update(Some(&still), true));
            }
            assert!(landing.update(Some(&still), true));
            landing.reset();
        }

        for _ in 0..9 {
            assert!(!landing.update(Some(&still), true));
        }
        assert!(!landing.update(None, false), "盲拍不算静止");
        for _ in 0..9 {
            assert!(!landing.update(Some(&still), true));
        }
        assert!(landing.update(Some(&still), true));
    }
}
