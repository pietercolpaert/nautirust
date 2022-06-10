use std::process::Child;

use async_std::fs::read_to_string;

use super::run::Values;
use crate::channel::Channel;
use crate::runner::Runner;

/// Gracefully stop the runners and channels specified in the config
#[derive(clap::Args, Debug)]
pub struct Command {
    file: String,
}

impl Command {
    pub async fn execute(self, _channels: Vec<Channel>, runners: Vec<Runner>) {
        let content = read_to_string(self.file).await.unwrap();
        let values: Values = serde_json::from_str(&content).unwrap();

        let mut procs: Vec<Child> = Vec::new();
        let used_channels = super::get_used_channels(&content, &_channels);

        used_channels.for_each(|Channel { stop, location, .. }| {
            super::add_add_subproc(stop, location, &mut procs)
        });

        let used_runners = runners.iter().filter(|runner| {
            values
                .values
                .iter()
                .any(|v| v.processor_config.runner_id == runner.id)
        });

        used_runners.for_each(
            |Runner {
                 ref location,
                 ref stop,
                 ..
             }| {
                super::add_add_subproc(stop, location, &mut procs)
            },
        );

        // Stops the processors in the reverse order
        while !procs.is_empty() {
            procs.pop().unwrap().wait().unwrap();
        }
    }
}
