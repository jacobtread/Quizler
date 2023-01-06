use std::str::FromStr;

/// Environment variable for the application port along with its default value
pub const PORT: (&str, u16) = ("QUIZLER_PORT", 80);

/// Retrieve and parse an environment variable from the provided pair
/// returning the default value on failure
///
/// `pair` The environment pair
pub fn from_env<V: FromStr>(pair: (&str, V)) -> V {
    if let Ok(value) = std::env::var(pair.0) {
        if let Ok(parsed) = value.parse() {
            return parsed;
        }
    }
    pair.1
}
