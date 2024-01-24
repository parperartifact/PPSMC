use std::time::Duration;

#[derive(Debug, Default)]
pub struct Statistic {
    pub post_reachable_time: Duration,
    pub post_image_time: Duration,
    pub post_propagate_time: Duration,
    pub fair_cycle_time: Duration,
    pub pre_image_time: Duration,
    pub pre_propagate_time: Duration,
    pub test_a: Duration,
}
