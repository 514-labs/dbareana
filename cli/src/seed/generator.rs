use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::{FirstName, LastName, Name};
use fake::faker::phone_number::en::PhoneNumber;
use fake::Fake;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::seed::foreign_key::ForeignKeyResolver;

/// Data type for generated values
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    Decimal,
    String,
    Boolean,
    Timestamp,
}

/// Core trait for data generation
pub trait DataGenerator: Send + Sync {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String>;
    fn data_type(&self) -> DataType;
}

/// Sequential integer generator (thread-safe)
pub struct SequentialGenerator {
    counter: Arc<AtomicUsize>,
    start: usize,
}

impl SequentialGenerator {
    pub fn new(start: usize) -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(start)),
            start,
        }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let start = options
            .get("start")
            .and_then(|v| v.as_integer())
            .unwrap_or(1) as usize;
        Ok(Self::new(start))
    }
}

impl DataGenerator for SequentialGenerator {
    fn generate(&self, _rng: &mut ChaCha8Rng) -> Result<String> {
        let val = self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(val.to_string())
    }

    fn data_type(&self) -> DataType {
        DataType::Integer
    }
}

/// Random integer generator
pub struct RandomIntGenerator {
    min: i64,
    max: i64,
}

impl RandomIntGenerator {
    pub fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let min = options
            .get("min")
            .and_then(|v| v.as_integer())
            .unwrap_or(0);
        let max = options
            .get("max")
            .and_then(|v| v.as_integer())
            .unwrap_or(100);
        Ok(Self::new(min, max))
    }
}

impl DataGenerator for RandomIntGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        let val = rng.gen_range(self.min..=self.max);
        Ok(val.to_string())
    }

    fn data_type(&self) -> DataType {
        DataType::Integer
    }
}

/// Random decimal generator
pub struct RandomDecimalGenerator {
    min: f64,
    max: f64,
    precision: usize,
}

impl RandomDecimalGenerator {
    pub fn new(min: f64, max: f64, precision: usize) -> Self {
        Self { min, max, precision }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let min = options
            .get("min")
            .and_then(|v| v.as_float())
            .unwrap_or(0.0);
        let max = options
            .get("max")
            .and_then(|v| v.as_float())
            .unwrap_or(100.0);
        let precision = options
            .get("precision")
            .and_then(|v| v.as_integer())
            .unwrap_or(2) as usize;
        Ok(Self::new(min, max, precision))
    }
}

impl DataGenerator for RandomDecimalGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        let val = rng.gen_range(self.min..=self.max);
        Ok(format!("{:.prec$}", val, prec = self.precision))
    }

    fn data_type(&self) -> DataType {
        DataType::Decimal
    }
}

/// Boolean generator
pub struct BooleanGenerator {
    true_probability: f64,
}

impl BooleanGenerator {
    pub fn new(true_probability: f64) -> Self {
        Self { true_probability }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let prob = options
            .get("true_probability")
            .and_then(|v| v.as_float())
            .unwrap_or(0.5);
        Ok(Self::new(prob))
    }
}

impl DataGenerator for BooleanGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        let val = rng.gen::<f64>() < self.true_probability;
        Ok(val.to_string())
    }

    fn data_type(&self) -> DataType {
        DataType::Boolean
    }
}

/// Timestamp generator
pub enum TimestampType {
    Now,
    Range { start: DateTime<Utc>, end: DateTime<Utc> },
    Relative { offset_seconds: i64 },
}

pub struct TimestampGenerator {
    timestamp_type: TimestampType,
}

impl TimestampGenerator {
    pub fn new(timestamp_type: TimestampType) -> Self {
        Self { timestamp_type }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let type_str = options
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("now");

        let timestamp_type = match type_str {
            "now" => TimestampType::Now,
            "range" => {
                let start_str = options
                    .get("start")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("start date required for range timestamp"))?;
                let end_str = options
                    .get("end")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("end date required for range timestamp"))?;

                let start = parse_date(start_str)?;
                let end = parse_date(end_str)?;

                TimestampType::Range { start, end }
            }
            "relative" => {
                let offset = options
                    .get("offset_seconds")
                    .and_then(|v| v.as_integer())
                    .ok_or_else(|| anyhow!("offset_seconds required for relative timestamp"))?;
                TimestampType::Relative {
                    offset_seconds: offset,
                }
            }
            _ => return Err(anyhow!("Unknown timestamp type: {}", type_str)),
        };

        Ok(Self::new(timestamp_type))
    }
}

