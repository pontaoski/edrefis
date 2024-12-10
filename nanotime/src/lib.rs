// SPDX-FileCopyrightText: 2022 Brandon McGriff <nightmareci@gmail.com>
// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MIT OR Unlicense

use std::time::{Instant, Duration};

fn now() -> Instant {
    Instant::now()
}

fn sleep(duration: Duration) {
    if duration.is_zero() {
        yield_time();
    } else {
        std::thread::sleep(duration);
    }
}

fn yield_time() {
    std::thread::yield_now();
}

fn interval(start: Instant, end: Instant) -> Duration {
    end.duration_since(start)
}

pub struct StepData {
    sleep_duration: Duration,

    zero_sleep_duration: Duration,
    accumulator: Duration,
    sleep_point: Instant,
}

const NSEC_PER_SEC: u64 = 1000000000;
const NSEC_PER_SEC_DIV_10: Duration = Duration::from_nanos(NSEC_PER_SEC / 10);
const NSEC_PER_SEC_DIV_1000: Duration = Duration::from_nanos(NSEC_PER_SEC / 1000);
const ZENOS_DIVISOR: u32 = 16;

impl StepData {
    pub fn new(sleep_duration: Duration) -> StepData {
        let start = now();
        sleep(Duration::ZERO);

        StepData {
            sleep_duration,
            zero_sleep_duration: interval(start, now()),
            accumulator: Duration::ZERO,
            sleep_point: now(),
        }
    }
    fn step_inner(&mut self, total_sleep_duration: Duration, start_point: Instant) {
        let mut current_sleep_duration = total_sleep_duration;

        {
            let mut max = NSEC_PER_SEC_DIV_1000;
            let mut start = now();
            while interval(self.sleep_point, start) + max < total_sleep_duration {
                sleep(NSEC_PER_SEC_DIV_1000);
                let next = now();
                let current_interval = interval(start, next);
                if current_interval > max {
                    max = current_interval;
                }
                start = next;
            }
            let initial_duration = interval(start_point, now());
            if initial_duration < current_sleep_duration {
                current_sleep_duration -= initial_duration;
            } else {
                return;
            }
        }

        current_sleep_duration /= ZENOS_DIVISOR;
        let mut max = self.zero_sleep_duration;
        while interval(self.sleep_point, now()) + max < total_sleep_duration && current_sleep_duration > Duration::ZERO {
            max = self.zero_sleep_duration;
            let mut start: Instant;

            while {
                if max < self.sleep_duration {
                    start = now();
                    interval(self.sleep_point, start) + max < total_sleep_duration
                } else {
                    start = now();
                    false
                }
            } {
                sleep(current_sleep_duration);
                let slept_duration = interval(start, now());
                if slept_duration > max {
                    max = slept_duration;
                }
            }

            current_sleep_duration /= ZENOS_DIVISOR;
        }
        if interval(self.sleep_point, now()) >= total_sleep_duration {
            return;
        }

        {
            let mut max = self.zero_sleep_duration;
            let mut start: Instant;

            while {
                start = now();
                interval(self.sleep_point, start) + max < total_sleep_duration
            } {
                sleep(Duration::ZERO);
                self.zero_sleep_duration = interval(start, now());
                if self.zero_sleep_duration > max {
                    max = self.zero_sleep_duration;
                }
            }
        }
    }
    pub fn step(&mut self) -> bool {
        let start_point = now();
        if interval(self.sleep_point, start_point) >= self.sleep_duration + NSEC_PER_SEC_DIV_10 {
            self.sleep_point = start_point;
            self.accumulator = Duration::ZERO;
        }

        let slept: bool;
        if self.accumulator < self.sleep_duration {
            let total_sleep_duration = self.sleep_duration - self.accumulator;
            self.step_inner(total_sleep_duration, start_point);

            // step_end
            let mut current_time: Instant;
            let mut accumulated: Duration;
            while {
                current_time = now();
                accumulated = interval(self.sleep_point, now());
                accumulated < total_sleep_duration
            } {}

            self.accumulator += accumulated;
            self.sleep_point = current_time;
            slept = true;
        } else {
            slept = false;
        }

        self.accumulator -= self.sleep_duration;
        slept
    }
}
