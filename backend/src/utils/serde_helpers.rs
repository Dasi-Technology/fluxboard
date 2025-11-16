use serde::Deserialize;

/// Custom deserializer to handle explicit null values for optional fields
///
/// This allows us to distinguish between three states:
/// - Field not present in JSON: `None` (don't update the field)
/// - Field present with `null`: `Some(None)` (set field to NULL in database)
/// - Field present with value: `Some(Some(value))` (set field to the value)
///
/// # Usage
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UpdateInput {
///     #[serde(default, deserialize_with = "deserialize_null_default")]
///     pub description: Option<Option<String>>,
/// }
/// ```
pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}
