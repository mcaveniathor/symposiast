use stucco::prelude::*;
use stucco::{FormState, Validator, to_html};

fn validate(state: FormState) -> Result<String, FormState> {
    let mut validator = Validator::new(state);
    let customer = validator
        .text("customer")
        .required("Enter a customer name")
        .max_chars(80, "Use 80 characters or fewer")
        .get();
    validator.finish(|| customer)
}

fn main() {
    let submitted = FormState::new().with_value("customer", "");
    match validate(submitted) {
        Ok(_) => panic!("an empty customer name must fail validation"),
        Err(state) => {
            let form = Form::post("/customers")
                .child(ErrorSummary::new(&state).field("customer", "customer-name"))
                .child(
                    Field::new("Customer", Input::text("customer").id("customer-name"))
                        .required()
                        .bind(&state),
                )
                .child(Button::new("Save").submit());
            let html = to_html(&form);
            assert!(html.contains("Enter a customer name"));
            assert!(html.contains("aria-invalid=\"true\""));
            assert!(html.contains("customer-name"));
            println!("{html}");
        }
    }
}
