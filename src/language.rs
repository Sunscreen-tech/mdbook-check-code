use crate::config::{CheckCodeConfig, LanguageConfig};
use anyhow::{Context, Result};
use std::borrow::Cow;
use std::fmt;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Metadata for a programming language including fence markers and file extension.
///
/// Uses `Cow<'static, str>` for the file extension to avoid allocations for known
/// languages while supporting dynamic extensions for custom languages.
///
/// # Special Cases
///
/// - **Makefile**: Returns "Makefile" (no dot prefix) instead of ".makefile" since
///   Makefiles conventionally use this exact filename rather than an extension.
#[derive(Debug, Clone)]
pub struct LanguageMetadata {
    pub fence_markers: Vec<String>,
    pub file_extension: Cow<'static, str>,
}

impl LanguageMetadata {
    /// Returns whether this file extension represents a complete filename rather than an extension.
    ///
    /// Some languages like Makefile use a specific filename convention without a dot prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use mdbook_check_code::get_language_metadata;
    ///
    /// let makefile_meta = get_language_metadata("makefile");
    /// assert!(makefile_meta.is_complete_filename());
    /// assert_eq!(makefile_meta.file_extension, "Makefile");
    ///
    /// let c_meta = get_language_metadata("c");
    /// assert!(!c_meta.is_complete_filename());
    /// assert_eq!(c_meta.file_extension, ".c");
    /// ```
    #[allow(dead_code)] // Public API utility, usage will grow
    pub fn is_complete_filename(&self) -> bool {
        !self.file_extension.starts_with('.')
    }
}

