use esf_dogma_engine::rust;

pub fn dump(output: &rust::Output) -> String {
    let mut statistics = Vec::new();
    flatten(&serde_json::to_value(output).unwrap(), "", &mut statistics);

    let width = statistics.iter().map(|(key, _)| key.len()).max().unwrap();
    statistics
        .iter()
        .map(|(key, value)| format!("{key:width$} = {value}\n"))
        .collect()
}

fn flatten(value: &serde_json::Value, path: &str, statistics: &mut Vec<(String, String)>) {
    let value = match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                let path = match path {
                    "" => key.to_string(),
                    path => format!("{path}/{key}"),
                };
                flatten(value, &path, statistics);
            }
            return;
        }
        serde_json::Value::Number(number) => format!("{:.6}", number.as_f64().unwrap() + 0.0),
        serde_json::Value::Null => "non-finite".to_string(),
        serde_json::Value::String(text) => text.clone(),
        value => value.to_string(),
    };

    statistics.push((path.to_string(), value));
}
