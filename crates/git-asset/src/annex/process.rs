//! Bounded Windows job-object process boundary; callers fix command vocabulary.
use super::*;
use process_wrap::tokio::{CommandWrap, KillOnDrop};
use std::{io, process::Stdio, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt};

pub(super) struct Output {
    pub stdout: Vec<u8>,
}
pub(super) fn supported() -> bool {
    cfg!(windows) && tokio::runtime::Handle::try_current().is_err()
}
pub(super) fn run(p: &Profile, annex: bool, args: &[&str]) -> Result<Output> {
    run_inner(p, annex, args, None)
}
#[cfg(feature = "annex-publish")]
pub(super) fn run_for_object(
    p: &Profile,
    object: &Object,
    annex: bool,
    args: &[&str],
) -> Result<Output> {
    let scope = object_scope(p, object)?;
    run_inner(p, annex, args, Some(&scope))
}
#[cfg(feature = "annex-publish")]
fn object_scope(p: &Profile, object: &Object) -> Result<Digest> {
    let store = Reader::new(p.clone())?.store();
    let bytes = git_asset_core::identity::canonical_json(&serde_json::json!([
        "git-asset.native-command.v1",
        store,
        object
    ]))
    .map_err(|_| Error::Invalid)?;
    Ok(Digest::blob(&bytes))
}
fn run_inner(p: &Profile, annex: bool, args: &[&str], scope: Option<&Digest>) -> Result<Output> {
    if !supported() {
        return Err(Error::Unsupported);
    }
    if std::fs::metadata(&p.empty_config)
        .map_err(|_| Error::Observation)?
        .len()
        != 0
        || std::fs::read_dir(&p.empty_hooks)
            .map_err(|_| Error::Observation)?
            .next()
            .is_some()
        || !p.temporary.is_dir()
    {
        return Err(Error::Invalid);
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Observation)?;
    runtime.block_on(execute(p, annex, args, true, scope))
}
#[cfg(all(test, windows))]
pub(super) fn fixture_command(p: &Profile, annex: bool, args: &[&str]) -> Result<Output> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Observation)?;
    runtime.block_on(execute(p, annex, args, false, None))
}
async fn execute(
    p: &Profile,
    annex: bool,
    args: &[&str],
    strict_stderr: bool,
    scope: Option<&Digest>,
) -> Result<Output> {
    let path = std::env::join_paths(&p.tools).map_err(|_| Error::Invalid)?;
    let mut wrap = CommandWrap::with_new(if annex { &p.annex } else { &p.git }, |c| {
        c.args(args)
            .current_dir(&p.repository)
            .env_clear()
            .env("PATH", path)
            .env("TEMP", &p.temporary)
            .env("TMP", &p.temporary)
            .env("USERPROFILE", &p.temporary)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", &p.empty_config)
            .env("GIT_CONFIG_COUNT", "4")
            .env("GIT_CONFIG_KEY_0", "core.hooksPath")
            .env("GIT_CONFIG_VALUE_0", &p.empty_hooks)
            .env("GIT_CONFIG_KEY_1", "core.fsmonitor")
            .env("GIT_CONFIG_VALUE_1", "false")
            .env("GIT_CONFIG_KEY_2", "credential.helper")
            .env("GIT_CONFIG_VALUE_2", "")
            .env("GIT_CONFIG_KEY_3", "annex.alwayscommit")
            .env("GIT_CONFIG_VALUE_3", "false")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ALLOW_PROTOCOL", "file")
            .env("GIT_DIR", if p.bare { "." } else { ".git" })
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if !p.bare {
            c.env("GIT_WORK_TREE", ".");
        }
        if let Some(value) = std::env::var_os("SystemRoot") {
            c.env("SystemRoot", value);
        }
    });
    wrap.wrap(KillOnDrop);
    #[cfg(windows)]
    let job = scope
        .map(|scope| git_asset_windows_process::Job::join(scope.as_str()).map(std::sync::Arc::new))
        .transpose()
        .map_err(|_| Error::Observation)?;
    #[cfg(windows)]
    if let Some(job) = &job
        && job.active().map_err(|_| Error::Observation)? != 0
    {
        // Never race an orphan of an earlier attempt. Caller must inspect/reconcile it.
        return Err(Error::Observation);
    }
    #[cfg(not(windows))]
    let _ = scope;
    #[cfg(windows)]
    {
        use process_wrap::tokio::{CreationFlags, JobObject};
        let mut flags = CreationFlags(Default::default());
        flags.0.0 = 0x08000000;
        wrap.wrap(flags);
        // Pinned process-wrap traverses registration order: assignment runs while
        // JobObject's pre_spawn suspension is still in effect, before its wrap_child resumes.
        if let Some(job) = &job {
            wrap.wrap(Assignment(job.clone()));
        }
        wrap.wrap(JobObject);
    }
    let mut child = wrap.spawn().map_err(|_| Error::Observation)?;
    let stdout = child.stdout().take().ok_or(Error::Observation)?;
    let stderr = child.stderr().take().ok_or(Error::Observation)?;
    let observed = tokio::time::timeout(Duration::from_millis(p.command_millis), async {
        tokio::try_join!(
            capture(stdout, p.output_bytes),
            capture(stderr, p.output_bytes),
            async {
                let status = child.wait().await?;
                // The pinned wrapper may return after the immediate child or a
                // notification. This continuously held job, not that return value,
                // establishes that all scoped descendants have finished.
                #[cfg(windows)]
                if let Some(job) = &job {
                    while job.active()? != 0 {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
                Ok::<_, io::Error>(status)
            }
        )
    })
    .await;
    match observed {
        Ok(Ok((stdout, stderr, status))) => {
            if status.success() && (!strict_stderr || stderr.is_empty()) {
                Ok(Output { stdout })
            } else if annex && status.code() == Some(1) && stdout.is_empty() && stderr.is_empty() {
                Err(Error::Missing)
            } else {
                Err(Error::Observation)
            }
        }
        failure => {
            let _ = child.start_kill();
            let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
            match failure {
                Ok(Err(e)) if e.kind() == io::ErrorKind::OutOfMemory => Err(Error::Budget),
                _ => Err(Error::Observation),
            }
        }
    }
}

#[cfg(windows)]
#[derive(Debug)]
struct Assignment(std::sync::Arc<git_asset_windows_process::Job>);
#[cfg(windows)]
impl process_wrap::tokio::CommandWrapper for Assignment {
    fn wrap_child(
        &mut self,
        mut child: Box<dyn process_wrap::tokio::ChildWrapper>,
        core: &CommandWrap,
    ) -> io::Result<Box<dyn process_wrap::tokio::ChildWrapper>> {
        let assigned = if core.has_wrap::<process_wrap::tokio::JobObject>() {
            child
                .process_handle()
                .ok_or_else(|| io::Error::from(io::ErrorKind::Unsupported))
                .and_then(|handle| self.0.assign(handle))
        } else {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        };
        if let Err(error) = assigned {
            let _ = child.start_kill();
            return Err(error);
        }
        Ok(child)
    }
}

async fn capture(reader: impl AsyncRead + Unpin, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = vec![];
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() > limit {
        return Err(io::Error::from(io::ErrorKind::OutOfMemory));
    }
    Ok(bytes)
}

#[cfg(all(test, windows, feature = "annex-publish"))]
mod tests;
