use clap::Parser;
use git2::Repository;
use std::env;
use std::error::Error;
use std::fs::write;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    release_branch: String,
    #[arg(short, long)]
    prerelease: bool,
    #[arg(long, default_value = "prerelease")]
    prerelease_suffix: String,
    #[arg(short, long, default_value = "")]
    tag_prefix: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let github_output_path = env::var("GITHUB_OUTPUT")?;
    let args = Args::parse();
    let working_directory = env::current_dir()?;

    let repository = Repository::discover(working_directory.as_path())
        .map_err(|_| "No git repository found in working directory or parent directories")?;

    let prerelease_matcher = if args.prerelease {
        args.prerelease_suffix.as_str()
    } else {
        ""
    };
    let tag_pattern = format!("{}*{}*", args.tag_prefix, prerelease_matcher);
    let tags = repository.tag_names(Some(&tag_pattern))?;
    let tags = tags.iter().flatten().collect::<Vec<_>>();

    let latest_tag = get_latest_tag(tag_pattern, args.tag_prefix.as_str(), tags)?;

    // Write as GitHub actions output
    write(github_output_path, format!("latest_tag={}\n", latest_tag))?;

    Ok(())
}

fn get_latest_tag(
    tag_pattern: String,
    tag_prefix: &str,
    tags: Vec<&str>,
) -> Result<String, Box<dyn Error>> {
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
        let latest_tag = get_latest_tag("v*".to_string(), "v", tags).unwrap();
        assert_eq!(latest_tag, "v2.0.0");

        // Check that the highest tag (including prerelease) is returned
        let tags = vec!["v1.0.0-beta.0", "v1.0.0", "v1.1.0-beta.0"];
        let latest_tag = get_latest_tag("v*beta*".to_string(), "v", tags).unwrap();
        assert_eq!(latest_tag, "v1.1.0-beta.0");

        // Test with different prerelease suffixes
        let tags = vec!["v1.0.0-beta.1", "v1.0.0-beta.2"];
        let latest_tag = get_latest_tag("v*beta*".to_string(), "v", tags).unwrap();
        assert_eq!(latest_tag, "v1.0.0-beta.2");

        // Test with multi-digit prerelease numbers (check that lexical comparison is not used)
        let tags = vec!["v1.0.0-beta.10", "v1.0.0-beta.2"];
        let latest_tag = get_latest_tag("v*beta*".to_string(), "v", tags).unwrap();
        assert_eq!(latest_tag, "v1.0.0-beta.10");

        // Check that no matching tags returns an error
        let tags: Vec<&str> = vec![];
        let result = get_latest_tag("v*".to_string(), "v", tags);
        assert!(result.is_err());
    }
}
