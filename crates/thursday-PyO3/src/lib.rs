use anyhow::Result;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "re_rs")]
mod rs_rs {
    use super::*;

    #[pyclass]
    struct Pattern {
        re: regex::Regex,
    }

    #[pyfunction]
    fn compile(pattern: &str) -> Result<Pattern> {
        let re = regex::Regex::new(pattern)?;
        Ok(Pattern { re })
    }

    #[pymethods]
    impl Pattern {
        fn search<'a>(&self, string: &'a str) -> Result<Option<&'a str>> {
            Ok(self.re.find(string).map(|matched| matched.as_str()))
        }
    }
}