impl DataGenerator for TimestampGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        let timestamp = match &self.timestamp_type {
            TimestampType::Now => Utc::now(),
            TimestampType::Range { start, end } => {
                let duration = end.signed_duration_since(*start);
                let random_seconds = rng.gen_range(0..duration.num_seconds());
                *start + Duration::seconds(random_seconds)
            }
            TimestampType::Relative { offset_seconds } => {
                Utc::now() + Duration::seconds(*offset_seconds)
            }
        };

        Ok(timestamp.to_rfc3339())
    }

    fn data_type(&self) -> DataType {
        DataType::Timestamp
    }
}

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    // Try parsing as full datetime first
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try parsing as date only (YYYY-MM-DD)
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(DateTime::from_naive_utc_and_offset(
            date.and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        ));
    }

    Err(anyhow!("Invalid date format: {}", date_str))
}

/// Email generator using fake crate
pub struct EmailGenerator;

impl EmailGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn from_options(_options: &HashMap<String, toml::Value>) -> Result<Self> {
        Ok(Self::new())
    }
}

impl DataGenerator for EmailGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        Ok(SafeEmail().fake_with_rng(rng))
    }

    fn data_type(&self) -> DataType {
        DataType::String
    }
}

/// Phone number generator
pub struct PhoneGenerator;

impl PhoneGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn from_options(_options: &HashMap<String, toml::Value>) -> Result<Self> {
        Ok(Self::new())
    }
}

impl DataGenerator for PhoneGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        Ok(PhoneNumber().fake_with_rng(rng))
    }

    fn data_type(&self) -> DataType {
        DataType::String
    }
}

/// Name generator
pub enum NameType {
    First,
    Last,
    Full,
}

pub struct NameGenerator {
    name_type: NameType,
}

impl NameGenerator {
    pub fn new(name_type: NameType) -> Self {
        Self { name_type }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let type_str = options
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("full");

        let name_type = match type_str {
            "first" => NameType::First,
            "last" => NameType::Last,
            "full" => NameType::Full,
            _ => return Err(anyhow!("Unknown name type: {}", type_str)),
        };

        Ok(Self::new(name_type))
    }
}

impl DataGenerator for NameGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        let name = match self.name_type {
            NameType::First => FirstName().fake_with_rng(rng),
            NameType::Last => LastName().fake_with_rng(rng),
            NameType::Full => Name().fake_with_rng(rng),
        };
        Ok(name)
    }

    fn data_type(&self) -> DataType {
        DataType::String
    }
}

/// Address generator
pub struct AddressGenerator;

impl AddressGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn from_options(_options: &HashMap<String, toml::Value>) -> Result<Self> {
        Ok(Self::new())
    }
}

impl DataGenerator for AddressGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        use fake::faker::address::en::{CityName, StateName, StreetName, ZipCode};

        let street: String = StreetName().fake_with_rng(rng);
        let city: String = CityName().fake_with_rng(rng);
        let state: String = StateName().fake_with_rng(rng);
        let zip: String = ZipCode().fake_with_rng(rng);

        Ok(format!("{}, {}, {} {}", street, city, state, zip))
    }

    fn data_type(&self) -> DataType {
        DataType::String
    }
}

/// Template generator for custom patterns
pub struct TemplateGenerator {
    template: String,
}

impl TemplateGenerator {
    pub fn new(template: String) -> Self {
        Self { template }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let template = options
            .get("template")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("template string required"))?
            .to_string();
        Ok(Self::new(template))
    }
}

impl DataGenerator for TemplateGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        let mut result = self.template.clone();

        // Replace {random_int:min:max} patterns
        while let Some(start) = result.find("{random_int:") {
            if let Some(end) = result[start..].find('}') {
                let pattern = &result[start + 12..start + end];
                let parts: Vec<&str> = pattern.split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(min), Ok(max)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>())
                    {
                        let val = rng.gen_range(min..=max);
                        result.replace_range(start..start + end + 1, &val.to_string());
                        continue;
                    }
                }
                result.replace_range(start..start + end + 1, "0");
            }
        }

        // Replace {timestamp} pattern
        result = result.replace("{timestamp}", &Utc::now().to_rfc3339());

        Ok(result)
    }

    fn data_type(&self) -> DataType {
        DataType::String
    }
}

