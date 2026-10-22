//! Tolerant accessors for Riot JSON. Riot mixes camelCase and PascalCase keys and ints, floats
//! and strings for the same field, so every reader here returns a default instead of failing:
//! a shape change must cost one field, never the whole match.

use serde_json::Value;

/// Stand-in for a missing object; `Value::get` on it always answers `None`.
pub(crate) static MISSING: Value = Value::Null;

/// Loose truthiness: `null`, `false`, `0`, `""`, `[]` and `{}` are falsy.
pub(crate) fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|float| float != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
    }
}

/// First key that is present, even when its value is `null`.
pub(crate) fn first_present<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| value.get(*key))
}

/// First key whose value is truthy.
pub(crate) fn first_truthy<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter()
        .filter_map(|key| value.get(*key))
        .find(|found| is_truthy(found))
}

/// First key holding a JSON object, or `MISSING` so callers can keep chaining `get`.
pub(crate) fn object<'a>(value: &'a Value, keys: &[&str]) -> &'a Value {
    keys.iter()
        .filter_map(|key| value.get(*key))
        .find(|found| found.is_object())
        .unwrap_or(&MISSING)
}

/// First non-empty array under any of `keys`, else an empty slice.
pub(crate) fn items<'a>(value: &'a Value, keys: &[&str]) -> &'a [Value] {
    keys.iter()
        .filter_map(|key| value.get(*key).and_then(Value::as_array))
        .find(|array| !array.is_empty())
        .map_or(&[], Vec::as_slice)
}

/// First non-empty string under any of `keys`, borrowed.
pub(crate) fn optional_str<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .filter_map(|key| value.get(*key).and_then(Value::as_str))
        .find(|found| !found.is_empty())
}

/// First non-empty string under any of `keys`, else `""`, borrowed.
pub(crate) fn text_ref<'a>(value: &'a Value, keys: &[&str]) -> &'a str {
    optional_str(value, keys).unwrap_or_default()
}

/// First non-empty string under any of `keys`.
pub(crate) fn optional_text(value: &Value, keys: &[&str]) -> Option<String> {
    optional_str(value, keys).map(str::to_owned)
}

/// First non-empty string under any of `keys`, else `""`.
pub(crate) fn text(value: &Value, keys: &[&str]) -> String {
    optional_text(value, keys).unwrap_or_default()
}

/// A JSON number (integer or float) as a float; anything else is `None`.
pub(crate) fn number(value: Option<&Value>) -> Option<f64> {
    value
        .filter(|found| found.is_number())
        .and_then(Value::as_f64)
}

/// A JSON number, or 0.
pub(crate) fn number_or_zero(value: Option<&Value>) -> f64 {
    number(value).unwrap_or(0.0)
}

fn exact_or_truncated(value: &Value) -> Option<i64> {
    if let Some(exact) = value.as_i64() {
        return Some(exact);
    }
    // Saturating `as` is the intended behaviour for absurd magnitudes.
    value.as_f64().map(|float| float as i64)
}

/// `int(_number(x))`: numbers truncate toward zero, everything else is 0.
pub(crate) fn whole(value: Option<&Value>) -> i64 {
    value
        .filter(|found| found.is_number())
        .and_then(exact_or_truncated)
        .unwrap_or(0)
}

/// A JSON integer, not a float or a string.
pub(crate) fn exact_int(value: Option<&Value>) -> Option<i64> {
    value.and_then(Value::as_i64)
}

/// An integer from a number or a numeric string; `None` for anything else.
pub(crate) fn lenient_int(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::String(text) => text.trim().parse().ok(),
        Value::Bool(flag) => Some(i64::from(*flag)),
        other if other.is_number() => exact_or_truncated(other),
        _ => None,
    }
}

/// A float from a number or a numeric string; 0 otherwise.
pub(crate) fn lenient_float(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::String(text)) => text.trim().parse().unwrap_or(0.0),
        other => number_or_zero(other),
    }
}

/// The `matchInfo` object of a match-details response.
pub(crate) fn match_info(detail: &Value) -> &Value {
    object(detail, &["matchInfo", "MatchInfo"])
}

/// The match id, from `matchInfo` or the top level.
pub(crate) fn match_id(detail: &Value) -> String {
    let from_info = text(match_info(detail), &["matchId", "MatchID", "MatchId"]);
    if from_info.is_empty() {
        text(detail, &["matchId", "MatchID"])
    } else {
        from_info
    }
}

/// A player's puuid under either casing.
pub(crate) fn subject(entry: &Value) -> String {
    text(entry, &["subject", "Subject"])
}

/// A player's team id under either casing.
pub(crate) fn team_id(entry: &Value) -> String {
    text(entry, &["teamId", "TeamID", "TeamId"])
}

/// Roster objects of a match-details response. Observers are dropped: they are not players
/// and carry no stats.
pub(crate) fn roster(detail: &Value) -> Vec<&Value> {
    items(detail, &["players", "Players"])
        .iter()
        .filter(|entry| entry.is_object())
        .filter(|entry| !entry.get("isObserver").is_some_and(is_truthy))
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn truthiness_treats_empty_values_as_false() {
        assert!(!is_truthy(&json!(0)));
        assert!(!is_truthy(&json!("")));
        assert!(!is_truthy(&json!({})));
        assert!(is_truthy(&json!(0.5)));
        assert!(is_truthy(&json!("x")));
    }

    #[test]
    fn text_skips_empty_values_and_falls_through_to_pascal_case() {
        let value = json!({"subject": "", "Subject": "abc"});
        assert_eq!(subject(&value), "abc");
    }

    #[test]
    fn whole_truncates_floats_and_ignores_strings() {
        assert_eq!(whole(Some(&json!(3.9))), 3);
        assert_eq!(whole(Some(&json!("7"))), 0);
        assert_eq!(whole(None), 0);
    }

    #[test]
    fn lenient_int_accepts_numeric_strings_only_when_integral() {
        assert_eq!(lenient_int(Some(&json!(" 12 "))), Some(12));
        assert_eq!(lenient_int(Some(&json!("12.5"))), None);
        assert_eq!(lenient_int(Some(&json!(null))), None);
    }

    #[test]
    fn roster_drops_observers_and_non_objects() {
        let detail =
            json!({"Players": [{"subject": "a"}, {"subject": "b", "isObserver": true}, 7]});
        assert_eq!(roster(&detail).len(), 1);
    }

    #[test]
    fn match_id_prefers_match_info_then_top_level() {
        assert_eq!(match_id(&json!({"matchInfo": {"matchId": "m1"}})), "m1");
        assert_eq!(match_id(&json!({"matchId": "m2"})), "m2");
        assert_eq!(match_id(&json!(null)), "");
    }
}
