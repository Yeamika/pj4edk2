//! Validate optional firmware telemetry before delivering it to the UI.
use pocketjs_boot_context_abi::{PowerSample, POWER_PERCENT, POWER_PERCENT_ESTIMATED, POWER_EXTERNAL,
    POWER_CHARGING, POWER_VOLTAGE, POWER_CURRENT};
use pocketjs_boot_ui::PowerTelemetry;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PowerState {
    pub percent: Option<u8>,
    pub estimated: bool,
    pub external: Option<bool>,
    pub telemetry: PowerTelemetry,
}
pub fn decode(sample: &PowerSample) -> PowerState {
    if sample.size < core::mem::size_of::<PowerSample>() as u32 {
        return PowerState::default();
    }
    let percent=(sample.valid & POWER_PERCENT != 0 && sample.percent <= 100).then_some(sample.percent as u8);
    PowerState {
        percent,
        estimated: percent.is_some() && sample.valid & POWER_PERCENT_ESTIMATED != 0,
        external: (sample.valid & POWER_EXTERNAL != 0 && sample.external_power <= 1)
            .then_some(sample.external_power != 0),
        telemetry: PowerTelemetry {
            charging: (sample.valid & POWER_CHARGING != 0 && sample.charging <= 1)
                .then_some(sample.charging != 0),
            voltage_mv: (sample.valid & POWER_VOLTAGE != 0 && sample.voltage_mv > 0)
                .then_some(sample.voltage_mv),
            current_ma: (sample.valid & POWER_CURRENT != 0).then_some(sample.current_ma),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn battery_current_keeps_its_sign_and_fields_expire_independently() {
        let mut sample=PowerSample { size: core::mem::size_of::<PowerSample>() as u32,
            valid:POWER_CURRENT|POWER_VOLTAGE|POWER_CHARGING|POWER_EXTERNAL,
            voltage_mv:4152,current_ma:1620,charging:1,external_power:1,
            ..PowerSample::default() };
        assert_eq!(decode(&sample).telemetry,PowerTelemetry {
            charging:Some(true),voltage_mv:Some(4152),current_ma:Some(1620) });
        sample.current_ma=-420;sample.charging=0;
        assert_eq!(decode(&sample).telemetry,PowerTelemetry {
            charging:Some(false),voltage_mv:Some(4152),current_ma:Some(-420) });
        sample.valid=POWER_CURRENT;sample.current_ma=0;
        assert_eq!(decode(&sample).telemetry,PowerTelemetry {
            charging:None,voltage_mv:None,current_ma:Some(0) });
        sample.valid=POWER_CHARGING|POWER_VOLTAGE;sample.charging=2;sample.voltage_mv=0;
        assert_eq!(decode(&sample).telemetry,PowerTelemetry::default());
        sample.size=0;
        assert_eq!(decode(&sample),PowerState::default());
    }
    #[test]
    fn absence_and_retained_soc_never_become_a_live_percentage() {
        let mut sample=PowerSample { size: core::mem::size_of::<PowerSample>() as u32,
            percent: 83, saved_soc_raw: 0x552d, ..PowerSample::default() };
        assert_eq!(decode(&sample),PowerState::default());
        sample.valid=POWER_PERCENT|POWER_EXTERNAL; sample.external_power=1;
        assert_eq!(decode(&sample),PowerState { percent:Some(83),estimated:false,external:Some(true),..PowerState::default() });
        sample.valid|=POWER_PERCENT_ESTIMATED;
        assert_eq!(decode(&sample),PowerState { percent:Some(83),estimated:true,external:Some(true),..PowerState::default() });
        sample.percent=101; sample.external_power=2;
        assert_eq!(decode(&sample),PowerState::default());
        sample.percent=0; sample.external_power=0;
        assert_eq!(decode(&sample),PowerState { percent:Some(0),estimated:true,external:Some(false),..PowerState::default() });
        sample.size=8;
        assert_eq!(decode(&sample),PowerState::default());
    }
}
