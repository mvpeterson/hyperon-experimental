//!
//! # Module Resolution
//!
//! ## Behavior of Module Resolution
//!
//!       ┌────────────────────┐           ⎽⎼⎻⎺ ⎺⎺⎺ ⎺⎻⎼⎽                    ⎽⎼⎻⎺ ⎺⎺⎺ ⎺⎻⎼⎽
//!      ╱                    ╱       ⎽⎼⎻⎺  bom in &self ⎺⎻⎼⎽ Yes      ⎽⎼⎻⎺ bom entry has ⎺⎻⎼⎽ No
//!     ╱  (import! module)  ╱─────►<      has entry for      >─────►<     fs_path attrib?     >───┐
//!    ╱                    ╱         ⎺⎻⎼⎽   module?     ⎽⎼⎻⎺          ⎺⎻⎼⎽               ⎽⎼⎻⎺     │
//!   └────────────────────┘               ⎺⎻⎼⎽ ⎽⎽⎽ ⎽⎼⎻⎺                    ⎺⎻⎼⎽ ⎽⎽⎽ ⎽⎼⎻⎺          │
//!                                              │ No                             │ Yes            │
//!  ┌─────────────────────────┐     ┌───────────▼─────────────┐      /───────────▼─────────────\  │
//!  │  Query ModuleCatalogs   │     │    Assume any module    │      │    Load the module at   │  │
//!  │     in order, with      │◄──┬─┤  version will satisfy   │      │   the file-system path  │  │
//!  │   version requirement   │   │ │       dependency        │      │   with first successful │  │
//!  │                         │   │ │                         │      │      FsModuleFormat     │  │
//!  └───────────┬─────────────┘   │ └───────────▲─────────────┘      \───────────▲─────────────/  │
//!              │                 │             │                                │                │
//!  /───────────▼─────────────\   │             │                    ┌───────────┴─────────────┐  │
//!  │  Load the module from   │   │             │                    │    clone module from    │  │
//!  │   the first catalog     │   │             │                    │     remote repo to      │  │
//!  │     that reports a      │   │             │                    │    local resource dir   │  │
//!  │    successful match     │   │             │                    │                         │  │
//!  \─────────────────────────/   │             │                    └───────────▲─────────────┘  │
//!                                │             │ No                             │ Yes            │
//!                                │Yes    ⎽⎼⎻⎺ ⎺⎺⎺ ⎺⎻⎼⎽                    ⎽⎼⎻⎺ ⎺⎺⎺ ⎺⎻⎼⎽          │
//!                                │  ⎽⎼⎻⎺ bom entry has ⎺⎻⎼⎽       No ⎽⎼⎻⎺ bom entry has ⎺⎻⎼⎽     │
//!                                └<     version attrib?     >◄─────<       git attrib?       >───┘
//!                                   ⎺⎻⎼⎽               ⎽⎼⎻⎺          ⎺⎻⎼⎽               ⎽⎼⎻⎺
//!                                        ⎺⎻⎼⎽ ⎽⎽⎽ ⎽⎼⎻⎺                    ⎺⎻⎼⎽ ⎽⎽⎽ ⎽⎼⎻⎺
//!

//QUESTION: Should modules loaded manually (without a catalog) be findable by import statements?
//  Currently it's a moot point because the only modules loaded manually are the stdlibs, and those are
//  included by default.  However, an API user might want to forego a catalog just to offer some specific
//  modules, e.g. repl control commands, debug accessors, etc.
//LP-TODO-NEXT Decision: I'll add a flag to `get_or_init_module` called `register`.  If it is true, the module will be
// published by a catalog that is part of the Runner, that is searched before any other catalogs.  If no,
// then the module will only be found by an attempt to load exactly the same ModuleDescriptor
//

//LP-TODO-NEXT validate that the search path includes the working dir by default
//

//LP-TODO-NEXT make a test to make sure circular imports are caught and don't lead to infinite recursion
//QUESTION: Should circular imports between modules be allowed?  The current implementation (and possibly
// the MeTTa language itself) disallow circular imports because there is no concept of forward declaration.
// It *may* be possible to pre-parse the language in order to make recursive imports possible, but I have
// not yet thought in detail about this.
//

