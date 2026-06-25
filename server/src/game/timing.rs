use std::time::Duration;

pub(super) fn duration_for(time_units: u64, frequency: usize) -> Duration {
    if time_units == 0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f64(time_units as f64 / frequency as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::duration_for;
    use std::time::Duration;

    #[test]
    fn converts_time_units_to_duration() {
        assert_eq!(duration_for(7, 100), Duration::from_millis(70));
    }
}
