use config::Team;
use semver::Version;
use std::collections::BTreeMap;
use super::{Result, Config, Manifest};

/// Static reducers over available manifests
impl Manifest {

    /// Find the hardcoded versions of services in a region
    ///
    /// Services without a hardcoded version are not returned.
    pub fn get_versions(conf: &Config, region: &str) -> Result<BTreeMap<String, Version>> {
        let services = Manifest::available()?;
        let mut output = BTreeMap::new();

        for svc in services {
            let mf = Manifest::stubbed(&svc, &conf, &region)?;
            if mf.regions.contains(&region.to_string()) {
                if let Some(v) = mf.version {
                    if let Ok(sv) = Version::parse(&v) {
                        output.insert(svc, sv);
                    }
                }
            }
        }
        Ok(output)
    }

    /// Find the hardcoded images of services in a region
    ///
    /// Services without a hardcoded image will assume the shipcat.conf specific default
    pub fn get_images(conf: &Config, region: &str) -> Result<BTreeMap<String, String>> {
        let services = Manifest::available()?;
        let mut output = BTreeMap::new();

        for svc in services {
            let mf = Manifest::stubbed(&svc, &conf, &region)?;
            if mf.regions.contains(&region.to_string()) {
                if let Some(i) = mf.image {
                    output.insert(svc, i);
                }
            }
        }
        Ok(output)
    }

    /// Generate codeowner strings for each service based based on team owners
    ///
    /// Cross references config.teams with manifest.metadata.team
    /// Each returned string is Github CODEOWNER syntax
    pub fn get_codeowners(conf: &Config, region: &str) -> Result<Vec<String>> {
        let services = Manifest::available()?;
        let mut output = vec![];

        for svc in services {
            let mf = Manifest::stubbed(&svc, &conf, &region)?;
            if let Some(md) = mf.metadata {
                let mut ghids = vec![];
                // unwraps guaranteed by validates on Manifest and Config
                let owners = &conf.teams.iter().find(|t| t.name == md.team).unwrap().owners;
                for o in owners.clone() {
                    ghids.push(format!("@{}", o.github.unwrap()));
                }
                if !owners.is_empty() {
                    output.push(format!("services/{}/* {}", mf.name, ghids.join(" ")));
                }
            }
        }
        Ok(output)
    }
}

/// Complete breakdown of resource usage in total, and split by team.
///
/// Normally this is computed by `Manifest::resources` for a region-wide total.
/// Looping over all regions is possible in the CLI.
#[derive(Serialize)]
pub struct ResourceBreakdown {
    /// Total totals
    pub totals: ResourceTotals,
    /// A partition of totals info teams
    pub teams: BTreeMap<String, ResourceTotals>,
}

impl ResourceBreakdown {
    /// Constructor to ensure all valid teams are filled in
    pub fn new(tx: Vec<Team>) -> ResourceBreakdown {
        let mut teams = BTreeMap::new();
        for t in tx {
            teams.insert(t.name, ResourceTotals::default());
        }
        ResourceBreakdown { teams, totals: ResourceTotals::default() }
    }

    /// Round all numbers to gigs and full cores (for all teams)
    pub fn normalise(mut self) -> Self {
        for (_, mut tt) in &mut self.teams {
            tt.base.round();
            tt.extra.round();
        }
        self.totals.base.round();
        self.totals.extra.round();
        self
    }
}

use super::structs::Resources;
use super::math::ResourceTotals;

impl Manifest {
    /// Compute resource usage for all available manifests in a region.
    pub fn resources(conf: &Config, region: &str) -> Result<ResourceBreakdown> {
        let services = Manifest::available()?;
        let mut bd = ResourceBreakdown::new(conf.teams.clone()); // zero for all the things

        let mut sum : Resources<f64> = Default::default();
        let mut extra : Resources<f64> = Default::default(); // autoscaling limits

        for svc in services {
            let mf = Manifest::basic(&svc, &conf, None)?;
            if mf.external || !mf.regions.contains(&region.to_string()) {
                continue; // only care about kube services in the current region
            }
            if let Some(ref md) = mf.metadata {
                let ResourceTotals { base: sb, extra: se } = mf.compute_resource_totals()?;
                sum += sb.clone();
                extra += se.clone();
                let e = bd.teams.get_mut(&md.team).unwrap(); // exists by ResourceBreakdown::new
                e.base += sb.clone();
                e.extra += se.clone();
            } else {
                bail!("{} service does not have resources specification and metadata", mf.name)
            }
        }
        bd.totals.base = sum;
        bd.totals.extra = extra;
        Ok(bd)
    }
}