use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct ContentDisposition {
    variables: HashMap<String, String>,
    is_file_field: bool,
    has_name_field: bool,
}

pub struct ContentDispositionParseResult {
    pub variables: HashMap<String, String>,
    pub is_file_field: bool,
    pub has_name_field: bool,
}

#[allow(dead_code)]
impl ContentDisposition {
    /// Constructs a `ContentDisposition` from a `content_disposition` string.
    pub fn create(content_disposition: &str) -> Self {
        let result = Self::parse(content_disposition);
        Self {
            variables: result.variables,
            is_file_field: result.is_file_field,
            has_name_field: result.has_name_field,
        }
    }

    /// Retrieves a reference to the value associated with the given key, if it exists.
    pub fn get_variable(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|v| v.as_str())
    }

    /// Returns a reference to the entire map of variables.
    pub fn get_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Checks if the content disposition represents a file field.
    pub fn is_file_field(&self) -> bool {
        self.is_file_field
    }

    /// Checks if the content disposition contains a "name" field.
    pub fn has_name_field(&self) -> bool {
        self.has_name_field
    }

    pub fn get_name(&self) -> Option<&str> {
        self.get_variable("name")
    }

    pub fn get_filename(&self) -> Option<&str> {
        self.get_variable("filename")
    }

    /// Parses a content disposition string into a HashMap of variables.
    pub fn parse(content_disposition: &str) -> ContentDispositionParseResult {
        let mut variables = HashMap::new();

        for part in content_disposition.split(';') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                // Trim whitespace and remove any surrounding quotes from the value
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                variables.insert(key, value);
            }
        }

        ContentDispositionParseResult {
            is_file_field: variables.contains_key("filename"),
            has_name_field: variables.contains_key("name"),
            variables,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test for creating a ContentDisposition instance from a valid content disposition string
    #[test]
    fn test_create_valid_disposition() {
        let content_disposition = "form-data; name=\"file\"; filename=\"example.txt\"";
        let content = ContentDisposition::create(content_disposition);

        // Check that the variables map is correctly populated
        assert_eq!(content.get_variable("name"), Some("file"));
        assert_eq!(content.get_variable("filename"), Some("example.txt"));

        // Check file and name field presence
        assert!(content.is_file_field());
        assert!(content.has_name_field());
    }

    // Test for creating a ContentDisposition instance with no filename (not a file field)
    #[test]
    fn test_create_no_filename() {
        let content_disposition = "form-data; name=\"file\"";
        let content = ContentDisposition::create(content_disposition);

        // Check that the variables map is correctly populated
        assert_eq!(content.get_variable("name"), Some("file"));
        assert_eq!(content.get_variable("filename"), None);

        // Check file and name field presence
        assert!(!content.is_file_field());
        assert!(content.has_name_field());
    }

    // Test for creating a ContentDisposition instance with an empty string
    #[test]
    fn test_create_empty_disposition() {
        let content_disposition = "";
        let content = ContentDisposition::create(content_disposition);

        // Check that no variables are set
        assert!(content.get_variables().is_empty());
        assert!(!content.is_file_field());
        assert!(!content.has_name_field());
    }

    // Test for parsing a content disposition string with multiple parameters
    #[test]
    fn test_parse_multiple_parameters() {
        let content_disposition =
            "form-data; name=\"file\"; filename=\"example.txt\"; another_param=\"value\"";
        let result = ContentDisposition::parse(content_disposition);

        // Check variables map is correctly populated
        assert_eq!(result.variables.get("name"), Some(&"file".to_string()));
        assert_eq!(
            result.variables.get("filename"),
            Some(&"example.txt".to_string())
        );
        assert_eq!(
            result.variables.get("another_param"),
            Some(&"value".to_string())
        );

        // Check file and name field presence
        assert!(result.is_file_field);
        assert!(result.has_name_field);
    }

    // Test for parsing content disposition with only "name"
    #[test]
    fn test_parse_name_only() {
        let content_disposition = "form-data; name=\"file\"";
        let result = ContentDisposition::parse(content_disposition);

        // Check variables map is correctly populated
        assert_eq!(result.variables.get("name"), Some(&"file".to_string()));

        // Check file and name field presence
        assert!(!result.is_file_field);
        assert!(result.has_name_field);
    }

    // Test for parsing content disposition with only "filename"
    #[test]
    fn test_parse_filename_only() {
        let content_disposition = "form-data; filename=\"example.txt\"";
        let result = ContentDisposition::parse(content_disposition);

        // Check variables map is correctly populated
        assert_eq!(
            result.variables.get("filename"),
            Some(&"example.txt".to_string())
        );

        // Check file and name field presence
        assert!(result.is_file_field);
        assert!(!result.has_name_field);
    }

    // Test for parsing content disposition with no fields
    #[test]
    fn test_parse_no_fields() {
        let content_disposition = "form-data";
        let result = ContentDisposition::parse(content_disposition);

        // Check that the variables map is empty
        assert!(result.variables.is_empty());

        // Check file and name field presence
        assert!(!result.is_file_field);
        assert!(!result.has_name_field);
    }

    // Test for parsing content disposition with quoted values
    #[test]
    fn test_parse_quoted_values() {
        let content_disposition = r#"form-data; name="some name"; filename="test.txt""#;
        let result = ContentDisposition::parse(content_disposition);

        // Check that variables are parsed correctly, without quotes
        assert_eq!(result.variables.get("name"), Some(&"some name".to_string()));
        assert_eq!(
            result.variables.get("filename"),
            Some(&"test.txt".to_string())
        );

        // Check file and name field presence
        assert!(result.is_file_field);
        assert!(result.has_name_field);
    }

    // Test for retrieving variables from ContentDisposition
    #[test]
    fn test_get_variable() {
        let content_disposition = "form-data; name=\"file\"; filename=\"example.txt\"";
        let content = ContentDisposition::create(content_disposition);

        // Test getting variables
        assert_eq!(content.get_variable("name"), Some("file"));
        assert_eq!(content.get_variable("filename"), Some("example.txt"));
        assert_eq!(content.get_variable("nonexistent"), None);
    }

    // Test for getting all variables
    #[test]
    fn test_get_variables() {
        let content_disposition = "form-data; name=\"file\"; filename=\"example.txt\"";
        let content = ContentDisposition::create(content_disposition);

        // Check the whole map of variables
        let variables = content.get_variables();
        assert_eq!(variables.get("name"), Some(&"file".to_string()));
        assert_eq!(variables.get("filename"), Some(&"example.txt".to_string()));
    }

    // Test for content disposition with extra spaces
    #[test]
    fn test_parse_with_spaces() {
        let content_disposition = " form-data ;  name = \"file\" ;  filename  =  \"example.txt\"  ";
        let result = ContentDisposition::parse(content_disposition);

        // Check that values are parsed correctly with extra spaces
        assert_eq!(result.variables.get("name"), Some(&"file".to_string()));
        assert_eq!(
            result.variables.get("filename"),
            Some(&"example.txt".to_string())
        );
    }
}
