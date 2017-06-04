use std::error::Error;
use std::io::{ self, Read, Write, BufRead, BufReader };
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, channel};

mod tests;



pub trait DockerCommand<'a> {
    fn init(&'a self) -> DockerResult;
    fn args(&self) -> Vec<&str>;
}



#[derive(Debug)]
pub struct Rocker {}
impl<'a> Rocker {
    pub fn build() -> DockerBuild<'a> {
        DockerBuild::new()
    }
}



#[derive(Debug)]
pub struct DockerResult {
    pub output: String,
    pub exit_status: i32
}



#[derive(Debug)]
pub struct DockerBuild<'a> {
    context: &'a str,
    file: &'a str,
    tag: Option<&'a str>,
}
impl<'a> DockerBuild<'a> {
    pub fn new() -> DockerBuild<'a> {
        DockerBuild {
            context: ".",
            file: "Dockerfile",
            tag: None,
        }
    }
    pub fn context(&self, context: &'a str) -> DockerBuild<'a> {
        DockerBuild {
            context: context,
            file: self.file,
            tag: self.tag,
        }
    }
    pub fn file(&self, file: &'a str) -> DockerBuild<'a> {
        DockerBuild {
            context: self.context,
            file: file,
            tag: self.tag,
        }
    }
    pub fn tag(&self, tag: &'a str) -> DockerBuild<'a> {
        DockerBuild {
            context: self.context,
            file: self.file,
            tag: Some(tag),
        }
    }
}
impl<'a> DockerCommand<'a> for DockerBuild<'a> {
    fn args(&self) -> Vec<&str> {
        let mut args = vec!["build", "-f", self.file];

        match self.tag {
            Some(tag) => { args.push("-t"); args.push(tag) },
            None => (),
        }

        args.push(self.context);

        args
    }

    fn init(&'a self) -> DockerResult {
        let args = self.args();
        let docker = Docker {
            args: args,
        };
        docker.init().unwrap()
    }
}



#[derive(Debug)]
pub struct Docker<'a> {
    args: Vec<&'a str>
}
impl<'a> Docker<'a> {
    pub fn init(&self) -> Result<DockerResult, io::Error> {
        println!("Running command: docker {}", &self.args.join(" "));
        let mut process = Command::new("docker")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&self.args)
            .spawn().unwrap();

        let stdout = process.stdout.take().unwrap();
        let stderr = process.stderr.take().unwrap();

        let reader = BufReader::new(stdout.chain(stderr));
        let mut output = String::new();

        for line in reader.lines() {
            let next = line.unwrap();
            output.push_str("\n");
            output.push_str(next.trim());
            println!("|  {}", next);
        }

        let exit_status = process.wait()?.code().unwrap_or(1);

        Ok(
            DockerResult {
                output: output,
                exit_status: exit_status,
            }
        )
    }
}

