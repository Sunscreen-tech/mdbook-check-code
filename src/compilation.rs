use crate::language::ConfiguredLanguage;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A compilation task representing a single code block to be compiled.
///
/// This struct contains all the information needed to independently compile
/// a code block in parallel without requiring shared state.
pub struct CompilationTask {
    language: ConfiguredLanguage,
    temp_path: PathBuf,
    chapter_path: Arc<PathBuf>,
    block_index: usize,
    code: String,
}

impl CompilationTask {
    pub fn new(
        language: ConfiguredLanguage,
        temp_path: PathBuf,
        chapter_path: Arc<PathBuf>,
        block_index: usize,
        code: String,
    ) -> Self {
        Self {
            language,
            temp_path,
            chapter_path,
            block_index,
            code,
        }
    }

    /// Executes compilation and consumes the task to produce a result.
    ///
    /// This method performs the actual compilation, measures duration,
    /// and converts any errors into the appropriate result format.
    pub fn compile(self) -> CompilationResult {
        log::debug!("Compiling {} block", self.language.name());

        let start = Instant::now();
        let compile_result = self.language.compile(&self.code, &self.temp_path);
        let duration = start.elapsed();

        CompilationResult {
            language: self.language,
            duration,
            chapter_path: self.chapter_path,
            block_index: self.block_index,
            code: self.code,
            error_message: compile_result.err().map(|e| e.to_string()),
        }
    }
}

/// Result of compiling a single code block.
///
/// This struct captures all compilation outcomes (success or failure)
/// along with timing information for statistics and trace logging.
pub struct CompilationResult {
    language: ConfiguredLanguage,
    duration: Duration,
    chapter_path: Arc<PathBuf>,
    block_index: usize,
    code: String,
    error_message: Option<String>,
}

impl CompilationResult {
    /// Returns true if compilation succeeded (no error message).
    pub fn success(&self) -> bool {
        self.error_message.is_none()
    }

    pub fn language(&self) -> &ConfiguredLanguage {
        &self.language
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn chapter_path(&self) -> &Path {
        &self.chapter_path
    }

    pub fn block_index(&self) -> usize {
        self.block_index
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

/// Compiles all tasks in parallel using the provided thread pool.
///
/// Returns a tuple of (results, total_parallel_duration).
pub fn compile_tasks(
    tasks: Vec<CompilationTask>,
    thread_pool: &rayon::ThreadPool,
) -> (Vec<CompilationResult>, Duration) {
    let parallel_start = Instant::now();
    let results: Vec<CompilationResult> = thread_pool.install(|| {
        tasks
            .into_par_iter()
            .map(CompilationTask::compile)
            .collect()
    });
    let parallel_duration = parallel_start.elapsed();

    (results, parallel_duration)
}
