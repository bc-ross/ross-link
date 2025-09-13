use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use anyhow::anyhow;
use pyo3::types::PyBytes;
use ross_core::MAX_CREDITS_PER_SEMESTER;
use ross_core::geneds::GenEd;
use ross_core::read_excel_file::read_vec;
use ross_core::schedule::CourseCodeSuffix;
use ross_core::write_excel_file::export_schedule;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::Path;

use ross_core::CC;
use ross_core::load_catalogs::CATALOGS;
use ross_core::model::generate_multi_schedules;
use ross_core::read_excel_file::read_file;
use ross_core::schedule::CourseCode;
use ross_core::schedule::Schedule as RossSchedule;
use ross_core::schedule::generate_schedule;
use ross_core::write_excel_file::save_schedule;

macro_rules! WE {
    ($x:expr) => {
        $x.map_err(|e| PyRuntimeError::new_err(format!("Rust error: {}", e)))?
    };
}

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum ReasonTypes {
    Core,
    Foundation,
    SkillsAndPerspective,
    ProgramRequired,
    ProgramElective,
    CourseReq,
}

#[pyclass(subclass)]
struct Schedule(RossSchedule);

fn str_to_cc(x: &str) -> CourseCode {
    let parts = x.split("-").collect::<Vec<_>>();
    CourseCode {
        stem: parts[0].into(),
        code: if let Some(y) = parts[1].parse::<u32>().ok() {
            CourseCodeSuffix::Number(y.try_into().unwrap())
        } else {
            CourseCodeSuffix::Special(parts[1].into())
        },
    }
}

#[pymethods]
impl Schedule {
    #[new]
    #[pyo3(signature = (programs, incoming=None))]
    fn new(programs: Vec<String>, incoming: Option<Vec<String>>) -> PyResult<Self> {
        let sched = WE!(generate_schedule(
            programs.iter().map(|x| x.as_str()).collect(),
            WE!(CATALOGS.first().ok_or(anyhow!("no catalogs found"))).clone(),
            incoming.map(|v| v.into_iter().map(|x| str_to_cc(&x)).collect()),
            None
        ));
        Ok(Schedule(sched))
    }

    #[staticmethod]
    #[pyo3(signature = (programs, incoming=None, courses=None))]
    pub fn _with_courses(
        programs: Vec<String>,
        incoming: Option<Vec<String>>,
        courses: Option<Vec<Vec<String>>>,
    ) -> PyResult<Self> {
        let sched = WE!(generate_schedule(
            programs.iter().map(|x| x.as_str()).collect(),
            WE!(CATALOGS.first().ok_or(anyhow!("no catalogs found"))).clone(),
            incoming.map(|v| v.into_iter().map(|x| str_to_cc(&x)).collect()),
            courses.map(|x| x
                .into_iter()
                .map(|v| v.into_iter().map(|x| str_to_cc(&x)).collect())
                .collect()),
        ));
        Ok(Schedule(sched))
    }

