use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use anyhow::anyhow;
use std::path::Path;

use ross_core::load_catalogs::CATALOGS;
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
    fn new(programs: Vec<String>) -> PyResult<Self> {
        let sched = WE!(generate_schedule(
            programs.iter().map(|x| x.as_str()).collect(),
            WE!(CATALOGS.first().ok_or(anyhow!("no catalogs found"))).clone(),
            Some(vec![CC!("THEO", 1100)]), // None,
        ));
        Ok(Schedule(sched))
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
}

#[pymodule]
fn ross_server(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Schedule>()?;
    Ok(())
}
