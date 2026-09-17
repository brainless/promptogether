pub mod gallery;
pub mod gallery_seed;
pub mod jobs;
pub mod jobs_lifecycle;
pub mod jobs_repo;
pub mod jobs_worker;

pub fn app_name() -> &'static str {
    "promptogether"
}

pub fn app_version() -> &'static str {
    "0.1.0"
}
