use clap::Parser;
use git2::Repository;
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs::write;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    release_branch: String,
    #[arg(long, default_value = "prerelease")]
    prerelease_suffix: String,
    #[arg(short, long, default_value = "")]
    tag_prefix: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let github_output_path =
        env::var("GITHUB_OUTPUT").map_err(|_| "GITHUB_OUTPUT environment variable missing.")?;
    let args = Args::parse();
    let working_directory = env::current_dir()?;

    let repository = Repository::discover(working_directory.as_path())
        .map_err(|_| "No git repository found in working directory or parent directories")?;

    let branch_name = repository
        .head()?
        .shorthand()
        .ok_or("Failed to get current branch name")?
        .to_string();

    let prerelease = branch_name != args.release_branch;

    if prerelease {
        println!(
            "Current branch ({branch_name}) is not the release branch ({release_branch}). Including only prerelease tags.",
            release_branch = args.release_branch
        );
    } else {
        println!(
            "Current branch ({branch_name}) is the release branch. Excluding prerelease tags."
        );
    }

    let tags = repository.tag_names(None)?;
    let tags = tags.iter().flatten().collect::<Vec<_>>();
    let latest_tag = get_latest_tag(tags, &args.tag_prefix, &args.prerelease_suffix, prerelease)?;

    println!("Latest tag found: {}", latest_tag);

    // Write as GitHub actions output
    write(github_output_path, format!("latest_tag={}\n", latest_tag))?;

    Ok(())
}

/// Generate the appropriate tag pattern based on whether prerelease tags are considered
/// # Arguments
/// * `prerelease` - A boolean indicating if prerelease tags should be included
/// * `tag_prefix` - The prefix for the tags (e.g., "v")
/// * `prerelease_suffix` - The suffix for prerelease tags (e.g. beta, rc)
/// # Returns
/// A Regex pattern to match the tags
/// # Errors
/// Returns an error if the regex pattern is invalid
fn get_tag_pattern(
    prerelease: bool,
    tag_prefix: &str,
    prerelease_suffix: &str,
) -> Result<Regex, Box<dyn Error>> {
    let tag_pattern = if prerelease {
        Regex::new(&format!(
            r"^{}\d+.\d+.\d+-{}\.\d+$",
            tag_prefix, prerelease_suffix
        ))?
    } else {
        Regex::new(&format!(r"^{}\d+.\d+.\d+$", tag_prefix))?
    };

    Ok(tag_pattern)
}

/// Get the latest tag from a list of tags based on semantic versioning
/// # Arguments
/// * `tags` - A vector of tag strings
/// * `tag_prefix` - The prefix for the tags (e.g., "v")
/// * `prerelease_suffix` - The suffix for prerelease tags (e.g. beta, rc)
/// * `prerelease` - A boolean indicating if prerelease tags should be included
/// # Returns
/// The latest tag as a string
/// # Errors
/// Returns an error if no matching tags are found
fn get_latest_tag(
    tags: Vec<&str>,
    tag_prefix: &str,
    prerelease_suffix: &str,
    prerelease: bool,
) -> Result<String, Box<dyn Error>> {
    let tag_pattern = get_tag_pattern(prerelease, tag_prefix, prerelease_suffix)?;

    let tags: Vec<&str> = tags
        .into_iter()
        .filter(|tag| tag_pattern.is_match(tag))
        .collect();

    let latest_tag = tags.iter().max_by(|a, b| {
        let a_version =
            semver::Version::parse(&a[tag_prefix.len()..]).unwrap_or(semver::Version::new(0, 0, 0));
        let b_version =
            semver::Version::parse(&b[tag_prefix.len()..]).unwrap_or(semver::Version::new(0, 0, 0));
        a_version.cmp(&b_version)
    });

    let latest_tag = if let Some(tag) = latest_tag {
        tag.to_string()
    } else {
        return Err(format!("No tags found matching pattern: {}", tag_pattern).into());
    };

    Ok(latest_tag.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_latest_tag() {
        // Check that the highest stable tag is returned even with an older prerelease of the same major.minor.patch version
        let tags = vec!["v1.0.0", "v1.2.0", "v1.1.5", "v2.0.0-beta.0", "v2.0.0"];
        let latest_tag = get_latest_tag(tags, "v", "beta", false).unwrap();
        assert_eq!(latest_tag, "v2.0.0");

        // Check that the highest stable tag is returned even with a newer prerelease of the same major.minor version
        let tags = vec!["v1.0.0", "v1.2.0", "v1.1.5", "v2.0.0-beta.0"];
        let latest_tag = get_latest_tag(tags, "v", "beta", false).unwrap();
        assert_eq!(latest_tag, "v1.2.0");

        // Check that the highest tag (including prerelease) is returned
        let tags = vec!["v1.0.0-beta.0", "v1.0.0", "v1.1.0-beta.0"];
        let latest_tag = get_latest_tag(tags, "v", "beta", true).unwrap();
        assert_eq!(latest_tag, "v1.1.0-beta.0");

        // Test with different prerelease suffixes
        let tags = vec!["v1.0.0-beta.1", "v1.0.0-beta.2"];
        let latest_tag = get_latest_tag(tags, "v", "beta", true).unwrap();
        assert_eq!(latest_tag, "v1.0.0-beta.2");

        // Test with multi-digit prerelease numbers (check that lexical comparison is not used)
        let tags = vec!["v1.0.0-beta.10", "v1.0.0-beta.2"];
        let latest_tag = get_latest_tag(tags, "v", "beta", true).unwrap();
        assert_eq!(latest_tag, "v1.0.0-beta.10");

        // Check that no matching tags returns an error
        let tags: Vec<&str> = vec![];
        let result = get_latest_tag(tags, "v", "beta", false);
        assert!(result.is_err());
    }
}
