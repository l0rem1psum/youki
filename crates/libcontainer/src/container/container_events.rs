use std::{thread, time::Duration};

use crate::error::LibcontainerError;

use super::{Container, ContainerStatus};
use libcgroups::common::CgroupManager;

impl Container {
    /// Displays container events
    ///
    /// # Example
    ///
    /// ```no_run
    /// use libcontainer::container::builder::ContainerBuilder;
    /// use libcontainer::syscall::syscall::create_syscall;
    /// use libcontainer::workload::default::DefaultExecutor;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut container = ContainerBuilder::new(
    ///     "74f1a4cb3801".to_owned(),
    ///     create_syscall().as_ref(),
    /// )
    /// .as_init("/var/run/docker/bundle")
    /// .build()?;
    ///
    /// container.events(5000, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn events(&mut self, interval: u32, stats: bool) -> Result<(), LibcontainerError> {
        self.refresh_status()?;
        if !self.state.status.eq(&ContainerStatus::Running) {
            tracing::error!(id = ?self.id(), status = ?self.state.status, "container is not running");
            return Err(LibcontainerError::IncorrectStatus);
        }

        let cgroups_path = self.spec()?.cgroup_path;
        let use_systemd = self.systemd();

        let cgroup_manager =
            libcgroups::common::create_cgroup_manager(cgroups_path, use_systemd, self.id())?;
        match stats {
            true => {
                let stats = cgroup_manager.stats()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&stats)
                        .map_err(LibcontainerError::OtherSerialization)?
                );
            }
            false => loop {
                let stats = cgroup_manager.stats()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&stats)
                        .map_err(LibcontainerError::OtherSerialization)?
                );
                thread::sleep(Duration::from_secs(interval as u64));
            },
        }

        Ok(())
    }
}
