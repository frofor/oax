#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

pub(crate) mod api;
pub(crate) mod cli;
pub(crate) mod cmd;
pub(crate) mod spec;

use clap_complete::CompleteEnv;
use cli::DELETE;
use cli::GET;
use cli::HEAD;
use cli::OPTIONS;
use cli::PATCH;
use cli::POST;
use cli::PUT;
use cli::TRACE;
use cli::cmd;

fn main() {
	CompleteEnv::with_factory(cmd).complete();

	let matches = cmd().get_matches();
	let Some((cmd, sub_matches)) = matches.subcommand() else {
		return;
	};

	match cmd {
		GET => cmd::get(&matches, sub_matches),
		POST => unimplemented!(),
		PUT => unimplemented!(),
		DELETE => unimplemented!(),
		OPTIONS => unimplemented!(),
		HEAD => unimplemented!(),
		PATCH => unimplemented!(),
		TRACE => unimplemented!(),
		_ => unreachable!(),
	}
}