//QUESTION on shared base dependencies & sat-set solving:
//The currently implemented design resolves each module's dependencies in a straightforward depth-first
//  order.  This is possible because the module system allows multiple instances of the same module to
//  be loaded simultaneously.  So each module can pick its best dependencies based on its bom and the
//  available catalogs.
//However, consider the following situation:
//  ModA depends on ModI for some interface types (for example a special String type)
//  ModB depends on ModI for the same interface types, but ModA and ModB don't know about each other
//  ModTop depends on both ModA and ModB, and uses functionality in ModA to create some objects that
//   it expects ModB to be able to use.  Therefore the system must guarantee the same version of ModI
//   gets imported by both ModA and ModB.
//This is precisely the opposite behavior from the ability of a module to carry around "private"
//  dependencies and know that those dependencies will always be loaded, and they won't be substituted
//  for another version.
//
//I see several possible solutions:
// 1.) We could disallow private dependencies altogether.  This is the approach taken by Cargo.
//  However this contravenes some of the desiderata outlined in this issue:
//  https://github.com/trueagi-io/hyperon-experimental/issues/470
// 2.) We could require explicit re-exporting of a dependency module used in a module's interface, which
//   would give the implementation an opportunity to find dependency module versions that work for
//   all other modules that use them in common.  ie. solve for the sat set.  Optionally, with this approach,
//   the module could also opt to re-export a private dependency as part of itself, making the interface
//   between ModA and ModB in the example deliberately incompatible. 
// 3.) We could require private dependencies to be explicitly specified as private in the bom.  With the
//  default behavior being a single module for each module name.  This might be a reasonable compromise
//  between 1 & 2, however we would likely need some form of linting, so that a user doesn't shoot
//  themselves in the foot by exporting an interface that includes items from a private dependency
//

use std::ffi::{OsStr, OsString};

use crate::metta::text::OwnedSExprParser;
use crate::metta::runner::*;
use crate::metta::runner::modules::*;

use xxhash_rust::xxh3::xxh3_64;

/// Implemented for types capable of locating MeTTa modules
///
/// For example, `ModuleCatalog` would be an interface to a module respository, analogous to `PyPI` or
/// `crates.io` but `ModuleCatalog` is also implemented for [Path] because any file system directory may be
/// capable of storing and indexing MeTTa modules.
///
/// `ModuleCatalog` types are closely connected with [ModuleLoader] types because the `ModuleCatalog` must
/// recognize the module in whatever media it exists, and supply the `ModuleLoader` to load that module
pub trait ModuleCatalog: std::fmt::Debug + Send + Sync {
    /// Returns the [ModuleDescriptor] for every module in the `ModuleCatalog` with the specified name
    fn lookup(&self, name: &str) -> Vec<ModuleDescriptor>;

    //TODO: Add this function when I add module versioning
    // /// Returns the [ModuleDescriptor] for every module in the `ModuleCatalog` with the specified name
    // /// matching the version requirements
    // fn lookup_within_version_range(name: &str, version_range: ) -> Vec<ModuleDescriptor>;
    //TODO: provide default implementation

    //TODO: Add this function when I add module versioning
    // /// Returns the [ModuleDescriptor] for the newest module in the `ModuleCatalog`, that falls within
    // /// the specified version range, or `None` if no module exists
    // fn lookup_newest_within_version_range(name: &str, version_range: ) -> Option<ModuleDescriptor>;
    //TODO: provide default implementation

    /// Returns a [ModuleLoader] for the specified module from the `ModuleCatalog`
    fn get_loader(&self, descriptor: &ModuleDescriptor) -> Result<Box<dyn ModuleLoader>, String>;
}

/// Implemented to supply a loader functions for MeTTa modules
///
/// In essence, a ModuleLoader is the mechanism to define a MeTTa module format, as it is responsible for
/// loading a module through the MeTTa API.
pub trait ModuleLoader: std::fmt::Debug + Send + Sync {

    /// Returns the [ModuleDescriptor] for the module the `ModuleLoader` is able to load
    ///
    /// NOTE: the [ModuleDescriptor] returned by this method may be different from the ModuleDescriptor
    ///   that gets passed to the loader method, on account of collision avoidance logic between two
    ///   different instances of a module that report the same version
    fn descriptor(&self) -> Result<ModuleDescriptor, String>;