    #[pyo3(signature = (reason, *, name=None, prog=None))]
    pub fn get_other_courses(
        &self,
        reason: ReasonTypes,
        name: Option<String>,
        prog: Option<String>,
    ) -> PyResult<Vec<String>> {
        Ok(HashSet::from_iter(
            match reason {
                ReasonTypes::Core => {
                    if let Some(n) = name {
                        self.0
                            .catalog
                            .geneds
                            .iter()
                            .filter_map(|x| {
                                if let GenEd::Core { name, req } = x {
                                    if name == &n {
                                        return Some(req.all_course_codes());
                                    }
                                }
                                None
                            })
                            .flatten()
                            .map(|c| c.to_string())
                            .collect()
                    } else {
                        return Err(PyRuntimeError::new_err(
                            "Name must be provided for Core reason type",
                        ));
                    }
                }
                ReasonTypes::Foundation => {
                    if let Some(n) = name {
                        self.0
                            .catalog
                            .geneds
                            .iter()
                            .filter_map(|x| {
                                if let GenEd::Foundation { name, req } = x {
                                    if name == &n {
                                        return Some(req.all_course_codes());
                                    }
                                }
                                None
                            })
                            .flatten()
                            .map(|c| c.to_string())
                            .collect()
                    } else {
                        return Err(PyRuntimeError::new_err(
                            "Name must be provided for Core reason type",
                        ));
                    }
                }
                ReasonTypes::SkillsAndPerspective => {
                    if let Some(n) = name {
                        self.0
                            .catalog
                            .geneds
                            .iter()
                            .filter_map(|x| {
                                if let GenEd::SkillAndPerspective { name, req } = x {
                                    if name == &n {
                                        return Some(req.all_course_codes());
                                    }
                                }
                                None
                            })
                            .flatten()
                            .map(|c| c.to_string())
                            .collect()
                    } else {
                        return Err(PyRuntimeError::new_err(
                            "Name must be provided for Core reason type",
                        ));
                    }
                }
                ReasonTypes::ProgramRequired => vec![],
                ReasonTypes::ProgramElective => {
                    if let Some(n) = name
                        && let Some(p) = prog
                    {
                        self.0
                            .catalog
                            .programs
                            .iter()
                            .filter(|x| x.name == p)
                            .map(|x| {
                                x.electives
                                    .iter()
                                    .filter_map(|e| {
                                        if e.name == n {
                                            Some(e.req.all_course_codes())
                                        } else {
                                            None
                                        }
                                    })
                                    .flatten()
                                    .map(|c| c.to_string())
                                    .collect::<Vec<_>>()
                            })
                            .flatten()
                            .collect()
                    } else {
                        return Err(PyRuntimeError::new_err(
                            "Name and Program must be provided for ProgramElective reason type",
                        ));
                    }
                }
                ReasonTypes::CourseReq => vec![],
            }
            .into_iter(),
        )
        .difference(
            &self
                .0
                .incoming
                .iter()
                .chain(self.0.courses.iter().flatten())
                .map(|c| c.to_string())
                .collect::<HashSet<_>>(),
        )
        .into_iter()
        .map(|x| x.to_string())
        .collect())
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
            ),
            None,
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

    pub fn get_reasons(&self) -> PyResult<HashMap<String, Vec<HashMap<String, String>>>> {
        Ok(WE!(self.0.get_reasons())
            .into_iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    v.into_iter()
                        .map(|r| {
                            let mut map = HashMap::new();
                            match r {
                                ross_core::transparency::CourseReasons::Core { name } => {
                                    map.insert("type".into(), "Core".into());
                                    map.insert("name".into(), name);
                                }
                                ross_core::transparency::CourseReasons::Foundation { name } => {
                                    map.insert("type".into(), "Foundation".into());
                                    map.insert("name".into(), name);
                                }
                                ross_core::transparency::CourseReasons::SkillsAndPerspective {
                                    name,
                                } => {
                                    map.insert("type".into(), "SkillsAndPerspective".into());
                                    map.insert("name".into(), name);
                                }
                                ross_core::transparency::CourseReasons::ProgramRequired {
                                    prog,
                                } => {
                                    map.insert("type".into(), "ProgramRequired".into());
                                    map.insert("program".into(), prog);
                                }
                                ross_core::transparency::CourseReasons::ProgramElective {
                                    prog,
                                    name,
                                } => {
                                    map.insert("type".into(), "ProgramElective".into());
                                    map.insert("program".into(), prog);
                                    map.insert("name".into(), name);
                                }
                                ross_core::transparency::CourseReasons::CourseReq { course } => {
                                    map.insert("type".into(), "CourseReq".into());
                                    map.insert("course".into(), course.to_string());
                                }
                            }
                            map
                        })
                        .collect(),
                )
            })
            .collect())
    }

    pub fn to_excel_bytes(&self) -> PyResult<PyObject> {
        let bytes = WE!(export_schedule(&self.0));
        Python::with_gil(|py| Ok(PyBytes::new(py, &bytes).into()))
    }

    #[staticmethod]
    fn from_excel_bytes(buf: &[u8]) -> PyResult<Self> {
        let sched = WE!(read_vec(buf));
        Ok(Self(sched))
    }

    #[pyo3(signature = (semesters, incoming=None))]
    pub fn swap_courses(
        &mut self,
        semesters: Vec<Vec<String>>,
        incoming: Option<Vec<String>>,
    ) -> PyResult<()> {
        self.0.courses = semesters
            .into_iter()
            .map(|sem| sem.into_iter().map(|x| str_to_cc(&x)).collect())
            .collect();
        if let Some(inc) = incoming {
            self.0.incoming = inc.into_iter().map(|x| str_to_cc(&x)).collect();
        }
        Ok(())
    }
}

#[pymodule]
fn ross_link(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Schedule>()?;
    m.add_class::<ReasonTypes>()?;
    Ok(())
}
