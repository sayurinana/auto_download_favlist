use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn run_bbdown(bvid: &str, work_dir: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "[dry-run] bbdown {} --work-dir {}",
            bvid,
            work_dir.display()
        );
        return Ok(());
    }

    let status = Command::new("bbdown")
        .arg(bvid)
        .arg("--work-dir")
        .arg(work_dir)
        .status()
        .with_context(|| "执行 bbdown 失败")?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        bail!("bbdown 执行失败，退出码 {code}");
    }

    Ok(())
}
