use std;
use super::super::lexer::Location;

#[derive(Debug, PartialEq)]
pub enum ExecError<'a> {
    StaticAnalysisFailed(Vec<StaticAnalysisError<'a>>),
}
pub type ExecResult<'a, T> = std::result::Result<T, ExecError<'a>>;

#[derive(Debug, PartialEq)]
pub enum StaticAnalysisError<'a> {
    CallUnknownFunction(&'a str, Location<'a>, Location<'a>),
}

pub type StaticAnalysisErrors<'a> = Vec<StaticAnalysisError<'a>>;
pub type BuildResult<'a, T> = (T, StaticAnalysisErrors<'a>);
