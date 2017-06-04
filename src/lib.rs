#![feature(conservative_impl_trait)]

use std::error::Error;
use std::io::{ self, Read, Write, BufRead, BufReader };
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, channel};

mod tests;



pub trait DockerCommand<'a> {
    fn args(&self) -> Vec<&str>;
}


#[derive(Debug)]
pub struct Rocker {}
impl<'a> Rocker {
    pub fn build() -> DockerBuild<'a> {
        DockerBuild::new()
    }

    pub fn create(image: &str) -> DockerCreate {
        DockerCreate::new(image)
    }
}



#[derive(Debug)]
pub struct DockerProcessResult {
    pub output: String,
    pub exit_status: i32,
}
#[derive(Debug)]
pub struct DockerBuildResult<'a> {
    pub process: DockerProcessResult,
    pub tag: Option<&'a str>,
}
#[derive(Debug)]
pub struct DockerCreateResult {
    pub process: DockerProcessResult,
    pub container_id: String,
}



//--------------//
// Docker Build //
//--------------//
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
    pub fn init(&'a self) -> DockerBuildResult {
        let docker = Docker {
            args: self.args(),
        };

        let docker_result = docker
            .init()
            .unwrap();

        DockerBuildResult {
            process: docker_result,
            tag: self.tag,
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
}

//---------------//
// Docker Create //
//---------------//
pub struct DockerCreate {
    image: String,
}
impl DockerCreate {
    pub fn new(image: &str) -> DockerCreate {
        DockerCreate {
            image: image.to_owned(),
        }
    }
    pub fn init(&self) -> DockerCreateResult {
        let docker = Docker {
            args: self.args(),
        };

        let docker_result = docker
            .init()
            .unwrap();

        let container_id = docker_result.output.trim().to_owned();

        DockerCreateResult {
            process: docker_result,
            container_id: container_id,
        }
    }
}
impl<'a> DockerCommand<'a> for DockerCreate {
    fn args(&self) -> Vec<&str> {
        vec!["create", self.image.as_str()]
    }
}



#[derive(Debug)]
pub struct Docker<'a> {
    args: Vec<&'a str>,
}
impl<'a> Docker<'a> {
    pub fn init(&self) -> Result<DockerProcessResult, io::Error> {
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
        let process_result = DockerProcessResult {
            output: output,
            exit_status: exit_status,
        };

        Ok(process_result)
    }
}

