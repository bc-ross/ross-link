use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use anyhow::anyhow;
use ross_core::schedule::CourseCodeSuffix;
use ross_core::MAX_CREDITS_PER_SEMESTER;
use std::collections::HashMap;
use std::path::Path;

use ross_core::load_catalogs::CATALOGS;
use ross_core::model::generate_multi_schedules;
use ross_core::read_excel_file::read_file;
use ross_core::schedule::generate_schedule;
use ross_core::schedule::CourseCode;
use ross_core::schedule::Schedule as RossSchedule;
use ross_core::write_excel_file::save_schedule;
use ross_core::CC;

macro_rules! WE {
    ($x:expr) => {
        $x.map_err(|e| PyRuntimeError::new_err(format!("Rust error: {}", e)))?
    };
}

#[pyclass]
struct Schedule(RossSchedule);

#[pymethods]
impl Schedule {
    #[new]
    fn new(programs: Vec<String>, incoming: Vec<String>) -> PyResult<Self> {
        let sched = WE!(generate_schedule(
            programs.iter().map(|x| x.as_str()).collect(),
            WE!(CATALOGS.first().ok_or(anyhow!("no catalogs found"))).clone(),
            Some(
                incoming
                    .into_iter()
                    .map(|x| {
                        let parts = x.split("-").collect::<Vec<_>>();
                        CourseCode {
                            stem: parts[0].into(),
                            code: if let Some(y) = parts[1].parse::<u32>().ok() {
                                CourseCodeSuffix::Number(y.try_into().unwrap())
                            } else {
                                CourseCodeSuffix::Special(parts[1].into())
                            },
                        }
                    })
                    .collect()
            )
        ));
        Ok(Schedule(sched))
    }

    pub fn validate(&mut self) -> PyResult<()> {
        let sched = &mut self.0;
        WE!(sched.validate());
        Ok(())
    }

    pub fn is_valid(&self) -> PyResult<bool> {
        let sched = &self.0;
        Ok(WE!(sched.is_valid()))
    }

    pub fn display(&self) -> PyResult<()> {
        let sched = &self.0;
        println!("Final schedule (two-stage, balanced):");
        let mut sched_credits = 0;
        for (s, semester) in std::iter::once(&sched.incoming)
            .chain(sched.courses.iter())
            .enumerate()
        {
            if s == 0 {
                println!("Semester 0 (incoming only):");
            } else {
                println!("Semester {s}");
            }
            let mut sem_credits = 0;
            for code in semester {
                let credits = sched
                    .catalog
                    .courses
                    .get(code)
                    .and_then(|(_, cr, _)| *cr)
                    .unwrap_or(0);
                println!("  {code} ({credits} credits)");
                sem_credits += credits;
            }
            println!("  Credits: {sem_credits}");
            if s > 0 {
                sched_credits += sem_credits;
            }
        }
        println!("Total credits (excluding incoming): {sched_credits}");

        Ok(())
    }

    pub fn save(&self, fname: String) -> PyResult<()> {
        WE!(save_schedule(&Path::new(&fname).to_path_buf(), &self.0));
        Ok(())
    }

    #[staticmethod]
    fn from_file(fname: String) -> PyResult<Self> {
        Ok(Schedule(WE!(read_file(&Path::new(&fname).to_path_buf()))))
    }

    pub fn get_courses(&self) -> PyResult<HashMap<String, Vec<(String, PyObject, Option<u32>)>>> {
        let sched = &self.0;
        let courses = Python::with_gil(|py| {
            let mut courses = HashMap::new();
            for (i, semester) in std::iter::once(&sched.incoming)
                .chain(sched.courses.iter())
                .enumerate()
            {
                let mut sem_courses = Vec::new();
                for code in semester {
                    if let Some(x) = sched.catalog.courses.get(code) {
                        // Convert CourseCodeSuffix to PyObject based on its variant
                        let py_suffix: PyObject = match &code.code {
                            CourseCodeSuffix::Number(num) => (*num).into_pyobject(py)?.into(),
                            CourseCodeSuffix::Special(text) => {
                                text.clone().into_pyobject(py)?.into()
                            }
                            CourseCodeSuffix::Unique(num) => (*num).into_pyobject(py)?.into(),
                        };
                        sem_courses.push((code.stem.to_string(), py_suffix, x.1));
                    } else {
                        return Err(PyRuntimeError::new_err(format!(
                            "Course code {code} not found in catalog"
                        )));
                    }
                }
                courses.insert(
                    if i > 0 {
                        format!("semester-{i}")
                    } else {
                        "incoming".into()
                    },
                    sem_courses,
                );
            }
            Ok(courses)
        });
        courses
    }

    #[staticmethod]
    fn get_programs() -> PyResult<Vec<String>> {
        Ok(WE!(CATALOGS.first().ok_or(anyhow!("no catalogs found")))
            .programs
            .iter()
            .map(|x| x.name.clone())
            .collect())
    }

    #[staticmethod]
    fn gen_valid_options(
        programs: Vec<String>,
        incoming: Vec<String>,
        options: u64,
    ) -> PyResult<Vec<Self>> {
        let sched = WE!(generate_schedule(
            programs.iter().map(|x| x.as_str()).collect(),
            WE!(CATALOGS.first().ok_or(anyhow!("no catalogs found"))).clone(),
            Some(
                incoming
                    .into_iter()
                    .map(|x| {
                        let parts = x.split("-").collect::<Vec<_>>();
                        CourseCode {
                            stem: parts[0].into(),
                            code: if let Some(y) = parts[1].parse::<u32>().ok() {
                                CourseCodeSuffix::Number(y.try_into().unwrap())
                            } else {
                                CourseCodeSuffix::Special(parts[1].into())
                            },
                        }
                    })
                    .collect()
            )
        ));
        Ok(WE!(generate_multi_schedules(
            sched,
            MAX_CREDITS_PER_SEMESTER,
            options
        ))
        .into_iter()
        .map(Schedule)
        .collect())
    }
}

#[pymodule]
fn ross_link(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Schedule>()?;
    Ok(())
}
