use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::iter;

use walkdir::WalkDir;

use tera::{self, Value, Tera, Context};
use serde_json;
use super::Result;

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn indent(v: Value, m: HashMap<String, Value>) -> tera::Result<Value> {
    let s : String = try_get_value!("indent", "value", String, v);
    // look up indent value or use `2` as default
    let num_spaces : u64 = m.get("spaces").map(Value::as_u64).unwrap_or(None).unwrap_or(2);
    // create an indent string of `num_spaces` spaces
    let indent = iter::repeat(' ').take(num_spaces as usize).collect::<String>();
    // prefix all non-empty lines with `indent`
    let mut xs = vec![];
    for l in s.lines() {
        xs.push(if l == "" { l.to_string() } else { format!("{}{}", indent, l) });
    }
    Ok(serde_json::to_value(&xs.join("\n")).unwrap())
}

fn add_templates(tera: &mut Tera, dir: &PathBuf, svc: &str, depth: usize) -> Result<()> {
    let sdirs = WalkDir::new(&dir)
        .min_depth(depth)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
        // files only
        .filter(|e| e.file_type().is_file())
        // only add files that end in .j2
        .filter(|e| {
            e.file_name().to_string_lossy().ends_with(".j2")
        })
        // if subdirectoried files, only from the directory of the relevant service
        .filter(|e| {
            let mut cmps = e.path().components();
            cmps.next(); // .
            cmps.next(); // services or templates
            // this next bit is only relevant if maxdepth is 2 and we don't want directories
            let last_comp = cmps.next().unwrap(); // folder name or file name!
            let dirname = last_comp.as_os_str().to_str().unwrap();
            let dirpth = dir.join(dirname);
            (!dirpth.is_dir() || dirname == svc)
        });

    // add all templates to the templating engine
    for entry in sdirs {
        let tpth = entry.path();
        debug!("Reading {}", entry.path().display());

        // read it
        let mut f = File::open(&tpth)?;
        let mut data = String::new();
        f.read_to_string(&mut data)?;

        // store in template engine internal hashmap under a easy name
        let fname = tpth.file_name().unwrap().to_string_lossy();
        debug!("Storing {}", fname);
        tera.add_raw_template(&fname, &data)?;
    }
    Ok(())
}

/// Initialise the `tera` templating engine with templates for a service
///
/// This will add global templates, and service specific templates that will be
/// globally available (i.e. by filename as the key).
///
/// Thus, a `Tera` instance is only suitable for one service at a time.
pub fn init(service: &str) -> Result<Tera> {
    let mut tera = Tera::default();
    tera.autoescape_on(vec!["html"]);
    tera.register_filter("indent", indent);

    let services_root = Path::new("."); // TODO: manifests root evar
    // adding templates from template subfolder first
    let tdir = Path::new(&services_root).join("templates");
    add_templates(&mut tera, &tdir, service, 1)?;
    // then templates from service subfolder (as they override)
    let sdir = Path::new(&services_root).join("services");
    add_templates(&mut tera, &sdir, service, 2)?;

    Ok(tera)
}

/// Render convenience function that also trims whitespace
pub fn render(tera: &Tera, tmpl: &str, context: &Context) -> Result<String> {
    let result = tera.render(tmpl, context)?;
    let mut xs = vec![];
    for l in result.lines() {
        // trim whitespace (mostly to satisfy linters)
        xs.push(l.trim_right());
    }
    Ok(xs.join("\n"))
}

/// A function that can render templates for a service
pub type ContextBoundRenderer = Box<Fn(&str, &Context) -> Result<(String)>>;

/// Create a one of boxed template renderer for a service
///
/// Use lightly as it invokes a full template scan per creation
pub fn service_bound_renderer(svc: &str) -> Result<ContextBoundRenderer> {
    let tera = init(svc)?;
    Ok(Box::new(move |tmpl, context| {
        render(&tera, tmpl, context)
    }))
}

/// One off template
pub fn one_off(tpl: &str, ctx: &Context) -> Result<String> {
    let autoescape = false; // we template url with slashes in them
    let res = Tera::one_off(tpl, ctx, autoescape)?;
    Ok(res)
}


// main helpers for the manifest
use super::{Config, Manifest};
impl Manifest {
    // This function defines what variables are available within .j2 templates and evars
    fn make_template_context(&self, conf: &Config, region: &str) -> Result<Context> {
        // need regional kong specifics here
        if conf.regions.get(region).is_none() {
            bail!("Unknown region {} in regions in config", region);
        }
        let reg = conf.regions[region].clone(); // must exist
        // same context as normal templates + base_urls
        let mut ctx = Context::new();
        ctx.add("env", &self.env.clone());
        ctx.add("service", &self.name.clone());
        ctx.add("region", &self.region.clone());
        ctx.add("base_urls", &reg.kong.base_urls);
        Ok(ctx)
    }

    /// Inline templates in values
    fn inline_configs(&mut self, conf: &Config, region: &str) -> Result<()> {
        let svc = self.name.clone();
        let rdr = service_bound_renderer(&self.name)?;
        let ctx = self.make_template_context(conf, region)?;
        if let Some(ref mut cfg) = self.configs {
            for f in &mut cfg.files {
                f.value = Some((rdr)(&f.name, &ctx).map_err(|e| {
                    // help out interleaved reconciles with service name
                    error!("{} failed templating: {}", &svc, e);
                    e
                })?);
            }
        }
        Ok(())
    }

    /// Template evars
    fn template_evars(&mut self, conf: &Config, region: &str) -> Result<()> {
        let ctx = self.make_template_context(conf, region)?;
        for (_, v) in &mut self.env {
            *v = one_off(v, &ctx)?;
        }
        Ok(())
    }

    /// Template everything in the correct order
    pub fn template(&mut self, conf: &Config, region: &str) -> Result<()> {
        self.template_evars(conf, region)?;
        self.inline_configs(conf, region)?;
        Ok(())
    }
}