    /// A function to load the module my making MeTTa API calls.  This function will be called by
    /// [Metta::get_or_init_module]
    ///
    /// NOTE: the [ModuleDescriptor] received in the `descriptor` argument should be passed unmodified
    ///   to `context.init_self_module()`
    fn loader(&self, context: &mut RunContext, descriptor: ModuleDescriptor) -> Result<(), String>;
}

/// The object responsible for locating and selecting dependency modules for each [MettaMod]
///
/// This structure is analegous to the `[dependencies]` section of a `Cargo.toml` file for a given module
#[derive(Clone, Debug, Default)]
//TODO: Use serde to deserialize a ModuleBom from an expression atom
pub struct ModuleBom {

    /// The public name of the module.  Should be composed of alpha-numeric characters with '-' and '_'
    /// characters allowed.  Must not contain any other punctuation. 
    pub name: String,

    //TODO: version field, to indicate the version of this module

    /// If `strict == true` then a dependency must be declared in the `ModuleBom`, otherwise a permissive
    /// version requirement will be assumed for any modules that are not explicitly declared
    pub strict: bool,

    /// Entries mapping module names to requirements for the module
    pub deps: HashMap<String, BomEntry>,
}

/// A single entry in a bom, specifying the properties of a module that will satisfy a dependency
#[derive(Clone, Debug, Default)]
pub struct BomEntry {
    /// Indicates that the dependency module should be loaded from a specific FS path
    /// 
    /// If the fs_path is specified, the other bom attributes will be ignored.
    //QUESTION: We need a MeTTa "style guide" for these field names, since they are effective going
    // to be part of the API, because a ModuleBom will be deserialized from atoms
    pub fs_path: Option<PathBuf>

    //TODO: field for fetching from a specific git repo
    //TODO: field to indicate acceptable version range for dependency
}

impl ModuleBom {
    /// Resolves which module to load from which available location or catalog, and returns the [ModuleLoader] to
    /// load that module
    pub fn resolve_module<'c>(&self, context: &'c RunContext, name: &str) -> Result<Option<(Box<dyn ModuleLoader + 'c>, ModuleDescriptor)>, String> {

        //See if we have a bom entry for the module
        if let Some(entry) = self.deps.get(name) {

            //If path is explicitly specified in the bom entry, then we must load the module at the
            // specified path, and cannot search anywhere else
            if let Some(path) = &entry.fs_path {

                //If the path is not an absolute path, assume it's relative to the module's working dir
                let path = if path.is_absolute() {
                    path.clone()
                } else {
                    context.module().working_dir()
                        .unwrap_or_else(|| panic!("Error loading {}. Module without a working directory cannot load dependencies by relative path", context.module().descriptor().name))
                        .join(path)
                };

                //Check all module formats, to try and load the module at the path
                for fmt in context.metta.environment().fs_mod_formats() {
                    if let Some(loader) = fmt.try_path(&path) {
                        match loader.descriptor() {
                            Ok(mut descriptor) => {
                                //Construct a uid based on a stable-hash of the path, because a module
                                // loaded by explicit path shouldn't be imported by any other module unless
                                // it explicitly imports it from the same path.
                                descriptor.uid = Some(xxh3_64(path.as_os_str().as_encoded_bytes()));

                                return Ok(Some((loader, descriptor)))
                            },
                            Err(err) => {
                                panic!("bom specifies invalid module at {}, Error: {}!", path.display(), err);
                            }
                        };
                    }
                }

                //
            }

            //TODO, if git URI is specified in the bom entry, clone the repo to a location in the environment
            // dir with a unique path (based on a random uuid), and resolve it within that directory's catalog

            //TODO, If a version range is specified in the bom entry, then use that version range

        } else {
            //If the Bom doesn't have an entry, it's an error if the Bom is flagged as "strict"
            if self.strict {
                return Ok(None);
            }
        }

        //QUESTION: Do we want to search the module's Working Dir before searching the environment's catalogs?
        // This would allow a module to import another module inside its directory or as a peer of itself
        // for single-file modules, without including an explicit bom entry.  On the other hand, we might want
        // to require module authors to include a bom entry to be explicit about their dependencies

        //Search the catalogs in the runner's Environment
        for catalog in context.metta.environment().catalogs() {
            log::trace!("Looking for module: \"{name}\" inside {catalog:?}");
            //TODO: use lookup_newest_within_version_range, as soon as I add module versioning
            let results = catalog.lookup(name);
            if results.len() > 0 {
                log::info!("Found module: \"{name}\" inside {catalog:?}");
                let descriptor = results.into_iter().next().unwrap();
                return Ok(Some((catalog.get_loader(&descriptor)?, descriptor)))
            }
        }

        Ok(None)
    }
}

