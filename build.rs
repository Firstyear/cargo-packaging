#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

include!("src/cli_opt.rs");

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };

    let comp_dir = PathBuf::from(outdir)
        .ancestors()
        .nth(2)
        .map(|p| p.join("completions"))
        .expect("Unable to process completions path");

    if !comp_dir.exists() {
        std::fs::create_dir(&comp_dir).expect("Unable to create completions dir");
    }

    generate_to(
        Shell::Bash,
        &mut Opt::command(),
        "rust-rpm-prov",
        comp_dir.clone(),
    )
    .ok();
    generate_to(
        Shell::Zsh,
        &mut Opt::command(),
        "rust-rpm-prov",
        comp_dir.clone(),
    )
    .ok();

    // clap_complete 3.2 derives the bash completion function name verbatim from
    // the bin name, producing `_rust-rpm-prov()`. Hyphens are not valid in POSIX
    // shell identifiers, so sourcing the file under /bin/sh (POSIX mode) fails
    // with "`_rust-rpm-prov': not a valid identifier". This surfaces as noise
    // during rpm %post scriptlets that re-source /etc/bash_completion.d/.
    // Rewrite the function name and the `complete -F` reference to use
    // underscores. Upstream fix tracked at clap-rs/clap#6421 / #6428.
    let bash_comp = comp_dir.join("rust-rpm-prov.bash");
    if let Ok(content) = fs::read_to_string(&bash_comp) {
        let sanitized = content
            .replace("_rust-rpm-prov()", "_rust_rpm_prov()")
            .replace("complete -F _rust-rpm-prov", "complete -F _rust_rpm_prov");
        fs::write(&bash_comp, sanitized).expect("Unable to sanitize bash completion");
    }
}
