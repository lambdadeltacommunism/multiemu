use super::Executor;
use crate::{component::memory::MemoryTranslationTable, task::Task};
use itertools::Itertools;
use num::{integer::lcm, ToPrimitive};
use num::{rational::Ratio, Integer};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub struct SingleThreadedExecutor {
    tasks: Vec<(u32, Box<dyn Task>)>,
    memory_translation_table: Arc<MemoryTranslationTable>,
    timestamp: Instant,
    current_tick: u32,
    rollover_tick: u32,
    tick_real_time: Ratio<u32>,
}

impl SingleThreadedExecutor {
    fn increment_tick(&mut self, amount: u32) {
        let new_tick = (self.current_tick + amount) % self.rollover_tick;

        if new_tick < self.current_tick {
            self.timestamp = Instant::now();
        }

        self.current_tick = new_tick;
    }
}

impl Executor for SingleThreadedExecutor {
    fn new(
        tasks: Vec<(Ratio<u32>, Box<dyn Task>)>,
        memory_translation_table: Arc<MemoryTranslationTable>,
    ) -> Self {
        let (rollover_tick, task_tick_rates, tick_real_time) =
            find_component_timings(&tasks.iter().map(|(ratio, _)| *ratio).collect::<Vec<_>>());

        tracing::info!(
            "A tick on this machine is a real world {:?}",
            Duration::from_secs_f32(tick_real_time.to_f32().unwrap())
        );

        Self {
            tasks: tasks
                .into_iter()
                .zip(task_tick_rates)
                .map(|((_, task), tick_rate)| (tick_rate, task))
                .collect(),
            memory_translation_table,
            timestamp: Instant::now(),
            current_tick: 0,
            rollover_tick,
            tick_real_time,
        }
    }

    fn run(&mut self, period: Duration) {
        let start_time = Instant::now();

        loop {
            let now = Instant::now();
            // Exit if the runtime does not allow us any more time
            let runtime_assigned_time_left = period.saturating_sub(now - start_time);
            if runtime_assigned_time_left.is_zero() {
                break;
            }

            // Exit if we are ahead of time
            let simulated_time = Duration::from_secs_f32(
                self.current_tick as f32 * self.tick_real_time.to_f32().unwrap(),
            );
            let real_time = now - self.timestamp;
            if simulated_time > real_time {
                break;
            }

            let max_batch_size = ((runtime_assigned_time_left.as_secs_f32()
                / self.tick_real_time.to_f32().unwrap())
            .floor() as u32)
                .clamp(1, (self.rollover_tick - self.current_tick).max(1));

            // Sort all the components
            let mut to_run: Vec<_> = self
                .tasks
                .iter_mut()
                .map(|(tick_rate, task)| (*tick_rate, self.current_tick % *tick_rate, task))
                .sorted_by_key(|(_, run_indication, _)| *run_indication)
                .collect();

            if to_run.is_empty() || to_run[0].1 != 0 {
                self.increment_tick(1);
                continue;
            }

            // We can do a special case here projecting this to infinity
            if to_run.len() == 1 {
                let (tick_rate, _, task) = &mut to_run[0];
                let batch_size = max_batch_size / *tick_rate;
                task.tick(batch_size, &self.memory_translation_table);
                self.increment_tick(max_batch_size);
                continue;
            }

            // time slicing not possible
            if to_run[1..]
                .iter()
                .any(|(_, run_indication, _)| *run_indication == 0)
            {
                for (_, _, task) in to_run
                    .into_iter()
                    .filter(|(_, run_indication, _)| *run_indication == 0)
                {
                    task.tick(1, &self.memory_translation_table);
                }

                self.increment_tick(1);
                continue;
            }

            // We can batch normally here
            let batch_size = (to_run[1].0 - to_run[1].1).min(max_batch_size);
            let (tick_rate, _, task) = &mut to_run[0];
            let normalized_batch_size = batch_size / *tick_rate;
            task.tick(normalized_batch_size, &self.memory_translation_table);
            self.increment_tick(batch_size);
        }
    }
}

fn find_component_timings(ratios: &[Ratio<u32>]) -> (u32, Vec<u32>, Ratio<u32>) {
    // Get the least common multiple of all denominators
    let common_denominator = ratios
        .iter()
        .map(|ratio| *ratio.denom())
        .fold(1u32, |acc, denom| acc.lcm(&denom));

    // Adjust numerators to the common denominator
    let adjusted_numerators: Vec<_> = ratios
        .iter()
        .map(|ratio| {
            let factor = common_denominator / ratio.denom();
            ratio.numer() * factor
        })
        .collect();

    // Get the least common multiple of the adjusted numerators
    let common_multiple = adjusted_numerators.clone().into_iter().reduce(lcm).unwrap();

    (
        common_multiple,
        adjusted_numerators
            .iter()
            .map(|numerator| common_multiple / numerator)
            .collect(),
        Ratio::new(common_multiple, common_denominator).recip(),
    )
}