/// A loader for a MeTTa module that lives within a single `.metta` file
#[derive(Debug)]
pub(crate) struct SingleFileModule(pub(crate) PathBuf);

impl ModuleLoader for SingleFileModule {
    fn descriptor(&self) -> Result<ModuleDescriptor, String> {

        //TODO: Try and read the module name and version here
        //In a single-file module, the discriptor information will be embedded within the MeTTa code
        // Therefore, we need to parse the whole text of the module looking for a `_module-bom` atom,
        // that we can then convert into a ModuleBom structure

        //If there is no internal name in the module, parse it from the module's path
        let name = self.0.file_stem().unwrap().to_str().unwrap();

        Ok(ModuleDescriptor::new(name.to_string()))
    }
    fn loader(&self, context: &mut RunContext, descriptor: ModuleDescriptor) -> Result<(), String> {

        let space = DynSpace::new(GroundingSpace::new());
        let working_dir = self.0.parent().unwrap();
        context.init_self_module(descriptor, space, Some(working_dir.into()));

        let program_text = std::fs::read_to_string(&self.0)
            .map_err(|err| format!("Could not read file, path: {}, error: {}", self.0.display(), err))?;

        let parser = OwnedSExprParser::new(program_text);
        context.push_parser(parser);

        Ok(())
    }
}

/// A loader for a MeTTa module implemented as a directory
#[derive(Debug)]
pub(crate) struct DirModule(pub(crate) PathBuf);

impl ModuleLoader for DirModule {
    fn descriptor(&self) -> Result<ModuleDescriptor, String> {

        //TODO: Try and read the module name and version here
        //If there is a `bom.metta` file, descriptor information from that file will take precedence.
        // Otherwise, try and parse a `_module-bom` atom from the `module.metta` file

        //If we still have not found an internal name, use the file-system name
        let name = self.0.file_stem().unwrap().to_str().unwrap();

        Ok(ModuleDescriptor::new(name.to_string()))
    }
    fn loader(&self, context: &mut RunContext, descriptor: ModuleDescriptor) -> Result<(), String> {

        let space = DynSpace::new(GroundingSpace::new());
        let working_dir = self.0.parent().unwrap();
        context.init_self_module(descriptor, space, Some(working_dir.into()));

        let module_metta_path = self.0.join("module.metta");
        let program_text = std::fs::read_to_string(&module_metta_path)
            .map_err(|err| format!("Failed to read metta file in directory module, path: {}, error: {}", module_metta_path.display(), err))?;

        let parser = OwnedSExprParser::new(program_text);
        context.push_parser(parser);

        Ok(())
    }
}

/// Implemented on a type to test if a given file-system path points to a MeTTa module, and to construct
/// possible paths within a parent directory for a module of a certain name
///
/// Objects implementing this trait work with in conjunction with [DirCatalog] and [ModuleBom] to facilitate
/// loading modules from include directories, specific paths, and remote `git` repositories.
pub trait FsModuleFormat: std::fmt::Debug + Send + Sync {

    /// Returns the possible paths inside a parent directory which may point to a module
    fn paths_for_name(&self, parent_dir: &Path, mod_name: &str) -> Vec<PathBuf>;

    /// Checks a specific path, and returns the [ModuleLoader] if a supported module resides at the path
    fn try_path(&self, path: &Path) -> Option<Box<dyn ModuleLoader>>;
}

/// An object to identify and load a single-file module (naked .metta files)
#[derive(Debug)]
pub struct SingleFileModuleFmt;

impl FsModuleFormat for SingleFileModuleFmt {
    fn paths_for_name(&self, parent_dir: &Path, mod_name: &str) -> Vec<PathBuf> {
        let path = parent_dir.join(mod_name);
        let path = push_extension(path, ".metta");
        vec![path]
    }
    fn try_path(&self, path: &Path) -> Option<Box<dyn ModuleLoader>> {
        if path.is_file() {
            Some(Box::new(SingleFileModule(path.into())))
        } else {
            None
        }
    }
}