/// Enum generator for selecting from predefined values
pub struct EnumGenerator {
    values: Vec<String>,
}

impl EnumGenerator {
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }

    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let values = options
            .get("values")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("values array required for enum generator"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        if values.is_empty() {
            return Err(anyhow!("enum values cannot be empty"));
        }

        Ok(Self::new(values))
    }
}

impl DataGenerator for EnumGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> Result<String> {
        let idx = rng.gen_range(0..self.values.len());
        Ok(self.values[idx].clone())
    }

    fn data_type(&self) -> DataType {
        DataType::String
    }
}

/// Foreign key generator using ForeignKeyResolver
pub struct ForeignKeyGenerator {
    resolver: Arc<ForeignKeyResolver>,
    references: (String, String), // (table, column)
}

impl ForeignKeyGenerator {
    pub fn new(resolver: Arc<ForeignKeyResolver>, references: (String, String)) -> Self {
        Self {
            resolver,
            references,
        }
    }

    pub fn from_options(
        options: &HashMap<String, toml::Value>,
        resolver: Arc<ForeignKeyResolver>,
    ) -> Result<Self> {
        let references = options
            .get("references")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow!("references table required for foreign_key generator"))?;

        let table = references
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("table required in references"))?
            .to_string();

        let column = references
            .get("column")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("column required in references"))?
            .to_string();

        Ok(Self::new(resolver, (table, column)))
    }
}

impl DataGenerator for ForeignKeyGenerator {
    fn generate(&self, _rng: &mut ChaCha8Rng) -> Result<String> {
        // We need to block on async in sync context
        // This is a limitation we'll handle by making the engine responsible
        // for calling random_id directly when needed
        Err(anyhow!(
            "ForeignKeyGenerator requires async context - use ForeignKeyResolver directly"
        ))
    }

    fn data_type(&self) -> DataType {
        DataType::String
    }
}

/// Factory function to create generators from config
pub fn create_generator(
    generator_type: &str,
    options: &HashMap<String, toml::Value>,
) -> Result<Box<dyn DataGenerator>> {
    match generator_type {
        "sequential" => Ok(Box::new(SequentialGenerator::from_options(options)?)),
        "random_int" => Ok(Box::new(RandomIntGenerator::from_options(options)?)),
        "random_decimal" => Ok(Box::new(RandomDecimalGenerator::from_options(options)?)),
        "boolean" => Ok(Box::new(BooleanGenerator::from_options(options)?)),
        "timestamp" => Ok(Box::new(TimestampGenerator::from_options(options)?)),
        "email" => Ok(Box::new(EmailGenerator::from_options(options)?)),
        "phone" => Ok(Box::new(PhoneGenerator::from_options(options)?)),
        "name" => Ok(Box::new(NameGenerator::from_options(options)?)),
        "address" => Ok(Box::new(AddressGenerator::from_options(options)?)),
        "template" => Ok(Box::new(TemplateGenerator::from_options(options)?)),
        "enum" => Ok(Box::new(EnumGenerator::from_options(options)?)),
        // Note: foreign_key is handled separately in the engine
        // because it requires async context
        _ => Err(anyhow!("Unknown generator type: {}", generator_type)),
    }
}

/// Information about a foreign key reference
#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    pub table: String,
    pub column: String,
}

impl ForeignKeyInfo {
    pub fn from_options(options: &HashMap<String, toml::Value>) -> Result<Self> {
        let references = options
            .get("references")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow!("references table required for foreign_key generator"))?;