/// Get language metadata (fence markers and file extension) for a language.
///
/// This function returns the canonical language name plus common aliases that highlight.js
/// recognizes for syntax highlighting, along with the standard file extension for the language.
/// If no built-in mapping exists, returns the language name as the fence marker and uses it
/// as the file extension with a dot prefix.
///
/// # Arguments
///
/// * `lang_name` - The language name from the configuration section (e.g., "c", "typescript")
///
/// # Returns
///
/// `LanguageMetadata` containing fence markers and file extension.
///
/// # Examples
///
/// ```
/// use mdbook_check_code::get_language_metadata;
///
/// let metadata = get_language_metadata("c");
/// assert_eq!(metadata.fence_markers, vec!["c", "h"]);
/// assert_eq!(metadata.file_extension, ".c");
///
/// let metadata = get_language_metadata("typescript");
/// assert_eq!(metadata.file_extension, ".ts");
/// ```
///
/// # Reference
///
/// - Fence markers based on highlight.js SUPPORTED_LANGUAGES.md
/// - File extensions based on GitHub Linguist languages.yml
pub fn get_language_metadata(lang_name: &str) -> LanguageMetadata {
    let (markers, ext) = match lang_name {
        // Numbers & Special
        "1c" => (vec!["1c"], ".bsl"),
        "4d" => (vec!["4d"], ".4dm"),

        // A
        "abap" => (vec!["sap-abap", "abap"], ".abap"),
        "abc" => (vec!["abc"], ".abc"),
        "abnf" => (vec!["abnf"], ".abnf"),
        "accesslog" => (vec!["accesslog"], ".log"),
        "actionscript" => (vec!["actionscript", "as"], ".as"),
        "ada" => (vec!["ada"], ".adb"),
        "aiken" => (vec!["aiken", "ak"], ".ak"),
        "alan" => (vec!["alan", "i", "ln"], ".alan"),
        "angelscript" => (vec!["angelscript", "asc"], ".as"),
        "apache" => (vec!["apache", "apacheconf"], ".conf"),
        "apex" => (vec!["apex"], ".cls"),
        "applescript" => (vec!["applescript", "osascript"], ".scpt"),
        "arcade" => (vec!["arcade"], ".arcade"),
        "arduino" => (vec!["arduino", "ino"], ".ino"),
        "armasm" => (vec!["armasm", "arm"], ".s"),
        "asciidoc" => (vec!["asciidoc", "adoc"], ".adoc"),
        "aspectj" => (vec!["aspectj"], ".aj"),
        "autohotkey" => (vec!["autohotkey"], ".ahk"),
        "autoit" => (vec!["autoit"], ".au3"),
        "avrasm" => (vec!["avrasm"], ".asm"),
        "awk" => (vec!["awk", "mawk", "nawk", "gawk"], ".awk"),

        // B
        "bash" => (vec!["bash", "sh", "zsh"], ".sh"),
        "basic" => (vec!["basic"], ".bas"),
        "bnf" => (vec!["bnf"], ".bnf"),
        "brainfuck" => (vec!["brainfuck", "bf"], ".bf"),

        // C
        "c" => (vec!["c", "h"], ".c"),
        "cal" => (vec!["cal"], ".cal"),
        "capnproto" => (vec!["capnproto", "capnp"], ".capnp"),
        "ceylon" => (vec!["ceylon"], ".ceylon"),
        "clean" => (vec!["clean", "icl", "dcl"], ".icl"),
        "clojure" => (vec!["clojure", "clj"], ".clj"),
        "clojurerepl" => (vec!["clojure-repl"], ".clj"),
        "cmake" => (vec!["cmake", "cmake.in"], ".cmake"),
        "coffeescript" => (vec!["coffeescript", "coffee", "cson", "iced"], ".coffee"),
        "coq" => (vec!["coq"], ".v"),
        "cos" => (vec!["cos", "cls"], ".cls"),
        "cpp" => (
            vec!["cpp", "hpp", "cc", "hh", "c++", "h++", "cxx", "hxx"],
            ".cpp",
        ),
        "crmsh" => (vec!["crmsh", "crm", "pcmk"], ".crmsh"),
        "crystal" => (vec!["crystal", "cr"], ".cr"),
        "csharp" => (vec!["csharp", "cs"], ".cs"),
        "csp" => (vec!["csp"], ".csp"),
        "css" => (vec!["css"], ".css"),

        // D
        "d" => (vec!["d"], ".d"),
        "dart" => (vec!["dart"], ".dart"),
        "delphi" => (vec!["delphi", "dpr", "dfm", "pas", "pascal"], ".pas"),
        "diff" => (vec!["diff", "patch"], ".diff"),
        "django" => (vec!["django", "jinja"], ".html"),
        "dns" => (vec!["dns", "zone", "bind"], ".zone"),
        "dockerfile" => (vec!["dockerfile", "docker"], ".dockerfile"),
        "dos" => (vec!["dos", "bat", "cmd"], ".bat"),
        "dsconfig" => (vec!["dsconfig"], ".dsconfig"),
        "dts" => (vec!["dts"], ".dts"),
        "dust" => (vec!["dust", "dst"], ".dust"),

        // E
        "ebnf" => (vec!["ebnf"], ".ebnf"),
        "elixir" => (vec!["elixir"], ".ex"),
        "elm" => (vec!["elm"], ".elm"),
        "erb" => (vec!["erb"], ".erb"),
        "erlang" => (vec!["erlang", "erl"], ".erl"),
        "erlang-repl" => (vec!["erlang-repl"], ".erl"),
        "excel" => (vec!["excel", "xls", "xlsx"], ".xlsx"),

        // F
        "fix" => (vec!["fix"], ".fix"),
        "flix" => (vec!["flix"], ".flix"),
        "fortran" => (vec!["fortran", "f90", "f95"], ".f90"),
        "fsharp" => (vec!["fsharp", "fs", "fsx", "fsi", "fsscript"], ".fs"),

        // G
        "gams" => (vec!["gams", "gms"], ".gms"),
        "gauss" => (vec!["gauss", "gss"], ".gss"),
        "gcode" => (vec!["gcode", "nc"], ".gcode"),
        "gherkin" => (vec!["gherkin"], ".feature"),
        "glsl" => (vec!["glsl"], ".glsl"),
        "gml" => (vec!["gml"], ".gml"),
        "go" => (vec!["go", "golang"], ".go"),
        "golo" => (vec!["golo", "gololang"], ".golo"),
        "gradle" => (vec!["gradle"], ".gradle"),
        "graphql" => (vec!["graphql", "gql"], ".graphql"),
        "groovy" => (vec!["groovy"], ".groovy"),

        // H
        "haml" => (vec!["haml"], ".haml"),
        "handlebars" => (
            vec!["handlebars", "hbs", "html.hbs", "html.handlebars"],
            ".hbs",
        ),
        "haskell" => (vec!["haskell", "hs"], ".hs"),
        "haxe" => (vec!["haxe", "hx"], ".hx"),
        "hsp" => (vec!["hsp"], ".hsp"),
        "html" => (vec!["html", "xhtml"], ".html"),
        "http" => (vec!["http", "https"], ".http"),
        "hy" => (vec!["hy", "hylang"], ".hy"),

        // I
        "inform7" => (vec!["inform7", "i7"], ".ni"),
        "ini" => (vec!["ini", "toml"], ".ini"),
        "irpf90" => (vec!["irpf90"], ".irpf90"),
        "isbl" => (vec!["isbl"], ".isbl"),

        // J
        "java" => (vec!["java", "jsp"], ".java"),
        "javascript" => (vec!["javascript", "js", "jsx"], ".js"),
        "jbosscli" => (vec!["jboss-cli", "wildfly-cli"], ".cli"),
        "json" => (vec!["json", "jsonc", "json5"], ".json"),
        "julia" => (vec!["julia", "julia-repl"], ".jl"),

        // K
        "kotlin" => (vec!["kotlin", "kt"], ".kt"),

        // L
        "lasso" => (vec!["lasso", "ls", "lassoscript"], ".lasso"),
        "latex" => (vec!["tex"], ".tex"),
        "ldif" => (vec!["ldif"], ".ldif"),
        "leaf" => (vec!["leaf"], ".leaf"),
        "less" => (vec!["less"], ".less"),
        "lisp" => (vec!["lisp"], ".lisp"),
        "livecodeserver" => (vec!["livecodeserver"], ".livecodescript"),
        "livescript" => (vec!["livescript", "ls"], ".ls"),
        "llvm" => (vec!["llvm"], ".ll"),
        "lsl" => (vec!["lsl"], ".lsl"),
        "lua" => (vec!["lua", "pluto"], ".lua"),

        // M
        "makefile" => (vec!["makefile", "mk", "mak", "make"], "Makefile"),
        "markdown" => (vec!["markdown", "md", "mkdown", "mkd"], ".md"),
        "mathematica" => (vec!["mathematica", "mma", "wl"], ".m"),
        "matlab" => (vec!["matlab"], ".m"),
        "maxima" => (vec!["maxima"], ".mac"),
        "mel" => (vec!["mel"], ".mel"),
        "mercury" => (vec!["mercury"], ".m"),
        "mipsasm" => (vec!["mipsasm", "mips"], ".s"),
        "mizar" => (vec!["mizar"], ".miz"),
        "mojolicious" => (vec!["mojolicious"], ".pm"),
        "monkey" => (vec!["monkey"], ".monkey"),
        "moonscript" => (vec!["moonscript", "moon"], ".moon"),

        // N
        "n1ql" => (vec!["n1ql"], ".n1ql"),
        "nestedtext" => (vec!["nestedtext", "nt"], ".nt"),
        "nginx" => (vec!["nginx", "nginxconf"], ".conf"),
        "nim" => (vec!["nim", "nimrod"], ".nim"),
        "nix" => (vec!["nix"], ".nix"),
        "node-repl" => (vec!["node-repl"], ".js"),
        "nsis" => (vec!["nsis"], ".nsi"),

        // O
        "objectivec" => (vec!["objectivec", "mm", "objc", "obj-c"], ".m"),
        "ocaml" => (vec!["ocaml", "ml"], ".ml"),
        "openscad" => (vec!["openscad", "scad"], ".scad"),
        "oxygene" => (vec!["oxygene"], ".pas"),

        // P
        "parser3" => (vec!["parser3"], ".p"),
        "perl" => (vec!["perl", "pl", "pm"], ".pl"),
        "pf" => (vec!["pf", "pf.conf"], ".conf"),
        "pgsql" => (vec!["pgsql", "postgres", "postgresql"], ".sql"),
        "php" => (vec!["php"], ".php"),
        "phptemplate" => (vec!["php-template"], ".php"),
        "plaintext" => (vec!["plaintext", "txt", "text"], ".txt"),
        "pony" => (vec!["pony"], ".pony"),
        "powershell" => (vec!["powershell", "ps", "ps1"], ".ps1"),
        "processing" => (vec!["processing"], ".pde"),
        "profile" => (vec!["profile"], ".profile"),
        "prolog" => (vec!["prolog"], ".pl"),
        "properties" => (vec!["properties"], ".properties"),
        "protobuf" => (vec!["protobuf"], ".proto"),
        "puppet" => (vec!["puppet", "pp"], ".pp"),
        "purebasic" => (vec!["purebasic", "pb", "pbi"], ".pb"),
        "python" => (vec!["python", "py", "gyp"], ".py"),
        "pythonrepl" => (vec!["python-repl", "pycon"], ".py"),

        // Q
        "q" => (vec!["k", "kdb"], ".q"),
        "qml" => (vec!["qml"], ".qml"),

        // R
        "r" => (vec!["r"], ".r"),
        "reasonml" => (vec!["reasonml", "re"], ".re"),
        "rib" => (vec!["rib"], ".rib"),
        "roboconf" => (vec!["graph", "instances"], ".graph"),
        "routeros" => (vec!["routeros", "mikrotik"], ".rsc"),
        "rsl" => (vec!["rsl"], ".rsl"),
        "ruby" => (
            vec!["ruby", "rb", "gemspec", "podspec", "thor", "irb"],
            ".rb",
        ),
        "ruleslanguage" => (vec!["ruleslanguage"], ".rule"),
        "rust" => (vec!["rust", "rs"], ".rs"),

        // S
        "sas" => (vec!["sas"], ".sas"),
        "scala" => (vec!["scala"], ".scala"),
        "scheme" => (vec!["scheme"], ".scm"),
        "scilab" => (vec!["scilab", "sci"], ".sci"),
        "scss" => (vec!["scss"], ".scss"),
        "shell" => (vec!["shell", "console"], ".sh"),
        "smali" => (vec!["smali"], ".smali"),
        "smalltalk" => (vec!["smalltalk", "st"], ".st"),
        "sml" => (vec!["sml", "ml"], ".sml"),
        "solidity" => (vec!["solidity", "sol"], ".sol"),
        "sqf" => (vec!["sqf"], ".sqf"),
        "sql" => (vec!["sql"], ".sql"),
        "stan" => (vec!["stan", "stanfuncs"], ".stan"),
        "stata" => (vec!["stata"], ".do"),
        "step21" => (vec!["step21", "p21", "step", "stp"], ".stp"),
        "stylus" => (vec!["stylus", "styl"], ".styl"),
        "subunit" => (vec!["subunit"], ".subunit"),
        "swift" => (vec!["swift"], ".swift"),

        // T
        "taggerscript" => (vec!["taggerscript"], ".tagger"),
        "tap" => (vec!["tap"], ".t"),
        "tcl" => (vec!["tcl", "tk"], ".tcl"),
        "thrift" => (vec!["thrift"], ".thrift"),
        "toml" => (vec!["toml"], ".toml"),
        "tp" => (vec!["tp"], ".tp"),
        "twig" => (vec!["twig", "craftcms"], ".twig"),
        "typescript" => (vec!["typescript", "ts", "tsx", "mts", "cts"], ".ts"),

        // V
        "vala" => (vec!["vala"], ".vala"),
        "vbnet" => (vec!["vbnet", "vb"], ".vb"),
        "vbscript" => (vec!["vbscript", "vbs"], ".vbs"),
        "vbscripthtmlvbscript" => (vec!["vbscript-html"], ".vbs"),
        "verilog" => (vec!["verilog", "v"], ".v"),
        "vhdl" => (vec!["vhdl"], ".vhd"),
        "vim" => (vec!["vim"], ".vim"),
        "wasm" => (vec!["wasm"], ".wat"),
        "wren" => (vec!["wren"], ".wren"),

        // X
        "x86asm" => (vec!["x86asm"], ".asm"),
        "xl" => (vec!["xl", "tao"], ".xl"),
        "xml" => (
            vec!["xml", "rss", "atom", "xjb", "xsd", "xsl", "plist", "svg"],
            ".xml",
        ),
        "xquery" => (vec!["xquery", "xpath", "xq"], ".xq"),

        // Y
        "yaml" => (vec!["yaml", "yml"], ".yaml"),

        // Z
        "zephir" => (vec!["zephir", "zep"], ".zep"),
        "zig" => (vec!["zig"], ".zig"),

        // Default: use the language name itself
        _ => {
            return LanguageMetadata {
                fence_markers: vec![lang_name.to_string()],
                file_extension: Cow::Owned(format!(".{}", lang_name)),
            }
        }
    };

    LanguageMetadata {
        fence_markers: markers.iter().map(|s| s.to_string()).collect(),
        file_extension: Cow::Borrowed(ext),
    }
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
///
/// # Display
///
/// The language can be formatted for display using the `Display` trait, which
/// combines the base language and variant (if present) into a string like "c" or "c-parasol".
pub struct ConfiguredLanguage {
    base_language: String,
    variant: Option<String>,
    config: LanguageConfig,
    file_extension: String,
}

impl fmt::Display for ConfiguredLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base_language)?;
        if let Some(ref v) = self.variant {
            write!(f, "-{}", v)?;
        }
        Ok(())
    }
}

