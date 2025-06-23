use pyo3::prelude::*;
use kalico_core::MeshDB;
use std::sync::OnceLock;

static DB: OnceLock<MeshDB> = OnceLock::new();

#[pyfunction]
fn save_mesh(points: Vec<(f32, f32, f32)>, kind: &str) -> PyResult<()> {
    let db = DB.get_or_init(|| MeshDB::new("/var/lib/kalico/mesh.duckdb").unwrap());
    db.add_mesh(&points, kind).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn meshdb(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(save_mesh, m)?)?;
    Ok(())
}