        let table = references
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("table required in references"))?
            .to_string();

        let column = references
            .get("column")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("column required in references"))?
            .to_string();

        Ok(Self { table, column })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_sequential_generator() {
        let gen = SequentialGenerator::new(1);
        let mut rng = test_rng();

        assert_eq!(gen.generate(&mut rng).unwrap(), "1");
        assert_eq!(gen.generate(&mut rng).unwrap(), "2");
        assert_eq!(gen.generate(&mut rng).unwrap(), "3");
        assert_eq!(gen.data_type(), DataType::Integer);
    }

    #[test]
    fn test_random_int_generator() {
        let gen = RandomIntGenerator::new(1, 10);
        let mut rng = test_rng();

        for _ in 0..100 {
            let val = gen.generate(&mut rng).unwrap();
            let num: i64 = val.parse().unwrap();
            assert!(num >= 1 && num <= 10);
        }
    }

    #[test]
    fn test_random_decimal_generator() {
        let gen = RandomDecimalGenerator::new(0.0, 100.0, 2);
        let mut rng = test_rng();

        let val = gen.generate(&mut rng).unwrap();
        let num: f64 = val.parse().unwrap();
        assert!(num >= 0.0 && num <= 100.0);

        // Check precision
        let parts: Vec<&str> = val.split('.').collect();
        if parts.len() == 2 {
            assert!(parts[1].len() <= 2);
        }
    }

    #[test]
    fn test_boolean_generator() {
        let gen = BooleanGenerator::new(0.5);
        let mut rng = test_rng();

        let mut true_count = 0;
        for _ in 0..100 {
            let val = gen.generate(&mut rng).unwrap();
            assert!(val == "true" || val == "false");
            if val == "true" {
                true_count += 1;
            }
        }

        // With 100 samples and 0.5 probability, we expect roughly 50% true
        assert!(true_count > 30 && true_count < 70);
    }

    #[test]
    fn test_email_generator() {
        let gen = EmailGenerator::new();
        let mut rng = test_rng();

        let email = gen.generate(&mut rng).unwrap();
        assert!(email.contains('@'));
        assert_eq!(gen.data_type(), DataType::String);
    }

    #[test]
    fn test_phone_generator() {
        let gen = PhoneGenerator::new();
        let mut rng = test_rng();

        let phone = gen.generate(&mut rng).unwrap();
        assert!(!phone.is_empty());
    }

    #[test]
    fn test_name_generator() {
        let mut rng = test_rng();

        let first = NameGenerator::new(NameType::First);
        let first_name = first.generate(&mut rng).unwrap();
        assert!(!first_name.is_empty());

        let last = NameGenerator::new(NameType::Last);
        let last_name = last.generate(&mut rng).unwrap();
        assert!(!last_name.is_empty());

        let full = NameGenerator::new(NameType::Full);
        let full_name = full.generate(&mut rng).unwrap();
        assert!(full_name.contains(' '));
    }

    #[test]
    fn test_address_generator() {
        let gen = AddressGenerator::new();
        let mut rng = test_rng();

        let address = gen.generate(&mut rng).unwrap();
        assert!(address.contains(','));
    }

    #[test]
    fn test_template_generator() {
        let gen = TemplateGenerator::new("User {random_int:1:100}".to_string());
        let mut rng = test_rng();

        let result = gen.generate(&mut rng).unwrap();
        assert!(result.starts_with("User "));
        let num_part = result.strip_prefix("User ").unwrap();
        let num: i64 = num_part.parse().unwrap();
        assert!(num >= 1 && num <= 100);
    }

    #[test]
    fn test_enum_generator() {
        let gen = EnumGenerator::new(vec![
            "pending".to_string(),
            "active".to_string(),
            "completed".to_string(),
        ]);
        let mut rng = test_rng();

        for _ in 0..10 {
            let val = gen.generate(&mut rng).unwrap();
            assert!(val == "pending" || val == "active" || val == "completed");
        }
    }

    #[test]
    fn test_deterministic_seeding() {
        let gen = RandomIntGenerator::new(1, 100);

        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        let val1 = gen.generate(&mut rng1).unwrap();
        let val2 = gen.generate(&mut rng2).unwrap();

        assert_eq!(val1, val2, "Same seed should produce identical output");
    }

    #[test]
    fn test_timestamp_generator_now() {
        let gen = TimestampGenerator::new(TimestampType::Now);
        let mut rng = test_rng();

        let timestamp = gen.generate(&mut rng).unwrap();
        assert!(DateTime::parse_from_rfc3339(&timestamp).is_ok());
    }

    #[test]
    fn test_timestamp_generator_range() {
        let start = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);

        let gen = TimestampGenerator::new(TimestampType::Range { start, end });
        let mut rng = test_rng();

        let timestamp = gen.generate(&mut rng).unwrap();
        let parsed = DateTime::parse_from_rfc3339(&timestamp)
            .unwrap()
            .with_timezone(&Utc);
        assert!(parsed >= start && parsed <= end);
    }

    #[test]
    fn test_create_generator_factory() {
        let mut options = HashMap::new();
        options.insert("start".to_string(), toml::Value::Integer(100));

        let gen = create_generator("sequential", &options).unwrap();
        let mut rng = test_rng();

        assert_eq!(gen.generate(&mut rng).unwrap(), "100");
    }
}
