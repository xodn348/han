#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyAnyMethods;

#[cfg(feature = "python")]
use crate::interpreter::Value;

#[cfg(feature = "python")]
pub fn run_python(code: &str) -> Result<String, String> {
    Python::with_gil(|py| {
        let result = py.run(pyo3::ffi::c_str!(""), None, None);
        if let Err(e) = result {
            return Err(e.to_string());
        }

        let locals = pyo3::types::PyDict::new(py);
        let wrapped = format!(
            "import io, sys\n_old_stdout = sys.stdout\nsys.stdout = _buf = io.StringIO()\n{}\n_result = _buf.getvalue()\nsys.stdout = _old_stdout",
            code
        );

        match py.run(
            &std::ffi::CString::new(wrapped).unwrap(),
            None,
            Some(&locals),
        ) {
            Ok(_) => match locals.get_item("_result") {
                Ok(Some(val)) => Ok(val.to_string()),
                _ => Ok(String::new()),
            },
            Err(e) => Err(e.to_string()),
        }
    })
}

#[cfg(feature = "python")]
pub fn eval_python(code: &str) -> Result<Value, String> {
    Python::with_gil(|py| {
        let result = py.eval(&std::ffi::CString::new(code).unwrap(), None, None);
        match result {
            Ok(val) => {
                if let Ok(i) = val.extract::<i64>() {
                    Ok(Value::Int(i))
                } else if let Ok(f) = val.extract::<f64>() {
                    Ok(Value::Float(f))
                } else if let Ok(s) = val.extract::<String>() {
                    Ok(Value::Str(s))
                } else if let Ok(b) = val.extract::<bool>() {
                    Ok(Value::Bool(b))
                } else {
                    Ok(Value::Str(val.to_string()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    })
}
