//! Small text and number helpers shared by the domain modules.

/// Round to `digits` decimals, correctly rounded on the exact binary value.
///
/// The float formatter rounds the stored value exactly, whereas `(x * 10^n).round() / 10^n`
/// can be off by one ulp (`2.675` is stored as `2.67499999...` and must round to `2.67`).
pub fn round_to_digits(value: f64, digits: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }
    format!("{value:.digits$}").parse().unwrap_or(value)
}

/// Uppercase the first letter of each alphabetic run and lowercase the rest:
/// `"kay/o"` becomes `"Kay/O"`, `"the_range"` becomes `"The_Range"`.
pub fn title_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut previous_is_alpha = false;
    for character in text.chars() {
        if character.is_alphabetic() {
            if previous_is_alpha {
                result.extend(character.to_lowercase());
            } else {
                result.extend(character.to_uppercase());
            }
            previous_is_alpha = true;
        } else {
            result.push(character);
            previous_is_alpha = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_to_digits_rounds_the_stored_value() {
        assert_eq!(round_to_digits(2.675, 2), 2.67);
        assert_eq!(round_to_digits(3.300_000_000_000_000_3, 2), 3.3);
        assert!(round_to_digits(f64::NAN, 2).is_nan());
    }

    #[test]
    fn title_case_capitalises_each_word() {
        assert_eq!(title_case("kay/o"), "Kay/O");
        assert_eq!(title_case("the_range"), "The_Range");
        assert_eq!(title_case("SPIKE RUSH"), "Spike Rush");
    }
}
