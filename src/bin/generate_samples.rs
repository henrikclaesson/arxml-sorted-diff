#![cfg(feature = "autosar")]

use anyhow::Result;
use clap::Parser;
use autosar_data::{AutosarModel, ElementName, AutosarVersion};
use std::path::PathBuf;
use std::str::FromStr;

/// Generate sample ARXML files for testing and benchmarking.
#[derive(Parser)]
struct Args {
    /// Output directory to write generated files into
    #[clap(short, long, default_value = ".")]
    out_dir: PathBuf,

    /// Number of top-level packages to generate
    #[clap(short = 'p', long, default_value = "50")]
    packages: usize,

    /// Number of subpackages per package to generate
    #[clap(short = 's', long, default_value = "200")]
    subpackages: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let model = AutosarModel::new();

    let outfile = args.out_dir.join("generated.arxml");
    let file = model.create_file(&outfile, AutosarVersion::LATEST)?;

    // Ensure AR-PACKAGES exists
    let ar_packages = model.root_element().get_or_create_sub_element(ElementName::ArPackages)?;

    for i in 0..args.packages {
        let pkg_name = format!("Pkg{:03}", i);
        let pkg = ar_packages.create_named_sub_element(ElementName::ArPackage, &pkg_name)?;

        // Create one Application SW Component inside the first package for testing
        if i == 0 {
            // Ensure an ELEMENTS container exists for package children and create the component inside it
            let elements_name = ElementName::from_str("ELEMENTS").expect("ELEMENTS variant present");
            let elements = pkg.get_or_create_sub_element(elements_name)?;

            // Add an APPLICATION-SW-COMPONENT-TYPE with one RUNNABLE-ENTITY
            let swc = elements.create_named_sub_element(ElementName::ApplicationSwComponentType, "SwComp1")?;
            // Create SWC internal behavior container then RUNNABLES -> RUNNABLE-ENTITY
            let internal_name = ElementName::from_str("INTERNAL-BEHAVIORS").expect("INTERNAL-BEHAVIORS variant present");
            let internal = swc.get_or_create_sub_element(internal_name)?;
            // Create child SWC-INTERNAL-BEHAVIOR then RUNNABLES -> RUNNABLE-ENTITY
            let internal_behavior_name = ElementName::from_str("SWC-INTERNAL-BEHAVIOR").expect("SWC-INTERNAL-BEHAVIOR variant present");
            let internal_behavior = internal.create_named_sub_element(internal_behavior_name, "internal1")?;
            let run_container_name = ElementName::from_str("RUNNABLES").expect("RUNNABLES variant present");
            let run_container = internal_behavior.get_or_create_sub_element(run_container_name)?;
            let run = run_container.create_named_sub_element(ElementName::RunnableEntity, "run1")?;
            // ensure the runnable and sw component are serialized into the file
            run.add_to_file(&file)?;
            swc.add_to_file(&file)?;
        }

        // Add many sibling packages under AR-PACKAGES to create lots of siblings
        for j in 0..args.subpackages {
            let sub_name = format!("{}_{:04}", pkg_name, j);
            let _subpkg = ar_packages.create_named_sub_element(ElementName::ArPackage, &sub_name)?;
            // ensure the new sibling is serialized into the file
            _subpkg.add_to_file(&file)?;
        }

        // Make sure this package is added to the file so it gets serialized
        pkg.add_to_file(&file)?;
    }

    // Also add the AR-PACKAGES element
    ar_packages.add_to_file(&file)?;

    // Sort model for deterministic output
    model.sort();

    // Write files to disk
    model.write()?;

    println!("Wrote: {}", outfile.display());
    Ok(())
}
