//! Integration tests for Signal.kind normalization with golden fixtures

use connectors::normalization::{
    ALL_SIGNAL_KINDS, normalize_example_payload, normalize_jira_webhook_kind,
    normalize_zoho_cliq_webhook_kind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Fixture structure for normalization tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationFixture {
    /// Provider identifier (e.g., "github", "jira", "slack")
    pub provider: String,
    /// Human-readable case name
    pub name: String,
    /// Raw provider payload input
    pub input: serde_json::Value,
    /// Expected normalized output
    pub expected: ExpectedOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutput {
    /// Expected normalized Signal.kind
    pub kind: String,
}

/// Providers that must document fixture coverage.
const ALL_PROVIDER_SLUGS: &[&str] = &[
    "example",
    "github",
    "gmail",
    "google-calendar",
    "google-drive",
    "jira",
    "zoho-cliq",
    "zoho-mail",
];

/// Providers that have normalization implementations in this harness.
const NORMALIZED_PROVIDERS: &[&str] = &["example", "jira", "zoho-cliq"];

/// Root directory for normalization fixtures
const FIXTURE_ROOT: &str = "tests/fixtures/normalization";

fn provider_has_normalizer(provider: &str) -> bool {
    NORMALIZED_PROVIDERS.contains(&provider)
}

/// Discover all normalization fixture files in the given directory
fn discover_fixture_files(fixture_dir: &Path) -> Result<Vec<DirEntry>, Box<dyn std::error::Error>> {
    let mut fixtures = Vec::new();

    for entry in WalkDir::new(fixture_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(path_str) = entry.path().extension().and_then(|s| s.to_str()) {
                if path_str == "json" {
                    fixtures.push(entry);
                }
            }
        }
    }

    Ok(fixtures)
}

/// Load and parse a normalization fixture from a file
fn load_fixture(file_path: &Path) -> Result<NormalizationFixture, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let fixture: NormalizationFixture = serde_json::from_str(&content)?;

    // Validate required fields
    if fixture.provider.is_empty() {
        return Err(format!(
            "Fixture {}: provider field is required",
            file_path.display()
        )
        .into());
    }
    if fixture.name.is_empty() {
        return Err(format!("Fixture {}: name field is required", file_path.display()).into());
    }
    if fixture.expected.kind.is_empty() {
        return Err(format!(
            "Fixture {}: expected.kind field is required",
            file_path.display()
        )
        .into());
    }

    Ok(fixture)
}

/// Check if a provider has a SKIP.md file
fn provider_has_skip_file(fixture_dir: &Path, provider: &str) -> bool {
    let skip_path = fixture_dir.join(provider).join("SKIP.md");
    skip_path.exists()
}

/// Validate that all covered kinds are in the canonical registry
fn validate_canonical_kinds(
    fixtures: &[NormalizationFixture],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut invalid_kinds = HashSet::new();

    for fixture in fixtures {
        if !ALL_SIGNAL_KINDS
            .iter()
            .any(|k| k.as_str() == fixture.expected.kind)
        {
            invalid_kinds.insert(fixture.expected.kind.clone());
        }
    }

    if !invalid_kinds.is_empty() {
        let mut error_msg = "Found Signal.kind values not in canonical registry:\n".to_string();
        for kind in invalid_kinds.iter() {
            error_msg.push_str(&format!("  - {}\n", kind));
        }
        error_msg.push_str("To add new kinds, update the normalization spec and registry.\n");
        return Err(error_msg.into());
    }

    Ok(())
}

/// Invoke the real normalization logic for a fixture payload.
fn normalize_fixture_kind(
    fixture: &NormalizationFixture,
) -> Result<String, Box<dyn std::error::Error>> {
    let kind = match fixture.provider.as_str() {
        "example" => normalize_example_payload(&fixture.input)
            .map_err(|e| format!("example normalization failed: {}", e))?,
        "jira" => normalize_jira_webhook_kind(&fixture.input)
            .ok_or_else(|| "Jira fixture did not contain a supported webhookEvent".to_string())?,
        "zoho-cliq" => normalize_zoho_cliq_webhook_kind(&fixture.input)
            .map_err(|e| format!("zoho-cliq normalization failed: {}", e))?,
        provider => {
            return Err(format!(
                "No normalization mapping implemented for provider: {}",
                provider
            )
            .into());
        }
    };

    Ok(kind.as_str().to_string())
}

