use super::remote::{self, resolver};

#[derive(Debug, Default)]
pub struct Verification {
    pub id: Option<String>,
    pub name: String,
    pub retries: i32,
}

#[derive(Debug, Default)]
pub struct Input {
    pub id: Option<String>,
    pub name: String,
}

impl Verification {
    pub fn new(inputs: &Vec<Input>, sources: &Option<Vec<i64>>) -> Self {
        let mut retries = 0;
        loop {
            match remote::verify(inputs, sources) {
                Ok(resolved) => return Verification::build(retries, Some(resolved), None),
                Err(err) => {
                    if retries < 3 {
                        retries += 1;
                    } else {
                        return Verification::build(retries, None, Some(err));
                    }
                }
            };
        }
    }

    fn build(
        _retries: i32,
        resolve: Option<resolver::ResponseData>,
        _err: Option<anyhow::Error>,
    ) -> Self {
        if let Some(res) = resolve {
            println!("{:#?}", res);
        }
        Verification {
            ..Default::default()
        }
    }
}
