#![allow(dead_code)]
#![allow(non_snake_case)]

#[cfg(test)]
mod test {
    #[at(client)]
    #[styled_component]
    pub fn FetchButton() -> Html {
        let result = use_state(|| "Click me".to_string());

        html! {
            <button onclick={{
                let result = result.clone();
                move |_| {
                    if let Ok(response) = on!(server, (*result).clone(), async |result, global_memory| {
                        global_memory.get("count")? += 1;
                        Ok(format!("{} | {}", result, global_memory.get("count")?))
                    }) {
                        *result = response;
                    }
                }
            }}>
                {*result}
            </button>
        }
    }
}