#[test]
fn test_normalization_golden_fixtures() {
    // Test that the fixture directory exists
    let fixture_dir = Path::new(FIXTURE_ROOT);
    if !fixture_dir.exists() {
        panic!(
            "Normalization fixture directory not found: {}",
            FIXTURE_ROOT
        );
    }

    // Discover all fixture files
    let fixture_files =
        discover_fixture_files(fixture_dir).expect("Failed to discover fixture files");

    if fixture_files.is_empty() {
        panic!("No normalization fixtures found in {}", FIXTURE_ROOT);
    }

    let mut loaded_fixtures = Vec::new();
    let mut failed_fixtures = Vec::new();

    // Load all fixtures
    for file_path in fixture_files {
        match load_fixture(file_path.path()) {
            Ok(fixture) => {
                println!(
                    "Loaded fixture: {} -> {}",
                    file_path.path().display(),
                    fixture.expected.kind
                );
                loaded_fixtures.push(fixture);
            }
            Err(e) => {
                eprintln!(
                    "Failed to load fixture {}: {}",
                    file_path.path().display(),
                    e
                );
                failed_fixtures.push(file_path.path().to_path_buf());
            }
        }
    }

    // Fail if any fixtures couldn't be loaded
    if !failed_fixtures.is_empty() {
        panic!("Failed to load {} fixtures", failed_fixtures.len());
    }

    // Validate that all expected kinds are canonical
    validate_canonical_kinds(&loaded_fixtures).expect("Found non-canonical Signal.kind values");

    // Test each fixture
    let mut passed = 0;
    let mut failed = 0;

    for fixture in loaded_fixtures {
        println!("Testing fixture: {} ({})", fixture.name, fixture.provider);

        // Run the normalization mapping
        match normalize_fixture_kind(&fixture) {
            Ok(actual_kind) => {
                if actual_kind == fixture.expected.kind {
                    println!(
                        "✓ {}: {} -> {}",
                        fixture.name, fixture.provider, actual_kind
                    );
                    passed += 1;
                } else {
                    eprintln!(
                        "✗ {}: Expected '{}', got '{}'",
                        fixture.name, fixture.expected.kind, actual_kind
                    );
                    failed += 1;
                }
            }
            Err(e) => {
                eprintln!("✗ {}: Mapping failed: {}", fixture.name, e);
                failed += 1;
            }
        }
    }

    println!(
        "\nNormalization test results: {} passed, {} failed",
        passed, failed
    );

    if failed > 0 {
        panic!("{} normalization tests failed", failed);
    }
}

#[test]
fn test_fixture_coverage_enforcement() {
    let fixture_dir = Path::new(FIXTURE_ROOT);

    if !fixture_dir.exists() {
        // If no fixtures exist yet, this test should be skipped
        // but we want to fail loudly if there are providers without fixtures
        return;
    }

    for provider in ALL_PROVIDER_SLUGS {
        let provider_dir = fixture_dir.join(provider);
        assert!(
            provider_dir.exists(),
            "Required provider directory {} not found in {}",
            provider,
            fixture_dir.display()
        );

        let fixtures = discover_fixture_files(&provider_dir)
            .unwrap_or_else(|_| panic!("Failed to discover fixtures for {}", provider));

        if fixtures.is_empty() {
            if provider_has_normalizer(provider) {
                panic!(
                    "Provider {} is normalized but has no fixtures. Add fixtures under {}.",
                    provider,
                    provider_dir.display()
                );
            }

            if !provider_has_skip_file(fixture_dir, provider) {
                panic!(
                    "Provider {} missing fixtures must include SKIP.md explaining the gap.",
                    provider
                );
            }
        } else if !provider_has_normalizer(provider) {
            panic!(
                "Provider {} has fixtures but no normalization harness implementation. \
                 Either add a SKIP.md or implement normalization before adding fixtures.",
                provider
            );
        }
    }
}

#[test]
fn test_canonical_registry_completeness() {
    // This test ensures our canonical registry is not empty
    // and contains expected core kinds
    assert!(
        !ALL_SIGNAL_KINDS.is_empty(),
        "Canonical kind registry cannot be empty"
    );

    // Verify some core kinds exist
    let core_kinds = ["issue_created", "pr_opened", "message_posted"];
    for kind in &core_kinds {
        assert!(
            ALL_SIGNAL_KINDS.iter().any(|k| k.as_str() == *kind),
            "Canonical registry missing core kind: {}",
            kind
        );
    }
}
