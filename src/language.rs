use crate::config::{CheckCodeConfig, LanguageConfig};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Get default fence markers for a language based on highlight.js language definitions.
///
/// This function returns the canonical language name plus common aliases that highlight.js
/// recognizes for syntax highlighting. If no built-in mapping exists, returns just the
/// language name itself.
///
/// # Arguments
///
/// * `lang_name` - The language name from the configuration section (e.g., "c", "typescript")
///
/// # Returns
///
/// A vector of fence marker strings that should recognize this language in markdown.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(get_default_fence_markers("c"), vec!["c", "h"]);
/// assert_eq!(get_default_fence_markers("typescript"), vec!["typescript", "ts", "tsx", "mts", "cts"]);
/// assert_eq!(get_default_fence_markers("unknown"), vec!["unknown"]);
/// ```
///
/// # Reference
///
/// Language aliases are based on highlight.js SUPPORTED_LANGUAGES.md:
/// https://github.com/highlightjs/highlight.js/blob/main/SUPPORTED_LANGUAGES.md
pub fn get_default_fence_markers(lang_name: &str) -> Vec<String> {
    match lang_name {
        // Numbers & Special
        "1c" => vec!["1c"],
        "4d" => vec!["4d"],

        // A
        "abap" => vec!["sap-abap", "abap"],
        "abc" => vec!["abc"],
        "abnf" => vec!["abnf"],
        "accesslog" => vec!["accesslog"],
        "actionscript" => vec!["actionscript", "as"],
        "ada" => vec!["ada"],
        "aiken" => vec!["aiken", "ak"],
        "alan" => vec!["alan", "i", "ln"],
        "angelscript" => vec!["angelscript", "asc"],
        "apache" => vec!["apache", "apacheconf"],
        "apex" => vec!["apex"],
        "applescript" => vec!["applescript", "osascript"],
        "arcade" => vec!["arcade"],
        "arduino" => vec!["arduino", "ino"],
        "armasm" => vec!["armasm", "arm"],
        "asciidoc" => vec!["asciidoc", "adoc"],
        "aspectj" => vec!["aspectj"],
        "autohotkey" => vec!["autohotkey"],
        "autoit" => vec!["autoit"],
        "avrasm" => vec!["avrasm"],
        "awk" => vec!["awk", "mawk", "nawk", "gawk"],

        // B
        "bash" => vec!["bash", "sh", "zsh"],
        "basic" => vec!["basic"],
        "bnf" => vec!["bnf"],
        "brainfuck" => vec!["brainfuck", "bf"],

        // C
        "c" => vec!["c", "h"],
        "cal" => vec!["cal"],
        "capnproto" => vec!["capnproto", "capnp"],
        "ceylon" => vec!["ceylon"],
        "clean" => vec!["clean", "icl", "dcl"],
        "clojure" => vec!["clojure", "clj"],
        "clojurerepl" => vec!["clojure-repl"],
        "cmake" => vec!["cmake", "cmake.in"],
        "coffeescript" => vec!["coffeescript", "coffee", "cson", "iced"],
        "coq" => vec!["coq"],
        "cos" => vec!["cos", "cls"],
        "cpp" => vec!["cpp", "hpp", "cc", "hh", "c++", "h++", "cxx", "hxx"],
        "crmsh" => vec!["crmsh", "crm", "pcmk"],
        "crystal" => vec!["crystal", "cr"],
        "csharp" => vec!["csharp", "cs"],
        "csp" => vec!["csp"],
        "css" => vec!["css"],

        // D
        "d" => vec!["d"],
        "dart" => vec!["dart"],
        "delphi" => vec!["delphi", "dpr", "dfm", "pas", "pascal"],
        "diff" => vec!["diff", "patch"],
        "django" => vec!["django", "jinja"],
        "dns" => vec!["dns", "zone", "bind"],
        "dockerfile" => vec!["dockerfile", "docker"],
        "dos" => vec!["dos", "bat", "cmd"],
        "dsconfig" => vec!["dsconfig"],
        "dts" => vec!["dts"],
        "dust" => vec!["dust", "dst"],

        // E
        "ebnf" => vec!["ebnf"],
        "elixir" => vec!["elixir"],
        "elm" => vec!["elm"],
        "erb" => vec!["erb"],
        "erlang" => vec!["erlang", "erl"],
        "erlang-repl" => vec!["erlang-repl"],
        "excel" => vec!["excel", "xls", "xlsx"],

        // F
        "fix" => vec!["fix"],
        "flix" => vec!["flix"],
        "fortran" => vec!["fortran", "f90", "f95"],
        "fsharp" => vec!["fsharp", "fs", "fsx", "fsi", "fsscript"],

        // G
        "gams" => vec!["gams", "gms"],
        "gauss" => vec!["gauss", "gss"],
        "gcode" => vec!["gcode", "nc"],
        "gherkin" => vec!["gherkin"],
        "glsl" => vec!["glsl"],
        "gml" => vec!["gml"],
        "go" => vec!["go", "golang"],
        "golo" => vec!["golo", "gololang"],
        "gradle" => vec!["gradle"],
        "graphql" => vec!["graphql", "gql"],
        "groovy" => vec!["groovy"],

        // H
        "haml" => vec!["haml"],
        "handlebars" => vec!["handlebars", "hbs", "html.hbs", "html.handlebars"],
        "haskell" => vec!["haskell", "hs"],
        "haxe" => vec!["haxe", "hx"],
        "hsp" => vec!["hsp"],
        "html" => vec!["html", "xhtml"],
        "http" => vec!["http", "https"],
        "hy" => vec!["hy", "hylang"],

        // I
        "inform7" => vec!["inform7", "i7"],
        "ini" => vec!["ini", "toml"],
        "irpf90" => vec!["irpf90"],
        "isbl" => vec!["isbl"],

        // J
        "java" => vec!["java", "jsp"],
        "javascript" => vec!["javascript", "js", "jsx"],
        "jbosscli" => vec!["jboss-cli", "wildfly-cli"],
        "json" => vec!["json", "jsonc", "json5"],
        "julia" => vec!["julia", "julia-repl"],

        // K
        "kotlin" => vec!["kotlin", "kt"],

        // L
        "lasso" => vec!["lasso", "ls", "lassoscript"],
        "latex" => vec!["tex"],
        "ldif" => vec!["ldif"],
        "leaf" => vec!["leaf"],
        "less" => vec!["less"],
        "lisp" => vec!["lisp"],
        "livecodeserver" => vec!["livecodeserver"],
        "livescript" => vec!["livescript", "ls"],
        "llvm" => vec!["llvm"],
        "lsl" => vec!["lsl"],
        "lua" => vec!["lua", "pluto"],

        // M
        "makefile" => vec!["makefile", "mk", "mak", "make"],
        "markdown" => vec!["markdown", "md", "mkdown", "mkd"],
        "mathematica" => vec!["mathematica", "mma", "wl"],
        "matlab" => vec!["matlab"],
        "maxima" => vec!["maxima"],
        "mel" => vec!["mel"],
        "mercury" => vec!["mercury"],
        "mipsasm" => vec!["mipsasm", "mips"],
        "mizar" => vec!["mizar"],
        "mojolicious" => vec!["mojolicious"],
        "monkey" => vec!["monkey"],
        "moonscript" => vec!["moonscript", "moon"],

        // N
        "n1ql" => vec!["n1ql"],
        "nestedtext" => vec!["nestedtext", "nt"],
        "nginx" => vec!["nginx", "nginxconf"],
        "nim" => vec!["nim", "nimrod"],
        "nix" => vec!["nix"],
        "node-repl" => vec!["node-repl"],
        "nsis" => vec!["nsis"],

        // O
        "objectivec" => vec!["objectivec", "mm", "objc", "obj-c"],
        "ocaml" => vec!["ocaml", "ml"],
        "openscad" => vec!["openscad", "scad"],
        "oxygene" => vec!["oxygene"],

        // P
        "parser3" => vec!["parser3"],
        "perl" => vec!["perl", "pl", "pm"],
        "pf" => vec!["pf", "pf.conf"],
        "pgsql" => vec!["pgsql", "postgres", "postgresql"],
        "php" => vec!["php"],
        "phptemplate" => vec!["php-template"],
        "plaintext" => vec!["plaintext", "txt", "text"],
        "pony" => vec!["pony"],
        "powershell" => vec!["powershell", "ps", "ps1"],
        "processing" => vec!["processing"],
        "profile" => vec!["profile"],
        "prolog" => vec!["prolog"],
        "properties" => vec!["properties"],
        "protobuf" => vec!["protobuf"],
        "puppet" => vec!["puppet", "pp"],
        "purebasic" => vec!["purebasic", "pb", "pbi"],
        "python" => vec!["python", "py", "gyp"],
        "pythonrepl" => vec!["python-repl", "pycon"],

        // Q
        "q" => vec!["k", "kdb"],
        "qml" => vec!["qml"],

        // R
        "r" => vec!["r"],
        "reasonml" => vec!["reasonml", "re"],
        "rib" => vec!["rib"],
        "roboconf" => vec!["graph", "instances"],
        "routeros" => vec!["routeros", "mikrotik"],
        "rsl" => vec!["rsl"],
        "ruby" => vec!["ruby", "rb", "gemspec", "podspec", "thor", "irb"],
        "ruleslanguage" => vec!["ruleslanguage"],
        "rust" => vec!["rust", "rs"],

        // S
        "sas" => vec!["sas"],
        "scala" => vec!["scala"],
        "scheme" => vec!["scheme"],
        "scilab" => vec!["scilab", "sci"],
        "scss" => vec!["scss"],
        "shell" => vec!["shell", "console"],
        "smali" => vec!["smali"],
        "smalltalk" => vec!["smalltalk", "st"],
        "sml" => vec!["sml", "ml"],
        "solidity" => vec!["solidity", "sol"],
        "sqf" => vec!["sqf"],
        "sql" => vec!["sql"],
        "stan" => vec!["stan", "stanfuncs"],
        "stata" => vec!["stata"],
        "step21" => vec!["step21", "p21", "step", "stp"],
        "stylus" => vec!["stylus", "styl"],
        "subunit" => vec!["subunit"],
        "swift" => vec!["swift"],

        // T
        "taggerscript" => vec!["taggerscript"],
        "tap" => vec!["tap"],
        "tcl" => vec!["tcl", "tk"],
        "thrift" => vec!["thrift"],
        "toml" => vec!["toml"],
        "tp" => vec!["tp"],
        "twig" => vec!["twig", "craftcms"],
        "typescript" => vec!["typescript", "ts", "tsx", "mts", "cts"],

        // V
        "vala" => vec!["vala"],
        "vbnet" => vec!["vbnet", "vb"],
        "vbscript" => vec!["vbscript", "vbs"],
        "vbscripthtmlvbscript" => vec!["vbscript-html"],
        "verilog" => vec!["verilog", "v"],
        "vhdl" => vec!["vhdl"],
        "vim" => vec!["vim"],
        "wasm" => vec!["wasm"],
        "wren" => vec!["wren"],

        // X
        "x86asm" => vec!["x86asm"],
        "xl" => vec!["xl", "tao"],
        "xml" => vec!["xml", "rss", "atom", "xjb", "xsd", "xsl", "plist", "svg"],
        "xquery" => vec!["xquery", "xpath", "xq"],

        // Y
        "yaml" => vec!["yaml", "yml"],

        // Z
        "zephir" => vec!["zephir", "zep"],
        "zig" => vec!["zig"],

        // Default: use the language name itself
        _ => vec![lang_name],
    }
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// A language implementation configured from `book.toml`.
///
/// This struct represents a language whose behavior is entirely determined by
/// configuration rather than hardcoded logic, using the compiler path, flags,
/// and preamble specified in the configuration.
///
/// # Configuration-Driven Design
///
/// All compilation behavior comes from `book.toml`:
/// - Compiler path (with `${VAR}` environment variable expansion)
/// - Compiler flags (array of strings)
/// - Optional preamble (prepended to all blocks)
/// - Fence markers (which markdown fences map to this language)
pub struct ConfiguredLanguage {
    name: String,
    config: LanguageConfig,
    file_extension: String,
}

impl ConfiguredLanguage {
    pub fn new(name: String, config: LanguageConfig) -> Self {
        let file_extension = Self::determine_file_extension(&config.fence_markers);
        Self {
            name,
            config,
            file_extension,
        }
    }

    /// Returns the name of this language (e.g., "c", "c-parasol", "typescript").
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the file extension for this language (e.g., ".c", ".ts").
    pub fn file_extension(&self) -> &str {
        &self.file_extension
    }

    /// Determine file extension from fence markers (first marker + common extensions)
    fn determine_file_extension(fence_markers: &[String]) -> String {
        if let Some(first_marker) = fence_markers.first() {
            match first_marker.as_str() {
                "c" | "parasol-c" | "parasol" => ".c".to_string(),
                "cpp" | "c++" | "cxx" => ".cpp".to_string(),
                "rust" | "rs" => ".rs".to_string(),
                "python" | "py" => ".py".to_string(),
                "javascript" | "js" => ".js".to_string(),
                "typescript" | "ts" => ".ts".to_string(),
                "go" => ".go".to_string(),
                "java" => ".java".to_string(),
                _ => format!(".{}", first_marker),
            }
        } else {
            ".txt".to_string()
        }
    }

    /// Compiles or validates the given code.
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to validate (may include preambles)
    /// * `temp_file` - Path where the code should be written for compilation
    ///
    /// # Returns
    ///
    /// * `Ok(())` if compilation succeeds
    /// * `Err` with compilation error details if it fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The temporary file cannot be created or written
    /// - The compiler executable cannot be found or executed
    /// - The code fails to compile
    pub fn compile(&self, code: &str, temp_file: &Path) -> Result<()> {
        // Write code with optional preamble to temp file
        let mut file = File::create(temp_file).context("Failed to create temporary file")?;

        if let Some(ref preamble) = self.config.preamble {
            writeln!(file, "{}", preamble)?;
            writeln!(file)?;
        }

        write!(file, "{}", code)?;
        drop(file);

        // Execute compiler with configured flags
        let output = Command::new(&self.config.compiler)
            .args(&self.config.flags)
            .arg(temp_file)
            .output()
            .with_context(|| {
                format!(
                    "Failed to execute compiler '{}' for language '{}'",
                    self.config.compiler, self.name
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            anyhow::bail!("{} compilation failed:\n{}", self.name, error_msg);
        }

        Ok(())
    }
}

/// Registry of available languages for code validation.
///
/// The registry is built from the configuration and provides lookup
/// functionality to find languages by their fence markers.
///
/// # Example
///
/// ```ignore
/// let config = CheckCodeConfig::from_preprocessor_context(&ctx)?;
/// let registry = LanguageRegistry::from_config(&config);
///
/// // Find a language by fence marker
/// if let Some(lang) = registry.find_by_fence("c", None) {
///     lang.compile(code, &temp_file)?;
/// }
/// ```
pub struct LanguageRegistry {
    config: CheckCodeConfig,
}

impl LanguageRegistry {
    /// Creates a new language registry from configuration.
    ///
    /// The registry stores the configuration and creates language instances
    /// on demand when `find_by_fence` is called.
    pub fn from_config(config: &CheckCodeConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Finds a language by its fence marker and optional variant.
    ///
    /// # Arguments
    ///
    /// * `fence` - The fence marker from a markdown code block (e.g., "c", "ts")
    /// * `variant` - Optional variant name (e.g., Some("parasol") for C with Parasol compiler)
    ///
    /// # Returns
    ///
    /// * `Some(Box<dyn Language>)` if a language with this fence marker exists
    /// * `None` if no language is configured for this fence marker
    ///
    /// # Variant Handling
    ///
    /// When a variant is specified, the variant's configuration (compiler, flags, preamble)
    /// overrides the base language configuration. The variant inherits fence_markers and
    /// file_extension from the base language.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Base C language
    /// if let Some(lang) = registry.find_by_fence("c", None) {
    ///     println!("Found language: {}", lang.name());
    /// }
    ///
    /// // Parasol variant of C
    /// if let Some(lang) = registry.find_by_fence("c", Some("parasol")) {
    ///     println!("Found language: {}", lang.name());
    /// }
    /// ```
    pub fn find_by_fence(&self, fence: &str, variant: Option<&str>) -> Option<ConfiguredLanguage> {
        // Find the base language config by fence marker (using resolved fence markers)
        let (lang_name, base_config) = self.config.languages().iter().find(|(name, config)| {
            if !config.enabled {
                return false;
            }
            let fence_markers = config.get_fence_markers(name);
            fence_markers.contains(&fence.to_string())
        })?;

        // If no variant is specified, create base language with resolved fence markers
        let variant_name = match variant {
            None => {
                // Get resolved fence markers for the base language
                let resolved_fence_markers = base_config.get_fence_markers(lang_name);

                // Create config with resolved fence markers
                let resolved_config = crate::config::LanguageConfig {
                    enabled: base_config.enabled,
                    compiler: base_config.compiler.clone(),
                    flags: base_config.flags.clone(),
                    preamble: base_config.preamble.clone(),
                    fence_markers: resolved_fence_markers,
                    variants: base_config.variants.clone(),
                };

                return Some(ConfiguredLanguage::new(lang_name.clone(), resolved_config));
            }
            Some(v) => v,
        };

        // Look up the variant config
        let variant_config = base_config.variants.get(variant_name)?;

        // Get resolved fence markers from base config
        let resolved_fence_markers = base_config.get_fence_markers(lang_name);

        // Create merged config: variant settings override base settings
        let merged_config = crate::config::LanguageConfig {
            enabled: base_config.enabled,
            compiler: variant_config.compiler.clone(),
            flags: variant_config.flags.clone(),
            preamble: variant_config.preamble.clone(),
            fence_markers: resolved_fence_markers,
            variants: std::collections::HashMap::new(), // Variants don't inherit variants
        };

        // Create a new language with variant-specific name
        let variant_lang_name = format!("{}-{}", lang_name, variant_name);
        Some(ConfiguredLanguage::new(variant_lang_name, merged_config))
    }
}