impl ConfiguredLanguage {
    pub fn new(base_language: String, variant: Option<String>, config: LanguageConfig) -> Self {
        // Get metadata for the base language to determine file extension
        let metadata = get_language_metadata(&base_language);
        let file_extension = metadata.file_extension.into_owned();

        Self {
            base_language,
            variant,
            config,
            file_extension,
        }
    }

    /// Returns the file extension for this language (e.g., ".c", ".ts").
    pub fn file_extension(&self) -> &str {
        &self.file_extension
    }

    /// Writes source code with optional preamble to a temporary file.
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to write
    /// * `temp_file` - Path where the code should be written
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written.
    async fn write_source_file(&self, code: &str, temp_file: &Path) -> Result<()> {
        let mut file = File::create(temp_file)
            .await
            .with_context(|| format!("Failed to create temporary file: {}", temp_file.display()))?;

        if let Some(ref preamble) = self.config.preamble {
            file.write_all(preamble.as_bytes()).await?;
            file.write_all(b"\n\n").await?;
        }

        file.write_all(code.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    /// Compiles or validates the given code asynchronously.
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
    pub async fn compile(&self, code: &str, temp_file: &Path) -> Result<()> {
        // Write code with optional preamble to temp file
        self.write_source_file(code, temp_file).await?;

        // Execute compiler with configured flags
        let output = Command::new(&self.config.compiler)
            .args(&self.config.flags)
            .arg(temp_file)
            .output()
            .await
            .with_context(|| {
                format!(
                    "Failed to execute compiler '{}' for language '{}'\nFlags: {:?}\nFile: {}",
                    self.config.compiler,
                    self,
                    self.config.flags,
                    temp_file.display()
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
            anyhow::bail!(
                "{} compilation failed\nCompiler: {}\nFlags: {:?}\nFile: {}\n\n{}",
                self,
                self.config.compiler,
                self.config.flags,
                temp_file.display(),
                error_msg
            );
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
    ///     println!("Found language: {}", lang);
    /// }
    ///
    /// // Parasol variant of C
    /// if let Some(lang) = registry.find_by_fence("c", Some("parasol")) {
    ///     println!("Found language: {}", lang);
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

                return Some(ConfiguredLanguage::new(
                    lang_name.clone(),
                    None,
                    resolved_config,
                ));
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

        // Create a new language with the base language and variant
        Some(ConfiguredLanguage::new(
            lang_name.clone(),
            Some(variant_name.to_string()),
            merged_config,
        ))
    }
}