/// An object to identify and load a MeTTa module implemented as a directory
#[derive(Debug)]
pub struct DirModuleFmt;

impl FsModuleFormat for DirModuleFmt {
    fn paths_for_name(&self, parent_dir: &Path, mod_name: &str) -> Vec<PathBuf> {
        let path = parent_dir.join(mod_name);
        vec![path]
    }
    fn try_path(&self, path: &Path) -> Option<Box<dyn ModuleLoader>> {
        if path.is_dir() {
            Some(Box::new(DirModule(path.into())))
        } else {
            None
        }
    }
}

/// Implements ModuleCatalog to load MeTTa modules from a file-system directory trying a number of
/// [FsModuleFormat] formats in succession
#[derive(Clone, Debug)]
pub struct DirCatalog {
    path: PathBuf,
    fmts: Arc<Vec<Box<dyn FsModuleFormat>>>,
}

impl DirCatalog {
    /// Internal function to initialize a DirCatalog for a directory in the module search path
    pub(crate) fn new(path: PathBuf, fmts: Arc<Vec<Box<dyn FsModuleFormat>>>) -> Self {
        Self {path, fmts}
    }
}

impl ModuleCatalog for DirCatalog {
    fn lookup(&self, name: &str) -> Vec<ModuleDescriptor> {

        //QUESTION: How should we handle modules with an internal "package-name" that differs from their
        // name in the file system?
        //
        //Cargo treats the module's internal "package-name" as authoritative, but that means it's impossible
        // to "install" a module simply by dropping it into a directory in the search path, because there
        // needs to be an index of all available modules in that directory.
        //
        //For us, I think we want a less formal approach akin to Python's, where we are allowed to drop a
        // module into a directory, and import it with a naked `import` statement (i.e. no bom entry)
        // but for that to work, we need to stipulate that the file name without any extensions matches
        // the module name.
        //

        let mut found_modules = vec![];

        //Inspect the directory using each FsModuleFormat, in order
        visit_modules_in_dir_using_mod_formats(&self.fmts, &self.path, name, |path, _loader, mut descriptor| {

            //Update the descriptor's uid with a stable-hash based on the path, because we might get
            // multiple instances purporting to be the same module
            descriptor.uid = Some(xxh3_64(path.as_os_str().as_encoded_bytes()));

            found_modules.push(descriptor);
            false
        });

        found_modules
    }
    fn get_loader(&self, descriptor: &ModuleDescriptor) -> Result<Box<dyn ModuleLoader>, String> {

        let mut matching_module = None;
        visit_modules_in_dir_using_mod_formats(&self.fmts, &self.path, &descriptor.name, |path, loader, mut resolved_descriptor| {
            //Reconstruct the descriptor using the same logic as above
            resolved_descriptor.uid = Some(xxh3_64(path.as_os_str().as_encoded_bytes()));
            if &resolved_descriptor == descriptor {
                matching_module = Some(loader);
                true
            } else {
                false
            }
        });

        match matching_module {
            Some(loader) => Ok(loader),
            None => Err(format!("Failed to load module {} in directory {}", &descriptor.name, self.path.display()))
        }
    }
}

/// Internal Utility Function.  Blindly appends an extension onto a path, even if the path already
/// has an extension
fn push_extension(path: PathBuf, extension: impl AsRef<OsStr>) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(extension.as_ref());
    os_string.into()
}

/// Internal function to try FsModuleFormat formats in order.  If the closure returns `true` this function
/// will exit, otherwise it will try every path returned by every format
fn visit_modules_in_dir_using_mod_formats(fmts: &[Box<dyn FsModuleFormat>], dir_path: &Path, mod_name: &str, mut f: impl FnMut(PathBuf, Box<dyn ModuleLoader>, ModuleDescriptor) -> bool) {

    for fmt in fmts {
        for path in fmt.paths_for_name(dir_path, mod_name) {
            if let Some(loader) = fmt.try_path(&path) {
                match loader.descriptor() {
                    Ok(resolved_descriptor) => {
                        if f(path, loader, resolved_descriptor) {
                            return;
                        }
                    },
                    Err(err) => {
                        log::warn!("Warning! invalid module at {:?}, Error: {}!", loader, err);
                    }
                };
            }
        }
    }
}
