use std::env;

pub fn get_str_envvar(envvar_name: &str, default_value: Option<&str>) -> Result<String, String> {
    if let Ok(value) = env::var(envvar_name) {
        println!("Set environment variable {} to {}", envvar_name, value);
        Ok(value)
    } else if let Some(default) = default_value {
        println!("Set environment variable {} to {}", envvar_name, default);
        Ok(default.to_string())
    } else {
        Err(format!(
            "envvar {} does not exist and no default value was given",
            envvar_name
        ))
    }
}

pub fn get_int_envvar(envvar_name: &str, default_value: Option<i32>) -> Result<i32, String> {
    let str_default_value = default_value.map(|v| v.to_string());
    let str_envvar = get_str_envvar(envvar_name, str_default_value.as_deref())?;

    str_envvar.parse::<i32>().map_err(|_| {
        format!(
            "envvar {} could not be converted to an integer",
            envvar_name
        )
    })
}
